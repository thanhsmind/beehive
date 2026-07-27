//! lock — the D9 cross-process store-lock protocol ported from
//! `.bee/bin/lib/lock.mjs` (rust-port-3, CONTEXT.md D3/D9).
//!
//! `.bee/bin/lib/lock.mjs` is FROZEN for the duration of the rust-port
//! feature (D1) — this module reproduces its protocol semantics exactly so
//! mjs and Rust processes can hold/contend the SAME lock files safely while
//! the two runtimes interleave on one store (D3). The named contract (D9):
//!
//! - lock path scheme: `.bee/locks/<sanitized-name>-<sha256-first-8-hex>.lock`
//!   (sanitization replaces Windows-invalid chars + control chars with `_`;
//!   the hash of the ORIGINAL name keeps distinct logical names distinct);
//! - lock body `{pid, session, ts, token}` written O_EXCL (`create_new`),
//!   one compact JSON line + `\n`;
//! - staleness windows [`STALE_MS`] 30 s / [`HARD_STALE_MS`] 1 h with a
//!   pid-liveness probe: a merely-old holder is a takeover CANDIDATE only —
//!   the steal requires the recorded pid provably dead (permission-denied
//!   counts as alive) below the ceiling, or the absolute 1 h ceiling passed
//!   regardless of the probe (pid-reuse guard of last resort);
//! - rename-based takeover verified by pid+token+ts identity BEFORE and
//!   AFTER the rename (never an unconditional unlink — see
//!   `settle_takeover`);
//! - transient-FS retry policy (15 × 20 ms on EBUSY/EPERM/ENOTEMPTY/EMFILE/
//!   ENFILE), the same deliberate duplicate lock.mjs itself carries;
//! - hooks-never-wait: [`LockOptions::try_once`] (`max_attempts: 1`) — a
//!   lifecycle checkpoint tries once and skips, never sleeps on the lock;
//! - `.bee/logs/contention.jsonl` fail-open telemetry on every acquire
//!   outcome, lock-free and swallowing every write failure.
//!
//! Conformance is proven cross-runtime by
//! `crates/bee-core/tests/lock_interop.rs`, which drives the REAL lock.mjs
//! in node children against this module on shared per-test temp roots —
//! never by this file's author's reading of lock.mjs alone.
//!
//! Two deliberate representation notes (semantics preserved, bytes not):
//! - serde_json serializes object keys sorted (mjs writes insertion order).
//!   Lock bodies and telemetry lines are always parsed, never byte-compared,
//!   and `.bee/logs/**` sits in the parity differ's whole-path exclusion set
//!   (rust-port validation decision W7), so key order is not part of the
//!   contract.
//! - mjs's `withStoreLock` `finally` release can propagate an unlink error
//!   out of the critical section; Rust's panic-path release runs in a
//!   [`Drop`] impl and must swallow errors there. The NORMAL return path
//!   releases explicitly and propagates errors, matching mjs.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::fsutil::ensure_dir;

/// Inter-attempt sleep for the waiting path (`with_store_lock`).
pub const RETRY_DELAY_MS: u64 = 50;
/// ~5 s worst-case wait before a typed LOCK_BUSY refusal.
pub const MAX_ATTEMPTS: u32 = 100;
/// Crashed-holder window: only a candidate once BOTH stale-aged AND pid-dead.
pub const STALE_MS: i64 = 30_000;
/// Absolute ceiling: past this age, takeover proceeds regardless of the pid
/// probe result — a pid-reuse/unknowable-liveness guard of last resort.
pub const HARD_STALE_MS: i64 = 3_600_000;

const TRANSIENT_FS_RETRY_ATTEMPTS: u32 = 15;
const TRANSIENT_FS_RETRY_DELAY_MS: u64 = 20;

/// `.bee/locks/` under `root` — same layout as lock.mjs's `locksDir(root)`.
pub fn locks_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("locks")
}

/// Maps a logical lock name (e.g. `cells:some-id`) to a filesystem-safe
/// basename, byte-identical to lock.mjs's `sanitizeLockName`: every
/// Windows-invalid char (`< > : " / \ | ? *`) and every control char
/// (U+0000–U+001F) becomes `_`, and the first 8 hex chars of sha256 over the
/// ORIGINAL name are appended so two DISTINCT logical names can never
/// collide after sanitization (`cells:a` vs `cells_a`). Pure function: both
/// runtimes MUST derive the identical path or they would silently stop
/// contending on the same lock at all — proven against the real
/// `lockFilePath` in `tests/lock_interop.rs`.
fn sanitize_lock_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if (c as u32) <= 0x1f => '_',
            c => c,
        })
        .collect();
    let hash = Sha256::digest(name.as_bytes());
    let hex: String = hash.iter().take(4).map(|b| format!("{b:02x}")).collect();
    format!("{sanitized}-{hex}")
}

/// Full lock file path for a logical name — `locksDir(root)/<sanitized>.lock`.
pub fn lock_file_path(root: &Path, name: &str) -> PathBuf {
    locks_dir(root).join(format!("{}.lock", sanitize_lock_name(name)))
}

// ---------------------------------------------------------------------------
// small JS-semantics helpers (holder bodies are arbitrary JSON on disk; the
// mjs source compares them with JS truthiness and `===` — mirrored here so a
// junk body behaves identically under both runtimes)
// ---------------------------------------------------------------------------

fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// JS `===` over two optional JSON fields (`None` = JS `undefined`;
/// `undefined === undefined` is true). Numbers compare numerically
/// (`1 === 1.0`), everything else by exact value+type.
fn js_strict_eq(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(Value::Number(x)), Some(Value::Number(y))) => {
            match (x.as_f64(), y.as_f64()) {
                (Some(fx), Some(fy)) => fx == fy,
                _ => x == y,
            }
        }
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// Compares two holder snapshots by pid+token+ts — a fresh random token per
/// acquisition means an exact match on all three is proof the content is the
/// SAME acquisition instance, never merely "looks similar". Faithful to the
/// mjs source's `Boolean(a && b && ...)`: both snapshots must exist AND be
/// JS-truthy (so a corrupt/unreadable holder — `None` here, `null` there —
/// never matches anything, which is exactly why a corrupt stale lock is
/// never taken over: `performTakeoverClaim` re-verifies against a `null`
/// snapshot and always declines).
fn same_holder_identity(a: Option<&Value>, b: Option<&Value>) -> bool {
    let (a, b) = match (a, b) {
        (Some(a), Some(b)) if js_truthy(a) && js_truthy(b) => (a, b),
        _ => return false,
    };
    js_strict_eq(a.get("pid"), b.get("pid"))
        && js_strict_eq(a.get("token"), b.get("token"))
        && js_strict_eq(a.get("ts"), b.get("ts"))
}

// ---------------------------------------------------------------------------
// time — `new Date(ms).toISOString()` equivalent (no chrono dep: bee-core
// stays lean for the D5 cold-start budget; algorithm is Howard Hinnant's
// civil-from-days, unit-proven against known node outputs)
// ---------------------------------------------------------------------------

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (y + i64::from(m <= 2), m, d)
}

/// ISO-8601 UTC with milliseconds — the exact shape node's
/// `Date.prototype.toISOString()` writes into lock bodies and telemetry
/// (`2026-07-26T05:04:15.015Z`). Public so conformance tests can backdate a
/// lock body's `ts` with the same formatting the protocol itself uses.
pub fn iso8601_millis(ms: i64) -> String {
    let days = ms.div_euclid(86_400_000);
    let msod = ms.rem_euclid(86_400_000);
    let (y, mo, d) = civil_from_days(days);
    let h = msod / 3_600_000;
    let mi = (msod / 60_000) % 60;
    let s = (msod / 1000) % 60;
    let mil = msod % 1000;
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{mil:03}Z")
}

// ---------------------------------------------------------------------------
// session label + token
// ---------------------------------------------------------------------------

