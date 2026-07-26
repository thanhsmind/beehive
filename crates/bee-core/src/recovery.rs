//! recovery — the transcript-recovery readers ported from
//! `.bee/bin/lib/recovery.mjs` (`readTranscriptTail`, `detectCrashCandidates`
//! and its supporting helpers) plus the two `bee.mjs`-PRIVATE status
//! helpers this cell's panel named that live in the same problem space
//! (`buildRecoveryBlock`, `buildContentionSummary`) — rust-port-20,
//! CONTEXT.md D3/D5. Read-only, zero subprocess.
//!
//! `.bee/bin/lib/recovery.mjs`/`.bee/bin/bee.mjs` are FROZEN for the
//! duration of the rust-port feature (D1). Every function here takes an
//! already-resolved `control_root`/`projects_root` rather than re-deriving
//! it — the git-worktree walk (`controlRootFor`) and the host
//! `$HOME`-derived Claude projects root (`claudeProjectsRoot`'s default
//! call) are binary-crate concerns this cell's own action note places
//! outside `bee-core` (rust-port-16's established caller-supplies-
//! already-resolved-values pattern); this library never reads the host
//! `$HOME` on its own initiative in a way a hermetic test could not fully
//! override (D5).

use std::fs::{self, File};
use std::io::{Read as _, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::capture::capture_queue_path;
use crate::cells::{self, Cell};
use crate::claims;
use crate::config::read_config_value;
use crate::decisions::active_decisions;
use crate::fsutil::read_jsonl;
use crate::jsdate::parse_iso_ms;
use crate::lock::iso8601_millis;
use crate::state::read_lane;

/// recovery.mjs `DEFAULT_TAIL_MAX_BYTES` — `readTranscriptTail`'s default
/// read window (256KB).
pub const DEFAULT_TAIL_MAX_BYTES: u64 = 262_144;

/// Non-terminal lane phases (recovery.mjs `TERMINAL_LANE_PHASES` is the
/// inverse: this cell names the SET actually tested against — matches
/// `!TERMINAL_LANE_PHASES.has(phase)`).
const TERMINAL_LANE_PHASES: [&str; 2] = ["idle", "compounding-complete"];

fn to_ms(value: Option<&str>) -> Option<i64> {
    value.and_then(parse_iso_ms)
}

fn event_timestamp_ms(event: &Value) -> Option<i64> {
    if let Some(ts) = event.get("timestamp").and_then(Value::as_str) {
        return parse_iso_ms(ts);
    }
    if let Some(at) = event.get("at").and_then(Value::as_str) {
        return parse_iso_ms(at);
    }
    None
}

// --- (2) readTranscriptTail --------------------------------------------

/// `readTranscriptTail(file, maxBytes)` — reads only the last `max_bytes`
/// window of a (possibly multi-MB) transcript file. When the window
/// starts mid-file, the first sliced line is necessarily a truncated
/// fragment of a JSON line and is dropped rather than parsed. Malformed
/// lines within the window are skipped, never panicking. A missing/
/// unreadable file, or a zero-byte file, is `[]` — never a panic.
pub fn read_transcript_tail(file: &Path, max_bytes: u64) -> Vec<Value> {
    let size = match fs::metadata(file) {
        Ok(m) => m.len(),
        Err(_) => return Vec::new(),
    };
    if size == 0 {
        return Vec::new();
    }
    let start = size.saturating_sub(max_bytes);
    let len = (size - start) as usize;
    let mut fh = match File::open(file) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    if fh.seek(SeekFrom::Start(start)).is_err() {
        return Vec::new();
    }
    let mut buf = vec![0u8; len];
    if fh.read_exact(&mut buf).is_err() {
        return Vec::new();
    }
    let mut text = String::from_utf8_lossy(&buf).into_owned();
    if start > 0 {
        // The window did not begin at a line boundary — the first line is
        // a truncated fragment (its opening bytes are outside the read
        // window).
        match text.find('\n') {
            Some(idx) => text = text[idx + 1..].to_string(),
            None => text = String::new(),
        }
    }
    let mut events = Vec::new();
    for line in text.split('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            events.push(v);
        }
    }
    events
}

