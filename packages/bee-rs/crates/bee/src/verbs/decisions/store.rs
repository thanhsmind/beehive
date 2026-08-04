// the jsonl reader/writer and the decisions store lock
//
// Split out of the single 3.5k-line verbs/decisions.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{append_jsonl, ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, AcquireOnce};
use crate::verbs::reservations::{
    date_parse_val, finish, jget, js_date_parse, js_disp, js_disp_opt, js_is_ws, js_number_flag,
    js_numberify, js_quote, js_trim, keys_known, now_iso, parse_flags,
    pseudo_uuid_v4, truthy, v_is_str, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── fsutil.mjs readJsonl ──────────────────────────────────────────────────
// Node skipped an unparseable LINE silently and read the rest of the file.
//
// CUTOVER (2026-08-01). This port used to delegate such a line instead,
// because serde's JSON grammar and V8's are not provably identical (chiefly
// lone-surrogate escapes, which V8 accepts and serde refuses) and guessing
// "skip" could have dropped a record Node would have kept. There is no Node
// left to hand it to, and no way to decode those escapes here, so the line is
// CORRUPT: it is skipped, exactly as Node skipped its own unparseable lines,
// and — unlike Node — it says so on stderr, because a silently dropped
// decision record is precisely the failure fsutil's warn exists to prevent.
// The RESULT is unchanged: the same event list, one line poorer.

pub(crate) fn read_jsonl(file: &Path) -> Vec<Value> {
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&bytes);
    let mut events = Vec::new();
    // Node's readJsonl splits on /\r?\n/, so the line NUMBER the warning
    // reports is this 1-based index into that same split.
    for (idx, line) in text.split('\n').enumerate() {
        let trimmed = js_trim(line); // JS trim also strips \r and a BOM
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(trimmed) {
            // js_numberify only fails on a non-finite f64, which no parsed
            // JSON value can hold; the raw value is the honest fallback.
            Ok(v) => events.push(js_numberify(&v).unwrap_or(v)),
            Err(_) => crate::fsutil::warn_corrupt_jsonl_line(file, idx + 1),
        }
    }
    events
}

// ─── decisions.mjs writeJsonlAtomic / appendJsonlBatch ─────────────────────

pub(crate) fn to_base36(mut n: u64) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".to_string();
    }
    let mut out = Vec::new();
    while n > 0 {
        out.push(DIGITS[(n % 36) as usize]);
        n /= 36;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

pub(crate) static WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn unique_suffix() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut hasher = Sha256::new();
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(nanos.to_le_bytes());
    let digest = hasher.finalize();
    let rand: String = digest[..4].iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}",
        std::process::id(),
        to_base36(WRITE_COUNTER.fetch_add(1, Ordering::Relaxed)),
        rand
    )
}

/// provenance: decisions.mjs writeJsonlAtomic — temp-write+rename the WHOLE
/// jsonl body; on failure the tmp is removed best-effort and the original
/// error propagates (here: Err2::Ex, the delegate-shaped failure).
pub(crate) fn write_jsonl_atomic(file: &Path, events: &[Value]) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        ensure_dir(dir)?;
    }
    let body = events.iter().map(jsjson::stringify).collect::<Vec<_>>().join("\n");
    let content = if body.is_empty() { String::new() } else { format!("{body}\n") };
    let mut name = file.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.tmp", unique_suffix()));
    let tmp = file.with_file_name(name);
    let result = std::fs::write(&tmp, content).and_then(|()| std::fs::rename(&tmp, file));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// provenance: decisions.mjs appendJsonlBatch — every event in ONE append.
pub(crate) fn append_jsonl_batch(file: &Path, events: &[Value]) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        ensure_dir(dir)?;
    }
    let body = events.iter().map(jsjson::stringify).collect::<Vec<_>>().join("\n");
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(file)?;
    f.write_all(format!("{body}\n").as_bytes())
}

// ─── withDecisionsLockSync (decisions.mjs) ─────────────────────────────────

/// DecisionsLockBusyError's message, byte-identical (`??`-style unknowns).
pub(crate) fn decisions_lock_busy_message(holder: &Option<Value>) -> String {
    let who = match holder {
        Some(Value::Object(h)) => {
            let field = |k: &str| {
                h.get(k)
                    .filter(|v| !v.is_null())
                    .map(js_disp)
                    .unwrap_or_else(|| "unknown".to_string())
            };
            format!("pid={} session={} since {}", field("pid"), field("session"), field("ts"))
        }
        // typeof [] === 'object' too: every field reads unknown.
        Some(Value::Array(_)) => "pid=unknown session=unknown since unknown".to_string(),
        _ => "unknown holder".to_string(),
    };
    format!("decisions store lock \"{DECISIONS_LOCK_NAME}\" busy: held by {who}")
}

/// Bounded sync retry over acquire-once — mirrors withDecisionsLockSync
/// (initial attempt + up to `retries` sleeps/retries; each attempt writes
/// the same contention telemetry Node's acquireStoreLockOnceSync does).
pub(crate) fn acquire_decisions_lock(root: &Path, retries: u32) -> Result<lock::LockGuard, String> {
    let mut attempt = 0u32;
    loop {
        match lock::acquire_store_lock_once(root, DECISIONS_LOCK_NAME) {
            AcquireOnce::Acquired(guard) => return Ok(guard),
            AcquireOnce::Busy { holder } => {
                if attempt >= retries {
                    return Err(decisions_lock_busy_message(&holder));
                }
                std::thread::sleep(std::time::Duration::from_millis(
                    DECISIONS_LOCK_RETRY_DELAY_MS,
                ));
                attempt += 1;
            }
        }
    }
}
