// bee hook session-init — port of hooks/bee-session-init.mjs (SessionStart:
// startup|resume|clear|compact).
//
// CUTOVER MODULE. Until the Node runtime was deleted this hook's entire real
// path was `Outcome::Delegate`: contract C2 demanded byte-identical output and
// the preamble pulls the whole vendored lib closure. C2 retired with Node, and
// a delegation with nowhere to delegate is a crash — so every branch below is
// native and no `Outcome::Delegate` remains in this file.
//
// WHAT IT DOES, per source (the .mjs's main(), in order):
//   * no resolvable root                          → exit 0, no output
//   * root without .bee/bin/lib/state.mjs         → exit 0, no output
//   * hookEnabled(root, "session-init") === false → exit 0, no output
//   * with a session_id: register/refresh the acting session — resolveContext
//     gives BOTH the control-plane re-root target and the workspace id;
//     createSession-or-heartbeat, then a best-effort registerWorkspace. EVERY
//     failure in that block is FAIL-OPEN: it logs a crash to
//     .bee/logs/hooks.jsonl and never blocks the preamble.
//   * handoff adoption, EVENT-SCOPE PIN intact (fsh-10 panel W1): only
//     "clear"/"startup" may adopt; a "startup" whose handoff.writer_session is
//     the acting session is refused SAME_SESSION_STARTUP; every other source
//     is refused WRONG_SOURCE. An adoption crash falls back to no outcome at
//     all, which renders as today's wait block.
//   * body: `compact` appends the `resume` compaction record and renders the
//     COMPACT CAPSULE; every other source renders the full session preamble.
//     (`resume` deliberately keeps the full preamble — a resume is a fresh
//     human-initiated return; only `compact` is the mid-work interruption.)
//   * lead: on `compact`/`resume` ONLY, the intent anchor is PREPENDED plus a
//     blank line. Fail-open to "" for a missing/corrupt anchor.
//   * stdout only when the output is non-empty after trim. Exit 0, always.
//
// Ported lib functions (source → fn):
//   hooks/adapter.mjs readHookContext/logCrash → crate::hooks::adapter
//   state.mjs hookEnabled                      → hook_enabled_failopen (below)
//   state.mjs resolveContext/controlRootFor    → resolve_context/control_root_for
//   state.mjs adoptHandoff                     → adopt_handoff (below)
//   claims.mjs createSession/heartbeatSession  → create_session/heartbeat_session
//   workspace-store.mjs registerWorkspace      → verbs::workspace_store
//   claims.mjs adoptClaim                      → verbs::state_group::adopt_claim
//   inject.mjs buildSessionPreamble + renderers→ hooks::session_preamble
//   compaction.mjs capsule/record + the state, config, cells, claims, intent
//     and pipeline readers this hook shares with it → hooks::compaction
//
// CORRUPT JSON never delegates: every reader here warns through
// fsutil::warn_corrupt_json and takes the fallback Node's readJson returned.

use crate::hooks::adapter::{log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::compaction::{
    append_compaction_record, build_compact_capsule, read_config_failopen, read_intent,
    read_session_record, resume_block, well_formed_id,
};
use crate::hooks::session_preamble::{build_session_preamble, read_handoff, HandoffOutcome};
use crate::hooks::Outcome;
use crate::roots::{resolve_roots_core, Resolution};
use crate::verbs::state_group::{adopt_claim, AdoptOutcome};
use crate::verbs::workspace_store::{register_workspace, RegisterSpec};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HOOK_NAME: &str = "session-init";

/// The fresh-session boundaries D1 names — the ONLY sources that may adopt.
const ADOPT_SOURCES: [&str; 2] = ["clear", "startup"];

/// compaction-hardening cz-6 (D6/D8): the ONE source whose orientation trims
/// to the compact capsule.
const CAPSULE_SOURCE: &str = "compact";

/// intent-anchor ia-1 (D4): the sources that mean "this same task is
/// continuing after its context was compressed". Deliberately DISJOINT from
/// ADOPT_SOURCES — the anchor is a prefix, never a router.
const ANCHOR_LEAD_SOURCES: [&str; 2] = ["compact", "resume"];

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = ctx.root.clone() else {
        return Outcome::Done(ExitCode::SUCCESS);
    };
    if !crate::hooks::adapter::bee_installed(&root) {
        return Outcome::Done(ExitCode::SUCCESS);
    }
    if !hook_enabled_failopen(&root) {
        return Outcome::Done(ExitCode::SUCCESS);
    }

    let session_id = trimmed(ctx.payload.get("session_id"));
    let event_source = trimmed(ctx.payload.get("source")).unwrap_or_default();

    if let Some(sid) = &session_id {
        register_acting_session(&ctx, &root, sid);
    }

    let handoff_outcome = match &session_id {
        Some(sid) => handoff_outcome_for(&ctx, &root, &event_source, sid),
        None => None,
    };

    let output = compose_output(
        &root,
        &event_source,
        session_id.as_deref(),
        handoff_outcome.as_ref(),
    );
    if !output.trim().is_empty() {
        use std::io::Write;
        let _ = std::io::stdout().write_all(output.as_bytes());
    }
    Outcome::Done(ExitCode::SUCCESS)
}

