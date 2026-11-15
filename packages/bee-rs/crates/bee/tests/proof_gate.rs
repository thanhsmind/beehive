// The proof gate's own contracts.
//
// bee's central claim is "green caps the cell; red refuses the cap". That
// claim is only worth as much as the command behind it, so the command itself
// needs pinning — twice over:
//
//   1. CI must run the command a local `bee cells finish` runs. It did not:
//      ci.yml appended `-- --test-threads=1` to a declared command that has no
//      such flag, so the suite CI proved green was a DIFFERENT suite from the
//      one gating cells — and the parallel one was the red one.
//   2. CI must run on the change it is gating. Both workflows were
//      schedule-only while the session preamble told agents to "check CI
//      instead of running anything locally", i.e. to trust evidence that could
//      predate the change by a day.
//
// Both are cheap to state and were expensive to miss, so they are laws here
// rather than review habits.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    // …/beehive/packages/bee-rs/crates/bee -> …/beehive
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// The declared commands, minus any leading `VAR=...` environment prefix (CI
/// already has cargo on PATH; a local session may not). `commands.test` is a
/// string or a list of strings.
fn declared_test_invocations(root: &Path) -> Vec<String> {
    let config: serde_json::Value =
        serde_json::from_str(&read(root, ".bee/config.json")).expect(".bee/config.json parses");
    test_invocations(&config["commands"]["test"])
}

fn test_invocations(test: &serde_json::Value) -> Vec<String> {
    let raws: Vec<&str> = match test {
        serde_json::Value::String(s) => vec![s.as_str()],
        serde_json::Value::Array(items) => items
            .iter()
            .map(|v| {
                v.as_str()
                    .expect(".bee/config.json commands.test entries are strings")
            })
            .collect(),
        _ => panic!(".bee/config.json declares commands.test as a string or a list of strings"),
    };
    raws.into_iter().map(strip_env_prefix).collect()
}

fn entries_without_a_ci_run_line(entries: &[String], ci: &str) -> Vec<String> {
    entries
        .iter()
        .filter(|c| !c.starts_with("cargo test"))
        .filter(|c| !run_lines(ci, c).contains(c))
        .cloned()
        .collect()
}

fn declared_test_invocation(root: &Path) -> String {
    declared_test_invocations(root)
        .into_iter()
        .find(|c| c.starts_with("cargo test"))
        .expect(".bee/config.json commands.test has no cargo test entry")
}

fn strip_env_prefix(command: &str) -> String {
    let mut rest = command.trim();
    // `NAME=value ` assignments, value possibly quoted, repeated.
    while let Some(eq) = rest.find('=') {
        let name = &rest[..eq];
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
            break;
        }
        let after = &rest[eq + 1..];
        let end = if let Some(stripped) = after.strip_prefix('"') {
            match stripped.find('"') {
                Some(close) => eq + 1 + 1 + close + 1,
                None => break,
            }
        } else {
            match after.find(' ') {
                Some(sp) => eq + 1 + sp,
                None => break,
            }
        };
        rest = rest[end..].trim_start();
    }
    rest.to_string()
}

fn cargo_test_lines(workflow: &str) -> Vec<String> {
    run_lines(workflow, "cargo test")
}

fn run_lines(workflow: &str, needle: &str) -> Vec<String> {
    workflow
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter(|l| l.contains(needle))
        .map(|l| {
            // `run: <cmd>`, redirections and pipeline tails are plumbing
            // around the invocation, not flags passed to it.
            let l = l.strip_prefix("run:").unwrap_or(l).trim();
            let l = l.split('|').next().unwrap_or(l).trim();
            let l = l.split(" 2>&1").next().unwrap_or(l).trim();
            l.trim_end_matches('\\').trim().to_string()
        })
        .collect()
}

const WORKFLOWS: [&str; 2] = [".github/workflows/ci.yml", ".github/workflows/windows.yml"];

