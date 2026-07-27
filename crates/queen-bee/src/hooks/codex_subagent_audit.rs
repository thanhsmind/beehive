//! codex_subagent_audit — Rust port of
//! `.bee/bin/hooks/bee-codex-subagent-audit.mjs` (rust-port-7). Codex exposes
//! `SubagentStart` only after the native subagent has started; this handler
//! records bounded audit evidence and is NEVER a pre-spawn authorization or
//! denial surface. The same handler closes the audit at `SubagentStop`.
//!
//! Deliberately NOT gated by the hooks-enabled toggle: the frozen mjs source
//! never imports `state.mjs`/calls `hookEnabled` for this hook — it is gated
//! purely by `hook_event_name` (`SubagentStart`/`SubagentStop` vs anything
//! else -> `unsupported-subagent-event` coverage gap, no audit record). This
//! is a faithful port of the actual source, not the cell description's
//! shorthand ("HOOK-ENABLED TOGGLE honored exactly") generalized past what
//! this specific hook implements — matching real behavior is the whole
//! point of a conformance rig; see
//! `tests/hook_conformance.rs::codex_subagent_audit_ignores_hook_toggle_and_gates_on_event_name_matches_oracle`
//! for the fixture proving this exact parity (both runtimes still log when
//! "disabled", both gate identically on the event name).

use serde_json::{Map, Value};

use crate::adapter;

const HOOK_NAME: &str = "codex-subagent-audit";

struct FieldLimit {
    name: &'static str,
    limit: usize,
}

const FIELD_LIMITS: &[FieldLimit] = &[
    FieldLimit { name: "session_id", limit: 120 },
    FieldLimit { name: "agent_id", limit: 120 },
    FieldLimit { name: "agent_name", limit: 120 },
    FieldLimit { name: "agent_type", limit: 80 },
];

fn bounded_audit_fields(payload: &Value) -> Map<String, Value> {
    let mut out = Map::new();
    for field in FIELD_LIMITS {
        let Some(raw) = payload.get(field.name).and_then(Value::as_str) else {
            continue;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let char_count = trimmed.chars().count();
        let value = if char_count <= field.limit {
            trimmed.to_string()
        } else {
            let truncated: String = trimmed.chars().take(field.limit).collect();
            format!("{truncated}...")
        };
        out.insert(field.name.to_string(), Value::String(value));
    }
    out
}

pub fn run(argv: &[String], raw_stdin: &str) -> i32 {
    let ctx = adapter::read_hook_context(HOOK_NAME, argv, raw_stdin);
    let Some(root) = ctx.root.clone() else {
        return 0;
    };

    let lifecycle = match ctx.event.as_str() {
        "SubagentStart" => "start",
        "SubagentStop" => "stop",
        _ => {
            adapter::log_coverage_gap(
                Some(&root),
                HOOK_NAME,
                "unsupported-subagent-event",
                "expected SubagentStart or SubagentStop; audit record omitted",
                ctx.source.as_deref(),
            );
            return 0;
        }
    };

    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(adapter::timestamp_now()));
    entry.insert("hook".into(), Value::String(HOOK_NAME.to_string()));
    entry.insert("event".into(), Value::String("subagent-audit".to_string()));
    entry.insert("lifecycle".into(), Value::String(lifecycle.to_string()));
    entry.insert("authority".into(), Value::String("audit-only".to_string()));
    if lifecycle == "start" {
        entry.insert("timing".into(), Value::String("post-start".to_string()));
    }
    if let Some(source) = &ctx.source {
        entry.insert("source".into(), Value::String(source.clone()));
    }
    for (key, value) in bounded_audit_fields(&ctx.payload) {
        entry.insert(key, value);
    }

    adapter::append_hook_log(&root, &Value::Object(entry));
    0
}
