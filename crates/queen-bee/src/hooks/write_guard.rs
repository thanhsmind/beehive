//! write_guard — Rust port of `.bee/bin/hooks/bee-write-guard.mjs`'s CORE
//! write-check spine (rust-port-9, CONTEXT.md D2/D7): the
//! Edit/Write/MultiEdit tool path — worktree containment (linked-invalid
//! exit 2, canonical rel-path resolution, companion-mount recognition,
//! sibling-worktree denial enrichment) feeding `bee_core::guards::check_write`
//! (gate/intake, reservations, cross-session and cross-worktree holds,
//! workspace ownership).
//!
//! `.bee/bin/hooks/bee-write-guard.mjs` is FROZEN for the duration of the
//! rust-port feature (D1) — this module is conformance-checked against
//! sha256-verified copies of it (`crates/queen-bee/tests/writeguard_core.rs`),
//! never edited to "improve" on it. Deny reasons are copied VERBATIM so a
//! deny's stderr is byte-identical across runtimes.
//!
//! Scope boundary (validation decision 2026-07-26, split 1 of 3):
//! - Bash-command analysis (`extractBashTargets`, `checkGitBashCommand`,
//!   CLI-shape check (d), internals-reach) is cell rust-port-11. A Bash
//!   payload here passes the linked-invalid gate below, then falls open
//!   (allow) — this port is DARK until the flip slice, so nothing depends
//!   on the missing branch yet.
//! - The read side (`checkRead`, read-size guard, privacy/scout),
//!   `apply_patch` target proving, and `AskUserQuestion` validation are
//!   cell rust-port-12 — same deliberate fall-open until they land.
//!
//! Exit contract (D7b): allow = exit 0 silent (or the allow+notice JSON on
//! stdout for advisory warnings); deliberate deny = exit 2 with the reason
//! on stderr; internal crash = exit 0 plus a `.bee/logs/hooks.jsonl` crash
//! line (fail-open).

use std::fs;
use std::io::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Component, Path, PathBuf};

use serde_json::{json, Value};

use bee_core::guards::{check_write, Verdict, WriteTopology};
use bee_core::state::read_state;

use crate::adapter::{self, HookContext};
use crate::hookconfig;

const HOOK_NAME: &str = "write-guard";
const READ_TOOLS: [&str; 3] = ["Read", "Glob", "Grep"];
const WRITE_TOOLS: [&str; 3] = ["Edit", "Write", "MultiEdit"];
const APPLY_PATCH_TOOLS: [&str; 2] = ["apply_patch", "ApplyPatch"];

const GENERIC_CONTAINMENT_MESSAGE: &str =
    "bee write guard denied this target: it could not be canonically contained inside the physical worktree. \
FIX: use a plain in-worktree path without traversal, outside absolute paths, or symlink escapes.";

const WORKTREE_LINK_INVALID_MESSAGE: &str =
    "bee worktree guard denied this write: WORKTREE_LINK_INVALID — linked worktree metadata could not be validated. \
FIX: repair or recreate the Git worktree before retrying; no worktree-local .bee store is trusted.";

// ─── node-path semantics helpers ────────────────────────────────────────────

