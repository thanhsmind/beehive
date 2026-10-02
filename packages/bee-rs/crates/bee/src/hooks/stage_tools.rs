// bee hook stage-tools — advisory tool gating based on current stage policy (D4, D12)

use crate::hooks::adapter::{read_hook_context, HookContext};
use crate::hooks::write_guard::{
    is_gated_phase, is_known_phase, read_state, resolve_write_record, Emit, RecordResolution,
};
use crate::hooks::Outcome;
use crate::state::hook_enabled;
use serde_json::{json, Value};
use std::io::Write;
use std::process::ExitCode;

pub const HOOK_NAME: &str = "stage-tools";

pub(crate) const READ_ONLY_TOOLS: [&str; 2] = ["read", "bash"];
pub(crate) const BASE_TOOL_SET: [&str; 10] = [
    "read", "bash", "edit", "write", "find", "grep", "ls", "powershell", "codemode", "tool_search",
];
pub(crate) const FULL_TOOL_SET: [&str; 14] = [
    "read", "bash", "edit", "write", "find", "grep", "ls", "powershell", "codemode", "tool_search",
    "bee_dispatch", "bee_advisor", "bee_steer", "verdict",
];
pub(crate) const LEADER_TOOL_SET: [&str; 11] = [
    "read", "bash", "find", "grep", "ls", "powershell", "codemode", "tool_search", "bee_dispatch",
    "bee_advisor", "bee_steer",
];
pub(crate) const GATED_TOOL_SET: [&str; 3] = ["read", "bash", "bee_advisor"];

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    let argv = argv.to_vec();
    let stdin = stdin.to_string();
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ctx = read_hook_context(HOOK_NAME, &argv, &stdin);
        run_inner(&ctx)
    }));
    match res {
        Ok(code) => Outcome::Done(code),
        Err(_) => Outcome::Done(ExitCode::SUCCESS), // advisory hooks fail open cleanly
    }
}

/// The whole phase-to-tools rule, in one place.
///
/// The tool gate keeps no phase list of its own. It asks the write guard, which
/// is the authority on when a write is allowed, and the guard sorts phases into
/// four groups (`write_guard/checks.rs`):
///
/// - `exploring` and `planning` — gated: source writes are refused until the
///   execution gate is approved (`is_gated_phase`). Narrow.
/// - `idle` and `compounding-complete` — terminal: writes are refused OUTSIDE
///   `.bee/`, `docs/`, `plans/` and `AGENTS.md` while `guards.idle_gate` is on
///   (`is_terminal_phase`, `GATE_ALLOWED_PREFIXES_INTAKE`). PARTIAL, and the
///   tool list cannot say "only under these paths". We keep the write tools, so
///   the docs-lane edit the guard allows here still works; a source write gets
///   the guard's own named intake refusal, which carries its remedy. The
///   opposite choice — narrowing — would block a write the guard permits.
/// - any phase the guard does not recognize — every write is refused outright
///   (`is_known_phase`). Narrow: a typo or a phase from a newer bee must not
///   hand the model tools whose every call the guard will deny.
/// - everything else (`swarming`, `reviewing`, `scribing`, `compounding`,
///   `grooming`) — open.
///
/// A second hand-kept list here once named only the four open phases and
/// drifted from the guard, which cost `grooming` its write tools.
pub(crate) fn allowed_tools_for(
    phase: &str,
    gate_approved: bool,
    lane: Option<&str>,
    worker: bool,
    harness_workflow: bool,
) -> &'static [&'static str] {
    let full: &'static [&'static str] = if harness_workflow { &FULL_TOOL_SET } else { &BASE_TOOL_SET };
    if harness_workflow && worker {
        return full;
    }
    if !gate_approved {
        let phase = Value::String(phase.to_string());
        if is_gated_phase(&phase) {
            return if harness_workflow { &GATED_TOOL_SET } else { &READ_ONLY_TOOLS };
        }
        if !is_known_phase(&phase) {
            return &READ_ONLY_TOOLS;
        }
        return full;
    }
    if harness_workflow
        && phase == "swarming"
        && matches!(lane, Some("small" | "standard" | "high-risk"))
    {
        return &LEADER_TOOL_SET;
    }
    full
}

