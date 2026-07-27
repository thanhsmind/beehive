//! decisions_lock_conformance — the rpl-4 obligations that OUTPUT DIFFING
//! STRUCTURALLY CANNOT SEE (CONTEXT.md D7c, tied to D9).
//!
//! `--cmd-check` runs one command per leg over independent clones and diffs
//! what came out. Three truths this cell owes are invisible to that:
//!
//! 1. **Race behavior.** Two runtimes appending to the SAME
//!    `.bee/decisions.jsonl` at the same time is not a thing two sequential
//!    single-process runs can exhibit. Node children drive the REAL frozen
//!    `.bee/bin/lib/decisions.mjs` while `bee_core::decisions` contends the
//!    same file from this process.
//!
//! 2. **`.bee/logs/contention.jsonl`'s shape.** `.bee/logs` is in the
//!    tree-diff EXCLUSION set (`bee-parity::differ::EXCLUDED_PREFIXES`) — the
//!    fail-open telemetry carve-out — so no parity scenario can ever compare
//!    it. It is read directly here, after a real contention, instead.
//!
//! 3. **The citation sweep WITH HITS.** `handleDecisionsSupersede` queues a
//!    capture stub per hit, and that stub embeds the FRESH event uuid inside
//!    a `dids` array and inside `outcome` prose — positions
//!    `bee_parity::normalize`'s key-gated, deny-by-default masking cannot
//!    reach. Rather than mask by pattern or exclude the capture queue from
//!    the tree diff (either of which would blind rpl-3's whole capture
//!    surface), the sweep is compared HERE against the mjs oracle directly,
//!    where the ids are known values and substitution is exact rather than
//!    heuristic. Same discipline as rpl-11: a divergence that cannot be
//!    compared is declared and moved somewhere it CAN be, never masked.
//!
//! Every store root below comes from `tempfile::tempdir()` — never the
//! repo's live `.bee/` store (both node drivers refuse a root that looks
//! like a bee checkout).

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use bee_core::capture::add_capture_stub_lists;
use bee_core::decisions::{
    log_decision, sweep_decision_citations, with_decisions_lock_sync, LogFields,
    DECISIONS_LOCK_NAME,
};
use serde_json::{json, Value};

// ─── locating the frozen mjs oracle and the drivers ────────────────────────

fn support(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support").join(name)
}

/// Resolve a frozen `.bee/bin/lib/<name>` by walking ancestors from
/// CARGO_MANIFEST_DIR — the same resolution scheme `lock_interop.rs` uses.
fn find_bin_lib(name: &str) -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        let candidate = dir.join(".bee").join("bin").join("lib").join(name);
        if candidate.exists() {
            return candidate;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "could not locate .bee/bin/lib/{name} walking ancestors from {}",
        env!("CARGO_MANIFEST_DIR")
    );
}

fn fresh_root() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join(".bee")).expect("mkdir .bee");
    fs::write(
        dir.path().join(".bee").join("onboarding.json"),
        "{\"schema_version\":\"1.0\",\"bee_version\":\"0.1.0\"}\n",
    )
    .expect("write onboarding.json");
    dir
}

fn decisions_driver(args: &[&str], session: Option<&str>) -> std::process::Output {
    let mut cmd = Command::new("node");
    cmd.arg(support("decisions_driver.mjs"));
    for a in args {
        cmd.arg(a);
    }
    if let Some(s) = session {
        cmd.env("BEE_SESSION_ID", s);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn node decisions_driver — is `node` on PATH?")
}

fn last_json_line(out: &std::process::Output, what: &str) -> Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let line = stdout
        .lines()
        .filter(|l| l.starts_with('{'))
        .next_back()
        .unwrap_or_else(|| panic!("{what}: no JSON line on stdout.\nstdout:\n{stdout}\nstderr:\n{stderr}"));
    serde_json::from_str(line)
        .unwrap_or_else(|e| panic!("{what}: non-JSON line {line:?}: {e}\nstderr:\n{stderr}"))
}

// ─── a node child HOLDING the decisions lock, via the real lock.mjs ────────

