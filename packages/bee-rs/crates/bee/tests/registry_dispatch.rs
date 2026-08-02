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

// ─── the other direction: SERVED but never DECLARED ────────────────────────
//
// `every_declared_command_dispatches_exactly_as_the_registry_says_it_does`
// walks registry → dispatcher and catches an entry with nothing behind it.
// Nothing walked dispatcher → registry, and the R6 cutover left that gap
// populated too: twelve shapes the binary serves were absent from the
// registry, so `bee --help --all` — the map an agent reads to find out what
// it may run — did not list them. `bee onboard` was one of them, and
// `skills/bee-hive/SKILL.md` tells the agent to run it by name.
//
// An undeclared verb fails WORSE than an undeclared flag: `--help` says the
// command does not exist, so the agent stops looking and improvises. That is
// the same expensive wrong turn `crate::catalog` was built to prevent, coming
// in through the door catalog cannot see.
//
// The served set is read out of the dispatcher's own match arms rather than
// hand-listed here, so a namespace that grows a verb grows this law's subject
// in the same commit. Each scan carries a floor: a refactor that renames the
// function or reshapes the match makes the scan find nothing, and a law with
// an empty subject must fail, not pass.

/// Namespaces whose verbs are deliberately NOT registry commands, each with
/// the reason it is not a gap. An exclusion without a reason is how the first
/// drift started, so the shape is the same as `unavailable`'s.
const NOT_A_COMMAND_SURFACE: [(&str, &str); 1] = [(
    "hook",
    "hooks are invoked by the runtime through wiring `bee onboard` writes, never typed by an \
     agent; listing nine of them in --help would add noise to the one surface that must stay \
     scannable",
)];

fn crate_src(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// The `"name" =>` arms of the `try_native` that follows `marker` in `src`.
/// `floor` is the vacuity guard: fewer arms than this means the scan lost its
/// subject and the law is no longer testing anything.
fn match_arm_verbs(src: &str, marker: &str, floor: usize) -> BTreeSet<String> {
    let start = src.find(marker).unwrap_or_else(|| panic!("{marker:?} no longer appears — the \
        dispatcher moved and this scan is now vacuous; re-point it"));
    let body = &src[start..];
    let end = body.find("\n}").unwrap_or(body.len());
    let mut found = BTreeSet::new();
    for line in body[..end].lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix('"') else { continue };
        let Some(name) = rest.split('"').next() else { continue };
        if line.contains("=>") && !name.is_empty() {
            found.insert(name.to_string());
        }
    }
    assert!(
        found.len() >= floor,
        "{marker}: found only {} match arm(s) ({found:?}), expected at least {floor} — the scan \
         no longer sees the dispatcher it is meant to police",
        found.len()
    );
    found
}

