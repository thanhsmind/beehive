//! decisions_composed_oracle — the `decisions` two-runtime oracle: the
//! comparisons `--cmd-check` structurally cannot make. It opened closing the
//! COMPOSED-HANDLER GAP the rpl-4 goal-check filed, and rpl-5's own
//! goal-check added a second kind at the bottom of the file: a DECLARED
//! divergence, where the two runtimes must NOT produce the same bytes and
//! `--cmd-check`'s zero-diff contract therefore has nothing to say.
//!
//! # What was missing, exactly
//!
//! rpl-4 proved `sweepDecisionCitations` and `addCaptureStub` separately
//! against the frozen mjs. What it never EXECUTED is the handler that
//! composes them: `queen_bee::ledger::decisions::handle_supersede`'s per-hit
//! capture-stub loop, and the `Propagation sweep: N citation(s) …` header
//! plus the two-space-indented `  {file}:{line}  {excerpt}` hit lines
//! underneath it. A reviewer read that code against `bee.mjs:1975-1992` and
//! found it matching — but a reading is not an assertion, and the rpl-4
//! `--cmd-check` scenarios that DO run `supersede` both hit ZERO citations,
//! so the entire hits branch had no executed oracle at all.
//!
//! # Why it cannot go through `--cmd-check`
//!
//! The fresh event uuid lands in two positions
//! `bee_parity::normalize`'s masking structurally cannot reach: as an ARRAY
//! ELEMENT inside each capture stub's `dids`, and inside the stub's
//! `outcome` PROSE. That masking is key-gated and deny-by-default on
//! purpose — widening it to pattern masking would blind rpl-3's whole
//! capture surface, which is exactly the store this test compares. So the
//! comparison happens here instead, where each leg's own uuids are KNOWN
//! values read back out of its own store and substituted exactly, rather
//! than guessed at by a pattern. Same discipline as rpl-11: a divergence
//! that cannot be compared where it is gets moved somewhere it CAN be,
//! never masked into silence.
//!
//! # Why both legs run as real processes
//!
//! The obligation is the COMPOSED handler, so neither leg may be
//! reconstructed from parts: the mjs leg is the frozen `.bee/bin/bee.mjs`
//! (D1) driven by argv, and the Rust leg is the real `queen-bee` binary
//! driven by the same argv. Every store root is a `tempfile::tempdir()` —
//! never the repo's live `.bee/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// The decision id the docs tree cites. Deliberately a uuid whose first
/// eight characters (`1178cfce`) are also a valid short8, so BOTH needles
/// `sweepDecisionCitations` builds are exercised by the same fixture.
const TARGET_ID: &str = "1178cfce-aaaa-4bbb-8ccc-000000000001";

const EVENT_ID_PLACEHOLDER: &str = "<REPLACEMENT-ID>";
const TS_PLACEHOLDER: &str = "<TS>";

/// Walk up from this crate's manifest dir to the checkout that owns the
/// frozen mjs entry point — the same resolution scheme
/// `bee-core/tests/decisions_lock_conformance.rs` uses for `bin/lib`.
fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..12 {
        if dir.join(".bee").join("bin").join("bee.mjs").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!(
        "could not locate .bee/bin/bee.mjs walking ancestors from {}",
        env!("CARGO_MANIFEST_DIR")
    );
}

/// A store root both runtimes can dispatch against, plus a `docs/**` tree
/// with real citations.
///
/// `queen-bee` refuses to dispatch without a command-registry snapshot whose
/// `source_sha256` still matches the live `command-registry.mjs`, so both are
/// copied in from the checkout. Nothing else is seeded: the point is to keep
/// the fixture small enough that every byte either leg prints is accounted
/// for by this file.
fn seed_root(repo: &Path) -> tempfile::TempDir {
    seed_root_with(repo, TARGET_ID, &[
        // THREE hits across TWO files, and the ordering is load-bearing
        // twice over: `collectSweepFiles` walks `docs/**` in byte-sorted
        // readdir order (`decisions/` before `history/` — the divergence
        // rpl-4 found between Node's `readdirSync` and Rust's `read_dir`),
        // and within a file the hits come out in line order. A single-hit
        // fixture would prove neither. Line 2 of `notes.md` cites the
        // SHORT8 and line 4 the FULL id, so both needles fire.
        (
            "docs/history/notes.md",
            "# Notes\nThis line cites decision 1178cfce in prose.\n\nAnd the full id 1178cfce-aaaa-4bbb-8ccc-000000000001 appears here too.\n",
        ),
        ("docs/decisions/other.md", "alpha\nsee 1178cfce for the rationale\n"),
    ])
}

