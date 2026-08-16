// the capture, decision and bypass nudges
//
// Split out of the single 2.8k-line hooks/session_close.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, ReadJson};
use crate::hooks::adapter::{emit_hook_output, encode_block, log_crash, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson::{self, js_to_string};
use crate::state::{bypass_level, capture_queue_threshold, read_config_raw};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── maybeCaptureQueueNudge (decision 0017) ────────────────────────────────

/// Pending (stub-minus-flush) capture-queue rows, in file order — the same
/// membership `pending_capture_stub_ids` used to compute on its own; kept
/// here as the single read so U3's threshold check (count + oldest `at`)
/// and the nudge's dedup hash (ids only) share one pass over the file.
pub(crate) fn pending_capture_stubs(root: &Path) -> Vec<Value> {
    let events = read_jsonl(&root.join(".bee").join("capture-queue.jsonl"));
    let mut flushed: HashSet<String> = HashSet::new();
    let mut stubs: Vec<Value> = Vec::new();
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
            stubs.push(event.clone());
        }
    }
    stubs
        .into_iter()
        .filter(|s| {
            s.get("id").and_then(primitive_key).map(|k| !flushed.contains(&k)).unwrap_or(true)
        })
        .collect()
}

/// The oldest pending stub's `at`, in epoch ms — NaN when there is no
/// pending stub or its `at` doesn't parse (an unresolvable timestamp is
/// never treated as a breach, same fallback shape the rest of this hook
/// uses for unparseable dates).
pub(crate) fn oldest_pending_stub_at_ms(stubs: &[Value]) -> f64 {
    stubs
        .iter()
        .filter_map(|s| js_date_parse_value(s.get("at").unwrap_or(&Value::Null)))
        .fold(f64::NAN, |acc, ms| if acc.is_nan() || ms < acc { ms } else { acc })
}