/// Mirrors lock.mjs's `envSessionId` (hardening-4a): `BEE_SESSION_ID` wins
/// over the legacy `CLAUDE_CODE_SESSION_ID`; trimmed, non-empty, else None.
/// A deliberate small duplicate of claims.mjs's canonical chain, used only
/// as the lock-holder LABEL — never to authorize anything.
fn env_session_id() -> Option<String> {
    for key in ["BEE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"] {
        if let Ok(v) = std::env::var(key) {
            let t = crate::fsutil::js_trim(&v);
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

fn random_hex(n_bytes: usize) -> io::Result<String> {
    let mut buf = vec![0u8; n_bytes];
    getrandom::fill(&mut buf).map_err(|e| io::Error::other(format!("entropy source failed: {e}")))?;
    Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
}

// ---------------------------------------------------------------------------
// transient-FS retry (rel180-4) — same shape, same budget as lock.mjs's own
// deliberate duplicate of claims.mjs's withTransientFsRetry
// ---------------------------------------------------------------------------

#[cfg(unix)]
fn is_transient_fs_error(err: &io::Error) -> bool {
    matches!(
        err.raw_os_error(),
        Some(code) if code == libc::EBUSY
            || code == libc::EPERM
            || code == libc::ENOTEMPTY
            || code == libc::EMFILE
            || code == libc::ENFILE
    )
}

#[cfg(windows)]
fn is_transient_fs_error(err: &io::Error) -> bool {
    // The Windows raw error codes libuv maps to node's EBUSY / EPERM /
    // ENOTEMPTY / EMFILE set: ERROR_ACCESS_DENIED, ERROR_SHARING_VIOLATION,
    // ERROR_LOCK_VIOLATION, ERROR_DIR_NOT_EMPTY, ERROR_TOO_MANY_OPEN_FILES.
    matches!(err.raw_os_error(), Some(5) | Some(32) | Some(33) | Some(145) | Some(4))
}

#[cfg(not(any(unix, windows)))]
fn is_transient_fs_error(_err: &io::Error) -> bool {
    false
}

fn with_transient_fs_retry<T>(mut f: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    let mut attempt = 0;
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(err) => {
                attempt += 1;
                if !is_transient_fs_error(&err) || attempt >= TRANSIENT_FS_RETRY_ATTEMPTS {
                    return Err(err);
                }
                std::thread::sleep(Duration::from_millis(TRANSIENT_FS_RETRY_DELAY_MS));
            }
        }
    }
}

/// `fs.rmSync(path, { force: true })` — a missing file is not an error.
fn remove_file_force(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// pid liveness
// ---------------------------------------------------------------------------

/// Synchronous liveness probe via the null-signal trick, faithful to mjs
/// `isPidAlive` including its JS `Number()` coercion of whatever the lock
/// body carried: missing/unparsable/non-positive-integer pid → dead;
/// `kill(pid, 0)` success → alive; ESRCH → dead; EPERM (exists, can't
/// signal) → alive; anything else → alive (liveness genuinely unknowable is
/// treated conservatively as alive so a live holder is never falsely stolen
/// below [`HARD_STALE_MS`] — the ceiling is exactly the guard for that case).
pub fn is_pid_alive(pid: Option<&Value>) -> bool {
    let n = js_number(pid);
    if !n.is_finite() || n.fract() != 0.0 || n <= 0.0 {
        return false;
    }
    pid_probe(n as i64)
}

/// JS `Number()` coercion over an optional JSON field (`None` = undefined →
/// NaN). Arrays/objects coerce to NaN here — an approximation of JS's
/// toPrimitive chain that can never differ for any body this protocol or
/// any real caller writes (pids are numbers or numeric strings).
fn js_number(v: Option<&Value>) -> f64 {
    match v {
        None => f64::NAN,
        Some(Value::Null) => 0.0,
        Some(Value::Bool(b)) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        Some(Value::String(s)) => {
            let t = crate::fsutil::js_trim(s);
            if t.is_empty() {
                0.0
            } else {
                t.parse::<f64>().unwrap_or(f64::NAN)
            }
        }
        Some(_) => f64::NAN,
    }
}

#[cfg(unix)]
fn pid_probe(pid: i64) -> bool {
    let rc = unsafe { libc::kill(pid as libc::pid_t, 0) };
    if rc == 0 {
        return true;
    }
    io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

#[cfg(not(unix))]
fn pid_probe(_pid: i64) -> bool {
    // No probe wired on this platform yet (Windows delivery is a Slice 6
    // contingency, D8): liveness is unknowable, and unknowable is treated as
    // ALIVE — exactly the mjs "any other errno → alive" posture — so a
    // holder here is only ever stolen past the HARD_STALE_MS ceiling.
    true
}

// ---------------------------------------------------------------------------
// contention telemetry (multisession-native-3, C3) — fail-open, lock-free
// ---------------------------------------------------------------------------

/// One JSON line per store-lock acquire OUTCOME appended to
/// `.bee/logs/contention.jsonl`. Deliberately a bare mkdir+append with every
/// failure swallowed (never routed through `fsutil::append_jsonl`, which
/// correctly propagates — see fsutil's module doc): a telemetry write must
/// never change a lock acquisition's outcome, and it runs INSIDE the lock
/// primitives so it must never itself take a lock.
fn append_contention_telemetry(root: &Path, record: &Value) {
    let attempt = || -> io::Result<()> {
        let logs_dir = root.join(".bee").join("logs");
        fs::create_dir_all(&logs_dir)?;
        let line = format!("{record}\n");
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logs_dir.join("contention.jsonl"))?;
        f.write_all(line.as_bytes())
    };
    let _ = attempt();
}

/// Schema per multisession-native C3: workflow_id/workspace_id/resource are
/// not knowable at this layer and always null — reserved fields, mirrored
/// verbatim from the mjs record builder.
fn build_contention_record(
    name: &str,
    session: Option<&str>,
    wait_start_ms: i64,
    holder_session: Option<&Value>,
    result: &str,
) -> Value {
    json!({
        "ts": iso8601_millis(now_ms()),
        "lock_name": name,
        "lock_wait_ms": now_ms() - wait_start_ms,
        "holder_session": holder_session.cloned().unwrap_or(Value::Null),
        "caller_session": session.map(|s| Value::String(s.to_string())).unwrap_or(Value::Null),
        "workflow_id": Value::Null,
        "workspace_id": Value::Null,
        "resource": Value::Null,
        "result": result,
    })
}

// ---------------------------------------------------------------------------
// core protocol steps
// ---------------------------------------------------------------------------

fn read_holder(lock_path: &Path) -> Option<Value> {
    let text = fs::read_to_string(lock_path).ok()?;
    serde_json::from_str::<Value>(&text).ok()
}

fn mtime_ms(meta: &fs::Metadata) -> Option<i64> {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
}

/// O_EXCL create+write of the body line. `Ok(false)` = somebody else holds
/// the file (EEXIST); transient FS errors retried per policy; anything else
/// propagates.
fn try_acquire(lock_path: &Path, body: &Value) -> io::Result<bool> {
    let line = format!("{body}\n");
    let result = with_transient_fs_retry(|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)?;
        f.write_all(line.as_bytes())
    });
    match result {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(e),
    }
}

/// Judges whether `lock_path` is CURRENTLY eligible for stale takeover
/// (mtime + pid-liveness rule, hardening-1-7-10) and, when eligible, returns
/// a SNAPSHOT of the holder body observed at this exact moment (possibly
/// `None` for an unreadable body — which `perform_takeover_claim` will then
/// never match, so a corrupt stale lock is never stolen, same as mjs).
/// `None` overall = not eligible (fresh/live holder, or the lock vanished —
/// normal retry, never an error).
fn judge_stale_takeover_eligibility(lock_path: &Path, now_ms: i64) -> Option<Option<Value>> {
    let meta = fs::metadata(lock_path).ok()?;
    let age_ms = now_ms - mtime_ms(&meta)?;
    if age_ms <= STALE_MS {
        return None;
    }
    let holder_before = read_holder(lock_path);
    if age_ms <= HARD_STALE_MS {
        let pid = holder_before.as_ref().and_then(|h| h.get("pid"));
        if is_pid_alive(pid) {
            return None; // live holder — legitimately long-running, never stolen
        }
    }
    Some(holder_before)
}

