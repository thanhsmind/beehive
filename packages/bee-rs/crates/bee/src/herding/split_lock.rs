// A cross-PROCESS advisory lock that serializes the pane split done by
// concurrent `bee herding run` invocations.
//
// Every spawn is its own OS process, so an in-process Mutex cannot help: the
// mutual exclusion has to live on the filesystem. This module is modelled on
// the store lock in hooks/prompt_context.rs (create_new + AlreadyExists,
// two-tier stale takeover, identity-checked release) but is deliberately
// self-contained — prompt_context.rs keeps its items private and is not
// touched by this module.
//
// Shape:
//   lock file  <main_root>/.bee/locks/herding-pane-split.lock
//   holder     {"pid":…, "ts":"…", "token":"…", "job_id":"…"}
//   acquire()  retries every 50ms until the wait budget is spent; Ok(None) on
//              budget exhaustion (the caller fails OPEN — a busy split lock
//              must never turn into a failed spawn), Err only on a real
//              filesystem failure.
//   SplitLock  releases in Drop, and removes the file only when the on-disk
//              holder still carries this acquisition's own pid + token.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Basename of the single pane-split lock. One lock for the whole main root:
/// panes are split into one shared terminal multiplexer session.
const LOCK_BASENAME: &str = "herding-pane-split.lock";

/// Poll interval while waiting for a busy lock.
const RETRY_INTERVAL: Duration = Duration::from_millis(50);

/// Soft stale window (lock.mjs STALE_MS): past this age the holder is a
/// takeover CANDIDATE, but a live recorded pid still keeps it.
const STALE_MS: f64 = 30_000.0;

/// Hard stale ceiling (lock.mjs HARD_STALE_MS): past this age the holder is
/// taken over even when its recorded pid is still alive — a pid that old has
/// almost certainly been recycled.
const HARD_STALE_MS: f64 = 3_600_000.0;

/// A held pane-split lock. Dropping it releases the lock.
pub(crate) struct SplitLock {
    path: PathBuf,
    pid: u32,
    token: String,
    released: bool,
}

