// lock — the cross-process mutual-exclusion primitive for bee's store
// mutators.
//
// The on-disk lock format is a stable file-format contract: lock path
// sanitization (unsafe-char substitution + 8-hex sha256 of the ORIGINAL
// name), holder body shape ({pid, session, ts, token}), the stale-takeover
// rule (STALE_MS only for a provably-dead holder pid, HARD_STALE_MS absolute
// ceiling), the two-phase verified takeover (a post-rename identity check;
// a pre-rename re-verification), token-matched release, transient-FS retry
// on the Windows sharing-violation error class, and contention.jsonl
// telemetry.
//
// `with_store_lock` is the retrying entry point and `acquire_store_lock_once`
// the single-attempt one, same semantics each.

#![allow(dead_code)] // consumers arrive with the mutating-verb ports (R3)

use crate::jsjson;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const RETRY_DELAY_MS: u64 = 50;
pub const MAX_ATTEMPTS: u32 = 100; // ~5s worst-case wait before LOCK_BUSY
const STALE_MS: u128 = 30_000;
const HARD_STALE_MS: u128 = 3_600_000; // 1h pid-reuse/unknowable-liveness ceiling

const TRANSIENT_FS_RETRY_ATTEMPTS: u32 = 15;
const TRANSIENT_FS_RETRY_DELAY_MS: u64 = 20;

#[derive(Debug)]
pub struct LockBusy {
    pub lock_name: String,
    pub holder: Option<Value>,
}

impl LockBusy {
    /// LockBusyError's message, byte-identical.
    pub fn message(&self) -> String {
        let who = match &self.holder {
            Some(Value::Object(h)) => {
                let field = |k: &str| {
                    h.get(k)
                        .filter(|v| !v.is_null())
                        .map(jsjson::js_to_string)
                        .unwrap_or_else(|| "unknown".to_string())
                };
                format!("pid={} session={} since {}", field("pid"), field("session"), field("ts"))
            }
            _ => "unknown holder".to_string(),
        };
        format!("lock \"{}\" busy: held by {}", self.lock_name, who)
    }
}

pub fn locks_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("locks")
}

/// sanitizeLockName: Windows-invalid chars + control chars -> '_', with an
/// 8-hex sha256 of the ORIGINAL name appended so distinct logical names can
/// never collide after substitution. Must stay byte-identical to Node's.
fn sanitize_lock_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\x00'..='\x1f' => '_',
            other => other,
        })
        .collect();
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    format!("{sanitized}-{}", &hash[..8])
}

pub fn lock_file_path(root: &Path, name: &str) -> PathBuf {
    locks_dir(root).join(format!("{}.lock", sanitize_lock_name(name)))
}

fn read_holder(lock_path: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(lock_path).ok()?;
    serde_json::from_str(&text).ok()
}

fn env_session_id() -> Option<String> {
    for var in ["BEE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"] {
        if let Ok(v) = std::env::var(var) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn now_iso(t: SystemTime) -> String {
    chrono::DateTime::<chrono::Utc>::from(t)
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)
}

/// Fresh random-enough token: 8 bytes hex, derived by hashing pid + a
/// monotonic counter + clock nanos (no RNG dependency; uniqueness per
/// acquisition is what the takeover identity check needs).
fn fresh_token() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let mut hasher = Sha256::new();
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(COUNTER.fetch_add(1, Ordering::Relaxed).to_le_bytes());
    hasher.update(nanos.to_le_bytes());
    let digest = hasher.finalize();
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

// ── transient-FS retry (rel180-4's Windows sharing-violation class) ────────

#[cfg(windows)]
fn is_transient_fs_error(e: &std::io::Error) -> bool {
    // Node maps: EBUSY<-ERROR_SHARING_VIOLATION(32)/ERROR_LOCK_VIOLATION(33),
    // EPERM<-ERROR_ACCESS_DENIED(5), ENOTEMPTY<-ERROR_DIR_NOT_EMPTY(145),
    // EMFILE<-ERROR_TOO_MANY_OPEN_FILES(4).
    matches!(e.raw_os_error(), Some(4 | 5 | 32 | 33 | 145))
}

#[cfg(unix)]
fn is_transient_fs_error(e: &std::io::Error) -> bool {
    matches!(e.raw_os_error(), Some(code) if {
        code == libc::EBUSY || code == libc::EPERM || code == libc::ENOTEMPTY
            || code == libc::EMFILE || code == libc::ENFILE
    })
}

fn with_transient_fs_retry<T>(mut f: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    let mut attempt = 0;
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if !is_transient_fs_error(&e) || attempt >= TRANSIENT_FS_RETRY_ATTEMPTS {
                    return Err(e);
                }
                std::thread::sleep(Duration::from_millis(TRANSIENT_FS_RETRY_DELAY_MS));
            }
        }
    }
}

