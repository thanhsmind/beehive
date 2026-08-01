// onboard::migration — the worktree-local coordination migration (msn-18d).
//
// Provenance: onboard_bee.mjs WORKTREE_COORD_STORES (l. 2720), walkJsonFiles
// (l. 2769), canonicalJson (l. 2796), classifyMigrationRecord (l. 2815),
// locateGitRootForMigration (l. 2865), readGitdirFileForMigration (l. 2876),
// resolveWorktreeContextForMigration (l. 2887), detectWorktreeMigration
// (l. 2921), buildMigrationConflictReason (l. 2950) and
// applyWorktreeMigration (l. 2967).
//
// ALL-OR-NOTHING across the whole migration set: one conflicting record
// anywhere refuses the ENTIRE onboarding run before a byte moves.

use super::util::{exists, join_rel, read_text_if_exists, write_file_atomic};
use crate::jsjson;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// One coordination store: kind, POSIX relDir, the record-shape filter, and
/// the logical id derivation.
struct Store {
    kind: &'static str,
    rel_dir: &'static str,
    matches: fn(&str) -> bool,
    id: fn(&str) -> String,
}

/// `/^[^/]+\.json$/`
fn flat_json(rel: &str) -> bool {
    !rel.contains('/') && rel.ends_with(".json")
}
fn strip_json(rel: &str) -> String {
    rel[..rel.len() - ".json".len()].to_string()
}
/// `/^[^/]+\/state\.json$/`
fn workflow_state(rel: &str) -> bool {
    match rel.split_once('/') {
        Some((head, tail)) => !head.is_empty() && !head.contains('/') && tail == "state.json",
        None => false,
    }
}
fn first_segment(rel: &str) -> String {
    rel.split('/').next().unwrap_or("").to_string()
}
fn always(_: &str) -> bool {
    true
}
fn identity_id(rel: &str) -> String {
    rel.to_string()
}
/// `/^[^/]+\/\d+\.json$/`
fn handoff_record(rel: &str) -> bool {
    match rel.split_once('/') {
        Some((head, tail)) => {
            !head.is_empty()
                && !head.contains('/')
                && tail.ends_with(".json")
                && {
                    let digits = &tail[..tail.len() - ".json".len()];
                    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
                }
        }
        None => false,
    }
}

const STORES: &[Store] = &[
    Store { kind: "session", rel_dir: ".bee/sessions", matches: flat_json, id: strip_json },
    Store { kind: "claim", rel_dir: ".bee/claims", matches: flat_json, id: strip_json },
    Store {
        kind: "workflow",
        rel_dir: ".bee/runtime/workflows",
        matches: workflow_state,
        id: first_segment,
    },
    Store {
        kind: "lease-cell",
        rel_dir: ".bee/runtime/leases/cells",
        matches: flat_json,
        id: strip_json,
    },
    Store {
        kind: "lease-path",
        rel_dir: ".bee/runtime/leases/paths",
        matches: always,
        id: identity_id,
    },
    Store {
        kind: "handoff",
        rel_dir: ".bee/runtime/handoffs",
        matches: handoff_record,
        id: identity_id,
    },
];

/// walkJsonFiles (l. 2769): POSIX-relative *.json paths; a missing or
/// unreadable directory contributes zero files rather than throwing.
fn walk_json_files(root_dir: &Path) -> Vec<String> {
    let mut results = Vec::new();
    let mut stack: Vec<String> = vec![String::new()];
    while let Some(rel_dir) = stack.pop() {
        let dir = if rel_dir.is_empty() { root_dir.to_path_buf() } else { join_rel(root_dir, &rel_dir) };
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let rel_path = if rel_dir.is_empty() { name.clone() } else { format!("{rel_dir}/{name}") };
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                stack.push(rel_path);
            } else if ft.is_file() && name.ends_with(".json") {
                results.push(rel_path);
            }
        }
    }
    results
}

