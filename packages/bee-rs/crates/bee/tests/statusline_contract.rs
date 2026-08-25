// statusline_contract — the vendored status line's two silent-failure guards.
//
// WHY THIS FILE EXISTS. `packages/bee/statusline/statusline-command.sh` is
// canonical; onboarding vendors it byte-for-byte into every opted-in host at
// `<repo>/.claude/statusline-command.sh` (spec onboarding R4). After the R6
// cutover BOTH guards over that arrangement were gone:
//
//   * The byte-equality sweep lived in `packages/bee/tests/test_lib.mjs`,
//     deleted with the Node tree. The same cutover commit fixed the VENDORED
//     copy's binary lookup and left the CANONICAL one on the old shape, so the
//     pair drifted in the very commit that removed the test that watched it —
//     and stayed drifted, because the next `bee onboard --apply` re-vendors
//     canonical over the host copy and quietly undid the fix.
//   * Nothing ever RAN the script. The canonical lookup hunted the binary
//     relative to `${BASH_SOURCE[0]}` (`$SELF_DIR/../bee`, `$SELF_DIR/../../bee`
//     and their `.exe` forms), which from `<repo>/.claude/` resolves to
//     `<repo>/bee` and `<repo>/../bee`. The binary is at `<repo>/.bee/bin/bee`.
//     Nothing matched, the per-model token/cost line vanished, and the script
//     exited 0 — a failure with no output at all, on either side.
//
// So the tests below are deliberately not text probes: one compares the two
// files as bytes, and three EXECUTE the real script in fixture hosts — an
// ordinary checkout carrying the binary, a real linked worktree that must fall
// through to the main checkout's copy, and a host with no binary at all, which
// must still render line one and exit 0 because a status line is never allowed
// to be why a prompt fails to render.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn read_bytes(rel: &str) -> Vec<u8> {
    let path = repo_root().join(rel);
    std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

const CANONICAL: &str = "packages/bee/statusline/statusline-command.sh";
const VENDORED: &str = ".claude/statusline-command.sh";

/// Onboarding R4: the canonical script and an opted-in host's vendored copy
/// must be byte-identical. Editing either alone is drift, and drift here is
/// invisible until a user notices a missing line — the vendored copy is
/// overwritten from canonical on the next apply, so a one-sided fix to the
/// host copy is erased rather than shipped.
#[test]
fn canonical_and_vendored_statusline_are_byte_identical() {
    let canonical = read_bytes(CANONICAL);
    let vendored = read_bytes(VENDORED);
    assert_eq!(
        canonical,
        vendored,
        "{CANONICAL} and {VENDORED} have drifted apart (onboarding R4). \
Edit the CANONICAL file and re-vendor with `bee onboard --repo-root . --apply`; \
a hand-edit of the vendored copy alone is erased by the next apply."
    );
}