struct HoldChild {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

/// Reuses `lock_driver.mjs` (rust-port-3) rather than teaching the decisions
/// driver to hold a lock: the holder's job is to occupy `decisions` through
/// the SAME frozen `lock.mjs` both runtimes share, which that driver already
/// does exactly.
fn spawn_decisions_lock_holder(root: &Path, session: &str) -> HoldChild {
    let mut child = Command::new("node")
        .arg(support("lock_driver.mjs"))
        .arg("hold")
        .arg(find_bin_lib("lock.mjs"))
        .arg(root)
        .arg(DECISIONS_LOCK_NAME)
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
    assert_eq!(v["acquired"], json!(true), "node holder failed to take a fresh decisions lock: {v}");
    HoldChild { child, stdin, stdout }
}

impl HoldChild {
    fn release(self) {
        let HoldChild { mut child, mut stdin, mut stdout } = self;
        stdin.write_all(b"release\n").expect("write release line");
        stdin.flush().expect("flush release line");
        let mut line = String::new();
        stdout.read_line(&mut line).expect("read release ack");
        let v: Value = serde_json::from_str(line.trim())
            .unwrap_or_else(|e| panic!("hold child release ack non-JSON {line:?}: {e}"));
        assert_eq!(v["released"], json!(true), "node holder did not confirm release: {v}");
        drop(stdin);
        child.wait().expect("wait hold child");
    }
}

fn contention_records(root: &Path) -> Vec<Value> {
    bee_core::fsutil::read_jsonl(&root.join(".bee").join("logs").join("contention.jsonl"))
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. THE RACE
// ═══════════════════════════════════════════════════════════════════════════

/// Truth: a live mjs process and the Rust writer working on ONE
/// `.bee/decisions.jsonl` at the same time lose nothing and tear nothing.
///
/// # Why an mjs ARCHIVER is in the race, and not only mjs loggers
///
/// The first version of this test raced mjs appends against Rust appends and
/// PASSED with the lock removed — because it was not testing the lock. A
/// jsonl append is one small `O_APPEND` write, and the kernel already
/// serializes those; no amount of concurrent appending can tear or lose one.
/// A race that green with the code under test deleted is not a proof.
///
/// What the decisions lock actually guards is stated in `decisions.mjs:39-47`:
/// `archiveDecisions` READS the whole journal, prunes it, and RENAMES a
/// replacement over it, so "a bare appendJsonl is only safe while nobody else
/// is rewriting the file underneath it, and archive breaks that assumption."
/// An append that lands between the archiver's read and its rename is
/// silently clobbered — a genuine lost update, and one only a cross-process
/// lock can prevent. So the mjs leg here runs the REWRITER (the real frozen
/// `archiveDecisions`, which this cell deliberately does not port) while the
/// Rust leg appends, which is precisely D9's claim: mjs and Rust processes
/// hold and contend the SAME lock safely. It is the same shape
/// `packages/bee/tests/race_decisions_child.mjs` uses within mjs, made
/// cross-runtime.
///
/// Two claims, both asserted:
///
/// - **zero lost updates** — every id either runtime REPORTED as successfully
///   appended is found, exactly once, in the active file or the archive. A
///   typed `DECISIONS_LOCK_BUSY` refusal is not a lost update (it is a loud,
///   contracted no-op), so refused attempts are counted but never expected;
///   and nothing may appear in BOTH files, which is what catches a
///   half-completed archive transaction.
/// - **zero interleaved partial lines** — every line of both files parses as
///   JSON on its own, checked with a STRICT parser rather than the fail-open
///   `read_jsonl` (which skips corrupt lines silently and would turn a torn
///   write into a green).
///
/// The barrier is a `go` FILE, not a sleep: sleeps blur the start edge that
/// makes this a race at all.
#[test]
fn mixed_mjs_and_rust_writers_never_lose_or_tear_an_append() {
    const NODE_WRITERS: usize = 1;
    const RUST_WRITERS: usize = 2;
    const PER_WRITER: usize = 30;
    // The cell's floor is 50 successful appends across the race.
    const EXPECTED_FLOOR: usize = 50;
    // Seeded events are dated well before this; every event logged DURING the
    // race is stamped "now", so only the pre-seeded corpus is ever eligible
    // for archiving and "a logged id must still be live" stays unambiguous.
    const OLD_DATE: &str = "2020-01-01T00:00:00.000Z";
    const CUTOFF: &str = "2021-01-01T00:00:00.000Z";
    const SEED_COUNT: usize = 40;

    let root_dir = fresh_root();
    let root = root_dir.path().to_path_buf();
    let go_file = root.join("go");
    let stop_file = root.join("stop");
    let decisions_mjs = find_bin_lib("decisions.mjs");

    // Pre-seed a backdated corpus so the archiver always has real work to do
    // on its first passes — an archiver that only ever threw
    // NOTHING_QUALIFIES would never reach its rewrite path at all.
    let journal = root.join(".bee").join("decisions.jsonl");
    {
        let mut body = String::new();
        for i in 0..SEED_COUNT {
            body.push_str(&format!(
                "{{\"id\":\"rpl4-seed-{i:04}\",\"type\":\"decide\",\"date\":\"{OLD_DATE}\",\"decision\":\"rpl4 seed {i}\",\"rationale\":\"rpl-4 race fixture seed\",\"alternatives\":null,\"scope\":\"rpl4-race-seed\",\"source\":\"user\",\"confidence\":null}}\n"
            ));
        }
        fs::write(&journal, body).expect("seed the backdated corpus");
    }

    // The mjs REWRITER — the real frozen archiveDecisions, looping under the
    // shared lock for the whole race window.
    let archiver = Command::new("node")
        .arg(support("decisions_driver.mjs"))
        .arg("archive-until")
        .arg(&decisions_mjs)
        .arg(&root)
        .arg(CUTOFF)
        .arg(&go_file)
        .arg(&stop_file)
        .env("BEE_SESSION_ID", "mjs-archiver")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn node archiver");

    let mut children: Vec<Child> = Vec::new();
    for w in 0..NODE_WRITERS {
        let child = Command::new("node")
            .arg(support("decisions_driver.mjs"))
            .arg("log-many")
            .arg(&decisions_mjs)
            .arg(&root)
            .arg(format!("mjs-w{w}"))
            .arg(PER_WRITER.to_string())
            .arg(&go_file)
            .env("BEE_SESSION_ID", format!("mjs-writer-{w}"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn node writer");
        children.push(child);
    }

    // Rust writers start blocked on the same barrier file.
    let rust_handles: Vec<_> = (0..RUST_WRITERS)
        .map(|w| {
            let root = root.clone();
            let go_file = go_file.clone();
            std::thread::spawn(move || {
                while !go_file.exists() { /* spin — same barrier as the node side */ }
                let mut ids: Vec<String> = Vec::new();
                let mut busy = 0usize;
                for i in 0..PER_WRITER {
                    let decision = format!("rust-w{w} decide {i}");
                    let fields = LogFields {
                        decision: &decision,
                        rationale: "rpl-4 mixed-runtime lock conformance fixture",
                        alternatives: None,
                        scope: "rpl4-race",
                        source: "user",
                        confidence: None,
                        tags: None,
                    };
                    let id = bee_core::capture::random_uuid().expect("uuid");
                    let date = bee_core::lock::iso8601_millis(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as i64)
                            .unwrap_or(0),
                    );
                    match log_decision(&root, &fields, &id, &date) {
                        Ok(event) => ids.push(event["id"].as_str().expect("id").to_string()),
                        Err(message) => {
                            assert!(
                                message.starts_with("decisions store lock \"decisions\" busy:"),
                                "rust writer failed for a reason other than the typed busy refusal: {message}"
                            );
                            busy += 1;
                        }
                    }
                }
                (ids, busy)
            })
        })
        .collect();

    // Give every racer a moment to reach the barrier, then open it. This
    // sleep is a scheduling nudge only — the go FILE is the mechanism.
    std::thread::sleep(std::time::Duration::from_millis(150));
    fs::write(&go_file, "1").expect("write go file");

    let mut expected_ids: Vec<String> = Vec::new();
    let mut total_busy = 0usize;

    for (w, child) in children.into_iter().enumerate() {
        let out = child.wait_with_output().expect("wait node writer");
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        assert!(
            out.status.success(),
            "node writer {w} exited {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            out.status.code()
        );
        let summary: Value = serde_json::from_str(
            stdout
                .lines()
                .filter(|l| l.starts_with('{'))
                .next_back()
                .unwrap_or_else(|| panic!("node writer {w} printed no summary line:\n{stdout}")),
        )
        .expect("writer summary JSON");
        assert_eq!(summary["failed"], Value::Null, "node writer {w} reported a hard failure: {summary}");
        total_busy += summary["busy"].as_u64().unwrap_or(0) as usize;
        for line in stdout.lines() {
            if let Some(id) = line.strip_prefix("ID ") {
                expected_ids.push(id.trim().to_string());
            }
        }
    }

    for handle in rust_handles {
        let (ids, busy) = handle.join().expect("join rust writer");
        total_busy += busy;
        expected_ids.extend(ids);
    }

    // Every logger has finished; close the archiver's window and collect it.
    fs::write(&stop_file, "1").expect("write stop file");
    let archiver_out = archiver.wait_with_output().expect("wait node archiver");
    let archiver_stdout = String::from_utf8_lossy(&archiver_out.stdout).into_owned();
    let archiver_stderr = String::from_utf8_lossy(&archiver_out.stderr).into_owned();
    assert!(
        archiver_out.status.success(),
        "node archiver exited {:?}\nstdout:\n{archiver_stdout}\nstderr:\n{archiver_stderr}",
        archiver_out.status.code()
    );
    let archiver_summary: Value = serde_json::from_str(
        archiver_stdout
            .lines()
            .filter(|l| l.starts_with('{'))
            .next_back()
            .unwrap_or_else(|| panic!("archiver printed no summary:\n{archiver_stdout}")),
    )
    .expect("archiver summary JSON");
    assert_eq!(archiver_summary["failed"], Value::Null, "the mjs archiver failed: {archiver_summary}");
    let archived_count = archiver_summary["archived"].as_u64().unwrap_or(0) as usize;
    // The rewriter must actually have rewritten. An archiver that only ever
    // threw NOTHING_QUALIFIES never reached the read-prune-rename path, and
    // this whole race would then be proving nothing about the lock.
    assert_eq!(
        archived_count, SEED_COUNT,
        "the mjs archiver moved {archived_count} of {SEED_COUNT} seeded events — the rewrite path did not run to completion, so this race did not exercise what the lock guards"
    );

    // Every line of BOTH files must parse ON ITS OWN — the torn-line
    // assertion, deliberately NOT using the fail-open `read_jsonl` (which
    // skips corrupt lines silently and would turn a torn write into a green).
    let strict_ids = |path: &Path, label: &str| -> Vec<String> {
        let Ok(body) = fs::read_to_string(path) else { return Vec::new() };
        body.lines()
            .enumerate()
            .map(|(n, line)| {
                let parsed: Value = serde_json::from_str(line).unwrap_or_else(|e| {
                    panic!("line {} of the raced {label} is not valid JSON ({e}) — a torn/interleaved write: {line:?}", n + 1)
                });
                parsed["id"]
                    .as_str()
                    .unwrap_or_else(|| panic!("line {} of {label} carries no string id: {line}", n + 1))
                    .to_string()
            })
            .collect()
    };
    let active_ids = strict_ids(&journal, "active journal");
    let archive_ids = strict_ids(&root.join(".bee").join("decisions-archive.jsonl"), "archive");

    assert!(
        expected_ids.len() >= EXPECTED_FLOOR,
        "only {} appends actually succeeded ({total_busy} refused as busy) — below the {EXPECTED_FLOOR}-round floor this test claims to drive",
        expected_ids.len()
    );

    // ZERO LOST UPDATES: every reported append survives, exactly once,
    // somewhere. This is the assertion an unlocked Rust append fails — the
    // archiver's rename replaces the journal with the contents it read
    // BEFORE that append landed.
    let mut missing: Vec<&String> = Vec::new();
    let mut duplicated: Vec<&String> = Vec::new();
    for id in &expected_ids {
        let in_active = active_ids.iter().filter(|x| *x == id).count();
        let in_archive = archive_ids.iter().filter(|x| *x == id).count();
        match in_active + in_archive {
            0 => missing.push(id),
            1 => {}
            _ => duplicated.push(id),
        }
    }
    assert!(
        missing.is_empty(),
        "{} of {} reported appends were LOST — clobbered by the mjs archiver's read-prune-rename, which is exactly what the shared decisions lock exists to prevent. First few: {:?}",
        missing.len(),
        expected_ids.len(),
        missing.iter().take(5).collect::<Vec<_>>()
    );
    assert!(
        duplicated.is_empty(),
        "{} reported append(s) appear more than once across the two files: {:?}",
        duplicated.len(),
        duplicated.iter().take(5).collect::<Vec<_>>()
    );

    // Every id logged DURING the race is dated "now", so none is old enough
    // for this cutoff — a logged event that ended up archived would mean the
    // archiver swept a row mid-write.
    let stranded: Vec<&String> = expected_ids.iter().filter(|id| archive_ids.contains(id)).collect();
    assert!(
        stranded.is_empty(),
        "{} live-logged id(s) were swept into the archive despite being dated after the cutoff: {:?}",
        stranded.len(),
        stranded.iter().take(5).collect::<Vec<_>>()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. CONTENTION TELEMETRY SHAPE
// ═══════════════════════════════════════════════════════════════════════════

/// The nine keys `buildContentionRecord` (`lock.mjs:139`) writes, IN ORDER.
const CONTENTION_KEYS: [&str; 9] = [
    "ts",
    "lock_name",
    "lock_wait_ms",
    "holder_session",
    "caller_session",
    "workflow_id",
    "workspace_id",
    "resource",
    "result",
];

fn assert_contention_shape(who: &str, record: &Value) {
    let obj = record
        .as_object()
        .unwrap_or_else(|| panic!("{who}: contention record is not an object: {record}"));
    let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    assert_eq!(
        keys, CONTENTION_KEYS,
        "{who}: contention record key set/order drifted from lock.mjs's builder"
    );
    let ts = obj["ts"].as_str().unwrap_or_else(|| panic!("{who}: ts is not a string"));
    // The same shape `bee_parity::normalize::Shape::IsoMillisZ` gates: a
    // second- or micro-precision stamp is a real divergence, not a detail.
    assert_eq!(ts.len(), 24, "{who}: ts {ts:?} is not toISOString()'s 24-char millisecond form");
    assert!(ts.ends_with('Z') && ts.as_bytes()[10] == b'T', "{who}: ts {ts:?} is not ISO-8601-Z");
    assert!(obj["lock_wait_ms"].is_number(), "{who}: lock_wait_ms is not a number");
    assert!(obj["workflow_id"].is_null(), "{who}: workflow_id must be the reserved null");
    assert!(obj["workspace_id"].is_null(), "{who}: workspace_id must be the reserved null");
    assert!(obj["resource"].is_null(), "{who}: resource must be the reserved null");
    assert!(
        obj["caller_session"].is_null() || obj["caller_session"].is_string(),
        "{who}: caller_session must be a string or null"
    );
}

/// Truth: contending the decisions lock produces the SAME telemetry from both
/// runtimes — the same record shape, the same retry budget, and the same
/// typed refusal text.
///
/// The probe is deliberately the FULL write path, not a bare
/// `acquire_store_lock_once`: a node child holds `decisions` for the whole
/// window, so `withDecisionsLockSync`'s bounded retry exhausts and each
/// runtime writes exactly ONE record per attempt. Sixteen attempts (one
/// initial acquire plus fifteen retries, `decisions.mjs:64-65`) is the number
/// both must produce — a Rust port that retried a different number of times
/// would still emit valid records and still refuse, and only this count would
/// catch it.
///
/// `caller_session` is the one field that legitimately differs: the mjs
/// contender is a child process this test gives an explicit
/// `BEE_SESSION_ID`, while the Rust contender is this test binary, whose
/// ambient environment is not this test's to rewrite (other tests share the
/// process). It is asserted STRUCTURALLY (string-or-null) rather than by
/// value, and `holder_session` — the field that proves the record actually
/// captured the other party — is asserted to be the holder's real id on BOTH.
#[test]
fn contention_records_are_shape_identical_across_runtimes() {
    const HOLDER_SESSION: &str = "rpl4-holder-session";
    const MJS_CONTENDER_SESSION: &str = "rpl4-mjs-contender";
    // decisions.mjs:64 — 1 initial acquire + 15 retries.
    const EXPECTED_ATTEMPTS: usize = 16;

    // ── leg A: the mjs writer contends ───────────────────────────────────
    let root_a_dir = fresh_root();
    let root_a = root_a_dir.path();
    let holder_a = spawn_decisions_lock_holder(root_a, HOLDER_SESSION);
    let out = decisions_driver(
        &[
            "log-once",
            find_bin_lib("decisions.mjs").to_str().unwrap(),
            root_a.to_str().unwrap(),
            "rpl4 mjs contender",
        ],
        Some(MJS_CONTENDER_SESSION),
    );
    let mjs_result = last_json_line(&out, "mjs log-once");
    holder_a.release();

    assert_eq!(mjs_result["ok"], json!(false), "the mjs contender unexpectedly acquired a held lock: {mjs_result}");
    assert_eq!(mjs_result["code"], json!("DECISIONS_LOCK_BUSY"), "unexpected mjs refusal code: {mjs_result}");
    let mjs_message = mjs_result["message"].as_str().expect("mjs refusal message").to_string();

    // ── leg B: the Rust writer contends, through the same write path ─────
    let root_b_dir = fresh_root();
    let root_b = root_b_dir.path();
    let holder_b = spawn_decisions_lock_holder(root_b, HOLDER_SESSION);
    let fields = LogFields {
        decision: "rpl4 rust contender",
        rationale: "rpl-4 contention probe",
        alternatives: None,
        scope: "repo",
        source: "user",
        confidence: None,
        tags: None,
    };
    let rust_result = log_decision(root_b, &fields, "rpl4-rust-contender-id", "2026-07-27T00:00:00.000Z");
    holder_b.release();

    let rust_message = rust_result.expect_err("the Rust contender unexpectedly acquired a held lock");

    // ── the refusal texts ────────────────────────────────────────────────
    //
    // Two values legitimately differ: the two legs are held by two DIFFERENT
    // holder processes, so `pid` and the `since` stamp (the holder's own
    // acquisition time) cannot match. Everything else — the wording, the
    // lock name, the `session=` the record captured — is invariant and is
    // compared literally, with those two elided by SHAPE rather than
    // wildcarded away: each is asserted to have the right form first.
    let normalize_refusal = |m: &str, who: &str| -> String {
        let (head, rest) = m
            .split_once("pid=")
            .unwrap_or_else(|| panic!("{who}: no pid= in refusal: {m}"));
        let (pid, rest) = rest.split_once(' ').unwrap_or_else(|| panic!("{who}: no pid value: {m}"));
        assert!(
            !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit()),
            "{who}: pid segment {pid:?} is not a bare number"
        );
        let (mid, since) = rest
            .split_once(" since ")
            .unwrap_or_else(|| panic!("{who}: no ` since ` in refusal: {m}"));
        assert_eq!(since.len(), 24, "{who}: `since` stamp {since:?} is not toISOString()'s 24-char form");
        format!("{head}pid=<PID> {mid} since <TS>")
    };
    assert_eq!(
        normalize_refusal(&rust_message, "rust"),
        normalize_refusal(&mjs_message, "mjs"),
        "the typed busy refusal diverges between runtimes\n--- rust ---\n{rust_message}\n--- mjs ---\n{mjs_message}"
    );
    assert!(
        rust_message.contains(&format!("session={HOLDER_SESSION}")),
        "the Rust refusal did not name the real holder session: {rust_message}"
    );

    // ── the telemetry ────────────────────────────────────────────────────
    let records_a = contention_records(root_a);
    let records_b = contention_records(root_b);

    for (who, records) in [("mjs", &records_a), ("rust", &records_b)] {
        for record in records.iter() {
            assert_contention_shape(who, record);
        }
    }

    // The holder's own acquire is the first record on both legs.
    let holder_records: Vec<&Value> = records_a
        .iter()
        .chain(records_b.iter())
        .filter(|r| r["result"] == json!("acquired"))
        .collect();
    assert_eq!(holder_records.len(), 2, "expected exactly one `acquired` record per leg");

    let busy_a: Vec<&Value> = records_a.iter().filter(|r| r["result"] == json!("busy")).collect();
    let busy_b: Vec<&Value> = records_b.iter().filter(|r| r["result"] == json!("busy")).collect();

    assert_eq!(
        busy_a.len(),
        EXPECTED_ATTEMPTS,
        "the mjs writer made {} attempts, not the {EXPECTED_ATTEMPTS} decisions.mjs's retry budget declares",
        busy_a.len()
    );
    assert_eq!(
        busy_b.len(),
        busy_a.len(),
        "the Rust writer made {} lock attempts against the mjs writer's {} — the retry budgets have diverged",
        busy_b.len(),
        busy_a.len()
    );

    for (who, busy) in [("mjs", &busy_a), ("rust", &busy_b)] {
        for record in busy.iter() {
            assert_eq!(record["lock_name"], json!(DECISIONS_LOCK_NAME), "{who}: wrong lock_name");
            assert_eq!(
                record["holder_session"],
                json!(HOLDER_SESSION),
                "{who}: the busy record did not capture the real holder's session"
            );
        }
    }
}

/// Truth: the Rust `withDecisionsLockSync` releases in every exit path, so a
/// refused body leaves no lock behind and the very next acquire succeeds.
/// `StoreLockGuard` has no `Drop`, deliberately (a crashed holder's lock file
/// IS the protocol), which is exactly why the `finally` had to be spelled out
/// and is worth pinning.
#[test]
fn the_decisions_lock_is_released_even_when_the_body_fails() {
    let root_dir = fresh_root();
    let root = root_dir.path();

    let failed: Result<(), String> = with_decisions_lock_sync(root, || Err("deliberate".to_string()));
    assert_eq!(failed.unwrap_err(), "deliberate");

    let again: Result<u8, String> = with_decisions_lock_sync(root, || Ok(7));
    assert_eq!(again.expect("the lock was not released after a failing body"), 7);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. THE CITATION SWEEP, CROSS-RUNTIME
// ═══════════════════════════════════════════════════════════════════════════

/// Build a `docs/` tree exercising every branch of `sweepDecisionCitations`
/// that a port can get wrong.
fn seed_sweep_docs(root: &Path, id: &str, short8: &str) {
    let docs = root.join("docs");
    fs::create_dir_all(docs.join("nested")).expect("mkdir docs/nested");

    // Word boundaries: only lines 1, 2 and 5 cite. `\b` is ASCII-word based,
    // so a hyphen IS a boundary but an alnum neighbour is not.
    fs::write(
        docs.join("boundaries.md"),
        format!(
            "cites {id} plainly\n\
             cites {short8}-suffixed by a hyphen\n\
             embedded abc{short8}def must NOT match\n\
             prefixed x{short8} must NOT match\n\
             upper {} matches case-insensitively\n\
             unrelated line\n",
            short8.to_uppercase()
        ),
    )
    .expect("write boundaries.md");

    // Extension filter: .json and .yaml are scanned, .log and an
    // extension-less file are not, and a dotfile has no extname at all.
    fs::write(docs.join("meta.json"), format!("{{\"see\": \"{short8}\"}}\n")).expect("write meta.json");
    fs::write(docs.join("conf.yaml"), format!("ref: {id}\n")).expect("write conf.yaml");
    fs::write(docs.join("noisy.log"), format!("{id} must be invisible\n")).expect("write noisy.log");
    fs::write(docs.join("README"), format!("{id} must be invisible\n")).expect("write README");
    fs::write(docs.join(".hidden"), format!("{id} must be invisible\n")).expect("write .hidden");

    // Recursion, plus CRLF line endings, plus the >160 UTF-16-unit excerpt
    // truncation. The padding is BMP-only on purpose: a cut that splits a
    // surrogate pair is a documented, deliberately untested divergence
    // (`serde_json` cannot hold a lone surrogate, `JSON.stringify` can).
    let long = format!("{}{id} trailing", "é".repeat(200));
    fs::write(
        docs.join("nested").join("deep.txt"),
        format!("\r\n  indented {short8} with surrounding space  \r\n{long}\r\n"),
    )
    .expect("write deep.txt");
}

/// Truth: the Rust sweep and the frozen mjs `sweepDecisionCitations` return
/// the IDENTICAL structure over the identical tree — same hits, same order,
/// same 1-based line numbers, same repo-relative paths, same trimmed and
/// UTF-16-truncated excerpts.
///
/// Both runtimes are pointed at the SAME physical directory rather than two
/// copies, which is what makes the `readdirSync`/`read_dir` traversal order
/// comparable at all: directory order is a filesystem property, and two
/// copies of a tree need not enumerate alike.
///
/// `scanned_at` is the one field that cannot agree — mjs stamps it inside the
/// function with no injection seam, while the Rust port threads it in from
/// the CLI edge. It is asserted to have `toISOString()`'s exact shape on the
/// mjs side and then substituted, exactly the "declared, shape-asserted"
/// treatment `bee_parity::normalize` gives every other volatile stamp.
#[test]
fn the_citation_sweep_matches_the_frozen_mjs_oracle() {
    const ID: &str = "1178cfce-0000-4000-8000-000000000000";
    const SHORT8: &str = "1178cfce";
    const SCANNED_AT: &str = "2026-07-27T00:00:00.000Z";

    let root_dir = fresh_root();
    let root = root_dir.path();
    seed_sweep_docs(root, ID, SHORT8);

    let out = decisions_driver(
        &[
            "sweep",
            find_bin_lib("decisions.mjs").to_str().unwrap(),
            root.to_str().unwrap(),
            ID,
            SHORT8,
        ],
        None,
    );
    let mut mjs = last_json_line(&out, "mjs sweep");

    let mjs_scanned_at = mjs["scanned_at"].as_str().expect("mjs scanned_at").to_string();
    assert_eq!(
        mjs_scanned_at.len(),
        24,
        "mjs scanned_at {mjs_scanned_at:?} is not toISOString()'s 24-char form"
    );
    mjs["scanned_at"] = json!(SCANNED_AT);

    let rust = sweep_decision_citations(root, ID, SHORT8, SCANNED_AT);

    assert_eq!(
        rust, mjs,
        "the Rust citation sweep diverges from the frozen mjs oracle\n--- rust ---\n{rust:#}\n--- mjs ---\n{mjs:#}"
    );

    // The fixture must actually EXERCISE the thing — a sweep that found
    // nothing would compare two empty structures and prove nothing.
    let hits = rust["files"].as_array().expect("files array");
    assert!(hits.len() >= 6, "the seeded docs tree produced only {} hits — too few to discriminate", hits.len());
    // Directory ORDER, pinned legibly and independently of the oracle dump:
    // libuv byte-sorts every `readdirSync`, so `boundaries.md` leads and
    // `nested/` (which sorts after `meta.json`) trails. Raw `read_dir` order
    // put `conf.yaml` first on this machine — see `collect_sweep_files`.
    let files_in_order: Vec<&str> = hits.iter().filter_map(|h| h["file"].as_str()).collect();
    assert_eq!(files_in_order.first(), Some(&"docs/boundaries.md"), "hits are not in byte-sorted directory order");
    assert_eq!(files_in_order.last(), Some(&"docs/nested/deep.txt"), "hits are not in byte-sorted directory order");
    assert_eq!(rust["hit_count"], json!(hits.len()));
    assert!(
        hits.iter().any(|h| h["excerpt"].as_str().is_some_and(|e| e.ends_with("..."))),
        "no truncated excerpt in the result — the >160-unit branch never ran"
    );
    assert!(
        !hits.iter().any(|h| {
            let f = h["file"].as_str().unwrap_or("");
            f.ends_with(".log") || f.ends_with("README") || f.ends_with(".hidden")
        }),
        "the extension filter let a non-text-extension file through"
    );
}

/// Truth: the capture stub `handleDecisionsSupersede` queues per hit is
/// byte-identical to the one the frozen mjs handler queues — including
/// `normalizeList`'s ARRAY branch, which rpl-3 deliberately did not port
/// because no CLI flag can produce that shape.
///
/// The comma-bearing path is the discriminating case: the array branch keeps
/// it as ONE entry, while the string branch rpl-3 ported would split it in
/// two. A port that reused the string branch by joining the array passes
/// every other fixture and fails only here.
#[test]
fn the_sweep_capture_stub_matches_the_frozen_mjs_oracle() {
    const OUTCOME: &str =
        "docs/a,b.md:12 still cites superseded decision old-id — reconcile against replacement new-id.";
    const AT: &str = "2026-07-27T00:00:01.000Z";
    let dids = ["old-id".to_string(), "new-id".to_string()];
    let files = ["docs/a,b.md".to_string()];

    let mjs_root_dir = fresh_root();
    let out = decisions_driver(
        &[
            "capture-stub",
            find_bin_lib("capture.mjs").to_str().unwrap(),
            mjs_root_dir.path().to_str().unwrap(),
            OUTCOME,
            &serde_json::to_string(&dids).unwrap(),
            &serde_json::to_string(&files).unwrap(),
            "supersede-sweep",
        ],
        None,
    );
    let mut mjs = last_json_line(&out, "mjs capture-stub");

    let rust_root_dir = fresh_root();
    let rust = add_capture_stub_lists(
        rust_root_dir.path(),
        OUTCOME,
        &dids,
        &files,
        Some("supersede-sweep"),
        "rpl4-stub-id",
        AT,
    )
    .expect("rust stub");

    // Both are generated per call and have no injection seam on the mjs side.
    mjs["id"] = json!("rpl4-stub-id");
    mjs["at"] = json!(AT);

    assert_eq!(
        rust, mjs,
        "the Rust sweep capture stub diverges from the frozen mjs oracle\n--- rust ---\n{rust:#}\n--- mjs ---\n{mjs:#}"
    );
    // The whole point: the comma-bearing path survived as ONE entry.
    assert_eq!(rust["files"], json!(["docs/a,b.md"]));

    // Key ORDER, not merely key set — `JSON.stringify` emits insertion order
    // and `serde_json`'s `preserve_order` is what reproduces it.
    let rust_keys: Vec<&str> = rust.as_object().unwrap().keys().map(String::as_str).collect();
    let mjs_keys: Vec<&str> = mjs.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(rust_keys, mjs_keys, "stub key ORDER diverged");
}

/// Truth: the three written EVENTS are byte-identical to the frozen mjs
/// module's, key order included — the claim `--cmd-check` proves through the
/// tree diff, pinned here directly against the oracle so a harness change can
/// never quietly retire it.
#[test]
fn a_logged_event_matches_the_frozen_mjs_oracle_byte_for_byte() {
    let mjs_root_dir = fresh_root();
    let mjs_root = mjs_root_dir.path();
    let out = decisions_driver(
        &[
            "log-once",
            find_bin_lib("decisions.mjs").to_str().unwrap(),
            mjs_root.to_str().unwrap(),
            "rpl4 oracle event",
        ],
        None,
    );
    let result = last_json_line(&out, "mjs log-once");
    assert_eq!(result["ok"], json!(true), "the mjs oracle refused a clean log: {result}");

    let mjs_line = fs::read_to_string(mjs_root.join(".bee").join("decisions.jsonl"))
        .expect("read mjs journal")
        .lines()
        .next_back()
        .expect("one mjs row")
        .to_string();

    let rust_root_dir = fresh_root();
    let rust_root = rust_root_dir.path();
    let fields = LogFields {
        decision: "rpl4 oracle event",
        rationale: "rpl-4 contention probe",
        alternatives: None,
        scope: "repo",
        source: "user",
        confidence: None,
        tags: None,
    };
    log_decision(rust_root, &fields, "rpl4-id", "2026-07-27T00:00:02.000Z").expect("rust log");
    let rust_line = fs::read_to_string(rust_root.join(".bee").join("decisions.jsonl"))
        .expect("read rust journal")
        .lines()
        .next_back()
        .expect("one rust row")
        .to_string();

    // Substitute the two values that have no injection seam on the mjs side,
    // by KEY, then compare the raw LINES — a structural comparison would
    // forgive exactly the key-order drift this is here to catch.
    let mjs_parsed: Value = serde_json::from_str(&mjs_line).expect("mjs row parses");
    let expected = mjs_line
        .replace(mjs_parsed["id"].as_str().expect("mjs id"), "rpl4-id")
        .replace(mjs_parsed["date"].as_str().expect("mjs date"), "2026-07-27T00:00:02.000Z");

    assert_eq!(
        rust_line, expected,
        "the appended decide LINE diverges from the frozen mjs oracle (key order or value spelling)"
    );
}
