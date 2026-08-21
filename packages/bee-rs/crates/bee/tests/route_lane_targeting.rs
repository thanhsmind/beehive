// `bee route` target resolution: `--no-lane`, and the unbound-session refusal
// that stops a blind `--set` from clobbering another feature's triage (srg-1).
//
// THE DEFECT THIS EXISTS TO CATCH. `run_route` used to resolve its target with
// a hardcoded `None`/`false`, so a session that had never run
// `bee state session bind` fell through to the DEFAULT record,
// `.bee/state.json` — which, in a repo running lanes, names a different
// feature entirely. The write landed silently: it stamped `route.feature` with
// state.json's own feature and overwrote that feature's recorded triage
// (lane/flags/files/rationale) with the caller's. `bee gate` cannot lose a
// route this way, because it threads `--lane`/`--no-lane` through the same
// resolution seam.
//
// The fix is deliberately NOT gate's flag pair: `bee route` already has a
// `--lane`, and it means the triage class (docs/tiny/small/spike/standard/
// high-risk), not a lane record. Route therefore gets `--no-lane` only, plus a
// refusal that makes the ambiguous case impossible to reach by accident.
//
// WHY AN INTEGRATION TEST, NOT A UNIT TEST. `run_route` is
// `fn(Flags, bool, Instant) -> Option<ExitCode>`, printing as a side effect,
// and its root comes from `prelude` reading `std::env::current_dir()` — there
// is no `run_route_body(root, flags)` split. Calling it in-process would mean
// mutating the test runner's own cwd across parallel tests. The sanctioned
// home is the built binary against a temp repo root, the same shape
// `workflow_verbs.rs` already uses.

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

/// A repo mid-flight: onboarding complete, a default record naming its OWN
/// feature, and (optionally) live lane records for other features.
fn fixture(base: &Path, lanes: &[(&str, &str)]) -> PathBuf {
    let dir = base.join("repo");
    std::fs::create_dir_all(dir.join(".bee").join("cells")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::create_dir_all(dir.join(".bee").join("lanes")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();
    std::fs::write(
        dir.join(".bee/state.json"),
        r#"{"phase":"shaping","feature":"feat-default","gates":{}}"#,
    )
    .unwrap();
    for (feature, phase) in lanes {
        std::fs::write(
            dir.join(".bee/lanes").join(format!("{feature}.json")),
            format!(r#"{{"phase":"{phase}","feature":"{feature}","gates":{{}}}}"#),
        )
        .unwrap();
    }
    dir
}

/// Binds `session` to `lane` the way `state session bind` does, and returns the
/// env pair that makes the CLI resolve that session.
fn bind_session(repo: &Path, session: &str, lane: &str) -> (&'static str, String) {
    std::fs::create_dir_all(repo.join(".bee").join("sessions")).unwrap();
    // A heartbeat far in the future can never read as stale, whatever the
    // staleness window is; the id is pinned by BEE_SESSION_ID either way.
    std::fs::write(
        repo.join(".bee/sessions").join(format!("{session}.json")),
        format!(
            r#"{{"id":"{session}","started_at":"2999-01-01T00:00:00.000Z","last_heartbeat":"2999-01-01T00:00:00.000Z","lane":"{lane}"}}"#
        ),
    )
    .unwrap();
    ("BEE_SESSION_ID", session.to_string())
}

fn run(cwd: &Path, args: &[&str], env: Option<(&str, String)>) -> (i32, String) {
    let mut cmd = Command::new(binary());
    cmd.args(args).current_dir(cwd);
    // A worker nickname leaking in from the calling shell would change nothing
    // here, but the tests must not depend on the ambient environment.
    cmd.env_remove("BEE_AGENT_NAME");
    cmd.env_remove("BEE_SESSION_ID");
    cmd.env_remove("CLAUDE_CODE_SESSION_ID");
    if let Some((k, v)) = env {
        cmd.env(k, v);
    }
    let out = cmd.output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn state_json(repo: &Path) -> String {
    std::fs::read_to_string(repo.join(".bee/state.json")).unwrap()
}

/// The canonical `--set` call. `--lane standard` here is route's OWN flag —
/// the triage class — never a lane-record selector.
fn set_args() -> Vec<&'static str> {
    vec![
        "route", "--set", "--class", "feature", "--lane", "standard", "--flags", "", "--files", "3",
    ]
}

#[test]
fn an_unbound_set_against_the_default_record_is_refused_while_lanes_are_live() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path(), &[("lane-a", "shaping"), ("lane-b", "planning")]);
    let before = state_json(&repo);

    let (code, out) = run(&repo, &set_args(), None);

    assert_eq!(code, 1, "the refusal must be a non-zero exit: {out}");
    assert!(
        out.contains("route --set: refused"),
        "the refusal must name what it refused: {out}"
    );
    assert!(
        out.contains("not bound to a lane") && out.contains("2 lane record(s) are live"),
        "the refusal must name the real live-lane count: {out}"
    );
    assert!(
        out.contains("lane-a") && out.contains("lane-b"),
        "the refusal must name the live lanes inline: {out}"
    );
    assert!(
        out.contains("carries feature \"feat-default\""),
        "the refusal must name what the default record actually holds: {out}"
    );
    // BOTH exits, so the caller never has to guess which one applies.
    assert!(
        out.contains("bee state session bind --lane"),
        "the refusal must name the bind fix: {out}"
    );
    assert!(
        out.contains("--no-lane"),
        "the refusal must name the --no-lane fix: {out}"
    );

    // Zero mutation: the refusal lands before any field is set, so the default
    // record is byte-identical — this is the whole point of the guard.
    assert_eq!(state_json(&repo), before, "the refusal wrote to the default record");
    assert!(
        !state_json(&repo).contains("route"),
        "no route landed anywhere in the default record: {}",
        state_json(&repo)
    );
}

#[test]
fn no_lane_makes_writing_the_default_record_an_explicit_allowed_act() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path(), &[("lane-a", "shaping"), ("lane-b", "planning")]);

    let mut args = set_args();
    args.push("--no-lane");
    let (code, out) = run(&repo, &args, None);

    assert_eq!(code, 0, "--no-lane must succeed: {out}");
    assert!(out.contains("Recorded route"), "the write must confirm: {out}");
    let state = state_json(&repo);
    assert!(state.contains("\"route\""), "the route must land on the default record: {state}");
    assert!(
        state.contains("\"feature\": \"feat-default\""),
        "the route is stamped for the default record's own feature: {state}"
    );
    // The lane records are untouched.
    for lane in ["lane-a", "lane-b"] {
        let raw = std::fs::read_to_string(repo.join(".bee/lanes").join(format!("{lane}.json"))).unwrap();
        assert!(!raw.contains("route"), "{lane} must be untouched: {raw}");
    }
}