/// `typeof v === 'string' && v.trim() ? v.trim() : null`.
fn trimmed(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    }
}

/// state.mjs hookEnabled over the merged tracked+overlay config: enabled
/// unless the file explicitly carries `false`. A corrupt config file warns and
/// reads as `{}` (Node's readJson fallback), which leaves the hook ENABLED —
/// orientation is never a place to fail a session.
fn hook_enabled_failopen(root: &Path) -> bool {
    !matches!(
        read_config_failopen(root).get("hooks").and_then(|h| h.get(HOOK_NAME)),
        Some(Value::Bool(false))
    )
}

// ─────────────── the emitted block (anchor lead + body) ────────────────────

/// intent-anchor ia-1 (D4/D5): PREFIX ONLY. The anchor is prepended ahead of
/// the body on a compact/resume start, and with no anchor the emitted string
/// is the body itself.
pub(crate) fn compose_output(
    root: &Path,
    event_source: &str,
    session_id: Option<&str>,
    handoff_outcome: Option<&HandoffOutcome>,
) -> String {
    let body = session_body(root, event_source, session_id, handoff_outcome);
    let anchor = intent_lead_block(root, event_source, session_id);
    if anchor.is_empty() {
        body
    } else {
        format!("{anchor}\n\n{body}")
    }
}

/// Read + render the anchor for a compact/resume start. "" for every other
/// source, for a missing/corrupt anchor, and for any failure at all — D5:
/// with no anchor, this hook's output is byte-identical to what it printed
/// before the feature existed.
pub(crate) fn intent_lead_block(
    root: &Path,
    event_source: &str,
    session_id: Option<&str>,
) -> String {
    if !ANCHOR_LEAD_SOURCES.contains(&event_source) {
        return String::new();
    }
    match read_intent(root, session_id) {
        Some(anchor) => resume_block(&anchor),
        None => String::new(),
    }
}

/// The body under the anchor: the compact capsule on `source=compact`, the
/// full session preamble on every other source (D6/D8).
///
/// THIN CALLER, BY DECISION (D3). Nothing here decides what a capsule
/// contains or how the counts work — that lives in hooks/compaction.rs. This
/// function knows exactly two things: which source trims, and that
/// `handoff_outcome` must be carried into the capsule too (D27), because a
/// compacted session holding a planned-next handoff otherwise silently loses
/// the `- Adoption not applied:` line that says WHY it is being told to wait.
///
/// The .mjs wrapped the capsule branch in a try/catch for a repo vendored
/// before compaction.mjs shipped. There is no dynamic import left to fail:
/// both calls below are fail-open inside hooks/compaction.rs itself, so the
/// catch has no reachable trigger and is not reproduced.
fn session_body(
    root: &Path,
    event_source: &str,
    session_id: Option<&str>,
    handoff_outcome: Option<&HandoffOutcome>,
) -> String {
    if event_source == CAPSULE_SOURCE {
        // D5: SessionStart(compact) IS the `resume` event — the other half of
        // the PreCompact record, and the one that proves the session came
        // back. The append is fail-open; it never affects what renders below.
        append_compaction_record(root, "resume", session_id);
        return build_compact_capsule(root, session_id, handoff_outcome);
    }
    build_session_preamble(root, session_id, handoff_outcome)
}