// ── pid liveness ───────────────────────────────────────────────────────────

/// isPidAlive: same decision table as process.kill(pid, 0) — missing/invalid
/// pid dead; exists (even if unsignalable) alive; unknowable alive (the
/// HARD_STALE_MS ceiling guards that case).
#[cfg(windows)]
pub fn is_pid_alive(pid: Option<f64>) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, GetLastError};
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    let Some(n) = valid_pid(pid) else { return false };
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, n) };
    if !handle.is_null() {
        unsafe { CloseHandle(handle) };
        return true;
    }
    // INVALID_PARAMETER is Windows' "no such process" (libuv maps it ESRCH);
    // ACCESS_DENIED and everything else read as alive.
    unsafe { GetLastError() != ERROR_INVALID_PARAMETER }
}

#[cfg(unix)]
pub fn is_pid_alive(pid: Option<f64>) -> bool {
    let Some(n) = valid_pid(pid) else { return false };
    let rc = unsafe { libc::kill(n as i32, 0) };
    if rc == 0 {
        return true;
    }
    std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

fn valid_pid(pid: Option<f64>) -> Option<u32> {
    let n = pid?;
    if n.fract() != 0.0 || n <= 0.0 || n > u32::MAX as f64 {
        return None;
    }
    Some(n as u32)
}

// ── acquisition primitives ─────────────────────────────────────────────────

fn try_acquire(lock_path: &Path, body: &Value) -> std::io::Result<bool> {
    let content = format!("{}\n", jsjson::stringify(body));
    let result = with_transient_fs_retry(|| {
        match std::fs::OpenOptions::new().write(true).create_new(true).open(lock_path) {
            Ok(mut f) => {
                use std::io::Write;
                f.write_all(content.as_bytes())
            }
            Err(e) => Err(e),
        }
    });
    match result {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(e),
    }
}

fn mtime_ms(path: &Path) -> Option<u128> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    modified.duration_since(UNIX_EPOCH).ok().map(|d| d.as_millis())
}

/// pid+token+ts triple match — proof the content is the SAME acquisition
/// instance (token is fresh-random per acquisition).
fn same_holder_identity(a: &Option<Value>, b: &Option<Value>) -> bool {
    match (a, b) {
        (Some(Value::Object(a)), Some(Value::Object(b))) => {
            ["pid", "token", "ts"].iter().all(|k| a.get(*k) == b.get(*k))
        }
        _ => false,
    }
}

fn judge_stale_takeover_eligibility(lock_path: &Path, now: u128) -> Option<Value> {
    let mtime = mtime_ms(lock_path)?; // vanished — normal retry
    let age = now.saturating_sub(mtime);
    if age <= STALE_MS {
        return None;
    }
    let holder_before = read_holder(lock_path);
    if age <= HARD_STALE_MS {
        let pid = holder_before
            .as_ref()
            .and_then(|h| h.get("pid"))
            .and_then(Value::as_f64);
        if is_pid_alive(pid) {
            return None; // live holder — legitimately long-running, never stolen
        }
    }
    Some(holder_before.map_or(Value::Null, |h| h))
}

