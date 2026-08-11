// dispatch prepare
//
// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::knowledge;
use crate::verbs::reservations::{
    finish, js_is_ws, parse_flags, prelude, pseudo_uuid_v4, truthy, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{
    release_reservations_for_agent, reserve_path_atomic, Err2, ReserveOutcome,
};
use serde_json::{Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

// ═══ dispatch prepare ══════════════════════════════════════════════════════

pub(crate) const DISPATCH_RUNTIMES: [&str; 2] = ["codex", "claude"];

pub(crate) const DISPATCH_KINDS: [&str; 4] = ["cell", "gather", "reviewer", "advisor"];

/// provenance: dispatch-prepare.mjs slotForKind (the PURPOSE MAP, advisor A1).
pub(crate) fn slot_for_kind(kind: &str) -> &'static str {
    match kind {
        "cell" | "gather" => "generation",
        "reviewer" => "review",
        _ => "advisor",
    }
}

/// provenance: dispatch-prepare.mjs purposeForKind — only 'cell' is
/// cell-execution; everything else is an explicit read-only gather.
pub(crate) fn purpose_is_gather(kind: &str) -> bool {
    kind != "cell"
}

pub(crate) struct Ownership {
    pub(crate) ok: bool,
    pub(crate) code: Option<&'static str>,
    pub(crate) status: Value,
    pub(crate) owner: Value,
    pub(crate) reason: String,
}

/// provenance: dispatch-prepare.mjs checkCellClaimOwnership (hardening-7) —
/// the CELL RECORD's own status/trace.worker, never the claims store.
pub(crate) fn check_cell_claim_ownership(cell: &Value, worker: &str) -> Ownership {
    let status = vget(cell, "status").cloned().unwrap_or(Value::Null);
    let status_str = tpl(vget(cell, "status"));
    let id = tpl(vget(cell, "id"));
    if !matches!(vget(cell, "status"), Some(Value::String(s)) if s == "claimed") {
        return Ownership {
            ok: false,
            code: Some("not_claimed"),
            status,
            owner: Value::Null,
            reason: format!(
                "cell \"{id}\" is \"{status_str}\", not \"claimed\" — dispatch prepare requires a claimed cell (run bee cells claim or bee cells claim-next first). Pass --force-ownership to override (audited)."
            ),
        };
    }
    let owner: Value = match vget(cell, "trace").and_then(|t| vget(t, "worker")) {
        Some(Value::String(w)) => Value::String(w.clone()),
        _ => Value::Null,
    };
    let owner_matches = matches!(&owner, Value::String(w) if w == worker);
    if !owner_matches {
        let shown = match &owner {
            Value::String(w) if !w.is_empty() => w.clone(),
            _ => "(unknown)".to_string(), // `owner || '(unknown)'`
        };
        return Ownership {
            ok: false,
            code: Some("not_owner"),
            status,
            owner,
            reason: format!(
                "cell \"{id}\" is claimed by worker \"{shown}\" — \"{worker}\" does not own this claim. Pass --force-ownership to override (audited)."
            ),
        };
    }
    Ownership { ok: true, code: None, status, owner, reason: String::new() }
}

/// provenance: dispatch-prepare.mjs PRIOR_ROUNDS_MAX_EVENT_LINES.
pub(crate) const PRIOR_ROUNDS_MAX_EVENT_LINES: usize = 12;

/// provenance: dispatch-prepare.mjs LEARNED_CONTEXT_MAX_LINES.
pub(crate) const LEARNED_CONTEXT_MAX_LINES: usize = 8;

/// How much of a cell's title the Agent-list row carries. Long enough to say
/// what the work is, short enough that the row still reads at a glance.
pub(crate) const DESCRIPTION_TITLE_MAX: usize = 60;

/// How much of the dispatch SUBJECT codex's `task_name` field carries. The
/// live-probed 0.145.0 schema (docs/knowledge/areas/hook-runtime/codex-spawn-
/// agent-dispatch-payload-schema-and-schema-agnosti.md) types `task_name` as
/// a plain required string with no documented charset or length limit, so
/// this exists to keep the row readable, not to satisfy a schema constraint —
/// kept as its own constant rather than reusing DESCRIPTION_TITLE_MAX in case
/// the two ever need to diverge.
pub(crate) const TASK_NAME_MAX: usize = 60;

/// provenance: dispatch-prepare.mjs priorRoundEventLines — the machine-
/// assembled digest of the cell record's own trace history, chronological
/// (ISO strings compare lexicographically; timeless events sink to the end in
/// insertion order, the sort being stable in both runtimes), capped at 12 with
/// one count line replacing the elided oldest.
pub(crate) fn prior_round_event_lines(cell: &Value) -> Vec<String> {
    let trace = match vget(cell, "trace") {
        Some(v) if is_plain_object(v) => v.clone(),
        _ => Value::Object(Map::new()),
    };
    let arr = |key: &str| -> Vec<Value> {
        match vget(&trace, key) {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        }
    };
    // (at, line) — `at` is `null` for a timeless event.
    let mut events: Vec<(Option<String>, String)> = Vec::new();
    let at_of = |v: &Value, key: &str| -> Option<String> {
        // `attempt.at || null` — a falsy `at` becomes null.
        match vget(v, key) {
            Some(x) if truthy(x) => Some(jsjson::js_to_string(x)),
            _ => None,
        }
    };

    for attempt in arr("attempts") {
        if !is_plain_object(&attempt) {
            continue;
        }
        let worker = truthy_str(vget(&attempt, "worker"))
            .map(str::to_string)
            .unwrap_or_else(|| "(unknown worker)".to_string());
        let verdict = vget(&attempt, "verdict");
        let sig = || match vget(&attempt, "failure_signature") {
            Some(v) if truthy(v) => jsjson::js_to_string(v),
            _ => "(none recorded)".to_string(),
        };
        if matches!(verdict, Some(Value::String(s)) if s == "blocked") {
            let note = one_line(vget(&attempt, "note"), 140);
            let reason = if note.is_empty() {
                format!("failure signature {}", sig())
            } else {
                note
            };
            events.push((at_of(&attempt, "at"), format!("- {worker} blocked: {reason}")));
        } else if matches!(verdict, Some(Value::String(s)) if s == "tests-red") {
            let note = one_line(vget(&attempt, "note"), 140);
            let note = if note.is_empty() { "(no excerpt recorded)".to_string() } else { note };
            events.push((at_of(&attempt, "at"), format!("- {worker} tests red: {note}")));
        } else if matches!(verdict, Some(Value::String(s)) if s == "fail") {
            events.push((
                at_of(&attempt, "at"),
                format!("- {worker} failed verify: failure signature {}", sig()),
            ));
        }
    }

    let capped_at = match vget(&trace, "capped_at") {
        Some(v) if truthy(v) => Some(jsjson::js_to_string(v)),
        _ => None,
    };
    for deviation in arr("deviations") {
        let Value::String(text) = &deviation else { continue };
        if js_trim(text).is_empty() {
            continue;
        }
        events.push((
            capped_at.clone(),
            format!("- (prior worker) deviation: {}", one_line(Some(&deviation), 140)),
        ));
    }

    for consult in arr("semantic_judge") {
        if !is_plain_object(&consult) {
            continue;
        }
        let judge = truthy_str(vget(&consult, "judge_model"))
            .map(str::to_string)
            .unwrap_or_else(|| "(judge)".to_string());
        let pointer = match vget(&consult, "failure_signature") {
            Some(v) if truthy(v) => {
                format!(" (failure signature {})", one_line(Some(v), 40))
            }
            _ => String::new(),
        };
        events.push((
            at_of(&consult, "recorded_at"),
            format!("- {judge} consult: {}{pointer}", tpl(vget(&consult, "verdict"))),
        ));
    }

    if let Some(Value::String(reason)) = vget(&trace, "reopened_reason") {
        if !js_trim(reason).is_empty() {
            events.push((
                at_of(&trace, "reopened_at"),
                format!(
                    "- (orchestrator) reopened: {}",
                    one_line(vget(&trace, "reopened_reason"), 140)
                ),
            ));
        }
    }
    if let Some(rework) = vget(&trace, "reopened_for_rework") {
        if truthy(rework) && is_plain_object(rework) {
            let reason = one_line(vget(rework, "reason"), 140);
            let reason = if reason.is_empty() {
                "NEEDS_REVISION verdict after cap".to_string()
            } else {
                reason
            };
            events.push((
                at_of(rework, "at"),
                format!("- (judge) reopened for rework: {reason}"),
            ));
        }
    }

    // Stable sort with Node's own comparator.
    events.sort_by(|a, b| match (&a.0, &b.0) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, _) => std::cmp::Ordering::Greater,
        (_, None) => std::cmp::Ordering::Less,
        (Some(x), Some(y)) => x.cmp(y),
    });
    let mut lines: Vec<String> = events.into_iter().map(|(_, line)| line).collect();
    if lines.len() > PRIOR_ROUNDS_MAX_EVENT_LINES {
        let kept = PRIOR_ROUNDS_MAX_EVENT_LINES - 1;
        let elided = lines.len() - kept;
        let tail = lines.split_off(lines.len() - kept);
        lines = std::iter::once(format!(
            "- ({elided} earlier event(s) elided — the cell record holds the rest)"
        ))
        .chain(tail)
        .collect();
    }
    lines
}