/// Attempt exactly one takeover rename of `lock_path`, moving WHATEVER
/// currently occupies it to a fresh pid+random-unique staging path. `None` =
/// another racer already consumed the source (ENOENT — a normal loss, never
/// an error). Returns the staging path plus whatever content the rename
/// actually captured — never assumed to still be the judged snapshot.
fn rename_for_takeover(lock_path: &Path, now_ms: i64) -> io::Result<Option<(PathBuf, Option<Value>)>> {
    let suffix = random_hex(4)?;
    let mut name = lock_path.as_os_str().to_owned();
    name.push(format!(".stale-{}-{}-{}", std::process::id(), now_ms, suffix));
    let stale_path = PathBuf::from(name);
    match with_transient_fs_retry(|| fs::rename(lock_path, &stale_path)) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    }
    let holder_after = read_holder(&stale_path);
    Ok(Some((stale_path, holder_after)))
}

/// Given what a takeover rename actually captured, decide whether it was
/// truly the lock judged stale (`holder_before`) — deleting the corpse and
/// reporting a win — or a MISMATCH, in which case put it back (unless a
/// third racer has since legitimately re-occupied `lock_path`, which must
/// never be clobbered) and report a loss (rel180-4).
fn settle_takeover(
    lock_path: &Path,
    stale_path: &Path,
    holder_before: Option<&Value>,
    holder_after: Option<&Value>,
) -> io::Result<bool> {
    if same_holder_identity(holder_after, holder_before) {
        with_transient_fs_retry(|| remove_file_force(stale_path))?;
        return Ok(true);
    }
    // We renamed away a DIFFERENT, fresher lock than the one judged stale —
    // put it back unless lock_path has since been re-occupied by yet another
    // legitimate acquisition (never clobber that); either way this call did
    // NOT win the takeover.
    let occupied = fs::metadata(lock_path).is_ok();
    if !occupied && with_transient_fs_retry(|| fs::rename(stale_path, lock_path)).is_ok() {
        return Ok(false);
    }
    // Lost the restore race too (or lock_path was occupied) — drop the
    // corpse rather than leave two files behind.
    with_transient_fs_retry(|| remove_file_force(stale_path))?;
    Ok(false)
}

/// Re-verifies the judged snapshot immediately before the rename (rel190-2
/// shrank the vacancy window this way), performs the rename, then settles by
/// post-rename identity — the full two-sided verification chain.
fn perform_takeover_claim(lock_path: &Path, now_ms: i64, holder_before: Option<&Value>) -> io::Result<bool> {
    if !same_holder_identity(read_holder(lock_path).as_ref(), holder_before) {
        return Ok(false); // already changed since we judged it stale — never touch it
    }
    let renamed = match rename_for_takeover(lock_path, now_ms)? {
        Some(r) => r,
        None => return Ok(false),
    };
    settle_takeover(lock_path, &renamed.0, holder_before, renamed.1.as_ref())
}

fn try_stale_takeover(lock_path: &Path, now_ms: i64) -> io::Result<bool> {
    let eligibility = match judge_stale_takeover_eligibility(lock_path, now_ms) {
        Some(e) => e,
        None => return Ok(false),
    };
    perform_takeover_claim(lock_path, now_ms, eligibility.as_ref())
}

// ---------------------------------------------------------------------------
// public API
// ---------------------------------------------------------------------------

/// A held store lock returned by [`acquire_store_lock_once`]. Release is
/// EXPLICIT, matching the mjs `{ acquired, release }` shape — there is no
/// auto-release on drop, because a crashed holder leaving its lock file
/// behind IS part of the protocol (that is what the stale-takeover path
/// exists for). `release()` is idempotent and only ever removes a lock this
/// acquisition created (matched by pid + per-call token), never someone
/// else's — including one that took over after this lock somehow went stale.
#[derive(Debug)]
pub struct StoreLockGuard {
    lock_path: PathBuf,
    token: String,
    pid: u32,
    released: bool,
}

impl StoreLockGuard {
    pub fn release(&mut self) -> io::Result<()> {
        if self.released {
            return Ok(());
        }
        self.released = true;
        checked_release(&self.lock_path, &self.token, self.pid)
    }

    /// The random per-acquisition token recorded in the lock body.
    pub fn token(&self) -> &str {
        &self.token
    }
}

fn checked_release(lock_path: &Path, token: &str, pid: u32) -> io::Result<()> {
    let holder = read_holder(lock_path);
    let matches = holder
        .as_ref()
        .map(|h| {
            js_strict_eq(h.get("token"), Some(&Value::String(token.to_string())))
                && js_strict_eq(h.get("pid"), Some(&json!(pid)))
        })
        .unwrap_or(false);
    if matches {
        with_transient_fs_retry(|| remove_file_force(lock_path))?;
    }
    Ok(())
}