fn seed_root_with(repo: &Path, target_id: &str, docs: &[(&str, &str)]) -> tempfile::TempDir {
    let escaped = serde_json::to_string(target_id).expect("the target id is a JSON string");
    seed_root_journal(
        repo,
        &format!(
            "{{\"id\":{escaped},\"type\":\"decide\",\"date\":\"2026-07-26T00:00:00.000Z\",\"decision\":\"oracle target\",\"rationale\":\"oracle seed\",\"alternatives\":null,\"scope\":\"repo\",\"source\":\"fixture\",\"confidence\":0}}\n"
        ),
        docs,
    )
}

/// The same root, with `.bee/decisions.jsonl` supplied verbatim — the shape
/// the cutoff cases below need, where the JOURNAL is the fixture and the
/// docs tree is irrelevant.
fn seed_root_journal(repo: &Path, journal: &str, docs: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join(".bee/cache")).expect("mkdir .bee/cache");
    fs::create_dir_all(root.join(".bee/bin/lib")).expect("mkdir .bee/bin/lib");
    fs::create_dir_all(root.join("docs/history")).expect("mkdir docs/history");
    fs::create_dir_all(root.join("docs/decisions")).expect("mkdir docs/decisions");

    fs::write(
        root.join(".bee/onboarding.json"),
        "{\"schema_version\":\"1.0\",\"bee_version\":\"0.1.0\"}\n",
    )
    .expect("write onboarding.json");
    for rel in [".bee/cache/command-registry.json", ".bee/bin/lib/command-registry.mjs"] {
        fs::copy(repo.join(rel), root.join(rel)).unwrap_or_else(|e| panic!("copy {rel}: {e}"));
    }
    fs::write(root.join(".bee/decisions.jsonl"), journal).expect("write decisions.jsonl");

    for (rel, body) in docs {
        fs::write(root.join(rel), body).unwrap_or_else(|e| panic!("write {rel}: {e}"));
    }
    dir
}

struct Leg {
    stdout: String,
    exit_code: i32,
    /// The `.bee/capture-queue.jsonl` this leg wrote, verbatim.
    capture_queue: String,
    /// The supersede event this leg appended (the LAST journal line).
    event: Value,
}

fn run_leg(command: Command, root: &Path) -> Leg {
    run_leg_for(command, root, TARGET_ID)
}

fn run_leg_for(mut command: Command, root: &Path, target_id: &str) -> Leg {
    let output = command
        .args([
            "decisions",
            "supersede",
            "--id",
            target_id,
            "--decision",
            "oracle replacement",
            "--rationale",
            "oracle rationale",
        ])
        // Both runtimes read `resolveSessionId`'s env chain; an ambient
        // value must never reach either leg (the same coverage-control
        // reasoning as `bee_parity::runner::run_argv`).
        .env_remove("BEE_SESSION_ID")
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .current_dir(root)
        .output()
        .expect("failed to spawn a supersede leg");

    let journal = fs::read_to_string(root.join(".bee/decisions.jsonl")).expect("read decisions.jsonl");
    let last = journal
        .lines()
        .filter(|l| !l.trim().is_empty())
        .next_back()
        .expect("the journal has at least the appended event");
    Leg {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        capture_queue: fs::read_to_string(root.join(".bee/capture-queue.jsonl")).unwrap_or_default(),
        event: serde_json::from_str(last).expect("the appended event parses"),
    }
}