// ───────────────────────── handoff adoption ────────────────────────────────

/// fsh-10 (D1, D4) — the ONLY place a planned-next handoff is ever adopted.
/// buildSessionPreamble stays a PURE renderer (PURITY PIN, panel W2); the
/// mutation lives here.
pub(crate) fn handoff_outcome_for(
    ctx: &HookContext,
    root: &Path,
    event_source: &str,
    session_id: &str,
) -> Option<HandoffOutcome> {
    let handoff = read_handoff(root)?;
    if !matches!(handoff.get("kind"), Some(Value::String(k)) if k == "planned-next") {
        return None;
    }
    if !ADOPT_SOURCES.contains(&event_source) {
        let shown = if event_source.is_empty() { "unknown" } else { event_source };
        return Some(HandoffOutcome {
            ok: false,
            code: Some("WRONG_SOURCE".to_string()),
            reason: Some(format!(
                "a planned-next handoff never auto-adopts on source \"{shown}\" — only \"clear\"/\"startup\" qualify (D1)."
            )),
            next_cell: None,
        });
    }
    // A "startup" whose handoff was written by the acting session is ALSO
    // refused without adopting: that is not a fresh-session boundary, just the
    // same session starting up again.
    if event_source == "startup"
        && handoff.get("writer_session") == Some(&Value::String(session_id.to_string()))
    {
        return Some(HandoffOutcome {
            ok: false,
            code: Some("SAME_SESSION_STARTUP".to_string()),
            reason: Some(
                "the acting session is the same session that wrote this handoff — not a fresh-session boundary, never self-adopted."
                    .to_string(),
            ),
            next_cell: None,
        });
    }
    match adopt_handoff(root, session_id) {
        Ok(outcome) => Some(outcome),
        // fail-open: an adoption crash never blocks the preamble; render as if
        // adoption were never attempted (today's wait block).
        Err(message) => {
            log_crash(Some(root), HOOK_NAME, &message, ctx.source);
            None
        }
    }
}

/// state.mjs adoptHandoff(root, sessionId) — clear-after-adopt with idempotent
/// recovery. `Err` is the throw the .mjs's own catch would logCrash.
fn adopt_handoff(root: &Path, session_id: &str) -> Result<HandoffOutcome, String> {
    let refusal = |code: &str, reason: String| HandoffOutcome {
        ok: false,
        code: Some(code.to_string()),
        reason: Some(reason),
        next_cell: None,
    };
    let Some(handoff) = read_handoff(root) else {
        return Ok(refusal("NO_HANDOFF", "no .bee/HANDOFF.json to adopt.".to_string()));
    };
    let kind = match handoff.get("kind") {
        Some(Value::String(k)) => k.clone(),
        _ => "undefined".to_string(),
    };
    if kind != "planned-next" {
        return Ok(refusal(
            "NOT_PLANNED_NEXT",
            format!(
                "handoff kind \"{kind}\" is not \"planned-next\" — a pause handoff is never adopted, it must be surfaced and WAITED on (D1)."
            ),
        ));
    }
    let next_cell = match handoff.get("next_cell") {
        Some(Value::String(s)) => s.trim().to_string(),
        _ => String::new(),
    };
    if next_cell.is_empty() {
        return Ok(refusal(
            "MALFORMED",
            "planned-next handoff has no next_cell to adopt.".to_string(),
        ));
    }

    let control = control_root_for(root);
    match adopt_claim(&control, &next_cell, session_id) {
        // adoptClaim's own typed failure (GATE_HELD / NOT_FOUND) — propagated
        // as-is; neither the claim nor the handoff has been touched. The Rust
        // port carries the reason but not the code, so the code is recovered
        // from the one sentence each refusal owns (only `reason` renders).
        Ok(AdoptOutcome::Fail { reason }) => {
            let code = if reason.contains("gated by another in-flight") {
                "GATE_HELD"
            } else {
                "NOT_FOUND"
            };
            Ok(refusal(code, reason))
        }
        Ok(AdoptOutcome::Adopted { .. }) => {
            let _ = std::fs::remove_file(root.join(".bee").join("HANDOFF.json")); // rmSync force
            Ok(HandoffOutcome {
                ok: true,
                code: None,
                reason: None,
                next_cell: Some(next_cell),
            })
        }
        Err(_) => Err(format!(
            "adoptHandoff: adopting claim \"{next_cell}\" for session \"{session_id}\" failed."
        )),
    }
}

