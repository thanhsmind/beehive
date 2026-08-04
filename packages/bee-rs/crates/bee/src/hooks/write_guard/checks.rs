// write-record resolution, checkWrite and checkGitBashCommand
//
// Split out of the single 5.9k-line hooks/write_guard.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── write-record resolution (provenance: guards.mjs resolveWriteRecord /
// resolveWriteTopology; state.mjs resolvePipeline/readLane) ────────────────

pub(crate) struct Topo {
    pub(crate) ctx: JsCtx,
    pub(crate) control_root: String,
}

/// provenance: guards.mjs resolveWriteTopology.
pub(crate) fn resolve_write_topology(root: &str, control_root_override: Option<&str>) -> R<Topo> {
    let ctx = match resolve_context(root)? {
        CtxOutcome::Ok(c) => c,
        CtxOutcome::Threw => JsCtx::default(),
    };
    let over = control_root_override
        .map(js_trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let control_root = over
        .or_else(|| ctx.control_root.clone())
        .unwrap_or_else(|| root.to_string());
    Ok(Topo { ctx, control_root })
}

pub(crate) enum RecordResolution {
    Ok { record: Map<String, Value>, source: &'static str },
    Fail { reason: String },
}

/// provenance: guards.mjs resolveWriteRecord + state.mjs resolvePipeline.
pub(crate) fn resolve_write_record(
    control_root: &str,
    state: &Map<String, Value>,
    session_id: Option<&str>,
    emit: &mut Emit,
) -> R<RecordResolution> {
    let sid = match session_id.map(js_trim).filter(|s| !s.is_empty()) {
        Some(s) => s,
        None => {
            return Ok(RecordResolution::Ok { record: state.clone(), source: "default" });
        }
    };
    // resolvePipeline(controlRoot, { sessionId }).
    let control2 = control_root_for_state(control_root)?;
    let defaults = |_: &mut Emit| -> R<RecordResolution> {
        Ok(RecordResolution::Ok {
            record: read_state(Path::new(control_root))?,
            source: "default",
        })
    };
    let session = match read_session(&control2, sid)? {
        Some(s) => s,
        None => return defaults(emit),
    };
    let bound = match session.get("lane") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => return defaults(emit),
    };
    let session_id_disp = js_disp_opt(session.get("id"));
    // lanePath validity (state.mjs requireLaneFeature).
    if bound.contains('/') || bound.contains('\\') || bound.contains("..") {
        return Ok(RecordResolution::Fail {
            reason: format!(
                "bee lane guard: session \"{}\" is bound to lane \"{}\", which is not a valid lane name (lane feature must be a plain id (no path separators).) — never guessed back to the default pipeline. FIX: rebind or unbind the session (claims bindSessionLane/unbindSessionLane).",
                session_id_disp, bound
            ),
        });
    }
    let file = Path::new(&control2).join(".bee").join("lanes").join(format!("{}.json", bound));
    let file_s = file.to_string_lossy().into_owned();
    let rel_file = np_relative(&control2, &file_s)?;
    if !file.exists() {
        return Ok(RecordResolution::Fail {
            reason: format!(
                "bee lane guard: session \"{}\" is bound to lane \"{}\" but {} does not exist — resolution never guesses back to the default pipeline. FIX: start the lane (startFeature with lane mode) or unbind the session.",
                session_id_disp, bound, rel_file
            ),
        });
    }
    // readLane(control2, bound).
    // readJson warns and yields null on corruption; laneRecordFrom(null) then
    // takes the corrupt-shape arm below — LANE_CORRUPT, same deny, same code.
    let parsed = read_json_g(&file)?;
    let lane_record = parsed.and_then(|v| match v {
        Value::Object(m) if m.get("feature") == Some(&Value::String(bound.clone())) => Some(m),
        _ => None,
    });
    let record = match lane_record {
        None => {
            // readLane's corrupt-shape console.warn line, then LANE_CORRUPT.
            emit.stderr.push_str(&format!(
                "readLane: skipping corrupt lane record \"{}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {}\").\n",
                rel_file, rel_file
            ));
            return Ok(RecordResolution::Fail {
                reason: format!(
                    "bee lane guard: session \"{}\" is bound to lane \"{}\" but its record is corrupt — display never guesses and mutations must refuse. FIX: inspect/restore {}, then retry.",
                    session_id_disp, bound, rel_file
                ),
            });
        }
        Some(m) => m,
    };
    // laneRecordFrom merge over defaultLaneRecord.
    let mut merged = Map::new();
    merged.insert("schema_version".into(), Value::String("1.0".into()));
    merged.insert("feature".into(), Value::String(bound.clone()));
    merged.insert("mode".into(), Value::Null);
    merged.insert("phase".into(), Value::String("idle".into()));
    let mut gates = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        gates.insert(g.into(), Value::Bool(false));
    }
    merged.insert("approved_gates".into(), Value::Object(gates.clone()));
    merged.insert("summary".into(), Value::String(String::new()));
    merged.insert("next_action".into(), Value::String(String::new()));
    merged.insert("created_at".into(), Value::Null);
    for (k, v) in &record {
        merged.insert(k.clone(), v.clone());
    }
    let mut merged_gates = gates;
    if let Some(Value::Object(over)) = record.get("approved_gates") {
        for (k, v) in over {
            merged_gates.insert(k.clone(), v.clone());
        }
    }
    merged.insert("approved_gates".into(), Value::Object(merged_gates));
    if merged.get("phase") == Some(&Value::String("validating".into())) {
        merged.insert("phase".into(), Value::String("planning".into()));
    }
    Ok(RecordResolution::Ok { record: merged, source: "lane" })
}