#[cfg(unix)]
mod runs {
    use super::*;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Command, Stdio};

    const MARKER: &str = "STUB-USAGE-SEGMENT-9f2a";

    const PAYLOAD: &str = r#"{"cwd":"/tmp/fixture-host","model":{"display_name":"Test Model"},"context_window":{"remaining_percentage":80}}"#;

    /// A fixture host shaped exactly like a vendored one: the REAL canonical
    /// script at `.claude/statusline-command.sh`, nothing else on the path
    /// between it and the binary.
    fn fixture(with_stub: bool) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let claude = dir.path().join(".claude");
        std::fs::create_dir_all(&claude).unwrap();
        std::fs::copy(repo_root().join(CANONICAL), claude.join("statusline-command.sh")).unwrap();
        if with_stub {
            let bin = dir.path().join(".bee").join("bin");
            std::fs::create_dir_all(&bin).unwrap();
            let stub = bin.join("bee");
            std::fs::write(&stub, format!("#!/usr/bin/env bash\ncat >/dev/null\necho '{MARKER}'\n"))
                .unwrap();
            std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        dir
    }

    fn which_jq() -> Option<PathBuf> {
        std::env::var_os("PATH").and_then(|path| {
            std::env::split_paths(&path).map(|p| p.join("jq")).find(|p| p.is_file())
        })
    }

    /// Returns (stdout, exit_ok), or None when the host has no `jq` — the
    /// script's own hard dependency, which it reports and fails open on.
    fn run(dir: &tempfile::TempDir) -> Option<(String, bool)> {
        if which_jq().is_none() {
            eprintln!("statusline_contract: skipped, no jq on this host");
            return None;
        }
        let script = dir.path().join(".claude").join("statusline-command.sh");
        let mut child = Command::new("bash")
            .arg(&script)
            .env("CLAUDE_PROJECT_DIR", dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("cannot spawn bash");
        child.stdin.take().unwrap().write_all(PAYLOAD.as_bytes()).unwrap();
        let out = child.wait_with_output().expect("statusline script did not finish");
        Some((String::from_utf8_lossy(&out.stdout).into_owned(), out.status.success()))
    }

    /// THE BUG, executed rather than described: a host whose only binary is at
    /// `<repo>/.bee/bin/bee` must reach it from `<repo>/.claude/`.
    #[test]
    fn statusline_reaches_the_bee_binary() {
        let dir = fixture(true);
        let Some((stdout, ok)) = run(&dir) else { return };
        assert!(
            stdout.contains(MARKER),
            "the usage segment never reached stdout — the script did not find \
<repo>/.bee/bin/bee from <repo>/.claude/. stdout was:\n{stdout}"
        );
        assert!(ok, "the statusline script must exit 0");
    }

    /// The main-checkout leg, on a REAL linked worktree rather than a mock of
    /// one. A linked worktree materialises tracked files only and the vendored
    /// binary is untracked, so a worktree host has no `.bee/bin/bee` of its
    /// own; `--git-common-dir` names the main repository's `.git`, whose
    /// parent is the checkout that holds the binary. Skipped when the host has
    /// no usable git.
    #[test]
    fn statusline_reaches_the_main_checkout_from_a_linked_worktree() {
        if which_jq().is_none() {
            eprintln!("statusline_contract: skipped, no jq on this host");
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let main = root.path().join("main");
        std::fs::create_dir_all(&main).unwrap();
        let git = |args: &[&str]| {
            Command::new("git")
                .args(["-c", "user.email=t@example.invalid", "-c", "user.name=t"])
                .args(["-c", "commit.gpgsign=false"])
                .args(args)
                .current_dir(&main)
                .output()
        };
        let Ok(init) = git(&["init", "-b", "main", "."]) else {
            eprintln!("statusline_contract: skipped, no git on this host");
            return;
        };
        assert!(init.status.success(), "git init failed: {init:?}");
        let commit = git(&["commit", "--allow-empty", "-m", "root"]).unwrap();
        assert!(commit.status.success(), "git commit failed: {commit:?}");

        // The binary lives in the MAIN checkout only.
        let bin = main.join(".bee").join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let stub = bin.join("bee");
        std::fs::write(&stub, format!("#!/usr/bin/env bash\ncat >/dev/null\necho '{MARKER}'\n"))
            .unwrap();
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();

        let wt = root.path().join("wt");
        let added = git(&["worktree", "add", wt.to_str().unwrap(), "-b", "side"]).unwrap();
        assert!(added.status.success(), "git worktree add failed: {added:?}");
        let claude = wt.join(".claude");
        std::fs::create_dir_all(&claude).unwrap();
        std::fs::copy(repo_root().join(CANONICAL), claude.join("statusline-command.sh")).unwrap();

        let mut child = Command::new("bash")
            .arg(claude.join("statusline-command.sh"))
            .env("CLAUDE_PROJECT_DIR", &wt)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("cannot spawn bash");
        child.stdin.take().unwrap().write_all(PAYLOAD.as_bytes()).unwrap();
        let out = child.wait_with_output().expect("statusline script did not finish");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains(MARKER),
            "a linked worktree never fell through to the main checkout's binary:\n{stdout}"
        );
        assert!(out.status.success(), "the statusline script must exit 0");
    }

    /// The other half of the same contract: no binary found anywhere still
    /// renders line one and exits 0. A status line is never allowed to be the
    /// reason a prompt fails to render.
    #[test]
    fn statusline_renders_line_one_without_any_binary() {
        let dir = fixture(false);
        let Some((stdout, ok)) = run(&dir) else { return };
        assert!(ok, "a host with no bee binary must still exit 0; stdout was:\n{stdout}");
        assert!(
            stdout.contains("Test Model") && stdout.contains("/tmp/fixture-host"),
            "line one went missing when no binary was found:\n{stdout}"
        );
    }
}
