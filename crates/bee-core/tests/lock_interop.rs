//! lock_interop — cross-runtime conformance suite for the D9 lock protocol
//! (rust-port-3): node children drive the REAL `.bee/bin/lib/lock.mjs`
//! (resolved by walking ancestors from CARGO_MANIFEST_DIR, passed to the
//! file-based driver `tests/support/lock_driver.mjs` — never `node -e`)
//! while `bee_core::lock` contends the same lock files, and the reverse.
//!
//! Every store root below comes from `tempfile::tempdir()` — never the
//! repo's live `.bee/` store (the driver refuses a root that looks like a
//! bee checkout). Staleness is controlled by backdating the lock body's
//! `ts` and the file mtime — never by sleeping.

use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bee_core::lock::{
    acquire_store_lock_once, iso8601_millis, lock_file_path, locks_dir, with_store_lock,
    LockOptions, OnceOutcome, WithLockError,
};

fn driver_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/lock_driver.mjs")
}

/// Resolve the frozen `.bee/bin/lib/lock.mjs` by walking ancestors from
/// CARGO_MANIFEST_DIR (the cell-mandated resolution scheme).
fn find_lock_mjs() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        let candidate = dir.join(".bee").join("bin").join("lib").join("lock.mjs");
        if candidate.exists() {
            return candidate;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "could not locate .bee/bin/lib/lock.mjs walking ancestors from {}",
        env!("CARGO_MANIFEST_DIR")
    );
}

fn run_driver_once(op: &str, root: &Path, name: &str, session: &str) -> Value {
    let output = Command::new("node")
        .arg(driver_script())
        .arg(op)
        .arg(find_lock_mjs())
        .arg(root)
        .arg(name)
        .env("BEE_SESSION_ID", session)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn node lock_driver — is `node` on PATH?");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let line = stdout
        .lines()
        .next()
        .unwrap_or_else(|| panic!("driver {op} produced no output — stderr: {stderr}"));
    serde_json::from_str(line)
        .unwrap_or_else(|e| panic!("driver {op} emitted non-JSON {line:?}: {e} — stderr: {stderr}"))
}

/// A node child holding a lock LIVE through the real lock.mjs until told to
/// release — the "live node holder" every deny/steal scenario needs.
struct HoldChild {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    pid: u64,
}

fn spawn_hold(root: &Path, name: &str, session: &str) -> HoldChild {
    let mut child = Command::new("node")
        .arg(driver_script())
        .arg("hold")
        .arg(find_lock_mjs())
        .arg(root)
        .arg(name)
        .env("BEE_SESSION_ID", session)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn node lock_driver hold");
    let stdin = child.stdin.take().expect("hold child stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("hold child stdout"));
    let mut line = String::new();
    stdout.read_line(&mut line).expect("read hold child first line");
    let v: Value = serde_json::from_str(line.trim())
        .unwrap_or_else(|e| panic!("hold child emitted non-JSON {line:?}: {e}"));
    assert_eq!(
        v["acquired"],
        json!(true),
        "node hold child failed to acquire a fresh lock: {v}"
    );
    let pid = v["pid"].as_u64().expect("hold child pid");
    HoldChild {
        child,
        stdin,
        stdout,
        pid,
    }
}

impl HoldChild {
    fn release(self) {
        let HoldChild {
            mut child,
            mut stdin,
            mut stdout,
            ..
        } = self;
        stdin.write_all(b"release\n").expect("write release line");
        stdin.flush().expect("flush release line");
        let mut line = String::new();
        stdout.read_line(&mut line).expect("read release ack");
        let v: Value = serde_json::from_str(line.trim())
            .unwrap_or_else(|e| panic!("hold child release ack non-JSON {line:?}: {e}"));
        assert_eq!(v["released"], json!(true), "node hold child did not confirm release: {v}");
        // Close the stdin pipe before waiting so the child sees EOF even if
        // its own explicit exit path ever regresses.
        drop(stdin);
        child.wait().expect("wait hold child");
    }
}

/// Backdate a lock: rewrite the body `ts` to `age` ago (pid/token/session
/// preserved) and set the file mtime to the same instant — never a sleep.
fn backdate_lock(lock_path: &Path, age: Duration) {
    let then = SystemTime::now() - age;
    let then_ms = then
        .duration_since(UNIX_EPOCH)
        .expect("backdated instant before epoch")
        .as_millis() as i64;
    let text = fs::read_to_string(lock_path).expect("read lock body to backdate");
    let mut body: Value = serde_json::from_str(text.trim()).expect("parse lock body to backdate");
    body["ts"] = Value::String(iso8601_millis(then_ms));
    fs::write(lock_path, format!("{body}\n")).expect("rewrite backdated body");
    let f = fs::OpenOptions::new()
        .write(true)
        .open(lock_path)
        .expect("open lock to backdate mtime");
    f.set_modified(then).expect("set_modified");
}

fn contention_records(root: &Path) -> Vec<Value> {
    bee_core::fsutil::read_jsonl(&root.join(".bee").join("logs").join("contention.jsonl"))
}

fn stale_leftovers(root: &Path) -> Vec<String> {
    let dir = locks_dir(root);
    match fs::read_dir(&dir) {
        Err(_) => Vec::new(),
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".stale-"))
            .collect(),
    }
}

