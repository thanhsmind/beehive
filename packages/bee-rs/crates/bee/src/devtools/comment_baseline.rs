use crate::comments;
use crate::fsutil::{self, ReadJson};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

pub(crate) const REMEDY: &str = "bee dev comment-baseline --write lowers a count, never raises \
                                 one. A count that rose from a merge is fixed by deleting the \
                                 merged comment lines.";

pub(crate) type Counts = BTreeMap<String, usize>;

pub(crate) struct Report {
    pub(crate) lines: Vec<String>,
    pub(crate) failed: bool,
}

pub(crate) fn baseline_path(root: &Path) -> PathBuf {
    root.join(".bee").join("comment-baseline.json")
}

pub(crate) fn load_baseline(root: &Path) -> Counts {
    let mut out = Counts::new();
    let ReadJson::Parsed(value) = fsutil::read_json(&baseline_path(root)) else {
        return out;
    };
    let Some(files) = value.get("files").and_then(Value::as_object) else {
        return out;
    };
    for (rel, count) in files {
        if let Some(number) = count.as_u64() {
            out.insert(rel.clone(), number as usize);
        }
    }
    out
}

pub(crate) fn write_baseline(root: &Path, baseline: &Counts) -> std::io::Result<()> {
    let mut files = Map::new();
    for (rel, count) in baseline {
        files.insert(rel.clone(), Value::from(*count as u64));
    }
    let mut doc = Map::new();
    doc.insert("files".to_string(), Value::Object(files));
    fsutil::write_json_atomic(&baseline_path(root), &Value::Object(doc))
}

pub(crate) fn current_counts(root: &Path) -> Counts {
    let mut out = Counts::new();
    for (rel, lang) in comments::walk_code_files(root) {
        let Ok(text) = std::fs::read_to_string(root.join(&rel)) else {
            continue;
        };
        out.insert(rel, comments::count_comment_lines(lang, &text));
    }
    out
}

pub(crate) fn over_baseline(counts: &Counts, baseline: &Counts) -> Vec<String> {
    let mut out = Vec::new();
    for (rel, count) in counts {
        let base = baseline.get(rel).copied().unwrap_or(0);
        if *count > base {
            out.push(format!("{rel}: {count} comment line(s), baseline {base}"));
        }
    }
    out
}

