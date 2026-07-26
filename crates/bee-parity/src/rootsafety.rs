//! ROOT-RESOLUTION SAFETY (validation decision B5, CONTEXT.md D7a/D3).
//!
//! Every temp root this harness touches MUST be a self-sufficient bee root
//! resolvable purely by `resolveRootsCore`'s "ordinary" first branch
//! (`.bee/bin/lib/state.mjs:760`): a directory with `.bee/onboarding.json`
//! present AND no `.git` at that same directory. That branch returns on its
//! FIRST loop iteration when both hold at the starting directory (cwd) — so
//! by construction, invoking `bee.mjs` with `cwd` set exactly to such a
//! directory resolves `storeRoot == cwd`, with no dependency on what lives
//! above it in the filesystem.
//!
//! This module asserts that construction holds (structural check) and then
//! empirically proves the store actually read back is THIS root's own
//! content, not some other store the process might have fallen back to
//! (content-signature check) — belt and suspenders against the exact
//! failure class B5 exists to prevent: silently reading or writing the
//! repo's live `.bee/`.

use std::path::Path;

use crate::differ::RunResult;

/// Structural half of the B5 assertion: onboarding marker present, no
/// `.git` at the root itself, and the root lives outside the repo's own
/// working tree (never a subdirectory of `repo_root`).
pub fn assert_structural_safety(repo_root: &Path, root: &Path) -> Result<(), String> {
    let onboarding = root.join(".bee").join("onboarding.json");
    if !onboarding.exists() {
        return Err(format!(
            "root-safety: {} is missing .bee/onboarding.json — not a self-sufficient bee root",
            root.display()
        ));
    }
    let dot_git = root.join(".git");
    if dot_git.exists() {
        return Err(format!(
            "root-safety: {} contains a .git — resolveRootsCore's ordinary branch would not match here",
            root.display()
        ));
    }
    let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let repo_canon = repo_root.canonicalize().unwrap_or_else(|_| repo_root.to_path_buf());
    if root_canon.starts_with(&repo_canon) {
        return Err(format!(
            "root-safety: {} is inside the repo working tree {} — temp roots must live outside it",
            root.display(),
            repo_root.display()
        ));
    }
    Ok(())
}

/// Empirical half: given the [`RunResult`] of a REAL `bee.mjs status
/// --json` run with `cwd` set to `root` (produced by
/// [`crate::runner::run_bee_status`]), assert the output is not just
/// well-formed status JSON but carries the exact signature this crate's
/// fixtures are generated with (`phase: "idle"`, `feature: null`,
/// `workers: []` — see `queen-bench::fixture::write_state`, mirrored in
/// `crate::mutate::FIXTURE_STATE_BODY`). A resolution that silently fell
/// back to a different store (e.g. the live repo's) would show a different
/// phase/feature/workers shape, so this doubles as the "resolved store
/// root equals the temp root" proof the cell requires before every command
/// — not just a crash check. Takes the run rather than spawning its own,
/// so the sanity read and the actual diffed run are never two different
/// subprocess calls that could disagree.
pub fn assert_resolves_to_fixture(run: &RunResult) -> Result<(), String> {
    if run.exit_code != 0 {
        return Err(format!(
            "root-safety: bee.mjs status --json exited {} at {}",
            run.exit_code,
            run.root.display()
        ));
    }
    let trimmed = run.stdout.trim_start();
    if !trimmed.starts_with('{') {
        return Err(format!(
            "root-safety: status --json at {} did not produce a JSON object (first 200 chars): {}",
            run.root.display(),
            &run.stdout[..run.stdout.len().min(200)]
        ));
    }
    // bee.mjs pretty-prints `status --json` (space after `:`, newlines,
    // 2-space indent), so the fixture-signature markers below are matched
    // against a whitespace-stripped copy — used ONLY for this diagnostic
    // presence check, never for the actual diff comparison (that stays in
    // `differ`/`normalize`, on the real, unstripped content).
    let compact: String = run.stdout.chars().filter(|c| !c.is_whitespace()).collect();
    for marker in ["\"phase\":\"idle\"", "\"feature\":null", "\"workers\":[]"] {
        if !compact.contains(marker) {
            return Err(format!(
                "root-safety: status --json at {} is missing fixture signature `{}` — resolution may have fallen back to a different store than the intended temp root (first 400 chars): {}",
                run.root.display(),
                marker,
                &run.stdout[..run.stdout.len().min(400)]
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rejects_missing_onboarding_marker() {
        let base = std::env::temp_dir().join(format!("bee-parity-rootsafety-test-a-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let repo_root = std::env::temp_dir().join("bee-parity-unused-repo-root");
        let err = assert_structural_safety(&repo_root, &base).unwrap_err();
        assert!(err.contains("onboarding.json"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn rejects_dot_git_at_root() {
        let base = std::env::temp_dir().join(format!("bee-parity-rootsafety-test-b-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join(".bee")).unwrap();
        fs::write(base.join(".bee").join("onboarding.json"), b"{}").unwrap();
        fs::create_dir_all(base.join(".git")).unwrap();
        let repo_root = std::env::temp_dir().join("bee-parity-unused-repo-root");
        let err = assert_structural_safety(&repo_root, &base).unwrap_err();
        assert!(err.contains(".git"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn rejects_root_inside_repo_tree() {
        let repo_root = std::env::temp_dir().join(format!("bee-parity-rootsafety-repo-{}", std::process::id()));
        let inside = repo_root.join("nested-root");
        let _ = fs::remove_dir_all(&repo_root);
        fs::create_dir_all(inside.join(".bee")).unwrap();
        fs::write(inside.join(".bee").join("onboarding.json"), b"{}").unwrap();
        let err = assert_structural_safety(&repo_root, &inside).unwrap_err();
        assert!(err.contains("inside the repo working tree"));
        let _ = fs::remove_dir_all(&repo_root);
    }

    #[test]
    fn accepts_a_valid_self_sufficient_root() {
        let repo_root = std::env::temp_dir().join("bee-parity-unused-repo-root-2");
        let base = std::env::temp_dir().join(format!("bee-parity-rootsafety-test-c-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join(".bee")).unwrap();
        fs::write(base.join(".bee").join("onboarding.json"), b"{}").unwrap();
        assert!(assert_structural_safety(&repo_root, &base).is_ok());
        let _ = fs::remove_dir_all(&base);
    }
}