fn field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("expected a string `{key}` on {value}"))
}

/// `new Date().toISOString()`'s exact shape, asserted rather than assumed —
/// the same `IsoMillisZ` gate `bee_parity::normalize` applies. Collapsing a
/// stamp to a placeholder without checking its shape would let a
/// second-precision or micro-precision port slip through this file.
fn assert_iso_millis_z(what: &str, value: &str) {
    let b = value.as_bytes();
    let ok = b.len() == 24
        && b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b':'
        && b[19] == b'.'
        && b[23] == b'Z'
        && [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18, 20, 21, 22]
            .iter()
            .all(|&i| b[i].is_ascii_digit());
    assert!(ok, "{what} is not a toISOString()-shaped stamp: {value:?}");
}

/// Replace this leg's OWN uncontrollable values — read back out of its own
/// store, never pattern-matched — with fixed placeholders.
///
/// This is substitution of KNOWN VALUES, which is why it is safe where
/// pattern masking is not: each replaced string is whatever THIS leg
/// actually wrote, so a port that emitted a uuid in the wrong PLACE, or a
/// different NUMBER of them, still fails — only the value is neutralized,
/// never its position.
///
/// **Every timestamp collapses to ONE placeholder, deliberately.** The first
/// version of this used a per-stub `<STUB-n-AT>`, and it failed for a reason
/// worth recording: the three `addCaptureStub` calls each read the wall
/// clock, so whether two consecutive stubs land in the SAME millisecond is
/// pure scheduling luck — mjs happened to give all three one stamp while
/// queen-bee split the third, and a global per-value replace then collapsed
/// the shared stamps into the earliest placeholder on one leg only. That is
/// a property of the machine, not of either implementation, and pinning it
/// would have made this test flaky in both directions. What the stamps DO
/// have to satisfy — `toISOString()`'s exact precision — is asserted above
/// instead, on every one of them, on both legs.
fn substitute(leg: &Leg, text: &str) -> String {
    let mut out = text.replace(field(&leg.event, "id"), EVENT_ID_PLACEHOLDER);

    let mut stamps: Vec<String> = vec![field(&leg.event, "date").to_string()];
    assert_iso_millis_z("event.date", &stamps[0]);
    let scanned_at = leg
        .event
        .get("sweep")
        .and_then(|s| s.get("scanned_at"))
        .and_then(Value::as_str)
        .expect("the event carries sweep.scanned_at");
    assert_iso_millis_z("sweep.scanned_at", scanned_at);
    stamps.push(scanned_at.to_string());

    for (i, line) in leg.capture_queue.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let row: Value = serde_json::from_str(line).expect("a capture-queue row parses");
        // Stub ids ARE distinct uuids and stay individually identified, so
        // a port that reused one id across three stubs still fails here.
        out = out.replace(field(&row, "id"), &format!("<STUB-{i}-ID>"));
        assert_iso_millis_z("stub.at", field(&row, "at"));
        stamps.push(field(&row, "at").to_string());
    }

    for stamp in stamps {
        out = out.replace(&stamp, TS_PLACEHOLDER);
    }
    out
}