/// Truth: both runtimes derive the IDENTICAL lock file path for the same
/// logical name — the precondition making every other test here meaningful
/// (different paths would mean the runtimes silently never contend at all).
#[test]
fn lock_path_scheme_matches_real_mjs_for_tricky_names() {
    let dir = tempfile::tempdir().unwrap();
    let names = [
        "reservations",
        "cells:some-id",
        "cells:a",
        "cells_a", // must stay DISTINCT from cells:a via the sha256 suffix
        "a/b:c",
        "weird *?<>|\"name",
        "kh\u{00f3}a-vi\u{1ec7}t",
        "ctrl\u{0001}x",
    ];
    let mut seen = std::collections::HashSet::new();
    for name in names {
        let rust_path = lock_file_path(dir.path(), name);
        let v = run_driver_once("path", dir.path(), name, "path-check");
        let mjs_path = v["lockPath"].as_str().expect("driver lockPath");
        assert_eq!(
            rust_path.to_string_lossy(),
            mjs_path,
            "lock path scheme diverged from the real lockFilePath for {name:?}"
        );
        assert!(
            seen.insert(mjs_path.to_string()),
            "two distinct logical names collided on one lock file: {name:?} -> {mjs_path}"
        );
    }
}

/// Truth (must-have 1, forward direction): a rust contender never acquires
/// while a live node holder exists — single attempt, try-once mode, and a
/// bounded retry loop all deny; after the node holder releases, rust
/// acquires the same lock file.
#[test]
fn lock_interop_rust_denied_while_live_node_holder_then_acquires_after_release() {
    let dir = tempfile::tempdir().unwrap();
    let name = "interop-live";
    let holder = spawn_hold(dir.path(), name, "node-holder-sess");

    // Single sync attempt: busy, holder body names the node process.
    match acquire_store_lock_once(dir.path(), name).unwrap() {
        OnceOutcome::Acquired(_) => panic!("rust acquired while a live node holder exists"),
        OnceOutcome::Busy { holder: h } => {
            let h = h.expect("holder body must be readable");
            assert_eq!(h["pid"], json!(holder.pid), "holder pid must be the node child");
            assert_eq!(h["session"], json!("node-holder-sess"));
        }
    }

    // Bounded retry loop: still busy, typed refusal naming the holder.
    let err = with_store_lock(
        dir.path(),
        name,
        LockOptions {
            max_attempts: 3,
            retry_delay_ms: 10,
        },
        || panic!("critical section ran while a live node holder exists"),
    )
    .unwrap_err();
    match &err {
        WithLockError::Busy { name: n, holder: h } => {
            assert_eq!(n, name);
            assert_eq!(h.as_ref().unwrap()["pid"], json!(holder.pid));
        }
        other => panic!("expected Busy, got {other:?}"),
    }
    let msg = err.to_string();
    assert!(
        msg.contains(&format!("lock \"{name}\" busy: held by pid={}", holder.pid))
            && msg.contains("session=node-holder-sess"),
        "LockBusy message shape diverged: {msg}"
    );

    // The rust contender's busy telemetry names the node holder session.
    let busy: Vec<Value> = contention_records(dir.path())
        .into_iter()
        .filter(|r| r["result"] == json!("busy"))
        .collect();
    assert!(!busy.is_empty(), "busy contention telemetry missing");
    assert!(
        busy.iter().all(|r| r["holder_session"] == json!("node-holder-sess")),
        "busy telemetry must name the node holder session: {busy:?}"
    );

    holder.release();
    match acquire_store_lock_once(dir.path(), name).unwrap() {
        OnceOutcome::Acquired(mut g) => g.release().unwrap(),
        OnceOutcome::Busy { holder } => panic!("still busy after node release: {holder:?}"),
    }
}