/// provenance: knowledge.mjs KNOWLEDGE_CONTEXT_LANE_BUDGETS /
/// KNOWLEDGE_CONTEXT_DEFAULT_BUDGET, read through `budgets[cell.lane] ?? default`.
pub(crate) fn lane_budget(lane: Option<&Value>) -> f64 {
    match lane {
        Some(Value::String(l)) => match l.as_str() {
            "tiny" => 8000.0,
            "small" => 12000.0,
            "standard" => 20000.0,
            "high-risk" => 30000.0,
            _ => 20000.0,
        },
        _ => 20000.0,
    }
}

/// provenance: dispatch-prepare.mjs bundleLearnedLines — the work-item
/// manifest first (every failure inside its try/catch falls through), then the
/// bundle index pointer. `Err(Delegate)` is NOT a JS-visible failure: it means
/// the lifted knowledge port cannot decide this bundle, so the whole command
/// re-runs under Node.
pub(crate) fn bundle_learned_lines(
    root: &Path,
    cell: &Value,
    read_first: &HashSet<String>,
) -> D<Vec<String>> {
    let Some(dir) = knowledge::bundle_dir(root) else { return Err(Delegate) };
    let budget = lane_budget(vget(cell, "lane"));
    let work = match vget(cell, "feature") {
        Some(Value::String(s)) => s.clone(),
        // A non-string `work` makes buildContextManifest throw missing_work
        // (`typeof work === 'string' ? work.trim() : ''`) -> the catch arm.
        _ => String::new(),
    };
    let manifest = if work.is_empty() {
        None
    } else {
        match knowledge::build_context_manifest(&dir, &work, budget, &knowledge::num(budget)) {
            knowledge::ManifestOut::Built(m) => Some(m),
            knowledge::ManifestOut::Thrown(_) => None, // caught by dispatch-prepare's try
            knowledge::ManifestOut::NeedsNode => return Err(Delegate),
        }
    };
    if let Some(manifest) = manifest {
        let Some(concepts) = knowledge::collect_concepts(&dir) else { return Err(Delegate) };
        // `new Map(...)`: last write wins per key (never hit for a real bundle,
        // where paths are unique).
        let mut titles: Vec<(String, Option<String>)> = Vec::new();
        for concept in &concepts {
            let key = format!("docs/knowledge/{}", concept.path);
            let title = match concept.data.get("title") {
                Some(Value::String(t)) if !t.is_empty() => Some(t.clone()),
                _ => None,
            };
            if let Some(slot) = titles.iter_mut().find(|(k, _)| *k == key) {
                slot.1 = title;
            } else {
                titles.push((key, title));
            }
        }
        let entries = match manifest.get("entries") {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        let mut lines = Vec::new();
        for entry in &entries {
            let path = tpl(vget(entry, "path"));
            if read_first.contains(&path) {
                continue; // read_first stays authoritative — never duplicated
            }
            // `titles.get(entry.path) || entry.path.slice(lastIndexOf('/') + 1)`
            let title = titles
                .iter()
                .find(|(k, _)| *k == path)
                .and_then(|(_, t)| t.clone())
                .unwrap_or_else(|| match path.rfind('/') {
                    Some(p) => path[p + 1..].to_string(),
                    None => path.clone(),
                });
            // The manifest path is a raw filesystem-derived string: a
            // crafted bundle filename could carry a newline and forge extra
            // bullet lines inside the worker prompt's "Learned context"
            // block, so it gets the same whitespace-collapsing treatment as
            // the title (a generous cap — paths are not expected to be
            // long, but never truncate a real one in practice).
            lines.push(format!(
                "- {} — {}",
                one_line(Some(&Value::String(path.clone())), 200),
                one_line(Some(&Value::String(title)), 140)
            ));
        }
        if !lines.is_empty() {
            return Ok(lines);
        }
    }
    if dir.join("index.md").exists() && !read_first.contains("docs/knowledge/index.md") {
        return Ok(vec![
            "- docs/knowledge/index.md — Knowledge bundle index (see \"Critical patterns\")"
                .to_string(),
        ]);
    }
    Ok(Vec::new())
}

/// provenance: knowledge.mjs bundleMode — a DIRECTORY is not a bundle: at
/// least one non-reserved markdown file must parse as a strict OKF concept
/// carrying a non-empty string `type`.
pub(crate) fn bundle_mode(root: &Path) -> D<bool> {
    let Some(dir) = knowledge::bundle_dir(root) else { return Err(Delegate) };
    match std::fs::metadata(&dir) {
        Ok(m) if m.is_dir() => {}
        _ => return Ok(false),
    }
    let Some(rels) = knowledge::list_bundle_markdown(&dir) else { return Err(Delegate) };
    for rel in rels {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if knowledge::is_reserved_basename(base) {
            continue;
        }
        let Ok(text) = knowledge::read_file_lossy(&knowledge::join_rel(&dir, &rel)) else { continue };
        match knowledge::parse_frontmatter(&text) {
            knowledge::Fm::Parsed { data, .. } => {
                if matches!(data.get("type"), Some(Value::String(t)) if !t.is_empty()) {
                    return Ok(true);
                }
            }
            _ => {}
        }
    }
    Ok(false)
}

/// provenance: dispatch-prepare.mjs learnedContextLines — source resolution,
/// first hit wins, capped at LEARNED_CONTEXT_MAX_LINES.
pub(crate) fn learned_context_lines(root: &Path, cell: &Value) -> D<Vec<String>> {
    let mut read_first: HashSet<String> = HashSet::new();
    if let Some(Value::Array(items)) = vget(cell, "read_first") {
        for entry in items {
            if let Value::String(s) = entry {
                let normalized = s.replace('\\', "/");
                let normalized = normalized.strip_prefix("./").unwrap_or(&normalized);
                read_first.insert(normalized.to_string());
            }
        }
    }
    let mut lines = if bundle_mode(root)? {
        bundle_learned_lines(root, cell, &read_first)?
    } else if root
        .join("docs")
        .join("history")
        .join("learnings")
        .join("critical-patterns.md")
        .exists()
        && !read_first.contains("docs/history/learnings/critical-patterns.md")
    {
        vec![
            "- docs/history/learnings/critical-patterns.md — Critical patterns (hard-won learnings)"
                .to_string(),
        ]
    } else {
        Vec::new()
    };
    lines.truncate(LEARNED_CONTEXT_MAX_LINES);
    Ok(lines)
}

/// provenance: dispatch-prepare.mjs cellPromptBody / promptBodyFor.
pub(crate) fn prompt_body_for(
    root: &Path,
    kind: &str,
    cell: Option<&Value>,
    worker: Option<&str>,
    // `Some((worktree_root, control_root))` only when the cell's feature has
    // a granted worktree — see `worktree_location` in `prepare_dispatch`,
    // the ONE resolution this and the envelope both read. `None` (an
    // unworktreed feature, or a non-cell kind) renders byte-identically to
    // before this Location block existed: the `{{#if worktree_root}}` marker
    // strips to nothing when its var is empty.
    worktree_location: Option<(&str, &str)>,
) -> D<Result<String, String>> {
    if kind != "cell" {
        let Some(template) = load_prompt(kind) else { return Err(Delegate) };
        return Ok(render(&template, &[]));
    }
    let cell = cell.expect("kind cell always carries a loaded cell");
    let Some(template) = load_prompt("worker-cell") else { return Err(Delegate) };
    let (worktree_root, control_root) = worktree_location.unwrap_or(("", ""));
    // The knowledge bundle (docs/knowledge/) and docs/history/ live in the
    // repo WORKING TREE, never the control store: when the cell's feature
    // carries a granted worktree, that worktree's own checkout is the
    // native root for the bundle read, exactly like a native verb invoked
    // from inside it (roots.rs's WIDE door) — never the control root's own
    // (possibly bundle-less, possibly stale) checkout.
    let bundle_root: &Path = if worktree_root.is_empty() { root } else { Path::new(worktree_root) };
    let learned = learned_context_lines(bundle_root, cell)?.join("\n");
    let prior = prior_round_event_lines(cell).join("\n");
    let cell_json = jsjson::stringify_pretty(cell);
    let feature = tpl(vget(cell, "feature"));
    let cell_id = tpl(vget(cell, "id"));
    Ok(render(
        &template,
        &[
            ("worker", worker.unwrap_or("undefined")),
            ("cell_id", &cell_id),
            ("feature", &feature),
            ("cell_json", &cell_json),
            ("learned_context", &learned),
            ("prior_rounds", &prior),
            ("worktree_root", worktree_root),
            ("control_root", control_root),
        ],
    ))
}

/// provenance: dispatch-prepare.mjs appendPrepareRecord — fail-open, exactly
/// like Node's try/catch: a log failure never blocks the payload.
pub(crate) fn append_prepare_record(root: &Path, record: &Map<String, Value>) {
    let mut line = Map::new();
    line.insert(
        "ts".into(),
        Value::String(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
    );
    line.insert("source".into(), Value::String("prepare".into()));
    for (k, v) in record {
        line.insert(k.clone(), v.clone());
    }
    let _ = crate::fsutil::append_jsonl(
        &root.join(".bee").join("logs").join("dispatch.jsonl"),
        &Value::Object(line),
    );
}

/// A prepareDispatch outcome: a returned value (envelope OR typed refusal), or
/// a thrown Error (malformed CALL).
pub(crate) enum Prepared {
    Value(Value),
    Thrown(String),
}

/// provenance: dispatch-prepare.mjs prepareDispatch(root, {...}). Throws only
/// on a malformed CALL; every legitimate cli-shaped / unconfigured-advisor /
/// native-unavailable / claim-ownership resolution is a typed {ok:false}
/// RETURN, not an exception.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_dispatch(
    root: &Path,
    runtime: &str,
    kind: &str,
    cell_id: Option<&str>,
    worker: Option<&str>,
    force_ownership: bool,
    classification: Option<&str>,
    purpose: Option<&str>,
    record_it: bool,
) -> D<Prepared> {
    // The runtime/kind gates already fired in the probe (validate() owns those
    // bytes), so both are known-good here.
    debug_assert!(DISPATCH_RUNTIMES.contains(&runtime) && DISPATCH_KINDS.contains(&kind));

    let mut cell: Option<Value> = None;
    let mut ownership_override: Option<Value> = None;
    let mut resolved_worker: Option<String> = None;

    if kind == "cell" {
        let Some(cell_id) = cell_id else {
            return Ok(Prepared::Thrown(
                "dispatch prepare: --cell is required when --kind cell.".to_string(),
            ));
        };
        let Some(loaded) = read_cell(root, cell_id)? else {
            return Ok(Prepared::Thrown(format!(
                "dispatch prepare: cell \"{cell_id}\" not found."
            )));
        };
        let Some(worker) = worker.filter(|w| !js_trim(w).is_empty()) else {
            return Ok(Prepared::Thrown(
                "dispatch prepare: --worker is required when --kind cell.".to_string(),
            ));
        };
        let trimmed = js_trim(worker).to_string();
        let ownership = check_cell_claim_ownership(&loaded, &trimmed);
        if !ownership.ok && !force_ownership {
            let mut refusal = Map::new();
            refusal.insert("ok".into(), Value::Bool(false));
            refusal.insert("type".into(), Value::String("refused".into()));
            refusal.insert("reason".into(), Value::String("claim_ownership".into()));
            refusal.insert(
                "code".into(),
                ownership.code.map(|c| Value::String(c.into())).unwrap_or(Value::Null),
            );
            refusal.insert("status".into(), ownership.status);
            refusal.insert("owner".into(), ownership.owner);
            refusal.insert("fix".into(), Value::String(ownership.reason));
            return Ok(Prepared::Value(Value::Object(refusal)));
        }
        if force_ownership {
            let mut ov = Map::new();
            ov.insert("forced_by".into(), Value::String(trimmed.clone()));
            ov.insert("bypassed".into(), Value::Bool(!ownership.ok));
            ov.insert(
                "code".into(),
                if ownership.ok {
                    Value::Null
                } else {
                    ownership.code.map(|c| Value::String(c.into())).unwrap_or(Value::Null)
                },
            );
            ov.insert(
                "owner_bypassed".into(),
                if ownership.ok { Value::Null } else { ownership.owner.clone() },
            );
            ov.insert(
                "status_bypassed".into(),
                if ownership.ok { Value::Null } else { ownership.status.clone() },
            );
            ov.insert("transferred".into(), Value::Bool(false));
            ov.insert("note".into(), Value::String("advisory bypass only — cell.trace.worker (the actual claim owner) was NOT transferred; no correct transfer primitive exists on this ownership axis (see comment above).".into()));
            ownership_override = Some(Value::Object(ov));
        }
        resolved_worker = Some(trimmed);
        cell = Some(loaded);
    }

    // `find_granted_worktree_for_feature` (status_full/topology.rs) resolved
    // ONCE here feeds both the envelope (below) and the rendered prompt's
    // Location block (prompt_body_for) — one resolution, two destinations,
    // never a second lookup that could drift from the first. `root` here is
    // always the MAIN checkout: a granted worktree's own `dispatch prepare`
    // call already refused through the narrow door in run_dispatch_prepare
    // (Roots::Unsupported(GrantedWorktree)) before reaching this function.
    let worktree_location: Option<(String, String)> = cell
        .as_ref()
        .and_then(|c| match vget(c, "feature") {
            Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .and_then(|feature| {
            crate::verbs::status_full::find_granted_worktree_for_feature(root, &feature)
                .map(|(_id, worktree_root)| (worktree_root, root.to_string_lossy().into_owned()))
        });

    let tier_token = slot_for_kind(kind);
    let models = read_models(root)?;
    let resolved = if kind == "advisor" {
        match resolve_advisor(&models, runtime) {
            Some(r) => r,
            None => {
                let mut refusal = Map::new();
                refusal.insert("ok".into(), Value::Bool(false));
                refusal.insert("reason".into(), Value::String("advisor_not_configured".into()));
                refusal.insert("fix".into(), Value::String(format!(
                    "set models.{runtime}.advisor in .bee/config.json to enable an advisor consult (resolveAdvisor never falls back to another tier)."
                )));
                return Ok(Prepared::Value(Value::Object(refusal)));
            }
        }
    } else {
        let r = resolve_tier(&models, tier_token, runtime, purpose_is_gather(kind));
        if let Resolved::Refused { slot } = &r {
            let mut refusal = Map::new();
            refusal.insert("ok".into(), Value::Bool(false));
            refusal.insert("type".into(), Value::String("refused".into()));
            refusal.insert("reason".into(), Value::String("cli_tier_gather_only".into()));
            refusal.insert("slot".into(), Value::String(slot.clone()));
            refusal.insert("fix".into(), Value::String(CLI_REFUSAL_FIX.into()));
            return Ok(Prepared::Value(Value::Object(refusal)));
        }
        r
    };

    let prompt_body = match prompt_body_for(
        root,
        kind,
        cell.as_ref(),
        resolved_worker.as_deref(),
        worktree_location.as_ref().map(|(w, c)| (w.as_str(), c.as_str())),
    )? {
        Ok(body) => body,
        Err(msg) => return Ok(Prepared::Thrown(msg)),
    };
    let requested_model = match &resolved {
        Resolved::Model { model, .. } => Some(model.clone()),
        _ => None,
    };
    // The generation tier carries TWO rendered agents — bee-build executes
    // a cell (reserves, writes, commits, caps), bee-gather reads and
    // reports (never writes, per .claude/agents/bee-gather.md). tier alone
    // cannot tell them apart (guard.rs's pinned_agent_type stays a tier-only
    // lookup, mirrored by hooks/model_guard.rs PINNED_AGENT_TYPE); `kind`
    // is the one signal that can, and only a --kind cell dispatch is a cell
    // execution (dp-2).
    let pinned_type =
        if kind == "cell" { "bee-build" } else { pinned_agent_type(tier_token) };

    // The dispatch SUBJECT — computed ONCE, here, before the transport match,
    // so every branch below (native override, native fallback, codex
    // spawn_agent, claude Agent) reads the identical value. It used to be
    // computed only inside the claude Agent arm, which is exactly how the
    // codex `task_name` branch was missed for a month (dispatch-label-
    // chokepoint plan.md — the fourth attempt at this one rule).
    //
    // `kind == "cell"`: subject is the cell's own "<id>: <title>" — the cell
    // is already loaded here for the prompt, and its title is what the row
    // is FOR. A whitespace-only or absent title is no title: the bare kind,
    // never a dangling "id: ".
    //
    // Every other kind: subject is "<kind>: <purpose>" when the caller passes
    // `--purpose`, and the bare kind otherwise — today's exact bytes when no
    // `--purpose` is given, so this stays back-compatible by construction.
    // Their purpose is the caller's to state; prepare never invents one.
    let subject = if kind == "cell" {
        cell.as_ref()
            .map(|c| (tpl(vget(c, "id")), one_line(vget(c, "title"), DESCRIPTION_TITLE_MAX)))
            .filter(|(_, title)| !title.trim().is_empty())
            .map(|(id, title)| format!("{id}: {title}"))
            .unwrap_or_else(|| kind.to_string())
    } else {
        match purpose.map(js_trim).filter(|p| !p.is_empty()) {
            Some(p) => format!(
                "{kind}: {}",
                one_line(Some(&Value::String(p.to_string())), DESCRIPTION_TITLE_MAX)
            ),
            None => kind.to_string(),
        }
    };

    let mut tool = String::new();
    let mut payload = Map::new();
    let mut channel = String::new();
    let mut refusal: Option<Value> = None;
    let mut native_confirmed = false;
    // envelopeExtra, kept as its two possible keys so the spread order below
    // stays byte-identical.
    let mut extra_transport: Option<&str> = None;
    let mut extra_fallback_reason: Option<&str> = None;

    match &resolved {
        Resolved::Native { model, effort, fallback, agent_type, .. } => {
            native_confirmed = classification == Some(NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE);
            if native_confirmed {
                tool = "spawn_agent".into();
                payload.insert(
                    "agent_type".into(),
                    Value::String(if agent_type.is_empty() {
                        "worker".to_string()
                    } else {
                        agent_type.clone()
                    }),
                );
                payload.insert(
                    "message".into(),
                    Value::String(format!("[bee-tier: {tier_token}]\n{prompt_body}")),
                );
                payload.insert("model".into(), Value::String(model.clone()));
                payload.insert("fork_turns".into(), Value::String("none".into()));
                if let Some(effort) = effort {
                    payload.insert("reasoning_effort".into(), Value::String(effort.clone()));
                }
                channel = "codex-native".into();
                extra_transport = Some("native-override");
            } else if let Some(command) = fallback.as_ref().filter(|c| !c.is_empty()) {
                // cli-exec: NO label field. This payload is `{command, stdin}`
                // only — a recorded limit (dispatch-label-chokepoint plan.md
                // "What this does not do"), not an oversight: no field exists
                // on an external CLI-executor call to carry a subject.
                tool = "Bash".into();
                payload.insert("command".into(), Value::String(command.clone()));
                payload.insert("stdin".into(), Value::String(prompt_body.clone()));
                channel = "cli-exec".into();
                extra_fallback_reason = Some("native_unavailable");
            } else {
                let mut r = Map::new();
                r.insert("ok".into(), Value::Bool(false));
                r.insert("type".into(), Value::String("refused".into()));
                r.insert("reason".into(), Value::String("native_unavailable".into()));
                r.insert(
                    "detail".into(),
                    Value::String(
                        classification
                            .filter(|c| !c.is_empty())
                            .unwrap_or(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY)
                            .to_string(),
                    ),
                );
                refusal = Some(Value::Object(r));
            }
        }
        Resolved::Cli { command } => {
            // cli-exec: NO label field — see the identical comment on the
            // native-fallback Bash arm above; this is the same recorded
            // limit, not an oversight.
            tool = "Bash".into();
            payload.insert("command".into(), Value::String(command.clone()));
            payload.insert("stdin".into(), Value::String(prompt_body.clone()));
            channel = "cli-exec".into();
        }
        _ if runtime == "codex" => {
            tool = "spawn_agent".into();
            // Carries the SAME subject as the claude Agent branch below,
            // instead of the bare cell id (or "bee-{kind}") it used to —
            // this arm is exactly the one the codex gap hid in (plan.md).
            // codex's `task_name` is a plain required string on the
            // live-probed 0.145.0 schema (see TASK_NAME_MAX); one-lined and
            // capped so a long subject cannot read like a paragraph.
            payload.insert(
                "task_name".into(),
                Value::String(one_line(Some(&Value::String(subject.clone())), TASK_NAME_MAX)),
            );
            payload.insert(
                "message".into(),
                Value::String(format!("[bee-tier: {tier_token}]\n{prompt_body}")),
            );
            payload.insert("fork_turns".into(), Value::String("none".into()));
            channel = "codex-native".into();
        }
        _ => {
            tool = "Agent".into();
            payload.insert("subagent_type".into(), Value::String(pinned_type.into()));
            payload.insert(
                "prompt".into(),
                Value::String(format!("[bee-tier: {tier_token}]\n{prompt_body}")),
            );
            // `requestedModel || tierToken`
            let model_tag = requested_model
                .clone()
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| tier_token.to_string());
            // A description that is only a model name is a red flag
            // (work-visibility D2) — and it was the one prepare emitted, so
            // orchestrators wrote their own bare `Execute <id>` instead and
            // the agent list read as a column of ids. `subject` is computed
            // once, above, and every transport branch (including this one)
            // reads it.
            payload.insert("description".into(), Value::String(format!("{subject} ({model_tag})")));
            if let Resolved::Model { model, .. } = &resolved {
                payload.insert("model".into(), Value::String(model.clone()));
            }
            channel = "claude-agent".into();
        }
    }

    if let Some(refusal) = refusal {
        return Ok(Prepared::Value(refusal));
    }

    let param_model = match (&channel[..], &resolved) {
        ("claude-agent", Resolved::Model { model, .. }) => Some(model.clone()),
        _ => None,
    };
    let economics = derive_economics(
        &channel,
        tier_token,
        param_model.as_deref(),
        &resolved,
        native_confirmed,
    );

    let dispatch_id = pseudo_uuid_v4();

    let mut record = Map::new();
    record.insert("dispatch_id".into(), Value::String(dispatch_id.clone()));
    record.insert("kind".into(), Value::String(kind.to_string()));
    record.insert(
        "cell".into(),
        match &cell {
            Some(c) => vget(c, "id").cloned().unwrap_or(Value::Null),
            None => Value::Null,
        },
    );
    record.insert("runtime".into(), Value::String(runtime.to_string()));
    let classification_value = match classification {
        Some(c) if !c.is_empty() => Value::String(c.to_string()),
        _ => Value::Null,
    };
    if let Some(reason) = extra_fallback_reason {
        record.insert("native_fallback_reason".into(), Value::String(reason.into()));
        record.insert("native_classification".into(), classification_value.clone());
    }
    if extra_transport.is_some() {
        record.insert("native_classification".into(), classification_value);
    }
    if let Some(ov) = &ownership_override {
        record.insert("ownership_override".into(), ov.clone());
    }
    for (k, v) in &economics {
        record.insert(k.clone(), v.clone());
    }
    // `record_it` is false on the PROBE pass: run() builds the whole envelope
    // once to discover delegate-shaped inputs before a byte is produced, then
    // rebuilds it for real. Gating the append here means a command that ends
    // up delegating never leaves a prepare line behind, and one that is served
    // leaves exactly one — Node's count.
    if record_it {
        append_prepare_record(root, &record);
    }

    let mut envelope = Map::new();
    envelope.insert("tool".into(), Value::String(tool));
    envelope.insert("payload".into(), Value::Object(payload));
    envelope.insert("dispatch_id".into(), Value::String(dispatch_id));
    envelope.insert("economics".into(), Value::Object(economics));
    // Present only when the cell's feature has a granted worktree — see
    // `worktree_location` above. A feature with no worktree split (or a
    // non-cell dispatch) omits both keys, so the envelope stays
    // byte-identical to before this Location block existed.
    if let Some((worktree_root, control_root)) = &worktree_location {
        envelope.insert("worktree_root".into(), Value::String(worktree_root.clone()));
        envelope.insert("control_root".into(), Value::String(control_root.clone()));
    }
    if let Some(t) = extra_transport {
        envelope.insert("transport".into(), Value::String(t.into()));
    }
    if let Some(r) = extra_fallback_reason {
        envelope.insert("fallback_reason".into(), Value::String(r.into()));
    }
    if let Some(ov) = ownership_override {
        envelope.insert("ownership_override".into(), ov);
    }
    Ok(Prepared::Value(Value::Object(envelope)))
}

/// provenance: bee.mjs readNativeTransportClassification — the delegating
/// slice. An absent / unreadable / unparseable probe record and a
/// schema-mismatched one both short-circuit to native_budget_only with NO
/// subprocess; anything past that point shells out to codex-cli, so it
/// delegates.
pub(crate) const NATIVE_TRANSPORT_PROBE_SCHEMA: &str = "native-transport-probe/1";

pub(crate) fn native_transport_classification(root: &Path) -> D<&'static str> {
    let file = root.join(".bee").join("native-transport-probe.json");
    // doctorSafeReadJson: unreadable OR unparseable both yield null.
    let record = match std::fs::read(&file) {
        Err(_) => None,
        Ok(bytes) => serde_json::from_str::<Value>(&String::from_utf8_lossy(&bytes)).ok(),
    };
    match record {
        None => Ok(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY),
        Some(r) if !matches!(vget(&r, "schema"), Some(Value::String(s)) if s == NATIVE_TRANSPORT_PROBE_SCHEMA) => {
            Ok(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY)
        }
        // A live probe record: doctorRepoIdentity + `codex --version` +
        // `codex features list` + the config-scope hash all have to run.
        Some(_) => Err(Delegate),
    }
}

/// bee.mjs's `claimAndReserveForDispatch` — the claim, then one reserve per
/// declared file IN DECLARATION ORDER, stopping at the first conflict and
/// unwinding everything this call created, in REVERSE (reservations first,
/// then the claim), so the refusal can truthfully say the repo is back in its
/// pre-call state.
///
/// Both doors are the SHARED ones (`cells::claim_cell_from_flags`,
/// `reservations::reserve_path_atomic`), never a second copy — re-deriving
/// them would fork the store-mutation logic C1 exists to protect.
pub(crate) fn claim_and_reserve_for_dispatch(
    root: &Path,
    topo: Option<(&Path, &str)>,
    cell_id: &str,
    worker: &str,
    session_flag: Option<&str>,
) -> R2<Result<(Value, Vec<Value>, bool, Option<String>), String>> {
    use crate::verbs::cells;
    // ttl/isolate are structurally absent: bee.mjs builds `{id, worker}` plus
    // `session-id` only when one was passed.
    let door = match cells::claim_cell_from_flags(root, cell_id, worker, session_flag, None) {
        Ok(d) => d,
        Err(cells::Fail::Delegate) => return Err(Err2::Ex),
        Err(cells::Fail::Thrown(m)) => return Ok(Err(m)),
    };
    let cell = door.cell;
    let claimed_id = match cell.get("id") {
        Some(Value::String(s)) => s.clone(),
        other => jsjson::js_to_string(other.unwrap_or(&Value::Null)),
    };
    // `Array.isArray(cell.files) ? cell.files.filter(f => typeof f === 'string' && f) : []`
    let files: Vec<String> = match cell.get("files") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|f| match f {
                Value::String(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };

    let mut reserved: Vec<Value> = Vec::new();
    for file_path in &files {
        let section = reserve_path_atomic(topo, root.to_str().ok_or(Err2::Ex)?, worker, &claimed_id, file_path)?;
        let conflict_lines: Vec<String> = match section {
            ReserveOutcome::Reserved(reservation) => {
                // The NORMALIZED path off the lease record, not files[i].
                reserved.push(reservation.get("path").cloned().unwrap_or(Value::Null));
                continue;
            }
            ReserveOutcome::Thrown(m) => return Ok(Err(m)),
            ReserveOutcome::ForeignHold(hold) => {
                let or_unknown = |k: &str| match hold.get(k) {
                    Some(v) if truthy(v) => jsjson::js_to_string(v),
                    _ => "unknown".to_string(),
                };
                vec![format!(
                    "- checkout \"{}\" holds \"{}\" (cross-worktree hold, feature {}, cell {})",
                    hold.get("holder").map_or("undefined".into(), jsjson::js_to_string),
                    hold.get("path").map_or("undefined".into(), jsjson::js_to_string),
                    or_unknown("feature"),
                    or_unknown("cell"),
                )]
            }
            ReserveOutcome::Conflicts(conflicts) => conflicts
                .iter()
                .map(|c| {
                    format!(
                        "- {} holds \"{}\" (cell {})",
                        c.get("agent").map_or("undefined".into(), jsjson::js_to_string),
                        c.get("path").map_or("undefined".into(), jsjson::js_to_string),
                        c.get("cell").map_or("undefined".into(), jsjson::js_to_string),
                    )
                })
                .collect(),
        };

        // Unwind, in reverse. Both rungs read stores this same call has
        // already probed (reserve_prechecks, and the claim door's own
        // prescans), so the Exotic arm below is unreachable in practice —
        // recorded as this branch's one accepted residual.
        let mut unwind_note = "the claim was unwound and state restored as found".to_string();
        let unwound = (|| -> R2<Result<(), String>> {
            if !reserved.is_empty() {
                if let Out::Thrown(m) = release_reservations_for_agent(
                    topo,
                    root.to_str().ok_or(Err2::Ex)?,
                    worker,
                    Some(&claimed_id),
                )? {
                    return Ok(Err(m));
                }
            }
            match cells::unclaim_cell(root, &claimed_id, door.session_id.as_deref(), false) {
                Ok(_) => Ok(Ok(())),
                Err(cells::Fail::Delegate) => Err(Err2::Ex),
                Err(cells::Fail::Thrown(m)) => Ok(Err(m)),
            }
        })()?;
        if let Err(message) = unwound {
            unwind_note = format!(
                "UNWIND FAILED ({message}) — restore by hand: bee reservations release --agent {worker} --cell {claimed_id} --json ; bee cells unclaim --id {claimed_id} --json"
            );
        }
        let mut lines = vec![format!(
            "dispatch prepare --claim: reservation conflict on cell \"{claimed_id}\" — nothing dispatched; {unwind_note}:"
        )];
        lines.extend(conflict_lines);
        return Ok(Err(lines.join("\n")));
    }

    // dp-r1: the claim (and every reservation) stands from here on — register
    // the claiming worker against the cell it now owns, THE SAME record shape
    // `bee state worker add --nickname <w> --cell <id> --tier <t> --status
    // running` writes (state_group::register_worker_for_cell reuses that
    // door's own write path). A registration failure never unwinds the claim
    // — it is real state the caller already holds — it only travels back as
    // `(worker_registered: false, Some(registration_error))` for the payload
    // to name loudly.
    let tier = match cell.get("tier") {
        Some(Value::String(t)) if !t.is_empty() => Some(t.clone()),
        _ => None,
    };
    let (worker_registered, registration_error) =
        match crate::verbs::state_group::register_worker_for_cell(root, worker, &claimed_id, tier.as_deref()) {
            Ok(()) => (true, None),
            Err(message) => (false, Some(message)),
        };
    Ok(Ok((cell, reserved, worker_registered, registration_error)))
}

pub(crate) fn run_dispatch_prepare(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(
        &flags,
        &["runtime", "kind", "cell", "worker", "force-ownership", "claim", "session-id", "purpose"],
    ) {
        return None;
    }
    // `flags.claim === true`: a bare `--claim` or `--claim=true`. Any other
    // spelling is validate()'s to answer.
    let claim = match flags.get("claim") {
        None => false,
        Some(FlagV::Present) => true,
        Some(FlagV::S(s)) if s == "true" => true,
        Some(FlagV::S(s)) if s == "false" => false,
        Some(FlagV::S(_)) => return None,
    };
    // `--session-id` is documented as ignored WITHOUT --claim; a caller that
    // passes it anyway is an unproven shape here. With --claim it is the
    // claim door's own `sessionFlag`.
    let session_flag: Option<String> = match (claim, flags.get("session-id")) {
        (_, None) => None,
        (false, Some(_)) => return None,
        (true, Some(FlagV::S(s))) => Some(s.clone()),
        (true, Some(FlagV::Present)) => return None, // String(true) — unproven
    };
    // validate(): boolean-typed --force-ownership given as =value.
    match flags.get("force-ownership") {
        None | Some(FlagV::Present) => {}
        Some(FlagV::S(s)) if s == "true" || s == "false" => {}
        Some(FlagV::S(_)) => return None,
    }
    // validate(): runtime/kind required + enum-checked.
    let runtime = flags.req_str("runtime")?.to_string();
    let kind = flags.req_str("kind")?.to_string();
    if !DISPATCH_RUNTIMES.contains(&runtime.as_str()) || !DISPATCH_KINDS.contains(&kind.as_str()) {
        return None; // validate()'s enum message
    }
    // `typeof flags.cell === 'string' && flags.cell ? flags.cell : null`
    let cell_id = flags.truthy_str("cell").map(str::to_string);
    let worker = flags.truthy_str("worker").map(str::to_string);
    // Only read for a non-cell kind (prepare_dispatch itself ignores it for
    // `--kind cell`); `one_line` inside prepare_dispatch collapses it to a
    // single line, so no separate "<one line>" validation is needed here.
    let purpose = flags.truthy_str("purpose").map(str::to_string);
    let force_ownership = matches!(flags.get("force-ownership"), Some(FlagV::Present));

    // ── everything that can still delegate happens BEFORE prelude: its
    //    drift-cache write would otherwise swallow the Node re-run's
    //    manifest_changed line. ─────────────────────────────────────────────
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, "dispatch prepare", use_json, t0, &why));
        }
        Roots::None => {
            return Some(emit_no_root_error(&cwd, "dispatch prepare", use_json, t0));
        }
    };
    let prompt_name = if kind == "cell" { "worker-cell" } else { kind.as_str() };
    if !prompts_match_disk(&root, prompt_name) {
        return None; // prompt skew ⇒ delegate (C4)
    }
    let classification = if runtime == "codex" {
        Some(native_transport_classification(&root).ok()?)
    } else {
        None
    };

    // ── --claim's own argument refusals. bee.mjs throws all three BEFORE
    //    prepareDispatch is ever called, so they are resolved here and the
    //    dry-run below is skipped entirely when one fires. ─────────────────
    let claim_arg_error: Option<String> = if !claim {
        None
    } else if kind != "cell" {
        Some(format!(
            "dispatch prepare: --claim is only valid with --kind cell (got --kind {kind}) — claiming and reserving are cell-execution moves; gather/reviewer/advisor dispatches never own a cell."
        ))
    } else if cell_id.is_none() {
        Some("dispatch prepare: --cell is required when --kind cell.".to_string())
    } else if worker.is_none() {
        Some("dispatch prepare: --worker is required when --kind cell.".to_string())
    } else {
        None
    };

    // Dry-run the whole build to surface every delegate-shaped input before a
    // single byte (or the prepare-time log line) is produced. The build is
    // free of side effects apart from appendPrepareRecord, which is applied on
    // the SECOND pass only.
    //
    // With --claim this pass is a DELEGATION PROBE ONLY: bee.mjs sequences the
    // claim BEFORE the payload build, so a Thrown produced here (over the
    // still-unclaimed cell) is not necessarily the message the real,
    // post-claim build produces. Its verdict is discarded; only "would this
    // delegate?" is kept, and it is kept because after the claim's O_EXCL
    // write nothing may delegate at all (campaign rule 2).
    let prepared = if claim_arg_error.is_some() {
        Prepared::Value(Value::Null) // unused — the refusal short-circuits below
    } else {
        prepare_dispatch(
            &root,
            &runtime,
            &kind,
            cell_id.as_deref(),
            worker.as_deref(),
            force_ownership,
            classification,
            purpose.as_deref(),
            false,
        )
        .ok()?
    };
    // The cross-worktree hold topology reservePathAtomic resolves for itself.
    // `prelude` above already narrowed this to an ORDINARY checkout, so this
    // is always `(workRoot, "main")` — a linked worktree delegated earlier.
    let topology = match claim {
        false => None,
        true => match crate::roots::resolve_store_root_worktree(&cwd) {
            crate::roots::RootsWt::Go(r) => r.hold_topology(),
            _ => return None,
        },
    };

    let ctx = match prelude("dispatch prepare", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    if let Some(message) = claim_arg_error {
        return finish(&ctx, Ok(Out::Thrown(message)));
    }

    // ── the claim + reserve gesture, before the payload build (Node's order).
    let mut claim_outcome: Option<Vec<Value>> = None;
    let mut worker_registered = false;
    let mut registration_error: Option<String> = None;
    if claim {
        let topo = topology.as_ref().map(|(m, h)| (m.as_path(), h.as_str()));
        match claim_and_reserve_for_dispatch(
            &ctx.root,
            topo,
            cell_id.as_deref().unwrap_or(""),
            worker.as_deref().unwrap_or(""),
            session_flag.as_deref(),
        ) {
            Ok(Ok((_cell, reserved, registered, reg_err))) => {
                claim_outcome = Some(reserved);
                worker_registered = registered;
                registration_error = reg_err;
            }
            Ok(Err(message)) => return finish(&ctx, Ok(Out::Thrown(message))),
            Err(e) => return finish(&ctx, Err(e)),
        }
    }

    let out: R2<Out> = match prepared {
        Prepared::Thrown(msg) if !claim => Ok(Out::Thrown(msg)),
        _ => {
            // Re-run for real so the prepare-time record is appended exactly
            // once, with a freshly minted dispatch_id/ts like Node's.
            match prepare_dispatch(
                &ctx.root,
                &runtime,
                &kind,
                cell_id.as_deref(),
                worker.as_deref(),
                force_ownership,
                classification,
                purpose.as_deref(),
                true,
            ) {
                Ok(Prepared::Value(result)) => {
                    // `claimOutcome ? {...out, claimed:true, reserved} : out`
                    // — the claim/reservations are real state either way, so
                    // the result names them beside the payload.
                    let result = match &claim_outcome {
                        None => result,
                        Some(reserved) => {
                            let mut m = match result {
                                Value::Object(m) => m,
                                other => return finish(&ctx, Ok(Out::Emit(other, String::new(), 0))),
                            };
                            m.insert("claimed".into(), Value::Bool(true));
                            m.insert("reserved".into(), Value::Array(reserved.clone()));
                            // dp-r1: named loudly either way — the payload IS
                            // the printed text below, json mode or not.
                            m.insert("worker_registered".into(), Value::Bool(worker_registered));
                            if let Some(err) = &registration_error {
                                m.insert("registration_error".into(), Value::String(err.clone()));
                            }
                            Value::Object(m)
                        }
                    };
                    let text = jsjson::stringify_pretty(&result);
                    Ok(Out::Emit(result, text, 0))
                }
                Ok(Prepared::Thrown(msg)) => Ok(Out::Thrown(msg)),
                // Accepted residual, recorded in the module header: a
                // delegate-shaped input that the pre-claim probe did not see.
                // Unreachable in practice — record=true differs from the
                // probe only by appendPrepareRecord.
                Err(_) => Err(Err2::Ex),
            }
        }
    };
    finish(&ctx, out)
}