// --- (3) hasCleanEndTrio -------------------------------------------------

fn is_conversational(e: &Value) -> bool {
    matches!(e.get("type").and_then(Value::as_str), Some("user") | Some("assistant"))
}

/// `hasCleanEndTrio(events)` — true when the tail ends with the terminal
/// pattern system/stop_hook_summary -> system/turn_duration -> last-prompt,
/// with nothing conversational (`user`/`assistant`) after it. Any
/// non-conversational bookkeeping event is tolerated between/after the
/// trio's three markers.
pub fn has_clean_end_trio(events: &[Value]) -> bool {
    if events.is_empty() {
        return false;
    }

    let mut stop_idx: Option<usize> = None;
    for i in (0..events.len()).rev() {
        let e = &events[i];
        if e.get("type").and_then(Value::as_str) == Some("system")
            && e.get("subtype").and_then(Value::as_str) == Some("stop_hook_summary")
        {
            stop_idx = Some(i);
            break;
        }
        if is_conversational(e) {
            return false; // still mid-turn at the tail
        }
    }
    let Some(stop_idx) = stop_idx else { return false };

    let mut turn_idx: Option<usize> = None;
    for (i, e) in events.iter().enumerate().skip(stop_idx + 1) {
        if e.get("type").and_then(Value::as_str) == Some("system")
            && e.get("subtype").and_then(Value::as_str) == Some("turn_duration")
        {
            turn_idx = Some(i);
            break;
        }
        if is_conversational(e) {
            return false;
        }
    }
    let Some(turn_idx) = turn_idx else { return false };

    let mut last_prompt_idx: Option<usize> = None;
    for (i, e) in events.iter().enumerate().skip(turn_idx + 1) {
        if e.get("type").and_then(Value::as_str) == Some("last-prompt") {
            last_prompt_idx = Some(i);
            break;
        }
        if is_conversational(e) {
            return false;
        }
    }
    let Some(last_prompt_idx) = last_prompt_idx else { return false };

    for e in events.iter().skip(last_prompt_idx + 1) {
        if is_conversational(e) {
            return false;
        }
    }
    true
}

// --- (0) config-driven transcript roots (hardening-5) --------------------

fn normalize_transcript_roots_config(raw: &Value) -> Vec<(String, String)> {
    let Some(arr) = raw.as_array() else { return Vec::new() };
    let mut out = Vec::new();
    for entry in arr {
        let Value::Object(map) = entry else { continue };
        let runtime = map.get("runtime").and_then(Value::as_str).map(str::trim).unwrap_or("");
        let root_path = map.get("path").and_then(Value::as_str).map(str::trim).unwrap_or("");
        if runtime.is_empty() || root_path.is_empty() {
            continue;
        }
        out.push((runtime.to_string(), root_path.to_string()));
    }
    out
}

fn io_error_code(err: &std::io::Error) -> String {
    match err.kind() {
        std::io::ErrorKind::NotFound => "ENOENT".to_string(),
        std::io::ErrorKind::PermissionDenied => "EACCES".to_string(),
        _ => "unreadable".to_string(),
    }
}

