use super::is_turn_boundary;
use serde_json::Value;

const AGREEMENT_PHRASES: &[&str] = &[
    "you're right",
    "you are right",
    "good idea",
    "great idea",
    "sounds good",
    "sounds right",
    "makes sense",
    "i agree",
    "agreed",
    "đúng rồi",
    "bạn đúng",
    "anh đúng",
    "chị đúng",
    "hợp lý",
    "đồng ý",
    "ý hay",
    "chuẩn rồi",
];

const CONDITIONAL_MARKERS: &[&str] = &[
    "checking",
    "check first",
    "kiểm tra trước",
    "để em kiểm tra",
    "để tôi kiểm tra",
];

const WARNING: &str = "bee: this reply agrees with a proposal but carries no [verified] label — rule agents-verify-before-agreeing: check the key assumption first, then agree with what the check showed.";

pub(crate) fn unverified_agreement_warning(events: &[Value]) -> Option<String> {
    let text = last_turn_assistant_text(events).to_lowercase();
    let agrees = AGREEMENT_PHRASES.iter().any(|p| text.contains(p));
    let labelled = text.contains("[verified");
    let conditional = CONDITIONAL_MARKERS.iter().any(|m| text.contains(m));
    (agrees && !labelled && !conditional).then(|| WARNING.to_string())
}

const PRINCIPLE_WARNING: &str = "bee: this routed reply names no bee-principle- slug and states no principle changed anything — rule agents-name-applied-principles: name each applied principle and the decision it changed, or say plainly that none did.";

const PLAYBOOK_WARNING: &str = "bee: bee orient named a situation file and this turn never opened a playbooks/ file — open the situation whose triggers match and make the todo list from its numbered steps.";

const NO_PRINCIPLE_STATEMENTS: &[&str] = &["no principle", "không principle", "không có principle"];

#[derive(Default)]
struct TurnTools {
    orient_ids: Vec<String>,
    orient_without_id: bool,
    outputs: Vec<(String, String)>,
    opened_playbook: bool,
}

pub(crate) fn routed_reply_warnings(events: &[Value]) -> Vec<String> {
    let start = events.iter().rposition(is_turn_boundary).map_or(0, |i| i + 1);
    let turn = &events[start..];
    let mut tools = TurnTools::default();
    for event in turn {
        collect_tool_activity(event, &mut tools);
    }
    let orient_outputs: Vec<&str> = tools
        .outputs
        .iter()
        .filter(|(id, _)| tools.orient_ids.contains(id) || (id.is_empty() && tools.orient_without_id))
        .map(|(_, text)| text.as_str())
        .collect();
    let listed_principles = orient_outputs.iter().any(|t| t.contains("Principles (class="));
    let listed_situation = orient_outputs.iter().any(|t| t.lines().any(|l| l.trim_start().starts_with("situation:")));
    let text = last_turn_assistant_text(turn).to_lowercase();
    let names_principle =
        text.contains("bee-principle-") || NO_PRINCIPLE_STATEMENTS.iter().any(|s| text.contains(s));
    let mut warnings = Vec::new();
    if listed_principles && !names_principle {
        warnings.push(PRINCIPLE_WARNING.to_string());
    }
    if listed_situation && !tools.opened_playbook {
        warnings.push(PLAYBOOK_WARNING.to_string());
    }
    warnings
}

fn collect_tool_activity(event: &Value, tools: &mut TurnTools) {
    let ev_type = event.get("type").and_then(Value::as_str).unwrap_or("");
    match ev_type {
        "assistant" | "user" => {
            let blocks = event.get("message").and_then(|m| m.get("content")).and_then(Value::as_array);
            for block in blocks.into_iter().flatten() {
                match block.get("type").and_then(Value::as_str) {
                    Some("tool_use") => {
                        let id = block.get("id").and_then(Value::as_str).unwrap_or("");
                        let name = block.get("name").and_then(Value::as_str).unwrap_or("");
                        let input = block.get("input");
                        let arg = |k: &str| input.and_then(|i| i.get(k)).and_then(Value::as_str).unwrap_or("");
                        let path = if arg("file_path").is_empty() { arg("path") } else { arg("file_path") };
                        record_call(tools, id, name, arg("command"), path);
                    }
                    Some("tool_result") => {
                        let id = block.get("tool_use_id").and_then(Value::as_str).unwrap_or("");
                        tools.outputs.push((id.to_string(), result_text(block.get("content"))));
                    }
                    _ => {}
                }
            }
        }
        "response_item" => {
            let Some(payload) = event.get("payload") else { return };
            let id = payload.get("call_id").and_then(Value::as_str).unwrap_or("");
            match payload.get("type").and_then(Value::as_str) {
                Some("function_call") => {
                    let name = payload.get("name").and_then(Value::as_str).unwrap_or("");
                    let args = payload.get("arguments").and_then(Value::as_str).unwrap_or("");
                    record_call(tools, id, name, args, "");
                }
                Some("function_call_output") => {
                    tools.outputs.push((id.to_string(), result_text(payload.get("output"))));
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn record_call(tools: &mut TurnTools, id: &str, name: &str, command: &str, path: &str) {
    if command.contains("bee orient") && command.contains("--class") {
        if id.is_empty() {
            tools.orient_without_id = true;
        } else {
            tools.orient_ids.push(id.to_string());
        }
    }
    let reads_file = name == "Read" && path.contains("/playbooks/");
    let shows_file = command.contains("/playbooks/")
        && command.split(|c: char| !c.is_alphanumeric()).any(|w| matches!(w, "cat" | "sed" | "head"));
    tools.opened_playbook |= reads_file || shows_file;
}

fn result_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(blocks)) => {
            blocks.iter().filter_map(|b| b.get("text").and_then(Value::as_str)).collect::<Vec<_>>().join("\n")
        }
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn last_turn_assistant_text(events: &[Value]) -> String {
    let mut texts: Vec<&str> = Vec::new();
    for event in events.iter().rev() {
        if is_turn_boundary(event) {
            break;
        }
        collect_assistant_text(event, &mut texts);
    }
    texts.join("\n")
}

fn collect_assistant_text<'a>(event: &'a Value, out: &mut Vec<&'a str>) {
    let ev_type = event.get("type").and_then(Value::as_str).unwrap_or("");
    let payload = event.get("payload");
    match ev_type {
        "assistant" => {
            let blocks = event.get("message").and_then(|m| m.get("content")).and_then(Value::as_array);
            out.extend(blocks.into_iter().flatten().filter_map(|b| text_of(b, &["text"])));
        }
        "response_item" => {
            let Some(payload) = payload else { return };
            if payload.get("type").and_then(Value::as_str) != Some("message")
                || payload.get("role").and_then(Value::as_str) != Some("assistant")
            {
                return;
            }
            match payload.get("content") {
                Some(Value::Array(blocks)) => {
                    out.extend(blocks.iter().filter_map(|b| text_of(b, &["output_text", "text"])));
                }
                Some(Value::String(s)) => out.push(s),
                _ => out.extend(payload.get("text").and_then(Value::as_str)),
            }
        }
        "event_msg" => {
            if let Some(p) = payload {
                if p.get("type").and_then(Value::as_str) == Some("task_complete") {
                    out.extend(p.get("last_agent_message").and_then(Value::as_str));
                }
            }
        }
        _ => {}
    }
}

fn text_of<'a>(block: &'a Value, kinds: &[&str]) -> Option<&'a str> {
    let kind = block.get("type").and_then(Value::as_str)?;
    kinds.contains(&kind).then(|| block.get("text").and_then(Value::as_str)).flatten()
}
