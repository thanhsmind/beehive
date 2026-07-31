// bee hook codex-subagent-audit — port of hooks/bee-codex-subagent-audit.mjs:
// bounded audit evidence for Codex SubagentStart/SubagentStop; audit-only,
// never an authorization surface. Always exits 0.

use crate::hooks::adapter::{append_hook_log, log_coverage_gap, now_iso, read_hook_context};
use crate::hooks::Outcome;
use serde_json::{Map, Value};
use std::process::ExitCode;

const HOOK_NAME: &str = "codex-subagent-audit";
const FIELD_LIMITS: [(&str, usize); 4] = [
    ("session_id", 120),
    ("agent_id", 120),
    ("agent_name", 120),
    ("agent_type", 80),
];

fn bounded_audit_fields(payload: &Map<String, Value>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (field, limit) in FIELD_LIMITS {
        let Some(value) = payload.get(field).and_then(Value::as_str) else { continue };
        let normalized = value.trim();
        if normalized.is_empty() {
            continue;
        }
        let bounded = if normalized.chars().count() <= limit {
            normalized.to_string()
        } else {
            format!("{}...", normalized.chars().take(limit).collect::<String>())
        };
        out.push((field.to_string(), bounded));
    }
    out
}

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    Outcome::Done(run_inner(argv, stdin))
}

fn run_inner(argv: &[String], stdin: &str) -> ExitCode {
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = &ctx.root else {
        return ExitCode::SUCCESS;
    };

    let lifecycle = match ctx.event.as_str() {
        "SubagentStart" => "start",
        "SubagentStop" => "stop",
        _ => {
            log_coverage_gap(
                root,
                HOOK_NAME,
                "unsupported-subagent-event",
                "expected SubagentStart or SubagentStop; audit record omitted",
                ctx.source,
            );
            return ExitCode::SUCCESS;
        }
    };

    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(now_iso()));
    entry.insert("hook".into(), Value::String(HOOK_NAME.to_string()));
    entry.insert("event".into(), Value::String("subagent-audit".to_string()));
    entry.insert("lifecycle".into(), Value::String(lifecycle.to_string()));
    entry.insert("authority".into(), Value::String("audit-only".to_string()));
    if lifecycle == "start" {
        entry.insert("timing".into(), Value::String("post-start".to_string()));
    }
    if let Some(s) = ctx.source {
        entry.insert("source".into(), Value::String(s.to_string()));
    }
    for (field, value) in bounded_audit_fields(&ctx.payload) {
        entry.insert(field, Value::String(value));
    }
    append_hook_log(root, &Value::Object(entry));
    ExitCode::SUCCESS
}