/// Truth (must-have 1, reverse direction): the real lock.mjs never acquires
/// while a live rust holder exists, and reads the rust-written body; after
/// rust releases, node acquires.
#[test]
fn lock_interop_node_denied_while_rust_holder_then_acquires_after_release() {
    let dir = tempfile::tempdir().unwrap();
    let name = "interop-reverse";
    let mut guard = match acquire_store_lock_once(dir.path(), name).unwrap() {
        OnceOutcome::Acquired(g) => g,
        OnceOutcome::Busy { holder } => panic!("fresh lock busy: {holder:?}"),
    };

    let denied = run_driver_once("acquire-once", dir.path(), name, "node-contender-sess");
    assert_eq!(denied["acquired"], json!(false), "node acquired while rust holds: {denied}");
    let holder = &denied["holder"];
    assert_eq!(
        holder["pid"],
        json!(std::process::id()),
        "node must parse the rust-written holder body: {denied}"
    );
    assert_eq!(
        holder["token"],
        json!(guard.token()),
        "node-read token must match the rust acquisition token"
    );

    guard.release().unwrap();
    let won = run_driver_once("acquire-once", dir.path(), name, "node-contender-sess");
    assert_eq!(won["acquired"], json!(true), "node still denied after rust release: {won}");
}

/// Truth (must-have 2): a crashed node holder (stale mtime + provably dead
/// pid) is taken over by rust through the liveness-probe path, verified by
/// pid+token+ts identity, leaving no stale corpse behind.
#[test]
fn lock_interop_stale_dead_node_holder_taken_over_by_rust() {
    let dir = tempfile::tempdir().unwrap();
    let name = "interop-dead-takeover";
    let abandoned = run_driver_once("acquire-abandon", dir.path(), name, "node-abandon-sess");
    assert_eq!(abandoned["acquired"], json!(true), "abandon fixture failed: {abandoned}");
    let dead_pid = abandoned["pid"].as_u64().unwrap();

    let lock_path = lock_file_path(dir.path(), name);
    // Below the crashed-holder window the dead pid must NOT yet be stolen.
    match acquire_store_lock_once(dir.path(), name).unwrap() {
        OnceOutcome::Busy { .. } => {}
        OnceOutcome::Acquired(_) => panic!("fresh (non-stale) lock of a dead pid was stolen"),
    }

    backdate_lock(&lock_path, Duration::from_secs(60)); // > STALE_MS, << HARD_STALE_MS
    match acquire_store_lock_once(dir.path(), name).unwrap() {
        OnceOutcome::Acquired(mut g) => {
            let body: Value =
                serde_json::from_str(fs::read_to_string(&lock_path).unwrap().trim()).unwrap();
            assert_eq!(body["pid"], json!(std::process::id()), "takeover must install the rust holder");
            assert_ne!(body["pid"], json!(dead_pid));
            assert!(
                stale_leftovers(dir.path()).is_empty(),
                "stale corpse left behind after verified takeover"
            );
            g.release().unwrap();
        }
        OnceOutcome::Busy { holder } => {
            panic!("stale dead-pid holder not taken over: {holder:?}")
        }
    }
}

/// Truth (must-have 2 guard): a stale-aged but provably LIVE holder below
/// the hard ceiling is never stolen — long critical sections are legitimate.
#[test]
fn lock_interop_stale_live_node_holder_below_ceiling_never_stolen() {
    let dir = tempfile::tempdir().unwrap();
    let name = "interop-live-stale";
    let holder = spawn_hold(dir.path(), name, "node-long-holder");
    let lock_path = lock_file_path(dir.path(), name);
    backdate_lock(&lock_path, Duration::from_secs(60)); // stale-aged, pid alive

    match acquire_store_lock_once(dir.path(), name).unwrap() {
        OnceOutcome::Acquired(_) => panic!("live holder stolen below the hard ceiling"),
        OnceOutcome::Busy { holder: h } => {
            assert_eq!(h.unwrap()["pid"], json!(holder.pid));
        }
    }
    assert!(lock_path.exists(), "live holder's lock must survive the takeover attempt");
    holder.release();
}