// ═══ dispatch wave ══════════════════════════════════════════════════════════
//
// `bee dispatch wave` — no bee.mjs counterpart (workflow-lessons wfl-4): the
// current schedule wave (`cells::compute_schedule`, the SAME door `cells
// schedule` reads), claimed and prepared in one call instead of one `dispatch
// prepare --cell <id> --worker <name> --claim` per cell. Every cell in the
// wave runs through the identical shared doors `dispatch prepare --claim`
// uses — `claim_and_reserve_for_dispatch` then `prepare_dispatch` — so a
// wave payload is byte-identical to what the single-cell command would have
// emitted for that cell; this command only saves the orchestrator the
// per-cell round trips. One cell's refusal (already claimed, a reservation
// conflict) never aborts the rest of the wave — it is recorded in `skipped`
// with a typed reason and the loop continues.
//
// dispatch review P1: this door MUTATES the shared control plane (claims,
// reservations, worker rows), unlike its read-only sibling `cells schedule`
// — whose all-features default it had inherited by construction. The
// resolved feature is exactly one of: an explicit `--feature`, the calling
// session's bound lane, or the default record's own `feature` (the same
// three-step `resolve_mutation_lock_scope` every mutating `state` verb
// already resolves against); nothing resolving is a typed refusal, never a
// silent every-feature grab. `--limit <n>` (a positive integer) caps how
// many cells of the current wave are actually claimed — the rest of the
// wave is simply left untouched, not reported — bounding a speculative
// batch by what the caller can actually spawn workers for; omitted, the
// whole wave stands as before.