impl SplitLock {
    /// Path of the lock file this guard holds. Read only by this module's
    /// own tests — `run.rs`, the production caller, holds the guard and
    /// never asks it where it lives — so this one accessor keeps the
    /// dead-code opt-out the module-level one used to cover.
    #[allow(dead_code)]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Remove the lock file, but ONLY when the on-disk holder is still this
    /// acquisition (same pid AND same token). A stale-takeover racer may have
    /// replaced the file already; deleting its lock would break the exclusion
    /// this module exists to provide.
    fn release(&mut self) {
        if self.released {
            return;
        }
        self.released = true;
        let Some(holder) = read_holder(&self.path) else { return };
        let pid_ok = holder.get("pid").and_then(Value::as_u64) == Some(self.pid as u64);
        let token_ok = holder.get("token").and_then(Value::as_str) == Some(self.token.as_str());
        if pid_ok && token_ok {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

impl Drop for SplitLock {
    fn drop(&mut self) {
        self.release();
    }
}

/// Acquire the pane-split lock under `main_root`, waiting up to `wait`.
///
/// Returns `Ok(Some(guard))` when the lock is held, `Ok(None)` when the wait
/// budget ran out with the lock still busy (the caller fails open and splits
/// anyway), and `Err` only when the filesystem itself failed — a missing
/// parent that cannot be created, or an unwritable locks directory.
pub(crate) fn acquire(
    main_root: &Path,
    job_id: &str,
    wait: Duration,
) -> Result<Option<SplitLock>, String> {
    let locks_dir = main_root.join(".bee").join("locks");
    std::fs::create_dir_all(&locks_dir)
        .map_err(|e| format!("herding split lock: cannot create {}: {e}", locks_dir.display()))?;
    let lock_path = locks_dir.join(LOCK_BASENAME);

    let pid = std::process::id();
    let deadline = std::time::Instant::now() + wait;
    loop {
        let token = random_token(8);
        let now = now_ms();
        let body = holder_body(pid, now, &token, job_id);
        if try_acquire(&lock_path, &body)? {
            return Ok(Some(SplitLock { path: lock_path, pid, token, released: false }));
        }
        // Busy. A stale holder (dead pid past the soft window, or ANY holder
        // past the hard ceiling) is taken over, then re-attempted at once.
        if try_stale_takeover(&lock_path, now) {
            let body = holder_body(pid, now_ms(), &token, job_id);
            if try_acquire(&lock_path, &body)? {
                return Ok(Some(SplitLock { path: lock_path, pid, token, released: false }));
            }
        }
        if std::time::Instant::now() >= deadline {
            return Ok(None);
        }
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        std::thread::sleep(RETRY_INTERVAL.min(left).max(Duration::from_millis(1)));
    }
}

fn holder_body(pid: u32, now_ms: i64, token: &str, job_id: &str) -> Value {
    let mut m = Map::new();
    m.insert("pid".into(), Value::Number(serde_json::Number::from(pid)));
    m.insert("ts".into(), Value::Number(serde_json::Number::from(now_ms)));
    m.insert("token".into(), Value::String(token.to_string()));
    m.insert("job_id".into(), Value::String(job_id.to_string()));
    Value::Object(m)
}

/// The one atomic step: `create_new` succeeds for exactly one racer, and every
/// other racer gets `AlreadyExists`. Any other error is a real fs failure.
fn try_acquire(lock_path: &Path, body: &Value) -> Result<bool, String> {
    let content = format!("{body}\n");
    match std::fs::OpenOptions::new().write(true).create_new(true).open(lock_path) {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(content.as_bytes())
                .map_err(|e| format!("herding split lock: write failed: {e}"))?;
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(format!("herding split lock: {}: {e}", lock_path.display())),
    }
}

fn read_holder(lock_path: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(lock_path).ok()?;
    serde_json::from_str::<Value>(&text).ok()
}

/// Two holder files are the same acquisition when pid, token and ts all match.
fn same_holder_identity(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) if a.is_object() && b.is_object() => {
            a.get("pid") == b.get("pid")
                && a.get("token") == b.get("token")
                && a.get("ts") == b.get("ts")
        }
        _ => false,
    }
}

/// The two-tier stale rule, plus the rename-verify-settle dance that makes the
/// takeover itself safe against concurrent takeovers.
///
/// Tier 1 (soft, `STALE_MS`): a holder younger than this is never touched.
/// Between the soft window and the hard ceiling, a holder whose recorded pid
/// is still alive keeps the lock.
/// Tier 2 (hard, `HARD_STALE_MS`): past this age the holder loses the lock
/// whatever its pid says.
fn try_stale_takeover(lock_path: &Path, now_ms: i64) -> bool {
    let Ok(meta) = std::fs::metadata(lock_path) else { return false };
    let mtime_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as f64)
        .unwrap_or(f64::MAX);
    let age = now_ms as f64 - mtime_ms;
    if age <= STALE_MS {
        return false;
    }
    let holder_before = read_holder(lock_path);
    if age <= HARD_STALE_MS {
        let pid = holder_before
            .as_ref()
            .filter(|h| h.is_object())
            .and_then(|h| h.get("pid"))
            .cloned()
            .unwrap_or(Value::Null);
        if is_pid_alive(&pid) {
            return false;
        }
    }
    // Claim the takeover by renaming the file aside. Rename is atomic, so only
    // one racer can win it; every loser sees ENOENT and simply retries.
    if !same_holder_identity(read_holder(lock_path).as_ref(), holder_before.as_ref()) {
        return false;
    }
    let stale_path = PathBuf::from(format!(
        "{}.stale-{}-{}-{}",
        lock_path.display(),
        std::process::id(),
        now_ms,
        random_token(4),
    ));
    if std::fs::rename(lock_path, &stale_path).is_err() {
        return false;
    }
    let holder_after = read_holder(&stale_path);
    if same_holder_identity(holder_after.as_ref(), holder_before.as_ref()) {
        let _ = std::fs::remove_file(&stale_path);
        return true;
    }
    // The file changed under us: put it back when nothing else took the slot.
    if std::fs::metadata(lock_path).is_err() && std::fs::rename(&stale_path, lock_path).is_ok() {
        return false;
    }
    let _ = std::fs::remove_file(&stale_path);
    false
}

