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
//! Scope boundary (validation decision 2026-07-26): rust-port-9 shipped the
//! Edit/Write/MultiEdit spine; rust-port-11 (split 2 of 3) adds the Bash
//! path — `extractBashTargets` write-target extraction, the intake-gate git
//! exemption (`checkGitBashCommand`, GIT SPAWN PARITY: rust spawns git
//! exactly where node does), the internals-reach guard, and CLI-shape
//! check (d) validated against the rust-port-8 registry bridge
//! (`.bee/cache/command-registry.json`; a stale snapshot skips check (d)
//! with a visible coverage-gap line — the ACCEPTED rust-only allow window).
//! - rust-port-12 (split 3 of 3) adds the read side (`checkRead` — privacy
//!   marker + scout-dir denies — plus the large-read size guard, ported
//!   here since bee-write-guard.mjs defines it in the HOOK, not
//!   guards.mjs), Codex `apply_patch` target-by-target provability (same
//!   gate/reservation decisions as Edit/Write/Bash on every PROVED target),
//!   and `AskUserQuestion` schema validation including the ask-guard-autofix
//!   repair path.
//!
//! Exit contract (D7b): allow = exit 0 silent (or the allow+notice JSON on
//! stdout for advisory warnings and the ask-guard-autofix repair); deliberate
//! deny = exit 2 with the reason on stderr; internal crash = exit 0 plus a
//! `.bee/logs/hooks.jsonl` crash line (fail-open).

use std::fs;
use std::io::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Component, Path, PathBuf};

use serde_json::{json, Value};

use bee_core::config::read_config_value;
use bee_core::guards::{
    check_ask_user_question, check_bin_lib_import_bash_command, check_git_bash_command, check_read, check_write,
    extract_bash_targets, AskVerdict, Verdict, WriteTopology,
};
use bee_core::registry::{load_registry, CommandRegistryEntry, Registry};
use bee_core::state::read_state;
use bee_core::tokenize::tokenize_command;
use bee_core::validate_args::validate;

use crate::adapter::{self, HookContext};
use crate::hookconfig;

const HOOK_NAME: &str = "write-guard";
const READ_TOOLS: [&str; 3] = ["Read", "Glob", "Grep"];
const WRITE_TOOLS: [&str; 3] = ["Edit", "Write", "MultiEdit"];
const APPLY_PATCH_TOOLS: [&str; 2] = ["apply_patch", "ApplyPatch"];

const GENERIC_CONTAINMENT_MESSAGE: &str =
    "bee write guard denied this target: it could not be canonically contained inside the physical worktree. \
FIX: use a plain in-worktree path without traversal, outside absolute paths, or symlink escapes.";

const GENERIC_BASH_CONTAINMENT_MESSAGE: &str =
    "bee write guard denied Bash: one or more extracted targets could not be canonically contained inside the physical worktree. \
FIX: use plain in-worktree paths without traversal, outside absolute paths, or symlink escapes.";

const WORKTREE_LINK_INVALID_MESSAGE: &str =
    "bee worktree guard denied this write: WORKTREE_LINK_INVALID — linked worktree metadata could not be validated. \
FIX: repair or recreate the Git worktree before retrying; no worktree-local .bee store is trusted.";

// ─── large-read guard (router-cost D1/D2/D3/D4) — rust-port-12 ────────────
// Ported here (not bee-core/guards.rs) because bee-write-guard.mjs defines
// these directly in the hook, not in guards.mjs — `checkRead` itself is
// pattern-only, no I/O.

/// Default `guards.max_read_lines` (D2's measured value) when the config key
/// is absent.
const DEFAULT_MAX_READ_LINES: u64 = 800;
/// Files larger than this are never measured — fail-open (allow) instead of
/// reading the whole thing into memory just to decide whether to deny it.
const READ_SIZE_GUARD_CAP_BYTES: u64 = 25 * 1024 * 1024;

fn resolve_max_read_lines(config: &Value) -> u64 {
    match config.get("guards").and_then(|g| g.get("max_read_lines")).and_then(Value::as_f64) {
        Some(n) if n.is_finite() && n > 0.0 => n as u64,
        _ => DEFAULT_MAX_READ_LINES,
    }
}