/// The composed handler, both legs, one argv, compared byte for byte after
/// exact substitution — stdout AND the capture-queue rows it queued.
#[test]
fn the_composed_sweep_with_hits_handler_matches_the_frozen_mjs() {
    let repo = repo_root();
    let mjs_root = seed_root(&repo);
    let rust_root = seed_root(&repo);

    let mut node = Command::new("node");
    node.arg(repo.join(".bee").join("bin").join("bee.mjs"));
    let mjs = run_leg(node, mjs_root.path());
    let rust = run_leg(Command::new(env!("CARGO_BIN_EXE_queen-bee")), rust_root.path());

    assert_eq!(mjs.exit_code, 0, "mjs leg failed:\n{}", mjs.stdout);
    assert_eq!(rust.exit_code, 0, "queen-bee leg failed:\n{}", rust.stdout);

    // The fixture has to actually REACH the hits branch — a zero-hit sweep
    // would make every assertion below pass while proving nothing, which is
    // precisely the hole rpl-4 left.
    assert_eq!(
        mjs.event.get("sweep").and_then(|s| s.get("hit_count")).and_then(Value::as_u64),
        Some(3),
        "the docs fixture must produce exactly 3 citation hits on the mjs leg, or this test is vacuous"
    );
    assert_eq!(
        mjs.capture_queue.lines().filter(|l| !l.trim().is_empty()).count(),
        3,
        "one capture stub per hit"
    );

    let mjs_stdout = substitute(&mjs, &mjs.stdout);
    let rust_stdout = substitute(&rust, &rust.stdout);
    assert_eq!(
        mjs_stdout, rust_stdout,
        "the composed handler's stdout diverged\n--- mjs ---\n{mjs_stdout}\n--- queen-bee ---\n{rust_stdout}"
    );

    // Spelled out rather than only compared, so the header wording, the
    // two-space indent and the double-space between `line` and `excerpt`
    // are pinned even if BOTH legs were to drift together.
    assert_eq!(
        mjs_stdout,
        format!(
            "Superseded {TARGET_ID} with {EVENT_ID_PLACEHOLDER}.\n\
             Propagation sweep: 3 citation(s) found under docs/** — a capture stub was queued for each.\n\
             \x20 docs/decisions/other.md:2  see 1178cfce for the rationale\n\
             \x20 docs/history/notes.md:2  This line cites decision 1178cfce in prose.\n\
             \x20 docs/history/notes.md:4  And the full id {TARGET_ID} appears here too.\n"
        )
    );

    // The queued rows — the half `--cmd-check` cannot reach at all. Compared
    // as RAW LINES, never re-parsed and re-serialized: a structural compare
    // would forgive exactly the key-order drift this exists to catch.
    let mjs_queue = substitute(&mjs, &mjs.capture_queue);
    let rust_queue = substitute(&rust, &rust.capture_queue);
    assert_eq!(
        mjs_queue, rust_queue,
        "the queued capture stubs diverged\n--- mjs ---\n{mjs_queue}\n--- queen-bee ---\n{rust_queue}"
    );

    // And the `dids` array element / `outcome` prose positions by name, so
    // the reason this test exists is asserted and not merely implied.
    assert!(
        mjs_queue.contains(&format!(
            "\"outcome\":\"docs/decisions/other.md:2 still cites superseded decision {TARGET_ID} — reconcile against replacement {EVENT_ID_PLACEHOLDER}.\""
        )),
        "the replacement id must appear inside the stub's outcome prose: {mjs_queue}"
    );
    assert!(
        mjs_queue.contains(&format!("\"dids\":[\"{TARGET_ID}\",\"{EVENT_ID_PLACEHOLDER}\"]")),
        "the replacement id must appear as a dids array element: {mjs_queue}"
    );
}

