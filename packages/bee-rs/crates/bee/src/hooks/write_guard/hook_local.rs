// the hook-local helpers: worktree-first refusal, large-read guard, checkout detection
//
// Split out of the single 5.9k-line hooks/write_guard.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── hook-local helpers (provenance: bee-write-guard.mjs top half) ─────────

pub(crate) const GENERIC_CONTAINMENT_MESSAGE: &str =
    "bee write guard denied this target: it could not be canonically contained inside the physical worktree. \
FIX: use a plain in-worktree path without traversal, outside absolute paths, or symlink escapes.";

pub(crate) const GENERIC_BASH_CONTAINMENT_MESSAGE: &str =
    "bee write guard denied Bash: one or more extracted targets could not be canonically contained inside the physical worktree. \
FIX: use plain in-worktree paths without traversal, outside absolute paths, or symlink escapes.";

// ─── harness-owned surface allowlist (guard-hardening E1) ──────────────────

/// The harness-owned write surfaces exempt from the outside-root containment
/// deny (docs/history/guard-hardening/CONTEXT.md E1): the harness memory root
/// `<home>/.claude/projects/` and the harness scratchpad root
/// `<system-temp>/claude/`. Held as canonicalized absolute prefixes; a base
/// that cannot be resolved contributes NOTHING (fail-closed — the deny
/// stands). The struct takes injected bases so tests never mutate the
/// environment.
pub(crate) struct HarnessRoots {
    pub(crate) roots: Vec<String>,
}

impl HarnessRoots {
    /// Live detection: home the way this crate already resolves it
    /// (USERPROFILE || HOME on Windows, HOME elsewhere — onboard/util.rs and
    /// status_full/store.rs shape), temp via std::env::temp_dir().
    pub(crate) fn detect() -> HarnessRoots {
        let home = if cfg!(windows) {
            std::env::var_os("USERPROFILE")
                .or_else(|| std::env::var_os("HOME"))
                .map(PathBuf::from)
        } else {
            std::env::var_os("HOME").map(PathBuf::from)
        };
        HarnessRoots::from_bases(home, Some(std::env::temp_dir()))
    }

    /// Build from explicit bases (None / empty / unresolvable → that surface
    /// contributes nothing).
    pub(crate) fn from_bases(home: Option<PathBuf>, temp: Option<PathBuf>) -> HarnessRoots {
        let mut roots = Vec::new();
        if let Some(h) = home.filter(|p| !p.as_os_str().is_empty()) {
            let base = h.join(".claude").join("projects").to_string_lossy().into_owned();
            if let Some(r) = canonical_allowlist_root(&base) {
                roots.push(r);
            }
        }
        if let Some(t) = temp.filter(|p| !p.as_os_str().is_empty()) {
            let base = t.join("claude").to_string_lossy().into_owned();
            if let Some(r) = canonical_allowlist_root(&base) {
                roots.push(r);
            }
            // On unix the temp dir is SHARED, so the harness names its
            // scratchpad root per user: `<temp>/claude-<uid>`. `<temp>/claude`
            // alone missed it — `is_under_root` compares whole segments, and
            // `/tmp/claude-1000` is not under `/tmp/claude` — so a plain
            // scratchpad write was denied for the containment reason. The uid
            // is the CALLING user's, never a `claude-*` prefix match: another
            // user's scratchpad stays outside the exemption. Windows needs no
            // suffix — its temp dir is already per-user.
            #[cfg(unix)]
            {
                let uid = unsafe { libc::getuid() };
                let base = t.join(format!("claude-{uid}")).to_string_lossy().into_owned();
                if let Some(r) = canonical_allowlist_root(&base) {
                    roots.push(r);
                }
            }
        }
        HarnessRoots { roots }
    }
}

/// Canonicalize an allowlist base the same way resolve_target_realpath treats
/// targets: realpath of the deepest existing ancestor, lexical tail appended
/// (case/prefix handling identical to the containment walk). None =
/// unresolvable → the caller drops the root (fail-closed).
fn canonical_allowlist_root(p: &str) -> Option<String> {
    if np_check_modelable(p).is_err() || !np_is_absolute(p) {
        return None;
    }
    let lexical = np_resolve1(p).ok()?;
    let (ancestor, unresolved) = walk_existing_ancestor(&lexical)?;
    let ancestor_real = realpath_any(&ancestor)?;
    np_resolve_segments(&ancestor_real, &unresolved).ok()
}