/// The worker nickname `dispatch wave` claims each cell under when the
/// caller does not hand one down: derived from the cell id alone, so two
/// runs against the same still-open cell (or a human reading the claim
/// trace) see a stable, self-explaining name — never a counter that drifts
/// with call order.
pub(crate) fn auto_wave_worker_name(cell_id: &str) -> String {
    format!("w-{cell_id}")
}

/// Classifies a `claim_and_reserve_for_dispatch` refusal string into the
/// named `skipped` reasons the cell calls out — "an unwind of that same
/// refusal itself failed", "already claimed", and "a reservation conflict"
/// — falling back to a generic `claim_refused` for any other typed refusal
/// that door can produce (budget caps, an unapproved execution gate, and the
/// like). Message-sniffing is the one option here: the door returns a
/// rendered String, not a typed enum, by the same design `dispatch prepare
/// --claim`'s own single-cell caller already accepts.
///
/// `UNWIND FAILED` is checked FIRST (review P2): the door's own reservation-
/// conflict message already embeds its unwind note in the same string
/// (`"...reservation conflict on cell ...; UNWIND FAILED (...)..."`), so a
/// naive "reservation conflict" match alone would bury a leaked claim behind
/// the ordinary, already-handled `reservation_conflict` reason instead of
/// surfacing the worse, still-open state.
pub(crate) fn wave_skip_reason(message: &str) -> &'static str {
    if message.contains("UNWIND FAILED") {
        "unwind_failed"
    } else if message.contains("reservation conflict") {
        "reservation_conflict"
    } else if message.contains("not \"open\"") {
        "already_claimed"
    } else {
        "claim_refused"
    }
}

