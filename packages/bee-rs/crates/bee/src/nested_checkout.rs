// nested_checkout — wcg-3's DIRECTORY-SCAN half of the shared-nested-checkout
// guard: guards.mjs `hasAnySharedNestedCheckout` + `scanForNestedCheckout`,
// the second of the two D1 enforcement surfaces (the first, the point-check
// `isSharedNestedCheckoutTarget`, is already native inside
// hooks/write_guard.rs).
//
// WHY THIS IS A MODULE AND NOT A COPY. The point-check walks UP from a
// concrete write target; `bee worktree new` has no target, so it must walk
// DOWN from root. guards.mjs's own comment says the two surfaces share the
// companion-marker verification and the submodule-registration exclusion
// "never a second copy of either". This module holds ONLY the down-walk; every
// predicate it consults is imported from hooks/write_guard.rs, which is why
// those five items were widened to pub(crate) there. Re-deriving them here
// would fork the guard — the drift contract C5 exists to prevent exactly that.
//
// ONE DELIBERATE DIVERGENCE (cutover class: contract C2 is retired when Node
// is deleted, so a Node-worded byte can no longer be matched). guards.mjs's F2
// error posture turns any non-ENOENT filesystem error into a JS THROW, which
// bee.mjs's handleWorktreeNew catches and interpolates into its fail-closed
// refusal:
//
//     ... the detection check itself errored (${detectionError.message}) ...
//
// where `.message` is a libuv/V8 string. The write-guard primitives collapse
// every such error to a message-less `Nd` (it only ever needed "delegate"), so
// this port supplies its OWN deterministic reason naming the operation that
// failed, in the same place, with the rest of the sentence byte-identical. The
// REFUSAL ITSELF — fail closed, zero mutation, same wording, same exit — is
// unchanged; only the parenthetical differs. Same approximation class as
// verbs/worktree.rs's `node_fs_error_message`.
//
// The D6 no-op contract is intact: a solo checkout (nobody else concurrently
// live) never scans the filesystem at all and always answers `false`.

use crate::hooks::write_guard as guard;
use std::path::Path;

/// A detection failure. `reason` is what the caller interpolates into the
/// fail-closed refusal in place of Node's V8 `.message` (see the header).
#[derive(Debug)]
pub(crate) struct DetectErr {
    pub(crate) reason: String,
}

impl DetectErr {
    fn new(reason: impl Into<String>) -> Self {
        DetectErr { reason: reason.into() }
    }
}

type D<T> = Result<T, DetectErr>;

/// guards.mjs NESTED_SCAN_SKIP_DIRS — the scout-excluded build/dep dirs plus
/// root's own `.git`. A nested repo under node_modules/ is a dependency's own
/// repo, never a companion-eligible shared checkout.
const NESTED_SCAN_SKIP_DIRS: &[&str] = &[
    "node_modules",
    "dist",
    "build",
    "vendor",
    "coverage",
    ".next",
    "__pycache__",
    ".git",
];

/// guards.mjs NESTED_SCAN_MAX_DEPTH.
const NESTED_SCAN_MAX_DEPTH: usize = 8;

