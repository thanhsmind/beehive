//! model_guard — Rust port of `.bee/bin/hooks/bee-model-guard.mjs`
//! (rust-port-10, CONTEXT.md D2/D7): PreToolUse (Agent|Task|spawn_agent)
//! dispatch-transport enforcement. This module is a THIN wrapper around
//! `bee_core::dispatch_guard::evaluate_dispatch` — the same seven deny
//! classes and allow paths the mjs source's `lib/dispatch-guard.mjs`
//! decides (decision 0023) — turning the decision into the exit code /
//! stderr / audit-log side effects the hook contract requires.
//!
//! `.bee/bin/hooks/bee-model-guard.mjs` + `.bee/bin/lib/dispatch-guard.mjs`
//! are FROZEN for the duration of the rust-port feature (D1) — this module
//! is conformance-checked against sha256-verified copies of them
//! (`crates/queen-bee/tests/modelguard_conformance.rs`), never edited to
//! "improve" on them. Deny reasons come from `dispatch_guard` verbatim, so
//! a deny's stderr is byte-identical across runtimes.
//!
//! Exit contract (D7b): allow = exit 0 (silent); deliberate deny = exit 2
//! with the reason on stderr; internal crash = exit 0 plus a
//! `.bee/logs/hooks.jsonl` crash line — proven via the SAME `run_fail_open`
//! wrapper `write_guard` already established (one fail-open implementation,
//! not a second hand-rolled copy).
//!
//! Every evaluated dispatch (allowed OR denied) appends one economics-audit
//! line to `.bee/logs/dispatch.jsonl` (g22-1/g22-2 parity) — fail-open,
//! never blocking the decision: `{ts, tool, transport, model, tier,
//! subagent_type, description, logical_tier, requested_model,
//! effective_model, effective_model_status, channel, enforcement}`.

use std::fs;
use std::io::Write as _;
use std::path::Path;

use serde_json::{json, Map, Value};

use bee_core::config::{self, ResolvePurpose};
use bee_core::dispatch_guard::{self, Decision, Economics};

use crate::adapter::{self, HookContext};
use crate::hookconfig;
use crate::hooks::write_guard::{crash_seam_panic_if_armed, run_fail_open};

const HOOK_NAME: &str = "model-guard";
const CODEX_SPAWN_TOOL: &str = "spawn_agent";
const DISPATCH_TOOLS: [&str; 2] = ["Agent", "Task"];

pub fn run(argv: &[String], raw_stdin: &str) -> i32 {
    let ctx: HookContext = adapter::read_hook_context(HOOK_NAME, argv, raw_stdin);
    let Some(root) = ctx.root.clone() else {
        return 0;
    };
    // Mirrors the mjs source's early existsSync guard: no state-lib
    // scaffold at all means this isn't (yet) a bee-onboarded runtime for
    // dispatch enforcement purposes — never a decision worth crash-logging.
    if !root.join(".bee").join("bin").join("lib").join("state.mjs").exists() {
        return 0;
    }
    let source = ctx.source.clone();
    run_fail_open(Some(&root), source.as_deref(), || {
        crash_seam_panic_if_armed(HOOK_NAME);
        decide(&ctx, &root)
    })
}