fn wave_skip(id: &str, reason: &'static str, detail: String) -> Value {
    let mut s = Map::new();
    s.insert("id".into(), Value::String(id.to_string()));
    s.insert("reason".into(), Value::String(reason.to_string()));
    s.insert("detail".into(), Value::String(detail));
    Value::Object(s)
}

/// Best-effort unwind for a wave-loop claim that must be undone before its
/// cell lands in `skipped`: either a claim+reserve that SUCCEEDED,
/// immediately followed by a `prepare_dispatch` that failed to build a
/// payload over the very cell it just loaded (kind is always "cell", the
/// worker is always non-empty, and ownership always matches the claim this
/// same call just took — see `prepare_dispatch`'s Thrown arms); or (review
/// P2) a `claim_and_reserve_for_dispatch` call whose OWN internal unwind
/// never ran because its failure propagated out through a bare `?` on
/// `reserve_path_atomic`, mid-loop, after the claim itself already stood —
/// the wave loop cannot see how many of that cell's files were reserved
/// before the failure, so it cannot hand this function an accurate
/// `reserved` list. Always attempts BOTH steps (release, then unclaim)
/// regardless of what the caller can prove was reserved —
/// `release_reservations_for_agent` is a no-op when nothing matches
/// `(worker, cell)`, so calling it on a claim that took zero reservations
/// costs nothing. Mirrors `claim_and_reserve_for_dispatch`'s own unwind
/// (reservations first, then the claim, in reverse), with `force_ownership`
/// on the unclaim since this call is undoing its own, still-fresh claim
/// rather than resolving a contested one.
fn unwind_wave_claim(root: &Path, topo: Option<(&Path, &str)>, worker: &str, id: &str) -> String {
    let mut note = "the claim and its reservations were unwound".to_string();
    let release_failed =
        match release_reservations_for_agent(topo, root.to_str().unwrap_or(""), worker, Some(id)) {
            Ok(Out::Thrown(m)) => Some(m),
            Err(_) => Some("release_reservations_for_agent hit an unproven shape".to_string()),
            Ok(_) => None,
        };
    let unclaim_failed = match crate::verbs::cells::unclaim_cell(root, id, None, true) {
        Ok(_) => None,
        Err(crate::verbs::cells::Fail::Delegate) => {
            Some("unclaim_cell hit an unproven shape".to_string())
        }
        Err(crate::verbs::cells::Fail::Thrown(m)) => Some(m),
    };
    if release_failed.is_some() || unclaim_failed.is_some() {
        note = format!(
            "UNWIND FAILED (release: {}; unclaim: {}) — restore by hand: bee reservations release --agent {worker} --cell {id} --json ; bee cells unclaim --id {id} --force-ownership --json",
            release_failed.as_deref().unwrap_or("ok"),
            unclaim_failed.as_deref().unwrap_or("ok"),
        );
    }
    note
}