#[test]
fn every_namespace_the_dispatcher_serves_is_declared_in_the_registry() {
    let p = payload();
    let invocations: BTreeSet<String> = p["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["invoke"].as_str())
        .map(str::to_string)
        .collect();

    let mut served: Vec<String> = Vec::new();
    for verb in match_arm_verbs(&crate_src("devtools/mod.rs"), "pub fn try_native", 5) {
        served.push(format!("bee dev {verb}"));
    }
    for verb in match_arm_verbs(&crate_src("herding.rs"), "pub fn try_native", 5) {
        served.push(format!("bee herding {verb}"));
    }
    // Two single-word probes the router answers before the verb tree; both are
    // spelled in `router::dispatch` and have no match arm to scan.
    for bare in ["bee onboard", "bee rs-info"] {
        served.push(bare.to_string());
    }
    assert!(served.len() >= 12, "the served set collapsed to {served:?}");

    let excluded: BTreeSet<&str> = NOT_A_COMMAND_SURFACE.iter().map(|(ns, _)| *ns).collect();
    let missing: Vec<&String> = served
        .iter()
        .filter(|inv| {
            let ns = inv.split_whitespace().nth(1).unwrap_or("");
            !excluded.contains(ns) && !invocations.contains(*inv)
        })
        .collect();

    assert!(
        missing.is_empty(),
        "the dispatcher serves these shapes and the registry does not declare them, so \
         `bee --help --all` reports them as unknown commands. Add an entry (with an example \
         this file can run), or add the namespace to NOT_A_COMMAND_SURFACE with its reason:\n\n  {}",
        missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n  ")
    );

    // The exclusion list is not a place to park a gap: every reason must be
    // written out, and the surface must still be one the binary really serves.
    for (ns, reason) in NOT_A_COMMAND_SURFACE {
        assert!(reason.len() > 40, "{ns}: the exclusion needs a real reason, got {reason:?}");
        assert!(
            !invocations.iter().any(|inv| inv.starts_with(&format!("bee {ns} "))),
            "{ns} is excluded as \"not a command surface\" but the registry declares it — drop \
             one or the other"
        );
    }
}

/// The scan bites. Without this, a rename inside `devtools` could leave both
/// the scanner and the law quietly matching nothing.
#[test]
fn the_served_set_scan_bites_on_a_new_arm_and_refuses_an_empty_one() {
    let fake = r#"
pub fn try_native(args: &[OsString]) -> Option<ExitCode> {
    match *name {
        "alpha" => a::run(flags),
        "beta" => b::run(flags),
        _ => None,
    }
}
"#;
    let found = match_arm_verbs(fake, "pub fn try_native", 2);
    assert_eq!(found.iter().map(String::as_str).collect::<Vec<_>>(), ["alpha", "beta"]);

    // …and the floor is real.
    let thin = "pub fn try_native() {\n    match *name {\n        _ => None,\n    }\n}\n";
    let panicked = std::panic::catch_unwind(|| match_arm_verbs(thin, "pub fn try_native", 2));
    assert!(panicked.is_err(), "a scan that finds nothing must fail, not pass vacuously");
}

/// The registry payload is the agent-facing map. After the Node runtime was
/// deleted it still named `lib/feedback.mjs`, `lib/compaction.mjs`,
/// `lib/prompt-renderer.mjs`, `claims.mjs`, `cells.mjs`, `lease-store.mjs`,
/// `workflow-store.mjs` — and
/// `.claude/skills/bee-herding/scripts/dispatch-interlock.mjs`, a path that
/// does not exist in any checkout. `registry.rs` already asserted this for its
/// one drift HINT; the 153 KB of text an agent actually reads had no such
/// check.
#[test]
fn the_registry_names_no_artifact_the_node_deletion_removed() {
    let raw = include_str!("../src/generated/registry_payload.json");
    assert!(raw.len() > 100_000, "the payload shrank unexpectedly: {} bytes", raw.len());

    let mut offenders: Vec<String> = Vec::new();
    for hit in raw.match_indices(".mjs") {
        let start = raw[..hit.0].rfind(' ').map(|s| s + 1).unwrap_or(0);
        let end = (hit.0 + 4).min(raw.len());
        offenders.push(raw[start..end].to_string());
    }
    assert!(
        offenders.is_empty(),
        "the registry describes commands in terms of files the R6 cutover deleted. An agent \
         reading `bee --help` is told to look for, or run, something that is not there:\n\n  {}",
        offenders.join("\n  ")
    );
}

/// `--help --all --json` is the map AGENTS.block.md points an agent at, and
/// it is 212 KB — roughly 53k tokens to answer "what may I call". `--names`
/// is the index form. These pin the two properties that make it usable: it
/// covers the SAME command set (an index that quietly drops rows is worse
/// than no index), and it is small enough to be the default read.
#[test]
fn the_names_index_covers_the_full_surface_at_a_fraction_of_the_size() {
    let bin = binary();
    let tmp = tempfile::tempdir().unwrap();
    let cwd = scratch_repo(tmp.path(), 0);

    let run = |args: &[&str]| -> (serde_json::Value, usize) {
        let out = Command::new(&bin).args(args).current_dir(&cwd).output().unwrap();
        assert!(out.status.success(), "`bee {}` failed", args.join(" "));
        let v = serde_json::from_slice(&out.stdout)
            .unwrap_or_else(|e| panic!("`bee {}` did not print JSON: {e}", args.join(" ")));
        (v, out.stdout.len())
    };

    let (full, full_bytes) = run(&["--help", "--all", "--json"]);
    let (index, index_bytes) = run(&["--help", "--all", "--names", "--json"]);

    let names_of = |v: &serde_json::Value| -> BTreeSet<String> {
        v["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        names_of(&full),
        names_of(&index),
        "the index must list exactly the commands the full view lists"
    );
    assert_eq!(index["view"], "names");
    assert_eq!(index["total_commands"], full["total_commands"]);

    // Every row still says whether it can actually be run, and carries enough
    // to decide whether to spend tokens on the full text.
    let unavailable_full: BTreeSet<String> = full["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c.get("unavailable").and_then(|u| u.as_object()).is_some())
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect();
    let unavailable_index: BTreeSet<String> = index["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["unavailable"] == serde_json::Value::Bool(true))
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        unavailable_full, unavailable_index,
        "an index that hides the not-built marker steers an agent into a dead verb"
    );
    for c in index["commands"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let summary = c["summary"].as_str().unwrap_or("");
        assert!(!summary.trim().is_empty(), "{name}: the index row has no summary");
        assert!(summary.chars().count() <= 161, "{name}: summary not cut: {summary}");
        assert!(c["invoke"].as_str().is_some_and(|i| i.starts_with("bee ")), "{name}");
    }

    // The whole point. If the index ever stops being dramatically cheaper,
    // it is not doing its job and the pointer in AGENTS.block.md is wrong.
    assert!(
        index_bytes * 4 < full_bytes,
        "the index is {index_bytes} bytes against the full view's {full_bytes} — not enough of a \
         saving to be worth a second surface"
    );

    // The flow surface takes the same flag.
    let (flow_index, _) = run(&["--help", "--names", "--json"]);
    assert_eq!(flow_index["surface"], "porcelain");
    assert!(!flow_index["commands"].as_array().unwrap().is_empty());
}
