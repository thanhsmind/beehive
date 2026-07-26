//! `--self-test`: proves the fixture generator both directions (advisor
//! note 2, rust-port Slice 0; rust-port-19 grew this to the three new
//! floors + the non-triviality proof) — all seven sub-pinned-size refusal
//! cases actually refuse, AND a happy-path generation succeeds, measurably
//! meets every pin on disk, and a real `node .bee/bin/bee.mjs status
//! --json` (a) accepts the generated store as its root and (b) reports
//! NON-TRIVIAL `review`/`recovery` blocks — never run with cwd inside the
//! repo tree (the fixture lives under the OS temp dir).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::fixture::{
    self, FixtureRequest, BACKLOG_FLOOR_BYTES, CELLS_FLOOR_COUNT, DECISIONS_FLOOR_BYTES,
    GIT_COMMITS_FLOOR_COUNT, RESERVATIONS_FLOOR_BYTES, REVIEW_CANDIDATES_FLOOR_COUNT,
    TRANSCRIPT_TAIL_FLOOR_BYTES,
};

pub fn run(repo_root: &Path) -> Result<String, String> {
    let mut summary = Vec::new();

    check_refusal_cases()?;
    summary.push("7/7 sub-pinned-size refusal cases refused".to_string());

    let (report, fixture_root) = run_happy_path()?;
    summary.push(format!(
        "happy-path generation met every pin (decisions={}B reservations={}B backlog={}B cells={} git_commits={} review_candidates={} transcript_tail={}B)",
        report.decisions_bytes,
        report.reservations_bytes,
        report.backlog_bytes,
        report.cells_count,
        report.git_commits_count,
        report.review_candidates_count,
        report.transcript_tail_bytes,
    ));

    let sanity_result = sanity_read(repo_root, &fixture_root);
    // Clean up regardless of the sanity-read outcome — never leave a stray
    // fixture behind on a failed self-test.
    let _ = fs::remove_dir_all(&fixture_root);
    let non_triviality = sanity_result?;
    summary.push("bee.mjs status --json sanity read against the generated store: exit 0".to_string());
    summary.push(non_triviality);

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

    let mut cells_case = FixtureRequest::at_floors(unused.clone());
    cells_case.cells_count = CELLS_FLOOR_COUNT - 1;
    expect_refusal("cells count", &cells_case)?;

    let mut review_candidates_case = FixtureRequest::at_floors(unused.clone());
    review_candidates_case.review_candidates_count = REVIEW_CANDIDATES_FLOOR_COUNT - 1;
    expect_refusal("review candidates count", &review_candidates_case)?;

    let mut git_commits_case = FixtureRequest::at_floors(unused.clone());
    git_commits_case.git_commits_count = GIT_COMMITS_FLOOR_COUNT - 1;
    expect_refusal("git commits count", &git_commits_case)?;

    let mut transcript_case = FixtureRequest::at_floors(unused);
    transcript_case.transcript_tail_bytes = TRANSCRIPT_TAIL_FLOOR_BYTES - 1;
    expect_refusal("transcript tail bytes", &transcript_case)?;

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

    // rust-port-19: measure the three new floors on disk too — never trust
    // the return value alone, same discipline as the four checks above.
    let candidates_path = bee_dir.join("review-candidates.jsonl");
    let candidates_text =
        fs::read_to_string(&candidates_path).map_err(|e| format!("read {}: {e}", candidates_path.display()))?;
    let candidates_count = candidates_text.lines().filter(|l| !l.trim().is_empty()).count();
    if candidates_count < REVIEW_CANDIDATES_FLOOR_COUNT {
        return Err(format!(
            "generated review-candidates.jsonl has {candidates_count} line(s), under the {REVIEW_CANDIDATES_FLOOR_COUNT} floor"
        ));
    }
    let reviews_dir = bee_dir.join("reviews");
    let review_sessions = fs::read_dir(&reviews_dir)
        .map_err(|e| format!("read_dir {}: {e}", reviews_dir.display()))?
        .count();
    if review_sessions < 3 {
        return Err(format!(
            "generated .bee/reviews/ has {review_sessions} session file(s), expected >= 3 (stale/reviewed/in-review coverage)"
        ));
    }

    let git_commits_out = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(&fixture_root)
        .output()
        .map_err(|e| format!("spawn git rev-list on generated fixture: {e}"))?;
    if !git_commits_out.status.success() {
        return Err(format!(
            "git rev-list --count HEAD on the generated fixture exited {:?}: {}",
            git_commits_out.status.code(),
            String::from_utf8_lossy(&git_commits_out.stderr)
        ));
    }
    let git_commits_count: usize = String::from_utf8_lossy(&git_commits_out.stdout)
        .trim()
        .parse()
        .map_err(|e| format!("parsing git rev-list --count output: {e}"))?;
    if git_commits_count < GIT_COMMITS_FLOOR_COUNT {
        return Err(format!(
            "generated fixture git history has {git_commits_count} commit(s), under the {GIT_COMMITS_FLOOR_COUNT} floor"
        ));
    }

    let transcript_len = file_len(&report.transcript_file)?;
    if transcript_len < TRANSCRIPT_TAIL_FLOOR_BYTES {
        return Err(format!(
            "generated crash-candidate transcript {} is {transcript_len} bytes, under the {TRANSCRIPT_TAIL_FLOOR_BYTES} byte floor",
            report.transcript_file.display()
        ));
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
/// dir, not under `repo_root`), and prove NON-TRIVIALITY on both expensive
/// blocks (rust-port-19: a fixture that still measures nothing is a failed
/// generation, not a pass) — never the host `$HOME`/live repo `.bee/`
/// (`fixture_root` is a temp dir with its own injected transcript root, and
/// this function only ever reads the SUBPROCESS's stdout, never the host
/// store directly). Returns a one-line summary of what it proved.
fn sanity_read(repo_root: &Path, fixture_root: &Path) -> Result<String, String> {
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
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let trimmed = stdout.trim_start();
    if !trimmed.starts_with('{') || !stdout.contains("\"phase\"") {
        return Err(format!(
            "bee.mjs status --json sanity read produced unexpected output (first 200 chars): {}",
            &stdout[..stdout.len().min(200)]
        ));
    }

    let value: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("bee.mjs status --json output did not parse as JSON: {e}"))?;

    // root resolution: the resolved store root the real bee.mjs picked must
    // BE this fixture (module doc comment's git-colocation claim, proven
    // empirically rather than just read from source). status --json has no
    // direct "root" field, so this is proven indirectly: onboarding.installed
    // is true (it read OUR onboarding.json, not some ancestor's) and drift is
    // false (managed:{} short-circuits computeRuntimeDrift, per fixture.rs's
    // write_onboarding comment) — a fixture that resolved to the WRONG root
    // (e.g. an ancestor with no onboarding.json at all) would report
    // installed:false instead.
    let onboarding_installed = value
        .get("onboarding")
        .and_then(|o| o.get("installed"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !onboarding_installed {
        return Err(
            "root_resolution: bee.mjs status --json reported onboarding.installed=false — the fixture did not resolve as its own bee root".to_string(),
        );
    }

    let review = value
        .get("review")
        .ok_or_else(|| "non_triviality: status --json output has no \"review\" key".to_string())?;
    if review.get("degraded").is_some() {
        return Err(format!(
            "non_triviality: review block is degraded (buildReviewBlock caught an exception): {review}"
        ));
    }
    let review_total = review
        .get("candidates")
        .and_then(|c| c.get("total"))
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "non_triviality: review.candidates.total missing or not a number".to_string())?;
    if review_total == 0 {
        return Err(
            "non_triviality: review.candidates.total is 0 — buildReviewBlock looped zero times, exactly the hollow-fixture failure this cell fixes".to_string(),
        );
    }

    let recovery = value
        .get("recovery")
        .ok_or_else(|| "non_triviality: status --json output has no \"recovery\" key".to_string())?;
    if recovery.get("degraded").is_some() {
        return Err(format!(
            "non_triviality: recovery block is degraded (buildRecoveryBlock caught an exception): {recovery}"
        ));
    }
    let recovery_candidates_len = recovery
        .get("candidates")
        .and_then(|c| c.as_array())
        .map(|a| a.len())
        .ok_or_else(|| "non_triviality: recovery.candidates missing or not an array".to_string())?;
    if recovery_candidates_len == 0 {
        return Err(
            "non_triviality: recovery.candidates is empty — detectCrashCandidates matched no encoded-project-dir, exactly the hollow-fixture failure this cell fixes".to_string(),
        );
    }

    Ok(format!(
        "non-triviality proved: root_resolution=self, review.candidates.total={review_total} (non-degraded), recovery.candidates={recovery_candidates_len} (non-degraded)"
    ))
}
