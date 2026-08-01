// bee hook session-init — port of hooks/bee-session-init.mjs (SessionStart:
// startup|resume|clear|compact).
//
// PORT SHAPE (latency contract: session-init fires once per session, so the
// heavy branches stay on Node for now). The .mjs's real work — session
// registration (claims.mjs createSession/heartbeatSession + workspace-store
// registerWorkspace), planned-next handoff adoption (state.mjs
// adoptHandoff), and above all the session preamble / compact capsule
// renderers (inject.mjs buildSessionPreamble, compaction.mjs
// buildCompactCapsule + appendCompactionRecord, intent.mjs resumeBlock) —
// pulls essentially the whole vendored lib surface (state, claims,
// workspace-store, inject → backlog/cells/capture/knowledge/decisions,
// compaction, intent). Those branches return Outcome::Delegate: the
// dispatcher re-runs the original Node wrapper with the same stdin, which
// is byte-identical end-to-end by construction.
//
// NATIVE surface (the fail-open spine, ported for real):
//   * no resolvable root                          → exit 0, no output
//   * root without .bee/bin/lib/state.mjs         → exit 0, no output
//     (the vendored-lib presence gate, bee-session-init.mjs:116)
//   * hookEnabled(root, "session-init") === false → exit 0, no output
//     (state.mjs hookEnabled over merged tracked+overlay config)
//
// DELEGATED branches (documented per the port contract):
//   * enabled hook on a vendored repo — every source (startup/resume/clear/
//     compact), with or without a session_id: the preamble/capsule always
//     renders, so the whole invocation delegates.
//   * corrupt .bee/config.json / config.local.json — Node's readJson warns
//     to stderr with the V8 message; never replicated natively.
//
// Delegation is decided on a hand-parsed payload BEFORE the shared adapter
// runs, so a delegated invocation performs zero native writes (including
// the adapter's coverage-gap log lines — Node produces the one canonical
// set). Native exits DO run the shared adapter first, so gap logging and
// root discovery stay byte-identical with the .mjs's readHookContext.
//
// Ported lib functions (source → private/reused fn):
//   state.mjs hookEnabled (readConfig hooks slice) → crate::state::hook_enabled
//   hooks/adapter.mjs readHookContext             → crate::hooks::adapter::read_hook_context
// (Everything else in bee-session-init.mjs's call graph is deliberately
// behind the Delegate boundary above.)

use crate::hooks::adapter::read_hook_context;
use crate::hooks::Outcome;
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::process::ExitCode;

const HOOK_NAME: &str = "session-init";

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    // Hand-parsed payload (adapter-normalization twin) so the delegate
    // decision is side-effect-free: Node must be the only writer of the
    // coverage-gap log lines on a delegated run.
    let payload = parse_payload(stdin);
    let cwd = payload
        .get("cwd")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let roots = crate::hooks::adapter::resolve_roots(&cwd);

    if let Some(root) = &roots.work_root {
        if root.join(".bee").join("bin").join("lib").join("state.mjs").is_file() {
            match crate::state::hook_enabled(root, HOOK_NAME) {
                // Corrupt config: Node's readJson warns with the V8 message.
                Err(_) => return Outcome::Delegate,
                // Enabled: registration + handoff adoption + preamble/capsule
                // — the heavy vendored-lib surface, delegated wholesale.
                Ok(true) => return Outcome::Delegate,
                Ok(false) => {}
            }
        }
    }

    // Native fail-open spine. Run the shared adapter now for byte/log
    // parity with the .mjs (coverage gaps, realpath'd root discovery).
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = &ctx.root else {
        return Outcome::Done(ExitCode::SUCCESS);
    };
    if !root.join(".bee").join("bin").join("lib").join("state.mjs").is_file() {
        return Outcome::Done(ExitCode::SUCCESS);
    }
    // Only reachable when the hook is disabled: exit 0, no output — the
    // .mjs returns right after its hookEnabled check (bee-session-init.mjs:122).
    Outcome::Done(ExitCode::SUCCESS)
}

fn parse_payload(raw: &str) -> Map<String, Value> {
    if raw.trim().is_empty() {
        return Map::new();
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(m)) => m,
        _ => Map::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn vendored_repo(dir: &Path) {
        std::fs::create_dir_all(dir.join(".bee").join("bin").join("lib")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join(".bee").join("bin").join("lib").join("state.mjs"), "// stub")
            .unwrap();
    }

    fn payload_for(dir: &Path) -> String {
        serde_json::json!({
            "hook_event_name": "SessionStart",
            "source": "startup",
            "session_id": "s1",
            "cwd": dir.to_string_lossy(),
        })
        .to_string()
    }

    #[test]
    fn missing_vendored_lib_exits_zero_natively() {
        // Root resolves (a .git dir) but the vendored state.mjs is absent —
        // the presence-gate arm.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let outcome = run(&[], &payload_for(tmp.path()));
        assert!(matches!(outcome, Outcome::Done(_)));
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
    }

    #[test]
    fn enabled_hook_delegates_to_node() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        let outcome = run(&[], &payload_for(tmp.path()));
        assert!(matches!(outcome, Outcome::Delegate));
    }

    #[test]
    fn corrupt_config_delegates_to_node() {
        let tmp = tempfile::tempdir().unwrap();
        vendored_repo(tmp.path());
        std::fs::write(tmp.path().join(".bee").join("config.json"), "{broken").unwrap();
        let outcome = run(&[], &payload_for(tmp.path()));
        assert!(matches!(outcome, Outcome::Delegate));
    }
}