fn rename_for_takeover(lock_path: &Path, now: u128) -> Option<(PathBuf, Option<Value>)> {
    let mut name = lock_path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".stale-{}-{}-{}", std::process::id(), now, &fresh_token()[..8]));
    let stale_path = lock_path.with_file_name(name);
    match with_transient_fs_retry(|| std::fs::rename(lock_path, &stale_path)) {
        Ok(()) => Some((stale_path.clone(), read_holder(&stale_path))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None, // racer consumed it — normal loss
        Err(_) => None, // unexpected rename failure reads as a lost race, not a crash
    }
}

fn settle_takeover(
    lock_path: &Path,
    stale_path: &Path,
    holder_before: &Option<Value>,
    holder_after: &Option<Value>,
) -> bool {
    if same_holder_identity(holder_after, holder_before) {
        let _ = with_transient_fs_retry(|| match std::fs::remove_file(stale_path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        });
        return true;
    }
    // Displaced a DIFFERENT, fresher lock — restore it unless a third racer
    // has since legitimately re-occupied lock_path (never clobber that).
    if std::fs::metadata(lock_path).is_err()
        && with_transient_fs_retry(|| std::fs::rename(stale_path, lock_path)).is_ok()
    {
        return false;
    }
    let _ = with_transient_fs_retry(|| match std::fs::remove_file(stale_path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    });
    false
}

fn try_stale_takeover(lock_path: &Path, now: u128) -> bool {
    let Some(holder_before_v) = judge_stale_takeover_eligibility(lock_path, now) else {
        return false;
    };
    let holder_before = if holder_before_v.is_null() { None } else { Some(holder_before_v) };
    // rel190-2 pre-rename re-verification: never touch content that changed
    // since it was judged stale.
    if !same_holder_identity(&read_holder(lock_path), &holder_before) {
        return false;
    }
    match rename_for_takeover(lock_path, now) {
        Some((stale_path, holder_after)) => {
            settle_takeover(lock_path, &stale_path, &holder_before, &holder_after)
        }
        None => false,
    }
}

// ── telemetry ──────────────────────────────────────────────────────────────

fn append_contention_telemetry(root: &Path, record: &Value) {
    let logs_dir = root.join(".bee").join("logs");
    if std::fs::create_dir_all(&logs_dir).is_err() {
        return; // fail-open: telemetry never changes an acquisition's outcome
    }
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs_dir.join("contention.jsonl"))
    {
        let _ = f.write_all(format!("{}\n", jsjson::stringify(record)).as_bytes());
    }
}

fn contention_record(
    name: &str,
    session: &Option<String>,
    wait_start_ms: u128,
    holder_session: Option<Value>,
    result: &str,
) -> Value {
    json!({
        "ts": now_iso(SystemTime::now()),
        "lock_name": name,
        "lock_wait_ms": (now_ms().saturating_sub(wait_start_ms)) as u64,
        "holder_session": holder_session.unwrap_or(Value::Null),
        "caller_session": session.as_deref().map_or(Value::Null, |s| json!(s)),
        "workflow_id": null,
        "workspace_id": null,
        "resource": null,
        "result": result,
    })
}

fn holder_session_of(v: &Option<Value>) -> Option<Value> {
    match v {
        Some(Value::Object(h)) => h.get("session").filter(|s| !s.is_null()).cloned(),
        _ => None,
    }
}

// ── public entry points ────────────────────────────────────────────────────

pub struct LockGuard {
    lock_path: PathBuf,
    token: String,
    released: bool,
}

impl LockGuard {
    /// Idempotent, token-matched release — never unlinks a lock some other
    /// holder has since taken over.
    pub fn release(&mut self) {
        if self.released {
            return;
        }
        self.released = true;
        if let Some(Value::Object(h)) = read_holder(&self.lock_path) {
            let pid_matches = h.get("pid").and_then(Value::as_u64) == Some(std::process::id() as u64);
            let token_matches = h.get("token").and_then(Value::as_str) == Some(self.token.as_str());
            if pid_matches && token_matches {
                let _ = with_transient_fs_retry(|| match std::fs::remove_file(&self.lock_path) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    other => other,
                });
            }
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        self.release();
    }
}

