// status --brief — the first natively-served porcelain path (R1 exit
// criterion). Mirrors bee.mjs end to end for the accepted argv shapes:
//   root resolution -> manifest-drift check (cache write) -> buildStatusBrief
//   -> emit (drift stderr line, stdout payload) -> timing (timings.jsonl +
//   "[bee] status Nms" stderr line).
//
// Conservative routing: `try_native` accepts ONLY argv where every token
// after `status` is exactly `--brief` or `--json` (with `--brief` present).
// Anything else — including linked-worktree roots and corrupt JSON inputs —
// returns None before ANY output, and the whole command re-runs under Node.

use crate::fsutil::append_jsonl;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::state::{bypass_level, read_config_raw, read_state_brief, ship_visibility, GATE_NAMES};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "status" {
        return None;
    }
    let mut brief = false;
    let mut use_json = false;
    for a in &args[1..] {
        match a.to_str()? {
            "--brief" => brief = true,
            "--json" => use_json = true,
            _ => return None,
        }
    }
    if !brief {
        return None;
    }
    run(use_json, t0)
}

fn run(use_json: bool, t0: Instant) -> Option<ExitCode> {
    let cwd = std::env::current_dir().ok()?;

    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::NeedsNode => return None,
        Roots::None => {
            // main()'s no-root emitError path, then the timing wrapper.
            let msg = "No bee repo root found (no .bee/onboarding.json or .git up the tree). Run bee-hive onboarding.";
            if use_json {
                println!("{}", jsjson::stringify(&json!({ "error": msg })));
            } else {
                eprintln!("{msg}");
            }
            record_timing(&cwd, t0, false);
            return Some(ExitCode::FAILURE);
        }
    };

    let drift = check_manifest_drift(&root).ok()?;
    let state = read_state_brief(&root).ok()?;
    let config = read_config_raw(&root).ok()?;

    // buildStatusBrief's frozen 7-key order.
    let mut result = Map::new();
    result.insert("phase".into(), state.phase);
    result.insert("feature".into(), state.feature);
    result.insert("mode".into(), state.mode);
    result.insert("gates".into(), Value::Object(state.gates));
    result.insert("gate_bypass_level".into(), json!(bypass_level(&config)));
    result.insert("ship_visibility".into(), json!(ship_visibility(&config)));
    result.insert("route".into(), state.route);

    if drift.manifest_changed {
        eprintln!("manifest_changed: true — {}", drift.hint);
    }
    if use_json {
        println!("{}", jsjson::stringify_pretty(&Value::Object(result)));
    } else {
        println!("{}", render_brief_text(&result));
    }
    record_timing(&root, t0, true);
    Some(ExitCode::SUCCESS)
}

/// renderStatusBriefText: `phase=<p> feature=<f> mode=<m> gates=<c/s/e/r> bypass=<lvl>`.
fn render_brief_text(result: &Map<String, Value>) -> String {
    let js_truthy = |v: &Value| match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        _ => true,
    };
    let gates_obj = result.get("gates").and_then(Value::as_object);
    let gates = GATE_NAMES
        .iter()
        .map(|g| {
            let v = gates_obj.and_then(|m| m.get(*g)).map(js_truthy).unwrap_or(false);
            if v { "true" } else { "false" }
        })
        .collect::<Vec<_>>()
        .join("/");
    // `${brief.feature ?? 'none'}` — null coalesces, everything else
    // stringifies raw.
    let coalesce = |key: &str| -> String {
        match result.get(key) {
            None | Some(Value::Null) => "none".to_string(),
            Some(v) => jsjson::js_to_string(v),
        }
    };
    format!(
        "phase={} feature={} mode={} gates={} bypass={}",
        result.get("phase").map(jsjson::js_to_string).unwrap_or_default(),
        coalesce("feature"),
        coalesce("mode"),
        gates,
        result.get("gate_bypass_level").map(jsjson::js_to_string).unwrap_or_default(),
    )
}

/// The direct-run timing wrapper: append .bee/logs/timings.jsonl (fail-open)
/// and always print the stderr line.
fn record_timing(root: &Path, t0: Instant, ok: bool) {
    let ms = t0.elapsed().as_millis() as u64;
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let _ = append_jsonl(
        &root.join(".bee").join("logs").join("timings.jsonl"),
        &json!({"ts": ts, "cmd": "status", "ms": ms, "ok": ok}),
    );
    eprintln!("[bee] status {ms}ms");
}