/// canonicalJson (l. 2796): sorted-key serialization used ONLY to compare two
/// records; a migrated record is always written with its original bytes.
fn canonical_json(value: &Value) -> String {
    match value {
        Value::Array(a) => {
            format!("[{}]", a.iter().map(canonical_json).collect::<Vec<_>>().join(","))
        }
        Value::Object(o) => {
            let mut keys: Vec<&String> = o.keys().collect();
            keys.sort();
            format!(
                "{{{}}}",
                keys.iter()
                    .map(|k| format!("{}:{}", jsjson::stringify(&json!(k)), canonical_json(&o[*k])))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        other => jsjson::stringify(other),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordStatus {
    Migrate,
    Duplicate,
    Conflict,
}

#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub kind: &'static str,
    pub path: String,
    pub id: String,
    pub status: RecordStatus,
    pub reason: Option<String>,
    pub local_abs: Option<PathBuf>,
    pub main_abs: Option<PathBuf>,
    pub content: Option<String>,
}

/// classifyMigrationRecord (l. 2815). Divergence note: Node embeds the V8
/// error message for an unreadable local record; the port names the same
/// failure without quoting the interpreter (an unreadable file is a conflict
/// either way, and the campaign rule forbids reproducing V8 text).
fn classify_migration_record(
    workspace_root: &Path,
    main_root: &Path,
    kind: &'static str,
    rel_path: &str,
    id: String,
) -> MigrationRecord {
    let local_abs = join_rel(workspace_root, rel_path);
    let main_abs = join_rel(main_root, rel_path);
    let base = |status: RecordStatus, reason: Option<String>| MigrationRecord {
        kind,
        path: rel_path.to_string(),
        id: id.clone(),
        status,
        reason,
        local_abs: None,
        main_abs: None,
        content: None,
    };
    let Ok(local_raw) = std::fs::read(&local_abs) else {
        return base(RecordStatus::Conflict, Some("local record unreadable".to_string()));
    };
    let local_raw = String::from_utf8_lossy(&local_raw).into_owned();
    let Ok(local_parsed) = serde_json::from_str::<Value>(&local_raw) else {
        return base(
            RecordStatus::Conflict,
            Some("local record is not valid JSON — cannot verify it is safe to migrate".to_string()),
        );
    };
    if !exists(&main_abs) {
        let mut r = base(RecordStatus::Migrate, None);
        r.local_abs = Some(local_abs);
        r.main_abs = Some(main_abs);
        r.content = Some(local_raw);
        return r;
    }
    let main_text = read_text_if_exists(&main_abs);
    let Ok(main_parsed) = serde_json::from_str::<Value>(&main_text) else {
        return base(
            RecordStatus::Conflict,
            Some("main store already has a record at this id but it is not valid JSON".to_string()),
        );
    };
    if canonical_json(&local_parsed) == canonical_json(&main_parsed) {
        let mut r = base(RecordStatus::Duplicate, None);
        r.local_abs = Some(local_abs);
        r.main_abs = Some(main_abs);
        return r;
    }
    base(
        RecordStatus::Conflict,
        Some("main's control store already has a DIFFERENT record under this id".to_string()),
    )
}

// ── the linked-worktree walk-up ────────────────────────────────────────────

fn locate_git_root(start: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut dir = super::util::path_resolve(&start.to_string_lossy());
    loop {
        let marker = dir.join(".git");
        if exists(&marker) {
            return Some((dir, marker));
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p.to_path_buf(),
            _ => return None,
        }
    }
}

/// readGitdirFileForMigration (l. 2876).
fn read_gitdir_file(file: &Path, base: &Path) -> Option<PathBuf> {
    let raw = std::fs::read(file).ok()?;
    let raw = String::from_utf8_lossy(&raw).trim().to_string();
    if raw.is_empty() {
        return None;
    }
    let raw = raw.strip_prefix("gitdir:").map(|r| r.trim().to_string()).unwrap_or(raw);
    let raw = raw.replace('\\', &std::path::MAIN_SEPARATOR.to_string());
    Some(path_resolve_from(base, &raw))
}

fn path_resolve_from(base: &Path, p: &str) -> PathBuf {
    let bytes = p.as_bytes();
    let is_abs = p.starts_with('/')
        || p.starts_with('\\')
        || (bytes.len() >= 2 && bytes[1] == b':');
    if is_abs {
        PathBuf::from(super::util::normalize_lexical(p))
    } else {
        PathBuf::from(super::util::normalize_lexical(&format!(
            "{}{}{}",
            base.to_string_lossy(),
            std::path::MAIN_SEPARATOR,
            p
        )))
    }
}

struct WtContext {
    workspace_root: Option<PathBuf>,
    main_root: Option<PathBuf>,
    worktree_id: Option<String>,
}

/// resolveWorktreeContextForMigration (l. 2887): a minimal, self-contained
/// replica of state.mjs's resolveRootsCore walk-up. Fails open to "ordinary".
fn resolve_worktree_context(start_dir: &Path) -> WtContext {
    let Some((work_root, marker)) = locate_git_root(start_dir) else {
        return WtContext { workspace_root: None, main_root: None, worktree_id: None };
    };
    let ordinary = || WtContext {
        workspace_root: Some(work_root.clone()),
        main_root: Some(work_root.clone()),
        worktree_id: None,
    };
    let Ok(meta) = std::fs::metadata(&marker) else { return ordinary() };
    if !meta.is_file() {
        return ordinary(); // a real .git DIRECTORY: ordinary checkout
    }
    let Some(gitdir) = read_gitdir_file(&marker, &work_root) else { return ordinary() };
    let worktrees_root = path_resolve_from(&gitdir, "..");
    let common_git_dir = path_resolve_from(&worktrees_root, "..");
    if super::util::basename(&common_git_dir) != ".git"
        || super::util::basename(&worktrees_root) != "worktrees"
    {
        return ordinary();
    }
    let id = super::util::basename(&gitdir);
    if id.is_empty() || id == "." || id == ".." {
        return ordinary();
    }
    let Some(reverse) = read_gitdir_file(&gitdir.join("gitdir"), &gitdir) else {
        return ordinary();
    };
    if reverse != super::util::path_resolve(&marker.to_string_lossy()) {
        return ordinary();
    }
    WtContext {
        workspace_root: Some(work_root),
        main_root: common_git_dir.parent().map(Path::to_path_buf),
        worktree_id: Some(id),
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorktreeMigration {
    pub applicable: bool,
    pub records: Vec<MigrationRecord>,
    pub conflicts: Vec<MigrationRecord>,
}

/// detectWorktreeMigration (l. 2921): read-only. `applicable` is false for
/// every ordinary/main/solo checkout (byte-identical, zero behavior change).
pub fn detect_worktree_migration(repo_root: &Path) -> WorktreeMigration {
    let context = resolve_worktree_context(repo_root);
    let (Some(workspace_root), Some(main_root), Some(_)) =
        (&context.workspace_root, &context.main_root, &context.worktree_id)
    else {
        return WorktreeMigration::default();
    };
    if main_root == workspace_root {
        return WorktreeMigration::default();
    }
    let mut records = Vec::new();
    for store in STORES {
        let base_dir = join_rel(workspace_root, store.rel_dir);
        for rel_file in walk_json_files(&base_dir) {
            if !(store.matches)(&rel_file) {
                continue;
            }
            let rel_path = format!("{}/{rel_file}", store.rel_dir);
            let id = (store.id)(&rel_file);
            records.push(classify_migration_record(
                workspace_root,
                main_root,
                store.kind,
                &rel_path,
                id,
            ));
        }
    }
    let conflicts: Vec<MigrationRecord> =
        records.iter().filter(|r| r.status == RecordStatus::Conflict).cloned().collect();
    WorktreeMigration { applicable: true, records, conflicts }
}

/// buildMigrationConflictReason (l. 2950): the loud, exact-list failure.
pub fn build_migration_conflict_reason(conflicts: &[MigrationRecord]) -> String {
    let lines: Vec<String> = conflicts
        .iter()
        .map(|c| {
            format!(
                "  - [{}] {} (id: {}) — {}",
                c.kind,
                c.path,
                c.id,
                c.reason.as_deref().unwrap_or("")
            )
        })
        .collect();
    format!(
        "{} worktree-local coordination record(s) could not be migrated into main's control store — onboarding refused UNTOUCHED, no partial migration:\n{}\nFIX: compare each local copy against main's, keep the correct one, delete the stale one by hand, then re-run onboarding.",
        conflicts.len(),
        lines.join("\n")
    )
}

/// The machine-readable `stranded` array (plan and refusal payloads).
pub fn stranded_json(conflicts: &[MigrationRecord]) -> Value {
    Value::Array(
        conflicts
            .iter()
            .map(|c| {
                json!({
                    "path": c.path,
                    "id": c.id,
                    "kind": c.kind,
                    "reason": c.reason.as_deref().unwrap_or(""),
                })
            })
            .collect(),
    )
}

/// applyWorktreeMigration (l. 2967): the only writer here. Called ONLY after
/// detectWorktreeMigration reported zero conflicts.
pub fn apply_worktree_migration(migration: &WorktreeMigration) {
    for record in &migration.records {
        match record.status {
            RecordStatus::Migrate => {
                if let (Some(main_abs), Some(content), Some(local_abs)) =
                    (&record.main_abs, &record.content, &record.local_abs)
                {
                    let _ = write_file_atomic(main_abs, content.as_bytes());
                    let _ = std::fs::remove_file(local_abs);
                }
            }
            RecordStatus::Duplicate => {
                if let Some(local_abs) = &record.local_abs {
                    let _ = std::fs::remove_file(local_abs);
                }
            }
            RecordStatus::Conflict => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_checkout_is_never_applicable() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let m = detect_worktree_migration(dir.path());
        assert!(!m.applicable);
        assert!(m.records.is_empty());
    }

    #[test]
    fn no_git_root_is_never_applicable() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!detect_worktree_migration(dir.path()).applicable);
    }

    #[test]
    fn store_matchers_mirror_the_regexes() {
        assert!(flat_json("abc.json"));
        assert!(!flat_json("a/b.json"));
        assert!(!flat_json("abc.adopting"));
        assert!(workflow_state("wf/state.json"));
        assert!(!workflow_state("wf/other.json"));
        assert!(handoff_record("mailbox/12.json"));
        assert!(!handoff_record("mailbox/x.json"));
        assert_eq!(strip_json("abc.json"), "abc");
        assert_eq!(first_segment("wf/state.json"), "wf");
    }

    #[test]
    fn canonical_json_sorts_keys_for_comparison_only() {
        let a: Value = serde_json::from_str(r#"{"b":1,"a":[2,{"d":3,"c":4}]}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"a":[2,{"c":4,"d":3}],"b":1}"#).unwrap();
        assert_eq!(canonical_json(&a), canonical_json(&b));
        assert_eq!(canonical_json(&a), r#"{"a":[2,{"c":4,"d":3}],"b":1}"#);
    }

    #[test]
    fn conflict_reason_names_every_stranded_record() {
        let c = MigrationRecord {
            kind: "claim",
            path: ".bee/claims/x.json".into(),
            id: "x".into(),
            status: RecordStatus::Conflict,
            reason: Some("boom".into()),
            local_abs: None,
            main_abs: None,
            content: None,
        };
        let text = build_migration_conflict_reason(&[c]);
        assert!(text.starts_with("1 worktree-local coordination record(s)"));
        assert!(text.contains("  - [claim] .bee/claims/x.json (id: x) — boom"));
        assert!(text.contains("FIX: compare each local copy"));
    }
}
