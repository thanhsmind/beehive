// the companion hook
//
// Split out of the single 4.2k-line verbs/worktree.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_roots_core, Resolution};
use crate::verbs::reservations::{js_numberify, js_trim, now_iso, parse_flags, FlagV, Flags};
use crate::verbs::workspace_store as ws;
use crate::verbs::{emit_no_root_error, record_timing};
use crate::{jsjson, lock};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

// ─── worktree-companion-hook (worktree-store.mjs) ─────────────────────────
//
// The companion pair — `runCompanionStart` (called from inside
// createFeatureWorktree's post-add block) and `teardownCompanionIfPresent`
// (called from mergeFeatureWorktreeStage, after every zero-mutation refusal
// has cleared and immediately before the merge is staged). Both were the last
// reason `worktree new --with-companion` and a companion `worktree merge`
// delegated: each one MUTATES (spawns a project-configured child, creates or
// unlinks a real symlink) at a point where nothing can fall back any more.
//
// bee never hardcodes what the companion tool is: `commands.
// worktree_companion_start` / `_mount` / `_end` in the host project's own
// `.bee/config.json` hold every tool-specific fact, and the ONLY contract on
// the start command's stdout is JSON carrying a non-empty `worktreePath`
// (plus an optional `sessionId`, carried through to the marker for `merge` to
// substitute into `_end`).

/// worktree-store.mjs COMPANION_MARKER_REL — `path.join('.bee',
/// 'companion-session.json')`, so the separator is the platform's. It is
/// deliberately NOT under `.bee/runtime/` (which is gitignored everywhere),
/// which is exactly why merge has to exclude it from the dirty-check by git
/// pathspec rather than rely on it already being gone.
pub(crate) fn companion_marker_rel() -> String {
    format!(".bee{MAIN_SEPARATOR}companion-session.json")
}

pub(crate) fn companion_marker_file(worktree_root: &Path) -> PathBuf {
    worktree_root.join(".bee").join("companion-session.json")
}

/// `spawnSync(command, { cwd, shell: true, encoding: 'utf8' })` with Node's
/// own null semantics for a spawn that never launched — the same GitOut shape
/// `run_git` produces, so the `(stderr || stdout || '').trim() || …` fallback
/// chains are shared rather than re-spelled.
///
/// No `shell_launchable()` pre-check is needed for either companion command:
/// unlike `runVerifyChild`, whose spawn-`error` event surfaces libuv's own
/// `spawn cmd.exe ENOENT` text, spawnSync's failure here collapses to
/// `status: null` with null pipes, which both call sites render as the fully
/// deterministic `(exit null): (no output)`.
pub(crate) fn shell_sync(command: &str, cwd: &Path) -> GitOut {
    match shell_child(command)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .output()
    {
        Ok(out) => GitOut {
            status: out.status.code(),
            stdout: Some(String::from_utf8_lossy(&out.stdout).into_owned()),
            stderr: Some(String::from_utf8_lossy(&out.stderr).into_owned()),
        },
        Err(_) => GitOut { status: None, stdout: None, stderr: None },
    }
}

/// `String.prototype.slice(0, n)` — UTF-16 code units, not chars or bytes.
pub(crate) fn js_slice_utf16(s: &str, n: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().collect();
    if units.len() <= n {
        return s.to_string();
    }
    String::from_utf16_lossy(&units[..n])
}

/// `haystack.replace('<needle>', replacement)` with a STRING pattern: the
/// FIRST occurrence only, and `$`-substitution patterns in the replacement are
/// honored exactly as JS does (`$$`, `$&`, `` $` ``, `$'`; `$n` is left
/// literal because a string pattern has no capture groups).
pub(crate) fn js_replace_first(haystack: &str, needle: &str, replacement: &str) -> String {
    let Some(at) = haystack.find(needle) else {
        return haystack.to_string();
    };
    let prefix = &haystack[..at];
    let suffix = &haystack[at + needle.len()..];
    let mut expanded = String::new();
    let mut chars = replacement.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '$' {
            expanded.push(c);
            continue;
        }
        match chars.peek() {
            Some('$') => {
                chars.next();
                expanded.push('$');
            }
            Some('&') => {
                chars.next();
                expanded.push_str(needle);
            }
            Some('`') => {
                chars.next();
                expanded.push_str(prefix);
            }
            Some('\'') => {
                chars.next();
                expanded.push_str(suffix);
            }
            _ => expanded.push('$'),
        }
    }
    format!("{prefix}{expanded}{suffix}")
}

