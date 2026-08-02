// the git plumbing and skill sync
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

// ─── worktree-store.mjs: git plumbing ─────────────────────────────────────

/// worktree-store.mjs runGit: `spawnSync('git', args, {cwd, encoding:'utf8'})`
/// — an argv array (NO shell), inherited env, default pipes.
///
/// The three fields every call site reads, with Node's own null semantics: a
/// spawn that never launched (git off PATH) leaves `status`/`stdout`/`stderr`
/// ALL null, and `runGit` never inspects `.error`, so ENOENT is
/// indistinguishable from a non-zero exit at every call site — including the
/// `` `exit ${status}` `` fallbacks, which then render the literal
/// "exit null".
pub(crate) struct GitOut {
    pub(crate) status: Option<i32>,
    pub(crate) stdout: Option<String>,
    pub(crate) stderr: Option<String>,
}

impl GitOut {
    /// `${result.status}` — JS renders a null status as "null".
    pub(crate) fn status_disp(&self) -> String {
        self.status.map_or_else(|| "null".to_string(), |c| c.to_string())
    }

    /// `(stderr || stdout || '').trim() || '(no output)'` — the companion
    /// hooks' variant of the fallback chain below (same `||` semantics, a
    /// different tail). Fully deterministic even for a never-launched spawn:
    /// spawnSync leaves every field null there, so both halves fall through to
    /// the literal "(no output)".
    pub(crate) fn no_output_text(&self) -> String {
        let first = self
            .stderr
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| self.stdout.as_deref().filter(|s| !s.is_empty()))
            .unwrap_or("");
        let trimmed = js_trim(first);
        if trimmed.is_empty() {
            "(no output)".to_string()
        } else {
            trimmed.to_string()
        }
    }
    /// The `(stderr || stdout || '').trim() || `exit ${status}`` fallback
    /// chain three refusals share, byte-for-byte (JS `||` on '' is falsy).
    pub(crate) fn fail_text(&self) -> String {
        let first = self
            .stderr
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| self.stdout.as_deref().filter(|s| !s.is_empty()))
            .unwrap_or("");
        let trimmed = js_trim(first);
        if trimmed.is_empty() {
            format!("exit {}", self.status_disp())
        } else {
            trimmed.to_string()
        }
    }
}

pub(crate) fn run_git(cwd: &Path, args: &[&str]) -> GitOut {
    match std::process::Command::new("git").args(args).current_dir(cwd).output() {
        Ok(out) => GitOut {
            status: out.status.code(),
            // encoding:'utf8' — Node decodes the whole buffer lossily.
            stdout: Some(String::from_utf8_lossy(&out.stdout).into_owned()),
            stderr: Some(String::from_utf8_lossy(&out.stderr).into_owned()),
        },
        // spawnSync's error shape: every field null, `.error` never read.
        Err(_) => GitOut { status: None, stdout: None, stderr: None },
    }
}

/// isOrdinaryCheckout: a `.git` DIRECTORY. A `.git` FILE (linked worktree) or
/// any stat failure is false.
pub(crate) fn is_ordinary_checkout(root: &Path) -> bool {
    std::fs::metadata(root.join(".git")).map(|m| m.is_dir()).unwrap_or(false)
}

