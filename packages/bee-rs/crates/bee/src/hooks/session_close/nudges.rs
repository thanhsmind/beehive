// the capture, decision and bypass nudges
//
// Split out of the single 2.8k-line hooks/session_close.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, ReadJson};
use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson::{self, js_to_string};
use crate::state::{bypass_level, read_config_raw};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── maybeCaptureQueueNudge (decision 0017) ────────────────────────────────

pub(crate) fn pending_capture_stub_ids(root: &Path) -> Vec<String> {
    let events = read_jsonl(&root.join(".bee").join("capture-queue.jsonl"));
    let mut flushed: HashSet<String> = HashSet::new();
    let mut stubs: Vec<&Value> = Vec::new();
    for event in &events {
        if !event.is_object() {
            continue;
        }
        let kind = event.get("kind").and_then(Value::as_str);
        let id_truthy = event.get("id").map(js_truthy).unwrap_or(false);
        if kind == Some("flush") && id_truthy {
            if let Some(k) = event.get("id").and_then(primitive_key) {
                flushed.insert(k);
            }
        } else if kind == Some("stub") && id_truthy {
            stubs.push(event);
        }
    }
    stubs
        .into_iter()
        .filter(|s| {
            s.get("id").and_then(primitive_key).map(|k| !flushed.contains(&k)).unwrap_or(true)
        })
        .map(|s| js_to_string(s.get("id").unwrap_or(&Value::Null)))
        .collect()
}

pub(crate) fn maybe_capture_queue_nudge(
    root: &Path,
) -> Result<Option<String>, Flow> {
    let pending = pending_capture_stub_ids(root);
    if pending.is_empty() {
        return Ok(None);
    }
    let mut ids = pending.clone();
    ids.sort(); // JS default sort over string ids
    let hash = ids.join("|");
    if !should_inject(root, "capture-queue-nudge", &hash)? {
        return Ok(None);
    }
    mark_injected(root, "capture-queue-nudge", &hash)?;
    Ok(Some(format!(
        "bee capture queue (decision 0017): {} settlement stub(s) are queued and \
unflushed. Flush them now via bee-capturing (drain oldest-first, merge each into its \
area spec) — or they must survive into the next session's preamble, never be dropped.",
        pending.len()
    )))
}

// ─── maybeCaptureNudge (decision 0003) ─────────────────────────────────────

/// state.mjs resolveProductRoot — warnings replicated byte-for-byte; emitted
/// on EVERY call, exactly as the .mjs re-runs it per call site.
pub(crate) fn resolve_product_root(root: &Path, config: &Map<String, Value>, stderr: &mut String) -> PathBuf {
    let configured = match config.get("product_root") {
        None | Some(Value::Null) => return root.to_path_buf(),
        Some(Value::String(s)) if s.is_empty() => return root.to_path_buf(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => {
            let type_of = match other {
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                _ => "object",
            };
            stderr.push_str(&format!(
                "bee: .bee/config.json product_root must be a string path (got {type_of}); \
ignoring it and using the bee root.\n"
            ));
            return root.to_path_buf();
        }
    };
    let candidate = Path::new(&configured);
    let resolved: PathBuf = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        normalize_join(root, candidate)
    };
    let is_dir = std::fs::metadata(&resolved).map(|m| m.is_dir()).unwrap_or(false);
    if !is_dir {
        stderr.push_str(&format!(
            "bee: config product_root \"{configured}\" -> \"{}\" is not an existing directory; \
product-doc reads (docs/backlog.md, docs/specs/) will find nothing until you fix \
.bee/config.json product_root. (GitHub #14)\n",
            resolved.display()
        ));
    }
    resolved
}