/// `path.resolve`-shaped lexical normalization (no filesystem access).
fn normalize_lexical(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn resolve_path(base: &Path, rel: &Path) -> PathBuf {
    let joined = if rel.is_absolute() { rel.to_path_buf() } else { base.join(rel) };
    normalize_lexical(&joined)
}

/// node `path.relative(from, to)` for two absolute, lexically-normalized
/// paths: strip the common component prefix, then `..` per remaining `from`
/// component plus the `to` remainder.
fn path_relative(from: &Path, to: &Path) -> PathBuf {
    let from_comps: Vec<_> = from.components().collect();
    let to_comps: Vec<_> = to.components().collect();
    let mut common = 0;
    while common < from_comps.len() && common < to_comps.len() && from_comps[common] == to_comps[common] {
        common += 1;
    }
    let mut out = PathBuf::new();
    for _ in common..from_comps.len() {
        out.push("..");
    }
    for comp in &to_comps[common..] {
        out.push(comp.as_os_str());
    }
    out
}

fn realpath_or_none(value: &Path) -> Option<PathBuf> {
    fs::canonicalize(value).ok()
}

/// bee-write-guard.mjs `normalizeToolPath`: preserve the shell's `\ `
/// escaped-space spelling, but treat every other backslash as a separator
/// (`/\\(?!\s)/g` — replaced unless followed by a whitespace character).
fn normalize_tool_path(raw_path: &str) -> String {
    let chars: Vec<char> = raw_path.chars().collect();
    let mut out = String::with_capacity(raw_path.len());
    for (i, &c) in chars.iter().enumerate() {
        if c == '\\' && !chars.get(i + 1).map(|n| n.is_whitespace()).unwrap_or(false) {
            out.push(std::path::MAIN_SEPARATOR);
        } else {
            out.push(c);
        }
    }
    out
}

/// A foreign Windows absolute/UNC spelling on a POSIX host (`^[A-Za-z]:[\\/]`
/// or `^\\\\`) cannot be safely mapped — mirrors the mjs guard clause.
fn windows_foreign_on_posix(raw_path: &str) -> bool {
    if std::path::MAIN_SEPARATOR == '\\' {
        return false;
    }
    let bytes = raw_path.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
    {
        return true;
    }
    raw_path.starts_with("\\\\")
}

/// Walk up through ENOENT segments to the first existing ancestor (lstat
/// semantics — `fs::symlink_metadata`), returning `(ancestor, unresolved
/// basenames deepest-last)`. `None` on a non-ENOENT error or filesystem root
/// exhausted — mirroring both mjs walk loops.
fn walk_to_existing_ancestor(lexical_target: &Path) -> Option<(PathBuf, Vec<std::ffi::OsString>)> {
    let mut cursor = lexical_target.to_path_buf();
    let mut unresolved: Vec<std::ffi::OsString> = Vec::new();
    loop {
        match fs::symlink_metadata(&cursor) {
            Ok(_) => return Some((cursor, unresolved)),
            Err(err) => {
                if err.kind() != std::io::ErrorKind::NotFound {
                    return None;
                }
                let parent = match cursor.parent() {
                    Some(p) if p != cursor => p.to_path_buf(),
                    _ => return None,
                };
                let base = cursor.file_name()?.to_os_string();
                unresolved.insert(0, base);
                cursor = parent;
            }
        }
    }
}

fn rel_forward_slashes(rel: &Path) -> String {
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Port of `canonicalRelPath(workRoot, cwd, rawPath)` — a forward-slash path
/// relative to the repo root, or `None` when the target escapes the repo.
fn canonical_rel_path(work_root: &Path, cwd: &str, raw_path: &str) -> Option<String> {
    if raw_path.is_empty() {
        return None;
    }
    let root_real = realpath_or_none(work_root)?;

    let normalized = normalize_tool_path(raw_path);
    if windows_foreign_on_posix(raw_path) {
        return None;
    }
    let normalized_path = Path::new(&normalized);
    if !normalized_path.is_absolute()
        && normalized
            .split(std::path::MAIN_SEPARATOR)
            .any(|segment| segment == "..")
    {
        return None;
    }

    let cwd_path = Path::new(cwd);
    let cwd_base: &Path = if cwd_path.is_absolute() { cwd_path } else { &root_real };
    let lexical_target = resolve_path(cwd_base, normalized_path);

    let (cursor, unresolved) = walk_to_existing_ancestor(&lexical_target)?;
    let ancestor_real = realpath_or_none(&cursor)?;
    let mut canonical_target = ancestor_real;
    for segment in &unresolved {
        canonical_target.push(segment);
    }
    let canonical_target = normalize_lexical(&canonical_target);

    let rel = path_relative(&root_real, &canonical_target);
    let rel_str = rel_forward_slashes(&rel);
    if rel_str.is_empty() || rel_str == "." || rel_str == ".." || rel_str.starts_with("../") || rel.is_absolute() {
        return None;
    }
    Some(rel_str)
}

/// Port of `resolveTargetRealpath(cwd, root, rawTarget)` — the resolved
/// ABSOLUTE path (a sibling/main root can live entirely outside `root`).
fn resolve_target_realpath(cwd: &str, root: &Path, raw_target: &str) -> Option<PathBuf> {
    if raw_target.is_empty() {
        return None;
    }
    let normalized = normalize_tool_path(raw_target);
    if windows_foreign_on_posix(raw_target) {
        return None;
    }
    let cwd_path = Path::new(cwd);
    let cwd_base: &Path = if cwd_path.is_absolute() { cwd_path } else { root };
    let lexical_target = resolve_path(cwd_base, Path::new(&normalized));
    let (cursor, unresolved) = walk_to_existing_ancestor(&lexical_target)?;
    let ancestor_real = realpath_or_none(&cursor)?;
    let mut out = ancestor_real;
    for segment in &unresolved {
        out.push(segment);
    }
    Some(normalize_lexical(&out))
}

/// True when real path `child` is real root `parent` itself or strictly
/// nested under it.
fn is_under_root(parent: &Path, child: &Path) -> bool {
    let rel = path_relative(parent, child);
    let rel_str = rel_forward_slashes(&rel);
    rel_str.is_empty() || (rel_str != ".." && !rel_str.starts_with("../") && !rel.is_absolute())
}

fn read_gitdir_pointer(file: &Path, base: &Path) -> Option<PathBuf> {
    let raw = fs::read_to_string(file).ok()?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let raw = raw.strip_prefix("gitdir:").map(str::trim).unwrap_or(raw);
    if raw.is_empty() {
        return None;
    }
    let normalized = raw.replace('\\', &std::path::MAIN_SEPARATOR.to_string());
    Some(resolve_path(base, Path::new(&normalized)))
}

struct CurrentWorktree {
    main_root: PathBuf,
    id: String,
}

/// Port of `deriveCurrentWorktree(workRoot)` — mirrors adapter/resolveRoots'
/// linked-valid branch without importing it: `Some` only when `workRoot`'s
/// own `.git` is a FILE pointing at `<mainRoot>/.git/worktrees/<id>`.
fn derive_current_worktree(work_root: &Path) -> Option<CurrentWorktree> {
    let marker = work_root.join(".git");
    let stat = fs::metadata(&marker).ok()?;
    if !stat.is_file() {
        return None;
    }
    let gitdir = read_gitdir_pointer(&marker, work_root)?;
    let worktrees_root = resolve_path(&gitdir, Path::new(".."));
    let common_git_dir = resolve_path(&worktrees_root, Path::new(".."));
    if worktrees_root.file_name() != Some(std::ffi::OsStr::new("worktrees"))
        || common_git_dir.file_name() != Some(std::ffi::OsStr::new(".git"))
    {
        return None;
    }
    let main_root = realpath_or_none(common_git_dir.parent()?)?;
    let id = gitdir.file_name()?.to_string_lossy().into_owned();
    Some(CurrentWorktree { main_root, id })
}

/// Port of `resolveGrantedWorktreeRoot(mainRoot, id)` — the SAME
/// bidirectional gitdir check worktree-store.mjs's resolveWorktreeById uses;
/// never trusts a one-directional pointer alone.
fn resolve_granted_worktree_root(main_root: &Path, id: &str) -> Option<PathBuf> {
    let git_worktree_dir = main_root.join(".git").join("worktrees").join(id);
    if !fs::metadata(&git_worktree_dir).ok()?.is_dir() {
        return None;
    }
    let forward = read_gitdir_pointer(&git_worktree_dir.join("gitdir"), &git_worktree_dir)?;
    let worktree_root = forward.parent()?.to_path_buf();
    let reverse = read_gitdir_pointer(&worktree_root.join(".git"), &worktree_root)?;
    if normalize_lexical(&reverse) != normalize_lexical(&git_worktree_dir) {
        return None;
    }
    realpath_or_none(&worktree_root)
}

/// Port of `readGrantedWorktreeIds(mainRoot)` — fail-open to `[]` on ANY
/// error; never throws, never allows.
fn read_granted_worktree_ids(main_root: &Path) -> Vec<String> {
    let file = main_root.join(".bee").join("runtime").join("worktree-grants.json");
    let Ok(raw) = fs::read_to_string(&file) else { return Vec::new() };
    let Ok(Value::Object(grants)) = serde_json::from_str::<Value>(&raw) else { return Vec::new() };
    grants
        .into_iter()
        .filter(|(_, v)| v == &Value::Bool(true))
        .map(|(k, _)| k)
        .collect()
}

/// Port of `describeCrossWorktreeTarget(root, cwd, rawTarget)` — a
/// replacement denial reason naming a known sibling/main checkout, or `None`
/// to keep the generic containment message. NEVER changes the deny decision.
fn describe_cross_worktree_target(root: &Path, cwd: &str, raw_target: &str) -> Option<String> {
    let target_real = resolve_target_realpath(cwd, root, raw_target)?;

    let current = derive_current_worktree(root);
    let main_root = match &current {
        Some(c) => c.main_root.clone(),
        None => realpath_or_none(root)?,
    };

    // Session rooted in a worktree, target inside the MAIN checkout instead.
    if current.is_some() && is_under_root(&main_root, &target_real) {
        return Some(
            "bee write guard denied this target: it could not be canonically contained inside the physical worktree — \
this path belongs to the main checkout, not this worktree. FIX: run this from a session rooted there."
                .to_string(),
        );
    }

    // Target inside a KNOWN GRANTED sibling worktree.
    for id in read_granted_worktree_ids(&main_root) {
        if let Some(c) = &current {
            if id == c.id {
                continue; // this session's own root, not a sibling
            }
        }
        if let Some(worktree_root) = resolve_granted_worktree_root(&main_root, &id) {
            if is_under_root(&worktree_root, &target_real) {
                return Some(format!(
                    "bee write guard denied this target: it could not be canonically contained inside the physical worktree — \
it resolves inside worktree \"{id}\". FIX: open a session with cwd={root} to work there, or merge it \
back from main via `bee worktree merge --id {id}`.",
                    root = worktree_root.display(),
                ));
            }
        }
    }

    None
}

/// Port of `resolveCompanionMountedRelPath(root, cwd, rawTarget)`
/// (fix-write-guard-symlink) — a root-relative path rooted at the marker's
/// `mountPath` when the target resolves inside the marker-declared,
/// realpath-verified companion mount; `None` on any mismatch or failure.
fn resolve_companion_mounted_rel_path(root: &Path, cwd: &str, raw_target: &str) -> Option<String> {
    let raw = fs::read_to_string(root.join(".bee").join("companion-session.json")).ok()?;
    let marker: Value = serde_json::from_str(&raw).ok()?;
    let declared_worktree_path = marker.get("worktreePath").and_then(Value::as_str).filter(|s| !s.is_empty())?;
    let mount_path = marker.get("mountPath").and_then(Value::as_str).filter(|s| !s.is_empty())?;

    let declared_real = realpath_or_none(Path::new(declared_worktree_path))?;
    let live_mount_real = realpath_or_none(&root.join(mount_path))?;
    if declared_real != live_mount_real {
        return None;
    }

    let target_real = resolve_target_realpath(cwd, root, raw_target)?;
    if !is_under_root(&live_mount_real, &target_real) {
        return None;
    }

    let offset = path_relative(&live_mount_real, &target_real);
    let offset_str = rel_forward_slashes(&offset);
    if offset_str.is_empty() {
        return Some(mount_path.to_string());
    }
    Some(format!("{mount_path}/{offset_str}"))
}

// ─── agent identity ─────────────────────────────────────────────────────────

fn nested_string(obj: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(v) = obj.get(*key).and_then(Value::as_str) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Manual port of the mjs `/\bBEE_AGENT_NAME=(["']?)([^"'\s]+)\1/` match
/// (regex-lite has no backreferences): `\b` before `B`, an optional quote,
/// a run of non-quote/non-whitespace characters, then the SAME quote (or
/// nothing when unquoted).
fn extract_bee_agent_name(command: &str) -> Option<String> {
    const NEEDLE: &str = "BEE_AGENT_NAME=";
    let bytes = command.as_bytes();
    let mut search_from = 0;
    while let Some(found) = command[search_from..].find(NEEDLE) {
        let start = search_from + found;
        let word_boundary = start == 0 || {
            let prev = bytes[start - 1] as char;
            !(prev.is_ascii_alphanumeric() || prev == '_')
        };
        if word_boundary {
            let after = &command[start + NEEDLE.len()..];
            let mut chars = after.chars().peekable();
            let quote = match chars.peek() {
                Some(&q @ ('"' | '\'')) => {
                    chars.next();
                    Some(q)
                }
                _ => None,
            };
            let mut captured = String::new();
            while let Some(&c) = chars.peek() {
                if c == '"' || c == '\'' || c.is_whitespace() {
                    break;
                }
                captured.push(c);
                chars.next();
            }
            if !captured.is_empty() {
                match quote {
                    Some(q) if chars.peek() == Some(&q) => return Some(captured),
                    None => return Some(captured),
                    _ => {}
                }
            }
        }
        search_from = start + NEEDLE.len();
    }
    None
}

fn infer_agent_name(payload: &Value, tool_input: &Value) -> Option<String> {
    if let Some(from_payload) = nested_string(payload, &["agent_name", "agentName", "agent_nickname", "subagent_type"]) {
        return Some(from_payload);
    }
    if let Some(command) = tool_input.get("command").and_then(Value::as_str) {
        if let Some(name) = extract_bee_agent_name(command) {
            return Some(name);
        }
    }
    std::env::var("BEE_AGENT_NAME").ok().filter(|v| !v.is_empty())
}

// ─── fail-open boundary ─────────────────────────────────────────────────────

/// The Rust twin of the mjs hook's shared `try { ... } catch (error) {
/// logCrash(...); return 0; }` boundary: any panic inside `f` is caught,
/// logged as a crash line to `.bee/logs/hooks.jsonl`, and the hook exits 0
/// (fail-open) — an internal bug must never flip a decision or an exit
/// code. Public so the conformance corpus can prove the wrapper itself
/// (crash fixture class: exit 0 plus crash line) with a genuine panic.
pub fn run_fail_open<F>(root: Option<&Path>, source: Option<&str>, f: F) -> i32
where
    F: FnOnce() -> i32,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(code) => code,
        Err(panic) => {
            let message = panic
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic".to_string());
            adapter::log_crash(root, HOOK_NAME, &message, source);
            0
        }
    }
}

// ─── main flow ──────────────────────────────────────────────────────────────

enum Outcome {
    Allow,
    /// Allow, with the PreToolUse allow+notice JSON already rendered for
    /// stdout (advisory reservation/cross-worktree warnings).
    AllowNotice(String),
    Deny(String),
}

pub fn run(argv: &[String], raw_stdin: &str) -> i32 {
    let ctx: HookContext = adapter::read_hook_context(HOOK_NAME, argv, raw_stdin);
    let Some(root) = ctx.root.clone() else {
        return 0;
    };

    let payload = &ctx.payload;
    let tool_name = payload
        .get("tool_name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .or_else(|| payload.get("toolName").and_then(Value::as_str))
        .unwrap_or("");

    let write_capable =
        WRITE_TOOLS.contains(&tool_name) || tool_name == "Bash" || APPLY_PATCH_TOOLS.contains(&tool_name);
    if write_capable && ctx.worktree_resolution == "linked-invalid" {
        let _ = std::io::stderr().write_all(WORKTREE_LINK_INVALID_MESSAGE.as_bytes());
        return 2;
    }

    let store_root = ctx.store_root.clone().unwrap_or_else(|| root.clone());
    if !store_root.join(".bee").join("bin").join("lib").join("state.mjs").exists() {
        return 0;
    }

    let source = ctx.source.clone();
    run_fail_open(Some(&root), source.as_deref(), || {
        match decide(&ctx, tool_name, &root, &store_root) {
            Outcome::Deny(reason) => {
                // Deliberate deny: exit 2 with the reason on stderr. A
                // log-write failure can never cancel this deny.
                let _ = std::io::stderr().write_all(reason.as_bytes());
                2
            }
            Outcome::AllowNotice(json_out) => {
                let _ = std::io::stdout().write_all(json_out.as_bytes());
                0
            }
            Outcome::Allow => 0,
        }
    })
}

fn decide(ctx: &HookContext, tool_name: &str, root: &Path, store_root: &Path) -> Outcome {
    if !hookconfig::hook_enabled(store_root, HOOK_NAME) {
        return Outcome::Allow;
    }

    let payload = &ctx.payload;
    let empty = Value::Object(serde_json::Map::new());
    let tool_input = match payload.get("tool_input") {
        Some(v @ Value::Object(_)) => v,
        _ => &empty,
    };
    let cwd = ctx.cwd.as_str();

    if READ_TOOLS.contains(&tool_name) || tool_name == "AskUserQuestion" {
        // rust-port-12 scope (read side, AskUserQuestion) — fall open.
        return Outcome::Allow;
    }
    if tool_name == "Bash" || APPLY_PATCH_TOOLS.contains(&tool_name) {
        // rust-port-11 (Bash target extraction + git/CLI-shape/internals
        // checks) and rust-port-12 (apply_patch proving) — fall open.
        return Outcome::Allow;
    }
    if !WRITE_TOOLS.contains(&tool_name) {
        return Outcome::Allow;
    }

    let state = read_state(store_root);
    let agent_name = infer_agent_name(payload, tool_input);
    // fsh-8 (D3/D4): absent/empty session_id reads as None — byte-identical
    // to the sessionless checkWrite contract.
    let session_id = payload
        .get("session_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let raw_target = tool_input.get("file_path").and_then(Value::as_str).unwrap_or("");
    let mut denial: Option<String> = None;
    let mut rel_paths: Vec<String> = Vec::new();
    match canonical_rel_path(root, cwd, raw_target)
        .or_else(|| resolve_companion_mounted_rel_path(root, cwd, raw_target))
    {
        Some(rel) => rel_paths.push(rel),
        None => {
            let enriched = describe_cross_worktree_target(root, cwd, raw_target);
            denial = Some(enriched.unwrap_or_else(|| GENERIC_CONTAINMENT_MESSAGE.to_string()));
        }
    }

    // The topology checkWrite would resolve itself via
    // resolveWriteTopology(storeRoot, ctx.controlRoot): the adapter's own
    // resolve_roots(store_root) is the same classification resolveContext
    // is built on (see bee_core::guards::WriteTopology).
    let topology = derive_topology(ctx, store_root);

    // Preserve the established diagnostic precedence: the concrete policy
    // reason remains the user-facing correction, first hit wins.
    let mut reservation_warnings: Vec<String> = Vec::new();
    if denial.is_none() {
        for rel in &rel_paths {
            match check_write(
                store_root,
                &state,
                rel,
                agent_name.as_deref(),
                session_id.as_deref(),
                &topology,
            ) {
                Verdict::Deny { reason, .. } => {
                    denial = Some(reason);
                    break;
                }
                Verdict::Allow { warning: Some(w) } => reservation_warnings.push(w),
                Verdict::Allow { warning: None } => {}
            }
        }
    }

    if let Some(reason) = denial {
        return Outcome::Deny(reason);
    }

    if !reservation_warnings.is_empty() {
        // multisession-native-13 (D4): allow + non-blocking systemMessage.
        let joined = reservation_warnings.join("\n");
        let output = json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": joined,
            },
            "systemMessage": joined,
        });
        return Outcome::AllowNotice(output.to_string());
    }

    Outcome::Allow
}

fn derive_topology(ctx: &HookContext, store_root: &Path) -> WriteTopology {
    let control_root_override = ctx.control_root.clone();
    let roots = adapter::resolve_roots(store_root);
    let give_up = roots.work_root.is_none() || roots.worktree_resolution == "linked-invalid";
    let (workspace_root, workspace_id, worktree_id) = if give_up {
        // resolveContext's all-null case (or resolveWriteTopology's
        // caught WorktreeLinkInvalidError) — fail-open, never a deny.
        (None, None, None)
    } else if roots.worktree_resolution == "linked-valid" {
        let granted = roots.store_root.is_some() && roots.store_root == roots.work_root;
        let workspace_id = if granted { roots.id.clone() } else { Some("main".to_string()) };
        (roots.work_root.clone(), workspace_id, roots.id.clone())
    } else {
        (roots.work_root.clone(), Some("main".to_string()), None)
    };
    let control_root = control_root_override
        .or_else(|| roots.main_root.clone())
        .or_else(|| roots.work_root.clone())
        .unwrap_or_else(|| store_root.to_path_buf());
    WriteTopology { workspace_root, control_root, workspace_id, worktree_id }
}