// ───────────────── session registration (msn-19, D2/D3) ────────────────────

/// state.mjs resolveContext's slice this hook and hooks/compaction.rs consume.
pub(crate) struct BeeContext {
    pub(crate) control_root: Option<PathBuf>,
    pub(crate) workspace_root: Option<PathBuf>,
    pub(crate) workspace_id: Option<String>,
    pub(crate) worktree_id: Option<String>,
}

/// state.mjs resolveContext(cwd), reusing crate::roots's resolveRootsCore
/// port. `Err` is the WorktreeLinkInvalidError the .mjs throws.
///
/// workspaceId follows worktree-store.mjs decideWorktreeStore: a linked-valid
/// worktree whose git-verified id is GRANTED in the main store carries its own
/// id, everything else is "main". Grants are already applied by
/// resolveRootsCore (storeRoot === workRoot exactly when granted).
pub(crate) fn resolve_context(cwd: &Path) -> Result<BeeContext, String> {
    match resolve_roots_core(cwd) {
        Resolution::Unresolved => Ok(BeeContext {
            control_root: None,
            workspace_root: None,
            workspace_id: None,
            worktree_id: None,
        }),
        Resolution::Ordinary { work_root, .. } => Ok(BeeContext {
            control_root: Some(work_root.clone()),
            workspace_root: Some(work_root),
            workspace_id: Some("main".to_string()),
            worktree_id: None,
        }),
        Resolution::LinkedValid { store_root, work_root, id, main_root } => {
            let granted = store_root == work_root;
            Ok(BeeContext {
                control_root: Some(main_root),
                workspace_root: Some(work_root),
                workspace_id: Some(if granted { id.clone() } else { "main".to_string() }),
                worktree_id: Some(id),
            })
        }
        Resolution::LinkInvalid { message } => Err(format!("WorktreeLinkInvalidError: {message}")),
    }
}

/// state.mjs controlRootFor(root) — resolveContext(root).controlRoot ?? root.
///
/// DIVERGENCE, named: Node's controlRootFor THROWS on a broken worktree link
/// and every caller in compaction.mjs catches that into "no record". Here the
/// throw resolves to `root` itself instead, which is the same fallback an
/// unresolvable context takes — the fail-open direction, and never a read of
/// somebody else's store.
pub(crate) fn control_root_for(root: &Path) -> PathBuf {
    match resolve_context(root) {
        Ok(ctx) => ctx.control_root.unwrap_or_else(|| root.to_path_buf()),
        Err(_) => root.to_path_buf(),
    }
}

