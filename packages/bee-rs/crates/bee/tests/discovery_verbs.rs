// `bee discovery list` / `bee discovery stub` — black-box, against the
// built binary, mirroring `workflow_verbs.rs`'s fixture+binary() shape.
//
// The two must_haves this cell was scoped around:
//   - `bee discovery list` reports an effort created by `bee discovery
//     stub` with its frontier count.
//   - `stub` onto an existing effort slug refuses typed, writes nothing.
//
// Frontier arithmetic itself (open/claimed/blocked-by combinations,
// malformed-MAP.md fail-open) is unit-tested in
// `src/verbs/discovery.rs`'s own `#[cfg(test)]` module, against
// `scan_discovery`/`scan_tickets` directly — no binary spawn needed there.
// This file covers the layer above: argv parsing, root resolution, the
// emitted text/JSON shape, and the refusal's "nothing written" guarantee.

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

/// A bare repo: onboarding complete, no discovery dir yet.
fn fixture(base: &Path) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    dir
}

fn run(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(binary()).args(args).current_dir(cwd).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)),
    )
}

fn run_json(cwd: &Path, args: &[&str]) -> (i32, serde_json::Value) {
    let out = Command::new(binary()).args(args).current_dir(cwd).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let v = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("`bee {}` did not print JSON: {stdout} ({e})", args.join(" ")));
    (out.status.code().unwrap_or(-1), v)
}

#[test]
fn list_reports_an_effort_created_by_stub_with_its_frontier_count() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());

    let (code, v) = run_json(
        &repo,
        &["discovery", "stub", "--effort", "onboarding-flow", "--from", "an itch about onboarding", "--json"],
    );
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["effort"], "onboarding-flow");
    let map_path = repo.join("docs/discovery/onboarding-flow/MAP.md");
    assert!(map_path.is_file(), "stub must create MAP.md");
    let map_text = std::fs::read_to_string(&map_path).unwrap();
    assert!(map_text.contains("(unknown — charting session needed)"), "{map_text}");
    assert!(map_text.contains("an itch about onboarding"), "{map_text}");

    // One open, unclaimed, unblocked ticket — hand-authored the way the
    // wayfinding skill would author one; this module never writes tickets.
    std::fs::create_dir_all(repo.join("docs/discovery/onboarding-flow/tickets")).unwrap();
    std::fs::write(
        repo.join("docs/discovery/onboarding-flow/tickets/001-scope.md"),
        "type: grilling\nstatus: open\n\nWhat should the onboarding flow cover?\n",
    )
    .unwrap();

    let (code, v) = run_json(&repo, &["discovery", "list", "--json"]);
    assert_eq!(code, 0, "{v}");
    let efforts = v["efforts"].as_array().unwrap();
    assert_eq!(efforts.len(), 1, "{v}");
    assert_eq!(efforts[0]["name"], "onboarding-flow");
    assert_eq!(efforts[0]["frontier"], 1);
    assert_eq!(efforts[0]["open"], 1);
    assert!(v["unreadable"].as_array().unwrap().is_empty(), "{v}");

    // The plain-text line names the same fields the JSON does.
    let (code, text) = run(&repo, &["discovery", "list"]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("onboarding-flow"), "{text}");
    assert!(text.contains("frontier 1"), "{text}");
}

#[test]
fn list_is_empty_with_no_discovery_dir_at_all() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    let (code, text) = run(&repo, &["discovery", "list"]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("No discovery maps."), "{text}");
}

#[test]
fn stub_onto_an_existing_effort_slug_refuses_typed_and_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());

    let (code, out) = run(&repo, &["discovery", "stub", "--effort", "dup", "--from", "first"]);
    assert_eq!(code, 0, "{out}");
    let map_path = repo.join("docs/discovery/dup/MAP.md");
    let before = std::fs::read_to_string(&map_path).unwrap();

    let (code, out) = run(&repo, &["discovery", "stub", "--effort", "dup", "--from", "second attempt"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("already exists"), "{out}");

    let after = std::fs::read_to_string(&map_path).unwrap();
    assert_eq!(before, after, "a refused stub must not touch the existing MAP.md");
}

#[test]
fn stub_rejects_a_non_kebab_slug_before_writing_anything() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    let (code, out) = run(&repo, &["discovery", "stub", "--effort", "Not Kebab", "--from", "x"]);
    assert_ne!(code, 0, "{out}");
    assert!(!repo.join("docs/discovery").exists(), "nothing must be written on refusal");
}

#[test]
fn an_unreadable_map_surfaces_the_remedy_line_never_a_crash() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path());
    let dir = repo.join("docs/discovery/broken");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("MAP.md"), [0xFF, 0xFE, 0x00, 0xFF]).unwrap();

    let (code, text) = run(&repo, &["discovery", "list"]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("unreadable "), "{text}");
    assert!(text.contains("— remedy: fix or delete"), "{text}");
}