/// provenance: none — new native command (workflow-lessons wfl-4).
pub(crate) fn run_dispatch_wave(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(
        &flags,
        &["runtime", "feature", "session-id", "limit"],
    ) {
        return None;
    }
    let runtime = flags.req_str("runtime")?.to_string();
    if !DISPATCH_RUNTIMES.contains(&runtime.as_str()) {
        return None; // validate()'s enum message equivalent
    }
    let feature_flag = flags.truthy_str("feature").map(str::to_string);
    let session_flag = flags.truthy_str("session-id").map(str::to_string);
    // `--limit`: shape only here (a bare `--limit` with no value is
    // unproven); the positive-integer value check waits for `ctx` below so
    // a bad value gets the same typed refusal every other value-shape
    // problem in this door gets, not a bare "unsupported command shape".
    let limit_flag = match flags.get("limit") {
        None => None,
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(FlagV::Present) => return None,
    };

    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, "dispatch wave", use_json, t0, &why));
        }
        Roots::None => {
            return Some(emit_no_root_error(&cwd, "dispatch wave", use_json, t0));
        }
    };
    // Every wave cell renders the "worker-cell" prompt (kind is always
    // "cell") — the same skew guard `dispatch prepare --claim` applies.
    if !prompts_match_disk(&root, "worker-cell") {
        return None;
    }
    let classification = if runtime == "codex" {
        Some(native_transport_classification(&root).ok()?)
    } else {
        None
    };
    // Claiming is not optional for `dispatch wave` (there is no read-only
    // mode), so the hold topology is always resolved up front — the same
    // narrow-door check `dispatch prepare --claim` makes.
    let topology: Option<(PathBuf, String)> =
        match crate::roots::resolve_store_root_worktree(&cwd) {
            crate::roots::RootsWt::Go(r) => r.hold_topology(),
            _ => return None,
        };

    let ctx = match prelude("dispatch wave", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };

    // dispatch review P1: resolve the ONE feature this wave targets — an
    // explicit `--feature`, else the calling session's bound lane, else the
    // default record's own `feature` — the identical three-step resolution
    // `resolve_mutation_lock_scope` gives every mutating `state` verb.
    // Nothing resolving is a typed refusal: a mutating wave never falls back
    // to the read-only `cells schedule` default of every feature at once.
    let scope = match crate::verbs::state_group::resolve_mutation_lock_scope(
        &ctx.root,
        feature_flag.as_deref(),
        false,
    ) {
        Ok(s) => s,
        Err(_) => return finish(&ctx, Err(Err2::Ex)),
    };
    let Some(feature) = scope.feature else {
        return finish(
            &ctx,
            Err(Err2::Msg(
                "dispatch wave: refused — no feature resolved (no --feature given, the \
                 calling session has no bound lane, and the default record names none). \
                 FIX: pass --feature <name> naming the pipeline to dispatch."
                    .to_string(),
            )),
        );
    };
    let limit: Option<usize> = match &limit_flag {
        None => None,
        Some(raw) => match raw.trim().parse::<u64>() {
            Ok(n) if n > 0 => Some(n as usize),
            _ => {
                return finish(
                    &ctx,
                    Err(Err2::Msg(
                        "dispatch wave: --limit must be a positive integer.".to_string(),
                    )),
                );
            }
        },
    };

    let cells = match crate::verbs::cells::list_cells(&ctx.root, Some(feature.as_str()), None) {
        Ok(cells) => cells,
        Err(crate::verbs::cells::Delegate) => return finish(&ctx, Err(Err2::Ex)),
    };
    // Same exotic-id guard `cells schedule` applies before scheduling.
    for cell in &cells {
        let schedulable =
            matches!(cell.get("status"), Some(Value::String(s)) if s == "open" || s == "claimed");
        if schedulable && !matches!(cell.get("id"), Some(Value::String(_))) {
            return finish(&ctx, Err(Err2::Ex));
        }
    }
    let schedule = crate::verbs::cells::compute_schedule(&cells);
    let mut wave_ids: Vec<String> = schedule.waves.first().cloned().unwrap_or_default();
    if let Some(n) = limit {
        wave_ids.truncate(n);
    }

    let topo = topology.as_ref().map(|(m, h)| (m.as_path(), h.as_str()));
    let mut wave_payloads: Vec<Value> = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();
    let mut economics: Vec<Value> = Vec::new();

    for id in &wave_ids {
        let worker = auto_wave_worker_name(id);
        match claim_and_reserve_for_dispatch(&ctx.root, topo, id, &worker, session_flag.as_deref())
        {
            Err(_) => {
                // dispatch review P2: `claim_and_reserve_for_dispatch`'s own
                // `?`-propagated Err never reaches ITS internal unwind, so
                // the claim (and any reservations it already took) may
                // still stand — best-effort undo it here rather than leak
                // it, and name the cell in the detail either way.
                let note = unwind_wave_claim(&ctx.root, topo, &worker, id);
                let unwind_failed = note.contains("UNWIND FAILED");
                skipped.push(wave_skip(
                    id,
                    if unwind_failed { "unwind_failed" } else { "unsupported" },
                    format!(
                        "cell \"{id}\" hit an unproven shape mid-wave — run \
                         `bee dispatch prepare --cell {id} --worker {worker} --runtime {runtime} --claim` \
                         for it directly. ({note})"
                    ),
                ));
            }
            Ok(Err(message)) => {
                let reason = wave_skip_reason(&message);
                skipped.push(wave_skip(id, reason, message));
            }
            Ok(Ok((_cell, reserved, worker_registered, registration_error))) => {
                match prepare_dispatch(
                    &ctx.root,
                    &runtime,
                    "cell",
                    Some(id.as_str()),
                    Some(worker.as_str()),
                    false,
                    classification,
                    None,
                    true,
                ) {
                    Ok(Prepared::Value(result)) => {
                        let mut m = match result {
                            Value::Object(m) => m,
                            // kind "cell" always renders an object envelope
                            // (either the full payload or a typed refusal);
                            // this arm is unreached in practice.
                            other => {
                                wave_payloads.push(other);
                                continue;
                            }
                        };
                        m.insert("claimed".into(), Value::Bool(true));
                        m.insert("reserved".into(), Value::Array(reserved));
                        m.insert("worker_registered".into(), Value::Bool(worker_registered));
                        if let Some(err) = &registration_error {
                            m.insert("registration_error".into(), Value::String(err.clone()));
                        }
                        if let Some(Value::Object(econ)) = m.get("economics") {
                            let mut e = econ.clone();
                            e.insert("id".into(), Value::String(id.clone()));
                            economics.push(Value::Object(e));
                        }
                        wave_payloads.push(Value::Object(m));
                    }
                    Ok(Prepared::Thrown(msg)) => {
                        let note = unwind_wave_claim(&ctx.root, topo, &worker, id);
                        skipped.push(wave_skip(
                            id,
                            "prepare_failed",
                            format!("cell \"{id}\": {msg} — {note}"),
                        ));
                    }
                    Err(Delegate) => {
                        let note = unwind_wave_claim(&ctx.root, topo, &worker, id);
                        skipped.push(wave_skip(
                            id,
                            "prepare_failed",
                            format!(
                                "cell \"{id}\": prepare_dispatch hit an unproven shape mid-wave — {note}"
                            ),
                        ));
                    }
                }
            }
        }
    }

    let mut result = Map::new();
    result.insert("wave".into(), Value::Array(wave_payloads));
    result.insert("skipped".into(), Value::Array(skipped));
    result.insert("economics".into(), Value::Array(economics));
    let result = Value::Object(result);
    let text = jsjson::stringify_pretty(&result);
    finish(&ctx, Ok(Out::Emit(result, text, 0)))
}