/// rpl-5, the UTF-16 INCONSISTENCY the rpl-4 goal-check filed.
///
/// `supersedeDecision` derives its short8 needle as `targetId.slice(0, 8)`
/// (`decisions.mjs:465`) — eight UTF-16 CODE UNITS. `bee_core` computed it
/// as `target_id.chars().take(8)` — eight SCALAR VALUES. The two agree on
/// every uuid, which is why nothing caught it; they disagree the moment an
/// astral character sits inside the first eight units, and `supersede`
/// accepts an ARBITRARY id (the parity fixture's own `fixture-decision-…`
/// ids are not uuids either), so the divergent input is reachable rather
/// than hypothetical.
///
/// The fixture makes the two answers mutually exclusive rather than merely
/// different. For this id the mjs needle is `ab🐝🐝🐝` (2 ASCII units + 3
/// astral characters at 2 units each = 8) while the scalar reading is
/// `ab🐝🐝🐝cd-`. The docs line contains the first and NOT the second, so
/// the scalar reading finds ZERO citations where mjs finds one — the sweep
/// silently misses a real citation, and the capture stub that should have
/// been queued never is.
///
/// Same file already got this right in `js_excerpt`, which is why this is an
/// inconsistency rather than an unknown — and why the fix extracts the
/// shared `js_slice_prefix` helper instead of adding a second spelling.
#[test]
fn short8_is_a_utf16_slice_not_a_scalar_take() {
    const ASTRAL_ID: &str = "ab\u{1F41D}\u{1F41D}\u{1F41D}cd-4aaa-8bbb-000000000001";
    const DOCS: &[(&str, &str)] = &[(
        "docs/history/notes.md",
        "# Notes\nsee ab\u{1F41D}\u{1F41D}\u{1F41D}x here\n",
    )];

    let repo = repo_root();
    let mjs_root = seed_root_with(&repo, ASTRAL_ID, DOCS);
    let rust_root = seed_root_with(&repo, ASTRAL_ID, DOCS);

    let mut node = Command::new("node");
    node.arg(repo.join(".bee").join("bin").join("bee.mjs"));
    let mjs = run_leg_for(node, mjs_root.path(), ASTRAL_ID);
    let rust = run_leg_for(Command::new(env!("CARGO_BIN_EXE_queen-bee")), rust_root.path(), ASTRAL_ID);

    assert_eq!(mjs.exit_code, 0, "mjs leg failed:\n{}", mjs.stdout);
    assert_eq!(rust.exit_code, 0, "queen-bee leg failed:\n{}", rust.stdout);

    // The ORACLE side of the pin: mjs is what decides there is a citation
    // here at all. If this ever stops being 1 the fixture has drifted and
    // the comparison below would be vacuous.
    let hits = |leg: &Leg| {
        leg.event
            .get("sweep")
            .and_then(|s| s.get("hit_count"))
            .and_then(Value::as_u64)
            .expect("the event carries sweep.hit_count")
    };
    assert_eq!(hits(&mjs), 1, "the frozen mjs must find exactly one citation for the UTF-16 needle");
    assert_eq!(
        hits(&rust),
        1,
        "queen-bee found {} citation(s) where mjs found 1 — the short8 needle disagrees with `slice(0, 8)`",
        hits(&rust)
    );

    let mjs_stdout = substitute(&mjs, &mjs.stdout);
    let rust_stdout = substitute(&rust, &rust.stdout);
    assert_eq!(mjs_stdout, rust_stdout, "the astral-id sweep diverged");
    assert_eq!(
        mjs_stdout,
        format!(
            "Superseded {ASTRAL_ID} with {EVENT_ID_PLACEHOLDER}.\n\
             Propagation sweep: 1 citation(s) found under docs/** — a capture stub was queued for each.\n\
             \x20 docs/history/notes.md:2  see ab\u{1F41D}\u{1F41D}\u{1F41D}x here\n"
        )
    );
    assert_eq!(
        substitute(&mjs, &mjs.capture_queue),
        substitute(&rust, &rust.capture_queue),
        "the astral-id capture stub diverged"
    );
}

// ── the `--before` / `--since` cutoff, and why it is pinned HERE ────────────
//
// rpl-5's goal-check REPRODUCED a silent divergence: on a host at
// `TZ=Asia/Bangkok`, `decisions archive --before 2026-07-26T00:00:00 --json`
// made mjs archive one row and queen-bee archive two, BOTH exiting 0 with no
// warning, because `date_parse_ms` read the offset-less date-time as UTC
// while V8 reads it as LOCAL. `archive` deletes rows from the active store
// and has no inverse verb, so the port was quietly destroying an extra row —
// and on a negative-offset host the same error inverts into keeping a row
// mjs would have removed.
//
// The fix is fail-closed, not local-time emulation: `bee_core::jsdate`
// refuses an offset-less date-time outright, so the port now stops where it
// used to guess. See that module's DECLARED DIVERGENCES note for why
// reproducing ECMA-262 local time is not on the table (no local-offset
// source in `std`; guessing one trades a detectable divergence for an
// undetectable one).
//
// That refusal is a DELIBERATE, declared divergence, which is exactly why it
// cannot live in `--cmd-check`: that harness's whole contract is "these two
// runtimes produced the same bytes", and here they must not. Same reasoning
// as the composed-handler tests above — a comparison that cannot be made
// where it is gets moved somewhere it CAN be, never masked into silence. So
// each case below asserts BOTH halves: the exact V8 reading on the mjs leg
// (which is what makes the divergence real rather than theoretical) and the
// port's loud refusal over a byte-untouched store.