// ─── plan.md freeze (hook-teeth D1) ────────────────────────────────────────
// docs/history/<feature>/plan.md denies Edit/Write/MultiEdit once that
// feature's shape gate is approved. The feature is resolved from the path
// segment itself — never from the calling session's own bound feature — so
// a write to one feature's plan.md is judged solely on that feature's gate
// state.

/// Parses "docs/history/<feature>/plan.md" and returns the feature id, or
/// None when the path is not exactly that shape (a nested path, a different
/// filename, or a bare docs/history/ target). CONTEXT.md and every other
/// docs/history file never match, so they never reach the deny below.
pub(crate) fn plan_freeze_feature(normalized: &str) -> Option<String> {
    let rest = normalized.strip_prefix("docs/history/")?;
    let feature = rest.strip_suffix("/plan.md")?;
    if feature.is_empty() || feature.contains('/') {
        return None;
    }
    Some(feature.to_string())
}

/// Lane-aware gate read for the plan-freeze check: `.bee/lanes/<feature>.json`
/// wins when it parses to an object, mirroring resolve_write_record's lane
/// precedence over the default pipeline. Otherwise the default state (the
/// `state` the caller already read) is consulted only when its own `feature`
/// field names this same feature — a mismatched, absent, or corrupt-lane
/// resolution is "no opinion", never a guess, so the caller allows.
pub(crate) fn plan_freeze_shape_approved(
    control_root: &str,
    state: &Map<String, Value>,
    feature: &str,
) -> R<bool> {
    let lane_file = Path::new(control_root)
        .join(".bee")
        .join("lanes")
        .join(format!("{}.json", feature));
    if let Some(Value::Object(lane)) = read_json_g(&lane_file)? {
        let shape = lane
            .get("approved_gates")
            .and_then(|g| g.as_object())
            .and_then(|g| g.get("shape"))
            .map(|v| v == &Value::Bool(true))
            .unwrap_or(false);
        return Ok(shape);
    }
    let state_feature = match state.get("feature") {
        Some(Value::String(s)) => Some(s.as_str()),
        _ => None,
    };
    if state_feature != Some(feature) {
        return Ok(false);
    }
    let shape = state
        .get("approved_gates")
        .and_then(|g| g.as_object())
        .and_then(|g| g.get("shape"))
        .map(|v| v == &Value::Bool(true))
        .unwrap_or(false);
    Ok(shape)
}