/// path.resolve(root, rel) for ordinary relative paths ('.'/'..' collapsed).
pub(crate) fn normalize_join(root: &Path, rel: &Path) -> PathBuf {
    let mut out = root.to_path_buf();
    for comp in rel.components() {
        use std::path::Component;
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// The newest .md mtimeMs under dir (flat or recursive). Err = fs error the
/// .mjs would throw (caught by the nudge's own try → null).
pub(crate) fn newest_md(dir: &Path, recursive: bool) -> Result<f64, ()> {
    let mut newest = 0.0f64;
    if !dir.exists() {
        return Ok(newest);
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(&current).map_err(|_| ())?;
        for entry in entries {
            let entry = entry.map_err(|_| ())?;
            let ft = entry.file_type().map_err(|_| ())?;
            if ft.is_dir() {
                if recursive {
                    stack.push(entry.path());
                }
                continue;
            }
            if !ft.is_file() || !entry.file_name().to_string_lossy().ends_with(".md") {
                continue;
            }
            let meta = std::fs::metadata(entry.path()).map_err(|_| ())?;
            let mtime = meta
                .modified()
                .map_err(|_| ())?
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64() * 1000.0)
                .unwrap_or(0.0);
            if mtime > newest {
                newest = mtime;
            }
        }
    }
    Ok(newest)
}

pub(crate) fn maybe_capture_nudge(
    root: &Path,
    config: &Map<String, Value>,
    stderr: &mut String,
) -> Result<Option<String>, Flow> {
    // state.mjs is import-gated by the wrapper already; resolveProductRoot may
    // warn (replicated), then the docs trees are checked.
    let product_root = resolve_product_root(root, config, stderr);
    let specs_dir = product_root.join("docs").join("specs");
    let knowledge_dir = product_root.join("docs").join("knowledge");
    if !specs_dir.exists() && !knowledge_dir.exists() {
        return Ok(None);
    }
    let Some((id, date)) = newest_active_decision(root) else { return Ok(None) };
    let decision_ts = if js_truthy(&date) { js_date_parse_value(&date) } else { None };
    let Some(decision_ts) = decision_ts.filter(|t| *t != 0.0) else {
        return Ok(None); // !decisionTs (0 or NaN)
    };
    let Ok(newest_spec) = newest_md(&specs_dir, false).and_then(|a| Ok(a.max(newest_md(&knowledge_dir, true)?)))
    else {
        return Ok(None); // fs throw → catch → null
    };
    if decision_ts <= newest_spec {
        return Ok(None);
    }
    let hash_src = if js_truthy(&id) { id } else { date };
    let hash = js_to_string(&hash_src);
    if !should_inject(root, "capture-nudge", &hash)? {
        return Ok(None);
    }
    mark_injected(root, "capture-nudge", &hash)?;
    // knowledge.mjs is imported AFTER the mark in the .mjs — a missing module
    // throws there and the nudge is consumed without being emitted.
    if bundle_mode(root, config, stderr) {
        return Ok(Some(
            "bee capture nudge (decision 0003): the newest decision is more recent than every \
concept in the knowledge bundle (docs/knowledge/) — a settled outcome may exist only \
in the decision log and the chat. Before finishing, invoke bee-capturing capture to \
author it as a concept in the touched area's bundle folder (or confirm no area is affected)."
                .to_string(),
        ));
    }
    Ok(Some(
        "bee capture nudge (decision 0003): the newest decision is more recent than every \
area spec under docs/specs/ — a settled outcome may exist only in the decision log \
and the chat. Before finishing, invoke bee-capturing capture to merge it into the \
touched area's spec (or confirm no spec is affected)."
            .to_string(),
    ))
}

// ─── maybeDecisionNudge (repository-harness lesson) ────────────────────────

pub(crate) fn git_status_porcelain(root: &Path) -> Result<String, ()> {
    use std::process::{Command, Stdio};
    let mut child = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ())?;
    let mut out = child.stdout.take().ok_or(())?;
    let reader = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = out.read_to_end(&mut buf);
        buf
    });
    let start = std::time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed().as_millis() > 3000 {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(()); // execSync timeout → throw → catch → null
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(_) => return Err(()),
        }
    };
    let buf = reader.join().map_err(|_| ())?;
    if !status.success() {
        return Err(()); // execSync non-zero exit → throw
    }
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

pub(crate) fn nudge_allowed(path: &str) -> bool {
    // /^(\.bee\/|docs\/|plans\/|AGENTS\.md$)/
    path.starts_with(".bee/") || path.starts_with("docs/") || path.starts_with("plans/") || path == "AGENTS.md"
}

pub(crate) fn maybe_decision_nudge(root: &Path) -> Result<Option<String>, Flow> {
    let Ok(out) = git_status_porcelain(root) else { return Ok(None) };
    let mut changed: Vec<String> = out
        .split('\n')
        .map(|line| {
            // line.slice(3).trim().replace(/^"|"$/g, "")
            let sliced: String = line.chars().skip(3).collect();
            let mut s = sliced.trim();
            if let Some(x) = s.strip_prefix('"') {
                s = x;
            }
            if let Some(x) = s.strip_suffix('"') {
                s = x;
            }
            s.to_string()
        })
        .filter(|p| !p.is_empty())
        .filter(|p| !nudge_allowed(p))
        .collect();
    if changed.is_empty() {
        return Ok(None);
    }
    let last_ts = newest_active_decision(root)
        .and_then(|(_, date)| if js_truthy(&date) { js_date_parse_value(&date) } else { Some(0.0) })
        .unwrap_or(0.0);
    if last_ts != 0.0 && !last_ts.is_nan() && now_ms() - last_ts < DECISION_RECENT_MS {
        return Ok(None);
    }
    let count = changed.len();
    changed.sort();
    let hash = changed.join("|");
    if !should_inject(root, "decision-nudge", &hash)? {
        return Ok(None);
    }
    mark_injected(root, "decision-nudge", &hash)?;
    Ok(Some(format!(
        "bee decision review: {count} source file(s) changed with no bee flow active \
and no recent decision logged. Before finishing, ask the user: is there a durable \
decision or convention here worth recording? If yes: bee decisions log \
--decision \"...\" --rationale \"...\" (or a dated learning in docs/history/learnings/). \
If not, carry on."
    )))
}

