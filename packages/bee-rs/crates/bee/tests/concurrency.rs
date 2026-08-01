// Cross-process concurrency contracts, black-box over the SHIPPED binary.
//
// R5 port of the five .mjs race suites under `scripts/tests/`. Those suites
// exist for one reason: the store's safety rests on O_EXCL / store-lock
// mutual exclusion, and mutual exclusion is only real under genuine OS-process
// interleaving. Before this file the Rust tree had NO concurrency coverage at
// all (`rg "thread::spawn"` over crates/ found nothing), so every one of those
// invariants was unproven natively.
//
// Node oracle → test mapping (each test names its own provenance again):
//   scripts/tests/test_claim_race.mjs             → one_claimant_wins_*
//   scripts/tests/test_reservation_race.mjs       → distinct_paths_* / one_reserver_wins_*
//   scripts/tests/test_store_lock.mjs             → store_lock_*
//   scripts/tests/test_state_write_concurrency.mjs→ a_concurrent_reader_never_sees_*
//   scripts/tests/test_worktree_holds_race.mjs    → every_mirrored_hold_survives_*
//
// PROVING THE NATIVE PATH RAN (expertise/tests/patterns/
// proving-the-code-under-test-ran.md). A race against a verb that DELEGATES to
// `node bee.mjs` proves nothing about the Rust code: Node would answer
// correctly and every assertion below would pass while testing the wrong
// runtime. There is no verb-level BEE_HOOK_NO_DELEGATE equivalent, so this
// file uses the other shape from that pattern — SABOTAGE THE FALLBACK. Every
// child runs with `BEE_JS_ENTRY` pointed at a file that does not exist;
// js_fallback.rs treats a set-but-wrong BEE_JS_ENTRY as a hard error, so ANY
// delegation dies with exit 127 and the distinctive line
// "bee(rs): BEE_JS_ENTRY points to a missing file". Each race first probes its
// verb once, sequentially: a probe that delegates prints a loud SKIP naming
// the unported verb instead of a silent return, and a delegation observed
// mid-race is a hard failure (it means the race pushed a native verb off its
// native path — worth knowing, never worth passing quietly).
//
// WHY REAL PROCESSES FOR THE GREEN PATH AND THREADS FOR THE RED CONTROLS.
// The product path is always raced as N genuinely concurrent OS processes
// running the shipped `bee` binary — that is the only thing that exercises
// cross-process O_EXCL / lock-file exclusion at all. The DELIBERATE-RED
// negative controls (the falsifiability half the .mjs suites carry: a variant
// with the exclusion REMOVED must fail the same assertion, or the green result
// proves nothing) are test-owned code, so they run as N OS threads inside the
// test process, ordered by a real `std::sync::Barrier`. That is a deliberate
// choice, not a shortcut:
//   * Node needed child processes for its controls because fs writes are
//     synchronous inside one event loop, so "concurrent" calls there never
//     interleave. Rust threads are real OS threads with true parallelism, so
//     the hazard is genuinely reproduced.
//   * A `Barrier` is a DETERMINISTIC happens-before, where the .mjs suites had
//     to build an fs ready-file handshake (and, before that, lived with a
//     flaky fixed sleep — see test_claim_race.mjs's own rel1710rc-3 notes).
//     Every control here therefore bites 10/10 rather than "usually".
//   * The controls commit with plain `fs::write`, never a rename-replace, so
//     they also fire on win32 — the .mjs suites env-skip their rename-based
//     controls there (WIN32_UNGUARDED_RENAME), losing that coverage entirely.
// No control needed a `pub(crate)` widening; none of them touch src.
//
// Every wait in this file is bounded (`wait_bounded`): a race test that can
// hang is worse than no race test.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// Racers per scenario, matching the .mjs suites' own RACERS.
const RACERS: usize = 8;
/// No child may outlive this. bee's own worst case is the ~5s store-lock retry
/// budget (lock.rs MAX_ATTEMPTS * RETRY_DELAY_MS); anything past this is a
/// wedge, and a wedge must fail loudly rather than hang the suite.
const CHILD_LIMIT: Duration = Duration::from_secs(90);
/// js_fallback.rs's exit code for a set-but-missing BEE_JS_ENTRY.
const DELEGATE_EXIT: i32 = 127;

fn bee_bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("bee")
}

// ── fixture ───────────────────────────────────────────────────────────────
//
// Each race gets its OWN tempfile repo — no scenario can observe another's
// store. Mirrors test_claim_race.mjs's makeRoot + writeApprovedState: a `.bee`
// with an onboarding marker (roots.rs stops its walk-up there, so the fixture
// can never accidentally resolve onto the real checkout) and a state file in
// an execution-approved phase, which `cells claim` requires before it will
// reach its claim protocol at all.
//
// Unlike hook_contracts.rs this needs no `copy_vendored_lib`: the vendored
// `.bee/bin/lib/` gate is a HOOK concern (the write guard byte-compares the
// closure). The five verbs raced here read only the store, which is why a
// fixture without it still exercises their real paths — confirmed by the
// delegation probe each test runs before racing.

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    /// The sabotaged Node entry — an absolute path that provably does not
    /// exist, so any delegation exits 127 and names itself.
    missing_entry: PathBuf,
}