/// provenance: guards.mjs resolveHoldTopology.
pub(crate) fn resolve_hold_topology<'a>(ctx: &'a JsCtx, control_root: &'a str) -> Option<(String, String)> {
    ctx.workspace_root.as_ref()?;
    match &ctx.worktree_id {
        None => Some((control_root.to_string(), "main".to_string())),
        Some(wt) => match &ctx.workspace_id {
            Some(ws) if ws == wt => Some((control_root.to_string(), ws.clone())),
            _ => None,
        },
    }
}

/// provenance: guards.mjs resolveWritePolicyMode.
pub(crate) fn resolve_write_policy_mode(config: &Map<String, Value>) -> &'static str {
    let configured = match config.get("guards") {
        Some(Value::Object(g)) => match g.get("write_policy") {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        },
        _ => String::new(),
    };
    match configured.as_str() {
        "observe" => "observe",
        "shared-disjoint" => "shared-disjoint",
        _ => "isolated",
    }
}

pub(crate) enum Ownership {
    Open,
    Corrupt,
    Blocked { owner: String },
}

/// provenance: guards.mjs checkWorkspaceOwnership.
pub(crate) fn check_workspace_ownership(control_root: &str, ctx: &JsCtx, session_id: &str) -> R<Ownership> {
    let workspace_id = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
    let workspace = match read_workspace(control_root, &workspace_id) {
        WorkspaceRead::Missing => return Ok(Ownership::Open),
        WorkspaceRead::Corrupt => return Ok(Ownership::Corrupt),
        WorkspaceRead::Ok(w) => w,
    };
    let owner = match workspace.get("write_owner_session") {
        Some(v) if truthy(v) => v.clone(),
        _ => return Ok(Ownership::Open),
    };
    if owner == Value::String(session_id.to_string()) {
        return Ok(Ownership::Open);
    }
    let owner_str = match &owner {
        Value::String(s) => s.clone(),
        _ => {
            // readSession(controlRoot, non-string) → sessionPath requireId
            // throws → caught → null → not live → never blocks.
            return Ok(Ownership::Open);
        }
    };
    let owner_session = read_session(control_root, &owner_str)?;
    let live = match owner_session {
        Some(s) => !heartbeat_stale(&s, now_ms())?,
        None => false,
    };
    if !live {
        return Ok(Ownership::Open);
    }
    Ok(Ownership::Blocked { owner: owner_str })
}

// ─── checkWrite (provenance: guards.mjs checkWrite, step for step) ─────────