const CUTOFF_ROW_1: &str = "aaaaaaaa-1111-4aaa-8bbb-000000000001";
const CUTOFF_ROW_2: &str = "bbbbbbbb-2222-4aaa-8bbb-000000000002";
const CUTOFF_ROW_3: &str = "cccccccc-3333-4aaa-8bbb-000000000003";

/// The judge's own three-row clone. The dates are chosen so the UTC and the
/// `+07:00` readings of `2026-07-26T00:00:00` fall on OPPOSITE sides of row
/// 3 — that row is the whole experiment:
///
/// * row 1 `2026-07-25T00:00:00Z` — older than either reading
/// * row 2 `2026-07-26T12:00:00Z` — newer than either reading
/// * row 3 `2026-07-25T20:00:00Z` — newer than the local reading
///   (`2026-07-25T17:00:00Z`), older than the UTC one (`2026-07-26T00:00:00Z`)
fn cutoff_journal() -> String {
    [
        (CUTOFF_ROW_1, "2026-07-25T00:00:00.000Z", "rpl5 cutoff row one"),
        (CUTOFF_ROW_2, "2026-07-26T12:00:00.000Z", "rpl5 cutoff row two"),
        (CUTOFF_ROW_3, "2026-07-25T20:00:00.000Z", "rpl5 cutoff row three"),
    ]
    .iter()
    .map(|(id, date, decision)| {
        format!(
            "{{\"id\":\"{id}\",\"type\":\"decide\",\"date\":\"{date}\",\"decision\":\"{decision}\",\"rationale\":\"rpl5 cutoff seed\",\"alternatives\":null,\"scope\":\"rpl5-cutoff\",\"source\":\"fixture\",\"confidence\":0}}\n"
        )
    })
    .collect()
}

struct Run {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

/// One leg, one argv, at a FIXED non-zero local offset. `TZ` is pinned so the
/// V8 reading below is a constant rather than a property of whoever runs the
/// suite — a UTC test host would make every assertion here vacuously true,
/// which is precisely how the bug survived a green suite the first time.
fn run_cutoff_leg(mut command: Command, root: &Path, argv: &[&str]) -> Run {
    let output = command
        .args(argv)
        .env("TZ", "Asia/Bangkok")
        .env_remove("BEE_SESSION_ID")
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .current_dir(root)
        .output()
        .expect("failed to spawn a cutoff leg");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    }
}

/// mjs writes exactly one `[bee] <cmd> <n>ms` work-visibility line to stderr
/// on every direct run (`bee.mjs:7298`) and queen-bee writes none, so the two
/// legs' stderr is only comparable with it removed. Removed the way
/// `bee_parity::normalize::strip_runtime_stderr_artifacts` removes it —
/// SHAPE-CHECKED and counted, never "drop anything starting with `[bee]`",
/// so a real diagnostic that happened to use the prefix cannot vanish here.
fn strip_mjs_timing_line(stderr: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut stripped = 0usize;
    for line in stderr.lines() {
        if let Some(rest) = line.strip_prefix("[bee] ") {
            let digits = rest.strip_suffix("ms").and_then(|r| r.rsplit(' ').next().map(str::to_string));
            assert!(
                digits.is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit())),
                "not a well-shaped mjs timing line: {line:?}"
            );
            stripped += 1;
            continue;
        }
        kept.push(line);
    }
    assert_eq!(stripped, 1, "mjs stderr must carry exactly one `[bee] …` timing line, got {stderr:?}");
    if kept.is_empty() {
        String::new()
    } else {
        format!("{}\n", kept.join("\n"))
    }
}