/// Null-byte sniff over a bounded prefix — the same cheap heuristic the mjs
/// source uses to distinguish text from binary content.
fn looks_binary(buffer: &[u8]) -> bool {
    let scan_len = buffer.len().min(8000);
    buffer[..scan_len].iter().any(|&b| b == 0)
}

/// Counts newline-delimited lines, counting a non-empty trailing partial
/// line (no final `\n`) as one more — "how many lines" a human would say,
/// including the last one.
fn count_lines(buffer: &[u8]) -> u64 {
    let mut count = buffer.iter().filter(|&&b| b == b'\n').count() as u64;
    if !buffer.is_empty() && buffer[buffer.len() - 1] != b'\n' {
        count += 1;
    }
    count
}

/// Port of `checkReadSizeDenial(absPath, label, threshold)` — every error
/// path (ENOENT, EACCES, a directory, a symlink loop, ...) returns `None`:
/// fail-open, never deny because measurement failed.
fn check_read_size_denial(abs_path: &Path, label: &str, threshold: u64) -> Option<String> {
    let meta = fs::metadata(abs_path).ok()?;
    if !meta.is_file() {
        return None;
    }
    if meta.len() > READ_SIZE_GUARD_CAP_BYTES {
        return None;
    }
    let buffer = fs::read(abs_path).ok()?;
    if looks_binary(&buffer) {
        return None;
    }
    let line_count = count_lines(&buffer);
    if line_count < threshold {
        return None;
    }
    Some(format!(
        "bee read-size guard: \"{label}\" is {line_count} lines (threshold: {threshold}) and this Read \
has neither `offset` nor `limit` — reading it unbounded would load the whole file into context. \
FIX: pass `limit` (and optionally `offset`) to read a slice, or dispatch a `bee-extract` worker to read the whole file."
    ))
}

// ─── Codex apply_patch target extraction (canonical envelope) ─────────────
// rust-port-12. Ported here (not bee-core/guards.rs) because
// bee-write-guard.mjs defines these directly in the hook.

/// One target line per file operation, matched manually (no regex crate
/// dependency in this bin): `*** Add File: <path>` | `*** Update File:
/// <path>` | `*** Delete File: <path>` | `*** Move to: <path>` (destination
/// of an Update File move). Equivalent to the mjs `PATCH_TARGET_RE` —
/// `^\*\*\*\s+(?:Add File|Update File|Delete File|Move to):\s*(.+)\s*$` —
/// captured then trimmed either way, so a greedy capture + explicit
/// `.trim()` reproduces the mjs lazy-quantifier result exactly.
const PATCH_TARGET_VERBS: [&str; 4] = ["Add File", "Update File", "Delete File", "Move to"];

