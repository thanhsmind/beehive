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

// ─── unresolvable shell syntax (guard-refusal-wording D1/D2) ─────────────

/// D1: a raw target token still carrying unexpanded shell syntax (`$VAR`,
/// `${VAR}`, or a backquote command substitution) was never expanded by a
/// shell — the guard only ever sees the literal characters. Resolving it as
/// though it were a plain relative path produces a fake in-repo path (e.g.
/// `$WT/foo` lexically "resolves" under cwd), so the Bash target loop
/// (main.rs) must classify it unresolvable BEFORE it is walked as a path.
/// D4: this classification is Bash-surface-only — it is NOT applied inside
/// the shared resolvers (`canonical_rel_path`, `resolve_target_realpath`),
/// because Edit/Write/MultiEdit `file_path` and apply_patch targets are
/// literal strings no shell ever expands (a file named `Foo$Bar.java` is a
/// valid literal target there).
pub(crate) fn has_unexpanded_shell_syntax(raw: &str) -> bool {
    raw.contains('$') || raw.contains('`')
}

/// D3: an echoed raw token can carry embedded ASCII control characters (a
/// double-quoted Bash argument may hold a literal newline via `$'...'` or an
/// actual embedded line break) — echoed unbounded, that token could inject
/// message-shaped lines into stderr. It can also run arbitrarily long. This
/// strips every ASCII control character (0x00-0x1F, 0x7F) and bounds the
/// result to 120 chars, appending an ellipsis when truncated, so an echoed
/// token can never inject fake lines or blow past a one-glance refusal.
pub(crate) fn bound_echoed_token(raw: &str) -> String {
    const MAX_CHARS: usize = 120;
    let cleaned: String = raw.chars().filter(|c| !c.is_ascii_control()).collect();
    if cleaned.chars().count() > MAX_CHARS {
        let truncated: String = cleaned.chars().take(MAX_CHARS).collect();
        format!("{truncated}…")
    } else {
        cleaned
    }
}

/// D2/D3: the unresolvable-target refusal for a shell-syntax token — distinct
/// from both `GENERIC_BASH_CONTAINMENT_MESSAGE` (a resolved-but-not-contained
/// target) and the gate's "writing X is blocked" sentences (a resolved,
/// in-repo target the phase itself denies). This message never lets the raw
/// token stand in for a path: it names the resolution failure and quotes the
/// token as the shell fragment it is — bounded and control-char-stripped via
/// `bound_echoed_token` so the echoed token can never inject message-shaped
/// lines into stderr. D3 also widens the FIX sentence: a `$` in the token
/// need not be an unexpanded variable — it may be a literal filename that
/// happens to contain a dollar sign, and that possibility gets its own
/// remedy rather than only ever prescribing variable expansion.
pub(crate) fn unresolvable_bash_target_message(raw: &str) -> String {
    let bounded = bound_echoed_token(raw);
    format!(
        "bee write guard denied Bash: the target \"{bounded}\" could not be resolved — it still carries \
unexpanded shell syntax ($VAR or a backquote command substitution) the guard cannot see through, so it is \
refused rather than risking an unchecked write. \
FIX: expand the variable or command substitution yourself before invoking the write, or pass a plain in-worktree \
path — if this is actually a literal filename that happens to contain a dollar sign rather than a variable, \
escape it (\\$) or quote it so the shell never expands it."
    )
}

// ─── brace expansion and cd opacity (write-guard-hardening D4) ─────────────
//
// Two more shell rewrites the guard cannot see through: a brace group the
// shell expands into several literal words before the write-verb ever sees
// it (`{a,b}`, `{1..9}`), and a `cd` earlier in a compound command that
// moves the shell's working directory out from under every target this
// extractor resolves relative to the ORIGINAL cwd. Both are Bash-surface-
// only, same reasoning as `has_unexpanded_shell_syntax` above — literal
// Edit/Write/apply_patch file_path values are never shell-rewritten.
//
// Plain glob characters (`*`, `?`, `[`) are deliberately NOT classified
// here — an `rm *.log` style command is everyday usage and its current
// (unchanged) behavior stands; only a brace group is in scope.