pub enum AcquireOnce {
    Acquired(LockGuard),
    Busy { holder: Option<Value> },
}

/// withStoreLock's acquisition loop, returning a guard instead of taking a
/// closure (Rust callers use RAII; the closure shape adds nothing here).
pub fn acquire_store_lock(root: &Path, name: &str, max_attempts: u32) -> Result<LockGuard, LockBusy> {
    let _ = std::fs::create_dir_all(locks_dir(root));
    let lock_path = lock_file_path(root, name);
    let token = fresh_token();
    let session = env_session_id();
    let wait_start = now_ms();
    let mut contended_holder_session: Option<Value> = None;

    for attempt in 0..max_attempts {
        let now = SystemTime::now();
        let body = json!({
            "pid": std::process::id(),
            "session": session.as_deref().map_or(Value::Null, |s| json!(s)),
            "ts": now_iso(now),
            "token": token,
        });
        let acquired = try_acquire(&lock_path, &body).unwrap_or(false);
        if acquired {
            append_contention_telemetry(
                root,
                &contention_record(name, &session, wait_start, contended_holder_session.take(), "acquired"),
            );
            return Ok(LockGuard { lock_path, token, released: false });
        }
        if let Some(s) = holder_session_of(&read_holder(&lock_path)) {
            contended_holder_session = Some(s);
        }
        // Staleness re-verified at THIS retry, on real mtime — never cached.
        if try_stale_takeover(&lock_path, now_ms()) {
            let body = json!({
                "pid": std::process::id(),
                "session": session.as_deref().map_or(Value::Null, |s| json!(s)),
                "ts": now_iso(SystemTime::now()),
                "token": token,
            });
            if try_acquire(&lock_path, &body).unwrap_or(false) {
                append_contention_telemetry(
                    root,
                    &contention_record(name, &session, wait_start, contended_holder_session.take(), "acquired"),
                );
                return Ok(LockGuard { lock_path, token, released: false });
            }
        }
        if attempt + 1 < max_attempts {
            std::thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
        }
    }

    let holder = read_holder(&lock_path);
    append_contention_telemetry(
        root,
        &contention_record(
            name,
            &session,
            wait_start,
            holder_session_of(&holder).or(contended_holder_session),
            "busy",
        ),
    );
    Err(LockBusy { lock_name: name.to_string(), holder })
}