/// Outcome of the single-attempt sync acquire — mirrors mjs's
/// `{ acquired: true, release }` / `{ acquired: false, holder }`.
#[derive(Debug)]
pub enum OnceOutcome {
    Acquired(StoreLockGuard),
    Busy { holder: Option<Value> },
}

/// The synchronous, single-attempt entry point (`acquireStoreLockOnceSync`):
/// exactly one acquire attempt, and — only if that first attempt found the
/// lock stale-eligible and won the takeover race — exactly one follow-up
/// acquire attempt. Anything else (a live holder, or losing the takeover
/// race) is reported back as [`OnceOutcome::Busy`] rather than waited out.
/// No retry loop, no sleep: this is the hooks-never-wait posture's sibling.
pub fn acquire_store_lock_once(root: &Path, name: &str) -> io::Result<OnceOutcome> {
    ensure_dir(&locks_dir(root))?;
    let lock_path = lock_file_path(root, name);
    let token = random_hex(8)?;
    let session = env_session_id();
    let session_value = session
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    let start_ms = now_ms();
    let pid = std::process::id();
    let body = json!({
        "pid": pid,
        "session": session_value,
        "ts": iso8601_millis(start_ms),
        "token": token,
    });

    let mut acquired = try_acquire(&lock_path, &body)?;
    let mut contended_holder_session: Option<Value> = None;
    if !acquired {
        if let Some(contending) = read_holder(&lock_path) {
            if contending.is_object() {
                if let Some(s) = contending.get("session") {
                    if js_truthy(s) {
                        contended_holder_session = Some(s.clone());
                    }
                }
            }
        }
        if try_stale_takeover(&lock_path, start_ms)? {
            let mut refreshed = body.clone();
            refreshed["ts"] = Value::String(iso8601_millis(now_ms()));
            acquired = try_acquire(&lock_path, &refreshed)?;
        }
    }
    if !acquired {
        let holder = read_holder(&lock_path);
        let holder_session = holder
            .as_ref()
            .and_then(|h| h.get("session"))
            .filter(|s| js_truthy(s))
            .cloned()
            .or(contended_holder_session);
        append_contention_telemetry(
            root,
            &build_contention_record(name, session.as_deref(), start_ms, holder_session.as_ref(), "busy"),
        );
        return Ok(OnceOutcome::Busy { holder });
    }
    append_contention_telemetry(
        root,
        &build_contention_record(name, session.as_deref(), start_ms, contended_holder_session.as_ref(), "acquired"),
    );
    Ok(OnceOutcome::Acquired(StoreLockGuard {
        lock_path,
        token,
        pid,
        released: false,
    }))
}

/// Retry/backoff options for [`with_store_lock`]. The default is the CLI
/// verb posture (~100 tries × 50 ms ≈ 5 s worst-case wait);
/// [`LockOptions::try_once`] is the hooks-never-wait posture (msh-5, D5
/// Δ3-amended: every store write on the hook-driven heartbeat/lease-renewal
/// touch path passes `maxAttempts: 1` instead of the CLI's retry budget).
#[derive(Debug, Clone, Copy)]
pub struct LockOptions {
    pub max_attempts: u32,
    pub retry_delay_ms: u64,
}

impl Default for LockOptions {
    fn default() -> Self {
        Self {
            max_attempts: MAX_ATTEMPTS,
            retry_delay_ms: RETRY_DELAY_MS,
        }
    }
}

impl LockOptions {
    pub fn try_once() -> Self {
        Self {
            max_attempts: 1,
            ..Self::default()
        }
    }
}

/// Typed refusal returned by [`with_store_lock`] on timeout — never a silent
/// fall-through to an unlocked write. `Busy` carries the holder parsed from
/// the lock body and renders the same message as mjs `LockBusyError`.
#[derive(Debug)]
pub enum WithLockError {
    Busy { name: String, holder: Option<Value> },
    Io(io::Error),
}

impl std::fmt::Display for WithLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WithLockError::Busy { name, holder } => {
                let who = match holder {
                    Some(h) if h.is_object() => format!(
                        "pid={} session={} since {}",
                        display_field(h.get("pid")),
                        display_field(h.get("session")),
                        display_field(h.get("ts")),
                    ),
                    _ => "unknown holder".to_string(),
                };
                write!(f, "lock \"{name}\" busy: held by {who}")
            }
            WithLockError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for WithLockError {}

impl From<io::Error> for WithLockError {
    fn from(e: io::Error) -> Self {
        WithLockError::Io(e)
    }
}

/// `holder.pid ?? 'unknown'` rendering: null/undefined → "unknown", strings
/// bare (no quotes), everything else via its JSON form.
fn display_field(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "unknown".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// Panic-path release backstop for [`with_store_lock`] — mjs's `finally`.
struct FinallyRelease<'a> {
    lock_path: &'a Path,
    token: &'a str,
    pid: u32,
    armed: bool,
}

impl Drop for FinallyRelease<'_> {
    fn drop(&mut self) {
        if self.armed {
            let _ = checked_release(self.lock_path, self.token, self.pid);
        }
    }
}

