// The registry↔dispatcher contract.
//
// THE DEFECT THIS EXISTS TO CATCH. `--help` renders from the embedded command
// registry; the dispatcher is a chain of per-verb `try_native` probes. Nothing
// tied the two together, so deleting the Node runtime (commit 5c62cad0) left
// the registry — and therefore `--help`, and therefore every agent reading it
// — advertising 23 commands with no implementation behind them at all:
//
//     $ bee doctor --runtime claude
//     bee: unsupported command shape: `bee doctor --runtime claude`.
//
// `doctor` is porcelain. It is the verb that answers "is this harness
// installed correctly", and on Codex the only one that lifts an install from
// degraded to ready. `bee config get/set/validate` — taught by
// docs/config-reference.md — went the same way, as did all of `perf`,
// `recovery`, `herding enable|disable|status`, `state advisor-ref *` and
// `state compact-*`.
//
// THE CONTRACT. Every registry entry carries its own canonical `examples[0]`.
// Run it against the built binary in a throwaway repo and the dispatcher's
// answer must match the entry's declaration, in BOTH directions:
//
//   entry has no `unavailable` marker  → the example must NOT be refused by
//                                        the dispatcher (it may still fail on
//                                        repo state — that is the verb
//                                        talking, which means it exists)
//   entry HAS an `unavailable` marker  → the example MUST hit the
//                                        command-unavailable refusal, so a
//                                        marker cannot outlive the gap it
//                                        describes
//
// The second direction matters as much as the first: when `doctor` is ported,
// this test fails until the marker comes off, which is how the registry stops
// drifting a second time.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// `router::REFUSAL_HEADLINES` verbatim — the leading phrase of each refusal
/// class. `router::refusal_headlines_are_stable` fails if a message is
/// reworded out from under this list, so the walk below cannot go vacuous by
/// quietly matching nothing.
const REFUSAL_MARKERS: [&str; 5] = [
    "bee: unknown command",
    "bee: not built into this binary",
    "bee: unexpected positional argument",
    "bee: missing required argument",
    "bee: unsupported argument shape",
];
const UNAVAILABLE_MARKER: &str = "bee: not built into this binary";

fn payload() -> serde_json::Value {
    serde_json::from_str(include_str!("../src/generated/registry_payload.json"))
        .expect("the embedded registry payload must be parseable JSON")
}