/// worktree-store.mjs validateCompanionMountPath — a typed, ZERO-MUTATION
/// refusal, same posture as every other pre-check in
/// createFeatureWorktreeLocked. The value becomes a symlink target INSIDE the
/// new worktree, so an absolute path or a `..` segment would place (or escape)
/// it somewhere the worktree does not own.
pub(crate) fn validate_companion_mount_path(mount_path: &str) -> Result<String, CErr> {
    if js_trim(mount_path).is_empty() {
        return Err(refuse(
            "WORKTREE_COMPANION_CONFIG_INVALID",
            format!(
                "commands.worktree_companion_mount must be a non-empty relative path string, got {}.",
                jsjson::stringify(&Value::String(mount_path.to_string()))
            ),
        ));
    }
    let normalized = js_trim(mount_path).to_string();
    if js_path_is_absolute(&normalized) || normalized.split(['\\', '/']).any(|seg| seg == "..") {
        return Err(refuse(
            "WORKTREE_COMPANION_CONFIG_INVALID",
            format!(
                "commands.worktree_companion_mount {} must be a relative path inside the worktree (no leading \"/\" and no \"..\" segments).",
                jsjson::stringify(&Value::String(normalized.clone()))
            ),
        ));
    }
    Ok(normalized)
}

/// `path.isAbsolute` — the win32 flavor on win32 (a leading separator, or a
/// drive letter FOLLOWED by a separator; `C:foo` is drive-relative, not
/// absolute), the posix one elsewhere.
pub(crate) fn js_path_is_absolute(p: &str) -> bool {
    let b = p.as_bytes();
    if b.is_empty() {
        return false;
    }
    if cfg!(windows) {
        if b[0] == b'/' || b[0] == b'\\' {
            return true;
        }
        b.len() > 2
            && b[0].is_ascii_alphabetic()
            && b[1] == b':'
            && (b[2] == b'/' || b[2] == b'\\')
    } else {
        b[0] == b'/'
    }
}

/// worktree-store.mjs runCompanionStart. Runs with `mainRoot` as cwd — the
/// same root the command was resolved from, so the configured command owns its
/// own `cd` into whatever nested tree it isolates (mirroring `commands.test`).
///
/// `Err(message)` is folded by the caller into the SAME post-add rollback
/// ladder as any other failure after `git worktree add` succeeded: a worktree
/// is never left created-but-half-configured.
///
/// ONE DELIBERATE DIVERGENCE (cutover class — C2 is retired once Node is
/// gone). Node's unparseable-stdout arm interpolates V8's own `JSON.parse`
/// message; serde's message goes there instead. Every other byte of that
/// sentence — including the 500-UTF-16-unit raw-stdout tail — is Node's. The
/// symlink arm's uv message is approximated the same way `node_fs_error_message`
/// already approximates elsewhere in this file; its errno CLASS (EPERM for a
/// win32 host without SeCreateSymbolicLinkPrivilege) is exact.
pub(crate) fn run_companion_start(
    main_root: &Path,
    worktree_root: &Path,
    companion_start_command: &str,
    mount_path: &str,
) -> Result<Value, String> {
    let spawned = shell_sync(companion_start_command, main_root);
    if spawned.status != Some(0) {
        return Err(format!(
            "commands.worktree_companion_start failed (exit {}): {}",
            spawned.status_disp(),
            spawned.no_output_text()
        ));
    }
    let stdout = spawned.stdout.clone().unwrap_or_default();
    let parsed: Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "commands.worktree_companion_start must print JSON with a \"worktreePath\" field to stdout — got unparseable output ({e}). Raw stdout: {}",
                js_slice_utf16(&stdout, 500)
            ))
        }
    };
    let worktree_path = match parsed.get("worktreePath") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            return Err(format!(
                "commands.worktree_companion_start's JSON output must include a non-empty \"worktreePath\" string — got {}.",
                jsjson::stringify(&parsed)
            ))
        }
    };
    let session_id = match parsed.get("sessionId") {
        Some(Value::String(s)) if !s.is_empty() => Value::String(s.clone()),
        _ => Value::Null,
    };

    let mount_full_path = js_path_join(worktree_root, mount_path);
    if let Some(dir) = mount_full_path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| node_fs_error_message(&e, "mkdir", dir))?;
    }
    symlink_dir(&worktree_path, &mount_full_path)
        .map_err(|e| node_symlink_error_message(&e, &worktree_path, &mount_full_path))?;

    let marker_path = companion_marker_file(worktree_root);
    if let Some(dir) = marker_path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| node_fs_error_message(&e, "mkdir", dir))?;
    }
    let marker = json!({
        "sessionId": session_id,
        "worktreePath": worktree_path,
        "mountPath": mount_path,
    });
    std::fs::write(
        &marker_path,
        format!("{}\n", jsjson::stringify_pretty(&marker)),
    )
    .map_err(|e| node_fs_error_message(&e, "open", &marker_path))?;

    Ok(marker)
}

