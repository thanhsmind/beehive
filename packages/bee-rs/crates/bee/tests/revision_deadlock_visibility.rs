// rdv-1 (friction row 618): a NEEDS_REVISION judge verdict reopens a capped
// cell back to "open" (handlers_meta.rs::run_judge_record). A dependent that
// lists the reopened cell as a dep then refuses `cells claim` on unmet deps
// with a GENERIC message — indistinguishable, to a cold reader, from a
// permanent deadlock. This suite drives the REAL judge-record CLI path (not
// a hand-built trace fixture) to reopen a dep, then asserts the dependent's
// claim refusal names the dep, quotes the verdict, and states both
// sanctioned roads — while an ordinary unmet dep (never judged at all) keeps
// today's byte-identical message, and claim-next's silent skip is untouched.
//
// WHY AN INTEGRATION TEST, NOT A UNIT TEST. `run_judge_record` is
// `fn(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode>`,
// dispatch-wrapped and rooted off `std::env::current_dir()` (same shape
// `workflow_verbs.rs` documents for its own trio) — the sanctioned home is
// the built binary against a temp repo root, the same shape
// `cells_archive_sweep.rs` already uses.

use std::path::{Path, PathBuf};
use std::process::Command;

fn binary() -> PathBuf {
    let mut dir = std::env::current_exe().expect("test binary path");
    dir.pop();
    if dir.ends_with("deps") {
        dir.pop();
    }
    let bin = dir.join(format!("bee{}", std::env::consts::EXE_SUFFIX));
    assert!(bin.is_file(), "built bee binary not found at {}", bin.display());
    bin
}

/// A repo with one routed, execution-approved lane ("rdv") so `cells claim`
/// and `cells judge-record` never trip D2/D3/D4 — only the deps check under
/// test.
fn fixture(base: &Path) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee").join("cells")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("lanes")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();
    std::fs::write(
        dir.join(".bee/state.json"),
        r#"{"phase":"executing","feature":"rdv","gates":{}}"#,
    )
    .unwrap();
    std::fs::write(
        dir.join(".bee/lanes/rdv.json"),
        r#"{"feature":"rdv","approved_gates":{"execution":true},"route":true}"#,
    )
    .unwrap();
    dir
}

fn write_cell(repo: &Path, id: &str, status: &str, deps: &[&str]) {
    let deps_json = format!(
        "[{}]",
        deps.iter().map(|d| format!("\"{d}\"")).collect::<Vec<_>>().join(",")
    );
    std::fs::write(
        repo.join(".bee/cells").join(format!("{id}.json")),
        format!(
            r#"{{"id":"{id}","feature":"rdv","status":"{status}","title":"t","lane":"tiny","deps":{deps_json},"verify":"echo ok"}}"#
        ),
    )
    .unwrap();
}

fn run(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(binary()).args(args).current_dir(cwd).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

/// A schema-valid `judge-verdict/1` NEEDS_REVISION verdict (at least one
/// FAIL check + a failure_signature, per judge.rs::validate_judge_verdict).
const NEEDS_REVISION_VERDICT: &str = r#"{
  "schema": "judge-verdict/1",
  "verdict": "NEEDS_REVISION",
  "checks": [{"id": "c1", "status": "FAIL", "evidence": "boom"}],
  "failure_signature": "sig-1",
  "fixability": "automatic",
  "confidence": "high"
}"#;

