use crate::hooks::adapter::HookContext;
use crate::verbs::handoff_door::{probe_door, HandoffDoor};
use serde_json::Value;
use std::path::Path;

const PHRASES: &[&str] = &[
    "open a new session",
    "start a new session",
    "open a fresh session",
    "start a fresh session",
    "new chat",
    "run /clear",
    "type /clear",
    "please run",
    "you can run",
    "you need to run",
    "you should run",
    "can you run",
    "then run",
    "mở session mới",
    "mở một session mới",
    "mở phiên mới",
    "tạo session mới",
    "bắt đầu session mới",
    "mở chat mới",
    "hãy chạy",
    "anh chạy",
    "bạn chạy",
    "chị chạy",
    "vui lòng chạy",
];

pub(crate) enum OwnerAskOutcome {
    Pass,
    Block(String),
    Advisory(String),
}

pub(crate) fn owner_ask_match(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();
    PHRASES.iter().copied().find(|phrase| lower.contains(phrase))
}

fn block_reason(phrase: &str) -> String {
    format!(
        "bee turn-end refused: the reply asks the owner to {phrase}. \
The owner never opens sessions or re-runs steps the agent can run. \
Run it yourself: bee state handoff send --task \"<owner request in their words; standing orders>\" \
(add --follow-up-of <run> when a finished run left work undone), \
or bee state handoff check to dispatch the follow-up for a handed-off run; then end the turn with one line."
    )
}

fn advisory_line(phrase: &str) -> String {
    format!(
        "bee: the reply asks the owner to {phrase}, and no waggledance door exists — \
write .bee/HANDOFF.json per the handoff playbook."
    )
}

fn push_tool_result_texts(event: &Value, out: &mut Vec<String>) {
    let collect_blocks = |blocks: &[Value], out: &mut Vec<String>| {
        for block in blocks {
            if block.get("type").and_then(Value::as_str) == Some("tool_result") {
                out.push(super::claims::result_text(block.get("content")));
            }
        }
    };
    if let Some(content) = event.get("message").and_then(|m| m.get("content")).and_then(Value::as_array) {
        collect_blocks(content, out);
    }
    if let Some(content) = event.get("content").and_then(Value::as_array) {
        collect_blocks(content, out);
    }
    if let Some(content) = event.get("payload").and_then(|p| p.get("content")).and_then(Value::as_array) {
        collect_blocks(content, out);
    }
    if event.get("type").and_then(Value::as_str) == Some("tool_result") {
        out.push(super::claims::result_text(event.get("content")));
    }
    if let Some(payload) = event.get("payload") {
        match payload.get("type").and_then(Value::as_str) {
            Some("tool_result") => out.push(super::claims::result_text(payload.get("content"))),
            Some("function_call_output") => out.push(super::claims::result_text(payload.get("output"))),
            _ => {}
        }
    }
}

fn extract_owner_only_targets(text: &str, targets: &mut Vec<String>) {
    const PREFIX: &str = "[owner-only: ";
    let mut rest = text;
    while let Some(start) = rest.find(PREFIX) {
        let after = &rest[start + PREFIX.len()..];
        let Some(end) = after.find(']') else {
            break;
        };
        let target = after[..end].trim();
        if !target.is_empty() {
            targets.push(target.to_string());
        }
        rest = &after[end + 1..];
    }
}

fn collect_owner_only_targets(events: &[Value]) -> Vec<String> {
    let mut texts = Vec::new();
    for event in events {
        push_tool_result_texts(event, &mut texts);
    }
    let mut targets = Vec::new();
    for text in texts {
        extract_owner_only_targets(&text, &mut targets);
    }
    targets
}

fn reply_names_owner_only_target(reply: &str, events: &[Value]) -> bool {
    let targets = collect_owner_only_targets(events);
    if targets.is_empty() {
        return false;
    }
    let home = super::node_homedir();
    targets.iter().any(|target| {
        if reply.contains(target.as_str()) {
            return true;
        }
        if !home.is_empty()
            && let Some(rest) = target.strip_prefix(&home)
            && (rest.starts_with('/') || rest.starts_with('\\'))
        {
            return reply.contains(&format!("~{rest}"));
        }
        false
    })
}

