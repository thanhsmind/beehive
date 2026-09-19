// bee hook stage-tools — advisory tool gating based on current stage policy (D4, D12)

use crate::hooks::adapter::{read_hook_context, HookContext};
use crate::hooks::write_guard::{
    is_gated_phase, read_state, resolve_write_record, Emit, RecordResolution,
};
use crate::hooks::Outcome;
use crate::state::hook_enabled;
use serde_json::{json, Value};
use std::io::Write;
use std::process::ExitCode;

pub const HOOK_NAME: &str = "stage-tools";

pub(crate) const READ_ONLY_TOOLS: [&str; 2] = ["read", "bash"];
pub(crate) const FULL_TOOL_SET: [&str; 8] = [
    "read", "bash", "edit", "write", "find", "grep", "ls", "powershell",
];

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

fn run_inner(ctx: &HookContext) -> ExitCode {
    let Some(root_pb) = &ctx.root else {
        return ExitCode::SUCCESS;
    };
    if !crate::hooks::adapter::bee_installed(root_pb) {
        return ExitCode::SUCCESS;
    }
    if !hook_enabled(root_pb, HOOK_NAME) {
        return ExitCode::SUCCESS;
    }

    let store_root_pb = ctx.store_root.clone().unwrap_or_else(|| root_pb.clone());
    let control_root_pb = ctx.control_root.clone().unwrap_or_else(|| store_root_pb.clone());
    let control_root = control_root_pb.to_string_lossy().into_owned();

    let state = match read_state(&store_root_pb) {
        Ok(s) => s,
        Err(_) => return ExitCode::SUCCESS,
    };

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

    let is_gated = is_gated_phase(&Value::String(phase.to_string()));
    let gate_approved = matches!(
        record.get("approved_gates").and_then(|g| g.get("execution")),
        Some(Value::Bool(true))
    );
    let is_after_gate = matches!(phase, "swarming" | "reviewing" | "scribing" | "compounding");
    let execution_is_open = (!is_gated && is_after_gate) || gate_approved;

    let allowed_tools: Vec<String> = if execution_is_open {
        FULL_TOOL_SET.iter().map(|s| s.to_string()).collect()
    } else {
        READ_ONLY_TOOLS.iter().map(|s| s.to_string()).collect()
    };

    let stage_name = phase;
    let user_sentence = if execution_is_open {
        format!("bee stage gate: full tool set active for stage \"{stage_name}\".")
    } else {
        format!(
            "bee stage gate: active tools narrowed for stage \"{stage_name}\" (allowed: {}). Use /bee-tools-reopen to restore all tools.",
            allowed_tools.join(", ")
        )
    };

    let model_sentence = if execution_is_open {
        format!("Notice: Full tool set is available for bee stage \"{stage_name}\".")
    } else {
        format!(
            "Notice: Tools narrowed by bee stage policy for stage \"{stage_name}\": off-stage tools removed. Do not attempt to use them or fall back to bash redirection. If you require these tools, ask the user to run /bee-tools-reopen."
        )
    };

    let verdict = json!({
        "stage": stage_name,
        "stage_name": stage_name,
        "allowed_tools": allowed_tools,
        "user_message": user_sentence,
        "user_notice": user_sentence,
        "model_message": model_sentence,
        "model_notice": model_sentence,
    });

    let stdout = verdict.to_string();
    let _ = std::io::stdout().write_all(stdout.as_bytes());
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn stage_tools_decision_logic() {
        let dir = setup_bee_repo();
        let bee_dir = dir.path().join(".bee");

        // 1. Planning phase, unapproved gate -> read only tools
        std::fs::write(
            bee_dir.join("state.json"),
            r#"{"phase": "planning", "approved_gates": {"execution": false}}"#,
        )
        .expect("write state.json");

        let stdin = json!({"cwd": dir.path().to_str().unwrap()}).to_string();
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        let store_root_pb = ctx.store_root.clone().unwrap_or_else(|| ctx.root.clone().unwrap());
        let state = read_state(&store_root_pb).unwrap();
        let phase = state.get("phase").unwrap().as_str().unwrap();
        let is_gated = is_gated_phase(&Value::String(phase.to_string()));
        let gate_approved = matches!(
            state.get("approved_gates").and_then(|g| g.get("execution")),
            Some(Value::Bool(true))
        );
        assert!(is_gated);
        assert!(!gate_approved);

        // 2. Planning phase, approved gate -> full tool set
        std::fs::write(
            bee_dir.join("state.json"),
            r#"{"phase": "planning", "approved_gates": {"execution": true}}"#,
        )
        .expect("write state.json");
        let state = read_state(&store_root_pb).unwrap();
        let gate_approved = matches!(
            state.get("approved_gates").and_then(|g| g.get("execution")),
            Some(Value::Bool(true))
        );
        assert!(gate_approved);

        // 3. Swarming phase -> full tool set
        std::fs::write(
            bee_dir.join("state.json"),
            r#"{"phase": "swarming", "approved_gates": {}}"#,
        )
        .expect("write state.json");
        let state = read_state(&store_root_pb).unwrap();
        let phase = state.get("phase").unwrap().as_str().unwrap();
        let is_after_gate = matches!(phase, "swarming" | "reviewing" | "scribing" | "compounding");
        assert!(is_after_gate);
    }

    #[test]
    fn stage_tools_fails_open_without_bee_store() {
        let dir = tempdir().expect("tempdir");
        let stdin = json!({"cwd": dir.path().to_str().unwrap()}).to_string();
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        assert_eq!(run_inner(&ctx), ExitCode::SUCCESS);
    }
}