/// RED-FIRST TARGET: a dep reopened by the REAL judge-record path names
/// itself, quotes the verdict, and states both sanctioned roads.
#[test]
fn claim_refusal_names_a_revision_reopened_dep_and_its_two_roads() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());

    // The dep starts capped — the only state judge-record reopens.
    write_cell(&repo, "rdv-dep", "capped", &[]);
    write_cell(&repo, "rdv-consumer", "open", &["rdv-dep"]);

    let verdict_file = tmp.path().join("verdict.json");
    std::fs::write(&verdict_file, NEEDS_REVISION_VERDICT).unwrap();

    // Drive the SAME function the CLI uses to record a judge verdict —
    // this is what actually reopens capped -> open, per handlers_meta.rs.
    let (code, out) = run(
        &repo,
        &["cells", "judge-record", "--id", "rdv-dep", "--file", verdict_file.to_str().unwrap()],
    );
    assert_eq!(code, 0, "{out}");
    let dep_after = std::fs::read_to_string(repo.join(".bee/cells/rdv-dep.json")).unwrap();
    assert!(dep_after.contains("\"status\": \"open\""), "{dep_after}");
    assert!(dep_after.contains("NEEDS_REVISION"), "{dep_after}");

    // Claiming the dependent must refuse — but naming the real cause.
    let (code, out) = run(&repo, &["cells", "claim", "--id", "rdv-consumer", "--worker", "w1"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("\"rdv-consumer\" has uncapped deps"), "{out}");
    assert!(out.contains("\"rdv-dep\""), "dep id must be named: {out}");
    assert!(out.contains("\"NEEDS_REVISION\""), "verdict kind must be quoted: {out}");
    // Both sanctioned roads, named explicitly — not a permanent deadlock.
    assert!(
        out.contains("claim and re-cap the reopened dependency"),
        "road (a) — claim+re-cap the dep — missing: {out}"
    );
    assert!(
        out.contains("bee cells update") && out.contains("recording a reason"),
        "road (b) — cells update with a recorded reason — missing: {out}"
    );
    // The dependent cell itself was never mutated by the refused claim (the
    // fixture wrote it compact, un-pretty-printed — a refusal must not
    // rewrite the file at all).
    let consumer_after = std::fs::read_to_string(repo.join(".bee/cells/rdv-consumer.json")).unwrap();
    assert!(consumer_after.contains("\"status\":\"open\""), "{consumer_after}");
}

/// PIN: an ordinary unmet dep (never touched by a judge) keeps today's
/// generic message byte-identical — the deps LAW does not change here.
#[test]
fn ordinary_unmet_dep_keeps_the_byte_identical_generic_message() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());

    write_cell(&repo, "ord-dep", "open", &[]);
    write_cell(&repo, "ord-consumer", "open", &["ord-dep"]);

    let (code, out) = run(&repo, &["cells", "claim", "--id", "ord-consumer", "--worker", "w1"]);
    assert_ne!(code, 0, "{out}");
    assert!(
        out.contains("claimCell: cell \"ord-consumer\" has uncapped deps: ord-dep — deps must be capped first."),
        "{out}"
    );
    // None of the revision-visibility language leaks into the ordinary case.
    assert!(!out.contains("NEEDS_REVISION"), "{out}");
    assert!(!out.contains("sanctioned"), "{out}");
    assert!(!out.contains("permanent deadlock"), "{out}");
}

/// PIN: claim-next's silent skip over an unready cell is untouched — it
/// still reports the generic "nothing ready" outcome, never naming a
/// specific dep or verdict, even when the only cell in the store is blocked
/// on a revision-reopened dep.
#[test]
fn claim_next_silent_skip_is_unchanged_by_revision_visibility() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());

    write_cell(&repo, "cn-dep", "capped", &[]);
    write_cell(&repo, "cn-consumer", "open", &["cn-dep"]);

    let verdict_file = tmp.path().join("verdict.json");
    std::fs::write(&verdict_file, NEEDS_REVISION_VERDICT).unwrap();
    let (code, out) = run(
        &repo,
        &["cells", "judge-record", "--id", "cn-dep", "--file", verdict_file.to_str().unwrap()],
    );
    assert_eq!(code, 0, "{out}");

    let (code, out) = run(&repo, &["cells", "claim-next", "--session-id", "sess-1", "--worker", "w1"]);
    assert_ne!(code, 0, "a genuinely empty ready set must still refuse: {out}");
    assert!(out.contains("NO_APPROVED_WORK"), "{out}");
    // The skip stays silent about the specific cell/dep/verdict — claim-next
    // never gained the named-refusal text `cells claim` now carries.
    assert!(!out.contains("cn-dep"), "{out}");
    assert!(!out.contains("NEEDS_REVISION"), "{out}");
}