/// The whole registration block, FAIL-OPEN end to end (the .mjs's outer
/// try/catch): every failure logs a crash to .bee/logs/hooks.jsonl and the
/// preamble still renders.
fn register_acting_session(ctx: &HookContext, root: &Path, session_id: &str) {
    let context = match resolve_context(root) {
        Ok(context) => context,
        Err(message) => {
            log_crash(Some(root), HOOK_NAME, &message, ctx.source);
            return;
        }
    };
    // msn-19 (D2/D3): sessions/claims are control-plane — resolveContext gives
    // BOTH the re-root target and the workspaceId to stamp onto the record,
    // from one git-worktree classification. An unresolvable context falls back
    // to `root` itself and a null workspaceId, never a throw.
    let session_root = context.control_root.clone().unwrap_or_else(|| root.to_path_buf());
    // hardening-1-7-10 D5 (Codex session bridge): persist the real hook
    // payload's transcript_path onto the session record at creation time, so
    // recovery can resolve THIS session's transcript from the stored path
    // instead of guessing via Claude's encoded-layout math.
    let transcript_path = match ctx.payload.get("transcript_path") {
        Some(Value::String(s)) => Some(s.as_str()),
        _ => None,
    };
    match create_session(
        &session_root,
        session_id,
        transcript_path,
        context.workspace_id.as_deref(),
    ) {
        Err(message) => {
            log_crash(Some(root), HOOK_NAME, &message, ctx.source);
            return;
        }
        Ok(true) => {}
        Ok(false) => {
            if let Err(message) = heartbeat_session(&session_root, session_id) {
                log_crash(Some(root), HOOK_NAME, &message, ctx.source);
                return;
            }
        }
    }
    // msn-19: "Main checkout auto-registered lazily (first touch)" — this hook
    // is the natural first-touch point for BOTH the main checkout and a linked
    // worktree (a worktree created via createFeatureWorktree already
    // registered itself; this call is then an idempotent no-op).
    // Best-effort: never blocks the preamble.
    if let Some(workspace_id) = &context.workspace_id {
        let workspace_root =
            context.workspace_root.clone().unwrap_or_else(|| root.to_path_buf());
        let spec = RegisterSpec {
            id: workspace_id,
            kind: if context.worktree_id.is_some() { "worktree" } else { "main" },
            root: &workspace_root.to_string_lossy(),
            branch: None,
            base_sha: None,
        };
        if register_workspace(&session_root, spec, &now_iso()).is_err() {
            log_crash(
                Some(root),
                HOOK_NAME,
                &format!("registerWorkspace(\"{workspace_id}\") failed."),
                ctx.source,
            );
        }
    }
}

/// claims.mjs createSession(root, {id, transcript_path, workspace_id}).
/// `Ok(true)` = created, `Ok(false)` = SESSION_EXISTS (the typed failure the
/// caller answers with a heartbeat), `Err` = the throw.
fn create_session(
    root: &Path,
    id: &str,
    transcript_path: Option<&str>,
    workspace_id: Option<&str>,
) -> Result<bool, String> {
    // requireId(id, 'session id')
    if id.trim().is_empty() {
        return Err("session id is required.".to_string());
    }
    if !well_formed_id(id) {
        return Err("session id must be a plain id (no path separators).".to_string());
    }
    let session_id = id.trim();
    let dir = root.join(".bee").join("sessions");
    std::fs::create_dir_all(&dir).map_err(|e| format!("ensureDir({}): {e}", dir.display()))?;

    let now = now_iso();
    let mut session = Map::new();
    session.insert("id".into(), Value::String(session_id.to_string()));
    session.insert("started_at".into(), Value::String(now.clone()));
    session.insert("last_heartbeat".into(), Value::String(now));
    // Both optional and OMITTED when absent/blank — "never write a placeholder".
    if let Some(path) = transcript_path.map(str::trim).filter(|s| !s.is_empty()) {
        session.insert("transcript_path".into(), Value::String(path.to_string()));
    }
    if let Some(workspace) = workspace_id.map(str::trim).filter(|s| !s.is_empty()) {
        session.insert("workspace_id".into(), Value::String(workspace.to_string()));
    }

    let file = dir.join(format!("{session_id}.json"));
    let body = format!("{}\n", crate::jsjson::stringify_pretty(&Value::Object(session)));
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&file) {
        Ok(mut handle) => {
            use std::io::Write;
            handle
                .write_all(body.as_bytes())
                .map_err(|e| format!("createSession write {}: {e}", file.display()))?;
            Ok(true)
        }
        // flag 'wx': EEXIST is the typed SESSION_EXISTS failure, not a throw.
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(format!("createSession open {}: {e}", file.display())),
    }
}