/// D4: true when `raw` carries a `{...}` group containing a comma or a `..`
/// range — the two shapes bash actually expands (`{a,b,c}`, `{1..9}`,
/// `{a..z}`). A singleton group with neither (`{foo}`) is not expanded by
/// bash and is left alone. The tokenizer already collapsed quoted and
/// unquoted spans into the same token by the time this runs (`tokenize` /
/// `tokenize_deep`), so a QUOTED literal like `'sta{t,t}e.json'` — a real
/// filename, never expanded by the shell — is classified the same as the
/// unquoted form and denied. That is an accepted fail-closed false
/// positive, consistent with the existing `$`/backquote classification
/// above (a literal `$FOO` filename is denied the same way a real unexpanded
/// variable would be).
pub(crate) fn has_brace_expansion(raw: &str) -> bool {
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '{' {
            if let Some(close_offset) = chars[i + 1..].iter().position(|&c| c == '}') {
                let inner: String = chars[i + 1..i + 1 + close_offset].iter().collect();
                if inner.contains(',') || inner.contains("..") {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// D4: the unresolvable-target refusal for a brace-expansion token —
/// sibling of `unresolvable_bash_target_message`, same bounding/stripping
/// via `bound_echoed_token`.
pub(crate) fn brace_expansion_bash_target_message(raw: &str) -> String {
    let bounded = bound_echoed_token(raw);
    format!(
        "bee write guard denied Bash: the target \"{bounded}\" could not be resolved — it carries brace \
expansion ({{a,b}} or {{1..9}}) the shell rewrites into several literal targets before the write-verb ever runs, \
which the guard cannot see through, so it is refused rather than risking an unchecked write. \
FIX: expand the brace group yourself into the literal path(s) and invoke the write once per path, or pass a \
plain in-worktree path — if this is actually a literal filename containing brace characters rather than an \
expansion, escape or quote it so the shell never expands it."
    )
}

/// D4: the unresolvable-target refusal for a target that follows a `cd`
/// earlier in the same compound Bash command — the shell's working
/// directory has moved by the time this target's write runs, so the
/// guard's cwd-relative resolution of it can no longer be trusted.
pub(crate) fn cd_opaque_bash_target_message(raw: &str) -> String {
    let bounded = bound_echoed_token(raw);
    format!(
        "bee write guard denied Bash: the target \"{bounded}\" could not be resolved — a `cd` earlier in this \
compound command moves the shell's working directory, so this target no longer resolves against the directory \
the guard checked it against, and it is refused rather than risking an unchecked write. \
FIX: run the write in its own Bash call without a preceding cd, or use a path that does not depend on the \
shell having changed directory."
    )
}

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

/// Distinguishes "the grants registry is present but unparseable" (fail
/// open — never guessed at, the pre-existing carve-out) from "the registry
/// is absent, or present and simply carries no entry for this feature"
/// (ordinary "no grants recorded" territory, which the no-grant deny arm
/// below is free to act on). Only a present-but-not-a-JSON-object file
/// counts as corrupt; a missing file reads as "no grants recorded", never
/// corruption.
pub(crate) fn worktree_grants_registry_corrupt(main_root: &str) -> bool {
    let file = Path::new(main_root)
        .join(".bee")
        .join("runtime")
        .join("worktree-grants.json");
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(_) => return false,
    };
    !matches!(serde_json::from_str::<Value>(&text), Ok(Value::Object(_)))
}

/// provenance: bee-write-guard.mjs findFeatureWorktreeGrant, split three ways
/// (docs/knowledge/patterns/20260713-a-guard-that-tests-one-state-is-a.md —
/// one None standing for three different states was exactly the hole here).
/// `NotFound` is the only state the no-grant deny arm may act on: every
/// registered id resolved cleanly and none of them named `feature`.
/// `Unresolvable` covers the two states a caller must never read as "no
/// grant" — a registered grant whose worktree directory no longer resolves
/// (resolve_granted_worktree_root → None), and a worktree that DOES resolve
/// but whose identity file is missing or unparseable
/// (read_worktree_recorded_feature → None) — either could have been the
/// grant for `feature`; the guard just can't tell, so it must not deny.
pub(crate) enum FeatureWorktreeGrant {
    Found(String, String),
    NotFound,
    Unresolvable,
}

/// provenance: bee-write-guard.mjs findFeatureWorktreeGrant.
pub(crate) fn find_feature_worktree_grant(main_root: &str, feature: &str) -> R<FeatureWorktreeGrant> {
    let mut unresolvable = false;
    for id in read_granted_worktree_ids(main_root) {
        let worktree_root = match resolve_granted_worktree_root(main_root, &id)? {
            Some(r) => r,
            None => {
                unresolvable = true;
                continue;
            }
        };
        match read_worktree_recorded_feature(&worktree_root) {
            Some(f) if f == feature => return Ok(FeatureWorktreeGrant::Found(id, worktree_root)),
            Some(_) => {} // resolved and readable, definitively a different feature
            None => unresolvable = true,
        }
    }
    if unresolvable {
        Ok(FeatureWorktreeGrant::Unresolvable)
    } else {
        Ok(FeatureWorktreeGrant::NotFound)
    }
}

/// provenance: bee-write-guard.mjs worktreeFirstExemptRel.
///
/// `other_live_session` gates ONLY the blanket `.md` clause (cell
/// dll-1): solo, every `.md` path stays exempt, byte-identical to
/// today; with a live peer present, a bare `.md` path outside the
/// prefix list below is no longer exempt — worktree-first can now see
/// it as an offender. The `rel.is_empty()` / `"**"` sentinels and the
/// prefix list itself stay unconditional either way; this function has
/// exactly one production caller, `check_worktree_first`'s offender
/// scan, which threads the same liveness fact it already computed once.
pub(crate) fn worktree_first_exempt_rel(rel: &str, other_live_session: bool) -> bool {
    if rel.is_empty() {
        return true;
    }
    if rel == "**" {
        return true;
    }
    if rel.ends_with(".md") && !other_live_session {
        return true;
    }
    // trun-5: kept on the INTAKE list (unchanged, blanket `docs/`) — this
    // exemption already independently allows every `*.md` above while solo,
    // and the brief for this cell says to leave this consumer's behavior
    // byte-for-byte unchanged rather than tie it to the new gated-phase
    // boundary.
    GATE_ALLOWED_PREFIXES_INTAKE.iter().any(|prefix| {
        if let Some(bare) = prefix.strip_suffix('/') {
            rel == bare || rel.starts_with(prefix)
        } else {
            rel == *prefix
        }
    })
}

/// provenance: bee-write-guard.mjs checkWorktreeFirstDenial.
///
/// `record` is the ACTING record — the lane record for a lane-bound session,
/// the default state.json otherwise (the same resolution `check_write` uses
/// via `resolve_write_record`); it is never the raw default state.json for a
/// lane-bound session. Every carve-out below is a fail-open bound, narrowest
/// first: lane "docs" never fires on EITHER arm while no other live session
/// is present (AGENTS.md gives main integration and release work, plus
/// docs-lane and a solo tiny fix when no other session is live — cell
/// dll-1, same condition as tiny's below), lane "tiny" never fires on the
/// NEW no-grant arm only (AGENTS.md's solo tiny fix — see that arm's own
/// carve-out comment below for the "solo" gap and why a feature already
/// holding a granted worktree does not get this exemption), and a
/// missing/empty route on the acting record is "no opinion" — never guessed
/// at.
///
/// The granted-worktree arm below is phase-independent — byte-for-byte the
/// pre-existing refusal (git show 96db1a33^), which predates the "swarming"
/// phase gate entirely. Only the NEW no-grant arm is phase-gated: it exists
/// for the live swarming lane alone, so a phase other than "swarming" never
/// takes it — everywhere else (reviewing, planning, scribing) the
/// pre-existing granted refusal keeps firing exactly as it always did.
///
/// `session_id` is the acting session's own id (main.rs's
/// `session_id.as_deref()`, the same value `resolve_write_record` already
/// takes) — used below both as the docs-lane gate's own liveness fact and,
/// unchanged, as the exclusion for the no-grant arm's "tiny" carve-out, so a
/// session never counts itself as the other live session that would take
/// either carve-out away. `main_root` is resolved once, up front — earlier
/// than the docs gate needed it before this cell — because the liveness
/// check (`is_concurrent_mode`, computed exactly once and reused by both
/// gates) needs it too; if resolution itself fails, that is read the same
/// as every other arm below reads it: no opinion, fail open (`Ok(None)`).
pub(crate) fn check_worktree_first(
    worktree_resolution: &str,
    root: &str,
    store_root: &Path,
    record: &Map<String, Value>,
    rel_paths: &[String],
    session_id: Option<&str>,
) -> R<Option<String>> {
    if worktree_resolution != "ordinary" {
        return Ok(None);
    }
    let feature = match record.get("feature") {
        Some(Value::String(f)) if !f.is_empty() => f.clone(),
        _ => return Ok(None),
    };
    let lane = match record.get("route") {
        Some(Value::Object(route)) => match route.get("lane") {
            Some(Value::String(l)) if !l.is_empty() => l.clone(),
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };
    // Hoisted ahead of the docs gate below (cell dll-1): resolution failure
    // reads exactly as it did in its old position — no opinion, fail open.
    let main_root = match realpath_any(root) {
        Some(m) => m,
        None => return Ok(None),
    };
    // The single liveness fact both the docs gate (immediately below) and
    // the tiny gate (further down, unchanged) share — computed once here,
    // never reimplemented, never called twice. Self-excluding via
    // `session_id`.
    //
    // CORRECTED at sfg-5 (this comment claimed a fail-open the code stopped
    // performing at sfg-4). The non-strict reader answers in two directions,
    // and they are not the same direction:
    //
    // - a MISSING or unreadable sessions DIRECTORY, and a session file whose
    //   JSON does not parse, both contribute no record — so they read as
    //   `false`, "no other live session", and the docs/tiny exemptions
    //   survive. That is the old fail-open, and it still holds.
    // - a session record that DOES parse but carries a `last_heartbeat` the
    //   reader cannot parse reads as a LIVE peer (sfg-4), so it removes those
    //   exemptions and CAN turn a permitted solo write into a refusal. That
    //   is deliberate: an unreadable byte is evidence about the byte, never
    //   that the session went away, and a guard never falls open on data it
    //   merely read. `heartbeat_stale` names the offending file and its
    //   remedy on stderr, so the refusal is never silent.
    let other_live_session = is_concurrent_mode(&main_root, session_id);
    // "docs" is exempt on BOTH arms — but only while no other live session
    // is present (cell dll-1, same condition as tiny's below). With a live
    // peer, docs work routes into a worktree like any other feature.
    if lane == "docs" && !other_live_session {
        return Ok(None);
    }
    let config = read_config(store_root)?;
    if config.get("worktree_first") == Some(&Value::String("off".into())) {
        return Ok(None);
    }
    let offender = match rel_paths
        .iter()
        .find(|rel| !worktree_first_exempt_rel(rel, other_live_session))
    {
        Some(o) => o.clone(),
        None => return Ok(None),
    };
    match find_feature_worktree_grant(&main_root, &feature)? {
        FeatureWorktreeGrant::Found(grant_id, grant_root) => {
            return Ok(Some(format!(
                "bee worktree-first guard: \"{offender}\" is a feature source write in the MAIN checkout, but the active \
feature \"{feature}\" (lane \"{lane}\") holds granted worktree \"{grant_id}\" — code-touching feature work \
lives in its worktree from the start; main stays clean for integration, docs-lane, and release work \
(docs/specs/worktree-first.md). FIX: open your session at {grant_root} and make this edit there, \
then land it from main with `bee worktree merge --id {grant_id}`. Deliberate override: set \
worktree_first: \"off\" in .bee/config.json to disable this refusal (a recorded, visible choice)."
            )));
        }
        FeatureWorktreeGrant::Unresolvable => {
            // One unreadable input must never become a confident refusal —
            // the same principle worktree_grants_registry_corrupt already
            // applies to the registry file, extended to a grant entry whose
            // worktree directory or identity file can't be read. Fails
            // open: this MIGHT be feature's own grant; the guard just can't
            // prove it, so it never claims the feature "holds no granted
            // worktree" — that claim would be factually false.
            return Ok(None);
        }
        FeatureWorktreeGrant::NotFound => {}
    }
    // From here the grant lookup is a clean, confident "no grant recorded
    // for this feature" — the NEW arm (wtf-1), which needs its own four
    // narrower gates before it may deny.
    //
    // "tiny" is exempt HERE, on the no-grant arm only — AGENTS.md's solo
    // tiny fix in main, read as "this work is small enough not to need a
    // worktree at all — while no other session is live." A feature that
    // already holds a granted worktree never reaches this arm (the granted
    // arm above returns first), so this exemption can never rescue a tiny
    // edit that collides with a live worktree — that case is exactly the
    // drift worktree-first exists to stop.
    //
    // Gated on the same shape as its cited source, state_group/workflows.rs
    // `is_code_touching_lane` (`lane == "tiny" && !other_live_session`,
    // pinned by verbs/state_group/tests.rs) — but via `is_concurrent_mode`
    // (store.rs), the self-excluding "another non-stale session exists"
    // predicate this hook module already carries, not that verb's
    // lane-display walk (`other_live_work_present`), which needs a
    // Path-rooted, Ex-returning session/lane reader this hook module does
    // not have. `is_concurrent_mode` is narrower — presence of a live
    // sibling session, not that session's own lane/phase liveness — a
    // conservative reading of the same rule: it can only deny a tiny write
    // that the canonical predicate would also deny, never the reverse. Gap
    // named at cell wtf-3, closed here at cell dmc-3.
    //
    // `other_live_session` is the same fact computed once, above, for the
    // docs gate (cell dll-1) — reused here, never recomputed: `session_id`
    // excludes the acting session itself, so a lone session never counts
    // its own heartbeat as "another live session".
    //
    // CORRECTED at sfg-5, same correction as the docs gate above: a missing
    // or corrupt sessions store still reads as `false` and still leaves this
    // tiny exemption standing, but an UNPARSEABLE HEARTBEAT in a record that
    // otherwise parses reads as a live peer and takes the exemption away
    // (sfg-4). The exemption's own source rule is "while no other session is
    // live", and a session the reader cannot date is not a session the reader
    // can call dead — so the narrow read is the honest one here too. The
    // warning `heartbeat_stale` queues names the file that caused it.
    if lane == "tiny" && !other_live_session {
        return Ok(None);
    }
    let phase = match record.get("phase") {
        Some(Value::String(p)) => p.clone(),
        _ => String::new(),
    };
    if phase != "swarming" {
        return Ok(None);
    }
    if worktree_grants_registry_corrupt(&main_root) {
        return Ok(None);
    }
    // `bee worktree new` refuses with WORKTREE_CALLER_NOT_ORDINARY outside a
    // git checkout (adapter.rs supports a .bee/onboarding.json root with no
    // .git at all) — never name that remedy where it cannot run. Such a
    // root can hold no grant either, so the granted arm above is unaffected.
    if !Path::new(&main_root).join(".git").exists() {
        return Ok(None);
    }
    // provenance: bee.mjs buildRouteWorktreeBlock's no-grant arm
    // (state_group/workflows.rs route_worktree_block) — same command,
    // same "code-touching ... MAIN checkout ... branches at feature
    // start (worktree-first)" framing, ported to the write-guard
    // refusal shape.
    Ok(Some(format!(
        "bee worktree-first guard: \"{offender}\" is a feature source write in the MAIN checkout, but the \
active feature \"{feature}\" (lane \"{lane}\") holds no granted worktree — lane \"{lane}\" is code-touching and \
this is the MAIN checkout — feature work branches at feature start (worktree-first). \
FIX: run `bee worktree new --feature {feature}`, then open your session at the printed worktree path and make \
this edit there. Deliberate override: set worktree_first: \"off\" in .bee/config.json to disable this refusal \
(a recorded, visible choice)."
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

/// What `is_shared_nested_checkout_target` found.
///
/// sfg-5 turned the old `bool` into three answers. The third one is the point:
/// the strict session scan this primitive opens with can hit a session file
/// that is present but unreadable, and that used to be `Err(Nd)` — a
/// DELEGATION, i.e. the whole write guard falling open on a real Edit or
/// Write. It is a refusal now, and it carries the file so the refusal can
/// name it.
pub(crate) enum SharedNested {
    /// Not a shared nested checkout (or no other live session).
    No,
    /// A shared nested checkout another live session can also reach.
    Yes,
    /// A session record on the strict concurrency path could not be read.
    UnreadableSession(PathBuf),
}

/// provenance: guards.mjs isSharedNestedCheckoutTarget (wcg-1/wcg-2,
/// Port-D4 controlRoot).
pub(crate) fn is_shared_nested_checkout_target(
    root: &str,
    abs_target: &str,
    exclude_session: Option<&str>,
    control_root: Option<&str>,
) -> R<SharedNested> {
    let concurrency_root = control_root.filter(|s| !s.is_empty()).unwrap_or(root);
    match is_concurrent_mode_strict(concurrency_root, exclude_session) {
        Ok(false) => return Ok(SharedNested::No),
        Ok(true) => {}
        Err(file) => return Ok(SharedNested::UnreadableSession(file)),
    }
    let root_real = match realpath_f2(root)? {
        Some(r) => r,
        None => return Ok(SharedNested::No),
    };
    if target_inside_verified_companion_mount(root, abs_target)? {
        return Ok(SharedNested::Yes);
    }
    if let Some(nested) = find_nested_checkout_dir(&root_real, abs_target)? {
        if !is_registered_submodule(&root_real, &nested)? {
            return Ok(SharedNested::Yes);
        }
    }
    Ok(SharedNested::No)
}

/// The refusal for a session record the strict scan cannot read (sfg-5).
///
/// DELIBERATE DEPARTURE from Node parity, and it is the whole reason this
/// function exists. Node's `readSession(strict)` throws, and the hook's typed
/// detection-error deny quotes a V8-worded crash log this port cannot
/// reproduce byte-for-byte. sfg-4 therefore left the branch DELEGATING rather
/// than approximate that wording — which meant one truncated
/// `.bee/sessions/<id>.json` switched the entire write guard off on a real
/// write. Matching a crash log is not worth a hole: a guard never falls open
/// on data it merely read. So bee refuses in its own words, and the words
/// carry what Node's crash log never did — the file, and how to clear it.
pub(crate) fn unreadable_session_refusal(rel: &str, file: &Path) -> String {
    format!(
        "bee shared-checkout guard: the session record \"{}\" is present but could not be read or \
parsed, so this guard cannot tell whether another live session can also reach \"{rel}\" — and a \
guard never falls open on data it merely read. Refusing this write instead of guessing. \
FIX: repair or delete that session file (`bee state session release` writes a clean record for a \
session that is finished), then retry.",
        file.display()
    )
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
