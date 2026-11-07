use super::*;
use crate::verbs::reservations::js_trim;
use serde_json::Value;
use std::path::Path;

const RECORD_RELATIVE: &str = ".bee/logs/test-results.json";

fn is_bookkeeping(path: &str) -> bool {
    path.starts_with(".bee/") || path.starts_with("docs/") || path.ends_with(".md")
}

fn git_ok(cwd: &Path, args: &[&str]) -> Option<String> {
    let out = run_git(cwd, args);
    if out.status != Some(0) {
        return None;
    }
    Some(out.stdout.unwrap_or_default())
}

fn changed_between(main_root: &Path, from: &str, to: &str) -> Option<Vec<String>> {
    let raw = git_ok(main_root, &["diff", "--no-renames", "--name-only", "-z", from, to])?;
    Some(raw.split('\0').filter(|f| !f.is_empty()).map(str::to_string).collect())
}

fn first_code_path(paths: &[String]) -> Option<&str> {
    paths.iter().map(String::as_str).find(|f| !is_bookkeeping(f))
}

fn verdict(
    main_root: &Path,
    worktree_root: &Path,
    declared: &[String],
    worktree_head: &str,
) -> Option<(&'static str, String)> {
    let record: Value = match std::fs::read_to_string(worktree_root.join(RECORD_RELATIVE))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
    {
        Some(v) => v,
        None => {
            return Some((
                "WORKTREE_MERGE_FULL_SUITE_MISSING",
                format!("has no readable bee test record at {RECORD_RELATIVE}"),
            ))
        }
    };
    if record.get("green") != Some(&Value::Bool(true)) {
        return Some((
            "WORKTREE_MERGE_FULL_SUITE_RED",
            "has a bee test record that is not green".to_string(),
        ));
    }
    let recorded: Vec<&str> = record
        .get("commands")
        .and_then(Value::as_array)
        .map(|c| c.iter().filter_map(|e| e.get("command").and_then(Value::as_str)).collect())
        .unwrap_or_default();
    if recorded != declared.iter().map(String::as_str).collect::<Vec<_>>() {
        return Some((
            "WORKTREE_MERGE_FULL_SUITE_STALE",
            "has a bee test record taken for a different commands.test".to_string(),
        ));
    }
    let head = match record.get("head").and_then(Value::as_str).filter(|h| !h.is_empty()) {
        Some(h) => h,
        None => {
            return Some((
                "WORKTREE_MERGE_FULL_SUITE_STALE",
                "has a bee test record that names no commit".to_string(),
            ))
        }
    };
    match changed_between(main_root, head, worktree_head) {
        Some(paths) => first_code_path(&paths).map(|path| {
            (
                "WORKTREE_MERGE_FULL_SUITE_STALE",
                format!("has a bee test record older than a later code change ({path})"),
            )
        }),
        None => Some((
            "WORKTREE_MERGE_FULL_SUITE_STALE",
            format!("has a bee test record at {head}, which this worktree's history does not reach"),
        )),
    }
}

pub(crate) fn check(main_root: &Path, id: &str, waiver: Option<&str>) -> Result<(), String> {
    let Some(declared) = crate::verbs::test_runner::declared_test_commands(main_root) else {
        return Ok(());
    };
    let Some(worktree_root) = resolve_worktree_by_id(main_root, id).filter(|r| r.exists()) else {
        return Ok(());
    };
    let Some(worktree_head) = git_ok(&worktree_root, &["rev-parse", "HEAD"]).map(|h| js_trim(&h).to_string()) else {
        return Ok(());
    };
    let Some(base) = git_ok(main_root, &["merge-base", "HEAD", &worktree_head]).map(|b| js_trim(&b).to_string()) else {
        return Ok(());
    };
    let Some(feature_paths) = changed_between(main_root, &base, &worktree_head) else {
        return Ok(());
    };
    let Some(code_path) = first_code_path(&feature_paths) else {
        return Ok(());
    };
    let Some((code, reason)) = verdict(main_root, &worktree_root, &declared, &worktree_head) else {
        return Ok(());
    };
    let feature = resolve_worktree_feature(&worktree_root)
        .feature
        .unwrap_or_else(|| "(unresolved)".to_string());
    if let Some(reason_given) = waiver {
        crate::audit::append(
            main_root,
            "merge",
            &feature,
            None,
            &format!("full-suite waiver: {reason_given}"),
            "worktree merge",
            None,
        )
        .map_err(|e| format!("could not write the merge audit row: {e}"))?;
        return Ok(());
    }
    Err(format!(
        "[{code}] worktree {id}'s feature \"{feature}\" changes code ({code_path}) and {reason} — the full suite runs once per feature, at its end, and \"bee worktree merge\" refuses until that run is recorded green on this code. FIX: run bee test in {}, then merge again (or pass --full-suite-waiver \"<reason>\")",
        p(&worktree_root)
    ))
}
