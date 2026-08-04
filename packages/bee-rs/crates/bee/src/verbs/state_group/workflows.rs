// `state workflows …` and `state route`
//
// Split out of the single 6.1k-line verbs/state_group.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{
    append_jsonl, ensure_dir, read_json, warn_corrupt_json, write_json_atomic, ReadJson,
};
use crate::jsjson;
use crate::lock::{self, AcquireOnce, LockGuard};
use crate::verbs::reservations::{
    date_parse_val, finish, iso_from_ms, jget, js_disp, js_disp_opt,
    js_numberify, js_trim, keys_known, now_iso, now_ms, parse_flags, prelude, truthy,
    Ctx, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::reservations::{list_reservations, paths_overlap, rebuild_reservations_projection};
use crate::verbs::workspace_store as ws;
use crate::verbs::workflow_store::{
    acquire_named_lock, acquire_workflow_lock, adopt_mailbox_handoff, create_workflow,
    find_live_workflow, NewWorkflow,
    gates_patch_from_record, lane_lock_name, lane_path, list_lanes, list_workflows,
    newest_open_handoff_mailbox_record, projection_lock_name, read_lane_display, read_lane_strict,
    rebuild_handoff_projection, rebuild_handoff_projection_reporting, rebuild_lane_projection,
    rebuild_lane_projection_reporting, rebuild_state_projection,
    rebuild_state_projection_reporting, update_workflow, update_workflow_assuming_lock,
    update_workflow_assuming_lock_with, wf_id, workflows_list_sort, write_lane,
    write_mailbox_handoff, MailboxAdopt,
};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

// ─── state workflows list / close (workflow-lifecycle wl-2) ────────────────

pub(crate) fn run_workflows_list(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match go("state workflows list", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let mut records = list_workflows(&ctx.root)?;
        workflows_list_sort(&mut records)?;
        let text = if records.is_empty() {
            "No workflow records.".to_string()
        } else {
            records
                .iter()
                .map(|r| {
                    format!(
                        "{} feature={} status={} phase={} created_at={}",
                        js_disp_opt(r.get("id")),
                        js_disp_opt(r.get("feature")),
                        js_disp_opt(r.get("status")),
                        js_disp_opt(r.get("phase")),
                        js_disp_opt(r.get("created_at")),
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let result = Value::Array(records.into_iter().map(Value::Object).collect());
        Ok(Out::Emit(result, text, 0))
    })();
    finish(&ctx, out)
}

/// bee.mjs resolveActiveFeatureForWorkflowsClose (review-p1-fixes p1-3, F5):
/// `Ok(feature)` on a real resolution (None is a definite "idle" answer),
/// `Err(reason)` when resolution ITSELF failed.
pub(crate) fn resolve_active_feature_for_workflows_close(
    root: &Path,
) -> Ex<Result<Option<String>, String>> {
    match resolve_mutation_target(root, None, "workflows close", false) {
        Ok(t) => Ok(Ok(match t.record().get("feature") {
            Some(v) if truthy(v) => Some(js_disp(v)),
            _ => None,
        })),
        Err(Err2::Msg(m)) => Ok(Err(m)),
        Err(Err2::Ex) => Err(Exotic),
    }
}

/// The shared tail of both unresolved-active refusals (F5).
pub(crate) fn workflows_close_unresolved_active_tail(reason: &str) -> String {
    format!(
        "Underlying resolution failure: {reason}\nA guard that cannot establish its precondition refuses \u{2014} it never proceeds on a null active feature.\nFIX: repair the routing record named above (restore .bee/state.json, start or unbind the session-bound lane), then retry \u{2014} or close one record explicitly with `bee state workflows close --id <id>`, the one mode that never consults the active feature."
    )
}

pub(crate) fn closed_row(record: &Map<String, Value>) -> Value {
    let mut row = Map::new();
    row.insert("id".into(), record.get("id").cloned().unwrap_or(Value::Null));
    row.insert(
        "feature".into(),
        record.get("feature").cloned().unwrap_or(Value::Null),
    );
    Value::Object(row)
}

pub(crate) fn run_workflows_close(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature", "id", "all-but-active"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "all-but-active") {
        return None;
    }
    let ctx = match go("state workflows close", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        let has_feature = matches!(flags.get("feature"), Some(FlagV::S(s)) if !s.is_empty());
        let has_id = matches!(flags.get("id"), Some(FlagV::S(s)) if !s.is_empty());
        let has_all_but_active = matches!(flags.get("all-but-active"), Some(FlagV::Present));
        let mode_count =
            [has_feature, has_id, has_all_but_active].iter().filter(|b| **b).count();
        if mode_count != 1 {
            return Ok(Out::Thrown(format!(
                "workflows close: requires exactly one of --feature <feature>, --id <id>, or --all-but-active. Example: {EXAMPLE_WORKFLOWS_CLOSE}"
            )));
        }
        // F5: computed for every mode, CONSULTED only by the two it protects.
        let active = resolve_active_feature_for_workflows_close(&ctx.root)?;
        let active_feature = match &active {
            Ok(f) => f.clone(),
            Err(_) => None,
        };

        if has_id {
            let id = flag_string(&flags, "id").unwrap_or_default();
            let records = list_workflows(&ctx.root)?;
            let live = records.iter().find(|r| {
                r.get("id").unwrap_or(&Value::Null) == &Value::String(id.clone())
                    && r.get("status").unwrap_or(&Value::Null) != &json!("closed")
            });
            if live.is_none() {
                return Ok(Out::Thrown(format!(
                    "workflows close --id: no live workflow record found with id \"{id}\"."
                )));
            }
            let mut patch = Map::new();
            patch.insert("status".into(), json!("closed"));
            let closed = update_workflow(&ctx.root, &id, patch)?;
            let text = format!(
                "Closed 1 workflow record: {} (feature \"{}\").",
                js_disp_opt(closed.get("id")),
                js_disp_opt(closed.get("feature"))
            );
            let mut result = Map::new();
            result.insert("closed".into(), Value::Array(vec![closed_row(&closed)]));
            return Ok(Out::Emit(Value::Object(result), text, 0));
        }

        if has_feature {
            let feature = flag_string(&flags, "feature").unwrap_or_default();
            let Ok(_) = &active else {
                let reason = active.unwrap_err();
                return Ok(Out::Thrown(format!(
                    "workflows close --feature: refused \u{2014} the currently active feature could not be resolved, so the guard that keeps \"{feature}\" from being closed while it IS the active feature cannot be evaluated. Nothing was closed.\n{}",
                    workflows_close_unresolved_active_tail(&reason)
                )));
            };
            if active_feature.as_deref() == Some(feature.as_str()) {
                return Ok(Out::Thrown(format!(
                    "workflows close --feature: refused \u{2014} \"{feature}\" is the currently active feature; use --id <id> to close its record explicitly."
                )));
            }
            let records = list_workflows(&ctx.root)?;
            let matches: Vec<String> = records
                .iter()
                .filter(|r| {
                    r.get("feature").unwrap_or(&Value::Null) == &Value::String(feature.clone())
                        && r.get("status").unwrap_or(&Value::Null) != &json!("closed")
                })
                .map(wf_id)
                .collect();
            if matches.is_empty() {
                return Ok(Out::Thrown(format!(
                    "workflows close --feature: no live workflow record found for feature \"{feature}\"."
                )));
            }
            let mut closed: Vec<Map<String, Value>> = Vec::new();
            for id in matches {
                let mut patch = Map::new();
                patch.insert("status".into(), json!("closed"));
                closed.push(update_workflow(&ctx.root, &id, patch)?);
            }
            let ids: Vec<String> = closed.iter().map(|r| js_disp_opt(r.get("id"))).collect();
            let text = format!(
                "Closed {} workflow record(s) for feature \"{feature}\": {}.",
                closed.len(),
                ids.join(", ")
            );
            let mut result = Map::new();
            result.insert(
                "closed".into(),
                Value::Array(closed.iter().map(closed_row).collect()),
            );
            return Ok(Out::Emit(Value::Object(result), text, 0));
        }

        // --all-but-active
        let Ok(_) = &active else {
            let reason = active.unwrap_err();
            return Ok(Out::Thrown(format!(
                "workflows close --all-but-active: refused \u{2014} the currently active feature could not be resolved, so \"all but active\" would silently degrade into \"all\": every live workflow record, in-flight work included, would be closed. Nothing was closed.\n{}",
                workflows_close_unresolved_active_tail(&reason)
            )));
        };
        // state.mjs closeWorkflowsForFeature({keepFeature}).
        let keep = active_feature
            .as_deref()
            .map(js_trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let records = list_workflows(&ctx.root)?;
        let mut closed: Vec<Value> = Vec::new();
        for record in &records {
            if record.get("status").unwrap_or(&Value::Null) == &json!("closed") {
                continue;
            }
            if let Some(keep) = &keep {
                if record.get("feature").unwrap_or(&Value::Null) == &Value::String(keep.clone()) {
                    continue;
                }
            }
            let id = wf_id(record);
            let mut patch = Map::new();
            patch.insert("status".into(), json!("closed"));
            update_workflow(&ctx.root, &id, patch)?;
            closed.push(closed_row(record));
        }
        if closed.is_empty() {
            return Ok(Out::Thrown(
                "workflows close --all-but-active: nothing to close \u{2014} no live workflow record other than the active feature.".to_string(),
            ));
        }
        let ids: Vec<String> = closed
            .iter()
            .map(|r| js_disp_opt(jget(r, "id")))
            .collect();
        let text = format!(
            "Closed {} workflow record(s), kept active feature \"{}\": {}.",
            closed.len(),
            active_feature.as_deref().unwrap_or("(none)"),
            ids.join(", ")
        );
        let mut result = Map::new();
        result.insert("closed".into(), Value::Array(closed));
        Ok(Out::Emit(Value::Object(result), text, 0))
    })();
    finish(&ctx, out)
}

// ─── state route (R6 coverage debt) ────────────────────────────────────────
//
// Provenance: bee.mjs handleStateRoute / validateRouteSetFlags /
// resolveRouteTarget / buildRouteWorktreeBlock / isCodeTouchingLane /
// otherLiveWorkPresent / rejectDryRun, lib/worktree-store.mjs
// findGrantedWorktreeForFeature, lib/claims.mjs activeWorkers.
//
// `--show` is a pure read through resolveMutationTarget. `--set` runs the
// same withMutationLock seam every other record-mutating state verb uses,
// plus the D1 `route` patch onto the live workflow record
// (updateWorkflowAssumingLock, non-self-deadlocking because the workflow lock
// is already held).
//
// ONE branch of `--set` stays with Node, and it is decided BEFORE any lock or
// write: buildRouteWorktreeBlock's `findGrantedWorktreeForFeature` walk needs
// worktree-store.mjs's bidirectional gitdir resolution, which lives in
// verbs/status_full.rs and is not reachable from here. The block can only be
// non-null when the recorded lane is code-touching AND the feature holds no
// granted worktree — so a repo whose grants registry has ZERO `true` entries
// can never have one, and the native path serves it exactly. The moment any
// grant is `true` AND the lane is code-touching, the whole verb returns None
// before the lock. (`--show`, and every non-code-touching lane, are native in
// every repo.)

pub(crate) const ROUTE_CLASS_VALUES: [&str; 7] =
    ["feature", "bugfix", "docs", "refactor", "research", "release", "spike"];
const ROUTE_LANE_VALUES: [&str; 6] =
    ["docs", "tiny", "small", "spike", "standard", "high-risk"];
const ROUTE_FLAG_VALUES: [&str; 10] = [
    "auth",
    "authorization",
    "data-model",
    "audit-security",
    "external-systems",
    "public-contracts",
    "cross-platform",
    "covered-contract-change",
    "proof-weakening",
    "multi-domain",
];

pub(crate) const EXAMPLE_ROUTE: &str =
    "bee state route --set --class feature --lane standard --flags multi-domain --files 7 --json";

/// `Number(v)` for a flag STRING — the full ToNumber conversion, not
/// reservations.rs's `js_number_flag` (which is Number.parseInt and would read
/// "2.5" as 2). `Ok(None)` is NaN (Node then reports the invalid-flag refusal);

/// `Err` is a spelling JS converts but this port does not model (hex/binary/
/// octal literals, Infinity). CUTOVER: `|n| >= 1e21` used to be on that list
/// because jsjson printed such a number differently than V8; jsjson now
/// implements the full ECMA Number::toString, so a large FINITE value is
/// simply accepted.
pub(crate) fn js_number_full(raw: &str) -> Ex<Option<f64>> {
    let t = js_trim(raw);
    if t.is_empty() {
        return Ok(Some(0.0)); // Number("") === Number("   ") === 0
    }
    let lower = t.to_ascii_lowercase();
    if lower.starts_with("0x")
        || lower.starts_with("0b")
        || lower.starts_with("0o")
        || lower.contains("infinity")
    {
        return Err(Exotic);
    }
    // grammar: [+-]? ( digits [ '.' digits? ] | '.' digits ) ( [eE][+-]?digits )?
    let bytes = t.as_bytes();
    let mut i = 0usize;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let int_len = i - int_start;
    let mut frac_len = 0usize;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let fs = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        frac_len = i - fs;
    }
    if int_len == 0 && frac_len == 0 {
        return Ok(None); // not numeric at all → NaN
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        let es = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return Ok(None);
        }
    }
    if i != bytes.len() {
        return Ok(None); // trailing junk → NaN
    }
    let num: f64 = t.parse().map_err(|_| Exotic)?;
    if !num.is_finite() {
        return Err(Exotic); // Number("1e400") === Infinity — unmodeled downstream
    }
    Ok(Some(num))
}

/// bee.mjs validateRouteSetFlags — the hand-rolled batch validation (`--flags
/// ""` is the valid spelling of "zero flags", which requireFlags would call
/// missing). Every refusal is deterministic.
pub(crate) fn validate_route_set_flags(flags: &Flags) -> Ex<Result<Map<String, Value>, String>> {
    let mut missing: Vec<&str> = Vec::new();
    let mut invalid: Vec<String> = Vec::new();

    // `x === undefined || x === '' || x === true` — Present IS JS `true`.
    let named = |key: &str| -> Option<String> {
        match flags.get(key) {
            Some(FlagV::S(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        }
    };
    let class_value = named("class");
    match &class_value {
        None => missing.push("--class"),
        Some(v) if !ROUTE_CLASS_VALUES.contains(&v.as_str()) => invalid.push(format!(
            "--class \"{v}\" (must be one of {})",
            ROUTE_CLASS_VALUES.join(", ")
        )),
        Some(_) => {}
    }
    let lane_value = named("lane");
    match &lane_value {
        None => missing.push("--lane"),
        Some(v) if !ROUTE_LANE_VALUES.contains(&v.as_str()) => invalid.push(format!(
            "--lane \"{v}\" (must be one of {})",
            ROUTE_LANE_VALUES.join(", ")
        )),
        Some(_) => {}
    }
    // `flags.flags === undefined || flags.flags === true` — '' is legal here.
    let mut flag_names: Vec<String> = Vec::new();
    match flags.get("flags") {
        None | Some(FlagV::Present) => missing.push("--flags"),
        Some(FlagV::S(raw)) => {
            flag_names = split_list(raw);
            let bad: Vec<&String> = flag_names
                .iter()
                .filter(|f| !ROUTE_FLAG_VALUES.contains(&f.as_str()))
                .collect();
            if !bad.is_empty() {
                invalid.push(format!(
                    "--flags \"{raw}\" names invalid flag(s) {} (legal set: {})",
                    bad.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    ROUTE_FLAG_VALUES.join(", ")
                ));
            }
        }
    }
    let mut product_files: Option<f64> = None;
    match named("files") {
        None => missing.push("--files"),
        Some(raw) => {
            // `Number(flags.files)`, then Number.isInteger(n) && n >= 0.
            let n = js_number_full(&raw)?;
            match n {
                Some(n) if n.is_finite() && n.fract() == 0.0 && n >= 0.0 => {
                    product_files = Some(n);
                }
                _ => invalid.push(format!("--files \"{raw}\" (must be a non-negative integer)")),
            }
        }
    }

    if !missing.is_empty() || !invalid.is_empty() {
        let mut parts: Vec<String> = Vec::new();
        if !missing.is_empty() {
            parts.push(format!("missing required flag(s): {}", missing.join(", ")));
        }
        if !invalid.is_empty() {
            parts.push(format!("invalid flag(s): {}", invalid.join("; ")));
        }
        return Ok(Err(format!(
            "route --set: {}. Example: {EXAMPLE_ROUTE}",
            parts.join("; ")
        )));
    }

    let mut route = Map::new();
    route.insert("class".into(), json!(class_value.unwrap()));
    route.insert("lane".into(), json!(lane_value.unwrap()));
    route.insert(
        "flags".into(),
        Value::Array(flag_names.into_iter().map(Value::String).collect()),
    );
    route.insert(
        "product_files".into(),
        Value::Number(serde_json::Number::from_f64(product_files.unwrap()).ok_or(Exotic)?),
    );
    // `flags.rationale !== undefined && flags.rationale !== true ? String(...) : null`
    route.insert(
        "rationale".into(),
        match flags.get("rationale") {
            Some(FlagV::S(s)) => json!(s),
            _ => Value::Null,
        },
    );
    route.insert("updated_at".into(), json!(now_iso()));
    Ok(Ok(route))
}

/// bee.mjs isCodeTouchingLane.
pub(crate) fn is_code_touching_lane(lane: &str, other_live_session: bool) -> bool {
    if lane.is_empty() || lane == "docs" {
        return false;
    }
    if lane == "tiny" && !other_live_session {
        return false;
    }
    true
}

/// bee.mjs otherLiveWorkPresent (D9a). activeWorkers' `cell` join is never
/// read here — only `lane` is — so the claims scan it performs contributes
/// nothing and is skipped: one row per OTHER live session is the whole input.
/// Fails quiet to `false`, exactly like the .mjs's own try/catch.
pub(crate) fn other_live_work_present(root: &Path) -> Ex<bool> {
    let self_id = resolve_session_id(None, root)?;
    let now = now_ms();
    let exclude = self_id.as_deref().unwrap_or("");
    for session in list_session_records(root)? {
        if matches!(session.get("id"), Some(Value::String(s)) if s == exclude) {
            continue;
        }
        if heartbeat_stale(&session, now)? {
            continue;
        }
        let lane = match session.get("lane") {
            Some(Value::String(l)) if !l.is_empty() => Some(l.clone()),
            _ => None,
        };
        let phase = match &lane {
            Some(l) => read_lane_display(root, l)?.and_then(|r| r.get("phase").cloned()),
            None => read_state_peek(root)?.get("phase").cloned(),
        };
        let live = match phase {
            Some(p) => truthy(&p) && &p != &json!("idle") && &p != &json!("compounding-complete"),
            None => false,
        };
        if live {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `Object.values(readGrants(...)).some((v) => v === true)` — the pre-lock
/// gate that decides whether findGrantedWorktreeForFeature could possibly
/// answer non-null (see this section's header).
pub(crate) fn any_granted_worktree(root: &Path) -> bool {
    match crate::roots::read_grants(&root.join(".bee")) {
        Value::Object(m) => m.values().any(|v| matches!(v, Value::Bool(true))),
        _ => false,
    }
}

/// bee.mjs buildRouteWorktreeBlock, for the ordinary-checkout arm the native
/// path always sits on, with `findGrantedWorktreeForFeature` proven null by
/// the caller's pre-lock gate.
pub(crate) fn route_worktree_block(feature: Option<&str>, lane: &str, code_touching: bool) -> Option<Value> {
    let feature = feature?;
    if !code_touching {
        return None;
    }
    let command = format!("bee worktree new --feature {feature}");
    let mut block = Map::new();
    block.insert("required".into(), Value::Bool(true));
    block.insert("command".into(), json!(command));
    block.insert("reason".into(), json!(format!(
        "lane \"{lane}\" is code-touching and this is the MAIN checkout \u{2014} feature work branches at feature start (worktree-first)"
    )));
    block.insert("notice".into(), json!(format!(
        "\u{26a0} WORKTREE-FIRST: lane \"{lane}\" is code-touching and this is the MAIN checkout. NEXT: run \"{command}\", then open your session at the printed worktree path \u{2014} main stays for integration, docs-lane, and release work; once the worktree is granted, main refuses feature source edits."
    )));
    Some(Value::Object(block))
}

pub(crate) fn run_route(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["set", "show", "class", "lane", "flags", "files", "rationale"]) {
        return None;
    }
    for b in ["set", "show"] {
        if !bool_flag_ok(&flags, b) {
            return None;
        }
    }
    let show = matches!(flags.get("show"), Some(FlagV::Present));
    let set = matches!(flags.get("set"), Some(FlagV::Present));
    let ctx = match go("state route", use_json, t0)? {
        Ok(c) => c,
        Err(code) => return Some(code),
    };
    let out = (|| -> R2<Out> {
        if show {
            let target = resolve_mutation_target(&ctx.root, None, "route show", false)?;
            // `target.record.route ?? null`, then `if (!route)`.
            let route = match target.record().get("route") {
                Some(v) if truthy(v) => v.clone(),
                _ => {
                    return Ok(Out::Emit(Value::Null, "No route recorded.".to_string(), 0));
                }
            };
            // `route.flags.length` / `route.flags.join(',')` throw on a
            // non-array; a stored route is always this shape.
            let Some(Value::Array(flag_names)) = jget(&route, "flags") else {
                return Err(Err2::Ex);
            };
            let joined: Vec<String> = flag_names.iter().map(js_disp).collect();
            let rationale = match jget(&route, "rationale") {
                Some(v) if truthy(v) => format!(" rationale=\"{}\"", js_disp(v)),
                _ => String::new(),
            };
            let text = format!(
                "class={} lane={} flags={} [{}] files={}{rationale}{}",
                js_disp_opt(jget(&route, "class")),
                js_disp_opt(jget(&route, "lane")),
                flag_names.len(),
                joined.join(","),
                js_disp_opt(jget(&route, "product_files")),
                target.lane_note(),
            );
            return Ok(Out::Emit(route, text, 0));
        }
        if !set {
            return Ok(Out::Thrown(
                "route: requires --set (to record a route) or --show (to read it back).".to_string(),
            ));
        }
        let route_object = match validate_route_set_flags(&flags)? {
            Ok(r) => r,
            Err(message) => return Ok(Out::Thrown(message)),
        };
        let lane_class = js_disp_opt(route_object.get("lane"));

        // ── the pre-lock delegation gate (see this section's header) ───────
        // otherLiveWorkPresent is only consulted for the tiny lane; probing it
        // here also front-loads its own delegation triggers.
        let other_live = if lane_class == "tiny" { other_live_work_present(&ctx.root)? } else { true };
        let code_touching = is_code_touching_lane(&lane_class, other_live);
        if code_touching && any_granted_worktree(&ctx.root) {
            return Err(Err2::Ex);
        }

        let scope = resolve_mutation_lock_scope(&ctx.root, None, false)?;
        let workflows = list_workflows(&ctx.root)?;
        let locks = acquire_mutation_locks(&ctx.root, &scope, &workflows)?;
        let mut target = resolve_mutation_target(&ctx.root, None, "route", false)?;
        let phase = target.record().get("phase").cloned().unwrap_or(Value::Null);
        let feature_set = target.record().get("feature").map(truthy).unwrap_or(false);
        if !feature_set
            || &phase == &json!("idle")
            || &phase == &json!("compounding-complete")
        {
            // `${phase ?? 'idle'}` / `${state.feature ?? 'none'}` — nullish.
            let phase_disp = match target.record().get("phase") {
                None | Some(Value::Null) => "idle".to_string(),
                Some(v) => js_disp(v),
            };
            let feature_disp = match target.record().get("feature") {
                None | Some(Value::Null) => "none".to_string(),
                Some(v) => js_disp(v),
            };
            return Ok(Out::Thrown(format!(
                "route --set: refused \u{2014} no active feature to attach a route to (phase \"{phase_disp}\", feature \"{feature_disp}\"). FIX: start a feature first (state start-feature), then record its route."
            )));
        }
        let lane_note = target.lane_note();
        let target_lane = target.lane().map(str::to_string);
        target
            .record_mut()
            .insert("route".into(), Value::Object(route_object.clone()));
        let record = target.record().clone();
        write_through_projection(&ctx.root, &target, &record, &[])?;
        // D1: also patch the live workflow record's own `route` field.
        let target_feature = match &target_lane {
            Some(l) => l.clone(),
            None => js_disp_opt(record.get("feature")),
        };
        let live = list_workflows(&ctx.root)?;
        if let Some(wf) = find_live_workflow(&live, &target_feature) {
            let mut patch = Map::new();
            patch.insert("route".into(), Value::Object(route_object.clone()));
            update_workflow_assuming_lock(&ctx.root, &wf_id(wf), patch)?;
        }
        // `state.feature ?? target.lane ?? null`
        let routed_feature = match record.get("feature") {
            Some(v) if !v.is_null() => Some(js_disp(v)),
            _ => target_lane.clone(),
        };
        drop(locks);

        let other_live = if lane_class == "tiny" { other_live_work_present(&ctx.root)? } else { true };
        let code_touching = is_code_touching_lane(&lane_class, other_live);
        let block = route_worktree_block(routed_feature.as_deref(), &lane_class, code_touching);

        let flags_len = match route_object.get("flags") {
            Some(Value::Array(a)) => a.len(),
            _ => 0,
        };
        let mut text = format!(
            "Recorded route (class={} lane={lane_class} flags={flags_len} files={}).{lane_note}",
            js_disp_opt(route_object.get("class")),
            js_disp_opt(route_object.get("product_files")),
        );
        let mut result = route_object;
        if let Some(block) = block {
            text.push('\n');
            text.push_str(&js_disp_opt(jget(&block, "notice")));
            result.insert("worktree".into(), block);
        }
        Ok(Out::Emit(Value::Object(result), text, 0))
    })();
    finish(&ctx, out)
}