/// worktree-store.mjs readCompanionMarker — a bare `JSON.parse(readFileSync)`
/// in a try, so a missing OR unparseable marker both read as "no companion
/// here". A parsed FALSY value (`null`, `false`, `0`, `""`) is treated as
/// absent too: every consumer guards with `if (!marker)` / `companionMarker ?`.
pub(crate) fn read_companion_marker(worktree_root: &Path) -> Option<Value> {
    let raw = std::fs::read(companion_marker_file(worktree_root)).ok()?;
    let parsed: Value = serde_json::from_slice(&raw).ok()?;
    if js_truthy(&parsed) {
        Some(parsed)
    } else {
        None
    }
}

/// JS truthiness of a parsed JSON value (an absent key is the caller's None).
pub(crate) fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// `marker.mountPath` as the string every downstream use requires.
///
/// EXPLICIT NATIVE REFUSAL (no Node original). A marker that parses to a
/// truthy value WITHOUT a string `mountPath` makes the .mjs reach
/// `mountPath.replace(...)` / `path.join(root, undefined)` and die with a V8
/// `TypeError: Cannot read properties of undefined (reading 'replace')`, which
/// bee.mjs's dispatcher then surfaces verbatim and integration-queue.mjs
/// persists into the queue record. That text cannot be reproduced and can no
/// longer be delegated, so the shape becomes a typed, zero-mutation refusal
/// that says what is actually wrong. Reached only by a hand-edited or
/// truncated marker: `runCompanionStart` always writes all three fields.
pub(crate) fn companion_mount_path(marker: &Value) -> Result<String, MErr> {
    match marker.get("mountPath") {
        Some(Value::String(s)) => Ok(s.clone()),
        other => Err(refuse_merge(
            "WORKTREE_MERGE_COMPANION_MARKER_INVALID",
            format!(
                "the companion marker at .bee/companion-session.json has no usable \"mountPath\" string (got {}) — merge cannot exclude the mounted symlink from the worktree dirty-check, and refuses rather than guess. FIX: repair or delete the marker (and unlink the mount by hand), then retry.",
                jsjson::stringify(&other.cloned().unwrap_or(Value::Null))
            ),
        )),
    }
}

/// worktree-store.mjs teardownCompanionIfPresent. Runs ONLY after every
/// zero-mutation refusal has cleared and immediately before the merge is
/// staged — running it earlier destroyed the mount even for a merge about to
/// be refused; running it later would let the companion session outlive a
/// merge attempt that is actually proceeding.
///
/// Never throws: a missing/failed `_end` command is carried as `.warning` on
/// the returned object, and the symlink + marker are removed best-effort
/// either way. No flag gates it — the marker's presence IS the signal.
pub(crate) fn teardown_companion_if_present(
    main_root: &Path,
    worktree_root: &Path,
    companion_end_command: Option<&str>,
    marker: Option<&Value>,
) -> Option<Value> {
    let marker = marker?;
    let mut warning: Option<String> = None;
    if let Some(command) = companion_end_command {
        // `companionEndCommand.replace('<id>', marker.sessionId || '')` — a
        // falsy sessionId (absent, null, '', 0, false) substitutes the empty
        // string; anything else goes through ToString.
        let replacement = match marker.get("sessionId") {
            Some(v) if js_truthy(v) => jsjson::js_to_string(v),
            _ => String::new(),
        };
        let substituted = js_replace_first(command, "<id>", &replacement);
        let spawned = shell_sync(&substituted, main_root);
        if spawned.status != Some(0) {
            warning = Some(format!(
                "commands.worktree_companion_end failed (exit {}): {} — the mounted symlink was still removed so the merge itself is not blocked; the companion session may need manual teardown.",
                spawned.status_disp(),
                spawned.no_output_text()
            ));
        }
    } else {
        warning = Some(
            "a companion marker exists on this worktree but commands.worktree_companion_end is not configured — the mounted symlink was removed so the merge is not blocked, but the companion session (if the tool has one) was never explicitly ended."
                .to_string(),
        );
    }

    // Both unlinks are best-effort: already gone, or never a real symlink —
    // either way the dirty-check that already ran is the authoritative signal.
    if let Some(Value::String(mount)) = marker.get("mountPath") {
        unlink_maybe_dir_symlink(&js_path_join(worktree_root, mount));
    }
    unlink_maybe_dir_symlink(&companion_marker_file(worktree_root));

    // Node's key order: { ended, sessionId, warning } — `warning: undefined`
    // is dropped by JSON.stringify, so an ended-cleanly companion carries only
    // the first two keys.
    let mut out = Map::new();
    out.insert("ended".into(), Value::Bool(warning.is_none()));
    // `sessionId: marker.sessionId || null` — the raw value when truthy.
    let session_id = match marker.get("sessionId") {
        Some(v) if js_truthy(v) => v.clone(),
        _ => Value::Null,
    };
    out.insert("sessionId".into(), session_id);
    if let Some(w) = warning {
        out.insert("warning".into(), Value::String(w));
    }
    Some(Value::Object(out))
}