pub(crate) fn evaluate_with(
    ctx: &HookContext,
    events: &[Value],
    is_herding_worker: bool,
    probe: impl FnOnce() -> HandoffDoor,
) -> OwnerAskOutcome {
    if ctx.event != "Stop" {
        return OwnerAskOutcome::Pass;
    }
    if ctx.payload.get("stop_hook_active").and_then(Value::as_bool).unwrap_or(false) {
        return OwnerAskOutcome::Pass;
    }
    if is_herding_worker {
        return OwnerAskOutcome::Pass;
    }
    let text = super::last_turn_assistant_text(events);
    let Some(phrase) = owner_ask_match(&text) else {
        return OwnerAskOutcome::Pass;
    };
    if reply_names_owner_only_target(&text, events) {
        return OwnerAskOutcome::Pass;
    }
    match probe() {
        HandoffDoor::Present { .. } => OwnerAskOutcome::Block(block_reason(phrase)),
        HandoffDoor::Absent => OwnerAskOutcome::Advisory(advisory_line(phrase)),
    }
}

pub(crate) fn evaluate(root: &Path, ctx: &HookContext, events: &[Value]) -> OwnerAskOutcome {
    let door_root = super::control_root(root, ctx);
    evaluate_with(
        ctx,
        events,
        crate::hooks::herding_worker_marker_set(),
        || probe_door(&door_root),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::adapter::read_hook_context;
    use serde_json::json;
    use std::cell::Cell;
    use std::path::PathBuf;

    fn stop_ctx(extra: Value) -> HookContext {
        let mut body = json!({
            "hook_event_name": "Stop",
            "cwd": "/tmp/demo",
            "session_id": "s-1",
        });
        if let Value::Object(map) = extra {
            for (k, v) in map {
                body[k.as_str()] = v;
            }
        }
        let stdin = serde_json::to_string(&body).unwrap();
        read_hook_context("session-close", &[], &stdin)
    }

    fn present_door() -> HandoffDoor {
        HandoffDoor::Present {
            binary: PathBuf::from("/usr/local/bin/waggledance"),
            project_id: "demo".to_string(),
        }
    }

    #[test]
    fn matches_english_phrases_case_insensitively() {
        assert_eq!(
            owner_ask_match("Please Open a New Session to continue."),
            Some("open a new session")
        );
        assert_eq!(
            owner_ask_match("You can Start a fresh session now."),
            Some("start a fresh session")
        );
        assert_eq!(owner_ask_match("Please RUN /CLEAR first."), Some("run /clear"));
        assert_eq!(owner_ask_match("You should run `cargo test`."), Some("you should run"));
    }

    #[test]
    fn matches_vietnamese_phrases() {
        assert_eq!(
            owner_ask_match("Anh vui lòng mở session mới giúp em nhé."),
            Some("mở session mới")
        );
        assert_eq!(
            owner_ask_match("Bạn chạy lệnh này giúp mình."),
            Some("bạn chạy")
        );
        assert_eq!(
            owner_ask_match("Hãy chạy `bee status` để kiểm tra."),
            Some("hãy chạy")
        );
    }

    #[test]
    fn negative_cases_return_none() {
        assert_eq!(owner_ask_match("All tests passed and the branch is merged."), None);
        assert_eq!(owner_ask_match("I ran `bee state handoff send` and dispatched run-1."), None);
        assert_eq!(owner_ask_match("Đã hoàn thành và gộp nhánh vào main."), None);
    }

    #[test]
    fn stop_with_match_and_door_present_blocks() {
        let ctx = stop_ctx(json!({}));
        let events = vec![json!({
            "type": "assistant",
            "message": { "content": [{ "type": "text", "text": "Please open a new session to finish." }] }
        })];
        let probed = Cell::new(false);
        let outcome = evaluate_with(&ctx, &events, false, || {
            probed.set(true);
            present_door()
        });
        assert!(probed.get());
        match outcome {
            OwnerAskOutcome::Block(reason) => {
                assert!(reason.contains("bee turn-end refused: the reply asks the owner to open a new session."));
                assert!(reason.contains("bee state handoff send --task"));
                assert!(reason.contains("bee state handoff check"));
            }
            _ => panic!("expected Block"),
        }
    }

    #[test]
    fn stop_hook_active_passes_without_probing() {
        let ctx = stop_ctx(json!({ "stop_hook_active": true }));
        let events = vec![json!({
            "type": "assistant",
            "message": { "content": [{ "type": "text", "text": "Please open a new session." }] }
        })];
        let probed = Cell::new(false);
        let outcome = evaluate_with(&ctx, &events, false, || {
            probed.set(true);
            present_door()
        });
        assert!(!probed.get());
        assert!(matches!(outcome, OwnerAskOutcome::Pass));
    }

    #[test]
    fn herding_worker_passes_without_probing() {
        let ctx = stop_ctx(json!({}));
        let events = vec![json!({
            "type": "assistant",
            "message": { "content": [{ "type": "text", "text": "Please open a new session." }] }
        })];
        let probed = Cell::new(false);
        let outcome = evaluate_with(&ctx, &events, true, || {
            probed.set(true);
            present_door()
        });
        assert!(!probed.get());
        assert!(matches!(outcome, OwnerAskOutcome::Pass));
    }

    #[test]
    fn door_absent_returns_advisory_without_blocking() {
        let ctx = stop_ctx(json!({}));
        let events = vec![json!({
            "type": "assistant",
            "message": { "content": [{ "type": "text", "text": "Anh mở phiên mới nhé." }] }
        })];
        let probed = Cell::new(false);
        let outcome = evaluate_with(&ctx, &events, false, || {
            probed.set(true);
            HandoffDoor::Absent
        });
        assert!(probed.get());
        match outcome {
            OwnerAskOutcome::Advisory(line) => {
                assert!(line.contains("mở phiên mới"));
                assert!(line.contains("no waggledance door exists"));
                assert!(line.contains(".bee/HANDOFF.json"));
                assert!(!line.contains('\n'));
            }
            _ => panic!("expected Advisory"),
        }
    }

    #[test]
    fn tool_output_only_does_not_match_or_probe() {
        let ctx = stop_ctx(json!({}));
        let events = vec![
            json!({
                "type": "user",
                "message": { "content": [{ "type": "tool_result", "tool_use_id": "t1", "content": "open a new session" }] }
            }),
            json!({
                "type": "assistant",
                "message": { "content": [{ "type": "text", "text": "Done." }] }
            }),
        ];
        let probed = Cell::new(false);
        let outcome = evaluate_with(&ctx, &events, false, || {
            probed.set(true);
            present_door()
        });
        assert!(!probed.get());
        assert!(matches!(outcome, OwnerAskOutcome::Pass));
    }

    fn owner_only_events(marker: Option<&str>) -> Vec<Value> {
        let refusal = match marker {
            Some(target) => format!("bee: write refused for the agent. [owner-only: {target}]"),
            None => "bee: write refused for the agent.".to_string(),
        };
        vec![
            json!({
                "type": "user",
                "message": { "content": [{ "type": "tool_result", "tool_use_id": "t1", "content": refusal }] }
            }),
            json!({
                "type": "assistant",
                "message": { "content": [{ "type": "text", "text": "bạn chạy cp .bee/team-only-agy.json .bee/team.json" }] }
            }),
        ]
    }

    fn evaluate_present(events: &[Value]) -> OwnerAskOutcome {
        evaluate_with(&stop_ctx(json!({})), events, false, present_door)
    }

    #[test]
    fn owner_only_marker_for_named_target_passes_with_door_present() {
        let outcome = evaluate_present(&owner_only_events(Some(".bee/team.json")));
        assert!(matches!(outcome, OwnerAskOutcome::Pass));
    }

    #[test]
    fn owner_ask_without_owner_only_marker_blocks() {
        let outcome = evaluate_present(&owner_only_events(None));
        assert!(matches!(outcome, OwnerAskOutcome::Block(_)));
    }

    #[test]
    fn owner_only_marker_for_another_target_blocks() {
        let outcome = evaluate_present(&owner_only_events(Some(".bee/state.json")));
        assert!(matches!(outcome, OwnerAskOutcome::Block(_)));
    }
}