/// claims.mjs heartbeatSession — the write is gated by the same bounded
/// `sessions` store lock the sweeper holds (15 attempts, 20ms apart, NEVER an
/// unbounded wait). LOCK_BUSY and SESSION_MISSING are typed RESULTS, not
/// throws, so both read as Ok here; only a real write failure is an error.
fn heartbeat_session(control_root: &Path, session_id: &str) -> Result<(), String> {
    const ATTEMPTS: u32 = 15;
    const DELAY_MS: u64 = 20;
    let mut acquired = None;
    for attempt in 0..ATTEMPTS {
        match crate::lock::acquire_store_lock_once(control_root, "sessions") {
            crate::lock::AcquireOnce::Acquired(guard) => {
                acquired = Some(guard);
                break;
            }
            crate::lock::AcquireOnce::Busy { .. } => {
                if attempt + 1 < ATTEMPTS {
                    std::thread::sleep(std::time::Duration::from_millis(DELAY_MS));
                }
            }
        }
    }
    let Some(mut guard) = acquired else {
        return Ok(()); // LOCK_BUSY: a typed refusal, never a throw
    };
    let result = (|| -> Result<(), String> {
        // D10a: the read lives INSIDE the lock.
        let Some(mut session) = read_session_record(control_root, session_id) else {
            return Ok(()); // SESSION_MISSING: a typed refusal
        };
        if session.get("id") != Some(&Value::String(session_id.trim().to_string())) {
            return Ok(());
        }
        session.insert("last_heartbeat".into(), Value::String(now_iso()));
        let file = control_root
            .join(".bee")
            .join("sessions")
            .join(format!("{}.json", session_id.trim()));
        crate::fsutil::write_json_atomic(&file, &Value::Object(session))
            .map_err(|e| format!("heartbeatSession write {}: {e}", file.display()))
    })();
    guard.release();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A repo where bee IS installed — the marker the activation gate reads.
    fn vendored_repo(dir: &Path) {
        std::fs::create_dir_all(dir.join(".bee")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join(".bee").join("onboarding.json"), "{}\n").unwrap();
    }

    fn payload_for(dir: &Path) -> String {
        payload_with(dir, "startup", Some("s1"))
    }

    fn payload_with(dir: &Path, source: &str, session: Option<&str>) -> String {
        let mut m = serde_json::Map::new();
        m.insert("hook_event_name".into(), json!("SessionStart"));
        m.insert("source".into(), json!(source));
        m.insert("cwd".into(), json!(dir.to_string_lossy()));
        if let Some(s) = session {
            m.insert("session_id".into(), json!(s));
        }
        Value::Object(m).to_string()
    }

    fn hook_ctx(dir: &Path) -> HookContext {
        read_hook_context(HOOK_NAME, &[], &payload_for(dir))
    }

    fn write_handoff(dir: &Path, record: Value) {
        std::fs::create_dir_all(dir.join(".bee")).unwrap();
        std::fs::write(
            dir.join(".bee").join("HANDOFF.json"),
            serde_json::to_string(&record).unwrap(),
        )
        .unwrap();
    }

    fn write_intent(dir: &Path, key: &str, request: &str) {
        let intent = dir.join(".bee").join("intent");
        std::fs::create_dir_all(&intent).unwrap();
        std::fs::write(
            intent.join(format!("{key}.json")),
            serde_json::to_string(&json!({ "request": request, "acceptance": "done" })).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn missing_vendored_lib_exits_zero_natively() {
        // Root resolves (a .git dir) but the vendored state.mjs is absent —
        // the presence-gate arm.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let outcome = run(&[], &payload_for(tmp.path()));
        assert!(matches!(outcome, Outcome::Done(_)));
        assert!(!tmp.path().join(".bee").join("sessions").exists(), "nothing ran");
    }

    #[test]
    fn disabled_hook_exits_zero_natively() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            r#"{"hooks":{"session-init":false}}"#,
        )
        .unwrap();
        let outcome = run(&[], &payload_for(tmp.path()));
        assert!(matches!(outcome, Outcome::Done(_)));
        assert!(
            !tmp.path().join(".bee").join("sessions").exists(),
            "a disabled hook registers nothing"
        );
    }

    /// Was `enabled_hook_delegates_to_node`. There is no Node to delegate to:
    /// the enabled hook now runs its whole path natively and exits 0. The
    /// observable proof that it went native rather than short-circuiting is
    /// the session record it registers.
    #[test]
    fn enabled_hook_runs_natively_and_exits_zero() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        let outcome = run(&[], &payload_for(tmp.path()));
        assert!(matches!(outcome, Outcome::Done(_)));
        let record = tmp.path().join(".bee").join("sessions").join("s1.json");
        assert!(record.is_file(), "the acting session must be registered natively");
        let stored: Value = serde_json::from_str(&std::fs::read_to_string(&record).unwrap()).unwrap();
        assert_eq!(stored["id"], json!("s1"));
        assert_eq!(stored["workspace_id"], json!("main"));
        assert!(stored["started_at"].is_string());
        // A second run finds the record and heartbeats instead of failing:
        // createSession's SESSION_EXISTS is a typed result, never a throw.
        assert!(matches!(run(&[], &payload_for(tmp.path())), Outcome::Done(_)));
        let after: Value = serde_json::from_str(&std::fs::read_to_string(&record).unwrap()).unwrap();
        assert_eq!(after["started_at"], stored["started_at"], "never recreated");
        assert!(after["last_heartbeat"].is_string());
    }

    /// Was `corrupt_config_delegates_to_node`. Node's readJson warned and
    /// returned the fallback; so does this, and the hook still runs.
    #[test]
    fn corrupt_config_runs_fail_open_and_warns() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        std::fs::write(tmp.path().join(".bee").join("config.json"), "{broken").unwrap();
        // A corrupt config reads as `{}`, so the hook stays ENABLED…
        assert!(hook_enabled_failopen(tmp.path()));
        // …and the whole invocation still completes and exits 0.
        let outcome = run(&[], &payload_for(tmp.path()));
        assert!(matches!(outcome, Outcome::Done(_)));
        assert!(tmp.path().join(".bee").join("sessions").join("s1.json").is_file());
    }

    #[test]
    fn the_emitted_block_never_names_the_node_runtime() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        write_intent(tmp.path(), "default", "ship the port");
        for source in ["startup", "resume", "clear", "compact"] {
            let text = compose_output(tmp.path(), source, Some("s1"), None);
            assert!(!text.contains(".mjs"), "source={source} leaked the Node runtime: {text}");
        }
    }

    // ── the three handoff-adoption arms ────────────────────────────────────

    #[test]
    fn a_non_adopting_source_is_refused_wrong_source() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        write_handoff(
            tmp.path(),
            json!({"kind": "planned-next", "writer_session": "other", "next_cell": "c1"}),
        );
        let ctx = hook_ctx(tmp.path());
        for source in ["compact", "resume", ""] {
            let outcome = handoff_outcome_for(&ctx, tmp.path(), source, "s1").unwrap();
            assert!(!outcome.ok);
            assert_eq!(outcome.code.as_deref(), Some("WRONG_SOURCE"));
            let shown = if source.is_empty() { "unknown" } else { source };
            assert_eq!(
                outcome.reason.as_deref(),
                Some(
                    format!(
                        "a planned-next handoff never auto-adopts on source \"{shown}\" — only \"clear\"/\"startup\" qualify (D1)."
                    )
                    .as_str()
                )
            );
        }
        // Nothing was adopted: the handoff is still on disk.
        assert!(tmp.path().join(".bee").join("HANDOFF.json").is_file());
    }

    #[test]
    fn a_startup_by_the_writing_session_is_refused_same_session_startup() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        write_handoff(
            tmp.path(),
            json!({"kind": "planned-next", "writer_session": "s1", "next_cell": "c1"}),
        );
        let ctx = hook_ctx(tmp.path());
        let outcome = handoff_outcome_for(&ctx, tmp.path(), "startup", "s1").unwrap();
        assert!(!outcome.ok);
        assert_eq!(outcome.code.as_deref(), Some("SAME_SESSION_STARTUP"));
        assert_eq!(
            outcome.reason.as_deref(),
            Some("the acting session is the same session that wrote this handoff — not a fresh-session boundary, never self-adopted.")
        );
        assert!(tmp.path().join(".bee").join("HANDOFF.json").is_file());
    }

    #[test]
    fn a_fresh_boundary_adopts_the_carried_claim_and_clears_the_handoff() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        let claims = tmp.path().join(".bee").join("claims");
        std::fs::create_dir_all(&claims).unwrap();
        std::fs::write(
            claims.join("c1.json"),
            serde_json::to_string(&json!({"cell": "c1", "session": "writer"})).unwrap(),
        )
        .unwrap();
        write_handoff(
            tmp.path(),
            json!({"kind": "planned-next", "writer_session": "writer", "next_cell": "c1"}),
        );
        let ctx = hook_ctx(tmp.path());
        let outcome = handoff_outcome_for(&ctx, tmp.path(), "clear", "s1").unwrap();
        assert!(outcome.ok, "a clear IS a fresh-session boundary");
        assert_eq!(outcome.next_cell.as_deref(), Some("c1"));
        let claim: Value =
            serde_json::from_str(&std::fs::read_to_string(claims.join("c1.json")).unwrap()).unwrap();
        assert_eq!(claim["session"], json!("s1"));
        assert_eq!(claim["adopted_from"], json!("writer"));
        assert!(
            !tmp.path().join(".bee").join("HANDOFF.json").exists(),
            "adoption clears the handoff"
        );
    }

    #[test]
    fn a_pause_handoff_never_produces_an_outcome_at_all() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        write_handoff(tmp.path(), json!({"kind": "pause", "next_action": "ask the human"}));
        let ctx = hook_ctx(tmp.path());
        for source in ["startup", "clear", "compact", "resume"] {
            assert!(
                handoff_outcome_for(&ctx, tmp.path(), source, "s1").is_none(),
                "source={source}"
            );
        }
    }

    // ── body selection + the anchor lead ───────────────────────────────────

    #[test]
    fn compact_renders_the_capsule_and_appends_the_resume_record() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        let body = compose_output(tmp.path(), "compact", Some("s1"), None);
        assert!(body.contains("compact capsule (source=compact)"), "{body}");
        let log = tmp.path().join(".bee").join("logs").join("compaction.jsonl");
        let rows = std::fs::read_to_string(&log).unwrap();
        let record: Value = serde_json::from_str(rows.trim()).unwrap();
        assert_eq!(record["event"], json!("resume"));
        assert_eq!(record["session"], json!("s1"));
    }

    #[test]
    fn startup_renders_the_full_preamble_and_writes_no_compaction_record() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        let body = compose_output(tmp.path(), "startup", Some("s1"), None);
        assert!(
            !body.contains("compact capsule (source=compact)"),
            "startup keeps the full preamble: {body}"
        );
        assert!(
            !tmp.path().join(".bee").join("logs").join("compaction.jsonl").exists(),
            "only source=compact appends the resume half of the record"
        );
        // `resume` is deliberately NOT the capsule source either (D6/D8).
        let resumed = compose_output(tmp.path(), "resume", Some("s1"), None);
        assert!(!resumed.contains("compact capsule (source=compact)"), "{resumed}");
    }

    #[test]
    fn the_anchor_leads_on_compact_and_resume_and_on_nothing_else() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        write_intent(tmp.path(), "default", "port the hook to Rust");
        for source in ["compact", "resume"] {
            let lead = intent_lead_block(tmp.path(), source, Some("s1"));
            assert!(lead.starts_with("## INTENT ANCHOR"), "source={source}: {lead}");
            assert!(lead.contains("port the hook to Rust"), "source={source}");
            let out = compose_output(tmp.path(), source, Some("s1"), None);
            assert!(out.starts_with(&lead), "the anchor LEADS on source={source}");
            assert!(out[lead.len()..].starts_with("\n\n"), "one blank line after the anchor");
        }
        for source in ["startup", "clear", ""] {
            assert_eq!(
                intent_lead_block(tmp.path(), source, Some("s1")),
                "",
                "source={source} must not lead with the anchor"
            );
        }
    }

    #[test]
    fn a_missing_or_corrupt_anchor_leads_with_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        assert_eq!(intent_lead_block(tmp.path(), "compact", Some("s1")), "");
        let intent = tmp.path().join(".bee").join("intent");
        std::fs::create_dir_all(&intent).unwrap();
        std::fs::write(intent.join("default.json"), "{broken").unwrap();
        assert_eq!(
            intent_lead_block(tmp.path(), "compact", Some("s1")),
            "",
            "a corrupt anchor is fail-open, never a failed session"
        );
    }

    #[test]
    fn an_ordinary_checkout_resolves_to_itself_with_the_main_workspace_id() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        let context = resolve_context(tmp.path()).unwrap();
        assert_eq!(context.workspace_id.as_deref(), Some("main"));
        assert!(context.worktree_id.is_none());
        assert_eq!(control_root_for(tmp.path()), context.control_root.unwrap());
    }
}