/// acquireStoreLockOnceSync: exactly one attempt (plus one follow-up when a
/// stale takeover was won) — hooks and sync funnels never WAIT on the lock.
pub fn acquire_store_lock_once(root: &Path, name: &str) -> AcquireOnce {
    match acquire_store_lock(root, name, 1) {
        Ok(guard) => AcquireOnce::Acquired(guard),
        Err(busy) => AcquireOnce::Busy { holder: busy.holder },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_name_sanitization_matches_node() {
        // Node: sanitize('cells:a') -> 'cells_a-' + sha256('cells:a')[..8]
        let sanitized = sanitize_lock_name("cells:a");
        assert!(sanitized.starts_with("cells_a-"));
        assert_eq!(sanitized.len(), "cells_a-".len() + 8);
        // Distinct originals that sanitize alike stay distinct via the hash.
        assert_ne!(sanitize_lock_name("cells:a"), sanitize_lock_name("cells/a"));
    }

    #[test]
    fn acquire_release_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let mut guard = match acquire_store_lock(tmp.path(), "test", 1) {
            Ok(g) => g,
            Err(b) => panic!("unexpected busy: {}", b.message()),
        };
        let lock_path = lock_file_path(tmp.path(), "test");
        assert!(lock_path.exists());
        // Holder body is Node-shaped.
        let holder = read_holder(&lock_path).unwrap();
        assert_eq!(holder["pid"], json!(std::process::id()));
        assert!(holder["token"].is_string());
        guard.release();
        assert!(!lock_path.exists());
        guard.release(); // idempotent
    }

    #[test]
    fn contention_reports_busy_with_holder() {
        let tmp = tempfile::tempdir().unwrap();
        let _guard = acquire_store_lock(tmp.path(), "test", 1).ok().unwrap();
        match acquire_store_lock(tmp.path(), "test", 1) {
            Ok(_) => panic!("second acquire must be busy"),
            Err(busy) => {
                assert!(busy.message().contains("lock \"test\" busy"));
                assert!(busy.holder.is_some());
            }
        }
        // Telemetry recorded both outcomes.
        let log = std::fs::read_to_string(tmp.path().join(".bee/logs/contention.jsonl")).unwrap();
        assert!(log.contains("\"result\":\"acquired\""));
        assert!(log.contains("\"result\":\"busy\""));
    }

    #[test]
    fn release_never_unlinks_a_taken_over_lock() {
        let tmp = tempfile::tempdir().unwrap();
        let mut guard = acquire_store_lock(tmp.path(), "test", 1).ok().unwrap();
        let lock_path = lock_file_path(tmp.path(), "test");
        // Simulate another holder having replaced the lock.
        std::fs::write(&lock_path, "{\"pid\":1,\"token\":\"other\",\"ts\":\"t\"}\n").unwrap();
        guard.release();
        assert!(lock_path.exists(), "foreign lock must survive our release");
    }

    #[test]
    fn stale_dead_pid_lock_is_taken_over() {
        let tmp = tempfile::tempdir().unwrap();
        let lock_path = lock_file_path(tmp.path(), "test");
        std::fs::create_dir_all(lock_path.parent().unwrap()).unwrap();
        // A dead-pid holder, aged past STALE_MS via mtime rewind.
        std::fs::write(&lock_path, "{\"pid\":999999999,\"session\":null,\"ts\":\"old\",\"token\":\"tok\"}\n").unwrap();
        let old = filetime::FileTime::from_unix_time(
            (now_ms() / 1000) as i64 - 120, // 2 minutes ago > STALE_MS
            0,
        );
        filetime::set_file_mtime(&lock_path, old).unwrap();
        let guard = acquire_store_lock(tmp.path(), "test", 2);
        assert!(guard.is_ok(), "stale dead-pid lock must be taken over");
    }

    #[test]
    fn fresh_live_lock_is_never_stolen() {
        let tmp = tempfile::tempdir().unwrap();
        let lock_path = lock_file_path(tmp.path(), "test");
        std::fs::create_dir_all(lock_path.parent().unwrap()).unwrap();
        // OUR OWN pid (provably alive), aged past STALE_MS but below the
        // hard ceiling — must never be stolen.
        std::fs::write(
            &lock_path,
            format!("{{\"pid\":{},\"session\":null,\"ts\":\"old\",\"token\":\"tok\"}}\n", std::process::id()),
        )
        .unwrap();
        let old = filetime::FileTime::from_unix_time((now_ms() / 1000) as i64 - 120, 0);
        filetime::set_file_mtime(&lock_path, old).unwrap();
        match acquire_store_lock(tmp.path(), "test", 1) {
            Ok(_) => panic!("live holder below the hard ceiling must never be stolen"),
            Err(busy) => assert!(busy.holder.is_some()),
        }
    }

    #[test]
    fn pid_liveness_table() {
        assert!(is_pid_alive(Some(std::process::id() as f64)));
        assert!(!is_pid_alive(None));
        assert!(!is_pid_alive(Some(-1.0)));
        assert!(!is_pid_alive(Some(1.5)));
        assert!(!is_pid_alive(Some(999_999_999.0)), "unlikely-to-exist pid should read dead");
    }
}
