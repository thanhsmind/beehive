//! tools_logger — Rust port of `.bee/bin/hooks/bee-tools-logger.mjs`
//! (rust-port-7). Passive measurement only, zero enforcement: this hook can
//! NEVER deny and NEVER emits a Codex `decision:"block"`. Line schema (AO15,
//! decision f1ca79b9): `{ts, tool_name, agent_id, agent_type}` —
//! `agent_id`/`agent_type` are `null` when absent from the payload;
//! `duration_ms`/`status` are appended ONLY when the payload itself carries
//! them. `tool_input`/`tool_response` bodies are NEVER logged.
//!
//! Fail-open: every failure -> `log_crash` + exit 0, matching the mjs
//! source's `catch (error) { logCrash(...); return 0; }`.
//!
//! Appends go through `bee_core::fsutil::append_jsonl` (D9-adjacent storage
//! discipline for this cell: "hook appends go through bee-core fsutil") —
//! functionally identical to the mjs source's raw `fs.appendFileSync` (one
//! compact JSON line + `\n`), just routed through the shared Rust primitive.

use std::path::Path;

use serde_json::{Map, Value};

use crate::adapter::{self, HookContext};
use crate::hookconfig;

const HOOK_NAME: &str = "tools-logger";

pub fn run(argv: &[String], raw_stdin: &str) -> i32 {
    let ctx: HookContext = adapter::read_hook_context(HOOK_NAME, argv, raw_stdin);
    let Some(root) = ctx.root.clone() else {
        return 0;
    };

    // Mirrors the mjs source's own existsSync backward-compat check before
    // it dynamically imports state.mjs.
    if !root.join(".bee").join("bin").join("lib").join("state.mjs").exists() {
        return 0;
    }

    if !hookconfig::hook_enabled(&root, HOOK_NAME) {
        return 0;
    }

    if let Err(err) = log_tool_call(&root, &ctx.payload) {
        adapter::log_crash(Some(&root), HOOK_NAME, &err, ctx.source.as_deref());
        return 0;
    }
    0
}

fn log_tool_call(root: &Path, payload: &Value) -> Result<(), String> {
    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(adapter::timestamp_now()));

    let tool_name = payload.get("tool_name").and_then(Value::as_str).unwrap_or("");
    entry.insert("tool_name".into(), Value::String(tool_name.to_string()));

    entry.insert(
        "agent_id".into(),
        payload
            .get("agent_id")
            .and_then(Value::as_str)
            .map(|s| Value::String(s.to_string()))
            .unwrap_or(Value::Null),
    );
    entry.insert(
        "agent_type".into(),
        payload
            .get("agent_type")
            .and_then(Value::as_str)
            .map(|s| Value::String(s.to_string()))
            .unwrap_or(Value::Null),
    );

    // Number.isFinite(payload.duration_ms): only a genuine JSON number
    // qualifies (a JSON-parsed number is always finite — NaN/Infinity
    // cannot round-trip through JSON).
    if let Some(raw) = payload.get("duration_ms") {
        if raw.is_number() {
            entry.insert("duration_ms".into(), raw.clone());
        }
    }
    if let Some(status) = payload.get("tool_status").and_then(Value::as_str) {
        if !status.is_empty() {
            entry.insert("status".into(), Value::String(status.to_string()));
        }
    }

    let logs_dir = root.join(".bee").join("logs");
    let file = logs_dir.join("tools.jsonl");
    bee_core::fsutil::append_jsonl(&file, &Value::Object(entry)).map_err(|e| e.to_string())
}
