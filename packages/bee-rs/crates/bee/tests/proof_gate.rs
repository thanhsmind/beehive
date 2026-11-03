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
fn declared_test_invocation(root: &Path) -> String {
    let config: serde_json::Value =
        serde_json::from_str(&read(root, ".bee/config.json")).expect(".bee/config.json parses");
    let raw = config["commands"]["test"]
        .as_str()
        .expect(".bee/config.json declares commands.test as a string");
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
fn ci_runs_the_declared_test_command_and_adds_no_flags_to_it() {
    let root = repo_root();
    let declared = declared_test_invocation(&root);
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
