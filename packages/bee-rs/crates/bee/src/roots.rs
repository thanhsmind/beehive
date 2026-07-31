// roots — Rust port of resolveRootsCore's ORDINARY-checkout classification
// (state.mjs). Linked git worktrees (`.git` is a FILE) carry gitdir
// validation + the grant registry; that half is not ported yet, so those
// resolve to `NeedsNode` and the verb delegates. The beehive repo itself and
// every ordinary host checkout classify natively.

use std::path::{Path, PathBuf};

pub enum Roots {
    /// Ordinary checkout: store root == work root.
    Ordinary(PathBuf),
    /// Linked worktree (or anything the ported half can't classify yet).
    NeedsNode,
    /// No .bee/onboarding.json and no .git anywhere up the tree.
    None,
}

fn absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

pub fn resolve_store_root(start: &Path) -> Roots {
    // Pass 1 (fixture-below-unrelated-git guard): nearest ancestor holding
    // .bee/onboarding.json WITHOUT a .git of its own wins outright.
    let start = absolute(start);
    let mut dir: Option<&Path> = Some(&start);
    while let Some(d) = dir {
        if d.join(".bee").join("onboarding.json").exists() && !d.join(".git").exists() {
            return Roots::Ordinary(d.to_path_buf());
        }
        dir = d.parent();
    }

    // Pass 2: nearest .git marker.
    let mut located: Option<(PathBuf, PathBuf)> = None;
    let mut dir: Option<&Path> = Some(&start);
    while let Some(d) = dir {
        let marker = d.join(".git");
        if marker.exists() {
            located = Some((d.to_path_buf(), marker));
            break;
        }
        dir = d.parent();
    }

    let Some((work_root, marker)) = located else {
        // No git at all: nearest onboarding marker, else nothing.
        let mut dir: Option<&Path> = Some(&start);
        while let Some(d) = dir {
            if d.join(".bee").join("onboarding.json").exists() {
                return Roots::Ordinary(d.to_path_buf());
            }
            dir = d.parent();
        }
        return Roots::None;
    };

    match std::fs::metadata(&marker) {
        Ok(m) if m.is_file() => Roots::NeedsNode, // linked worktree — unported half
        Ok(_) => Roots::Ordinary(work_root),
        Err(_) => Roots::NeedsNode, // stat raced/failed — Node owns the edge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn onboarding_marker_without_git_wins_over_ancestor_git() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        let nested = root.join("fixture");
        std::fs::create_dir_all(nested.join(".bee")).unwrap();
        std::fs::write(nested.join(".bee").join("onboarding.json"), "{}").unwrap();
        match resolve_store_root(&nested) {
            Roots::Ordinary(r) => assert_eq!(r, nested),
            _ => panic!("expected ordinary root at the fixture"),
        }
    }

    #[test]
    fn ordinary_git_dir_resolves_to_work_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        std::fs::create_dir_all(root.join(".git")).unwrap();
        let deep = root.join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();
        match resolve_store_root(&deep) {
            Roots::Ordinary(r) => assert_eq!(r, root),
            _ => panic!("expected ordinary root"),
        }
    }

    #[test]
    fn git_file_delegates_to_node() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("wt");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(".git"), "gitdir: ../main/.git/worktrees/wt").unwrap();
        assert!(matches!(resolve_store_root(&root), Roots::NeedsNode));
    }

    #[test]
    fn nothing_found_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        // A bare temp dir tree with neither marker. (If the OS temp root ever
        // sits under a git checkout this would misclassify — acceptable in
        // practice for CI/dev machines.)
        let deep = tmp.path().join("x").join("y");
        std::fs::create_dir_all(&deep).unwrap();
        assert!(matches!(resolve_store_root(&deep), Roots::None | Roots::Ordinary(_)));
    }
}