struct CutoffCase {
    name: &'static str,
    argv: &'static [&'static str],
    /// The port's refusal, byte for byte, on stderr.
    rust_stderr: &'static str,
    mjs_exit: i32,
    mjs_stdout: &'static str,
    mjs_stderr: &'static str,
    /// The ids left in `.bee/decisions.jsonl` on the MJS leg afterwards, or
    /// `None` when the verb is read-only / refused and must not touch it.
    mjs_journal_ids_after: Option<&'static [&'static str]>,
}

/// The three forms the goal-check measured, each fail-closed in the port and
/// each FINITE in V8 — verified against the real `node` here, not assumed.
#[test]
fn offsetless_and_legacy_cutoffs_fail_closed_where_v8_reads_them() {
    let cases: &[CutoffCase] = &[
        // 1. THE REPRODUCTION. mjs's local reading archives only row 1 and
        //    keeps 2; the pre-fix port's UTC reading also swept row 3 away.
        CutoffCase {
            name: "archive --before <offset-less date-time>",
            argv: &["decisions", "archive", "--before", "2026-07-26T00:00:00"],
            rust_stderr: "archiveDecisions: --before must be a valid ISO date, got \"2026-07-26T00:00:00\".\n",
            mjs_exit: 0,
            mjs_stdout: "Archived 1 decision(s) to .bee/decisions-archive.jsonl (kept 2 active, cutoff 2026-07-26T00:00:00).\n",
            mjs_stderr: "",
            // The identity of the survivors, not just their count: row 3 is
            // the one the port used to delete and mjs keeps.
            mjs_journal_ids_after: Some(&[CUTOFF_ROW_2, CUTOFF_ROW_3]),
        },
        // 2. The READ side of the same parse, which diverged identically.
        CutoffCase {
            name: "search --since <offset-less date-time>",
            // TEXT mode deliberately: with `--json`, `emitError` routes the
            // payload to STDOUT, and this table's contract is a refusal on
            // STDERR. The `--json` answer set is pinned by the companion
            // test below instead.
            argv: &["decisions", "search", "--since", "2026-07-26T00:00:00"],
            rust_stderr: "--since must be a valid ISO date, got \"2026-07-26T00:00:00\".\n",
            mjs_exit: 0,
            mjs_stdout: "",
            mjs_stderr: "",
            mjs_journal_ids_after: Some(&[CUTOFF_ROW_1, CUTOFF_ROW_2, CUTOFF_ROW_3]),
        },
        // 3. THE DECLARED ARTIFACT the goal-check asked to pin rather than
        //    fix: a V8 legacy-fallback form. Both runtimes exit 1 and leave
        //    the store untouched — they fail closed TOGETHER — but they say
        //    different things, because mjs gets far enough to run the age
        //    rule and find nothing qualifying while the port never accepts
        //    the cutoff at all. Pinned here (not in `--cmd-check`, which
        //    demands identical bytes) so the wording divergence is a
        //    recorded, bounded artifact instead of an unnoticed one.
        CutoffCase {
            name: "archive --before <V8 legacy-fallback form>",
            argv: &["decisions", "archive", "--before", "2026/01/01"],
            rust_stderr: "archiveDecisions: --before must be a valid ISO date, got \"2026/01/01\".\n",
            mjs_exit: 1,
            mjs_stdout: "",
            mjs_stderr: "archiveDecisions: nothing qualifies for archiving — no superseded/redacted events and no decide events strictly older than 2026/01/01 (decision-propagation D4c: --before is explicit or the verb refuses; there is never a default age-based purge).\n",
            mjs_journal_ids_after: Some(&[CUTOFF_ROW_1, CUTOFF_ROW_2, CUTOFF_ROW_3]),
        },
    ];

    let repo = repo_root();
    let journal = cutoff_journal();

    for case in cases {
        let mjs_root = seed_root_journal(&repo, &journal, &[]);
        let rust_root = seed_root_journal(&repo, &journal, &[]);

        let mut node = Command::new("node");
        node.arg(repo.join(".bee").join("bin").join("bee.mjs"));
        let mjs = run_cutoff_leg(node, mjs_root.path(), case.argv);
        let rust = run_cutoff_leg(
            Command::new(env!("CARGO_BIN_EXE_queen-bee")),
            rust_root.path(),
            case.argv,
        );

        // ── the V8 half: the divergence is real, measured, and non-vacuous ──
        assert_eq!(mjs.exit_code, case.mjs_exit, "[{}] mjs exit\nstderr: {}", case.name, mjs.stderr);
        assert_eq!(strip_mjs_timing_line(&mjs.stderr), case.mjs_stderr, "[{}] mjs stderr", case.name);
        if !case.mjs_stdout.is_empty() {
            assert_eq!(mjs.stdout, case.mjs_stdout, "[{}] mjs stdout", case.name);
        }

        // ── the port half: a LOUD refusal over a byte-untouched store ───────
        assert_eq!(rust.exit_code, 1, "[{}] the port must refuse, not guess\nstdout: {}", case.name, rust.stdout);
        assert_eq!(rust.stderr, case.rust_stderr, "[{}] queen-bee stderr", case.name);
        assert_eq!(rust.stdout, "", "[{}] a refusal prints nothing on stdout", case.name);
        assert_eq!(
            fs::read_to_string(rust_root.path().join(".bee/decisions.jsonl")).expect("read the port's journal"),
            journal,
            "[{}] the port refused but still wrote to .bee/decisions.jsonl",
            case.name
        );
        assert!(
            !rust_root.path().join(".bee/decisions-archive.jsonl").exists(),
            "[{}] the port refused but still created an archive sidecar",
            case.name
        );

        // ── and what mjs actually did to ITS store, by id ───────────────────
        if let Some(expected) = case.mjs_journal_ids_after {
            let after = fs::read_to_string(mjs_root.path().join(".bee/decisions.jsonl")).expect("read the mjs journal");
            let ids: Vec<String> = after
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| {
                    let row: Value = serde_json::from_str(l).expect("a journal row parses");
                    field(&row, "id").to_string()
                })
                .collect();
            assert_eq!(ids, expected, "[{}] the mjs leg's surviving rows", case.name);
        }
    }
}