// ─── maybeBypassBlock (GitHub #18) ─────────────────────────────────────────

pub(crate) fn level_covers_gate(level: &str, mode: &Value) -> bool {
    if level == "total" || level == "full" {
        return true;
    }
    if level == "normal" {
        return matches!(mode, Value::String(s) if matches!(s.as_str(), "tiny" | "small" | "standard"));
    }
    false
}

pub(crate) fn maybe_bypass_block(
    root: &Path,
    ctx: &HookContext,
    config: &Map<String, Value>,
    session_id: Option<&str>,
    stderr: &mut String,
) -> Result<Option<String>, Flow> {
    if ctx.event != "Stop" {
        return Ok(None);
    }
    let level = bypass_level(config);
    if level == "off" {
        return Ok(None);
    }
    let pipeline = resolve_pipeline(root, ctx, session_id, stderr)?;
    let record = match pipeline {
        Pipeline::Ok { record } => record,
        Pipeline::Refused => read_state_record(root)?,
    };
    let phase_val = if js_truthy(&record.phase) { record.phase.clone() } else { Value::String("idle".into()) };
    // PHASE_GATE property lookup coerces the key to a string.
    let phase = js_to_string(&phase_val);
    if phase != "planning" {
        return Ok(None);
    }
    let gate = "execution";
    let mode = if js_truthy(&record.mode) { record.mode.clone() } else { Value::Null };
    if !level_covers_gate(level, &mode) {
        return Ok(None);
    }
    // Gate 2 is the MERGED shape+execution approval, so it has passed only
    // when BOTH components are true. Testing `execution` alone would let a
    // half-open merged gate slip past the very net that exists to close it:
    // the standalone `--name` path can still grant one component without the
    // other, and a record in that state is not an approved Gate 2.
    let merged_passed = ["shape", gate]
        .iter()
        .all(|field| record.gates.get(*field) == Some(&Value::Bool(true)));
    if merged_passed {
        return Ok(None); // gate already passed — nothing to force
    }
    let key = "bypass-stop-net";
    let hash = format!("{}:{phase}:{gate}:{level}", session_id.unwrap_or("nosession"));
    if !should_inject(root, key, &hash)? {
        return Ok(None);
    }
    mark_injected(root, key, &hash)?;

    // Planning's gate is the merged shape+execution approval, numbered 2 since
    // validation-diet D2 folded the old standalone execution gate (then Gate 3)
    // into it. The gate field is never "shape" here — PHASE_GATE maps
    // planning→execution — so the label names the component being set.
    let gate_no = "2";
    let consult_sentence = if mode == Value::String("high-risk".into()) {
        // `bee state advisor-ref record` used to be prescribed here. It is
        // declared in the registry and NOT built into this binary (the R6
        // Node deletion took its only implementation), so this hook was
        // handing the agent a command that answers with a refusal — at the
        // one moment the agent is being told not to ask the human. It names
        // the working record instead; the registry marker on
        // `state.advisor-ref.record` carries the same fix.
        "High-risk execution requires a live advisor consult first: resolve the advisor from \
config (models.<runtime>.advisor), run it read-only with the evidence bundle on stdin, then \
record it with bee decisions log --decision \"advisor consult: <identity>\" \
--rationale \"<the digest, or the path holding it>\" — do this BEFORE setting the gate. "
    } else {
        ""
    };
    Ok(Some(format!(
        "⚡ GATE BYPASS ({level}): you are stopping mid-{phase} with Gate {gate_no} \
(shape+execution) still pending, but bypass level \"{level}\" requires auto-approval at \
this lane — do NOT ask the human. {consult_sentence}Set the gate yourself now: \
bee state gate --merge --approved true ; log a one-line \
audit decision (bee decisions log --decision \"auto-approved Gate \
{gate_no} (bypass): <choice>\" --rationale \"<why>\"); post the short \"⚡ auto-approved \
Gate {gate_no} (bypass)\" line; then CONTINUE to the next phase. Do not re-emit the \
gate question. (If you genuinely need information only the human holds — not a \
rubber-stamp — ask that specific question instead; this net blocks once, then steps \
aside.)"
    )))
}
