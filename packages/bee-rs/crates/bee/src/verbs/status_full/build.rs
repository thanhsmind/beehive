// buildStatus
//
// Split out of the single 7k-line verbs/status_full.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, LinkedRoots, RootsWt};
use crate::state::{bypass_level, read_config_raw};
use crate::verbs::workflow_store::{default_wf_gates, find_live_workflow, list_workflows};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use crate::version::BEE_VERSION;

/// D3/D7: the persisted gate RECORD (state/actor/at/reason/bypass_level per
/// gate), read from the live workflow that names the current feature — not
/// from the projected `approved_gates` booleans `status["gates"]` already
/// carries. A gate that was never touched reads `state: "pending"`; a gate
/// that was refused reads `state: "rejected"` — the two are indistinguishable
/// in the boolean projection today, which is exactly the gap this closes.
/// No live feature, or no live workflow naming it, falls back to
/// `default_wf_gates()` — every gate `pending`, nobody having acted on it —
/// the same default a brand-new workflow record seeds (trun-1).
fn build_gate_records(ctx: &Ctx, feature: Option<&Value>) -> R<Value> {
    let feature_str = feature.and_then(|f| f.as_str());
    let live_gates = match feature_str {
        Some(f) => {
            let workflows = list_workflows(&ctx.root).map_err(|_| Ex::Bail)?;
            find_live_workflow(&workflows, f).and_then(|wf| wf.get("gates").cloned())
        }
        None => None,
    };
    Ok(live_gates.unwrap_or_else(|| Value::Object(default_wf_gates())))
}

/// models-show-verb D4: the status `models` section carries each role's
/// `description` back onto the normalized slot — for DISPLAY, and nowhere
/// else.
///
/// `normalize_models` strips `description` on purpose, and that strip is what
/// keeps resolution blind to the field (D5: resolution, the model guard, and
/// `dispatch prepare` are untouched — they keep reading the stripped map, so
/// a described role and an undescribed one dispatch identically). But
/// `bee status --json` is a READ surface: an agent reading the models section
/// to pick a role needs the operator's own sentence about what the role is
/// for, and an open role set means the NAME is the only other thing bee can
/// publish about a role it never invented. So the merge happens HERE, on the
/// copy that goes into the JSON, never in `normalize_models`.
///
/// Three slot shapes, three answers:
///
/// - object slot (`{model}`, `{kind:"cli",…}`, `{kind:"native",…}`, the
///   explicit-fallback composite): `description` is appended beside the
///   fields it already carries.
/// - bare-string slot (`"sonnet"` — a seeded default, or a raw slot whose
///   only usable content was the description): it widens to
///   `{model:"sonnet", description:"…"}`. `render::format_slot` already
///   prints that object as `sonnet`, so the human `Models (…)` line is
///   unchanged by the widening.
/// - null slot (the config turned the role off): left alone. There is
///   nothing to describe, and widening it would turn "off" into an object.
///
/// A raw slot with no `description`, an empty/whitespace one, or a
/// non-string one is untouched — so a config that never uses the field
/// produces exactly the bytes it produced before this existed.
///
/// The text is trimmed but NOT truncated. The dispatch-door line truncates
/// (`hooks::model_guard::role_slot_description`) because it is a one-line
/// budget; JSON has no such budget, and a clipped sentence in machine output
/// is the status stating something the config does not say.
fn models_with_descriptions(normalized: &JMap, raw_models: Option<&Value>) -> JMap {
    let mut out = normalized.clone();
    let Some(raw_models) = raw_models.and_then(Value::as_object) else { return out };
    for (runtime, slots) in out.iter_mut() {
        let Some(raw_slots) = raw_models.get(runtime).and_then(Value::as_object) else { continue };
        let Some(slots) = slots.as_object_mut() else { continue };
        for (slot, value) in slots.iter_mut() {
            let described = raw_slots
                .get(slot)
                .and_then(Value::as_object)
                .and_then(|s| s.get("description"))
                .and_then(Value::as_str)
                .map(js_trim)
                .filter(|d| !d.is_empty());
            let Some(description) = described else { continue };
            match value {
                Value::Object(fields) => {
                    fields.insert("description".into(), json!(description));
                }
                Value::String(model) => {
                    *value = json!({ "model": model.clone(), "description": description });
                }
                _ => {}
            }
        }
    }
    out
}