#[test]
fn a_bound_session_still_writes_its_own_lane_record_and_is_never_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path(), &[("lane-a", "shaping"), ("lane-b", "planning")]);
    let env = bind_session(&repo, "sess-1", "lane-a");
    let before = state_json(&repo);

    let (code, out) = run(&repo, &set_args(), Some(env));

    assert_eq!(code, 0, "today's working path must be unchanged: {out}");
    assert!(out.contains("(lane \"lane-a\")"), "the write must name the lane: {out}");
    let lane_a = std::fs::read_to_string(repo.join(".bee/lanes/lane-a.json")).unwrap();
    assert!(lane_a.contains("\"route\""), "the route lands on the lane record: {lane_a}");
    assert!(
        lane_a.contains("\"feature\": \"lane-a\""),
        "the route is stamped for the lane's own feature: {lane_a}"
    );
    assert_eq!(state_json(&repo), before, "a bound session never touches the default record");
}

#[test]
fn a_repo_with_no_live_lane_records_behaves_exactly_as_before() {
    let tmp = tempfile::tempdir().unwrap();
    // No lane records at all, then the same repo with only TERMINAL lanes —
    // neither is a rival writer, so neither may trip the guard.
    let repo = fixture(tmp.path(), &[]);
    let (code, out) = run(&repo, &set_args(), None);
    assert_eq!(code, 0, "a single-lane repo must be untouched by the guard: {out}");
    assert!(state_json(&repo).contains("\"route\""), "the route must land: {}", state_json(&repo));

    let tmp2 = tempfile::tempdir().unwrap();
    let repo2 = fixture(
        tmp2.path(),
        &[("lane-done", "compounding-complete"), ("lane-idle", "idle")],
    );
    let (code2, out2) = run(&repo2, &set_args(), None);
    assert_eq!(code2, 0, "terminal lane records are not live work: {out2}");
    assert!(state_json(&repo2).contains("\"route\""), "the route must land: {}", state_json(&repo2));
}

#[test]
fn show_is_never_refused_and_no_lane_retargets_the_read() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path(), &[("lane-a", "shaping"), ("lane-b", "planning")]);

    // (1) unbound, lanes live — the exact shape `--set` refuses. Reading the
    // default record is harmless, so it answers instead of refusing.
    let (code, out) = run(&repo, &["route", "--show"], None);
    assert_eq!(code, 0, "--show must never be refused: {out}");
    assert!(!out.contains("refused"), "--show must never be refused: {out}");

    // (2) a bound session reads its LANE, and --no-lane sends the same read
    // back to the default record — the `no_lane` bool reaches the read path
    // too, not only the write path.
    let env = bind_session(&repo, "sess-1", "lane-a");
    let mut args = set_args();
    args.push("--no-lane");
    let (code, out) = run(&repo, &args, None);
    assert_eq!(code, 0, "seed the default record: {out}");

    let (code, out) = run(&repo, &["route", "--show"], Some(env.clone()));
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("No route recorded."),
        "a bound session reads its own lane, which has no route: {out}"
    );

    let (code, out) = run(&repo, &["route", "--show", "--no-lane"], Some(env));
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("class=feature") && !out.contains("(lane "),
        "--no-lane must read the default record: {out}"
    );
}

#[test]
fn no_lane_is_type_checked_like_every_other_boolean_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = fixture(tmp.path(), &[]);
    let before = state_json(&repo);

    let mut args = set_args();
    args.push("--no-lane=garbage");
    let (code, out) = run(&repo, &args, None);

    assert_ne!(code, 0, "a non-boolean --no-lane must not be accepted: {out}");
    assert!(
        out.contains("unsupported argument shape"),
        "bool_flag_ok hands a bad value to the dispatcher's own refusal: {out}"
    );
    assert_eq!(state_json(&repo), before, "a rejected flag value writes nothing");
}