#[test]
fn ci_runs_the_declared_test_command_and_adds_no_flags_to_it() {
    let root = repo_root();
    let declared = declared_test_invocation(&root);
    let ci = read(&root, WORKFLOWS[0]);
    let missing = entries_without_a_ci_run_line(&declared_test_invocations(&root), &ci);
    assert!(
        missing.is_empty(),
        "{} has no run line equal to the declared test entries {missing:?}",
        WORKFLOWS[0]
    );
    assert!(
        declared.starts_with("cargo test"),
        "commands.test is expected to be a cargo invocation; got {declared:?}"
    );
    for wf in WORKFLOWS {
        let lines = cargo_test_lines(&read(&root, wf));
        assert!(!lines.is_empty(), "{wf} runs no cargo test at all");
        for line in lines {
            assert_eq!(
                line, declared,
                "{wf} runs a different suite from .bee/config.json commands.test.\n\
                 A gate whose CI proof is a different command from the local one proves \
                 nothing about the local one — that is exactly how `-- --test-threads=1` \
                 kept a flaky parallel suite green on CI for the whole cutover."
            );
        }
    }
}

#[test]
fn a_list_valued_test_path_fails_parity_when_ci_has_no_run_line_for_its_clippy_entry() {
    let clippy = "cargo clippy --manifest-path packages/bee-rs/Cargo.toml";
    let suite = "cargo test --release --manifest-path packages/bee-rs/Cargo.toml";
    let entries = test_invocations(&serde_json::json!([
        format!("PATH=\"x:$PATH\" {clippy}"),
        format!("PATH=\"x:$PATH\" {suite}"),
    ]));
    assert_eq!(entries, vec![clippy.to_string(), suite.to_string()]);

    let without = format!("steps:\n  - name: Run tests\n    run: {suite}\n");
    assert_eq!(entries_without_a_ci_run_line(&entries, &without), vec![clippy.to_string()]);

    let commented = format!("steps:\n  # run: {clippy}\n  - name: Run tests\n    run: {suite}\n");
    assert_eq!(entries_without_a_ci_run_line(&entries, &commented), vec![clippy.to_string()]);

    let flagged = format!(
        "steps:\n  - name: Clippy\n    run: {clippy} -- -D warnings\n  - name: Run tests\n    run: {suite}\n"
    );
    assert_eq!(entries_without_a_ci_run_line(&entries, &flagged), vec![clippy.to_string()]);

    let with =
        format!("steps:\n  - name: Clippy\n    run: {clippy}\n  - name: Run tests\n    run: {suite}\n");
    assert!(entries_without_a_ci_run_line(&entries, &with).is_empty());
}

#[test]
fn a_string_valued_test_path_still_reads_as_one_entry() {
    let entries = test_invocations(&serde_json::json!("PATH=\"x:$PATH\" cargo test --release"));
    assert_eq!(entries, vec!["cargo test --release".to_string()]);
    assert!(entries_without_a_ci_run_line(&entries, "steps: []\n").is_empty());
}

#[cfg(unix)]
fn release_test_gate_harness(root: &Path) -> String {
    let script = read(root, "scripts/release.sh");
    let line_starting = |prefix: &str| {
        script
            .lines()
            .find(|l| l.starts_with(prefix))
            .unwrap_or_else(|| panic!("scripts/release.sh has no line starting {prefix:?}"))
            .to_string()
    };
    let start = script
        .find("if command -v jq >/dev/null 2>&1; then")
        .expect("scripts/release.sh no longer reads commands.test behind `command -v jq`");
    let end_marker = "done <<< \"$TEST_CMDS\"";
    let end = start
        + script[start..]
            .find(end_marker)
            .expect("scripts/release.sh no longer loops over $TEST_CMDS")
        + end_marker.len();
    format!(
        "{}\n{}\n{}\necho GATE-PASSED\n",
        line_starting("log()"),
        line_starting("fail()"),
        &script[start..end]
    )
}