fn s(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// guards.mjs hasAnySharedNestedCheckout (wcg-3, Port-D4 controlRoot).
///
/// `control_root` scopes the concurrency check to the coordination root (which
/// differs from the physical `root` when linked worktrees share one control
/// plane); the filesystem walk stays root-scoped, always.
///
/// `exclude_session_id` is the ACTING session — its own live heartbeat is
/// never "another" session (bee.mjs Port-D6).
pub(crate) fn has_any_shared_nested_checkout(
    root: &Path,
    control_root: &str,
    exclude_session_id: Option<&str>,
) -> D<bool> {
    // D6: additive — fires only when a second session is concurrently live.
    // reservations.rs's port is strict-equivalent by construction (an
    // unreadable session record is an error there, never a silent "solo"),
    // which is the fail-closed posture guards.mjs's `strict: true` was added
    // for; see that function's own doc comment.
    let concurrent =
        crate::verbs::reservations::is_concurrent_mode_excluding(control_root, exclude_session_id)
            .map_err(|_| {
                DetectErr::new(format!(
                    "a session record under {control_root} could not be read or parsed"
                ))
            })?;
    if !concurrent {
        return Ok(false);
    }

    let root_s = s(root);
    let root_real = match guard::realpath_f2(&root_s).map_err(|_| realpath_err(&root_s))? {
        Some(r) => r,
        None => return Ok(false),
    };

    // Shape (a): a marker-verified companion mount present in this checkout.
    // THE guard's own verification, not a second copy of it.
    let mount = guard::resolve_verified_companion_mount_real(&root_s).map_err(|_| {
        DetectErr::new(format!(
            "the companion marker at {root_s}/.bee/companion-session.json could not be read or parsed"
        ))
    })?;
    if mount.is_some() {
        return Ok(true);
    }

    // Shape (b): any plain nested `.git` strictly under root, excluding a
    // registration-verified submodule — the STR65 incident shape.
    scan_for_nested_checkout(&root_real, &root_real, 0)
}

fn realpath_err(path: &str) -> DetectErr {
    DetectErr::new(format!("{path} could not be resolved to a real path"))
}

/// guards.mjs scanForNestedCheckout — a bounded, symlink-free DFS for the
/// FIRST companion-eligible nested checkout strictly under `root_real`.
///
/// `entry.isDirectory()` on a Node Dirent is lstat-derived, so it skips
/// regular files AND symlinks; `std::fs::DirEntry::file_type` is lstat-derived
/// too, so the symlink exclusion is preserved (D2 shape (b) is a PHYSICAL
/// nested repo; the symlink/companion shape is shape (a) above).
fn scan_for_nested_checkout(root_real: &str, dir: &str, depth: usize) -> D<bool> {
    if depth > NESTED_SCAN_MAX_DEPTH {
        return Ok(false);
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        // F2: a dir that vanished mid-scan (ENOENT, benign race) just prunes
        // this branch; EACCES/EIO/EMFILE mean the scan cannot honestly claim
        // "nothing found here" and must fail the whole detection.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => {
            return Err(DetectErr::new(format!(
                "{} while reading the directory {dir}",
                errno_phrase(&e)
            )))
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(DetectErr::new(format!(
                    "{} while reading the directory {dir}",
                    errno_phrase(&e)
                )))
            }
        };
        let is_dir = match entry.file_type() {
            Ok(t) => t.is_dir(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(DetectErr::new(format!(
                    "{} while inspecting {}",
                    errno_phrase(&e),
                    s(&entry.path())
                )))
            }
        };
        if !is_dir {
            continue; // skips regular files AND symlinks
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if NESTED_SCAN_SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        let child = s(&entry.path());
        if guard::has_git_node_f2(&child).map_err(|_| {
            DetectErr::new(format!("the .git node under {child} could not be stat'd"))
        })? {
            let child_real = guard::realpath_f2(&child).map_err(|_| realpath_err(&child))?;
            // `&&` short-circuits in the .mjs, so the submodule check (which
            // can itself fail detection) is only ever reached for a child that
            // already passed the identity/containment tests.
            if let Some(child_real) = child_real {
                if child_real != root_real
                    && guard::is_under_root(root_real, &child_real)
                        .map_err(|_| realpath_err(&child_real))?
                    && !guard::is_registered_submodule(root_real, &child_real).map_err(|_| {
                        DetectErr::new(format!(
                            "{root_real}/.gitmodules could not be read while checking whether {child_real} is a registered submodule"
                        ))
                    })?
                {
                    return Ok(true);
                }
            }
            continue; // a `.git`-bearing dir — never descend into a nested repo
        }
        if scan_for_nested_checkout(root_real, &child, depth + 1)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The errno-class phrase this module substitutes for libuv's own wording (see
/// the header's documented divergence). Deterministic per errno class, never
/// per-platform text.
fn errno_phrase(e: &std::io::Error) -> &'static str {
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => "permission was denied",
        _ => "a filesystem error occurred",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A root with `.bee/` and a sessions dir — the only state the concurrency
    /// half reads.
    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("main");
        std::fs::create_dir_all(root.join(".bee").join("sessions")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        (tmp, root)
    }

    fn write_live_session(root: &std::path::Path, id: &str) {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();
        std::fs::write(
            root.join(".bee").join("sessions").join(format!("{id}.json")),
            serde_json::to_string(&serde_json::json!({"id": id, "last_heartbeat": now})).unwrap(),
        )
        .unwrap();
    }

    fn nested_repo(root: &std::path::Path, rel: &str) {
        let dir = root.join(rel);
        std::fs::create_dir_all(dir.join(".git")).unwrap();
    }

    /// D6: a solo checkout NEVER scans — even with a nested repo sitting right
    /// there, the answer is false and no filesystem walk happens.
    #[test]
    fn solo_checkout_is_a_pure_no_op() {
        let (_tmp, root) = fixture();
        nested_repo(&root, "shared");
        let got = has_any_shared_nested_checkout(&root, &root.to_string_lossy(), None).unwrap();
        assert!(!got, "nobody else is live — the guard must not fire");
    }

    /// Shape (b): a plain nested `.git` under root, with a second live session.
    #[test]
    fn nested_checkout_with_a_second_live_session_is_flagged() {
        let (_tmp, root) = fixture();
        write_live_session(&root, "other-session");
        assert!(
            !has_any_shared_nested_checkout(&root, &root.to_string_lossy(), None).unwrap(),
            "no nested checkout yet"
        );
        nested_repo(&root, "shared");
        assert!(has_any_shared_nested_checkout(&root, &root.to_string_lossy(), None).unwrap());
    }

    /// The ACTING session's own heartbeat is never "another" session
    /// (Port-D6): excluding it collapses the whole guard back to the D6 no-op.
    #[test]
    fn the_acting_session_is_excluded() {
        let (_tmp, root) = fixture();
        write_live_session(&root, "me");
        nested_repo(&root, "shared");
        let ctrl = root.to_string_lossy().into_owned();
        assert!(has_any_shared_nested_checkout(&root, &ctrl, None).unwrap());
        assert!(!has_any_shared_nested_checkout(&root, &ctrl, Some("me")).unwrap());
    }

    /// A `.gitmodules`-REGISTERED submodule is excluded (spike case C), and a
    /// registration naming some OTHER path does not excuse this one.
    #[test]
    fn a_registered_submodule_is_excluded() {
        let (_tmp, root) = fixture();
        write_live_session(&root, "other-session");
        nested_repo(&root, "libs/sub");
        let ctrl = root.to_string_lossy().into_owned();
        assert!(has_any_shared_nested_checkout(&root, &ctrl, None).unwrap());

        std::fs::write(
            root.join(".gitmodules"),
            "[submodule \"sub\"]\n\tpath = libs/sub\n",
        )
        .unwrap();
        assert!(!has_any_shared_nested_checkout(&root, &ctrl, None).unwrap());

        std::fs::write(
            root.join(".gitmodules"),
            "[submodule \"other\"]\n\tpath = libs/elsewhere\n",
        )
        .unwrap();
        assert!(has_any_shared_nested_checkout(&root, &ctrl, None).unwrap());
    }

    /// The scan prunes the scout-excluded dirs (a dependency's own repo is
    /// never a shared checkout) and stops at NESTED_SCAN_MAX_DEPTH.
    #[test]
    fn skip_dirs_and_the_depth_bound_are_honored() {
        let (_tmp, root) = fixture();
        write_live_session(&root, "other-session");
        let ctrl = root.to_string_lossy().into_owned();

        nested_repo(&root, "node_modules/pkg");
        nested_repo(&root, "dist/thing");
        assert!(
            !has_any_shared_nested_checkout(&root, &ctrl, None).unwrap(),
            "excluded dirs are never descended"
        );

        // depth 0 is root's own children, so a repo at depth 10 is past the
        // bound (guards.mjs: `if (depth > NESTED_SCAN_MAX_DEPTH) return false`).
        nested_repo(&root, "a/b/c/d/e/f/g/h/i/j/deep");
        assert!(!has_any_shared_nested_checkout(&root, &ctrl, None).unwrap());
        nested_repo(&root, "a/b/near");
        assert!(has_any_shared_nested_checkout(&root, &ctrl, None).unwrap());
    }

    /// Shape (a): a marker-verified companion mount fires the guard through
    /// THE write guard's own verification (never a second copy).
    #[test]
    fn a_verified_companion_mount_is_flagged() {
        const CAP: &str = "symlink creation denied — needs SeCreateSymbolicLinkPrivilege \
(Developer Mode or an elevated shell)";
        if !symlink_capable() {
            eprintln!(
                "SKIP (env-limited: {CAP}) — a verified companion mount is flagged by the wcg-3 scan"
            );
            return;
        }
        let (tmp, root) = fixture();
        write_live_session(&root, "other-session");
        let ctrl = root.to_string_lossy().into_owned();

        let companion = tmp.path().join("companion-checkout");
        std::fs::create_dir_all(&companion).unwrap();
        let mount = root.join("vendor").join("companion");
        std::fs::create_dir_all(mount.parent().unwrap()).unwrap();
        symlink_dir(&companion, &mount).unwrap();
        std::fs::write(
            root.join(".bee").join("companion-session.json"),
            serde_json::to_string(&serde_json::json!({
                "sessionId": "s1",
                "worktreePath": companion.to_string_lossy(),
                "mountPath": "vendor/companion",
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(has_any_shared_nested_checkout(&root, &ctrl, None).unwrap());

        // A marker whose declared worktreePath no longer matches the live
        // symlink is NOT verified — never grant on where the link points today.
        std::fs::write(
            root.join(".bee").join("companion-session.json"),
            serde_json::to_string(&serde_json::json!({
                "sessionId": "s1",
                "worktreePath": tmp.path().join("somewhere-else").to_string_lossy(),
                "mountPath": "vendor/companion",
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(!has_any_shared_nested_checkout(&root, &ctrl, None).unwrap());
    }

    fn symlink_capable() -> bool {
        use std::sync::OnceLock;
        static CAP: OnceLock<bool> = OnceLock::new();
        *CAP.get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("t");
            std::fs::create_dir(&target).unwrap();
            symlink_dir(&target, &dir.path().join("l")).is_ok()
        })
    }

    fn symlink_dir(target: &std::path::Path, link: &std::path::Path) -> std::io::Result<()> {
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(target, link)
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link)
        }
    }
}