/// `scanTranscriptRoots(root, {projectsRoot})` — the Claude default root
/// PLUS every configured extra root (`.bee/config.json`
/// `recovery.transcript_roots`), each tagged with its runtime and a scan
/// result. A bad CONFIGURED root gets exactly one stderr warning naming
/// its path; the Claude DEFAULT root's own missing case stays silent
/// (matches the mjs source's D2/hardening-5 contract).
pub fn scan_transcript_roots(root: &Path, projects_root: &Path) -> Vec<Value> {
    // rust-port-22: lowest-shared-primitive counter (see
    // `crate::read_accounting`'s module doc comment) — one increment per
    // CALL to this shared function (function-invocation unit, reconciles
    // with decision e119fc8b's stated "2"), distinct from the
    // per-fs::metadata-stat counter below (filesystem-operation unit,
    // which scales with how many roots are configured).
    crate::read_accounting::record_transcript_root_scan_invocation();
    let config = read_config_value(root);
    let configured_raw = config.get("recovery").and_then(|r| r.get("transcript_roots")).cloned().unwrap_or(Value::Null);
    let configured = normalize_transcript_roots_config(&configured_raw);

    let mut entries: Vec<(String, String, bool)> =
        vec![("claude".to_string(), projects_root.to_string_lossy().into_owned(), false)];
    for (rt, p) in configured {
        entries.push((rt, p, true));
    }

    entries
        .into_iter()
        .map(|(runtime, path_str, is_configured)| {
            let meta_result = fs::metadata(&path_str);
            // rust-port-22: the actual filesystem-operation counter (see
            // `crate::read_accounting`'s module doc comment) — one
            // increment per real `fs::metadata` stat, counted whether it
            // succeeds or not, deliberately separate from the
            // invocation-level counter above.
            crate::read_accounting::record_transcript_root_stat_op();
            let (scanned, reason) = match meta_result {
                Ok(meta) => {
                    if meta.is_dir() {
                        (true, None)
                    } else {
                        (false, Some("not-a-directory".to_string()))
                    }
                }
                Err(err) => (false, Some(io_error_code(&err))),
            };
            if !scanned && is_configured {
                eprintln!(
                    "recovery: configured transcript root \"{path_str}\" (runtime \"{runtime}\") is {} — skipping (config: recovery.transcript_roots)",
                    reason.as_deref().unwrap_or("unreadable")
                );
            }
            json!({"runtime": runtime, "path": path_str, "scanned": scanned, "reason": reason})
        })
        .collect()
}

// --- transcript location (perf.mjs subset this reader needs) --------------

/// perf.mjs `encodeProjectDir` — mirrors Claude Code's project-dir
/// encoding: the absolute project path with `/`, `\`, and `.` each
/// replaced by `-`.
pub fn encode_project_dir(project_path: &str) -> String {
    project_path.chars().map(|c| if c == '\\' || c == '/' || c == '.' { '-' } else { c }).collect()
}

/// perf.mjs `claudeProjectsRoot(env, homedir)`, parameterized (never reads
/// the process environment/home directory itself — D5's "never the host
/// $HOME" applies to every code path a hermetic test could reach, so the
/// override-able form is the one this crate exposes as its primary API).
pub fn claude_projects_root_with(config_dir_env: Option<&str>, home_dir: &Path) -> PathBuf {
    match config_dir_env {
        Some(s) if !s.is_empty() => Path::new(s).join("projects"),
        _ => home_dir.join(".claude").join("projects"),
    }
}

/// The production default: reads `CLAUDE_CONFIG_DIR`/`HOME` from the real
/// process environment. Never called by this cell's own hermetic tests
/// (D5) — provided for a future production call site (e.g. `queen-bee`'s
/// own `status` command).
pub fn claude_projects_root() -> PathBuf {
    let env = std::env::var("CLAUDE_CONFIG_DIR").ok();
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default();
    claude_projects_root_with(env.as_deref(), Path::new(&home))
}

/// perf.mjs `resolveTranscript(projectsRoot, projectPath, {sessionId,
/// transcriptPath})`. `transcript_path`, when non-blank and existing on
/// disk, is returned directly BEFORE any layout math runs at all
/// (hardening-1-7-10 D5). Otherwise both `projects_root` and
/// `project_path` are required; with `session_id`, the exact
/// `<enc>/<sessionId>.jsonl` path (or `None` if absent); without it, the
/// newest-mtime top-level `*.jsonl` in that directory (the live session),
/// or `None`.
pub fn resolve_transcript(
    projects_root: Option<&Path>,
    project_path: Option<&Path>,
    session_id: Option<&str>,
    transcript_path: Option<&str>,
) -> Option<PathBuf> {
    if let Some(tp) = transcript_path {
        let trimmed = tp.trim();
        if !trimmed.is_empty() && Path::new(trimmed).exists() {
            return Some(PathBuf::from(trimmed));
        }
    }
    let proot = projects_root?;
    let ppath = project_path?;
    let dir = proot.join(encode_project_dir(&ppath.to_string_lossy()));
    if let Some(sid) = session_id {
        let file = dir.join(format!("{sid}.jsonl"));
        return if file.exists() { Some(file) } else { None };
    }
    let entries = fs::read_dir(&dir).ok()?;
    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(mtime) = meta.modified() else { continue };
        let better = best.as_ref().map(|(_, m)| mtime > *m).unwrap_or(true);
        if better {
            best = Some((path, mtime));
        }
    }
    best.map(|(p, _)| p)
}