#[cfg(unix)]
fn run_release_test_gate(test: serde_json::Value, without_jq: bool) -> (bool, String, String, String) {
    let root = repo_root();
    let dir = std::env::temp_dir().join(format!(
        "proof-gate-release-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let config = dir.join("config.json");
    std::fs::write(&config, serde_json::json!({ "commands": { "test": test } }).to_string())
        .unwrap();
    let shadow = if without_jq {
        "command() { if [ \"$1\" = -v ] && [ \"$2\" = jq ]; then return 1; fi; builtin command \"$@\"; }\n"
    } else {
        ""
    };
    let harness = dir.join("gate.sh");
    std::fs::write(&harness, format!("{shadow}{}", release_test_gate_harness(&root))).unwrap();
    let out = std::process::Command::new("bash")
        .arg(&harness)
        .current_dir(&dir)
        .env("BEE_CONFIG", &config)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("bash runs");
    let trace = std::fs::read_to_string(dir.join("trace")).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        trace,
    )
}

#[cfg(unix)]
#[test]
fn the_release_gate_runs_each_listed_entry_in_order_and_stops_at_the_first_red() {
    for without_jq in [false, true] {
        let (ok, stdout, stderr, trace) = run_release_test_gate(
            serde_json::json!([
                "echo one >> trace",
                "cat > /dev/null; echo two >> trace",
                "false",
                "echo four >> trace",
            ]),
            without_jq,
        );
        assert!(!ok, "a red entry must fail the gate (without_jq={without_jq}): {stderr}");
        assert_eq!(trace, "one\ntwo\n", "without_jq={without_jq}: {stderr}");
        assert!(!stdout.contains("GATE-PASSED"), "without_jq={without_jq}: {stdout}");
        assert!(
            stderr.contains("went RED at `false`"),
            "the refusal must name the red entry (without_jq={without_jq}): {stderr}"
        );

        let (ok, stdout, stderr, trace) =
            run_release_test_gate(serde_json::json!(["echo one >> trace", "echo two >> trace"]), without_jq);
        assert!(ok, "without_jq={without_jq}: {stderr}");
        assert_eq!(trace, "one\ntwo\n", "without_jq={without_jq}: {stderr}");
        assert!(stdout.contains("GATE-PASSED"), "without_jq={without_jq}: {stdout}");
    }
}

#[cfg(unix)]
#[test]
fn the_release_gate_still_runs_a_string_valued_test_path() {
    for without_jq in [false, true] {
        let (ok, stdout, stderr, trace) =
            run_release_test_gate(serde_json::json!("echo solo >> trace"), without_jq);
        assert!(ok, "without_jq={without_jq}: {stderr}");
        assert_eq!(trace, "solo\n", "without_jq={without_jq}: {stderr}");
        assert!(stdout.contains("GATE-PASSED"), "without_jq={without_jq}: {stdout}");

        let (ok, stdout, stderr, _) = run_release_test_gate(serde_json::json!("false"), without_jq);
        assert!(!ok, "without_jq={without_jq}: {stderr}");
        assert!(!stdout.contains("GATE-PASSED"), "without_jq={without_jq}: {stdout}");
    }
}

#[test]
fn ci_runs_on_the_change_it_gates_not_only_on_a_timer() {
    let root = repo_root();
    for wf in WORKFLOWS {
        let text = read(&root, wf);
        // The `on:` block ends at the first column-0 key after it.
        let on = text
            .split_once("\non:")
            .map(|(_, rest)| rest)
            .unwrap_or_else(|| panic!("{wf} has no on: block"));
        let block: String = on
            .lines()
            .take_while(|l| l.is_empty() || l.starts_with(' ') || l.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        for trigger in ["push:", "pull_request:"] {
            assert!(
                block.lines().map(str::trim).any(|l| l == trigger),
                "{wf} does not trigger on {trigger} — its freshest evidence can predate the \
                 change it gates. Block was:\n{block}"
            );
        }
    }
}

#[test]
fn env_prefix_stripping_only_eats_environment_assignments() {
    assert_eq!(strip_env_prefix("cargo test --release"), "cargo test --release");
    assert_eq!(strip_env_prefix("PATH=\"a:$PATH\" cargo test"), "cargo test");
    assert_eq!(strip_env_prefix("A=1 B=2 cargo test"), "cargo test");
    // Not an assignment: an `=` inside the command itself is left alone.
    assert_eq!(strip_env_prefix("cargo test --cfg x=y"), "cargo test --cfg x=y");
}

// ── the qualified-proof teaching fence ─────────────────────────────────────
//
// `PROOF_RESULT_VALUES` closed the proof line's result segment over three
// values (docs/history/proof-strength-and-expiry, D1). A doc, prompt or
// refusal message still showing the now-refused bare `green` teaches the
// refused form — the reader copies the example and the cap refuses.
//
// This is an ALLOWLIST, `route_class_parity.rs`'s design: it names the sites
// that TEACH the form by example and asserts each one still shows a qualified
// value. It is deliberately NOT a whole-tree denylist — the read path stays
// tolerant of historical bare-`green` caps (D2), and the fixtures in
// `verbs/cells/proof.rs`, `verbs/work.rs`, `verbs/mailbox.rs` and
// `verbs/drivers/close.rs` are that tolerance's in-tree evidence. A denylist
// would forbid the evidence along with the mistake.
//
// The rot guard is the anchor: a listed site whose anchor is gone FAILS
// rather than quietly dropping out of the set.

/// The vocabulary's single home. This file reads it as TEXT — the constant is
/// `pub(crate)` and an integration test cannot import it — and states no value
/// of its own, so the fence cannot agree with a stale copy of itself.
const FINISH_SUPPORT_RS: &str = "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs";

/// (path, anchor). The anchor identifies the line carrying that site's
/// example; that line must hold one of `PROOF_RESULT_VALUES`. Where a site
/// teaches by example twice, one anchor matches both lines and both are
/// checked.
const TEACHING_SITES: &[(&str, &str)] = &[
    ("packages/bee/prompts/worker-cell.md", "cargo test -p bee"),
    ("skills/bee-swarming/references/worker-details.md", "cargo test -p bee"),
    ("docs/product-description/verification/lifecycle.md", r#""outcome":"note added""#),
    ("docs/product-description/verification/foundations.md", r#"--report "<cmd>"#),
    ("docs/product-description/lifecycle/execution.md", "cargo test -p auth"),
    ("site/guide/vi/cell-lane.html", "cargo test -p auth"),
    (
        "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
        "Test gates disabled by repo declaration",
    ),
    (
        "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs",
        "Test gates disabled by repo declaration",
    ),
    (
        "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
        "refused — --report is required",
    ),
];

/// Read the `const PROOF_RESULT_VALUES: [&str; N] = [ … ];` initializer out of
/// `finish_support.rs` as text. The declared arity is checked against what was
/// found, so a misparse says so instead of fencing a half-read list.
fn proof_result_values(src: &str) -> Vec<String> {
    let decl = "const PROOF_RESULT_VALUES: [&str; ";
    let at = src.find(decl).unwrap_or_else(|| {
        panic!(
            "`PROOF_RESULT_VALUES` is no longer declared as `{decl}...` in {FINISH_SUPPORT_RS}.\n\n\
             FIX: point this parser at the new declaration — do NOT paste the values in here."
        )
    });
    let rest = &src[at + decl.len()..];
    let arity: usize = rest[..rest.find(']').expect("the `[&str; N]` type is unterminated")]
        .trim()
        .parse()
        .expect("`PROOF_RESULT_VALUES` has a non-numeric arity in its type");

    let eq = rest.find('=').expect("the declaration has no `=`");
    let open = eq + rest[eq..].find('[').expect("the initializer has no `[`");
    let close = open + rest[open..].find(']').expect("the initializer has no `]`");
    // One value per line, each on its own line under its meaning comment.
    let values: Vec<String> = rest[open + 1..close]
        .lines()
        .filter_map(|l| l.trim().strip_prefix('"'))
        .filter_map(|l| l.split('"').next())
        .map(str::to_string)
        .collect();

    assert_eq!(
        values.len(),
        arity,
        "read {values:?} out of `PROOF_RESULT_VALUES` but its type declares {arity} value(s) — \
         this fence misparsed the source and would guard the wrong list"
    );
    values
}

#[test]
fn every_site_teaching_the_proof_line_shows_a_qualified_result() {
    let root = repo_root();
    let values = proof_result_values(&read(&root, FINISH_SUPPORT_RS));
    let mut bare: Vec<String> = Vec::new();

    for (rel, anchor) in TEACHING_SITES {
        let text = read(&root, rel);
        let mut anchored = 0usize;
        for (i, line) in text.lines().enumerate() {
            if !line.contains(anchor) {
                continue;
            }
            anchored += 1;
            if !values.iter().any(|v| line.contains(v.as_str())) {
                bare.push(format!("  {rel}:{}\n      {}", i + 1, line.trim()));
            }
        }
        assert!(
            anchored > 0,
            "{rel} no longer carries the anchor {anchor:?}, so this fence stopped reading it.\n\n\
             A site that silently drops out of the allowlist is the drift this test exists to \
             catch. FIX: update the anchor here, or drop the site if that document genuinely \
             stopped teaching the proof line by example."
        );
    }

    assert!(
        bare.is_empty(),
        "site(s) teaching the cap's proof line show a result segment that is not one of [{}] \
         ({FINISH_SUPPORT_RS}):\n\n{}\n\nAn example showing an unqualified `green` teaches the \
         form the cap now REFUSES — the reader copies it and the cap refuses. FIX: qualify the \
         example with the honest value for that site.",
        values.join(" "),
        bare.join("\n"),
    );
}