struct Live {
    counts: Counts,
    branches: Vec<String>,
    gap: Option<String>,
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").arg("-C").arg(root).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

fn unmerged_branches(root: &Path) -> Option<Vec<String>> {
    let current = git(root, &["rev-parse", "--abbrev-ref", "HEAD"])?.trim().to_string();
    let listed = git(root, &["branch", "--no-merged", "main", "--format=%(refname:short)"])?;
    Some(
        listed
            .lines()
            .map(str::trim)
            .filter(|branch| branch.starts_with("wt/") && *branch != current)
            .map(str::to_string)
            .collect(),
    )
}

fn branch_counts(root: &Path, branch: &str) -> Option<Counts> {
    let listing = git(root, &["ls-tree", "-r", "--name-only", branch])?;
    let mut out = Counts::new();
    for rel in listing.lines() {
        let rel = rel.trim();
        if rel.is_empty() || !comments::under_code_root(rel) {
            continue;
        }
        let by_name = comments::code_lang(rel, "");
        let name = rel.rsplit('/').next().unwrap_or(rel);
        if by_name.is_none() && name.contains('.') {
            continue;
        }
        let text = git(root, &["show", &format!("{branch}:{rel}")])?;
        let lang = match by_name {
            Some(lang) => lang,
            None => match comments::code_lang(rel, text.lines().next().unwrap_or("")) {
                Some(lang) => lang,
                None => continue,
            },
        };
        out.insert(rel.to_string(), comments::count_comment_lines(lang, &text));
    }
    Some(out)
}

fn live_counts(root: &Path) -> Live {
    let tree = current_counts(root);
    let Some(branches) = unmerged_branches(root) else {
        return Live {
            counts: tree,
            branches: Vec::new(),
            gap: Some("git could not list the unmerged branches".to_string()),
        };
    };
    let mut counts = tree.clone();
    for branch in &branches {
        let Some(other) = branch_counts(root, branch) else {
            return Live {
                counts: tree,
                branches: Vec::new(),
                gap: Some(format!("branch {branch} could not be read")),
            };
        };
        for (rel, count) in other {
            let slot = counts.entry(rel).or_insert(0);
            if count > *slot {
                *slot = count;
            }
        }
    }
    Live { counts, branches, gap: None }
}

pub(crate) fn check(root: &Path) -> Report {
    let counts = current_counts(root);
    let baseline = load_baseline(root);
    let mut lines = over_baseline(&counts, &baseline);
    if lines.is_empty() {
        let total = counts.len();
        return Report {
            lines: vec![format!("comment-baseline --check: {total} file(s) at or below baseline")],
            failed: false,
        };
    }
    lines.push(REMEDY.to_string());
    Report { lines, failed: true }
}

pub(crate) fn write(root: &Path) -> Report {
    let seeding = !baseline_path(root).is_file();
    let baseline = load_baseline(root);
    let live = live_counts(root);
    if !seeding {
        let mut lines = over_baseline(&live.counts, &baseline);
        if !lines.is_empty() {
            lines.push(REMEDY.to_string());
            return Report { lines, failed: true };
        }
    }
    if let Err(error) = write_baseline(root, &live.counts) {
        let path = baseline_path(root).display().to_string();
        return Report {
            lines: vec![format!("comment-baseline --write: could not write {path}: {error}")],
            failed: true,
        };
    }
    let total = live.counts.len();
    let mut lines = Vec::new();
    if seeding {
        match &live.gap {
            Some(gap) => lines.push(format!(
                "comment-baseline --write: seeded {total} file(s) from the working tree only — \
                 {gap}"
            )),
            None => {
                let count = live.branches.len();
                let names = if live.branches.is_empty() {
                    "none".to_string()
                } else {
                    live.branches.join(", ")
                };
                lines.push(format!(
                    "comment-baseline --write: seeded {total} file(s) from main plus {count} \
                     unmerged branch(es) ({names})"
                ));
            }
        }
        return Report { lines, failed: false };
    }
    let mut lowered = 0usize;
    let mut dropped = 0usize;
    for (rel, base) in &baseline {
        match live.counts.get(rel) {
            Some(count) if count < base => lowered += 1,
            Some(_) => {}
            None => dropped += 1,
        }
    }
    lines.push(format!(
        "comment-baseline --write: {total} file(s) recorded, {lowered} lowered, {dropped} dropped"
    ));
    if let Some(gap) = &live.gap {
        lines.push(format!(
            "comment-baseline --write: the unmerged branches were not folded in — {gap}"
        ));
    }
    Report { lines, failed: false }
}

pub(super) fn run(args: &[&str]) -> Option<ExitCode> {
    let root = super::bee_source_root()?;
    let flags: Vec<&str> =
        ["--check", "--write"].into_iter().filter(|flag| args.contains(flag)).collect();
    if flags.len() != 1 {
        eprintln!("usage: bee dev comment-baseline (--check | --write)");
        return Some(ExitCode::FAILURE);
    }
    let report = if flags[0] == "--check" { check(&root) } else { write(&root) };
    for line in &report.lines {
        if report.failed {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }
    Some(if report.failed { ExitCode::FAILURE } else { ExitCode::SUCCESS })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_git(root: &Path, args: &[&str]) {
        let out = Command::new("git").arg("-C").arg(root).args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    fn write_file(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn fixture() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        run_git(root, &["init", "-q", "-b", "main"]);
        run_git(root, &["config", "user.email", "ratchet@example.com"]);
        run_git(root, &["config", "user.name", "ratchet"]);
        run_git(root, &["config", "commit.gpgsign", "false"]);
        write_file(root, "scripts/a.sh", "echo one\n");
        run_git(root, &["add", "-A"]);
        run_git(root, &["commit", "-qm", "base"]);
        tmp
    }

    fn seed_at(root: &Path, body: &str) {
        write_file(root, "scripts/a.sh", body);
        let report = write(root);
        assert!(!report.failed, "seed refused: {:?}", report.lines);
    }

    #[test]
    fn a_seed_holds_the_higher_count_an_unmerged_branch_carries() {
        let tmp = fixture();
        let root = tmp.path();
        run_git(root, &["checkout", "-q", "-b", "wt/sibling"]);
        write_file(root, "scripts/a.sh", "# one\n# two\necho one\n");
        run_git(root, &["commit", "-qam", "sibling"]);
        run_git(root, &["checkout", "-q", "main"]);

        let report = write(root);
        assert!(!report.failed, "{:?}", report.lines);
        assert_eq!(
            report.lines,
            vec![
                "comment-baseline --write: seeded 1 file(s) from main plus 1 unmerged branch(es) \
                 (wt/sibling)"
                    .to_string()
            ]
        );
        assert_eq!(load_baseline(root).get("scripts/a.sh").copied(), Some(2));
        assert!(!check(root).failed);
    }

    #[test]
    fn a_still_unmerged_branch_keeps_its_headroom_on_a_later_write() {
        let tmp = fixture();
        let root = tmp.path();
        run_git(root, &["checkout", "-q", "-b", "wt/sibling"]);
        write_file(root, "scripts/a.sh", "# one\n# two\necho one\n");
        run_git(root, &["commit", "-qam", "sibling"]);
        run_git(root, &["checkout", "-q", "main"]);
        assert!(!write(root).failed);

        let report = write(root);
        assert!(!report.failed, "{:?}", report.lines);
        assert_eq!(load_baseline(root).get("scripts/a.sh").copied(), Some(2));
    }

    #[test]
    fn write_lowers_a_count_that_fell_everywhere() {
        let tmp = fixture();
        let root = tmp.path();
        seed_at(root, "# one\n# two\necho one\n");
        assert_eq!(load_baseline(root).get("scripts/a.sh").copied(), Some(2));

        write_file(root, "scripts/a.sh", "# one\necho one\n");
        let report = write(root);
        assert!(!report.failed, "{:?}", report.lines);
        assert_eq!(
            report.lines,
            vec!["comment-baseline --write: 1 file(s) recorded, 1 lowered, 0 dropped".to_string()]
        );
        assert_eq!(load_baseline(root).get("scripts/a.sh").copied(), Some(1));
    }

    #[test]
    fn write_refuses_to_raise_a_count_and_names_the_file() {
        let tmp = fixture();
        let root = tmp.path();
        seed_at(root, "# one\necho one\n");

        write_file(root, "scripts/a.sh", "# one\n# two\n# three\necho one\n");
        let report = write(root);
        assert!(report.failed, "{:?}", report.lines);
        assert_eq!(
            report.lines,
            vec![
                "scripts/a.sh: 3 comment line(s), baseline 1".to_string(),
                REMEDY.to_string(),
            ]
        );
        assert_eq!(load_baseline(root).get("scripts/a.sh").copied(), Some(1));
    }

    #[test]
    fn write_drops_the_entry_of_a_file_that_no_longer_exists() {
        let tmp = fixture();
        let root = tmp.path();
        write_file(root, "scripts/b.sh", "# gone soon\necho two\n");
        seed_at(root, "# one\necho one\n");
        assert_eq!(load_baseline(root).get("scripts/b.sh").copied(), Some(1));

        std::fs::remove_file(root.join("scripts/b.sh")).unwrap();
        let report = write(root);
        assert!(!report.failed, "{:?}", report.lines);
        assert_eq!(
            report.lines,
            vec!["comment-baseline --write: 1 file(s) recorded, 0 lowered, 1 dropped".to_string()]
        );
        assert!(!load_baseline(root).contains_key("scripts/b.sh"));
    }

    #[test]
    fn baseline_keys_are_slash_separated_on_every_platform() {
        let tmp = fixture();
        let root = tmp.path();
        write_file(root, "scripts/sub/deep/c.py", "# note\nx = 1\n");
        seed_at(root, "echo one\n");

        let baseline = load_baseline(root);
        assert_eq!(baseline.get("scripts/sub/deep/c.py").copied(), Some(1));
        assert!(baseline.keys().all(|rel| !rel.contains('\\')), "{baseline:?}");
    }

    #[test]
    fn check_names_the_file_the_count_the_baseline_and_both_remedy_sentences() {
        let tmp = fixture();
        let root = tmp.path();
        seed_at(root, "# one\necho one\n");
        assert_eq!(
            check(root).lines,
            vec!["comment-baseline --check: 1 file(s) at or below baseline".to_string()]
        );

        write_file(root, "scripts/a.sh", "# one\n# two\necho one\n");
        let report = check(root);
        assert!(report.failed, "{:?}", report.lines);
        assert_eq!(
            report.lines,
            vec!["scripts/a.sh: 2 comment line(s), baseline 1".to_string(), REMEDY.to_string()]
        );
        assert!(REMEDY.contains("lowers a count, never raises one."), "{REMEDY}");
        assert!(REMEDY.contains("fixed by deleting the merged comment lines."), "{REMEDY}");
    }

    #[test]
    fn a_new_code_file_with_a_comment_is_a_rise_over_an_absent_entry() {
        let tmp = fixture();
        let root = tmp.path();
        seed_at(root, "echo one\n");

        write_file(root, "scripts/fresh.sh", "# new why\necho fresh\n");
        let report = check(root);
        assert!(report.failed, "{:?}", report.lines);
        assert_eq!(report.lines[0], "scripts/fresh.sh: 1 comment line(s), baseline 0");
    }

    #[test]
    fn every_code_file_is_at_or_below_its_comment_baseline() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf();
        let counts = current_counts(&root);
        assert!(
            !counts.is_empty(),
            "the comment ratchet walked {} and found no code file — the repo root moved and this \
             test is now vacuous",
            root.display()
        );
        let over = over_baseline(&counts, &load_baseline(&root));
        assert!(over.is_empty(), "{}\n{REMEDY}", over.join("\n"));
    }
}