// --- lastDurableSettlement -------------------------------------------------

/// The three shared stores `lastDurableSettlement`/`detectCrashCandidates`
/// read — the cp-1 perf optimization lets a caller that already loaded
/// them once (this module's own `detect_crash_candidates` loop) thread
/// them straight through instead of re-reading per stale session.
pub struct SharedInputs {
    pub decisions: Vec<Value>,
    pub capture_events: Vec<Value>,
    pub cells: Vec<Cell>,
}

/// `lastDurableSettlement(root, lane, injected)` — the max timestamp
/// across decisions.jsonl, capture stubs, and cell-trace cap timestamps.
/// `lane` scopes capture stubs and cell traces when given; decisions.jsonl
/// carries no per-feature/lane field, so decisions are always read
/// globally. No settlement anywhere -> `None`.
pub fn last_durable_settlement(root: &Path, lane: Option<&str>, injected: Option<&SharedInputs>) -> Option<String> {
    let owned;
    let (decisions, capture_events, cells): (&[Value], &[Value], &[Cell]) = match injected {
        Some(inj) => (&inj.decisions, &inj.capture_events, &inj.cells),
        None => {
            owned = (
                active_decisions(root, None, false),
                read_jsonl::<Value>(&capture_queue_path(root)),
                cells::list_cells(root),
            );
            (&owned.0, &owned.1, &owned.2)
        }
    };

    let mut max_ms: Option<i64> = None;
    let mut bump = |ms: Option<i64>| {
        if let Some(v) = ms {
            if max_ms.map(|m| v > m).unwrap_or(true) {
                max_ms = Some(v);
            }
        }
    };

    for event in decisions {
        bump(to_ms(event.get("date").and_then(Value::as_str)));
    }
    for event in capture_events {
        if event.get("kind").and_then(Value::as_str) != Some("stub") {
            continue;
        }
        if let Some(l) = lane {
            if event.get("lane").and_then(Value::as_str) != Some(l) {
                continue;
            }
        }
        bump(to_ms(event.get("at").and_then(Value::as_str)));
    }
    for cell in cells {
        if let Some(l) = lane {
            if cell.feature() != Some(l) {
                continue;
            }
        }
        bump(to_ms(cell.capped_at()));
    }

    max_ms.map(iso8601_millis)
}

// --- work-signal helper ----------------------------------------------------

/// `sessionHasActiveClaim(root, sessionId, nowMs)` — true when
/// `session_id` currently holds at least one non-expired cell claim.
/// `control_root` is caller-supplied (msn-18b PLANE RULE: claims are
/// control-plane, resolved through `controlRootFor` in mjs — this port
/// takes the resolved root directly).
pub fn session_has_active_claim(control_root: &Path, session_id: &str, now_ms: i64) -> bool {
    let dir = claims::claims_dir(control_root);
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(cell_id) = name.strip_suffix(".json") else { continue };
        if let Some(claim) = claims::read_claim(control_root, cell_id) {
            if claim.session.as_deref() == Some(session_id) && claims::is_claim_active(&claim, now_ms) {
                return true;
            }
        }
    }
    false
}