pub(crate) fn maybe_capture_queue_nudge(
    root: &Path,
) -> Result<Option<String>, Flow> {
    let stubs = pending_capture_stubs(root);
    if stubs.is_empty() {
        return Ok(None);
    }
    let mut ids: Vec<String> =
        stubs.iter().map(|s| js_to_string(s.get("id").unwrap_or(&Value::Null))).collect();
    ids.sort(); // JS default sort over string ids
    let hash = ids.join("|");
    if !should_inject(root, "capture-queue-nudge", &hash)? {
        return Ok(None);
    }
    mark_injected(root, "capture-queue-nudge", &hash)?;
    let count = stubs.len();
    // U3: past the configured threshold — count exceeds it, OR the oldest
    // pending stub is older than the configured day count — the nudge
    // escalates to overdue wording naming the breach. Never a hard block:
    // this is still an advisory Stop message, same as the wording below.
    let config = read_config_raw(root);
    let threshold = capture_queue_threshold(&config);
    let oldest_ms = oldest_pending_stub_at_ms(&stubs);
    let oldest_age_days = if oldest_ms.is_nan() { None } else { Some((now_ms() - oldest_ms) / 86_400_000.0) };
    let over_count = count as u64 > threshold.count;
    let over_age = oldest_age_days.map(|d| d > threshold.days).unwrap_or(false);
    if over_count || over_age {
        let oldest_days = oldest_age_days.unwrap_or(0.0).max(0.0).floor() as u64;
        return Ok(Some(format!(
            "bee capture queue (decision 0017): OVERDUE — {count} stub(s) pending, oldest \
{oldest_days} days — flush before new work. Flush them now via bee-capturing (drain \
oldest-first, merge each into its area spec) — or they must survive into the next \
session's preamble, never be dropped."
        )));
    }
    Ok(Some(format!(
        "bee capture queue (decision 0017): {count} settlement stub(s) are queued and \
unflushed. Flush them now via bee-capturing (drain oldest-first, merge each into its \
area spec) — or they must survive into the next session's preamble, never be dropped."
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
--decision \"...\" --rationale \"...\" --relation none (or supersedes:<id>/touches:<id> \
if it changes an earlier one) (or a dated learning in docs/history/learnings/). \
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
--rationale \"<the digest, or the path holding it>\" --relation none — do this BEFORE \
setting the gate. "
    } else {
        ""
    };
    Ok(Some(format!(
        "⚡ GATE BYPASS ({level}): you are stopping mid-{phase} with Gate {gate_no} \
(shape+execution) still pending, but bypass level \"{level}\" requires auto-approval at \
this lane — do NOT ask the human. {consult_sentence}Set the gate yourself now: \
bee state gate --merge --approved true ; log a one-line \
audit decision (bee decisions log --decision \"auto-approved Gate \
{gate_no} (bypass): <choice>\" --rationale \"<why>\" --relation none); post the short \"⚡ auto-approved \
Gate {gate_no} (bypass)\" line; then CONTINUE to the next phase. Do not re-emit the \
gate question. (If you genuinely need information only the human holds — not a \
rubber-stamp — ask that specific question instead; this net blocks once, then steps \
aside.)"
    )))
}

// ─── tests: U3 capture-queue pressure escalation ───────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn queue_path(root: &Path) -> PathBuf {
        root.join(".bee").join("capture-queue.jsonl")
    }

    fn write_queue(root: &Path, lines: &[String]) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(queue_path(root), format!("{}\n", lines.join("\n"))).unwrap();
    }

    fn stub(id: &str, at: &str) -> String {
        format!(r#"{{"kind":"stub","id":"{id}","at":"{at}","outcome":"x"}}"#)
    }

    fn write_config(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("config.json"), content).unwrap();
    }

    /// Below the default threshold (5 stubs, 7 days): wording stays exactly
    /// what it was before U3 — byte-identical, per the must_haves contract.
    #[test]
    fn under_threshold_wording_is_byte_identical_to_before() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_queue(root, &[stub("s1", &now_iso())]);
        let msg = maybe_capture_queue_nudge(root).unwrap().unwrap();
        assert_eq!(
            msg,
            "bee capture queue (decision 0017): 1 settlement stub(s) are queued and \
unflushed. Flush them now via bee-capturing (drain oldest-first, merge each into its \
area spec) — or they must survive into the next session's preamble, never be dropped."
        );
    }

    #[test]
    fn over_count_threshold_escalates_to_overdue_wording() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = now_iso();
        let lines: Vec<String> = (0..6).map(|i| stub(&format!("s{i}"), &now)).collect();
        write_queue(root, &lines);
        let msg = maybe_capture_queue_nudge(root).unwrap().unwrap();
        assert!(
            msg.starts_with(
                "bee capture queue (decision 0017): OVERDUE — 6 stub(s) pending, oldest 0 days — flush before new work."
            ),
            "{msg}"
        );
    }

    #[test]
    fn over_age_threshold_escalates_even_under_count() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let old = ms_to_iso(now_ms() - 10.0 * 86_400_000.0).unwrap();
        write_queue(root, &[stub("s1", &old)]);
        let msg = maybe_capture_queue_nudge(root).unwrap().unwrap();
        assert!(
            msg.starts_with(
                "bee capture queue (decision 0017): OVERDUE — 1 stub(s) pending, oldest 10 days — flush before new work."
            ),
            "{msg}"
        );
    }

    #[test]
    fn configured_threshold_overrides_the_default() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{"capture_queue_threshold":{"count":1,"days":30}}"#);
        let now = now_iso();
        write_queue(root, &[stub("s1", &now), stub("s2", &now)]);
        let msg = maybe_capture_queue_nudge(root).unwrap().unwrap();
        assert!(
            msg.starts_with("bee capture queue (decision 0017): OVERDUE — 2 stub(s) pending"),
            "{msg}"
        );
    }

    /// A malformed threshold config falls back to the default (5, 7) — the
    /// nudge itself never hard-blocks, just as a healthy config would keep
    /// wording unescalated below the default.
    #[test]
    fn malformed_threshold_config_falls_back_to_default_and_never_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_config(root, r#"{"capture_queue_threshold":{"count":-1,"days":7}}"#);
        write_queue(root, &[stub("s1", &now_iso())]);
        let msg = maybe_capture_queue_nudge(root).unwrap().unwrap();
        assert!(msg.starts_with("bee capture queue (decision 0017): 1 settlement stub(s)"), "{msg}");
    }
}