/// `withStoreLock(root, name, fn, options)` — run `f` with
/// `.bee/locks/<name>.lock` held exclusively across processes (and across
/// runtimes: mjs holders deny this function and vice versa). `f`'s return
/// value propagates unchanged. Release always runs (a `Drop` backstop covers
/// panics), and only ever removes a lock THIS acquisition created (pid +
/// per-call token match). Staleness is re-verified at every retry on the
/// real filesystem mtime — never cached from an earlier check. The
/// inter-attempt sleep only runs when another attempt will follow. On
/// exhaustion returns [`WithLockError::Busy`] naming the current holder.
pub fn with_store_lock<T>(
    root: &Path,
    name: &str,
    options: LockOptions,
    f: impl FnOnce() -> T,
) -> Result<T, WithLockError> {
    ensure_dir(&locks_dir(root)).map_err(WithLockError::Io)?;
    let lock_path = lock_file_path(root, name);
    let token = random_hex(8)?;
    let pid = std::process::id();
    let session = env_session_id();
    let session_value = session
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    let wait_start_ms = now_ms();
    let mut acquired = false;
    let mut contended_holder_session: Option<Value> = None;

    for attempt in 0..options.max_attempts {
        let nm = now_ms();
        let body = json!({
            "pid": pid,
            "session": session_value.clone(),
            "ts": iso8601_millis(nm),
            "token": token,
        });
        if try_acquire(&lock_path, &body)? {
            acquired = true;
            break;
        }
        if let Some(contending) = read_holder(&lock_path) {
            if contending.is_object() {
                if let Some(s) = contending.get("session") {
                    if js_truthy(s) {
                        contended_holder_session = Some(s.clone());
                    }
                }
            }
        }
        // Staleness is re-verified at THIS retry, on the real filesystem
        // mtime — never cached from an earlier check.
        if try_stale_takeover(&lock_path, nm)? {
            // We just freed the slot ourselves; race for it immediately
            // rather than waiting a full retry interval behind everyone else.
            let mut refreshed = body.clone();
            refreshed["ts"] = Value::String(iso8601_millis(now_ms()));
            if try_acquire(&lock_path, &refreshed)? {
                acquired = true;
                break;
            }
        }
        if attempt + 1 < options.max_attempts {
            std::thread::sleep(Duration::from_millis(options.retry_delay_ms));
        }
    }

    if !acquired {
        let holder = read_holder(&lock_path);
        let holder_session = holder
            .as_ref()
            .and_then(|h| h.get("session"))
            .filter(|s| js_truthy(s))
            .cloned()
            .or(contended_holder_session);
        append_contention_telemetry(
            root,
            &build_contention_record(
                name,
                session.as_deref(),
                wait_start_ms,
                holder_session.as_ref(),
                "busy",
            ),
        );
        return Err(WithLockError::Busy {
            name: name.to_string(),
            holder,
        });
    }

    append_contention_telemetry(
        root,
        &build_contention_record(
            name,
            session.as_deref(),
            wait_start_ms,
            contended_holder_session.as_ref(),
            "acquired",
        ),
    );

    let mut finally = FinallyRelease {
        lock_path: &lock_path,
        token: &token,
        pid,
        armed: true,
    };
    let out = f();
    finally.armed = false; // normal path releases explicitly, propagating errors like mjs's finally
    drop(finally);
    checked_release(&lock_path, &token, pid).map_err(WithLockError::Io)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn lock_path_scheme_distinct_names_never_collide_after_sanitization() {
        let root = tmp_root();
        let a = lock_file_path(root.path(), "cells:a");
        let b = lock_file_path(root.path(), "cells_a");
        let c = lock_file_path(root.path(), "cells/a");
        assert_ne!(a, b, "sanitization collision: cells:a vs cells_a");
        assert_ne!(a, c, "sanitization collision: cells:a vs cells/a");
        assert_ne!(b, c, "sanitization collision: cells_a vs cells/a");
        // Pure function: same logical name -> same file, across calls.
        assert_eq!(a, lock_file_path(root.path(), "cells:a"));
        // Shape: sanitized stem + "-" + 8 hex + ".lock" under .bee/locks/.
        let fname = a.file_name().unwrap().to_string_lossy().into_owned();
        assert!(fname.starts_with("cells_a-") && fname.ends_with(".lock"), "unexpected shape: {fname}");
        assert_eq!(a.parent().unwrap(), locks_dir(root.path()));
    }

    #[test]
    fn lock_iso8601_millis_matches_node_to_iso_string() {
        // Table of node oracle values: new Date(ms).toISOString().
        let cases: Vec<(i64, &str)> = vec![
            (0, "1970-01-01T00:00:00.000Z"),
            (946_684_800_000, "2000-01-01T00:00:00.000Z"),
            (1_700_000_000_000, "2023-11-14T22:13:20.000Z"),
            (1_784_005_455_015, "2026-07-14T05:04:15.015Z"),
            (951_782_400_000, "2000-02-29T00:00:00.000Z"), // leap day
        ];
        for (ms, expected) in cases {
            assert_eq!(iso8601_millis(ms), expected, "ms={ms}");
        }
    }

    #[test]
    fn lock_is_pid_alive_probe_semantics() {
        // Own pid: provably alive.
        assert!(is_pid_alive(Some(&json!(std::process::id()))));
        // Numeric-string pid of a live process: alive (JS Number coercion).
        assert!(is_pid_alive(Some(&json!(std::process::id().to_string()))));
        // Reaped child: provably dead.
        let status = std::process::Command::new("true").status().expect("spawn true");
        assert!(status.success());
        // (the child's pid is gone after wait; probe a definitely-exited pid
        // by spawning + waiting and reading its id)
        let child = std::process::Command::new("true").spawn().expect("spawn true");
        let dead_pid = child.id();
        let mut child = child;
        child.wait().expect("wait child");
        assert!(!is_pid_alive(Some(&json!(dead_pid))), "reaped child pid probed alive");
        // Junk pids: dead without probing (Number()/isInteger gate).
        for junk in [json!(0), json!(-5), json!(1.5), json!("abc"), json!(null), json!(false)] {
            assert!(!is_pid_alive(Some(&junk)), "junk pid {junk} probed alive");
        }
        assert!(!is_pid_alive(None), "undefined pid probed alive");
    }

    #[test]
    fn lock_same_holder_identity_js_semantics_table() {
        let full = json!({"pid": 1, "token": "t", "ts": "x"});
        let cases: Vec<(Option<Value>, Option<Value>, bool, &str)> = vec![
            (Some(full.clone()), Some(full.clone()), true, "identical bodies match"),
            (
                Some(full.clone()),
                Some(json!({"pid": 1, "token": "OTHER", "ts": "x"})),
                false,
                "token mismatch",
            ),
            (
                Some(full.clone()),
                Some(json!({"pid": 2, "token": "t", "ts": "x"})),
                false,
                "pid mismatch",
            ),
            (
                Some(full.clone()),
                Some(json!({"pid": 1, "token": "t", "ts": "y"})),
                false,
                "ts mismatch",
            ),
            (None, None, false, "corrupt vs corrupt never matches (Boolean(null && ...))"),
            (Some(full.clone()), None, false, "corrupt candidate never matches"),
            (
                Some(json!({})),
                Some(json!({})),
                true,
                "two empty objects: undefined === undefined on all three fields",
            ),
            (
                Some(json!(5)),
                Some(json!(7)),
                true,
                "two truthy non-objects: all fields undefined === undefined (JS quirk, ported faithfully)",
            ),
            (Some(json!(0)), Some(full.clone()), false, "falsy candidate never matches"),
            (
                Some(json!({"pid": 1, "token": "t", "ts": "x", "session": "a"})),
                Some(json!({"pid": 1, "token": "t", "ts": "x", "session": "b"})),
                true,
                "session is NOT part of identity",
            ),
        ];
        for (a, b, expected, why) in cases {
            assert_eq!(same_holder_identity(a.as_ref(), b.as_ref()), expected, "{why}");
        }
    }

    #[test]
    fn lock_corrupt_stale_lock_is_never_taken_over() {
        // mjs behavior preserved: judge passes an unreadable holder through
        // as a null snapshot, and perform_takeover_claim can never match it,
        // so even a way-past-ceiling corrupt lock stays untouched.
        let root = tmp_root();
        ensure_dir(&locks_dir(root.path())).unwrap();
        let lock_path = lock_file_path(root.path(), "corrupt");
        fs::write(&lock_path, b"{ not json").unwrap();
        let two_hours_ago = SystemTime::now() - Duration::from_secs(2 * 3600);
        let f = fs::OpenOptions::new().write(true).open(&lock_path).unwrap();
        f.set_modified(two_hours_ago).unwrap();
        drop(f);

        let won = try_stale_takeover(&lock_path, now_ms()).unwrap();
        assert!(!won, "corrupt lock was taken over — mjs never does this");
        assert!(lock_path.exists(), "corrupt lock file must be left untouched");
        assert_eq!(fs::read(&lock_path).unwrap(), b"{ not json", "corrupt body must be byte-identical");
    }

    #[test]
    fn lock_settle_takeover_branch_table() {
        // Three branches of the post-rename decision (rel180-4/rel190-2).
        let judged = json!({"pid": 999_999, "token": "judged", "ts": "2020-01-01T00:00:00.000Z"});
        let fresher = json!({"pid": 4242, "token": "fresher", "ts": "2026-01-01T00:00:00.000Z"});
        let third = json!({"pid": 7, "token": "third", "ts": "2026-02-02T00:00:00.000Z"});

        // (a) match: corpse deleted, win reported.
        {
            let root = tmp_root();
            ensure_dir(&locks_dir(root.path())).unwrap();
            let lock_path = lock_file_path(root.path(), "settle");
            let stale = lock_path.with_extension("lock.stale-x");
            fs::write(&stale, format!("{judged}\n")).unwrap();
            let won = settle_takeover(&lock_path, &stale, Some(&judged), Some(&judged)).unwrap();
            assert!(won);
            assert!(!stale.exists(), "matched corpse must be deleted");
        }
        // (b) mismatch, lock path vacant: displaced fresher lock restored, loss reported.
        {
            let root = tmp_root();
            ensure_dir(&locks_dir(root.path())).unwrap();
            let lock_path = lock_file_path(root.path(), "settle");
            let stale = lock_path.with_extension("lock.stale-x");
            fs::write(&stale, format!("{fresher}\n")).unwrap();
            let won = settle_takeover(&lock_path, &stale, Some(&judged), Some(&fresher)).unwrap();
            assert!(!won);
            assert!(!stale.exists());
            let restored = read_holder(&lock_path).expect("displaced lock must be restored");
            assert!(same_holder_identity(Some(&restored), Some(&fresher)));
        }
        // (c) mismatch, lock path re-occupied by a third racer: never
        // clobbered; corpse dropped, loss reported.
        {
            let root = tmp_root();
            ensure_dir(&locks_dir(root.path())).unwrap();
            let lock_path = lock_file_path(root.path(), "settle");
            let stale = lock_path.with_extension("lock.stale-x");
            fs::write(&stale, format!("{fresher}\n")).unwrap();
            fs::write(&lock_path, format!("{third}\n")).unwrap();
            let won = settle_takeover(&lock_path, &stale, Some(&judged), Some(&fresher)).unwrap();
            assert!(!won);
            assert!(!stale.exists(), "corpse must be dropped, not left behind");
            let occupant = read_holder(&lock_path).expect("third racer's lock must survive");
            assert!(same_holder_identity(Some(&occupant), Some(&third)), "third racer's lock was clobbered");
        }
    }

    #[test]
    fn lock_pre_rename_reverification_declines_changed_content() {
        // perform_takeover_claim must decline when the body changed between
        // judgment and claim (rel190-2's pre-rename identity check).
        let root = tmp_root();
        ensure_dir(&locks_dir(root.path())).unwrap();
        let lock_path = lock_file_path(root.path(), "reverify");
        let judged = json!({"pid": 999_999, "token": "old", "ts": "2020-01-01T00:00:00.000Z"});
        let replaced = json!({"pid": 4242, "token": "new", "ts": "2026-01-01T00:00:00.000Z"});
        fs::write(&lock_path, format!("{replaced}\n")).unwrap();
        let won = perform_takeover_claim(&lock_path, now_ms(), Some(&judged)).unwrap();
        assert!(!won, "claim must decline content that changed since judgment");
        let occupant = read_holder(&lock_path).unwrap();
        assert!(same_holder_identity(Some(&occupant), Some(&replaced)), "changed lock must be untouched");
    }

    #[test]
    fn lock_transient_fs_retry_policy() {
        // Transient error: retried until success.
        let mut calls = 0;
        let result = with_transient_fs_retry(|| {
            calls += 1;
            if calls < 3 {
                Err(io::Error::from_raw_os_error(transient_code()))
            } else {
                Ok(calls)
            }
        });
        assert_eq!(result.unwrap(), 3);

        // Non-transient error: immediate propagation, exactly one call.
        let mut calls = 0;
        let result: io::Result<()> = with_transient_fs_retry(|| {
            calls += 1;
            Err(io::Error::new(io::ErrorKind::NotFound, "nope"))
        });
        assert!(result.is_err());
        assert_eq!(calls, 1);

        // Exhaustion: budget is exactly TRANSIENT_FS_RETRY_ATTEMPTS calls.
        let mut calls = 0;
        let result: io::Result<()> = with_transient_fs_retry(|| {
            calls += 1;
            Err(io::Error::from_raw_os_error(transient_code()))
        });
        assert!(result.is_err());
        assert_eq!(calls, TRANSIENT_FS_RETRY_ATTEMPTS);
    }

    #[cfg(unix)]
    fn transient_code() -> i32 {
        libc::EBUSY
    }
    #[cfg(windows)]
    fn transient_code() -> i32 {
        32 // ERROR_SHARING_VIOLATION
    }

    #[test]
    fn lock_body_shape_matches_protocol() {
        let root = tmp_root();
        let outcome = acquire_store_lock_once(root.path(), "shape").unwrap();
        let mut guard = match outcome {
            OnceOutcome::Acquired(g) => g,
            OnceOutcome::Busy { holder } => panic!("fresh lock busy: {holder:?}"),
        };
        let lock_path = lock_file_path(root.path(), "shape");
        let raw = fs::read_to_string(&lock_path).unwrap();
        assert!(raw.ends_with('\n'), "body line must end with newline like mjs");
        let body: Value = serde_json::from_str(raw.trim()).unwrap();
        assert_eq!(body["pid"], json!(std::process::id()));
        assert!(body.get("session").is_some(), "session field must be present (null allowed)");
        let ts = body["ts"].as_str().expect("ts string");
        assert!(
            ts.len() == 24 && ts.ends_with('Z') && &ts[10..11] == "T",
            "ts must be ISO-8601 millis Z: {ts}"
        );
        let token = body["token"].as_str().expect("token string");
        assert_eq!(token.len(), 16, "token must be 8 random bytes hex");
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(token, guard.token());
        guard.release().unwrap();
        assert!(!lock_path.exists(), "release must remove own lock");
        // Idempotent.
        guard.release().unwrap();
    }

    #[test]
    fn lock_release_never_removes_someone_elses_lock() {
        let root = tmp_root();
        let mut guard = match acquire_store_lock_once(root.path(), "own").unwrap() {
            OnceOutcome::Acquired(g) => g,
            _ => panic!("fresh lock busy"),
        };
        // Simulate a takeover: someone replaced the body after our acquire.
        let lock_path = lock_file_path(root.path(), "own");
        let foreign = json!({"pid": 1, "session": null, "ts": "2026-01-01T00:00:00.000Z", "token": "feedfeedfeedfeed"});
        fs::write(&lock_path, format!("{foreign}\n")).unwrap();
        guard.release().unwrap();
        assert!(lock_path.exists(), "release removed a lock it does not own");
    }
}