/// resolveBaseRefCommit: `git rev-parse --verify --end-of-options <ref>^{commit}`.
pub(crate) fn resolve_base_ref_commit(cwd: &Path, r#ref: &str) -> Option<String> {
    let spec = format!("{}^{{commit}}", r#ref);
    let result = run_git(cwd, &["rev-parse", "--verify", "--end-of-options", &spec]);
    if result.status != Some(0) {
        return None;
    }
    let sha = js_trim(result.stdout.as_deref().unwrap_or("")).to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
}

/// branchExists: `git rev-parse --verify --quiet refs/heads/<branch>`. A
/// spawn failure yields `status === null` → false, so the guard passes and
/// `git worktree add` becomes the real gate — reproduced exactly.
pub(crate) fn branch_exists(main_root: &Path, branch: &str) -> bool {
    let spec = format!("refs/heads/{branch}");
    run_git(main_root, &["rev-parse", "--verify", "--quiet", &spec]).status == Some(0)
}

/// `path.resolve` semantics for a single (possibly relative) segment:
/// absolute wins outright, otherwise join, then lexically normalize `.`/`..`.
pub(crate) fn js_path_resolve(base: &Path, segment: &str) -> PathBuf {
    let seg = Path::new(segment);
    let joined = if seg.is_absolute() { seg.to_path_buf() } else { base.join(seg) };
    let mut out = PathBuf::new();
    for comp in joined.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Node's uv-formatted fs error message, the ONE approximation this module
/// makes (documented in the header): only reachable inside `postAddMessage`,
/// i.e. after `git worktree add` already succeeded but `.git` could not be
/// read. Same class as verbs/state_group.rs's "prune's mid-loop rmSync
/// failure message is reconstructed from the errno class".
pub(crate) fn node_fs_error_message(err: &std::io::Error, syscall: &str, path: &Path) -> String {
    let (code, text) = node_errno_class(err);
    format!("{code}: {text}, {syscall} '{}'", p(path))
}

/// The errno class behind both formatters. `EPERM` is broken out from `EACCES`
/// because it is the one a companion mount actually hits on win32: creating a
/// directory symlink without SeCreateSymbolicLinkPrivilege fails with
/// ERROR_PRIVILEGE_NOT_HELD (1314), which libuv maps to EPERM, and Rust
/// surfaces it as an uncategorized error rather than PermissionDenied.
pub(crate) fn node_errno_class(err: &std::io::Error) -> (&'static str, &'static str) {
    use std::io::ErrorKind::*;
    // ERROR_PRIVILEGE_NOT_HELD — the win32 symlink case. Every other win32
    // code keeps the kind()-based mapping below (ERROR_ACCESS_DENIED is
    // EACCES in libuv too, which PermissionDenied already produces).
    if cfg!(windows) && err.raw_os_error() == Some(1314) {
        return ("EPERM", "operation not permitted");
    }
    match err.kind() {
        NotFound => ("ENOENT", "no such file or directory"),
        PermissionDenied => ("EACCES", "permission denied"),
        AlreadyExists => ("EEXIST", "file already exists"),
        IsADirectory => ("EISDIR", "illegal operation on a directory"),
        _ => ("EIO", "i/o error"),
    }
}

/// `fs.symlinkSync(target, path, 'dir')`'s uv error message shape —
/// `EPERM: operation not permitted, symlink '<target>' -> '<path>'`. Only
/// reachable AFTER `git worktree add` has succeeded, so it can never be
/// delegated.
///
/// The TARGET is reported the way Node reports it: `preprocessSymlinkDestination`
/// resolves it (win32 `path.resolve`) and rewrites `/` as `\` before the call,
/// so the error carries the RESOLVED spelling, not the raw config string.
/// Measured against a live Node oracle on a host without
/// SeCreateSymbolicLinkPrivilege: `EPERM: operation not permitted, symlink
/// 'E:\…\shared' -> 'E:\…\vendor\companion'`.
pub(crate) fn node_symlink_error_message(err: &std::io::Error, target: &str, path: &Path) -> String {
    let (code, text) = node_errno_class(err);
    format!(
        "{code}: {text}, symlink '{}' -> '{}'",
        js_path_resolve_from_cwd(target),
        p(path)
    )
}

/// `path.resolve(p)` — the win32 flavor on win32 (a rooted-but-driveless path
/// keeps the cwd's drive; every separator becomes `\`; `.`/`..` are folded
/// lexically), the identity on posix, where Node passes the target through
/// untouched.
pub(crate) fn js_path_resolve_from_cwd(raw: &str) -> String {
    if !cfg!(windows) {
        return raw.to_string();
    }
    let cwd = std::env::current_dir().unwrap_or_default();
    let cwd_s = cwd.to_string_lossy().into_owned();
    let b = raw.as_bytes();
    let has_drive = b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':';
    let rooted = !b.is_empty() && (b[0] == b'/' || b[0] == b'\\');
    let joined = if has_drive && b.len() > 2 && (b[2] == b'/' || b[2] == b'\\') {
        raw.to_string()
    } else if rooted {
        // Node keeps the cwd's DRIVE for a rooted-but-driveless path.
        let drive = if cwd_s.as_bytes().len() >= 2 && cwd_s.as_bytes()[1] == b':' {
            cwd_s[..2].to_string()
        } else {
            String::new()
        };
        format!("{drive}{raw}")
    } else {
        format!("{cwd_s}\\{raw}")
    };
    // Lexical normalization over the unified separator.
    let unified = joined.replace('/', "\\");
    let (prefix, rest) = if unified.as_bytes().len() >= 2 && unified.as_bytes()[1] == b':' {
        (unified[..2].to_string(), &unified[2..])
    } else {
        (String::new(), unified.as_str())
    };
    let absolute = rest.starts_with('\\');
    let mut parts: Vec<&str> = Vec::new();
    for seg in rest.split('\\') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    let mut out = prefix;
    if absolute {
        out.push('\\');
    }
    out.push_str(&parts.join("\\"));
    out
}

/// `path.join(base, rel)` with win32 separator normalization: a config value
/// spelled `vendor/companion` becomes `vendor\companion` there, which is what
/// Node's own `path.join` produces and therefore what every message that
/// interpolates the mount path must print.
pub(crate) fn js_path_join(base: &Path, rel: &str) -> PathBuf {
    if cfg!(windows) {
        base.join(rel.replace('/', "\\"))
    } else {
        base.join(rel)
    }
}

/// `fs.symlinkSync(target, path, 'dir')` — a DIRECTORY symlink on every
/// platform (never a junction: Node only falls back to a junction for
/// `type: 'junction'`, which this call site does not pass).
pub(crate) fn symlink_dir(target: &str, link: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(target, link)
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(target, link)
    }
}

/// `fs.unlinkSync(p)` for a path that may be a DIRECTORY symlink. libuv's
/// uv_fs_unlink detects a directory reparse point on win32 and calls
/// RemoveDirectory; `std::fs::remove_file` does not, so the second attempt is
/// what keeps a companion mount removable there. Best-effort at both call
/// sites (Node wraps each in its own try/catch), exactly like Node.
pub(crate) fn unlink_maybe_dir_symlink(path: &Path) {
    if std::fs::remove_file(path).is_err() {
        let _ = std::fs::remove_dir(path);
    }
}

/// `/^gitdir:\s*(.+)$/` applied to an ALREADY-TRIMMED pointer file: no `/m`,
/// and `.` never matches a line terminator, so a multi-line pointer simply
/// does not match. Shared by readWorktreeGitVerifiedId (the forward read) and
/// resolveWorktreeById (the reverse read) so the two can never drift.
pub(crate) fn parse_gitdir_pointer(trimmed: &str) -> Option<&str> {
    trimmed
        .strip_prefix("gitdir:")
        .map(str::trim_start)
        .filter(|rest| !rest.is_empty() && !rest.contains(['\n', '\r', '\u{2028}', '\u{2029}']))
}

/// readWorktreeGitVerifiedId — the AUTHORITATIVE id: git suffixes a colliding
/// basename, so the id is always re-read from the new worktree's own `.git`
/// pointer rather than assumed to be the directory name.
pub(crate) fn read_worktree_git_verified_id(worktree_root: &Path) -> Result<String, String> {
    let git_file = worktree_root.join(".git");
    let raw = std::fs::read_to_string(&git_file)
        .map_err(|e| node_fs_error_message(&e, "open", &git_file))?;
    let raw = js_trim(&raw);
    let captured = parse_gitdir_pointer(raw);
    let Some(captured) = captured else {
        return Err(format!(
            "worktree .git file at {} is not a valid \"gitdir: ...\" pointer",
            p(&git_file)
        ));
    };
    let normalized = js_trim(captured).replace('\\', &MAIN_SEPARATOR.to_string());
    let gitdir = js_path_resolve(worktree_root, &normalized);
    Ok(gitdir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default())
}

// ─── worktree-store.mjs syncWorktreeSkills ────────────────────────────────

/// `[path.join('.claude','skills'), path.join('.agents','skills')]` — the
/// separator is the platform's, and it shows up verbatim in the
/// "no bee-* skill directories found under …" reason.
pub(crate) fn skill_sync_roots() -> [String; 2] {
    [
        format!(".claude{MAIN_SEPARATOR}skills"),
        format!(".agents{MAIN_SEPARATOR}skills"),
    ]
}

/// `fs.cpSync(src, dest, {recursive: true})` for the plain dir/file trees a
/// `bee-*` skill directory is. Anything else (symlink, device) is an error,
/// which lands in `skipped[].reason` exactly as a cpSync throw would.
pub(crate) fn cp_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    let meta = std::fs::symlink_metadata(src)?;
    if meta.is_dir() {
        std::fs::create_dir_all(dest)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            cp_recursive(&entry.path(), &dest.join(entry.file_name()))?;
        }
        Ok(())
    } else if meta.is_file() {
        std::fs::copy(src, dest).map(|_| ())
    } else {
        Err(std::io::Error::other("unsupported file type"))
    }
}

