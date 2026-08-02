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

/// The declared command, minus any leading `VAR=...` environment prefix (CI
/// already has cargo on PATH; a local session may not).
fn declared_verify_invocation(root: &Path) -> String {
    let config: serde_json::Value =
        serde_json::from_str(&read(root, ".bee/config.json")).expect(".bee/config.json parses");
    let raw = config["commands"]["verify"]
        .as_str()
        .expect(".bee/config.json declares commands.verify as a string");
    strip_env_prefix(raw)
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
    workflow
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter(|l| l.contains("cargo test"))
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
fn ci_runs_the_declared_verify_command_and_adds_no_flags_to_it() {
    let root = repo_root();
    let declared = declared_verify_invocation(&root);
    assert!(
        declared.starts_with("cargo test"),
        "commands.verify is expected to be a cargo invocation; got {declared:?}"
    );
    for wf in WORKFLOWS {
        let lines = cargo_test_lines(&read(&root, wf));
        assert!(!lines.is_empty(), "{wf} runs no cargo test at all");
        for line in lines {
            assert_eq!(
                line, declared,
                "{wf} runs a different suite from .bee/config.json commands.verify.\n\
                 A gate whose CI proof is a different command from the local one proves \
                 nothing about the local one — that is exactly how `-- --test-threads=1` \
                 kept a flaky parallel suite green on CI for the whole cutover."
            );
        }
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