/// The `--since` answer itself, which case 2 above deliberately leaves to
/// this assertion: under V8's LOCAL reading the offset-less cutoff selects
/// rows 2 AND 3, where a UTC reading selects row 2 alone. Without this the
/// case-2 refusal would be pinned against a V8 side nobody had checked was
/// on the other reading at all.
#[test]
fn the_offsetless_since_v8_reads_as_local_selects_the_boundary_row() {
    let repo = repo_root();
    let root = seed_root_journal(&repo, &cutoff_journal(), &[]);
    let mut node = Command::new("node");
    node.arg(repo.join(".bee").join("bin").join("bee.mjs"));
    let mjs = run_cutoff_leg(
        node,
        root.path(),
        &["decisions", "search", "--since", "2026-07-26T00:00:00", "--json"],
    );
    assert_eq!(mjs.exit_code, 0, "mjs leg failed:\n{}", mjs.stderr);

    let payload: Value = serde_json::from_str(&mjs.stdout).expect("the --json payload parses");
    let ids: Vec<&str> = payload
        .get("decisions")
        .and_then(Value::as_array)
        .expect("the payload carries a decisions array")
        .iter()
        .map(|row| field(row, "id"))
        .collect();
    // Order is REVERSE FILE ORDER, not date order: `activeDecisions`'s
    // default (no `--all`) branch is a literal `.reverse()`
    // (`decisions.mjs:832`), and only the `--all` union branch sorts by date.
    // This fixture is the first one whose dates are NOT monotonic with file
    // order, so it is the first that can tell those two apart — row 3 is
    // OLDER than row 2 and still comes out first.
    assert_eq!(
        ids,
        vec![CUTOFF_ROW_3, CUTOFF_ROW_2],
        "V8 must read the offset-less cutoff as LOCAL (+07:00 here); a UTC reading would return only {CUTOFF_ROW_2}"
    );
}