/// syncWorktreeSkills — best-effort, NEVER throws, never rolls back a
/// worktree. Its three return shapes are Node's exactly.
pub(crate) fn sync_worktree_skills(main_root: &Path, worktree_root: &Path) -> Value {
    let roots = skill_sync_roots();
    let mut synced: Vec<Value> = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();

    for rel in &roots {
        let src_root = main_root.join(rel);
        let Ok(entries) = std::fs::read_dir(&src_root) else {
            continue; // main checkout has no such root
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if !is_dir || !name.starts_with("bee-") {
                continue;
            }
            let rel_path = format!("{rel}{MAIN_SEPARATOR}{name}");
            let dest = worktree_root.join(&rel_path);
            if dest.exists() {
                skipped.push(json!({ "path": rel_path, "reason": "already exists in worktree" }));
                continue;
            }
            let attempt = dest
                .parent()
                .map(std::fs::create_dir_all)
                .unwrap_or(Ok(()))
                .and_then(|()| cp_recursive(&src_root.join(&name), &dest));
            match attempt {
                Ok(()) => synced.push(json!(rel_path)),
                Err(e) => skipped.push(json!({
                    "path": rel_path,
                    "reason": node_fs_error_message(&e, "cp", &dest),
                })),
            }
        }
    }

    if synced.is_empty() && skipped.is_empty() {
        return json!({
            "attempted": false,
            "applied": false,
            "reason": format!(
                "no bee-* skill directories found under {} in the main checkout.",
                roots.join(" or ")
            ),
        });
    }
    if synced.is_empty() {
        return json!({
            "attempted": true,
            "applied": false,
            "reason": "already present in worktree",
            "synced": synced,
            "skipped": skipped,
        });
    }
    let reason = if skipped.is_empty() {
        format!("synced {} bee-* skill dir(s)", synced.len())
    } else {
        format!(
            "synced {} bee-* skill dir(s), {} skipped (see .skipped)",
            synced.len(),
            skipped.len()
        )
    };
    json!({
        "attempted": true,
        "applied": true,
        "reason": reason,
        "synced": synced,
        "skipped": skipped,
    })
}