pub(crate) enum WV {
    Allow,
    AllowWarn(String),
    Deny(String),
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn check_write(
    root: &str,
    state: &Map<String, Value>,
    rel_path: &str,
    agent_name: Option<&str>,
    session_id: Option<&str>,
    control_root_override: Option<&str>,
    emit: &mut Emit,
) -> R<WV> {
    let normalized = normalize_rel(rel_path);

    if let Some(verb) = direct_edit_verb(&normalized) {
        return Ok(WV::Deny(format!(
            "bee direct-edit guard: \"{}\" is CLI-owned — direct edits are blocked in every phase. \
Hand-edited state files reintroduce schema drift (the exact class the CLI validates away). \
FIX: use {} instead of editing this file directly.",
            normalized, verb
        )));
    }

    if let Some(ext) = docs_history_code_deny(&normalized) {
        return Ok(WV::Deny(format!(
            "bee docs-history guard: \"{}\" writes a \"{}\" code file into docs/history/, which is \
the tech-agnostic KNOWLEDGE layer (.md only — CONTEXT.md, plan.md, reports, walkthrough). Code never lives there. \
FIX: put a persistent verify/helper script in the project's own scripts (committed with the product) and point \
the cell's verify command at it; put a disposable proof in .bee/spikes/<feature>/. Never docs/history.",
            normalized, ext
        )));
    }

    if let Some(kind) = scratch_shape_deny(&normalized) {
        return Ok(WV::Deny(format!(
            "bee scratch-shape guard: \"{}\" looks like {} landing in a tracked directory. \
Every ephemeral file bee writes for its own working purposes belongs in .bee/tmp/<feature-or-session>/ \
(feasibility code in .bee/spikes/<feature>/), never a tracked path (docs/specs/doctrine-layer.md). \
FIX: write it to .bee/tmp/ instead (or .bee/spikes/ for a feasibility proof), and let `bee tmp sweep` clear it later.",
            normalized, kind
        )));
    }

    let topo = resolve_write_topology(root, control_root_override)?;
    let ctx = &topo.ctx;
    let control_root = topo.control_root.clone();

    // D1 (hook-teeth): plan.md freezes mechanically once its own feature's
    // shape gate is approved — resolved from the path segment, never from
    // the calling session's bound feature, and phase-independent like the
    // other path-scoped denies above.
    if let Some(feature) = plan_freeze_feature(&normalized) {
        if plan_freeze_shape_approved(&control_root, state, &feature)? {
            return Ok(WV::Deny(format!(
                "bee plan-freeze guard: \"{}\" is {}'s plan.md and its shape gate is approved — \
plan.md freezes once shape is locked and no longer accepts direct edits. \
FIX: stamp a revision (`bee state plan-rev bump --lane {}`) instead of editing in place, or \
unapprove the shape gate first to redraft the plan.",
                normalized, feature, feature
            )));
        }
    }

    let (record, source) = match resolve_write_record(&control_root, state, session_id, emit)? {
        RecordResolution::Fail { reason } => return Ok(WV::Deny(reason)),
        RecordResolution::Ok { record, source } => (record, source),
    };

    // Cross-session hold deny (fsh-7 D3) — sessionId-gated.
    if let Some(sid) = session_id.map(js_trim).filter(|s| !s.is_empty()) {
        if reservation_store_corrupt(root) {
            let res_rel = np_relative(
                root,
                &Path::new(root).join(".bee").join("reservations.json").to_string_lossy(),
            )?;
            return Ok(WV::Deny(format!(
                "bee hold guard: the reservation store ({}) is present but \
unreadable/corrupt — failing closed for a session-aware write rather than silently treating it as empty. \
FIX: inspect/restore the reservation store, then retry.",
                res_rel
            )));
        }
        let hold_conflicts = find_session_conflicts(root, sid, &[normalized.clone()])?;
        if !hold_conflicts.is_empty() {
            let acting_workspace = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
            let mut same_workspace: Vec<&Resv> = Vec::new();
            for holder in &hold_conflicts {
                let holder_session = holder.session.clone().unwrap_or(Value::Null);
                if session_workspace_id(&control_root, &holder_session)? == acting_workspace {
                    same_workspace.push(holder);
                }
            }
            if let Some(holder) = same_workspace.first() {
                return Ok(WV::Deny(format!(
                    "bee cross-session hold: \"{}\" is held by session \"{}\" (agent {}, cell {}), {}. \
Wait for the hold to expire or coordinate with that session — a cross-session hold is a hard block (D3).",
                    normalized,
                    js_disp_opt(holder.session.as_ref()),
                    js_disp_opt(holder.agent.as_ref()),
                    js_disp_opt(holder.cell.as_ref()),
                    hold_expiry(holder)?
                )));
            }
        }
    }

    // Cross-worktree foreign-hold consultation (xwh-4 / msn-14 D4).
    if let Some((main_root, holder_id)) = resolve_hold_topology(ctx, &control_root) {
        if holds_store_corrupt(&main_root) {
            return Ok(WV::Deny(
                "bee cross-worktree hold guard: the shared holds ledger (.bee/runtime/cross-worktree-holds.json \
in the main checkout) is present but unreadable/corrupt — failing closed rather than silently \
treating it as empty. FIX: inspect/restore the ledger in the main checkout, then retry."
                    .to_string(),
            ));
        }
        let foreign = find_foreign_holds(&main_root, &holder_id, &[normalized.clone()])?;
        if let Some(hold) = foreign.first() {
            let feature_disp = match hold.get("feature") {
                Some(v) if truthy(v) => js_disp(v),
                _ => "unknown".to_string(),
            };
            let cell_clause = match hold.get("cell") {
                Some(v) if truthy(v) => format!(", cell {}", js_disp(v)),
                _ => String::new(),
            };
            if is_exclusive_path(Path::new(root), &normalized)? {
                return Ok(WV::Deny(format!(
                    "bee cross-worktree hold: \"{}\" is held by checkout \"{}\" (feature {}{}), {}. \
Wait for the hold to expire or coordinate with that checkout — a cross-worktree hold is a hard block.",
                    normalized,
                    js_disp_opt(hold.get("holder")),
                    feature_disp,
                    cell_clause,
                    foreign_hold_expiry(hold)?
                )));
            }
            return Ok(WV::AllowWarn(format!(
                "bee cross-worktree hold: \"{}\" is also held by checkout \"{}\" (feature {}{}), {} — \
advisory only (different workspace, not an exclusive resource). \
Coordinate with that checkout if possible; otherwise \"bee worktree merge\" will surface any real conflict \
between the two checkouts at merge time.",
                normalized,
                js_disp_opt(hold.get("holder")),
                feature_disp,
                cell_clause,
                foreign_hold_expiry(hold)?
            )));
        }
    }

    // phase = record?.phase || 'idle'
    let phase = match record.get("phase") {
        Some(v) if truthy(v) => v.clone(),
        _ => Value::String("idle".into()),
    };

    // Workspace-ownership deny (msn-21, class (c)).
    if let Some(sid) = session_id.map(js_trim).filter(|s| !s.is_empty()) {
        if source == "default" && phase != Value::String("swarming".into()) {
            let config = read_config(Path::new(&control_root))?;
            if resolve_write_policy_mode(&config) == "isolated" {
                match check_workspace_ownership(&control_root, ctx, sid)? {
                    Ownership::Open => {}
                    Ownership::Corrupt => {
                        let ws_id = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
                        let ws_file = Path::new(&control_root)
                            .join(".bee")
                            .join("runtime")
                            .join("workspaces")
                            .join(format!("{}.json", ws_id));
                        let ws_rel = np_relative(&control_root, &ws_file.to_string_lossy())?;
                        return Ok(WV::Deny(format!(
                            "bee workspace-ownership guard: the workspace record for \"{}\" ({}) is present but \
unreadable/corrupt — failing closed for a session-aware write rather than silently treating it as \
unowned. FIX: inspect/restore the workspace record, then retry.",
                            ws_id, ws_rel
                        )));
                    }
                    Ownership::Blocked { owner } => {
                        let ws_id = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
                        return Ok(WV::Deny(format!(
                            "bee write-policy: workspace \"{}\" is write-owned by session \"{}\" \
— a second write-capable session defaults to isolation, never a shared write into the same checkout. \
FIX: coordinate with that session, wait for its heartbeat to go stale, or start your own feature with \
`bee state start-feature --isolate` (or set guards.auto_isolate to true in .bee/config.json) to work \
in a fresh worktree instead.",
                            ws_id, owner
                        )));
                    }
                }
            }
        }
    }

    if is_terminal_phase(&phase) {
        let config = read_config(Path::new(&control_root))?;
        let idle_gate_on = !matches!(
            config.get("guards"),
            Some(g) if truthy(g) && g.get("idle_gate") == Some(&Value::Bool(false))
        );
        if idle_gate_on && !under_allowed_prefix(&normalized) {
            return Ok(WV::Deny(intake_refusal(
                &phase,
                &format!("writing \"{}\"", normalized),
                "",
            )));
        }
        return Ok(WV::Allow);
    }

    if is_gated_phase(&phase) {
        let execution_approved = record
            .get("approved_gates")
            .and_then(|g| g.get("execution"))
            == Some(&Value::Bool(true));
        if !execution_approved && !under_allowed_prefix(&normalized) {
            return Ok(WV::Deny(format!(
                "bee gate: phase is \"{}\" and gate \"execution\" is not approved — \
writing \"{}\" is blocked. Allowed now: {}. \
Get execution approval (bee-hive) before touching source files.",
                js_disp(&phase),
                normalized,
                GATE_ALLOWED_PREFIXES.join(", ")
            )));
        }
        return Ok(WV::Allow);
    }

    if phase == Value::String("swarming".into()) {
        let env_agent = std::env::var("BEE_AGENT_NAME").ok().filter(|s| !s.is_empty());
        let agent = agent_name
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or(env_agent);
        if let Some(agent) = agent {
            let conflicts = find_conflicts(root, &agent, &[normalized.clone()])?;
            if !conflicts.is_empty() {
                let hard: Vec<&Resv> =
                    conflicts.iter().filter(|c| is_hard_conflict(c, &normalized)).collect();
                if !hard.is_empty() {
                    let held = hard
                        .iter()
                        .map(|c| {
                            format!(
                                "{} holds \"{}\" (cell {})",
                                js_disp_opt(c.agent.as_ref()),
                                c.path,
                                js_disp_opt(c.cell.as_ref())
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Ok(WV::Deny(format!(
                        "bee reservation conflict: \"{}\" is reserved by another agent — {}. \
Reserve the path first or return [BLOCKED] to the orchestrator.",
                        normalized, held
                    )));
                }
                let warned = conflicts
                    .iter()
                    .map(|c| {
                        format!(
                            "{}'s declared intent \"{}\" (cell {}) covers \"{}\"",
                            js_disp_opt(c.agent.as_ref()),
                            c.path,
                            js_disp_opt(c.cell.as_ref()),
                            normalized
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                return Ok(WV::AllowWarn(format!(
                    "bee reservation intent: {} — advisory only (kind: intent), not a hard block.",
                    warned
                )));
            }
        }
        return Ok(WV::Allow);
    }

    if !is_known_phase(&phase) {
        return Ok(WV::Deny(format!(
            "bee phase guard: phase \"{}\" is not a recognized phase — writing \"{}\" is refused rather \
than silently allowed through an unhandled state. FIX: restore a valid phase (bee state set), or if this \
is a legitimate new phase, add explicit dispatch for it in checkWrite.",
            js_disp(&phase),
            normalized
        )));
    }

    Ok(WV::Allow)
}

// ─── checkGitBashCommand (provenance: guards.mjs, full port) ───────────────

#[allow(clippy::too_many_arguments)]
pub(crate) fn check_git_bash_command(
    root: &str,
    state: &Map<String, Value>,
    command: &str,
    cwd: &str,
    session_id: Option<&str>,
    control_root_override: Option<&str>,
    emit: &mut Emit,
) -> R<Option<WV>> {
    let topo = resolve_write_topology(root, control_root_override)?;
    let record = match resolve_write_record(&topo.control_root, state, session_id, emit)? {
        RecordResolution::Fail { reason } => return Ok(Some(WV::Deny(reason))),
        RecordResolution::Ok { record, .. } => record,
    };
    let phase = match record.get("phase") {
        Some(v) if truthy(v) => v.clone(),
        _ => Value::String("idle".into()),
    };

    let tokens = tokenize(command);
    let invocation = match find_git_invocation(&tokens) {
        None => return Ok(None),
        Some(inv) => inv,
    };
    let subcommand = invocation.subcommand.clone();
    let rest = invocation.rest;

    // gc-2: concurrent-worker whole-tree denial — phase-independent.
    if let Some(classified) = classify_concurrent_tree_verb(subcommand.as_deref(), &rest) {
        match resolve_live_worker_count(root, &topo.control_root, &topo.ctx)? {
            WorkerCount::Unresolved(reason) => {
                return Ok(Some(WV::Deny(concurrent_tree_refusal(
                    &classified.verb,
                    classified.why,
                    &format!(
                        "the live-worker count could not be resolved ({}), which is treated as more than one worker",
                        reason
                    ),
                ))));
            }
            WorkerCount::Resolved(count) => {
                if count > 1 {
                    return Ok(Some(WV::Deny(concurrent_tree_refusal(
                        &classified.verb,
                        classified.why,
                        &format!("{} workers are live in this checkout", count),
                    ))));
                }
            }
        }
    }

    if !is_terminal_phase(&phase) {
        return Ok(None);
    }

    let config = read_config(Path::new(&topo.control_root))?;
    let idle_gate_on = !matches!(
        config.get("guards"),
        Some(g) if truthy(g) && g.get("idle_gate") == Some(&Value::Bool(false))
    );
    if !idle_gate_on {
        return Ok(None);
    }

    const READONLY: [&str; 12] = [
        "status", "log", "diff", "show", "rev-parse", "ls-files", "check-ignore", "merge-base",
        "rev-list", "describe", "blame", "cat-file",
    ];
    if let Some(sub) = &subcommand {
        if READONLY.contains(&sub.as_str()) {
            return Ok(None); // { allow: true } — no denial
        }
        let flag_gated: Option<&[&str]> = match sub.as_str() {
            "branch" | "tag" => Some(&["--list"]),
            "remote" => Some(&["-v", "--verbose"]),
            _ => None,
        };
        if let Some(flags) = flag_gated {
            if rest.iter().any(|t| flags.contains(&t.as_str())) {
                return Ok(None);
            }
        }
    }

    if subcommand.as_deref() == Some("push") {
        return Ok(Some(WV::Deny(intake_refusal(
            &phase,
            "`git push`",
            "git push is outward-facing and is never exempted from this gate, regardless of what it would push. ",
        ))));
    }

    const MUTATING: [&str; 15] = [
        "commit", "add", "rm", "mv", "checkout", "restore", "tag", "merge", "reset", "stash",
        "clean", "apply", "cherry-pick", "revert", "rebase",
    ];
    const PATH_RESOLVABLE: [&str; 6] = ["commit", "add", "rm", "mv", "checkout", "restore"];
    if let Some(sub) = &subcommand {
        if MUTATING.contains(&sub.as_str()) {
            let resolved = if PATH_RESOLVABLE.contains(&sub.as_str()) {
                resolve_git_mutation_paths(cwd, sub, &rest)
            } else {
                None
            };
            let paths = match resolved {
                None => {
                    return Ok(Some(WV::Deny(intake_refusal(
                        &phase,
                        &format!(
                            "running `git {}` (its changed paths could not be proved bookkeeping-only)",
                            sub
                        ),
                        "",
                    ))));
                }
                Some(p) => p,
            };
            let offending = paths
                .iter()
                .map(|p| normalize_rel(p))
                .find(|p| !under_allowed_prefix(p));
            if let Some(off) = offending {
                return Ok(Some(WV::Deny(intake_refusal(
                    &phase,
                    &format!("running `git {}` — it would change \"{}\"", sub, off),
                    "",
                ))));
            }
            return Ok(None); // { allow: true, kind: 'git-bookkeeping' }
        }
    }

    let named = match &subcommand {
        Some(s) => s.clone(),
        None => js_trim(command).to_string(),
    };
    Ok(Some(WV::Deny(intake_refusal(
        &phase,
        &format!("running `git {}`", named),
        "This git subcommand is not recognized as read-only or as a modeled bookkeeping-eligible mutation, so it is refused rather than assumed safe. ",
    ))))
}
