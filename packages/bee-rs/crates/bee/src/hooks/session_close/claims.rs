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