// ─── buildStatus (bee.mjs ~874-1047) ───────────────────────────────────────

pub(crate) fn build_status(ctx: &mut Ctx, lanes_full: bool) -> R<JMap> {
    let state = read_state_full(ctx)?;
    let onboarding_raw = read_onboarding(ctx)?.unwrap_or(Value::Null);
    let handoff = read_handoff(ctx)?.unwrap_or(Value::Null);
    let cells = list_cells(ctx, None, None)?;
    let (mut open, mut claimed, mut capped, mut blocked) = (0i64, 0i64, 0i64, 0i64);
    for cell in &cells {
        match vget(cell, "status").and_then(|v| v.as_str()) {
            Some("open") => open += 1,
            Some("claimed") => claimed += 1,
            Some("capped") => capped += 1,
            Some("blocked") => blocked += 1,
            _ => {}
        }
    }
    let archived = archived_totals(ctx)?;
    let all_reservations = list_reservations(ctx, false);
    let active_res = list_reservations(ctx, true);
    // expiredUnreleased: `active.includes(r)` compares OBJECT REFERENCES from
    // two separate listReservations calls — always false in Node, faithfully
    // replicated: every released_at==null row counts (released_at is always
    // null by construction).
    let expired_unreleased = all_reservations
        .iter()
        .filter(|r| nullish(r.get("released_at")))
        .count();

    let config1 = read_config(ctx)?; // readConfig #1: `commands`
    let commands = config1.commands.clone();
    let backlog = read_backlog_counts(ctx)?;

    let mut staleness: Vec<Value> = Vec::new();
    if commands.is_empty() {
        staleness.push(json!(
            "No standard commands recorded — capture the host project's setup/start/test/verify into .bee/config.json `commands` so sessions can run the CI status gate."
        ));
    }
    if truthy(&onboarding_raw) {
        let v = vget(&onboarding_raw, "bee_version");
        if opt_truthy(v) && !str_eq(v, BEE_VERSION) {
            staleness.push(json!(format!(
                "Onboarding installed bee {} but plugin is {BEE_VERSION} — re-run onboarding.",
                tpl(v)
            )));
        }
    }
    if truthy(&handoff) && opt_truthy(vget(&handoff, "written_at")) {
        let age = now_ms() - date_parse_val(vget(&handoff, "written_at"));
        if age.is_finite() && age > STALE_HANDOFF_MS {
            staleness.push(json!(format!(
                "HANDOFF.json is older than 7 days (written {}).",
                tpl(vget(&handoff, "written_at"))
            )));
        }
    }
    if expired_unreleased > 0 {
        staleness.push(json!(format!(
            "{expired_unreleased} reservation(s) expired but never released — run bee reservations sweep."
        )));
    }
    if has_stale_advisor_key(ctx)? {
        staleness.push(json!(STALE_ADVISOR_KEY_WARNING));
    }
    let raw_for_validation = read_raw_config_for_validation(ctx)?;
    for problem in validate_models_config(raw_for_validation.as_ref()) {
        // `${problem.runtime ? ` models.${runtime}.${slot}:` : ''}` — slot is
        // explicitly null on runtime-level rows, templating as "null".
        let runtime_part = match (problem.runtime, problem.slot) {
            (Some(rt), slot) if !rt.is_empty() => {
                format!(" models.{rt}.{}:", slot.as_deref().unwrap_or("null"))
            }
            _ => String::new(),
        };
        staleness.push(json!(format!(
            "config validate [{}]{} {}",
            problem.code, runtime_part, problem.message
        )));
    }
    let raw_for_validation2 = read_raw_config_for_validation(ctx)?;
    for problem in validate_agent_files_drift(ctx, raw_for_validation2.as_ref()) {
        staleness.push(json!(format!(
            "config validate [{}] {} ({}): {}",
            problem.code,
            problem.agent.unwrap_or("undefined"),
            problem.slot.as_deref().unwrap_or("undefined"),
            problem.message
        )));
    }
    let phase_known = state
        .get("phase")
        .and_then(|v| v.as_str())
        .map(|p| KNOWN_PHASES.contains(&p))
        .unwrap_or(false);
    if !phase_known {
        staleness.push(json!(format!(
            "Unknown phase \"{}\" — not in the enum ({}; terminal alias: compounding-complete). Set state.phase to a valid value (idle at feature close); invented phases break machine-checkable handoffs (decision 0004).",
            tpl(state.get("phase")),
            PHASES.join(", ")
        )));
    }
    let review = build_review_block(ctx)?;
    let recovery = build_recovery_block(ctx)?;

    let execution_approved = state
        .get("approved_gates")
        .map(|g| matches!(vget(g, "execution"), Some(Value::Bool(true))))
        .unwrap_or(false);
    let feature_or_null = state
        .get("feature")
        .filter(|f| truthy(f))
        .cloned();
    let gate_records = build_gate_records(ctx, feature_or_null.as_ref())?;
    let ready = ready_cells(ctx, feature_or_null.as_ref())?;
    let unreviewed_count = review
        .get("candidates")
        .and_then(|c| vget(c, "unreviewed"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let recommended: Value = if !truthy(&onboarding_raw) {
        json!("Onboarding missing — run bee-hive onboarding.")
    } else if truthy(&handoff) {
        json!("HANDOFF present — present it to the user and WAIT. Never auto-resume.")
    } else if str_eq(state.get("phase"), "swarming") && !execution_approved {
        json!("NOT ready to swarm: gate \"execution\" is not approved.")
    } else if execution_approved && !ready.is_empty() {
        let ids: Vec<Value> = ready.iter().map(|c| vget(c, "id").cloned().unwrap_or(Value::Null)).collect();
        json!(format!(
            "{} ready cell(s): {} — orchestrator assigns them.",
            ready.len(),
            js_join(&ids, ", ")
        ))
    } else if state
        .get("phase")
        .and_then(|v| v.as_str())
        .map(|p| POST_EXECUTION_REVIEW_PHASES.contains(&p))
        .unwrap_or(false)
        && unreviewed_count > 0.0
    {
        json!(format!(
            "{} review candidate(s) awaiting: full review is user-invoked only, never dispatched automatically.",
            tpl(review.get("candidates").and_then(|c| vget(c, "unreviewed")))
        ))
    } else {
        match state.get("next_action") {
            Some(v) if truthy(v) => v.clone(),
            _ => json!("Invoke bee-hive."),
        }
    };

    let (drift_flag, drift_detail) = compute_runtime_drift(ctx, &onboarding_raw);
    let repo_hive = find_repo_hive(ctx);
    let (source_kind, source_root) = match &repo_hive {
        Some(hive) => classify_source(hive, &home_dir()),
        None => ("unknown".into(), None),
    };
    let worktree_notice = ungranted_worktree_notice(ctx);
    let contention = build_contention_summary(ctx)?;

    // Return-object literal: property VALUES evaluate top to bottom, so the
    // remaining readConfig-bearing calls happen in this exact order.
    let mut status = JMap::new();
    {
        let mut onboarding = JMap::new();
        onboarding.insert("installed".into(), json!(truthy(&onboarding_raw)));
        onboarding.insert(
            "bee_version".into(),
            match vget(&onboarding_raw, "bee_version") {
                None | Some(Value::Null) => Value::Null,
                Some(v) => v.clone(),
            },
        );
        onboarding.insert("plugin_version".into(), json!(BEE_VERSION));
        onboarding.insert("drift".into(), json!(drift_flag));
        if !drift_detail.is_empty() {
            onboarding.insert(
                "drift_detail".into(),
                Value::Array(drift_detail.iter().map(|d| json!(d)).collect()),
            );
        }
        status.insert("onboarding".into(), Value::Object(onboarding));
    }
    {
        let mut source = JMap::new();
        source.insert("kind".into(), json!(source_kind));
        source.insert(
            "root".into(),
            source_root.map(|r| json!(r)).unwrap_or(Value::Null),
        );
        status.insert("source".into(), Value::Object(source));
    }
    status.insert("phase".into(), state.get("phase").cloned().unwrap_or(Value::Null));
    status.insert("mode".into(), state.get("mode").cloned().unwrap_or(Value::Null));
    status.insert("feature".into(), state.get("feature").cloned().unwrap_or(Value::Null));
    status.insert(
        "gates".into(),
        state.get("approved_gates").cloned().unwrap_or(Value::Null),
    );
    // D3/D7 (additive only — `gates` above keeps its name, type, and
    // meaning): the full persisted gate record beside the projected
    // booleans, so a dashboard reads `pending` vs `rejected` without
    // re-deriving either from a bare `false`.
    status.insert("gate_records".into(), gate_records);
    // D4/D7: the run's own persisted lifecycle name, read straight off the
    // projected state.json field apply_workflow_d1_fields carries — never
    // re-derived here, matching the record's own "persisted, not derived"
    // discipline. Null when no live workflow has ever written it (a
    // pre-migration or brand-new-idle repo).
    status.insert("run_state".into(), state.get("run_state").cloned().unwrap_or(Value::Null));
    // D1 (awaiting-human): what `run_state: "awaiting-approval"` is waiting
    // ON — `kind`/`subject`/`asked_at`/`session`, or null when there is no
    // live wait. Read straight off the same projected field ah-1 carries
    // through `apply_workflow_d1_fields` — never re-derived here, matching
    // `run_state`'s own "persisted, not derived" discipline just above.
    status.insert("waiting_on".into(), state.get("waiting_on").cloned().unwrap_or(Value::Null));
    let level1 = bypass_level_root(ctx)?; // readConfig
    status.insert("gate_bypass".into(), json!(level1 != "off"));
    let level2 = bypass_level_root(ctx)?; // readConfig (Node calls it twice)
    status.insert("gate_bypass_level".into(), json!(level2));
    let ship = ship_visibility(ctx)?; // readConfig
    status.insert("ship_visibility".into(), json!(ship));
    status.insert(
        "route".into(),
        match state.get("route") {
            None | Some(Value::Null) => Value::Null,
            Some(v) => v.clone(),
        },
    );
    let config_models = read_config(ctx)?; // readConfig: `models`
    // models-show-verb D4: DISPLAY-only merge — see `models_with_descriptions`.
    status.insert(
        "models".into(),
        Value::Object(models_with_descriptions(
            &config_models.models,
            config_models.raw.get("models"),
        )),
    );
    // ── PUBLIC CONTRACT: `tier_mix` is renamed to `role_mix` (D6) ──────────
    //
    // `bee status --json` is published output, so this is a BREAKING key
    // change and it is taken deliberately, not as a side effect of the
    // internals moving. The three options were: rename, emit both keys for a
    // period, or keep `tier_mix` and change what is inside it.
    //
    // Rename wins because NO truthful content survives under the old key. A
    // reader of `tier_mix` reads `counts.extraction` / `counts.generation` /
    // `counts.ceiling` / `counts.untiered`, `tiered`, and `ceilingShare`.
    // After the split there are no tiers at all: every one of those
    // sub-fields would be absent, and every consumer that treats an absent
    // number as 0 — which is what a dashboard does — would report "no
    // ceiling usage, no untiered cells" forever. That is a SILENT wrong
    // answer. A key that is simply gone is a loud one: the consumer fails at
    // the read and its owner is told to look. Keeping the key, or emitting it
    // as an alias of the new object, buys compatibility of the key name only
    // while handing every existing field reader a permanent zero — the exact
    // dead-denominator defect this cell exists to remove.
    //
    // Blast radius, checked rather than assumed: no doc surface names
    // `tier_mix` (`docs/07-contracts.md` does not enumerate it), and the one
    // in-repo consumer is the published-contract probe in `tests.rs`, which
    // is updated to pin the rename rather than quietly deleted.
    {
        let mix = role_mix(ctx, feature_or_null.as_ref())?;
        let mut rm = JMap::new();
        rm.insert("counts".into(), Value::Object(mix.counts));
        rm.insert("cells".into(), json!(mix.cells));
        rm.insert("escalated".into(), json!(mix.escalated));
        rm.insert(
            "escalationShare".into(),
            if mix.escalation_share.fract() == 0.0 {
                json!(mix.escalation_share as i64)
            } else {
                json!(mix.escalation_share)
            },
        );
        status.insert("role_mix".into(), Value::Object(rm));
    }
    status.insert(
        "ceiling_scarcity".into(),
        ceiling_scarcity_warning(ctx)?.map(Value::Object).unwrap_or(Value::Null),
    );
    status.insert("handoff".into(), handoff.clone());
    {
        let mut c = JMap::new();
        c.insert("open".into(), json!(open));
        c.insert("claimed".into(), json!(claimed));
        c.insert("capped".into(), json!(capped));
        c.insert("blocked".into(), json!(blocked));
        c.insert("archived".into(), Value::Object(archived));
        c.insert("archivable".into(), Value::Object(archivable_backlog(&cells, feature_or_null.as_ref())));
        status.insert("cells".into(), Value::Object(c));
    }
    if lanes_full {
        let rows = build_lane_rows(ctx)?;
        status.insert(
            "lanes".into(),
            Value::Array(rows.into_iter().map(Value::Object).collect()),
        );
    } else {
        status.insert("lanes".into(), Value::Object(build_lane_summary(ctx)?));
    }
    status.insert("review".into(), Value::Object(review));
    status.insert("recovery".into(), Value::Object(recovery));
    {
        let mut sd = scribing_debt(ctx)?;
        sd.insert("orphaned".into(), Value::Object(global_scribing_debt(ctx)?));
        status.insert("scribing_debt".into(), Value::Object(sd));
    }
    status.insert("capture_queue".into(), Value::Object(capture_queue_summary(ctx)));
    // D4 (wayfinding-flow): open discovery maps under docs/discovery/ —
    // resume must be un-missable. `scan_discovery` (verbs/discovery.rs) is
    // the ONE scan; the session preamble (hooks/session_preamble/budget.rs)
    // calls the same scan independently since it renders in-process, never
    // through `bee status`. Always present (empty arrays when no map
    // exists) — `render_status_text` below is what guards the visible line.
    {
        let (efforts, unreadable) = crate::verbs::discovery::scan_discovery(&ctx.root);
        let effort_values: Vec<Value> = efforts
            .iter()
            .map(|e| {
                json!({
                    "name": e.name,
                    "destination": e.destination,
                    "open": e.open,
                    "frontier": e.frontier,
                })
            })
            .collect();
        let unreadable_values: Vec<Value> =
            unreadable.iter().map(|p| json!(p.display().to_string())).collect();
        let mut om = JMap::new();
        om.insert("efforts".into(), Value::Array(effort_values));
        om.insert("unreadable".into(), Value::Array(unreadable_values));
        status.insert("open_maps".into(), Value::Object(om));
    }
    status.insert(
        "pbi".into(),
        match &backlog {
            Some(counts) => {
                let mut p = JMap::new();
                p.insert("proposed".into(), counts.get("proposed").cloned().unwrap_or(Value::Null));
                p.insert("in_flight".into(), counts.get("inFlight").cloned().unwrap_or(Value::Null));
                p.insert("done".into(), counts.get("done").cloned().unwrap_or(Value::Null));
                Value::Object(p)
            }
            None => Value::Null,
        },
    );
    status.insert("commands".into(), Value::Object(commands));
    status.insert(
        "active_reservations".into(),
        Value::Array(active_res.into_iter().map(Value::Object).collect()),
    );
    {
        let ctrl_root = control_root_for(ctx)?; // readConfig inside resolveContext
        let workers = active_workers(ctx, &ctrl_root, None)?;
        status.insert(
            "workers".into(),
            Value::Array(workers.into_iter().map(Value::Object).collect()),
        );
    }
    status.insert(
        "critical_patterns_present".into(),
        json!(ctx
            .root
            .join("docs")
            .join("history")
            .join("learnings")
            .join("critical-patterns.md")
            .exists()),
    );
    {
        let recent = active_decisions(ctx, Some(3));
        let rows: Vec<Value> = recent
            .iter()
            .map(|event| {
                let mut o = JMap::new();
                match vget(event, "id") {
                    Some(v) => {
                        o.insert("id".into(), v.clone());
                    }
                    None => {} // undefined -> dropped by JSON.stringify
                }
                match vget(event, "date") {
                    Some(v) => {
                        o.insert("date".into(), v.clone());
                    }
                    None => {}
                }
                o.insert("decision".into(), json!(datamark(vget(event, "decision"))));
                Value::Object(o)
            })
            .collect();
        status.insert("recent_decisions".into(), Value::Array(rows));
    }
    status.insert("staleness_warnings".into(), Value::Array(staleness));
    status.insert("recommended_next".into(), recommended);
    if let Some(notice) = worktree_notice {
        status.insert("worktree_notice".into(), json!(notice));
    }
    if let Some(contention) = contention {
        status.insert("contention".into(), Value::Object(contention));
    }
    Ok(status)
}