const PRISTINE_STATE: &str = r#"{
  "schema_version": "1.0",
  "phase": "swarming",
  "feature": "race-feat",
  "mode": "high-risk",
  "approved_gates": { "context": true, "shape": true, "execution": true, "review": false },
  "workers": []
}
"#;

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(dir.path()).unwrap();
    for sub in [".bee/cells", ".bee/locks", ".bee/logs", ".bee/runtime"] {
        std::fs::create_dir_all(root.join(sub)).unwrap();
    }
    std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
    std::fs::write(root.join(".bee").join("state.json"), PRISTINE_STATE).unwrap();
    let missing_entry = root.join(".bee").join("no-such-bee-entry.mjs");
    assert!(!missing_entry.exists(), "the delegation tripwire must point at a MISSING file");
    Fixture { _dir: dir, root, missing_entry }
}

/// An open cell, shaped like test_claim_race.mjs's makeCell.
fn write_open_cell(fx: &Fixture, id: &str) {
    let cell = serde_json::json!({
        "id": id,
        "feature": "race-feat",
        "title": format!("Cell {id}"),
        "lane": "tiny",
        "status": "open",
        "deps": [],
        "action": "race target",
        "verify": "true",
        "trace": {},
    });
    std::fs::write(
        fx.root.join(".bee").join("cells").join(format!("{id}.json")),
        serde_json::to_string_pretty(&cell).unwrap() + "\n",
    )
    .unwrap();
}

fn read_store(fx: &Fixture, rel: &str) -> Option<Value> {
    let text = std::fs::read_to_string(fx.root.join(rel)).ok()?;
    serde_json::from_str(&text).ok()
}

// ── running the shipped binary ────────────────────────────────────────────

#[derive(Debug)]
struct Racer {
    label: String,
    code: i32,
    stdout: String,
    stderr: String,
}

impl Racer {
    fn json(&self) -> Option<Value> {
        serde_json::from_str::<Value>(self.stdout.trim()).ok()
    }
    /// `--json` refusals print `{"error": "…"}` on stdout; infrastructure
    /// failures land on stderr. Look at both so a message assertion can never
    /// miss the channel it actually used.
    fn message(&self) -> String {
        let from_json = self
            .json()
            .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()))
            .unwrap_or_default();
        format!("{from_json}\n{}\n{}", self.stdout, self.stderr)
    }
    fn delegated(&self) -> bool {
        self.code == DELEGATE_EXIT && self.stderr.contains("BEE_JS_ENTRY points to a missing file")
    }
}