/// Split an example on spaces, honouring the double quotes the registry uses
/// for multi-word values.
fn split_example(example: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for ch in example.chars() {
        match ch {
            '"' => quoted = !quoted,
            ' ' if !quoted => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// A repo the binary will accept as a root, with a declared test command that
/// runs nothing: `bee test` and `bee cells finish` appear in the walk.
fn scratch_repo(base: &Path, n: usize) -> std::path::PathBuf {
    let dir = base.join(format!("repo-{n}"));
    std::fs::create_dir_all(dir.join(".bee").join("logs")).unwrap();
    std::fs::write(dir.join(".bee/onboarding.json"), r#"{"version":1,"completed":true}"#).unwrap();
    std::fs::write(dir.join(".bee/config.json"), r#"{"commands":{"test":"none"}}"#).unwrap();
    std::fs::write(dir.join(".bee/state.json"), r#"{"phase":"executing","gates":{}}"#).unwrap();
    dir
}

struct Verdict {
    refused: bool,
    unavailable: bool,
    output: String,
}

fn run_example(bin: &Path, cwd: &Path, example: &str) -> Verdict {
    let argv = split_example(example);
    assert_eq!(argv.first().map(String::as_str), Some("bee"), "example must start with bee");
    let out = Command::new(bin)
        .args(&argv[1..])
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("spawning {}: {e}", bin.display()));
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Verdict {
        refused: REFUSAL_MARKERS.iter().any(|m| output.contains(m)),
        unavailable: output.contains(UNAVAILABLE_MARKER),
        output,
    }
}

fn binary() -> std::path::PathBuf {
    // The test binary lives in target/<profile>/deps/, next to bee itself.
    let mut dir = std::env::current_exe().expect("test binary path");
    dir.pop();
    if dir.ends_with("deps") {
        dir.pop();
    }
    let bin = dir.join(format!("bee{}", std::env::consts::EXE_SUFFIX));
    assert!(bin.is_file(), "built bee binary not found at {}", bin.display());
    bin
}

#[test]
fn every_declared_command_dispatches_exactly_as_the_registry_says_it_does() {
    let bin = binary();
    let tmp = tempfile::tempdir().unwrap();
    let p = payload();
    let commands = p["commands"].as_array().expect("commands array");

    let mut advertised_but_dead: Vec<String> = Vec::new();
    let mut marked_but_alive: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for (i, entry) in commands.iter().enumerate() {
        let name = entry["name"].as_str().expect("string name");
        let example = entry["examples"][0].as_str().expect("at least one example");
        let declared_unavailable = entry.get("unavailable").and_then(|u| u.as_object()).is_some();

        let cwd = scratch_repo(tmp.path(), i);
        let v = run_example(&bin, &cwd, example);
        checked += 1;

        if declared_unavailable {
            if !v.unavailable {
                marked_but_alive.push(format!("{name}\n    {example}\n    -> {}", first_line(&v.output)));
            }
        } else if v.refused || v.unavailable {
            advertised_but_dead
                .push(format!("{name}\n    {example}\n    -> {}", first_line(&v.output)));
        }
    }

    assert!(checked > 100, "only {checked} commands walked — the test went vacuous");
    assert!(
        advertised_but_dead.is_empty(),
        "these commands are in the registry (and so in `bee --help`) but the dispatcher refuses \
         their own canonical example. Either serve the shape, or mark the entry `unavailable` \
         with a reason and a fix:\n\n{}",
        advertised_but_dead.join("\n")
    );
    assert!(
        marked_but_alive.is_empty(),
        "these commands are marked `unavailable` in the registry but the dispatcher did NOT give \
         the command-unavailable refusal — the marker is stale, drop it:\n\n{}",
        marked_but_alive.join("\n")
    );
}

fn first_line(s: &str) -> String {
    s.lines().find(|l| !l.trim().is_empty()).unwrap_or("(no output)").trim().to_string()
}

/// The refusal is a REFUSAL: non-zero exit, output on exactly one stream, and
/// a `kind` a caller can branch on instead of regex-matching prose.
#[test]
fn the_json_refusal_carries_a_machine_readable_kind() {
    let bin = binary();
    let tmp = tempfile::tempdir().unwrap();
    let cwd = scratch_repo(tmp.path(), 0);
    let cases = [
        (vec!["definitely-not-a-bee-verb", "--json"], "unknown_command"),
        (vec!["knowledge", "context", "--json"], "missing_required_argument"),
        (vec!["doctor", "--runtime", "codex", "--json"], "command_unavailable"),
        (vec!["state", "show", "--json"], "unknown_command"),
        (vec!["status", "wat", "--json"], "unexpected_argument"),
    ];
    for (argv, kind) in cases {
        let out = Command::new(&bin).args(&argv).current_dir(&cwd).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{argv:?} must exit non-zero");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let v: serde_json::Value = serde_json::from_str(stdout.trim())
            .unwrap_or_else(|e| panic!("{argv:?} did not print JSON: {stdout:?} ({e})"));
        assert_eq!(v["kind"], kind, "{argv:?} classified as {}", v["kind"]);
        assert!(v["error"].as_str().is_some_and(|e| !e.is_empty()), "{argv:?} has no error text");
    }
}

/// The two namespaces, end to end against the built binary.
///
/// The porcelain/plumbing split used to be a help FILTER: `bee --help` showed
/// 17 of 124 top-level verbs and nothing else moved, so the lifecycle's own
/// steps were still spelled in terms of the store they live in (`state route`,
/// `intent set`, `state gate`, `cells finish`) and the skills carried the
/// translation. These assert the split is real in both directions.
#[test]
fn the_flow_surface_and_the_internal_namespace_are_both_callable() {
    let bin = binary();
    let tmp = tempfile::tempdir().unwrap();
    let cwd = scratch_repo(tmp.path(), 0);

    // 1. The flow surface is exactly what `bee --help` lists, and every entry
    //    on it dispatches.
    let out = Command::new(&bin).args(["--help", "--json"]).current_dir(&cwd).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("--help --json is JSON");
    let listed: Vec<String> = v["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect();
    for flow in ["route", "shape", "gate", "finish"] {
        assert!(listed.contains(&flow.to_string()), "`{flow}` is missing from the flow surface: {listed:?}");
        let out = Command::new(&bin).args([flow, "--help"]).current_dir(&cwd).output().unwrap();
        assert!(out.status.success(), "`bee {flow} --help` failed");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains(&format!("bee {flow}")), "`bee {flow} --help` names something else: {text}");
    }

    // 2. `bee internal …` dispatches the plumbing verb it wraps.
    let bare = Command::new(&bin).args(["state", "lanes", "--json"]).current_dir(&cwd).output().unwrap();
    let wrapped = Command::new(&bin)
        .args(["internal", "state", "lanes", "--json"])
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert_eq!(bare.stdout, wrapped.stdout, "`bee internal <verb>` must be the same call");
    assert_eq!(bare.status.code(), wrapped.status.code());

    // 3. …and REFUSES a flow verb, so the boundary is not decoration.
    let out = Command::new(&bin).args(["internal", "gate"]).current_dir(&cwd).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("is a flow command, not plumbing"), "{stderr}");

    // 4. The namespace has its own help surface, and it holds no flow verb.
    let out = Command::new(&bin)
        .args(["internal", "--help", "--json"])
        .current_dir(&cwd)
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["surface"], "internal");
    let inner: Vec<&str> =
        v["commands"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert!(inner.len() > 50, "the plumbing surface is suspiciously small: {}", inner.len());
    for flow in ["route", "shape", "gate", "finish", "status", "orient"] {
        assert!(!inner.contains(&flow), "`{flow}` is flow, not plumbing");
    }
    // Spelled the way the namespace is called.
    let first = v["commands"][0]["invoke"].as_str().unwrap();
    assert!(first.starts_with("bee internal "), "{first}");
}

/// Every gap is described, not merely flagged — a marker without a reason and
/// a way forward just moves the dead end.
#[test]
fn every_unavailable_marker_names_a_reason_and_a_fix() {
    let p = payload();
    let mut seen = BTreeSet::new();
    for entry in p["commands"].as_array().unwrap() {
        let Some(gap) = entry.get("unavailable") else { continue };
        let name = entry["name"].as_str().unwrap();
        for key in ["reason", "fix"] {
            let text = gap[key].as_str().unwrap_or_default();
            assert!(!text.trim().is_empty(), "{name}: unavailable.{key} is empty");
        }
        seen.insert(name.to_string());
    }
    assert!(
        !seen.is_empty(),
        "no command is marked unavailable — if every gap was closed, delete this test with the \
         marker support; do not leave it passing vacuously"
    );
}
