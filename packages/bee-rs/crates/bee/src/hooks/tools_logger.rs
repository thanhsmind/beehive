// bee hook tools-logger — port of hooks/bee-tools-logger.mjs: PostToolUse
// passive measurement, zero enforcement. Can NEVER deny, never writes a
// verdict, always exits 0. Line schema {ts, tool_name, agent_id, agent_type}
// (+duration_ms/status only when the payload carries them); tool bodies are
// NEVER logged.

use crate::hooks::adapter::{log_crash, now_iso, read_hook_context};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::Path;
use std::process::ExitCode;

const HOOK_NAME: &str = "tools-logger";

fn log_tool_call(root: &Path, payload: &Map<String, Value>) -> std::io::Result<()> {
    let logs_dir = root.join(".bee").join("logs");
    std::fs::create_dir_all(&logs_dir)?;
    let str_or = |key: &str| payload.get(key).and_then(Value::as_str);
    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(now_iso()));
    entry.insert("tool_name".into(), Value::String(str_or("tool_name").unwrap_or("").to_string()));
    entry.insert("agent_id".into(), str_or("agent_id").map_or(Value::Null, |s| Value::String(s.to_string())));
    entry.insert("agent_type".into(), str_or("agent_type").map_or(Value::Null, |s| Value::String(s.to_string())));
    if let Some(n) = payload.get("duration_ms").and_then(Value::as_f64) {
        if n.is_finite() {
            entry.insert("duration_ms".into(), payload["duration_ms"].clone());
        }
    }
    if let Some(s) = str_or("tool_status") {
        if !s.is_empty() {
            entry.insert("status".into(), Value::String(s.to_string()));
        }
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs_dir.join("tools.jsonl"))?;
    f.write_all(format!("{}\n", jsjson::stringify(&Value::Object(entry))).as_bytes())
}

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    Outcome::Done(run_inner(argv, stdin))
}

fn run_inner(argv: &[String], stdin: &str) -> ExitCode {
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = &ctx.root else {
        return ExitCode::SUCCESS;
    };
    // Vendored-lib presence gate, kept for behavior parity on hosts without
    // a .bee/bin frame.
    if !crate::hooks::adapter::bee_installed(root) {
        return ExitCode::SUCCESS;
    }
    let enabled = match hook_enabled(root, HOOK_NAME) {
        Ok(e) => e,
        Err(_) => {
            // The .mjs surfaces a corrupt-config readJson warn via Node; a
            // logger must stay silent-fail-open here — log the crash shape.
            log_crash(Some(root), HOOK_NAME, "config unreadable", ctx.source);
            return ExitCode::SUCCESS;
        }
    };
    if !enabled {
        return ExitCode::SUCCESS;
    }
    if let Err(e) = log_tool_call(root, &ctx.payload) {
        log_crash(Some(root), HOOK_NAME, &format!("Error: {e}"), ctx.source);
    }
    ExitCode::SUCCESS
}