/// guard-hardening E1: does this wall-failed target's RESOLVED location sit
/// inside a harness-owned surface? Consulted ONLY at the containment failure
/// sites in main.rs — the exemption bypasses the containment deny alone;
/// reservation/hold, gate-boundary, direct-edit, and secret checks keep their
/// existing order and reach by code order (an exempt target never enters
/// rel_paths, and in-repo targets never reach this check). Every resolution
/// failure answers false — the deny stands.
pub(crate) fn harness_allowlisted_target(
    harness: &HarnessRoots,
    root: &str,
    cwd: &str,
    raw: &Value,
) -> R<bool> {
    if harness.roots.is_empty() {
        return Ok(false);
    }
    let resolved = match resolve_target_realpath(cwd, root, raw)? {
        Some(t) => t,
        None => return Ok(false), // unresolvable target → fail-closed
    };
    for r in &harness.roots {
        if is_under_root(r, &resolved)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// provenance: bee-write-guard.mjs HOME_PREFIXED_TARGET_RE /
/// isHomePrefixedTarget (gmr-1).
pub(crate) fn is_home_prefixed(raw: &str) -> bool {
    let tail_after = |prefix: &str| -> Option<char> {
        raw.strip_prefix(prefix).and_then(|rest| rest.chars().next())
    };
    if let Some(rest) = raw.strip_prefix('~') {
        // ~[A-Za-z0-9._+-]* then a separator
        let mut chars = rest.chars();
        let mut c = chars.next();
        while let Some(ch) = c {
            if ch == '/' || ch == '\\' {
                return true;
            }
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '+' | '-') {
                c = chars.next();
                continue;
            }
            return false;
        }
        return false;
    }
    if matches!(tail_after("$HOME"), Some('/') | Some('\\')) {
        return true;
    }
    if matches!(tail_after("${HOME}"), Some('/') | Some('\\')) {
        return true;
    }
    false
}

/// provenance: bee-write-guard.mjs normalizeToolPath — replace(/\\(?!\s)/g,
/// path.sep): identity on Windows; on POSIX a backslash not followed by JS
/// whitespace becomes '/'.
pub(crate) fn normalize_tool_path(raw: &str) -> String {
    if cfg!(windows) {
        return raw.to_string();
    }
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c == '\\' {
            match chars.get(i + 1) {
                Some(&n) if js_is_ws(n) => out.push('\\'),
                _ => out.push('/'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// provenance: bee-write-guard.mjs lexicalRelPath.
pub(crate) fn lexical_rel_path(root: &str, cwd: &str, raw: Option<&Value>) -> R<Option<String>> {
    let raw_s = match raw {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    np_check_modelable(&raw_s)?;
    let base = if !cwd.is_empty() { cwd } else { root };
    let abs = if np_is_absolute(&raw_s) {
        np_resolve1(&raw_s)?
    } else {
        np_resolve2(base, &raw_s)?
    };
    let rel = np_relative(root, &abs)?;
    if rel.is_empty() || rel == "." || rel.starts_with("..") || np_is_absolute(&rel) {
        return Ok(None);
    }
    Ok(Some(rel.split(SEP).collect::<Vec<_>>().join("/")))
}

/// provenance: bee-write-guard.mjs canonicalRelPath.
pub(crate) fn canonical_rel_path(root: &str, cwd: &str, raw: Option<&Value>) -> R<Option<String>> {
    let raw_s = match raw {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    if is_home_prefixed(&raw_s) {
        return Ok(None);
    }
    let root_real = match realpath_any(root) {
        Some(r) => r,
        None => return Ok(None),
    };
    let normalized = normalize_tool_path(&raw_s);
    #[cfg(not(windows))]
    {
        // Foreign Windows spellings on a POSIX host cannot be safely mapped.
        let b = raw_s.as_bytes();
        let win_drive = b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/');
        if win_drive || raw_s.starts_with("\\\\") {
            return Ok(None);
        }
    }
    if !np_is_absolute(&normalized)
        && normalized.split(SEP).any(|s| s == "..")
    {
        return Ok(None);
    }
    np_check_modelable(&normalized)?;
    let cwd_base = if np_is_absolute(cwd) { cwd } else { root_real.as_str() };
    let lexical = if np_is_absolute(&normalized) {
        np_resolve1(&normalized)?
    } else {
        np_resolve2(cwd_base, &normalized)?
    };
    let (ancestor, unresolved) = match walk_existing_ancestor(&lexical) {
        Some(x) => x,
        None => return Ok(None),
    };
    let ancestor_real = match realpath_any(&ancestor) {
        Some(r) => r,
        None => return Ok(None),
    };
    let canonical = np_resolve_segments(&ancestor_real, &unresolved)?;
    let rel = np_relative(&root_real, &canonical)?;
    if rel.is_empty()
        || rel == "."
        || rel == ".."
        || rel.starts_with(&format!("..{}", SEP))
        || np_is_absolute(&rel)
    {
        return Ok(None);
    }
    Ok(Some(rel.split(SEP).collect::<Vec<_>>().join("/")))
}

/// provenance: bee-write-guard.mjs resolveTargetRealpath (catch-all flavor).
pub(crate) fn resolve_target_realpath(cwd: &str, root: &str, raw: &Value) -> R<Option<String>> {
    let raw_s = match raw {
        Value::String(s) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    if is_home_prefixed(&raw_s) {
        return Ok(None);
    }
    let normalized = normalize_tool_path(&raw_s);
    #[cfg(not(windows))]
    {
        let b = raw_s.as_bytes();
        let win_drive = b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/');
        if win_drive || raw_s.starts_with("\\\\") {
            return Ok(None);
        }
    }
    np_check_modelable(&normalized)?;
    let cwd_base = if np_is_absolute(cwd) { cwd } else { root };
    let lexical = if np_is_absolute(&normalized) {
        np_resolve1(&normalized)?
    } else {
        np_resolve2(cwd_base, &normalized)?
    };
    let (ancestor, unresolved) = match walk_existing_ancestor(&lexical) {
        Some(x) => x,
        None => return Ok(None),
    };
    let ancestor_real = match realpath_any(&ancestor) {
        Some(r) => r,
        None => return Ok(None),
    };
    Ok(Some(np_resolve_segments(&ancestor_real, &unresolved)?))
}

/// provenance: bee-write-guard.mjs lexicalAbsTarget (wcg-2 D1b).
pub(crate) fn lexical_abs_target(root: &str, cwd: &str, raw: &str) -> R<String> {
    let normalized = normalize_tool_path(raw);
    np_check_modelable(&normalized)?;
    let cwd_base = if np_is_absolute(cwd) { cwd } else { root };
    if np_is_absolute(&normalized) {
        np_resolve1(&normalized)
    } else {
        np_resolve2(cwd_base, &normalized)
    }
}

/// provenance: bee-write-guard.mjs isUnderRoot.
pub(crate) fn is_under_root(parent_real: &str, child_real: &str) -> R<bool> {
    if parent_real.is_empty() || child_real.is_empty() {
        return Ok(false);
    }
    let rel = np_relative(parent_real, child_real)?;
    Ok(rel.is_empty()
        || (rel != ".." && !rel.starts_with(&format!("..{}", SEP)) && !np_is_absolute(&rel)))
}

/// provenance: bee-write-guard.mjs readGitdirPointer (catch-all flavor).
pub(crate) fn read_gitdir_pointer(file: &Path, base: &str) -> R<Option<String>> {
    let raw = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    let mut raw = js_trim(&raw);
    if let Some(rest) = raw.strip_prefix("gitdir:") {
        raw = js_trim(rest);
    }
    if raw.is_empty() {
        return Ok(None);
    }
    let fixed = if cfg!(windows) { raw.to_string() } else { raw.replace('\\', "/") };
    Ok(Some(np_resolve2(base, &fixed)?))
}

/// provenance: bee-write-guard.mjs deriveCurrentWorktree.
pub(crate) fn derive_current_worktree(root: &str) -> R<Option<(String, String)>> {
    let marker = Path::new(root).join(".git");
    let stat = match std::fs::metadata(&marker) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    if !stat.is_file() {
        return Ok(None);
    }
    let gitdir = match read_gitdir_pointer(&marker, root)? {
        Some(g) => g,
        None => return Ok(None),
    };
    let worktrees_root = np_resolve2(&gitdir, "..")?;
    let common_git_dir = np_resolve2(&worktrees_root, "..")?;
    if np_basename(&worktrees_root) != "worktrees" || np_basename(&common_git_dir) != ".git" {
        return Ok(None);
    }
    let main_root = match realpath_any(&np_dirname(&common_git_dir)) {
        Some(m) => m,
        None => return Ok(None),
    };
    Ok(Some((main_root, np_basename(&gitdir))))
}

/// provenance: bee-write-guard.mjs resolveGrantedWorktreeRoot.
pub(crate) fn resolve_granted_worktree_root(main_root: &str, id: &str) -> R<Option<String>> {
    let gwd = Path::new(main_root).join(".git").join("worktrees").join(id);
    match std::fs::metadata(&gwd) {
        Ok(s) if s.is_dir() => {}
        _ => return Ok(None),
    }
    let gwd_s = gwd.to_string_lossy().into_owned();
    let forward = match read_gitdir_pointer(&gwd.join("gitdir"), &gwd_s)? {
        Some(f) => f,
        None => return Ok(None),
    };
    let worktree_root = np_dirname(&forward);
    let reverse = match read_gitdir_pointer(&Path::new(&worktree_root).join(".git"), &worktree_root)? {
        Some(r) => r,
        None => return Ok(None),
    };
    if np_resolve1(&reverse)? != np_resolve1(&gwd_s)? {
        return Ok(None);
    }
    Ok(realpath_any(&worktree_root))
}

/// provenance: bee-write-guard.mjs readGrantedWorktreeIds.
pub(crate) fn read_granted_worktree_ids(main_root: &str) -> Vec<String> {
    let file = Path::new(main_root)
        .join(".bee")
        .join("runtime")
        .join("worktree-grants.json");
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(m)) => m
            .iter()
            .filter(|(_, v)| **v == Value::Bool(true))
            .map(|(k, _)| k.clone())
            .collect(),
        _ => Vec::new(),
    }
}

/// provenance: bee-write-guard.mjs describeCrossWorktreeTarget (message-only
/// enrichment; every failure keeps the generic containment message).
pub(crate) fn describe_cross_worktree_target(root: &str, cwd: &str, raw: &Value) -> R<Option<String>> {
    let target_real = match resolve_target_realpath(cwd, root, raw)? {
        Some(t) => t,
        None => return Ok(None),
    };
    let current = derive_current_worktree(root)?;
    let main_root = match &current {
        Some((m, _)) => m.clone(),
        None => match realpath_any(root) {
            Some(m) => m,
            None => return Ok(None),
        },
    };
    if current.is_some() && is_under_root(&main_root, &target_real)? {
        return Ok(Some(
            "bee write guard denied this target: it could not be canonically contained inside the physical worktree — \
this path belongs to the main checkout, not this worktree. FIX: run this from a session rooted there."
                .to_string(),
        ));
    }
    for id in read_granted_worktree_ids(&main_root) {
        if let Some((_, cur_id)) = &current {
            if &id == cur_id {
                continue;
            }
        }
        if let Some(worktree_root) = resolve_granted_worktree_root(&main_root, &id)? {
            if is_under_root(&worktree_root, &target_real)? {
                return Ok(Some(format!(
                    "bee write guard denied this target: it could not be canonically contained inside the physical worktree — \
it resolves inside worktree \"{id}\". FIX: open a session with cwd={worktree_root} to work there, or merge it \
back from main via `bee worktree merge --id {id}`."
                )));
            }
        }
    }
    Ok(None)
}

// ─── worktree-first refusal (provenance: bee-write-guard.mjs §worktree-first,
// docs/specs/worktree-first.md §2) ─────────────────────────────────────────

/// provenance: bee-write-guard.mjs readWorktreeRecordedFeature (plain
/// try/catch parses — no readJson warn, so corrupt files just fall through).
pub(crate) fn read_worktree_recorded_feature(worktree_root: &str) -> Option<String> {
    let identity_file = Path::new(worktree_root)
        .join(".bee")
        .join("runtime")
        .join("worktree-identity.json");
    if let Ok(text) = std::fs::read_to_string(&identity_file) {
        if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&text) {
            if let Some(Value::String(f)) = m.get("feature") {
                if !f.is_empty() {
                    return Some(f.clone());
                }
            }
        }
    }
    let state_file = Path::new(worktree_root).join(".bee").join("state.json");
    if let Ok(text) = std::fs::read_to_string(&state_file) {
        if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&text) {
            if let Some(Value::String(f)) = m.get("feature") {
                if !f.is_empty() {
                    return Some(f.clone());
                }
            }
        }
    }
    None
}

/// provenance: bee-write-guard.mjs findFeatureWorktreeGrant.
pub(crate) fn find_feature_worktree_grant(main_root: &str, feature: &str) -> R<Option<(String, String)>> {
    for id in read_granted_worktree_ids(main_root) {
        if let Some(worktree_root) = resolve_granted_worktree_root(main_root, &id)? {
            if read_worktree_recorded_feature(&worktree_root).as_deref() == Some(feature) {
                return Ok(Some((id, worktree_root)));
            }
        }
    }
    Ok(None)
}

/// provenance: bee-write-guard.mjs worktreeFirstExemptRel.
pub(crate) fn worktree_first_exempt_rel(rel: &str) -> bool {
    if rel.is_empty() {
        return true;
    }
    if rel == "**" {
        return true;
    }
    if rel.ends_with(".md") {
        return true;
    }
    GATE_ALLOWED_PREFIXES.iter().any(|prefix| {
        if let Some(bare) = prefix.strip_suffix('/') {
            rel == bare || rel.starts_with(prefix)
        } else {
            rel == *prefix
        }
    })
}

/// provenance: bee-write-guard.mjs checkWorktreeFirstDenial.
pub(crate) fn check_worktree_first(
    worktree_resolution: &str,
    root: &str,
    store_root: &Path,
    state: &Map<String, Value>,
    rel_paths: &[String],
) -> R<Option<String>> {
    if worktree_resolution != "ordinary" {
        return Ok(None);
    }
    let feature = match state.get("feature") {
        Some(Value::String(f)) if !f.is_empty() => f.clone(),
        _ => return Ok(None),
    };
    let lane = match state.get("route") {
        Some(Value::Object(route)) => match route.get("lane") {
            Some(Value::String(l)) if !l.is_empty() => l.clone(),
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };
    if lane == "docs" {
        return Ok(None);
    }
    let config = read_config(store_root)?;
    if config.get("worktree_first") == Some(&Value::String("off".into())) {
        return Ok(None);
    }
    let offender = match rel_paths.iter().find(|rel| !worktree_first_exempt_rel(rel)) {
        Some(o) => o.clone(),
        None => return Ok(None),
    };
    let main_root = match realpath_any(root) {
        Some(m) => m,
        None => return Ok(None),
    };
    let (grant_id, grant_root) = match find_feature_worktree_grant(&main_root, &feature)? {
        Some(g) => g,
        None => return Ok(None),
    };
    Ok(Some(format!(
        "bee worktree-first guard: \"{offender}\" is a feature source write in the MAIN checkout, but the active \
feature \"{feature}\" (lane \"{lane}\") holds granted worktree \"{grant_id}\" — code-touching feature work \
lives in its worktree from the start; main stays clean for integration, docs-lane, and release work \
(docs/specs/worktree-first.md). FIX: open your session at {grant_root} and make this edit there, \
then land it from main with `bee worktree merge --id {grant_id}`. Deliberate override: set \
worktree_first: \"off\" in .bee/config.json to disable this refusal (a recorded, visible choice)."
    )))
}

// ─── large-read guard (provenance: bee-write-guard.mjs router-cost rc-1) ───

pub(crate) fn resolve_max_read_lines(config: &Map<String, Value>) -> f64 {
    match config.get("guards") {
        Some(g) if truthy(g) => match g.get("max_read_lines") {
            Some(Value::Number(n)) => {
                let f = n.as_f64().unwrap_or(f64::NAN);
                if f.is_finite() && f > 0.0 { f } else { 800.0 }
            }
            _ => 800.0,
        },
        _ => 800.0,
    }
}

pub(crate) fn check_read_size_denial(abs: &Path, label: &str, threshold: f64) -> Option<String> {
    let stat = std::fs::metadata(abs).ok()?;
    if !stat.is_file() {
        return None;
    }
    if stat.len() > 25 * 1024 * 1024 {
        return None;
    }
    let buffer = std::fs::read(abs).ok()?;
    if buffer.iter().take(8000).any(|&b| b == 0) {
        return None;
    }
    let mut count = buffer.iter().filter(|&&b| b == 10).count();
    if !buffer.is_empty() && *buffer.last().unwrap() != 10 {
        count += 1;
    }
    if (count as f64) < threshold {
        return None;
    }
    Some(format!(
        "bee read-size guard: \"{label}\" is {count} lines (threshold: {}) and this Read \
has neither `offset` nor `limit` — reading it unbounded would load the whole file into context. \
FIX: pass `limit` (and optionally `offset`) to read a slice, or dispatch a `bee-extract` worker to read the whole file.",
        jsjson::js_f64_to_string(threshold)
    ))
}

// ─── companion mount / memory root delegation gates ────────────────────────

/// provenance: bee-write-guard.mjs resolveCompanionMountedRelPath — consulted
/// only for a target that already failed containment. A present marker means
/// live symlink verification the port does not replicate → Nd; an absent
/// marker is the .mjs's own catch → null.
pub(crate) fn companion_mount_rel(root: &str) -> R<Option<String>> {
    let marker = Path::new(root).join(".bee").join("companion-session.json");
    if marker.exists() {
        return Err(Nd);
    }
    Ok(None)
}

/// provenance: bee-write-guard.mjs isMemoryRootHit / isDeclaredMemoryRootTarget
/// (gmr-3) — a declared guards.memory_root (non-empty string) engages marker
/// verification the port does not replicate → Nd; anything else is false.
pub(crate) fn memory_root_hit(store_root: &Path) -> R<bool> {
    let config = read_config(store_root)?;
    match config.get("guards") {
        Some(Value::Object(g)) => match g.get("memory_root") {
            Some(Value::String(s)) if !js_trim(s).is_empty() => Err(Nd),
            _ => Ok(false),
        },
        _ => Ok(false),
    }
}

// ─── shared nested/companion checkout detection (provenance: guards.mjs
// wcg-1 isSharedNestedCheckoutTarget + helpers; F2 error posture: ENOENT is
// silent, any other fs error is a JS throw → Nd, which delegates to Node's
// typed fail-closed detection-error refusal) ───────────────────────────────

pub(crate) fn has_git_node_f2(dir: &str) -> R<bool> {
    match std::fs::metadata(Path::new(dir).join(".git")) {
        Ok(_) => Ok(true),
        Err(e) if io_err_is_enoent(&e) => Ok(false),
        Err(_) => Err(Nd),
    }
}

pub(crate) fn resolve_existing_realpath_f2(abs: &str) -> R<Option<String>> {
    let mut cursor = abs.to_string();
    let mut unresolved: Vec<String> = Vec::new();
    loop {
        match realpath_f2(&cursor)? {
            Some(real) => {
                return Ok(Some(if unresolved.is_empty() {
                    real
                } else {
                    np_resolve_segments(&real, &unresolved)?
                }));
            }
            None => {
                let parent = np_dirname(&cursor);
                if parent == cursor {
                    return Ok(None);
                }
                unresolved.insert(0, np_basename(&cursor));
                cursor = parent;
            }
        }
    }
}

/// pub(crate) since the wcg-3 port (`crate::nested_checkout`): guards.mjs's
/// own comment says this verification "lives in exactly one place", shared by
/// the point-check (`target_inside_verified_companion_mount`) and the
/// directory-scan (`hasAnySharedNestedCheckout`). Widening it is what keeps
/// that true across the two Rust modules.
pub(crate) fn resolve_verified_companion_mount_real(root: &str) -> R<Option<String>> {
    let marker_file = Path::new(root).join(".bee").join("companion-session.json");
    let raw = match std::fs::read(&marker_file) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) if io_err_is_enoent(&e) => return Ok(None),
        Err(_) => return Err(Nd), // F2: propagates in Node
    };
    let marker: Value = serde_json::from_str(&raw).map_err(|_| Nd)?; // F2: corrupt marker throws
    let obj = match &marker {
        Value::Object(m) => m,
        _ => return Ok(None),
    };
    let declared = match obj.get("worktreePath") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    let mount = match obj.get("mountPath") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return Ok(None),
    };
    let declared_real = match realpath_f2(&declared)? {
        Some(d) => d,
        None => return Ok(None),
    };
    let live = Path::new(root).join(&mount).to_string_lossy().into_owned();
    let live_mount_real = match realpath_f2(&live)? {
        Some(l) => l,
        None => return Ok(None),
    };
    if declared_real != live_mount_real {
        return Ok(None);
    }
    Ok(Some(live_mount_real))
}

pub(crate) fn target_inside_verified_companion_mount(root: &str, abs_target: &str) -> R<bool> {
    let live = match resolve_verified_companion_mount_real(root)? {
        Some(l) => l,
        None => return Ok(false),
    };
    let target_real = match resolve_existing_realpath_f2(abs_target)? {
        Some(t) => t,
        None => return Ok(false),
    };
    is_under_root(&live, &target_real)
}

pub(crate) fn find_nested_checkout_dir(root_real: &str, abs_target: &str) -> R<Option<String>> {
    let mut cursor = abs_target.to_string();
    loop {
        let parent = np_dirname(&cursor);
        if parent == cursor {
            return Ok(None);
        }
        cursor = parent;
        let cursor_real = match realpath_f2(&cursor)? {
            Some(c) => c,
            None => continue, // does not exist yet — keep climbing
        };
        if cursor_real == root_real {
            return Ok(None);
        }
        if !is_under_root(root_real, &cursor_real)? {
            return Ok(None);
        }
        if has_git_node_f2(&cursor_real)? {
            return Ok(Some(cursor_real));
        }
    }
}

pub(crate) fn is_registered_submodule(root_real: &str, nested_real: &str) -> R<bool> {
    let content = match std::fs::read(Path::new(root_real).join(".gitmodules")) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) if io_err_is_enoent(&e) => return Ok(false),
        Err(_) => return Err(Nd),
    };
    for line in content.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        // /^\s*path\s*=\s*(.+?)\s*$/
        let after_ws = line.trim_start_matches(js_is_ws);
        let Some(after_path) = after_ws.strip_prefix("path") else { continue };
        let after_ws2 = after_path.trim_start_matches(js_is_ws);
        let Some(rest) = after_ws2.strip_prefix('=') else { continue };
        if rest.is_empty() {
            continue; // (.+?) needs at least one char
        }
        let cap = {
            let t = js_trim(rest);
            if t.is_empty() {
                // all-whitespace remainder: the lazy capture holds one ws char
                rest.chars().last().map(|c| c.to_string()).unwrap_or_default()
            } else {
                t.to_string()
            }
        };
        let entry = np_resolve2(root_real, &cap)?;
        if let Some(entry_real) = realpath_f2(&entry)? {
            if entry_real == nested_real {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// provenance: guards.mjs isSharedNestedCheckoutTarget (wcg-1/wcg-2,
/// Port-D4 controlRoot).
pub(crate) fn is_shared_nested_checkout_target(
    root: &str,
    abs_target: &str,
    exclude_session: Option<&str>,
    control_root: Option<&str>,
) -> R<bool> {
    let concurrency_root = control_root.filter(|s| !s.is_empty()).unwrap_or(root);
    if !is_concurrent_mode(concurrency_root, exclude_session, true)? {
        return Ok(false);
    }
    let root_real = match realpath_f2(root)? {
        Some(r) => r,
        None => return Ok(false),
    };
    if target_inside_verified_companion_mount(root, abs_target)? {
        return Ok(true);
    }
    if let Some(nested) = find_nested_checkout_dir(&root_real, abs_target)? {
        if !is_registered_submodule(&root_real, &nested)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// provenance: bee-write-guard.mjs sharedNestedCheckoutRefusal (wcg-2 D3/D4).
pub(crate) fn shared_nested_checkout_refusal(rel: &str) -> String {
    format!(
        "bee shared-checkout guard: \"{rel}\" is inside a nested checkout that another \
live session can also reach, and no verified companion mount covers it. \
Writing here can silently overwrite the other session's work — the exact \
failure this guard exists to prevent. \
FIX: open a FRESH companion worktree — run `bee worktree new --with-companion` \
to create a new worktree that mounts this shared checkout under a verified \
marker, then do this work there. The current worktree cannot be converted \
into a companion mount; you must create a new one."
    )
}
