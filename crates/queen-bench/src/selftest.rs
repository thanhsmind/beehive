//! `--self-test`: proves the fixture generator both directions (advisor
//! note 2, rust-port Slice 0) — all four sub-pinned-size refusal cases
//! actually refuse, AND a happy-path generation succeeds, measurably meets
//! every pin on disk, and a real `node .bee/bin/bee.mjs status --json`
//! accepts the generated store as its root (never run with cwd inside the
//! repo tree — the fixture lives under the OS temp dir).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::fixture::{
    self, FixtureRequest, BACKLOG_FLOOR_BYTES, CELLS_FLOOR_COUNT, DECISIONS_FLOOR_BYTES,
    RESERVATIONS_FLOOR_BYTES,
};

pub fn run(repo_root: &Path) -> Result<String, String> {
    let mut summary = Vec::new();

    check_refusal_cases()?;
    summary.push("4/4 sub-pinned-size refusal cases refused".to_string());

    let (report, fixture_root) = run_happy_path()?;
    summary.push(format!(
        "happy-path generation met every pin (decisions={}B reservations={}B backlog={}B cells={})",
        report.decisions_bytes, report.reservations_bytes, report.backlog_bytes, report.cells_count
    ));

    let sanity_result = sanity_read(repo_root, &fixture_root);
    // Clean up regardless of the sanity-read outcome — never leave a stray
    // fixture behind on a failed self-test.
    let _ = fs::remove_dir_all(&fixture_root);
    sanity_result?;
    summary.push("bee.mjs status --json sanity read against the generated store: exit 0".to_string());

    Ok(summary.join("; "))
}

fn check_refusal_cases() -> Result<(), String> {
    let unused = PathBuf::from("/unused-in-refusal-check");

    let mut decisions_case = FixtureRequest::at_floors(unused.clone());
    decisions_case.decisions_bytes = DECISIONS_FLOOR_BYTES - 1;
    expect_refusal("decisions.jsonl", &decisions_case)?;

    let mut reservations_case = FixtureRequest::at_floors(unused.clone());
    reservations_case.reservations_bytes = RESERVATIONS_FLOOR_BYTES - 1;
    expect_refusal("reservations.json", &reservations_case)?;

    let mut backlog_case = FixtureRequest::at_floors(unused.clone());
    backlog_case.backlog_bytes = BACKLOG_FLOOR_BYTES - 1;
    expect_refusal("backlog.jsonl", &backlog_case)?;

    let mut cells_case = FixtureRequest::at_floors(unused);
    cells_case.cells_count = CELLS_FLOOR_COUNT - 1;
    expect_refusal("cells count", &cells_case)?;

    Ok(())
}

fn expect_refusal(label: &str, req: &FixtureRequest) -> Result<(), String> {
    match fixture::generate(req) {
        Ok(_) => Err(format!(
            "REFUSAL CHECK FAILED: sub-pinned-size case '{label}' unexpectedly succeeded"
        )),
        Err(_) => Ok(()),
    }
}

fn run_happy_path() -> Result<(fixture::FixtureReport, PathBuf), String> {
    let fixture_root = std::env::temp_dir().join(format!(
        "queen-bench-selftest-{}-{}",
        std::process::id(),
        now_nanos()
    ));
    let _ = fs::remove_dir_all(&fixture_root);

    let req = FixtureRequest::at_floors(fixture_root.clone());
    let report = fixture::generate(&req)
        .map_err(|e| format!("happy-path generation unexpectedly refused: {e}"))?;

    // Measure on disk — never trust the return value alone (advisor note 1's
    // "measurably meets every pin").
    let bee_dir = fixture_root.join(".bee");
    let decisions_len = file_len(&bee_dir.join("decisions.jsonl"))?;
    if decisions_len < DECISIONS_FLOOR_BYTES {
        return Err(format!(
            "generated decisions.jsonl is {decisions_len} bytes, under the {DECISIONS_FLOOR_BYTES} byte floor"
        ));
    }
    let reservations_len = file_len(&bee_dir.join("reservations.json"))?;
    if reservations_len < RESERVATIONS_FLOOR_BYTES {
        return Err(format!(
            "generated reservations.json is {reservations_len} bytes, under the {RESERVATIONS_FLOOR_BYTES} byte floor"
        ));
    }
    let backlog_len = file_len(&bee_dir.join("backlog.jsonl"))?;
    if backlog_len < BACKLOG_FLOOR_BYTES {
        return Err(format!(
            "generated backlog.jsonl is {backlog_len} bytes, under the {BACKLOG_FLOOR_BYTES} byte floor"
        ));
    }
    let cells_count = fs::read_dir(bee_dir.join("cells"))
        .map_err(|e| format!("read_dir cells: {e}"))?
        .count();
    if cells_count < CELLS_FLOOR_COUNT {
        return Err(format!(
            "generated cells dir has {cells_count} file(s), under the {CELLS_FLOOR_COUNT} floor"
        ));
    }
    if !bee_dir.join("onboarding.json").exists() {
        return Err("generated store is missing .bee/onboarding.json — root resolution would fail".to_string());
    }

    Ok((report, fixture_root))
}

fn file_len(path: &Path) -> Result<u64, String> {
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("stat {}: {e}", path.display()))
}

fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// Run the real `bee.mjs status --json` with cwd at the generated fixture
/// root, never inside the repo tree (the fixture root is under the OS temp
/// dir, not under `repo_root`).
fn sanity_read(repo_root: &Path, fixture_root: &Path) -> Result<(), String> {
    if fixture_root.starts_with(repo_root) {
        return Err(format!(
            "refusing to run the sanity-read subprocess: fixture root {} is inside the repo tree {}",
            fixture_root.display(),
            repo_root.display()
        ));
    }
    let bee_mjs = repo_root.join(".bee").join("bin").join("bee.mjs");
    if !bee_mjs.exists() {
        return Err(format!("bee.mjs not found at {}", bee_mjs.display()));
    }
    let output = Command::new("node")
        .arg(&bee_mjs)
        .arg("status")
        .arg("--json")
        .current_dir(fixture_root)
        .output()
        .map_err(|e| format!("failed to spawn node sanity read: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "bee.mjs status --json sanity read exited {:?}; stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim_start();
    if !trimmed.starts_with('{') || !stdout.contains("\"phase\"") {
        return Err(format!(
            "bee.mjs status --json sanity read produced unexpected output (first 200 chars): {}",
            &stdout[..stdout.len().min(200)]
        ));
    }
    Ok(())
}