/// Truth (must-have 2 ceiling): past HARD_STALE_MS the takeover proceeds
/// regardless of liveness (pid-reuse guard of last resort) — and the
/// displaced node holder's own release() must NOT remove the rust lock
/// (release only ever removes a lock matched by its own pid+token).
#[test]
fn lock_interop_hard_stale_live_holder_stolen_and_release_stays_scoped() {
    let dir = tempfile::tempdir().unwrap();
    let name = "interop-hard-stale";
    let holder = spawn_hold(dir.path(), name, "node-ancient-holder");
    let lock_path = lock_file_path(dir.path(), name);
    backdate_lock(&lock_path, Duration::from_secs(2 * 3600)); // past the 1h ceiling

    let mut guard = match acquire_store_lock_once(dir.path(), name).unwrap() {
        OnceOutcome::Acquired(g) => g,
        OnceOutcome::Busy { holder } => {
            panic!("past-ceiling holder not taken over: {holder:?}")
        }
    };
    let body: Value = serde_json::from_str(fs::read_to_string(&lock_path).unwrap().trim()).unwrap();
    assert_eq!(body["pid"], json!(std::process::id()));

    // The displaced node holder releases: its token no longer matches, so
    // the rust lock must survive untouched.
    holder.release();
    assert!(lock_path.exists(), "node release removed a lock it no longer owns");
    let body_after: Value =
        serde_json::from_str(fs::read_to_string(&lock_path).unwrap().trim()).unwrap();
    assert_eq!(body_after, body, "rust holder body changed across the foreign release");
    guard.release().unwrap();
    assert!(!lock_path.exists());
}

/// Truth (must-have 2, reverse direction): the real lock.mjs takes over a
/// rust-planted stale dead-pid lock — node's judge/claim path reads
/// rust-written bodies.
#[test]
fn lock_interop_node_takes_over_rust_planted_stale_dead_holder() {
    let dir = tempfile::tempdir().unwrap();
    let name = "interop-node-takeover";
    // A provably dead pid: spawn-and-reap a real child.
    let mut child = std::process::Command::new("true").spawn().expect("spawn true");
    let dead_pid = child.id();
    child.wait().expect("reap child");

    fs::create_dir_all(locks_dir(dir.path())).unwrap();
    let lock_path = lock_file_path(dir.path(), name);
    let body = json!({
        "pid": dead_pid,
        "session": "rust-planted",
        "ts": iso8601_millis(0),
        "token": "deadbeefdeadbeef",
    });
    fs::write(&lock_path, format!("{body}\n")).unwrap();
    backdate_lock(&lock_path, Duration::from_secs(60));

    let won = run_driver_once("acquire-once", dir.path(), name, "node-taker");
    assert_eq!(
        won["acquired"],
        json!(true),
        "node failed to take over a rust-planted stale dead-pid lock: {won}"
    );
}

/// Truth (must-have 3): the two-simultaneous-holders detector — under the
/// real protocol, a rust contender NEVER holds together with a live node
/// holder; and the detector itself is proven able to fail red by running the
/// same check against a deliberate protocol violation (the naive
/// unconditional-unlink takeover the spike's negative control reproduced).
#[test]
fn lock_two_simultaneous_holders_detector_fails_red_on_deliberate_violation() {
    // Part 1 — protocol path: no double hold, ever.
    let dir = tempfile::tempdir().unwrap();
    let name = "interop-negative";
    let holder = spawn_hold(dir.path(), name, "node-negative-holder");
    let mut protocol_double_hold = false;
    for _ in 0..5 {
        if let OnceOutcome::Acquired(_) = acquire_store_lock_once(dir.path(), name).unwrap() {
            protocol_double_hold = true; // node has NOT released — this would be a second holder
        }
    }
    assert!(
        !protocol_double_hold,
        "protocol violation: rust acquired while the node holder was live"
    );

    // Part 2 — red proof: DELIBERATELY violate the protocol (unconditional
    // unlink of a live holder's lock, the exact anti-pattern
    // settle_takeover's identity verification exists to prevent) and assert
    // the same detector DOES fire — the negative test can go red.
    let lock_path = lock_file_path(dir.path(), name);
    fs::remove_file(&lock_path).expect("deliberate violation: unlink live holder's lock");
    let mut violation_double_hold = false;
    if let OnceOutcome::Acquired(mut g) = acquire_store_lock_once(dir.path(), name).unwrap() {
        violation_double_hold = true; // node still holds; rust now also "holds"
        g.release().unwrap();
    }
    assert!(
        violation_double_hold,
        "negative control failed: the detector cannot observe a deliberate violation, so the protocol assertion above proves nothing"
    );
    holder.release();
}