/// `resolveSessionId({flag: currentSessionId})` as called from
/// `detectCrashCandidates` (no `root` argument passed there, so the
/// root-inference branch of the real `resolveSessionId` never fires for
/// THIS call site) — flag, then `BEE_SESSION_ID`, then
/// `CLAUDE_CODE_SESSION_ID`.
fn resolve_current_session_id(current_session_id: Option<&str>) -> Option<String> {
    if let Some(s) = current_session_id {
        let t = s.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    for var in ["BEE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"] {
        if let Ok(v) = std::env::var(var) {
            let t = v.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

fn or_null(s: &Option<String>) -> Value {
    match s {
        Some(v) if !v.is_empty() => Value::String(v.clone()),
        _ => Value::Null,
    }
}

// --- (1) detectCrashCandidates ---------------------------------------------

/// `detectCrashCandidates(root, {projectsRoot, projectPath, now,
/// currentSessionId})` — every session that is a "recoverable crash":
/// heartbeat-stale, not the live session, transcript exists and lacks the
/// clean-end trio, AND shows at least one work signal. `control_root` is
/// caller-supplied (msn-18b: sessions/claims are control-plane).
pub fn detect_crash_candidates(
    root: &Path,
    control_root: &Path,
    projects_root: &Path,
    project_path: &Path,
    now_ms: i64,
    current_session_id: Option<&str>,
) -> Vec<Value> {
    let resolved_current = resolve_current_session_id(current_session_id);
    let sessions = claims::list_session_records(control_root);
    if sessions.is_empty() {
        return Vec::new();
    }

    let roots = scan_transcript_roots(root, projects_root);

    let mut shared: Option<SharedInputs> = None;
    let mut candidates = Vec::new();

    for session in &sessions {
        if session.id.is_empty() {
            continue;
        }
        if resolved_current.as_deref() == Some(session.id.as_str()) {
            continue; // the live session is never a candidate
        }
        if !claims::heartbeat_stale(Some(session), now_ms, claims::DEFAULT_HEARTBEAT_STALE_SECONDS) {
            continue; // fresh heartbeat -> not stale, not a crash
        }

        let mut transcript: Option<PathBuf> = None;
        let mut transcript_runtime: Option<String> = None;

        let stored_path = session.transcript_path.as_deref().map(str::trim).filter(|s| !s.is_empty());
        if let Some(sp) = stored_path {
            if let Some(found) = resolve_transcript(None, None, None, Some(sp)) {
                let found_str = found.to_string_lossy().into_owned();
                let matched = roots.iter().find(|r| {
                    if r.get("scanned").and_then(Value::as_bool) != Some(true) {
                        return false;
                    }
                    let rp = r.get("path").and_then(Value::as_str).unwrap_or("");
                    let prefixed = if rp.ends_with(std::path::MAIN_SEPARATOR) {
                        rp.to_string()
                    } else {
                        format!("{rp}{}", std::path::MAIN_SEPARATOR)
                    };
                    found_str.starts_with(&prefixed)
                });
                transcript_runtime = matched.and_then(|r| r.get("runtime").and_then(Value::as_str)).map(str::to_string);
                transcript = Some(found);
            }
        }
        if transcript.is_none() {
            for r in &roots {
                if r.get("scanned").and_then(Value::as_bool) != Some(true) {
                    continue; // missing/unreadable root — already warned (if configured)
                }
                let rp = r.get("path").and_then(Value::as_str).unwrap_or("");
                if let Some(found) = resolve_transcript(Some(Path::new(rp)), Some(project_path), Some(&session.id), None) {
                    transcript = Some(found);
                    transcript_runtime = r.get("runtime").and_then(Value::as_str).map(str::to_string);
                    break;
                }
            }
        }
        let Some(transcript) = transcript else { continue }; // transcript must exist to prove an abrupt stop

        let tail = read_transcript_tail(&transcript, DEFAULT_TAIL_MAX_BYTES);
        if has_clean_end_trio(&tail) {
            continue; // clean stop -> excluded, not a crash
        }

        let lane = session.lane.clone().filter(|l| !l.is_empty());

        if shared.is_none() {
            shared = Some(SharedInputs {
                decisions: active_decisions(root, None, false),
                capture_events: read_jsonl::<Value>(&capture_queue_path(root)),
                cells: cells::list_cells(root),
            });
        }
        let since = last_durable_settlement(root, lane.as_deref(), shared.as_ref());
        let since_ms = since.as_deref().and_then(parse_iso_ms).or_else(|| session.started_at.as_deref().and_then(parse_iso_ms));

        let mut work_signal: Option<&'static str> = None;
        if let Some(ref l) = lane {
            if let Some(lane_record) = read_lane(root, l) {
                if !TERMINAL_LANE_PHASES.contains(&lane_record.phase.as_str()) {
                    work_signal = Some("lane");
                }
            }
        }
        if work_signal.is_none() && session_has_active_claim(control_root, &session.id, now_ms) {
            work_signal = Some("claimed_cells");
        }
        if work_signal.is_none() {
            let mut last_activity_ms: Option<i64> = None;
            for event in &tail {
                if let Some(t) = event_timestamp_ms(event) {
                    if last_activity_ms.map(|m| t > m).unwrap_or(true) {
                        last_activity_ms = Some(t);
                    }
                }
            }
            if let (Some(last), Some(since_v)) = (last_activity_ms, since_ms) {
                if last > since_v {
                    work_signal = Some("transcript_activity");
                }
            }
        }
        let Some(work_signal) = work_signal else { continue }; // heartbeat-stale with nothing at risk -> not worth recovering

        candidates.push(json!({
            "session_id": session.id,
            "lane": lane,
            "transcript": transcript.to_string_lossy(),
            "runtime": transcript_runtime,
            "started_at": or_null(&session.started_at),
            "last_heartbeat": or_null(&session.last_heartbeat),
            "work_signal": work_signal,
            "since": since_ms.map(iso8601_millis),
        }));
    }
    candidates
}

// --- buildRecoveryBlock (bee.mjs-private) ----------------------------------

/// `buildRecoveryBlock(root)` — `{candidates, roots}`. Every reader this
/// composes is already fail-open at every intermediate read (this
/// function never panics), matching the mjs source's belt-and-suspenders
/// try/catch (which exists there only in case some FUTURE change to those
/// readers' contract starts throwing).
pub fn build_recovery_block(
    root: &Path,
    control_root: &Path,
    projects_root: &Path,
    project_path: &Path,
    now_ms: i64,
    current_session_id: Option<&str>,
) -> Value {
    let candidates = detect_crash_candidates(root, control_root, projects_root, project_path, now_ms, current_session_id);
    let roots = scan_transcript_roots(root, projects_root);
    json!({"candidates": candidates, "roots": roots})
}

// --- buildContentionSummary (bee.mjs-private) ------------------------------

const CONTENTION_TAIL_MAX_BYTES: u64 = 65_536; // 64KB tail window
const CONTENTION_RECENT_BUSY_LIMIT: usize = 5;
const CONTENTION_TOP_LOCKS_LIMIT: usize = 5;

/// `buildContentionSummary(root)` — bounded tail-window summary of
/// `.bee/logs/contention.jsonl`. `None` (bee.mjs omits the `contention`
/// key entirely) when the log is absent OR the tail window has no
/// `result: "busy"` events. Every numeric field sourced from an event is
/// PASSED THROUGH the original parsed [`Value`] rather than round-tripped
/// via `f64` — `serde_json::Number`'s integer/float variants are not
/// `PartialEq` to each other even at equal numeric value, so a rebuilt
/// `json!(1500.0)` could mismatch a parsed `1500` in the oracle diff; a
/// verbatim clone can't drift from what the source line actually said.
pub fn build_contention_summary(root: &Path) -> Option<Value> {
    let file = root.join(".bee").join("logs").join("contention.jsonl");
    let events = read_transcript_tail(&file, CONTENTION_TAIL_MAX_BYTES);
    if events.is_empty() {
        return None;
    }
    let busy_events: Vec<&Value> = events.iter().filter(|e| e.get("result").and_then(Value::as_str) == Some("busy")).collect();
    if busy_events.is_empty() {
        return None;
    }

    let mut by_lock: Vec<(String, i64)> = Vec::new();
    for e in &busy_events {
        let name = e.get("lock_name").and_then(Value::as_str).filter(|s| !s.is_empty()).unwrap_or("unknown").to_string();
        if let Some(entry) = by_lock.iter_mut().find(|(n, _)| *n == name) {
            entry.1 += 1;
        } else {
            by_lock.push((name, 1));
        }
    }
    by_lock.sort_by(|a, b| b.1.cmp(&a.1)); // stable sort — ties keep first-seen (Map insertion) order, matching JS's stable Array.sort
    let top_locks: Vec<Value> =
        by_lock.iter().take(CONTENTION_TOP_LOCKS_LIMIT).map(|(name, count)| json!({"lock_name": name, "busy_count": count})).collect();

    // worst_wait_ms spans every event in the window (not just 'busy').
    let mut worst_wait_cmp: Option<f64> = None;
    let mut worst_wait_value: Value = Value::Null;
    let mut worst_wait_lock: Option<String> = None;
    for e in &events {
        let raw = e.get("lock_wait_ms");
        let cmp = raw.filter(|v| v.is_number()).and_then(Value::as_f64).filter(|f| f.is_finite());
        if let Some(w) = cmp {
            if worst_wait_cmp.map(|m| w > m).unwrap_or(true) {
                worst_wait_cmp = Some(w);
                worst_wait_value = raw.cloned().unwrap_or(Value::Null);
                worst_wait_lock = e.get("lock_name").and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_string);
            }
        }
    }

    let recent_busy: Vec<Value> = busy_events
        .iter()
        .rev()
        .take(CONTENTION_RECENT_BUSY_LIMIT)
        .map(|e| {
            let lock_wait_ms = e.get("lock_wait_ms").filter(|v| v.is_number()).cloned().unwrap_or(Value::Null);
            json!({
                "ts": e.get("ts").and_then(Value::as_str),
                "lock_name": e.get("lock_name").and_then(Value::as_str),
                "holder_session": e.get("holder_session").cloned().unwrap_or(Value::Null),
                "caller_session": e.get("caller_session").cloned().unwrap_or(Value::Null),
                "lock_wait_ms": lock_wait_ms,
            })
        })
        .collect();

    Some(json!({
        "busy_count": busy_events.len(),
        "recent_busy": recent_busy,
        "top_locks": top_locks,
        "worst_wait_ms": worst_wait_value,
        "worst_wait_lock": worst_wait_lock,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn read_transcript_tail_missing_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_transcript_tail(&dir.path().join("nope.jsonl"), 1024), Vec::<Value>::new());
    }

    #[test]
    fn read_transcript_tail_drops_truncated_leading_fragment() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("t.jsonl");
        let line1 = json!({"type": "assistant", "n": 1}).to_string();
        let line2 = json!({"type": "assistant", "n": 2}).to_string();
        write(&file, &format!("{line1}\n{line2}\n"));
        // A window landing 5 bytes INTO line1's content (never exactly on
        // a line boundary) must drop that truncated fragment and keep
        // only line2 — matching the mjs source's unconditional "start>0
        // -> drop up to the first newline" rule.
        let events = read_transcript_tail(&file, line2.len() as u64 + 1 + 5);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["n"], json!(2));
    }

    #[test]
    fn read_transcript_tail_window_landing_exactly_on_a_line_boundary_still_drops_that_line() {
        // Not a divergence from mjs: `readTranscriptTail`'s `start > 0`
        // branch drops up to the first newline UNCONDITIONALLY, even when
        // `start` happens to land exactly at the next line's first byte —
        // the mjs source has no "already aligned" special case, so this
        // port must not add one either (D1 freeze: byte-identical, not
        // "improved").
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("t.jsonl");
        let line1 = json!({"type": "assistant", "n": 1}).to_string();
        let line2 = json!({"type": "assistant", "n": 2}).to_string();
        write(&file, &format!("{line1}\n{line2}\n"));
        let events = read_transcript_tail(&file, line2.len() as u64 + 1);
        assert_eq!(events, Vec::<Value>::new());
    }

    #[test]
    fn has_clean_end_trio_true_for_the_terminal_pattern() {
        let events = vec![
            json!({"type": "assistant"}),
            json!({"type": "system", "subtype": "stop_hook_summary"}),
            json!({"type": "system", "subtype": "turn_duration"}),
            json!({"type": "last-prompt"}),
        ];
        assert!(has_clean_end_trio(&events));
    }

    #[test]
    fn has_clean_end_trio_false_when_conversational_after() {
        let events = vec![
            json!({"type": "system", "subtype": "stop_hook_summary"}),
            json!({"type": "system", "subtype": "turn_duration"}),
            json!({"type": "last-prompt"}),
            json!({"type": "user"}),
        ];
        assert!(!has_clean_end_trio(&events));
    }

    #[test]
    fn encode_project_dir_matches_mjs() {
        assert_eq!(encode_project_dir("/a/b/c"), "-a-b-c");
        assert_eq!(encode_project_dir("/tmp/x.y/1"), "-tmp-x-y-1");
    }
}