/// Null-signal pid probe. A non-integer, zero or negative pid is never alive.
fn is_pid_alive(pid: &Value) -> bool {
    let n = match pid {
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => s.trim().parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    };
    if !n.is_finite() || n.fract() != 0.0 || n <= 0.0 {
        return false;
    }
    pid_alive_os(n as u32)
}

#[cfg(windows)]
fn pid_alive_os(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ACCESS_DENIED};
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !handle.is_null() {
            CloseHandle(handle);
            true
        } else {
            // ACCESS_DENIED means the process exists but is not ours to probe.
            GetLastError() == ERROR_ACCESS_DENIED
        }
    }
}

#[cfg(unix)]
fn pid_alive_os(pid: u32) -> bool {
    let ok = unsafe { libc::kill(pid as i32, 0) } == 0;
    // EPERM means alive-but-not-ours; only ESRCH proves the pid is gone.
    ok || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// No rand crate in-tree: hash pid + a process-wide counter + wall clock +
/// the monotonic clock. Unique per acquisition, in-process and across processes.
fn random_token(bytes: usize) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seed = format!(
        "{}-{}-{}-{:?}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        nanos,
        std::time::Instant::now(),
    );
    let mut h = Sha256::new();
    h.update(seed.as_bytes());
    let hex: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    hex[..bytes * 2].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// A private temp root per test, with no dependency beyond std.
    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "bee-herding-split-lock-{}-{}-{}",
            tag,
            std::process::id(),
            random_token(6),
        ));
        std::fs::create_dir_all(&dir).expect("temp root");
        dir
    }

    fn lock_path_of(root: &Path) -> PathBuf {
        root.join(".bee").join("locks").join(LOCK_BASENAME)
    }

    #[test]
    fn second_acquire_is_none_while_a_live_holder_holds() {
        let root = temp_root("busy");
        let held = acquire(&root, "job-a", Duration::from_millis(50))
            .expect("first acquire")
            .expect("first acquire holds");
        assert!(held.path().exists(), "lock file must exist while held");

        let start = std::time::Instant::now();
        let second = acquire(&root, "job-b", Duration::from_millis(200)).expect("second acquire ok");
        assert!(second.is_none(), "a live holder must block the second acquire");
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "the second acquire must give up inside its budget, took {:?}",
            start.elapsed()
        );
        drop(held);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn lock_is_reacquirable_after_the_guard_drops() {
        let root = temp_root("reacquire");
        let path = lock_path_of(&root);
        {
            let g = acquire(&root, "job-1", Duration::from_millis(50))
                .expect("acquire")
                .expect("holds");
            assert_eq!(g.path(), path.as_path());
        }
        assert!(!path.exists(), "Drop must remove the lock file");

        let again = acquire(&root, "job-2", Duration::from_millis(50))
            .expect("re-acquire")
            .expect("re-acquire holds");
        let holder = read_holder(&path).expect("holder json");
        assert_eq!(holder.get("job_id").and_then(Value::as_str), Some("job-2"));
        drop(again);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_dead_pid_with_an_old_mtime_is_taken_over() {
        let root = temp_root("stale");
        let path = lock_path_of(&root);
        std::fs::create_dir_all(path.parent().unwrap()).expect("locks dir");
        // pid 0 is never a live process, so is_pid_alive says dead.
        let stale = holder_body(0, 0, "deadbeefdeadbeef", "job-dead");
        std::fs::write(&path, format!("{stale}\n")).expect("write stale holder");
        // Back-date the mtime past the soft window but well inside the hard
        // ceiling, so the takeover has to come from the dead-pid rule.
        let old = std::time::SystemTime::now() - Duration::from_secs(60);
        set_mtime(&path, old);

        let taken = acquire(&root, "job-live", Duration::from_millis(200))
            .expect("acquire over stale holder")
            .expect("stale holder must be taken over");
        let holder = read_holder(&path).expect("holder json");
        assert_eq!(holder.get("job_id").and_then(Value::as_str), Some("job-live"));
        assert_eq!(
            holder.get("pid").and_then(Value::as_u64),
            Some(std::process::id() as u64)
        );
        drop(taken);
        assert!(!path.exists(), "the taken-over lock releases normally");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn release_leaves_a_foreign_holder_alone() {
        let root = temp_root("foreign");
        let path = lock_path_of(&root);
        let guard = acquire(&root, "job-mine", Duration::from_millis(50))
            .expect("acquire")
            .expect("holds");
        // Simulate a takeover racer replacing the holder under us.
        let foreign = holder_body(std::process::id(), now_ms(), "0000000000000000", "job-other");
        std::fs::write(&path, format!("{foreign}\n")).expect("overwrite holder");
        drop(guard);
        assert!(path.exists(), "release must not delete another acquisition's lock");
        let holder = read_holder(&path).expect("holder json");
        assert_eq!(holder.get("job_id").and_then(Value::as_str), Some("job-other"));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The real thing: N threads contend over the actual file. Each holder
    /// increments a shared "inside" counter, checks it never exceeded 1, then
    /// decrements and releases. A broken lock trips the concurrency assert.
    #[test]
    fn concurrent_acquires_are_mutually_exclusive() {
        let root = Arc::new(temp_root("contend"));
        let inside = Arc::new(AtomicUsize::new(0));
        let max_seen = Arc::new(AtomicUsize::new(0));
        let acquired = Arc::new(AtomicUsize::new(0));
        let threads = 8usize;
        let rounds = 6usize;

        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let root = Arc::clone(&root);
                let inside = Arc::clone(&inside);
                let max_seen = Arc::clone(&max_seen);
                let acquired = Arc::clone(&acquired);
                std::thread::spawn(move || {
                    for r in 0..rounds {
                        let job = format!("job-{t}-{r}");
                        let guard = acquire(&root, &job, Duration::from_secs(20))
                            .expect("acquire must not error");
                        let Some(guard) = guard else { continue };
                        let now = inside.fetch_add(1, Ordering::SeqCst) + 1;
                        max_seen.fetch_max(now, Ordering::SeqCst);
                        // Hold the lock long enough for a broken lock to overlap.
                        std::thread::sleep(Duration::from_millis(2));
                        assert_eq!(
                            inside.load(Ordering::SeqCst),
                            1,
                            "two holders inside the critical section at once"
                        );
                        inside.fetch_sub(1, Ordering::SeqCst);
                        acquired.fetch_add(1, Ordering::SeqCst);
                        drop(guard);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("worker thread must not panic");
        }
        assert_eq!(
            max_seen.load(Ordering::SeqCst),
            1,
            "the critical section was entered concurrently"
        );
        assert!(
            acquired.load(Ordering::SeqCst) >= threads,
            "every thread should have won the lock at least once, got {}",
            acquired.load(Ordering::SeqCst)
        );
        assert!(!lock_path_of(&root).exists(), "no lock file may be left behind");
        let _ = std::fs::remove_dir_all(root.as_path());
    }

    #[cfg(unix)]
    fn set_mtime(path: &Path, when: std::time::SystemTime) {
        let secs = when
            .duration_since(std::time::UNIX_EPOCH)
            .expect("epoch")
            .as_secs() as i64;
        let tv = libc::timeval { tv_sec: secs as libc::time_t, tv_usec: 0 };
        let times = [tv, tv];
        let c = std::ffi::CString::new(path.as_os_str().to_str().expect("utf8 path")).expect("cstr");
        let rc = unsafe { libc::utimes(c.as_ptr(), times.as_ptr()) };
        assert_eq!(rc, 0, "utimes must succeed on {}", path.display());
    }

    #[cfg(windows)]
    fn set_mtime(path: &Path, when: std::time::SystemTime) {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect("open for set_mtime");
        f.set_modified(when).expect("set_modified");
    }
}