/// Truth: hooks-never-wait — try-once mode returns Busy immediately against
/// a held lock, with no retry sleeps (bounded far below one retry budget).
#[test]
fn lock_try_once_mode_returns_busy_without_waiting() {
    let dir = tempfile::tempdir().unwrap();
    let name = "try-once";
    let mut guard = match acquire_store_lock_once(dir.path(), name).unwrap() {
        OnceOutcome::Acquired(g) => g,
        _ => panic!("fresh lock busy"),
    };

    let started = Instant::now();
    let result = with_store_lock(dir.path(), name, LockOptions::try_once(), || {
        panic!("critical section ran through a held lock")
    });
    let elapsed = started.elapsed();
    assert!(matches!(result, Err(WithLockError::Busy { .. })), "try-once must refuse typed Busy");
    assert!(
        elapsed < Duration::from_millis(2500),
        "try-once waited {elapsed:?} — it must never sleep on the lock"
    );
    guard.release().unwrap();
}

/// Truth: every acquire outcome lands one telemetry line with the C3 schema,
/// and a broken log location never changes an acquire's outcome (fail-open).
#[test]
fn lock_contention_telemetry_schema_and_fail_open() {
    let dir = tempfile::tempdir().unwrap();
    let name = "telemetry";

    // acquired record
    let mut guard = match acquire_store_lock_once(dir.path(), name).unwrap() {
        OnceOutcome::Acquired(g) => g,
        _ => panic!("fresh lock busy"),
    };
    // busy record (same-process second contender — no reentrancy in this protocol)
    assert!(matches!(
        acquire_store_lock_once(dir.path(), name).unwrap(),
        OnceOutcome::Busy { .. }
    ));
    guard.release().unwrap();

    let records = contention_records(dir.path());
    assert_eq!(records.len(), 2, "one line per acquire outcome: {records:?}");
    let expected_results = [json!("acquired"), json!("busy")];
    for (record, expected) in records.iter().zip(expected_results.iter()) {
        assert_eq!(&record["result"], expected);
        assert_eq!(record["lock_name"], json!(name));
        let ts = record["ts"].as_str().expect("ts");
        assert!(ts.len() == 24 && ts.ends_with('Z'), "ts must be ISO-8601 millis Z: {ts}");
        assert!(record["lock_wait_ms"].as_i64().expect("lock_wait_ms") >= 0);
        for reserved in ["workflow_id", "workspace_id", "resource"] {
            assert_eq!(record[reserved], Value::Null, "{reserved} must be reserved-null");
        }
        for field in ["holder_session", "caller_session"] {
            assert!(record.get(field).is_some(), "{field} must be present");
        }
    }

    // Fail-open: .bee/logs occupied by a FILE — mkdir fails, telemetry is
    // swallowed, the acquire outcome is unchanged.
    let broken = tempfile::tempdir().unwrap();
    fs::create_dir_all(broken.path().join(".bee")).unwrap();
    fs::write(broken.path().join(".bee").join("logs"), b"not a directory").unwrap();
    match acquire_store_lock_once(broken.path(), name).unwrap() {
        OnceOutcome::Acquired(mut g) => g.release().unwrap(),
        OnceOutcome::Busy { holder } => {
            panic!("broken telemetry location changed the acquire outcome: {holder:?}")
        }
    }
}

/// Truth: the waiting entry point runs its closure while holding the lock
/// and releases afterward — the same lock file is then free for the real
/// lock.mjs to acquire.
#[test]
fn lock_with_store_lock_runs_closure_then_node_can_acquire() {
    let dir = tempfile::tempdir().unwrap();
    let name = "closure";
    let lock_path = lock_file_path(dir.path(), name);
    let observed = with_store_lock(dir.path(), name, LockOptions::default(), || {
        assert!(lock_path.exists(), "lock must be held while the closure runs");
        41 + 1
    })
    .expect("with_store_lock");
    assert_eq!(observed, 42, "closure return value must propagate unchanged");
    assert!(!lock_path.exists(), "lock must be released after the closure");

    let won = run_driver_once("acquire-once", dir.path(), name, "node-after-closure");
    assert_eq!(won["acquired"], json!(true), "node denied after rust release: {won}");
}