fn bee_cmd(fx: &Fixture, args: &[String]) -> Command {
    let mut cmd = Command::new(bee_bin());
    cmd.args(args)
        .current_dir(&fx.root)
        .env("BEE_JS_ENTRY", &fx.missing_entry)
        .env_remove("BEE_RS_TRACE")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

/// Bounded wait. `try_wait` + a deadline instead of a blocking `wait`, so a
/// wedged child fails the test with a named limit rather than hanging cargo
/// forever. Output is a few hundred bytes per racer, well inside the pipe
/// buffer, so polling cannot deadlock on a full pipe here.
fn wait_bounded(mut child: Child, label: &str) -> Racer {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                assert!(
                    started.elapsed() <= CHILD_LIMIT,
                    "{label}: child still running after {CHILD_LIMIT:?} — a race test must never \
                     hang; bee's own worst case is the ~5s store-lock retry budget"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("{label}: try_wait failed: {e}"),
        }
    }
    let out = child.wait_with_output().unwrap_or_else(|e| panic!("{label}: {e}"));
    Racer {
        label: label.to_string(),
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn run_bee(fx: &Fixture, args: &[&str], label: &str) -> Racer {
    let argv: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let child = bee_cmd(fx, &argv)
        .spawn()
        .unwrap_or_else(|e| panic!("{label}: failed to launch the bee binary: {e}"));
    wait_bounded(child, label)
}

/// N genuinely concurrent OS processes. Every racer is spawned from its own
/// thread and all threads rendezvous on a `Barrier` immediately before
/// `Command::spawn`, so process creation happens simultaneously rather than in
/// a staggered loop.
fn race(fx: &Fixture, argvs: Vec<Vec<String>>, label: &str) -> Vec<Racer> {
    let gate = Barrier::new(argvs.len());
    std::thread::scope(|scope| {
        let handles: Vec<_> = argvs
            .iter()
            .enumerate()
            .map(|(i, argv)| {
                let gate = &gate;
                let label = format!("{label}#{i}");
                scope.spawn(move || {
                    let mut cmd = bee_cmd(fx, argv);
                    gate.wait();
                    let child = cmd
                        .spawn()
                        .unwrap_or_else(|e| panic!("{label}: failed to launch the bee binary: {e}"));
                    wait_bounded(child, &label)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("a racer thread panicked"))
            .collect()
    })
}

/// The delegation probe. Runs the verb ONCE, sequentially, with the tripwire
/// armed. Returns false (and prints a loud SKIP naming the verb) when the verb
/// is not served natively — never a silent return.
fn native_probe(fx: &Fixture, args: &[&str], scenario: &str) -> bool {
    let out = run_bee(fx, args, "probe");
    if out.delegated() {
        eprintln!(
            "SKIP (unported: `bee {}` still delegates to `node bee.mjs` on this argv shape — \
             BEE_JS_ENTRY sabotage tripwire, exit {DELEGATE_EXIT}) — racing it would prove \
             nothing about the Rust store, so scenario \"{scenario}\" is not run",
            args.join(" ")
        );
        return false;
    }
    true
}

/// A delegation observed DURING a race is never acceptable: the probe already
/// proved the verb is native, so a 127 here means concurrency pushed it off
/// its native path and the scenario silently stopped testing Rust.
fn assert_all_native(racers: &[Racer], scenario: &str) {
    for r in racers {
        assert!(
            !r.delegated(),
            "{scenario}: racer {} fell back to Node MID-RACE (exit {DELEGATE_EXIT}) — the \
             sequential probe was native, so concurrency drove this verb off the Rust path. \
             stderr: {}",
            r.label,
            r.stderr
        );
    }
}

// ── store-lock file plumbing (mirrors lock.rs, black-box) ─────────────────
//
// sanitizeLockName: Windows-invalid + control chars -> '_', with an 8-hex
// sha256 of the ORIGINAL name appended. Duplicated here by hand on purpose —
// this is a black-box test, and a lock path it computed by calling the
// production helper could never catch that helper changing shape.

fn lock_file(root: &Path, name: &str) -> PathBuf {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\x00'..='\x1f' => '_',
            other => other,
        })
        .collect();
    let hash = format!("{:x}", Sha256::digest(name.as_bytes()));
    root.join(".bee")
        .join("locks")
        .join(format!("{sanitized}-{}.lock", &hash[..8]))
}

fn seed_lock(path: &Path, pid: u32, session: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let body = serde_json::json!({
        "pid": pid,
        "session": session,
        "ts": "2026-01-01T00:00:00.000Z",
        "token": format!("{session}-token"),
    });
    std::fs::write(path, body.to_string() + "\n").unwrap();
}

fn backdate(path: &Path, secs: u64) {
    let when = SystemTime::now() - Duration::from_secs(secs);
    filetime::set_file_mtime(path, filetime::FileTime::from_system_time(when)).unwrap();
}

/// lease-store.mjs resolveResourceFile: `path:<canonical>` -> sha256 -> a
/// per-resource file under `.bee/runtime/leases/paths/`. Only the RED control
/// needs this (it forges the pre-O_EXCL shape by hand).
fn lease_file(root: &Path, reserved: &str) -> PathBuf {
    let canonical = reserved
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_string();
    let hash = format!("{:x}", Sha256::digest(format!("path:{canonical}").as_bytes()));
    root.join(".bee")
        .join("runtime")
        .join("leases")
        .join("paths")
        .join(format!("{hash}.json"))
}

/// Windows can transiently refuse a write while another handle is closing.
/// Only the test-owned RED controls use this; the product path does its own
/// retrying inside the binary.
fn write_retry(path: &Path, text: &str) {
    for _ in 0..100 {
        if std::fs::write(path, text).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("could not write {} after 100 attempts", path.display());
}

/// The shared shape of every DELIBERATE-RED control: N threads each take a
/// snapshot, rendezvous so every read is PROVEN to happen-before every write,
/// then commit their own stale snapshot. Writes are staggered by thread index
/// so the control demonstrates a lost update rather than a filesystem sharing
/// violation — the hazard under study is last-writer-wins, not fs contention.
fn unguarded<T, S, C>(n: usize, snapshot: S, commit: C) -> Vec<T>
where
    T: Send,
    S: Fn(usize) -> T + Sync,
    C: Fn(usize, &T) + Sync,
{
    let gate = Barrier::new(n);
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..n)
            .map(|i| {
                let (gate, snapshot, commit) = (&gate, &snapshot, &commit);
                scope.spawn(move || {
                    let seen = snapshot(i);
                    gate.wait();
                    std::thread::sleep(Duration::from_millis(15 * i as u64));
                    commit(i, &seen);
                    seen
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("control thread panicked")).collect()
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// 0. the tripwire's own falsifiability
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn the_delegation_tripwire_actually_bites() {
    // Harness self-check. Every `native_probe` above is only meaningful if a
    // real delegation is actually detectable — a tripwire that never fires
    // would silently turn every race in this file into a test of `node
    // bee.mjs`. An argv shape nothing in router.rs claims must therefore come
    // back as the named 127, not as a passing command.
    let fx = fixture();
    let out = run_bee(&fx, &["definitely-not-a-bee-verb"], "tripwire-self-check");
    assert_eq!(
        out.code, DELEGATE_EXIT,
        "an unported argv shape must die on the sabotaged BEE_JS_ENTRY (exit {DELEGATE_EXIT}); \
         got exit {} — the delegation detector is broken and every race here is vacuous. \
         stdout={} stderr={}",
        out.code, out.stdout, out.stderr
    );
    assert!(
        out.delegated(),
        "the delegation must NAME itself so a SKIP line can say which verb was unported; \
         stderr was: {}",
        out.stderr
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. cells claim — exactly one winner, N-1 typed CLAIMED refusals
//    Oracle: scripts/tests/test_claim_race.mjs scenario (a).
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn one_claimant_wins_the_cell_and_every_loser_is_a_typed_claimed_refusal() {
    let fx = fixture();
    write_open_cell(&fx, "probe-cell");
    write_open_cell(&fx, "race-a");
    if !native_probe(
        &fx,
        &["cells", "claim", "--id", "probe-cell", "--worker", "probe", "--session-id", "sess-probe", "--json"],
        "cells claim race",
    ) {
        return;
    }

    let argvs: Vec<Vec<String>> = (0..RACERS)
        .map(|i| {
            ["cells", "claim", "--id", "race-a", "--worker"]
                .iter()
                .map(|s| s.to_string())
                .chain([format!("worker-{i}"), "--session-id".into(), format!("sess-{i}"), "--json".into()])
                .collect()
        })
        .collect();
    let racers = race(&fx, argvs, "claim");
    assert_all_native(&racers, "cells claim race");

    let winners: Vec<&Racer> = racers.iter().filter(|r| r.code == 0).collect();
    let losers: Vec<&Racer> = racers.iter().filter(|r| r.code != 0).collect();
    assert_eq!(
        winners.len(),
        1,
        "exactly one of {RACERS} concurrent claimants may win the cell; got {} winners: {:#?}",
        winners.len(),
        winners.iter().map(|r| &r.stdout).collect::<Vec<_>>()
    );
    assert_eq!(losers.len(), RACERS - 1, "every non-winner must be a refusal, not a crash");

    let winner = winners[0].json().expect("the winner emits the claimed cell as JSON");
    assert_eq!(winner["status"], "claimed");
    let winner_session = winner["trace"]["claim_session"]
        .as_str()
        .expect("the winning claim stamps trace.claim_session")
        .to_string();

    // The VALUE of each refusal, not merely its existence: it must be typed
    // CLAIMED, name the ACTUAL winner's session, and carry the expiry — a
    // loser has to be able to decide whether to wait or pick other work.
    for loser in &losers {
        let msg = loser.message();
        assert!(
            msg.contains("CLAIMED"),
            "a losing claimant's refusal must be typed CLAIMED; got: {msg}"
        );
        assert!(
            msg.contains(&winner_session),
            "a losing claimant's refusal must name the actual winner's session \
             \"{winner_session}\"; got: {msg}"
        );
        assert!(
            msg.contains("expires"),
            "a losing claimant's refusal must name the claim's expiry; got: {msg}"
        );
    }

    // The store agrees with the reported outcome.
    let cell = read_store(&fx, ".bee/cells/race-a.json").expect("the cell survives the race");
    assert_eq!(cell["status"], "claimed", "the settled cell must be claimed: {cell}");
    let claim = read_store(&fx, ".bee/claims/race-a.json").expect("the winner's claim file survives");
    assert_eq!(
        claim["session"], *winner_session,
        "the claims-store file must belong to the winner, not to a loser that lost the O_EXCL race"
    );
    assert!(
        !fx.root.join(".bee/claims/race-a.adopting").exists(),
        "no adoption gate file may leak once the race settles"
    );
}

#[test]
fn deliberate_red_an_unguarded_claim_lets_every_racer_believe_it_won() {
    // DELIBERATE RED (falsifiability) — oracle: test_claim_race.mjs (b).
    // The green result above is only meaningful if a claim WITHOUT the O_EXCL
    // gate would double-claim. This is the pre-fix shape: read the cell, check
    // `status == "open"`, then write "claimed" — with nothing exclusive in
    // front of it. The barrier proves every read happens-before every write,
    // so the double-claim is structural rather than timing luck.
    let fx = fixture();
    write_open_cell(&fx, "race-b");
    let cell_path = fx.root.join(".bee/cells/race-b.json");

    let saw_open = unguarded(
        RACERS,
        |_| {
            let text = std::fs::read_to_string(&cell_path).unwrap();
            let cell: Value = serde_json::from_str(&text).unwrap();
            cell["status"] == "open"
        },
        |i, saw_open| {
            if *saw_open {
                let text = std::fs::read_to_string(&cell_path).unwrap();
                let mut cell: Value = serde_json::from_str(&text).unwrap();
                cell["status"] = Value::String("claimed".into());
                cell["trace"]["worker"] = Value::String(format!("worker-unsafe-{i}"));
                write_retry(&cell_path, &cell.to_string());
            }
        },
    );

    let believed = saw_open.iter().filter(|b| **b).count();
    assert_eq!(
        believed, RACERS,
        "DETECTOR DID NOT BITE: only {believed}/{RACERS} unguarded racers saw the cell \"open\" \
         and believed they had claimed it. The barrier guarantees every read precedes every \
         write, so this control MUST show every racer double-claiming — otherwise the \
         single-winner result above proves nothing."
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. reservations reserve — no lost update on distinct paths, one winner on a
//    shared path.  Oracle: scripts/tests/test_reservation_race.mjs (a)/(b)/(c).
// ═══════════════════════════════════════════════════════════════════════════

fn reserve_argv(cell: &str, path: &str, i: usize) -> Vec<String> {
    vec![
        "reservations".into(),
        "reserve".into(),
        "--agent".into(),
        format!("worker-{i}"),
        "--cell".into(),
        cell.into(),
        "--path".into(),
        path.into(),
        "--session".into(),
        format!("sess-{i}"),
        "--json".into(),
    ]
}

#[test]
fn distinct_paths_all_survive_a_concurrent_reserve() {
    // Oracle: test_reservation_race.mjs (a). N racers, N distinct paths, one
    // store: every row must survive. This is the lost-update guard.
    let fx = fixture();
    if !native_probe(
        &fx,
        &["reservations", "reserve", "--agent", "probe", "--cell", "probe", "--path", "probe/only.ts", "--session", "sess-probe", "--json"],
        "reservations reserve, distinct paths",
    ) {
        return;
    }

    let argvs: Vec<Vec<String>> = (0..RACERS)
        .map(|i| reserve_argv("race-a", &format!("src/lib/file-{i}.ts"), i))
        .collect();
    let racers = race(&fx, argvs, "reserve-distinct");
    assert_all_native(&racers, "reservations reserve, distinct paths");

    for r in &racers {
        assert_eq!(
            r.code, 0,
            "a reserve on its OWN path can never conflict; racer {} failed: {}",
            r.label,
            r.message()
        );
    }

    let list = run_bee(&fx, &["reservations", "list", "--active-only", "--json"], "list");
    assert!(!list.delegated(), "reservations list delegated: {}", list.stderr);
    let rows = list.json().expect("reservations list emits JSON");
    let active: Vec<&Value> = rows["reservations"]
        .as_array()
        .expect("a reservations array")
        .iter()
        .filter(|r| r["cell"] == "race-a")
        .collect();
    assert_eq!(
        active.len(),
        RACERS,
        "LOST UPDATE: expected all {RACERS} distinct-path rows to survive, got {}: {:#?}",
        active.len(),
        active
    );
    let mut paths: Vec<&str> = active.iter().filter_map(|r| r["path"].as_str()).collect();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(paths.len(), RACERS, "the survivors must cover all {RACERS} distinct paths: {paths:?}");
}

#[test]
fn one_reserver_wins_a_shared_path_and_every_loser_names_the_holder() {
    // Oracle: test_reservation_race.mjs (b) — the scenario the lease store's
    // O_EXCL create is actually load-bearing for.
    let fx = fixture();
    if !native_probe(
        &fx,
        &["reservations", "reserve", "--agent", "probe", "--cell", "probe", "--path", "probe/only.ts", "--session", "sess-probe", "--json"],
        "reservations reserve, shared path",
    ) {
        return;
    }

    let argvs: Vec<Vec<String>> = (0..RACERS)
        .map(|i| reserve_argv("race-b", "src/api/router.ts", i))
        .collect();
    let racers = race(&fx, argvs, "reserve-shared");
    assert_all_native(&racers, "reservations reserve, shared path");

    let winners: Vec<&Racer> = racers.iter().filter(|r| r.code == 0).collect();
    let losers: Vec<&Racer> = racers.iter().filter(|r| r.code != 0).collect();
    assert_eq!(
        winners.len(),
        1,
        "exactly one of {RACERS} racers may hold one path; got {}: {:#?}",
        winners.len(),
        winners.iter().map(|r| &r.stdout).collect::<Vec<_>>()
    );
    assert_eq!(losers.len(), RACERS - 1);

    let winner = winners[0].json().expect("the winner emits its reservation");
    assert_eq!(winner["ok"], true);
    let winner_agent = winner["reservation"]["agent"].as_str().expect("agent").to_string();

    for loser in &losers {
        let body = loser.json().unwrap_or_else(|| panic!("a loser must emit typed JSON: {}", loser.message()));
        assert_eq!(body["ok"], false, "a loser must report ok:false: {body}");
        let conflicts = body["conflicts"].as_array().unwrap_or_else(|| {
            panic!("a loser must carry a non-empty typed conflicts array: {body}")
        });
        assert!(!conflicts.is_empty(), "conflicts must not be empty: {body}");
        assert!(
            conflicts.iter().any(|c| c["agent"] == *winner_agent),
            "a loser's conflicts must name the ACTUAL holder \"{winner_agent}\": {body}"
        );
    }

    let list = run_bee(&fx, &["reservations", "list", "--active-only", "--json"], "list");
    let rows = list.json().expect("reservations list emits JSON");
    let held = rows["reservations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["cell"] == "race-b")
        .count();
    assert_eq!(held, 1, "exactly one row may survive on the shared path, got {held}");
}

#[test]
fn deliberate_red_a_non_exclusive_lease_create_double_grants_the_same_path() {
    // DELIBERATE RED — oracle: test_reservation_race.mjs (c). Mimics the
    // PRE-O_EXCL shape: check whether the resource's lease file exists, then
    // unconditionally write it. Every racer believes it exclusively acquired
    // the identical resource — the double grant the 'wx' create exists to kill.
    let fx = fixture();
    let file = lease_file(&fx.root, "src/lib/unsafe-shared.ts");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();

    let believed = unguarded(
        RACERS,
        |_| !file.exists(),
        |i, _| {
            let body = serde_json::json!({
                "resource": "path:src/lib/unsafe-shared.ts",
                "mode": "write",
                "session_id": format!("unsafe-sess-{i}"),
                "workspace_id": format!("agent:worker-unsafe-{i}"),
                "kind": "lease",
            });
            write_retry(&file, &(body.to_string() + "\n"));
        },
    );

    let winners = believed.iter().filter(|b| **b).count();
    assert_eq!(
        winners, RACERS,
        "DETECTOR DID NOT BITE: only {winners}/{RACERS} unguarded racers believed they had \
         exclusively acquired the SAME resource. A non-atomic check-then-write MUST double-grant \
         here, or the single-winner result above proves nothing."
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. the real store lock — zero mutual-exclusion violations, exact final count
//    Oracle: scripts/tests/test_store_lock.mjs (a)/(c)/(d)/(b).
//
// The .mjs suite drove lock.mjs directly with a synthetic counter critical
// section. There is no CLI surface for that, so this drives the lock through a
// real product mutator instead: `state worker add` is a read-modify-write of
// `.bee/state.json` inside `withStoreLock(root, "state")`
// (state_group.rs worker_mutate → acquire_state_lock). Its append IS the
// counter: N racers must yield exactly N distinct entries, none lost, none
// duplicated — the same "no lost update" claim, expressed on the store the
// lock actually protects.
// ═══════════════════════════════════════════════════════════════════════════

fn worker_add_argv(i: usize) -> Vec<String> {
    vec![
        "state".into(),
        "worker".into(),
        "add".into(),
        "--nickname".into(),
        format!("w{i}"),
        "--cell".into(),
        format!("c{i}"),
        "--json".into(),
    ]
}

/// test_store_lock.mjs's assessLockedRun, ported: a racer that legitimately
/// exhausts its retry budget while another racer holds the lock is a typed
/// LOCK_BUSY refusal, not a failure. What must hold is that the store agrees
/// with the racers' own self-reported outcomes — every success survives, no
/// success is duplicated, and nothing else appears.
fn assess_locked_run(fx: &Fixture, racers: &[Racer], scenario: &str) -> usize {
    assert_all_native(racers, scenario);
    let mut expected: Vec<String> = Vec::new();
    for (i, r) in racers.iter().enumerate() {
        if r.code == 0 {
            expected.push(format!("w{i}"));
            continue;
        }
        let msg = r.message();
        assert!(
            msg.contains("busy: held by"),
            "{scenario}: racer {} neither completed nor was refused with a typed LOCK_BUSY — \
             every racer must account for itself, never vanish. exit {} :: {msg}",
            r.label,
            r.code
        );
    }
    let state = read_store(fx, ".bee/state.json").expect("state.json survives the race");
    let mut got: Vec<String> = state["workers"]
        .as_array()
        .unwrap_or_else(|| panic!("{scenario}: state.workers must stay an array: {state}"))
        .iter()
        .filter_map(|w| w["nickname"].as_str().map(|s| s.to_string()))
        .collect();
    got.sort();
    let mut want = expected.clone();
    want.sort();
    assert_eq!(
        got, want,
        "{scenario}: the store must contain EXACTLY the entries the racers reported writing — \
         a missing one is a lost update, an extra one is a phantom write"
    );
    assert!(
        !fx.root.join(".bee/state.json.tmp").exists(),
        "{scenario}: an atomic write left a temp file behind"
    );
    expected.len()
}

#[test]
fn store_lock_serializes_concurrent_state_mutators_with_no_lost_update() {
    // Oracle: test_store_lock.mjs (a).
    //
    // Process startup alone staggers racers enough that they may barely
    // overlap, which would make a green result meaningless. So the racers are
    // held at a real starting line: the `state` lock is pre-seeded as held by
    // THIS test process (a live pid, fresh mtime — never eligible for stale
    // takeover), every racer spawns and enters bee's own retry loop, and the
    // lock is then released. All N then contend within one 50ms retry tick.
    // The contention log below proves that actually happened.
    let fx = fixture();
    if !native_probe(&fx, &["state", "worker", "add", "--nickname", "probe", "--cell", "probe", "--json"], "store lock race") {
        return;
    }
    std::fs::write(fx.root.join(".bee/state.json"), PRISTINE_STATE).unwrap();

    let lock = lock_file(&fx.root, "state");
    seed_lock(&lock, std::process::id(), "starting-line-holder");
    let releaser = {
        let lock = lock.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(700));
            std::fs::remove_file(&lock).expect("the starting-line lock must be releasable");
        })
    };

    let racers = race(&fx, (0..RACERS).map(worker_add_argv).collect(), "worker-add");
    releaser.join().expect("the releaser thread panicked");
    let completed = assess_locked_run(&fx, &racers, "store lock, fresh lock");
    assert!(
        completed >= 2,
        "only {completed} racer(s) completed — with a ~5s retry budget and a 700ms starting line \
         this scenario must actually exercise the lock, not report green on a single writer"
    );

    // Vacuity guard, using the lock's own telemetry (lock.rs's
    // contention.jsonl): if the racers had not truly overlapped, every acquire
    // would show a ~0ms wait and this scenario would prove nothing about
    // exclusion. At least two racers must have queued behind another holder.
    let log = std::fs::read_to_string(fx.root.join(".bee/logs/contention.jsonl"))
        .expect("every store-lock acquire appends one contention record");
    let waited = log
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|r| r["lock_name"] == "state" && r["lock_wait_ms"].as_u64().unwrap_or(0) > 0)
        .count();
    assert!(
        waited >= 2,
        "only {waited} acquire(s) recorded any wait on the `state` lock — the racers never \
         actually contended, so a zero-violation result would be luck, not exclusion. \
         contention.jsonl:\n{log}"
    );
}

#[test]
fn store_lock_survives_a_pre_seeded_stale_lock_without_wedging_or_double_entry() {
    // Oracle: test_store_lock.mjs (c). A crashed holder's lock file must not
    // wedge the store forever — but the takeover must never let two racers in
    // at once either. Every racer starts against the SAME stale file, so
    // several attempt the takeover concurrently.
    let fx = fixture();
    if !native_probe(&fx, &["state", "worker", "add", "--nickname", "probe", "--cell", "probe", "--json"], "stale store lock race") {
        return;
    }
    std::fs::write(fx.root.join(".bee/state.json"), PRISTINE_STATE).unwrap();

    let lock = lock_file(&fx.root, "state");
    seed_lock(&lock, 999_999_999, "stale-simulated-holder");
    backdate(&lock, 45); // past lock.rs's STALE_MS (30s), owner pid provably dead

    let racers = race(&fx, (0..RACERS).map(worker_add_argv).collect(), "worker-add-stale");
    let completed = assess_locked_run(&fx, &racers, "store lock, pre-seeded stale lock");
    assert!(
        completed >= 1,
        "a stale lock must never wedge progress — zero racers completed the takeover"
    );
    assert!(
        !lock.exists(),
        "the pre-seeded stale lock was never taken over and cleared — progress can wedge \
         permanently at {}",
        lock.display()
    );
}

#[test]
fn a_live_held_store_lock_refuses_with_a_typed_busy_naming_the_holder() {
    // Oracle: test_store_lock.mjs (d)+(e). A lock held by a LIVE pid is not
    // stealable: the caller must be refused, after the real retry budget, with
    // a message naming the holder — and the holder's file must be left exactly
    // as it found it.
    let fx = fixture();
    if !native_probe(&fx, &["state", "worker", "add", "--nickname", "probe", "--cell", "probe", "--json"], "live-held store lock") {
        return;
    }

    let lock = lock_file(&fx.root, "state");
    seed_lock(&lock, std::process::id(), "live-holder-session"); // fresh mtime, alive pid
    let started = Instant::now();
    let out = run_bee(&fx, &["state", "worker", "add", "--nickname", "wx", "--cell", "cx", "--json"], "busy");
    let elapsed = started.elapsed();

    assert!(!out.delegated(), "the busy path must stay native: {}", out.stderr);
    assert_ne!(out.code, 0, "a live-held lock must refuse, never succeed: {}", out.stdout);
    let msg = out.message();
    assert!(msg.contains("busy: held by"), "the refusal must be the typed LOCK_BUSY: {msg}");
    assert!(
        msg.contains("live-holder-session"),
        "the refusal must NAME the holder so the caller can decide whether to wait: {msg}"
    );
    assert!(
        elapsed >= Duration::from_millis(2500),
        "the refusal came back in {elapsed:?} — too fast to have exercised the real ~5s retry \
         budget, so this is not the contended path"
    );
    assert!(
        elapsed < Duration::from_secs(30),
        "the refusal took {elapsed:?} — the retry budget must be bounded, never a hang"
    );
    assert!(
        lock.exists(),
        "a live holder's lock file must be left untouched by a refused caller"
    );
    let state = read_store(&fx, ".bee/state.json").expect("state.json");
    assert!(
        !state["workers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["nickname"] == "wx"),
        "a refused mutator must not have written anything: {state}"
    );
}

#[test]
fn deliberate_red_an_unlocked_state_mutator_silently_loses_updates() {
    // DELIBERATE RED — oracle: test_store_lock.mjs (b). Same read-modify-write
    // body as `state worker add`, with NO lock: every racer merges onto its own
    // stale snapshot and the last write wins, so entries vanish. Without this
    // the green results above could just mean "nothing ever races here".
    let fx = fixture();
    let state_path = fx.root.join(".bee/state.json");

    unguarded(
        RACERS,
        |_| {
            let text = std::fs::read_to_string(&state_path).unwrap();
            serde_json::from_str::<Value>(&text).unwrap()
        },
        |i, seen| {
            let mut merged = seen.clone();
            merged["workers"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({ "nickname": format!("w{i}"), "cell": format!("c{i}") }));
            write_retry(&state_path, &merged.to_string());
        },
    );

    let state: Value = serde_json::from_str(&std::fs::read_to_string(&state_path).unwrap()).unwrap();
    let survived = state["workers"].as_array().unwrap().len();
    assert!(
        survived < RACERS,
        "DETECTOR DID NOT BITE: all {survived}/{RACERS} unlocked writes survived. This control \
         must demonstrate a lost update (last-writer-wins clobbering a stale-snapshot merge), or \
         the exact-count results above prove nothing."
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. a concurrent reader never observes a half-written store
//    Oracle: scripts/tests/test_state_write_concurrency.mjs.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn a_concurrent_reader_never_sees_empty_or_truncated_state_json() {
    // The monitor mirrors the .mjs suite's setInterval reader: poll the target
    // throughout the write burst and verify every read is well-formed JSON —
    // never empty, never truncated, never a half-written fragment. A read
    // ERROR is not a violation (the .mjs monitor ignores `err` too): on
    // Windows an atomic replace can transiently deny the open, and that is the
    // OS refusing a read, not the reader observing a torn file.
    let fx = fixture();
    if !native_probe(&fx, &["state", "worker", "add", "--nickname", "probe", "--cell", "probe", "--json"], "state write concurrency") {
        return;
    }
    std::fs::write(fx.root.join(".bee/state.json"), PRISTINE_STATE).unwrap();

    let target = fx.root.join(".bee/state.json");
    let stop = Arc::new(AtomicBool::new(false));
    let reads = Arc::new(AtomicUsize::new(0));
    let violations: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let monitor = {
        let (target, stop, reads, violations) =
            (target.clone(), stop.clone(), reads.clone(), violations.clone());
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                if let Ok(text) = std::fs::read_to_string(&target) {
                    reads.fetch_add(1, Ordering::Relaxed);
                    if text.is_empty() {
                        violations.lock().unwrap().push("observed an EMPTY read".into());
                    } else if let Err(e) = serde_json::from_str::<Value>(&text) {
                        violations.lock().unwrap().push(format!(
                            "observed a non-parseable read: {e} :: raw={:?}",
                            &text[..text.len().min(200)]
                        ));
                    }
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        })
    };

    let racers = race(&fx, (0..RACERS).map(worker_add_argv).collect(), "state-writer");
    stop.store(true, Ordering::Relaxed);
    monitor.join().expect("the monitor thread panicked");
    assert_all_native(&racers, "state write concurrency");

    let seen = violations.lock().unwrap().clone();
    assert!(
        seen.is_empty(),
        "a concurrent reader observed a torn store while {RACERS} writers hammered it — the \
         write path is not atomic: {seen:#?}"
    );
    let count = reads.load(Ordering::Relaxed);
    assert!(
        count >= 20,
        "the monitor completed only {count} reads — too few to have overlapped the write burst, \
         so a clean result proves nothing"
    );
    let state = read_store(&fx, ".bee/state.json").expect("the final store is parseable JSON");
    assert!(state.is_object(), "the final store must still be a JSON object: {state}");
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. the shared cross-worktree holds ledger — every mirrored entry survives
//    Oracle: scripts/tests/test_worktree_holds_race.mjs (a)/(b).
//
// `reservations reserve` mirrors a hold into
// `<mainRoot>/.bee/runtime/cross-worktree-holds.json` under the SAME
// `cross-worktree-holds` store lock Node uses (reservations.rs's header). In an
// ordinary checkout resolveHoldTopology answers {workRoot, "main"}, so the
// mirror runs for real here. Unlike the per-path lease files this ledger is ONE
// shared file every racer read-modify-writes — the classic lost-update target.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn every_mirrored_hold_survives_a_concurrent_reserve_into_the_shared_ledger() {
    let fx = fixture();
    if !native_probe(
        &fx,
        &["reservations", "reserve", "--agent", "probe", "--cell", "probe", "--path", "probe/only.ts", "--session", "sess-probe", "--json"],
        "cross-worktree holds race",
    ) {
        return;
    }

    let argvs: Vec<Vec<String>> = (0..RACERS)
        .map(|i| reserve_argv("race-holds", &format!("src/holds/file-{i}.ts"), i))
        .collect();
    let racers = race(&fx, argvs, "mirror-hold");
    assert_all_native(&racers, "cross-worktree holds race");
    for r in &racers {
        assert_eq!(r.code, 0, "racer {} failed on its own path: {}", r.label, r.message());
    }

    let ledger = read_store(&fx, ".bee/runtime/cross-worktree-holds.json")
        .expect("the shared cross-worktree holds ledger must exist after a reserve");
    let active: Vec<&Value> = ledger["holds"]
        .as_array()
        .expect("holds must be an array")
        .iter()
        .filter(|h| h["released_at"].is_null() && h["cell"] == "race-holds")
        .collect();
    assert_eq!(
        active.len(),
        RACERS,
        "LOST UPDATE: expected all {RACERS} mirrored holds to survive the shared-ledger \
         read-modify-write, got {}: {:#?}",
        active.len(),
        ledger["holds"]
    );
    let mut paths: Vec<&str> = active.iter().filter_map(|h| h["path"].as_str()).collect();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(paths.len(), RACERS, "the surviving holds must cover all {RACERS} paths: {paths:?}");
}

#[test]
fn deliberate_red_an_unlocked_ledger_append_silently_drops_holds() {
    // DELIBERATE RED — oracle: test_worktree_holds_race.mjs (b). The pre-fix
    // shape: read the ledger, append, write, with no store lock around it.
    // Fewer than N rows survive because a later writer's read predates an
    // earlier writer's write and clobbers it on save.
    let fx = fixture();
    let ledger = fx.root.join(".bee/runtime/cross-worktree-holds.json");
    std::fs::write(&ledger, r#"{"holds":[]}"#).unwrap();

    unguarded(
        RACERS,
        |_| {
            let text = std::fs::read_to_string(&ledger).unwrap();
            serde_json::from_str::<Value>(&text).unwrap()
        },
        |i, seen| {
            let mut merged = seen.clone();
            merged["holds"].as_array_mut().unwrap().push(serde_json::json!({
                "path": format!("src/lib/unsafe-{i}.ts"),
                "holder": format!("wt-unsafe-{i}"),
                "cell": "race-unsafe",
                "released_at": Value::Null,
            }));
            write_retry(&ledger, &merged.to_string());
        },
    );

    let store: Value = serde_json::from_str(&std::fs::read_to_string(&ledger).unwrap()).unwrap();
    let survived = store["holds"].as_array().unwrap().len();
    assert!(
        survived < RACERS,
        "DETECTOR DID NOT BITE: all {survived}/{RACERS} unlocked ledger appends survived. This \
         control must demonstrate a lost update, or the all-survive result above proves nothing."
    );
}
