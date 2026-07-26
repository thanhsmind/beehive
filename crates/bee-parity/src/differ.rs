//! Store-tree + stdout/exit differ (CONTEXT.md D7a, validation decision
//! W7 + advisor note 3).
//!
//! Diff policy, exactly:
//! - whole-path exclusion set = `{.bee/logs/**, .bee/cache/**, .bee/tmp/**}`
//!   — files under these are skipped entirely (not even existence-compared);
//!   this is the fail-open telemetry carve-out, not a normalization.
//! - every other file is diffed after [`crate::normalize::normalize`] is
//!   applied to its content with each SIDE's own root (so `<ROOT>` never
//!   collides between legs).

use std::fs;
use std::path::{Path, PathBuf};

use crate::normalize;

/// The fixed whole-path exclusion set (CONTEXT.md D7a). Paths are relative
/// to a store root, always `.bee`-rooted.
///
/// `.bee/runtime` joined the set in rust-port-15, under the locked D3
/// addendum (decision a7d7b3d5): the gix review derivation writes ONE
/// additive runtime artifact, `.bee/runtime/review-git-cache.json`, which
/// the frozen mjs leg never reads or writes. It is a whole-path EXCLUSION,
/// deliberately not a normalization — nothing about its content is
/// rewritten to make two legs agree; the tree comparison simply does not
/// consider a directory only one runtime is contractually allowed to
/// touch. Same fail-open carve-out shape as logs/cache/tmp.
const EXCLUDED_PREFIXES: [&str; 4] = [".bee/logs", ".bee/cache", ".bee/tmp", ".bee/runtime"];

fn is_excluded(rel: &Path) -> bool {
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    EXCLUDED_PREFIXES
        .iter()
        .any(|p| rel_str == *p || rel_str.starts_with(&format!("{p}/")))
}

/// One captured leg: the run's own root, its stdout, and its exit code.
#[derive(Debug, Clone)]
pub struct RunResult {
    pub root: PathBuf,
    pub stdout: String,
    pub exit_code: i32,
}

#[derive(Debug, Default)]
pub struct DiffReport {
    pub exit_diff: Option<String>,
    pub stdout_diff: Option<String>,
    pub tree_diffs: Vec<String>,
}

impl DiffReport {
    pub fn is_clean(&self) -> bool {
        self.exit_diff.is_none() && self.stdout_diff.is_none() && self.tree_diffs.is_empty()
    }

    pub fn describe(&self) -> String {
        if self.is_clean() {
            return "zero diff".to_string();
        }
        let mut parts = Vec::new();
        if let Some(e) = &self.exit_diff {
            parts.push(e.clone());
        }
        if let Some(s) = &self.stdout_diff {
            parts.push(s.clone());
        }
        parts.extend(self.tree_diffs.iter().cloned());
        parts.join(" | ")
    }
}

/// Diff two legs: stdout (normalized per-leg), exit code, and the post-run
/// store tree under each root (excluding the fixed exclusion set,
/// normalized per-leg before comparison).
pub fn diff_legs(a: &RunResult, b: &RunResult) -> Result<DiffReport, String> {
    let mut report = DiffReport::default();

    if a.exit_code != b.exit_code {
        report.exit_diff = Some(format!(
            "exit code differs: leg A ({}) = {}, leg B ({}) = {}",
            a.root.display(),
            a.exit_code,
            b.root.display(),
            b.exit_code
        ));
    }

    let a_root_str = a.root.display().to_string();
    let b_root_str = b.root.display().to_string();
    let a_stdout_norm = normalize::normalize(&a.stdout, &a_root_str);
    let b_stdout_norm = normalize::normalize(&b.stdout, &b_root_str);
    if a_stdout_norm != b_stdout_norm {
        report.stdout_diff = Some(format!(
            "stdout differs after normalization:\n--- leg A ({})\n{}\n--- leg B ({})\n{}",
            a.root.display(),
            a_stdout_norm,
            b.root.display(),
            b_stdout_norm
        ));
    }

    report.tree_diffs = diff_trees(&a.root, &b.root)?;
    Ok(report)
}

/// Collect every relative path under `root` (rooted at `root` itself, not
/// just `.bee`), skipping the fixed exclusion set entirely.
fn collect_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    if root.exists() {
        walk(root, root, &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir entry under {}: {e}", dir.display()))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|e| format!("strip_prefix {}: {e}", path.display()))?
            .to_path_buf();
        if is_excluded(&rel) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| format!("file_type {}: {e}", path.display()))?;
        if file_type.is_dir() {
            walk(root, &path, out)?;
        } else if file_type.is_file() {
            out.push(rel);
        }
        // symlinks: not part of any store contract here; skip silently
        // rather than following (avoids cycles from a stray link).
    }
    Ok(())
}

fn diff_trees(root_a: &Path, root_b: &Path) -> Result<Vec<String>, String> {
    let files_a = collect_files(root_a)?;
    let files_b = collect_files(root_b)?;

    let mut diffs = Vec::new();

    for rel in &files_a {
        if !files_b.contains(rel) {
            diffs.push(format!("{} present under leg A but missing under leg B", rel.display()));
        }
    }
    for rel in &files_b {
        if !files_a.contains(rel) {
            diffs.push(format!("{} present under leg B but missing under leg A", rel.display()));
        }
    }

    let a_root_str = root_a.display().to_string();
    let b_root_str = root_b.display().to_string();
    for rel in &files_a {
        if !files_b.contains(rel) {
            continue;
        }
        let path_a = root_a.join(rel);
        let path_b = root_b.join(rel);
        let content_a = fs::read(&path_a).map_err(|e| format!("read {}: {e}", path_a.display()))?;
        let content_b = fs::read(&path_b).map_err(|e| format!("read {}: {e}", path_b.display()))?;

        match (String::from_utf8(content_a.clone()), String::from_utf8(content_b.clone())) {
            (Ok(text_a), Ok(text_b)) => {
                let norm_a = normalize::normalize(&text_a, &a_root_str);
                let norm_b = normalize::normalize(&text_b, &b_root_str);
                if norm_a != norm_b {
                    diffs.push(format!("{} differs after normalization", rel.display()));
                }
            }
            _ => {
                // Binary file: no text normalization applies (root-path
                // rewriting is a text-level allowlist member); compare
                // raw bytes so a real difference still surfaces.
                if content_a != content_b {
                    diffs.push(format!("{} differs (binary compare)", rel.display()));
                }
            }
        }
    }

    Ok(diffs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_logs_cache_tmp_runtime_prefixes() {
        assert!(is_excluded(Path::new(".bee/logs/hooks.jsonl")));
        assert!(is_excluded(Path::new(".bee/cache/inject-cache.json")));
        assert!(is_excluded(Path::new(".bee/tmp/scratch.txt")));
        // D3 addendum a7d7b3d5 — the rust-only review-git cache.
        assert!(is_excluded(Path::new(".bee/runtime/review-git-cache.json")));
        assert!(!is_excluded(Path::new(".bee/state.json")));
        assert!(!is_excluded(Path::new(".bee/cells/fixture-00001.json")));
    }

    #[test]
    fn exclusion_is_prefix_scoped_not_substring() {
        // A sibling directory that merely starts with the same characters
        // (".bee/logsomething") must NOT be excluded — prefix match is
        // segment-scoped (checks for a trailing '/'), not a bare
        // `starts_with` on the raw string.
        assert!(!is_excluded(Path::new(".bee/logsomething/file.json")));
    }
}
