// Test-only shared helpers.
//
// WHY THIS EXISTS. Three test modules (roots, verbs/reservations,
// verbs/status_full) build their fixtures by shelling out to a real
// `git init` + `git worktree add`, then assert on how bee resolves the
// resulting checkout. Twenty-one tests do this, each in its own tempdir, so
// they look independent — and on a developer machine they are: the whole
// cluster passes in parallel, locally and in a clean clone.
//
// On a two-core GitHub windows-latest runner it did not. The 2.0.1 push
// turned the Windows lane from `-- --test-threads=1` (which every green
// nightly had run under) to the declared command's default parallelism, and
// seventeen of these went red together — every one of them reporting that a
// worktree `git` had just created did not resolve as linked-valid, while the
// same suite took 78s instead of 18s. Nothing else in that lane changed, and
// no other theory survives the evidence: an 8.3-short-path or a canonicalisation
// bug would have failed the serial nightly too.
//
// So the fixtures are serialized against each other and nothing else. The
// other ~936 tests keep running in parallel — pinning the whole suite to one
// thread to accommodate this cluster would trade an 8-second dev loop for a
// 70-second one, and would hide the next defect of this shape rather than
// name it.
//
// This is a containment, not a diagnosis: what makes concurrent `git worktree
// add` unreliable on that runner is still unknown. It is recorded here rather
// than in a commit message so the next reader finds it at the lock.

use std::sync::{Mutex, MutexGuard, OnceLock};

/// Held for a whole test body, not just the fixture call: the resolution the
/// test then performs reads the very files `git` wrote.
pub(crate) type GitGuard = MutexGuard<'static, ()>;

/// Serializes every fixture that shells out to git.
///
/// Poison-tolerant on purpose. A panicking test — which is exactly what a
/// failing assertion is — must not convert its siblings' failures into
/// `PoisonError`, because that replaces the real assertion message with an
/// unrelated one and makes a red suite unreadable.
pub(crate) fn git_fixture_lock() -> GitGuard {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