fn run_inner(ctx: &HookContext) -> ExitCode {
    let Some(verdict) = build_verdict(ctx) else {
        return ExitCode::SUCCESS;
    };
    let stdout = verdict.to_string();
    let _ = std::io::stdout().write_all(stdout.as_bytes());
    ExitCode::SUCCESS
}

/// `None` means "stay silent and fail open" — the hook is advisory, so every
/// missing precondition ends the run without a verdict and without an error.
fn build_verdict(ctx: &HookContext) -> Option<Value> {
    let root_pb = ctx.root.as_ref()?;
    if !crate::hooks::adapter::bee_installed(root_pb) {
        return None;
    }
    if !hook_enabled(root_pb, HOOK_NAME) {
        return None;
    }

    let store_root_pb = ctx.store_root.clone().unwrap_or_else(|| root_pb.clone());
    let control_root_pb = ctx.control_root.clone().unwrap_or_else(|| store_root_pb.clone());
    let control_root = control_root_pb.to_string_lossy().into_owned();

    let state = read_state(&store_root_pb).ok()?;

    let session_id = ctx.payload.get("session_id").and_then(Value::as_str);
    let mut emit = Emit::default();

    let record = match resolve_write_record(&control_root, &state, session_id, &mut emit) {
        Ok(RecordResolution::Ok { record, .. }) => record,
        _ => state,
    };

    let phase = match record.get("phase") {
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim(),
        _ => "idle",
    };

    let gate_approved = matches!(
        record.get("approved_gates").and_then(|g| g.get("execution")),
        Some(Value::Bool(true))
    );
    let lane = record
        .get("route")
        .and_then(|r| r.get("lane"))
        .and_then(Value::as_str);
    let harness_workflow = crate::state::read_config_raw(&store_root_pb).get("pi_harness_workflow")
        != Some(&Value::Bool(false));
    let worker = crate::hooks::herding_worker_marker_set();
    let allowed = allowed_tools_for(phase, gate_approved, lane, worker, harness_workflow);
    let leads = allowed == LEADER_TOOL_SET;
    let full: &'static [&'static str] = if harness_workflow { &FULL_TOOL_SET } else { &BASE_TOOL_SET };
    let execution_is_open = allowed == full;
    let allowed_tools: Vec<String> = allowed.iter().map(|s| s.to_string()).collect();

    let stage_name = phase;
    let user_sentence = if execution_is_open {
        format!("bee stage gate: full tool set active for stage \"{stage_name}\".")
    } else if leads {
        format!(
            "bee stage gate: leader tools for stage \"{stage_name}\": edit and write removed, start workers with bee_dispatch. Use /bee-tools-reopen to restore all tools."
        )
    } else {
        format!(
            "bee stage gate: active tools narrowed for stage \"{stage_name}\" (allowed: {}). Use /bee-tools-reopen to restore all tools.",
            allowed_tools.join(", ")
        )
    };

    let model_sentence = if execution_is_open {
        format!("Notice: Full tool set is available for bee stage \"{stage_name}\".")
    } else if leads {
        format!(
            "Notice: You lead bee stage \"{stage_name}\": edit and write are removed. Dispatch each cell to a worker with bee_dispatch; do not write source inline or fall back to bash redirection. If you require these tools, ask the user to run /bee-tools-reopen."
        )
    } else {
        format!(
            "Notice: Tools narrowed by bee stage policy for stage \"{stage_name}\": off-stage tools removed. Do not attempt to use them or fall back to bash redirection. If you require these tools, ask the user to run /bee-tools-reopen."
        )
    };

    Some(json!({
        "stage": stage_name,
        "stage_name": stage_name,
        "allowed_tools": allowed_tools,
        "user_message": user_sentence,
        "user_notice": user_sentence,
        "model_message": model_sentence,
        "model_notice": model_sentence,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::write_guard::KNOWN_PHASES;
    use tempfile::tempdir;

    fn setup_bee_repo() -> tempfile::TempDir {
        let dir = tempdir().expect("tempdir");
        let bee_dir = dir.path().join(".bee");
        std::fs::create_dir_all(&bee_dir).expect("create .bee dir");
        std::fs::write(
            bee_dir.join("onboarding.json"),
            r#"{"completed": true}"#,
        )
        .expect("write onboarding.json");
        dir
    }

    #[test]
    fn stage_tools_returns_read_only_in_planning_phase_without_gate() {
        let dir = setup_bee_repo();
        std::fs::write(
            dir.path().join(".bee").join("state.json"),
            r#"{"phase": "planning", "approved_gates": {"execution": false}}"#,
        )
        .expect("write state.json");

        let stdin = json!({
            "cwd": dir.path().to_str().unwrap(),
            "session_id": "test-sess",
        })
        .to_string();

        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        assert_eq!(run_inner(&ctx), ExitCode::SUCCESS);
    }

    /// The literal phase-to-tools table, spelled out rather than derived, so a
    /// wrong rule cannot agree with a wrong expectation. A phase added to the
    /// write guard and not to this table fails
    /// `stage_tools_table_covers_every_phase_the_write_guard_knows`.
    const EXPECTED_WITHOUT_GATE: [(&str, bool); 9] = [
        // (phase, narrowed to read+bash)
        ("exploring", true),           // gated: source writes refused
        ("planning", true),            // gated: source writes refused
        ("swarming", false),           // open
        ("reviewing", false),          // open
        ("scribing", false),           // open
        ("compounding", false),        // open
        ("grooming", false),           // open — the phase this fix restored
        ("idle", false),               // terminal: docs/.bee/plans/AGENTS.md only
        ("compounding-complete", false), // terminal: same partial allowance
    ];

    struct HerdingWorkerEnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        prior: Option<std::ffi::OsString>,
    }

    impl HerdingWorkerEnvGuard {
        fn non_worker() -> Self {
            let lock = crate::hooks::herding_env_lock();
            let prior = std::env::var_os("BEE_HERDING_WORKER");
            // SAFETY: herding_env_lock serializes every test that touches this var.
            unsafe { std::env::remove_var("BEE_HERDING_WORKER") };
            Self { _lock: lock, prior }
        }
    }

    impl Drop for HerdingWorkerEnvGuard {
        fn drop(&mut self) {
            match &self.prior {
                // SAFETY: herding_env_lock serializes every test that touches this var.
                Some(v) => unsafe { std::env::set_var("BEE_HERDING_WORKER", v) },
                // SAFETY: herding_env_lock serializes every test that touches this var.
                None => unsafe { std::env::remove_var("BEE_HERDING_WORKER") },
            }
        }
    }

    #[test]
    fn stage_tools_matches_the_write_guards_phase_table() {
        for (phase, expect_narrow) in EXPECTED_WITHOUT_GATE {
            let allowed = allowed_tools_for(phase, false, None, false, true);
            assert_eq!(
                allowed == GATED_TOOL_SET,
                expect_narrow,
                "phase {phase:?}: expected narrowed={expect_narrow}, got {allowed:?}"
            );
        }
    }

    #[test]
    fn stage_tools_table_covers_every_phase_the_write_guard_knows() {
        // The guard owns the phase list. If it grows one, this table must grow
        // with it rather than let the new phase fall through a default.
        for phase in KNOWN_PHASES {
            assert!(
                EXPECTED_WITHOUT_GATE.iter().any(|(p, _)| p == phase),
                "phase {phase:?} is known to the write guard but absent from the tool-gate table"
            );
        }
        assert_eq!(EXPECTED_WITHOUT_GATE.len(), KNOWN_PHASES.len());
    }

    #[test]
    fn stage_tools_opens_every_phase_once_the_execution_gate_is_approved() {
        for phase in KNOWN_PHASES {
            assert_eq!(
                allowed_tools_for(phase, true, None, false, true),
                FULL_TOOL_SET,
                "phase {phase:?} with an approved execution gate must carry the full tool set"
            );
        }
    }

    #[test]
    fn stage_tools_keeps_pi_codemode_and_tool_search_once_writes_are_allowed() {
        for (phase, gate) in [("planning", true), ("swarming", false), ("grooming", false)] {
            let allowed = allowed_tools_for(phase, gate, None, false, true);
            for tool in ["codemode", "tool_search"] {
                assert!(allowed.contains(&tool), "{phase:?} gate={gate} must keep {tool:?}");
            }
        }
        assert!(!READ_ONLY_TOOLS.contains(&"codemode"));
        assert!(!READ_ONLY_TOOLS.contains(&"tool_search"));
    }

    #[test]
    fn stage_tools_narrows_an_unknown_phase_because_the_guard_denies_every_write_there() {
        // write_guard/checks.rs refuses EVERY write under an unrecognized phase
        // ("bee phase guard: phase ... is not a recognized phase"). Handing the
        // model write tools there spends its turns on calls that always deny.
        assert_eq!(allowed_tools_for("wayfinding", false, None, false, true), READ_ONLY_TOOLS);
        assert_eq!(allowed_tools_for("", false, None, false, true), READ_ONLY_TOOLS);
    }

    #[test]
    fn stage_tools_reads_the_execution_gate_out_of_the_state_record() {
        // Guards the JSON path itself: a wrong key name here would leave every
        // phase narrowed and no table test would notice.
        let _env = HerdingWorkerEnvGuard::non_worker();
        let dir = setup_bee_repo();
        let state_path = dir.path().join(".bee").join("state.json");
        let stdin = json!({"cwd": dir.path().to_str().unwrap()}).to_string();

        std::fs::write(
            &state_path,
            r#"{"phase": "planning", "approved_gates": {"execution": false}}"#,
        )
        .expect("write state.json");
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        let verdict = build_verdict(&ctx).expect("verdict");
        assert_eq!(verdict["allowed_tools"], json!(GATED_TOOL_SET));

        std::fs::write(
            &state_path,
            r#"{"phase": "planning", "approved_gates": {"execution": true}}"#,
        )
        .expect("write state.json");
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        let verdict = build_verdict(&ctx).expect("verdict");
        assert_eq!(verdict["allowed_tools"], json!(FULL_TOOL_SET));
    }

    #[test]
    fn stage_tools_keeps_the_verdict_keys_it_publishes() {
        // The Pi belt reads exactly two of these: `allowed_tools` (with
        // `allowedTools`/`tools` fallbacks) and `stage` (with a `stage_name`
        // fallback) — .pi/extensions/bee-guard.ts, the turn_start handler. The
        // belt builds its own user and model sentences, so the four message
        // keys below have no reader today; they stay because the payload is
        // published, and this test states plainly which are load-bearing.
        let _env = HerdingWorkerEnvGuard::non_worker();
        let dir = setup_bee_repo();
        std::fs::write(
            dir.path().join(".bee").join("state.json"),
            r#"{"phase": "grooming", "approved_gates": {"execution": false}}"#,
        )
        .expect("write state.json");

        let stdin = json!({"cwd": dir.path().to_str().unwrap()}).to_string();
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        let verdict = build_verdict(&ctx).expect("verdict");

        // Read by the belt.
        assert_eq!(verdict["stage"], json!("grooming"));
        assert_eq!(verdict["stage_name"], json!("grooming"));
        assert_eq!(verdict["allowed_tools"], json!(FULL_TOOL_SET));

        // Published, no reader today.
        for key in ["user_message", "user_notice", "model_message", "model_notice"] {
            assert!(verdict.get(key).is_some(), "missing key {key:?}");
        }
    }

    const LEADER_LANES: [&str; 3] = ["small", "standard", "high-risk"];

    #[test]
    fn stage_tools_full_set_names_every_tool_bee_registers() {
        for tool in ["bee_dispatch", "bee_advisor", "bee_steer", "verdict"] {
            assert!(FULL_TOOL_SET.contains(&tool), "full set must keep {tool:?}");
        }
        assert_eq!(READ_ONLY_TOOLS, ["read", "bash"]);
    }

    #[test]
    fn stage_tools_hands_a_leader_dispatch_instead_of_edit_and_write() {
        let expected: Vec<&str> = FULL_TOOL_SET
            .iter()
            .copied()
            .filter(|t| !matches!(*t, "edit" | "write" | "verdict"))
            .collect();
        for lane in LEADER_LANES {
            let allowed = allowed_tools_for("swarming", true, Some(lane), false, true);
            assert_eq!(allowed, expected.as_slice(), "lane {lane:?}");
            for tool in ["bee_dispatch", "bee_advisor", "bee_steer"] {
                assert!(allowed.contains(&tool), "lane {lane:?} must carry {tool:?}");
            }
            assert!(!allowed.contains(&"verdict"), "lane {lane:?} must not carry verdict");
        }
    }

    #[test]
    fn stage_tools_gives_a_worker_session_the_full_set_with_verdict() {
        for lane in LEADER_LANES {
            let allowed = allowed_tools_for("swarming", true, Some(lane), true, true);
            assert_eq!(allowed, FULL_TOOL_SET, "lane {lane:?}");
            assert!(allowed.contains(&"verdict"));
        }
    }

    #[test]
    fn stage_tools_keeps_the_full_set_off_the_leader_lanes_and_phase() {
        for lane in [Some("tiny"), Some("docs"), Some("spike"), Some(""), None] {
            assert_eq!(allowed_tools_for("swarming", true, lane, false, true), FULL_TOOL_SET, "lane {lane:?}");
        }
        for phase in ["reviewing", "scribing", "compounding", "grooming", "idle"] {
            assert_eq!(allowed_tools_for(phase, true, Some("small"), false, true), FULL_TOOL_SET, "phase {phase:?}");
        }
        assert_eq!(allowed_tools_for("swarming", false, Some("small"), false, true), FULL_TOOL_SET);
    }

    #[test]
    fn stage_tools_gives_the_gated_phases_read_bash_and_advisor() {
        for phase in ["exploring", "planning"] {
            for lane in [Some("tiny"), Some("small"), None] {
                assert_eq!(allowed_tools_for(phase, false, lane, false, true), GATED_TOOL_SET, "{phase:?} {lane:?}");
            }
        }
    }

    #[test]
    fn stage_tools_restores_todays_answer_when_the_harness_workflow_is_off() {
        assert_eq!(allowed_tools_for("swarming", true, Some("small"), false, false), BASE_TOOL_SET);
        assert_eq!(allowed_tools_for("swarming", true, Some("small"), true, false), BASE_TOOL_SET);
        assert_eq!(allowed_tools_for("grooming", false, None, false, false), BASE_TOOL_SET);
        assert_eq!(allowed_tools_for("planning", false, Some("small"), false, false), READ_ONLY_TOOLS);
    }

    #[test]
    fn stage_tools_reads_lane_and_harness_switch_into_the_leader_verdict() {
        let _env = HerdingWorkerEnvGuard::non_worker();
        let dir = setup_bee_repo();
        std::fs::write(
            dir.path().join(".bee").join("state.json"),
            r#"{"phase": "swarming", "route": {"lane": "small"}, "approved_gates": {"execution": true}}"#,
        )
        .expect("write state.json");
        let stdin = json!({"cwd": dir.path().to_str().unwrap()}).to_string();

        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        let verdict = build_verdict(&ctx).expect("verdict");
        assert_eq!(verdict["allowed_tools"], json!(LEADER_TOOL_SET));
        for key in ["user_message", "model_message"] {
            let text = verdict[key].as_str().expect("sentence");
            assert!(text.contains("bee_dispatch"), "{key} must name bee_dispatch: {text}");
            assert!(text.contains("/bee-tools-reopen"), "{key} must name /bee-tools-reopen: {text}");
        }

        std::fs::write(
            dir.path().join(".bee").join("config.json"),
            r#"{"pi_harness_workflow": false}"#,
        )
        .expect("write config.json");
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        let verdict = build_verdict(&ctx).expect("verdict");
        assert_eq!(verdict["allowed_tools"], json!(BASE_TOOL_SET));
    }

    #[test]
    fn stage_tools_fails_open_without_bee_store() {
        let dir = tempdir().expect("tempdir");
        let stdin = json!({"cwd": dir.path().to_str().unwrap()}).to_string();
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        assert_eq!(run_inner(&ctx), ExitCode::SUCCESS);
    }
}