fn parse_patch_target_line(line: &str) -> Option<String> {
    let after_stars = line.strip_prefix("***")?;
    let first = after_stars.chars().next()?;
    if !first.is_whitespace() {
        return None;
    }
    let after_ws = after_stars.trim_start();
    for verb in PATCH_TARGET_VERBS {
        if let Some(rest) = after_ws.strip_prefix(verb) {
            if let Some(rest) = rest.strip_prefix(':') {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

/// Port of `applyPatchText(toolInput)` — the canonical Codex shape is
/// `tool_input.input`; tolerate the patch envelope arriving under
/// `patch`/`command` without forking per runtime.
fn apply_patch_text(tool_input: &Value) -> Option<String> {
    for key in ["input", "patch", "command"] {
        if let Some(v) = tool_input.get(key).and_then(Value::as_str) {
            if v.contains("*** Begin Patch") {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Port of `extractApplyPatchTargets(patchText)`.
fn extract_apply_patch_targets(patch_text: &str) -> Vec<String> {
    patch_text.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line)).filter_map(parse_patch_target_line).collect()
}

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

// ─── read-side dispatch (checkRead + read-size guard) — rust-port-12 ──────

/// Port of `lexicalRelPath(root, cwd, rawPath)` — LEXICAL (no filesystem
/// access) relative-path resolution for the read side; `None` when the
/// target escapes the repo. Deliberately simpler than `canonical_rel_path`
/// (no realpath walk) — mirrors the mjs source's own simpler read helper.
fn lexical_rel_path(root: &Path, cwd: &str, raw_path: &str) -> Option<String> {
    if raw_path.is_empty() {
        return None;
    }
    let raw = Path::new(raw_path);
    let abs = if raw.is_absolute() {
        normalize_lexical(raw)
    } else {
        let cwd_path = Path::new(cwd);
        let base: &Path = if !cwd.is_empty() { cwd_path } else { root };
        resolve_path(base, raw)
    };
    let root_abs = normalize_lexical(root);
    let rel = path_relative(&root_abs, &abs);
    let rel_str = rel_forward_slashes(&rel);
    if rel_str.is_empty() || rel_str == "." || rel_str.starts_with("..") || rel.is_absolute() {
        return None;
    }
    Some(rel_str)
}

/// Port of the mjs hook's `READ_TOOLS.has(toolName)` branch — `checkRead`
/// (privacy/scout) first, then the large-read size guard for a bare `Read`
/// call carrying neither `offset` nor `limit`. `Glob`/`Grep` never carry
/// whole-file content, so the size guard is scoped to `Read` only, exactly
/// like the mjs source.
fn read_outcome(tool_name: &str, root: &Path, store_root: &Path, tool_input: &Value, cwd: &str) -> Outcome {
    let raw_target = {
        let fp = tool_input.get("file_path").and_then(Value::as_str).filter(|s| !s.is_empty());
        let p = tool_input.get("path").and_then(Value::as_str).filter(|s| !s.is_empty());
        fp.or(p).unwrap_or("")
    };
    let Some(rel) = lexical_rel_path(root, cwd, raw_target) else {
        return Outcome::Allow;
    };

    let verdict = check_read(&rel);
    if !verdict.allow {
        let mut parts = vec![verdict
            .reason
            .clone()
            .unwrap_or_else(|| format!("bee {} guard denied: {rel}", verdict.kind.unwrap_or("read")))];
        if let Some(marker) = &verdict.marker {
            parts.push(marker.clone());
        }
        return Outcome::Deny(parts.join("\n"));
    }

    if tool_name == "Read" && tool_input.get("offset").is_none() && tool_input.get("limit").is_none() {
        let config = read_config_value(store_root);
        let threshold = resolve_max_read_lines(&config);
        if let Some(reason) = check_read_size_denial(&root.join(&rel), &rel, threshold) {
            return Outcome::Deny(reason);
        }
    }

    Outcome::Allow
}

/// Port of the mjs hook's `toolName === "AskUserQuestion"` branch — a
/// schema deny surfaces as an ordinary `Outcome::Deny`; an ask-guard-autofix
/// repair surfaces as the PreToolUse `updatedInput` allow+notice contract
/// (D2), same JSON shape the mjs source emits on stdout.
fn ask_user_question_outcome(tool_input: &Value) -> Outcome {
    match check_ask_user_question(tool_input) {
        AskVerdict::Allow => Outcome::Allow,
        AskVerdict::Deny { reason, .. } => Outcome::Deny(reason),
        AskVerdict::Fixed { fixed, notes } => {
            let notes_joined = notes.join("; ");
            let output = json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "allow",
                    "permissionDecisionReason": notes_joined,
                    "updatedInput": fixed,
                    "additionalContext": format!("bee AskUserQuestion guard auto-fixed: {notes_joined}"),
                },
                "systemMessage": format!("bee AskUserQuestion guard: {notes_joined}"),
            });
            Outcome::AllowNotice(output.to_string())
        }
    }
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

    if tool_name == "AskUserQuestion" {
        return ask_user_question_outcome(tool_input);
    }
    if READ_TOOLS.contains(&tool_name) {
        return read_outcome(tool_name, root, store_root, tool_input, cwd);
    }
    if !WRITE_TOOLS.contains(&tool_name) && tool_name != "Bash" && !APPLY_PATCH_TOOLS.contains(&tool_name) {
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

    let command: String = if tool_name == "Bash" {
        tool_input.get("command").and_then(Value::as_str).unwrap_or("").to_string()
    } else {
        String::new()
    };

    let mut denial: Option<String> = None;
    let mut rel_paths: Vec<String> = Vec::new();
    if tool_name == "Bash" {
        if !command.is_empty() {
            let targets = extract_bash_targets(&command);
            let canonicalized: Vec<(String, Option<String>)> = targets
                .paths
                .iter()
                .map(|p| {
                    let rel = canonical_rel_path(root, cwd, p)
                        .or_else(|| resolve_companion_mounted_rel_path(root, cwd, p));
                    (p.clone(), rel)
                })
                .collect();
            rel_paths = canonicalized.iter().filter_map(|(_, rel)| rel.clone()).collect();
            if rel_paths.len() != canonicalized.len() {
                let first_failing =
                    canonicalized.iter().find(|(_, rel)| rel.is_none()).map(|(raw, _)| raw.clone());
                let enriched = first_failing.and_then(|raw| describe_cross_worktree_target(root, cwd, &raw));
                denial = Some(enriched.unwrap_or_else(|| GENERIC_BASH_CONTAINMENT_MESSAGE.to_string()));
            } else if rel_paths.is_empty() && targets.broad_write {
                rel_paths.push("**".to_string());
            }
        }
    } else if APPLY_PATCH_TOOLS.contains(&tool_name) {
        // D2 / approach.md §2: an intercepted apply_patch runs the existing
        // gate/direct-edit/reservation decisions on every PROVED target.
        match apply_patch_text(tool_input) {
            None => {
                // Malformed OUTER payload: apply_patch fired but tool_input
                // carries no recognizable "*** Begin Patch" envelope at all —
                // nothing was genuinely intercepted, so this stays D2's
                // visible fail-open.
                adapter::log_coverage_gap(
                    Some(root),
                    HOOK_NAME,
                    "applypatch-unparsed",
                    "apply_patch intercepted but no canonical patch envelope found in tool_input",
                    ctx.source.as_deref(),
                );
            }
            Some(patch_text) => {
                let targets = extract_apply_patch_targets(&patch_text);
                let resolved: Vec<Option<String>> =
                    targets.iter().map(|p| canonical_rel_path(root, cwd, p)).collect();
                rel_paths = resolved.iter().filter_map(|r| r.clone()).collect();
                if targets.is_empty() || rel_paths.len() < targets.len() {
                    // P1 repair (codex-parity-4): the envelope WAS
                    // intercepted, but the target set cannot be fully proved
                    // (no Add/Update/Delete/Move line parsed at all, or a
                    // parsed target escapes the repo / fails to resolve) —
                    // deny rather than risk an unchecked write. Still logged
                    // as a visible coverage gap for audit (D2).
                    let detail = if targets.is_empty() {
                        "apply_patch intercepted but no Add/Update/Delete/Move/\"Move to\" target line could be parsed from the patch body".to_string()
                    } else {
                        format!(
                            "apply_patch intercepted but {} of {} target(s) could not be proved inside the repo",
                            targets.len() - rel_paths.len(),
                            targets.len(),
                        )
                    };
                    adapter::log_coverage_gap(
                        Some(root),
                        HOOK_NAME,
                        "applypatch-unparsed",
                        &detail,
                        ctx.source.as_deref(),
                    );
                    denial = Some(
                        "bee apply_patch guard: this patch's target set could not be fully proved inside the repo — \
denying rather than risking an unchecked write. \
FIX: use canonical \"*** Add File:\", \"*** Update File:\", \"*** Delete File:\", and \"*** Move to:\" \
lines naming plain in-repo relative paths (no path traversal, no unresolvable escapes), then resubmit."
                            .to_string(),
                    );
                }
            }
        }
    } else {
        let raw_target = tool_input.get("file_path").and_then(Value::as_str).unwrap_or("");
        match canonical_rel_path(root, cwd, raw_target)
            .or_else(|| resolve_companion_mounted_rel_path(root, cwd, raw_target))
        {
            Some(rel) => rel_paths.push(rel),
            None => {
                let enriched = describe_cross_worktree_target(root, cwd, raw_target);
                denial = Some(enriched.unwrap_or_else(|| GENERIC_CONTAINMENT_MESSAGE.to_string()));
            }
        }
    }

    // The topology checkWrite would resolve itself via
    // resolveWriteTopology(storeRoot, ctx.controlRoot): the adapter's own
    // resolve_roots(store_root) is the same classification resolveContext
    // is built on (see bee_core::guards::WriteTopology).
    let topology = derive_topology(ctx, store_root);

    // Preserve the established diagnostic precedence when a mixed request
    // contains both an unprovable target and a proved policy-denied target:
    // the whole request is denied either way, and the concrete policy
    // reason remains the user-facing correction — the loop runs even with a
    // containment denial already set, and a policy deny replaces it,
    // exactly like the mjs hook's own loop.
    let mut reservation_warnings: Vec<String> = Vec::new();
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

    // Intake-gate git exemption (D1/D3/D4, cell ige-2, closes P46 / GH #1):
    // additive and scoped to Bash only — check_git_bash_command itself
    // returns None unless the phase is terminal AND the command contains a
    // recognizable `git` invocation, so it can never override or discard a
    // denial the checks above already computed.
    if denial.is_none() && tool_name == "Bash" && !command.is_empty() {
        if let Some(Verdict::Deny { reason, .. }) = check_git_bash_command(
            store_root,
            &state,
            &command,
            cwd,
            session_id.as_deref(),
            &topology.control_root,
        ) {
            denial = Some(reason);
        }
    }

    // Internals-reach guard (state-query-surface, cell sqs-a, D 3fbe2f79):
    // deny an inline-eval `node -e`/`--eval`/`-p` whose script imports a
    // bin/lib or packages/bee/lib module — never a file-based run. Additive,
    // Bash only, first-hit-wins like every other check above.
    if denial.is_none() && tool_name == "Bash" && !command.is_empty() {
        if let Some(Verdict::Deny { reason, .. }) = check_bin_lib_import_bash_command(&command) {
            denial = Some(reason);
        }
    }

    // Check (d) — CLI-shape validation (additive, D4). Runs unconditionally
    // for Bash calls but can only ever ASSIGN a denial when none exists yet
    // (first hit wins). Its containment is deliberately separate from the
    // shared fail-open boundary: a fault here fails open for THIS check
    // only (logged), never discarding a denial checks (a)-(c) already
    // computed — mirroring the mjs hook's dedicated try/catch.
    if tool_name == "Bash" && !command.is_empty() {
        match catch_unwind(AssertUnwindSafe(|| cli_shape_denial(ctx, root, store_root, &command))) {
            Ok(Some(reason)) => {
                if denial.is_none() {
                    denial = Some(reason);
                }
            }
            Ok(None) => {}
            Err(panic) => {
                let message = panic
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| panic.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".to_string());
                adapter::log_crash(Some(root), HOOK_NAME, &message, ctx.source.as_deref());
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

// ─── check (d): CLI-shape validation (harness-integration D4, additive) ────
// Port of bee-write-guard.mjs's own checkCliShape/resolveCliCommandName/
// parseCliFlags/splitCliSegments (they live in the HOOK there, not
// guards.mjs — layout mirrored here). The registry comes from the
// rust-port-8 JSON bridge (`.bee/cache/command-registry.json`) instead of a
// dynamic import of command-registry.mjs; `load_registry` re-hashes the
// live mjs source, and a stale snapshot skips this check with a visible
// coverage-gap line — the ACCEPTED rust-only allow window (validation
// decision 2026-07-26, two-fixture stale-registry semantics).

const CLI_SEGMENT_SEPARATORS: [&str; 5] = ["&&", "||", ";", "|", "&"];

fn split_cli_segments(tokens: &[String]) -> Vec<Vec<String>> {
    let mut segments: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for token in tokens {
        if CLI_SEGMENT_SEPARATORS.contains(&token.as_str()) {
            if !current.is_empty() {
                segments.push(std::mem::take(&mut current));
            }
        } else {
            current.push(token.clone());
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

/// The mjs `LEGACY_HELPER_RE` (`/^bee_([a-z]+)\.mjs$/i`) — a TRANSITION
/// GUARD for hosts mid-upgrade (shim-retire, decision bbc6bcea D1/D3); the
/// captured group keeps its original case, exactly like the JS capture.
fn legacy_helper_group(base: &str) -> Option<String> {
    let lower = base.to_ascii_lowercase();
    if !lower.starts_with("bee_") || !lower.ends_with(".mjs") || base.len() < 9 {
        return None;
    }
    let middle = &base[4..base.len() - 4];
    if middle.is_empty() || !middle.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    Some(middle.to_string())
}

/// The mjs `DISPATCHER_RE` (`/^bee\.mjs$/i`) — flagged at D7 flip time as an
/// invocation-string recognizer that must learn the new binary name.
fn is_dispatcher_basename(base: &str) -> bool {
    base.eq_ignore_ascii_case("bee.mjs")
}

/// Port of `resolveCliCommandName(scriptBasename, positionalTokens,
/// registry)` — longest-prefix match over the registry's own names, the
/// SAME rule bee.mjs's resolveCommand uses. `None` when the shape is
/// ambiguous or no prefix length matches — left to fail open, never guessed.
fn resolve_cli_command_name(
    script_basename: &str,
    positional: &[String],
    names: &std::collections::HashSet<&str>,
) -> Option<(String, usize)> {
    let legacy_match = legacy_helper_group(script_basename);
    let is_dispatcher = legacy_match.is_none() && is_dispatcher_basename(script_basename);
    if legacy_match.is_none() && !is_dispatcher {
        return None;
    }

    let group: Option<String> = match &legacy_match {
        Some(g) => Some(g.clone()),
        None => positional.first().cloned(),
    };
    if legacy_match.is_some() && group.as_deref() == Some("status") {
        return Some(("status".to_string(), 0));
    }
    if is_dispatcher {
        match group.as_deref() {
            None | Some("") => return None,
            Some(g) if g.starts_with('-') => return None,
            Some("status") => return Some(("status".to_string(), 1)),
            _ => {}
        }
    }
    let group = group?;

    // Collect the run of non-flag tokens after the group — the same
    // "leading tokens" shape bee.mjs's own resolveCommand matches against.
    let scan_from: &[String] = if is_dispatcher { &positional[1..] } else { positional };
    let mut verb_tokens: Vec<&str> = Vec::new();
    for token in scan_from {
        if token.starts_with('-') {
            break;
        }
        verb_tokens.push(token.as_str());
    }
    if verb_tokens.is_empty() {
        return None; // no verb token at all: ambiguous, fail open
    }

    let mut name_segments: Vec<&str> = vec![group.as_str()];
    name_segments.extend(verb_tokens);
    for n in (2..=name_segments.len()).rev() {
        let candidate = name_segments[..n].join(".");
        if names.contains(candidate.as_str()) {
            // Legacy shape: positional holds ONLY verb tokens (the group
            // came from the script name), so consumed = n - 1. Dispatcher
            // shape: positional[0] IS the group, so consumed = n.
            return Some((candidate, if is_dispatcher { n } else { n - 1 }));
        }
    }
    None
}

fn upsert_flag(parsed: &mut Vec<(String, Value)>, name: String, value: Value) {
    if let Some(slot) = parsed.iter_mut().find(|(k, _)| *k == name) {
        slot.1 = value;
    } else {
        parsed.push((name, value));
    }
}

/// Port of `parseCliFlags(flagTokens, propertiesSchema)` — the schema is the
/// parsing contract (a `boolean`-typed flag consumes no value), not a
/// hardcoded flag list. Insertion order preserved (JS object key order for
/// string-keyed flags).
fn parse_cli_flags(flag_tokens: &[String], properties: Option<&Value>) -> Vec<(String, Value)> {
    let mut parsed: Vec<(String, Value)> = Vec::new();
    let mut i = 0usize;
    while i < flag_tokens.len() {
        let token = &flag_tokens[i];
        if !token.starts_with("--") {
            i += 1;
            continue;
        }
        if let Some(eq) = token.find('=') {
            upsert_flag(&mut parsed, token[2..eq].to_string(), Value::String(token[eq + 1..].to_string()));
            i += 1;
            continue;
        }
        let name = token[2..].to_string();
        let is_boolean = properties
            .and_then(|p| p.get(&name))
            .and_then(|s| s.get("type"))
            .and_then(Value::as_str)
            == Some("boolean");
        if is_boolean {
            upsert_flag(&mut parsed, name, Value::Bool(true));
        } else if let Some(next) = flag_tokens.get(i + 1) {
            // Consume the next token as the value unconditionally, even if
            // it starts with "--" — matching bee.mjs's parseFlags exactly.
            upsert_flag(&mut parsed, name, Value::String(next.clone()));
            i += 1;
        } else {
            upsert_flag(&mut parsed, name, Value::Bool(true));
        }
        i += 1;
    }
    parsed
}

/// Port of `checkCliShape(command, registry, validateFn)` — scan every shell
/// segment for a recognizable bee-cli invocation and validate it against the
/// registry. `Some(reason)` on the first structural mismatch, else `None`.
fn check_cli_shape(command: &str, entries: &[CommandRegistryEntry]) -> Option<String> {
    if command.is_empty() {
        return None;
    }
    let names: std::collections::HashSet<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    let segments = split_cli_segments(&tokenize_command(command));
    for segment in &segments {
        for i in 0..segment.len() {
            let base = segment[i].replace('\\', "/").rsplit('/').next().unwrap_or("").to_string();
            if legacy_helper_group(&base).is_none() && !is_dispatcher_basename(&base) {
                continue;
            }
            let positional = &segment[i + 1..];
            let Some((command_name, consumed)) = resolve_cli_command_name(&base, positional, &names) else {
                break; // ambiguous shape for this segment: fail open
            };
            let Some(entry) = entries.iter().find(|e| e.name == command_name) else {
                break; // unknown command name: dispatcher's concern, not this guard's
            };
            let flag_tokens = &positional[consumed..];
            let parsed_args = parse_cli_flags(flag_tokens, entry.parameters.get("properties"));
            let problems = validate(&entry.parameters, &parsed_args);
            if !problems.is_empty() {
                // ce-1 (cli-ergonomics D1): render every problem joined; the
                // pinned substrings stay exactly where the mjs source puts
                // them — "bee CLI-shape guard" + entry.name in the opening
                // clause, "field: <first>" naming the FIRST problem's field.
                let field = problems[0].field.clone().filter(|f| !f.is_empty());
                let detail = problems
                    .iter()
                    .map(|p| {
                        let flag_suffix = p
                            .field
                            .as_ref()
                            .filter(|f| !f.is_empty())
                            .map(|f| format!(" (--{f})"))
                            .unwrap_or_default();
                        format!("{}{}", p.reason, flag_suffix)
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                let field_suffix = field.map(|f| format!(" (field: {f})")).unwrap_or_default();
                return Some(format!(
                    "bee CLI-shape guard: \"{}\" does not match {}'s schema — {detail}{field_suffix}. \
Correction: run `{}` with the required parameters (see `bee --help --json`).",
                    command.trim(),
                    entry.name,
                    entry.invoke,
                ));
            }
            break; // this segment resolved to one bee-cli call; move to the next segment
        }
    }
    None
}

/// Loads the registry bridge and runs check (d), or skips it VISIBLY: a
/// stale snapshot logs a `cli-shape-registry-stale` coverage gap (the
/// accepted rust-only allow window — node imports the live mjs and cannot
/// go stale); an unreadable/absent bridge logs a crash line, the rust twin
/// of the mjs hook's `catch (cliError) { logCrash(...) }` around its
/// dynamic registry import. Either skip fails open for this check only.
fn cli_shape_denial(ctx: &HookContext, root: &Path, store_root: &Path, command: &str) -> Option<String> {
    let dump_path = store_root.join(".bee").join("cache").join("command-registry.json");
    let source_path = store_root.join(".bee").join("bin").join("lib").join("command-registry.mjs");
    match load_registry(&dump_path, &source_path) {
        Ok(Registry::Fresh(dump)) => check_cli_shape(command, &dump.entries),
        Ok(Registry::Stale { embedded_sha256, live_sha256, .. }) => {
            adapter::log_coverage_gap(
                Some(root),
                HOOK_NAME,
                "cli-shape-registry-stale",
                &format!(
                    "command-registry.json snapshot (source_sha256 {embedded_sha256}) no longer matches the live \
command-registry.mjs ({live_sha256}) — CLI-shape validation skipped; regenerate with `node scripts/dump_command_registry.mjs`"
                ),
                ctx.source.as_deref(),
            );
            None
        }
        Err(err) => {
            adapter::log_crash(
                Some(root),
                HOOK_NAME,
                &format!("cli-shape registry bridge unavailable: {err}"),
                ctx.source.as_deref(),
            );
            None
        }
    }
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