fn decide(ctx: &HookContext, root: &Path) -> i32 {
    let payload = &ctx.payload;
    // isCodexSpawn keys on the AUTHORITATIVE payload.tool_name field ONLY
    // (never the toolName fallback below) — mirrors the mjs source exactly.
    let is_codex_spawn = payload.get("tool_name").and_then(Value::as_str) == Some(CODEX_SPAWN_TOOL);
    let tool_name = payload
        .get("tool_name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .or_else(|| payload.get("toolName").and_then(Value::as_str))
        .unwrap_or("");
    if !is_codex_spawn && !DISPATCH_TOOLS.contains(&tool_name) {
        return 0;
    }

    if !hookconfig::hook_enabled(root, HOOK_NAME) {
        return 0;
    }

    let tool_input_val = payload.get("tool_input");
    let result = dispatch_guard::evaluate_dispatch(tool_name, tool_input_val, root);
    let Some(transport) = result.transport.clone() else {
        // "no opinion" — never log a dispatch line for it.
        return 0;
    };
    // A real transport is only ever produced when evaluate_dispatch's own
    // object check on tool_input passed, so it is a genuine object here;
    // an empty-object fallback is a defensive default only, never expected
    // to be exercised.
    let empty = Value::Object(Map::new());
    let tool_input = tool_input_val.unwrap_or(&empty);

    let economics = derive_dispatch_economics(root, is_codex_spawn, result.model.as_deref(), result.tier.as_deref());

    if result.decision == Decision::Deny {
        return deny_with(
            root,
            tool_name,
            tool_input,
            result.reason.as_deref().unwrap_or(""),
            &transport,
            result.model.as_deref(),
            result.tier.as_deref(),
            &economics,
        );
    }
    // Allow: log the evaluated transport exactly as every allow branch did
    // before the extraction (model-param / marker / codex-spawn-marker).
    log_dispatch(root, tool_name, tool_input, &transport, result.model.as_deref(), result.tier.as_deref(), &economics);
    0
}

/// mjs `deriveDispatchEconomics` — `resolved` is looked up fresh ONLY when
/// a tier is present, so a bare/unmarked dispatch never pays for a lookup
/// it cannot use.
fn derive_dispatch_economics(root: &Path, is_codex_spawn: bool, model: Option<&str>, tier: Option<&str>) -> Economics {
    let channel = if is_codex_spawn { "codex-native" } else { "claude-agent" };
    let runtime = if is_codex_spawn { "codex" } else { "claude" };
    let resolved = tier.map(|t| config::resolve_tier(root, t, runtime, ResolvePurpose::Cell));
    dispatch_guard::derive_economics(channel, tier, model, resolved.as_ref(), false)
}

/// mjs `logDispatch` — a log failure never changes the guard's decision or
/// exit code.
fn log_dispatch(root: &Path, tool_name: &str, tool_input: &Value, transport: &str, model: Option<&str>, tier: Option<&str>, economics: &Economics) {
    let logs_dir = root.join(".bee").join("logs");
    if fs::create_dir_all(&logs_dir).is_err() {
        return;
    }
    let description: String = tool_input.get("description").and_then(Value::as_str).map(|d| d.chars().take(120).collect()).unwrap_or_default();
    let subagent_type = tool_input.get("subagent_type").and_then(Value::as_str);

    let mut line = Map::new();
    line.insert("ts".to_string(), Value::String(adapter::timestamp_now()));
    line.insert("tool".to_string(), Value::String(tool_name.to_string()));
    line.insert("transport".to_string(), Value::String(transport.to_string()));
    line.insert("model".to_string(), model.map(Value::from).unwrap_or(Value::Null));
    line.insert("tier".to_string(), tier.map(Value::from).unwrap_or(Value::Null));
    line.insert("subagent_type".to_string(), subagent_type.map(Value::from).unwrap_or(Value::Null));
    line.insert("description".to_string(), Value::String(description));
    line.insert("logical_tier".to_string(), economics.logical_tier.clone().map(Value::String).unwrap_or(Value::Null));
    line.insert("requested_model".to_string(), economics.requested_model.clone().map(Value::String).unwrap_or(Value::Null));
    line.insert("effective_model".to_string(), economics.effective_model.clone().map(Value::String).unwrap_or(Value::Null));
    line.insert("effective_model_status".to_string(), Value::String(economics.effective_model_status.clone()));
    line.insert("channel".to_string(), Value::String(economics.channel.clone()));
    line.insert("enforcement".to_string(), Value::String(economics.enforcement.clone()));

    let mut text = serde_json::to_string(&Value::Object(line)).unwrap_or_default();
    text.push('\n');
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(logs_dir.join("dispatch.jsonl")) {
        let _ = f.write_all(text.as_bytes());
    }
}

/// mjs `logDeny` — `appendHookLog` is itself fail-open: a log failure never
/// blocks the deny.
fn log_deny(root: &Path, tool_name: &str, tool_input: &Value) {
    let keys: Vec<Value> = match tool_input {
        Value::Object(m) => m.keys().map(|k| Value::String(k.clone())).collect(),
        _ => Vec::new(),
    };
    adapter::append_hook_log(
        root,
        &json!({
            "ts": adapter::timestamp_now(),
            "hook": HOOK_NAME,
            "event": "deny",
            "tool_name": tool_name,
            "tool_input_keys": keys,
        }),
    );
}

/// mjs `denyWith` — records the rejected dispatch in the audit log (with
/// the transport label naming WHY it was rejected), appends the deny
/// event, and writes the reason to stderr for a PreToolUse exit 2. Both log
/// calls are fail-open — a log failure never changes the exit code.
fn deny_with(root: &Path, tool_name: &str, tool_input: &Value, reason: &str, transport: &str, model: Option<&str>, tier: Option<&str>, economics: &Economics) -> i32 {
    log_dispatch(root, tool_name, tool_input, transport, model, tier, economics);
    log_deny(root, tool_name, tool_input);
    let _ = std::io::stderr().write_all(reason.as_bytes());
    2
}
