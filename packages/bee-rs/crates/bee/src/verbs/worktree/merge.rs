// mergeFeatureWorktree
//
// Split out of the single 4.2k-line verbs/worktree.rs. D7/D8
// (docs/history/test-doctrine/CONTEXT.md, td-3): the "verify child" this
// file used to spawn (`commands.test`, unlocked, against the
// merged-but-uncommitted tree) is retired — `bee worktree merge` no longer
// spawns a test command at all; `merge_stage` (phases.rs) reads the D8
// proof lines instead, as a zero-mutation precondition BEFORE `git merge`
// ever runs.
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

// ─── worktree-store.mjs mergeFeatureWorktree ──────────────────────────────
//
// The staged transaction, whole (see the module header for the two
// delegation gates that keep every V8-worded arm out of reach):
//   P1  mergeFeatureWorktreeStage   — LOCKED ('worktree-admin' on mainRoot);
//                                     also where the D7/D8 proof-check
//                                     precondition now runs (td-3)
//   P3  mergeFeatureWorktreeFinish  — RE-LOCKED
// Node acquired 'worktree-admin' TWICE on every non-terminal merge (P1 then
// P3) even when its old "P2" verify child never ran, so this port still
// does — a single hold would drop one `result: "acquired"` row from
// .bee/logs/contention.jsonl.

/// `WorktreeMergeError` — `[CODE] message`, the only observable byte.
pub(crate) fn refuse_merge(code: &str, message: String) -> MErr {
    MErr::Thrown(format!("[{code}] {message}"))
}

/// The merge's failure channel. `Thrown` is a message bee.mjs's dispatcher
/// surfaces through emitError AND processAsOwner persists into the queue
/// record's `error` field — every arm that produces one is deterministic.
pub(crate) enum MErr {
    Thrown(String),
    /// Only ever returned BEFORE `git merge --no-ff --no-commit` runs.
    Ex,
}

pub(crate) type MR<T> = Result<T, MErr>;

/// The merge's own `{ok, ...}` answer: the result object plus the exit code
/// bee.mjs derives from it and the queue status processAsOwner writes.
pub(crate) struct MergeAnswer {
    pub(crate) result: Map<String, Value>,
    pub(crate) ok: bool,
}

/// gitStatusPorcelain — deliberately WITHOUT `--ignored` (decision D8a). A
/// git failure is a plain (untyped) Error whose bytes are still fully
/// deterministic, including the literal "exit null" a never-launched spawn
/// renders.
pub(crate) fn git_status_porcelain(cwd: &Path) -> Result<String, String> {
    let r = run_git(cwd, &["status", "--porcelain"]);
    if r.status != Some(0) {
        return Err(format!(
            "\"git status --porcelain\" failed in {}: {}",
            p(cwd),
            r.fail_text()
        ));
    }
    Ok(r.stdout.unwrap_or_default())
}

pub(crate) fn is_tree_dirty(cwd: &Path) -> Result<bool, String> {
    Ok(!js_trim(&git_status_porcelain(cwd)?).is_empty())
}

/// gitStatusPorcelainExcluding — `git status --porcelain -- :(exclude)<p> …`.
///
/// Deliberately NOT post-hoc text filtering: porcelain COLLAPSES an untracked
/// directory (or a symlink-to-directory, which is exactly what a companion
/// mount is) to one summary line for its top-level name, so a mount at
/// `vendor/companion` shows only as `?? vendor/` and a text filter for the
/// mount path would never match — the merge would refuse forever. Asking git
/// itself never to report those paths removes them at the source, at any
/// depth and under any quoting. Multiple `:(exclude)` pathspecs with no
/// positive pathspec among them still mean "everything else in the tree"
/// (git's own pathspec-magic contract), so excluding two is not a narrowing.
///
/// Pathspecs are `/`-only even on Windows, hence the `\` → `/` rewrite.
pub(crate) fn git_status_porcelain_excluding(cwd: &Path, exclude_paths: &[String]) -> Result<String, String> {
    let pathspecs: Vec<String> = exclude_paths
        .iter()
        .map(|p| format!(":(exclude){}", p.replace('\\', "/")))
        .collect();
    let mut args: Vec<&str> = vec!["status", "--porcelain", "--"];
    args.extend(pathspecs.iter().map(String::as_str));
    let r = run_git(cwd, &args);
    if r.status != Some(0) {
        return Err(format!(
            "\"git status --porcelain -- {}\" failed in {}: {}",
            pathspecs.join(" "),
            p(cwd),
            r.fail_text()
        ));
    }
    Ok(r.stdout.unwrap_or_default())
}

pub(crate) fn is_tree_dirty_excluding(cwd: &Path, exclude_paths: &[String]) -> Result<bool, String> {
    Ok(!js_trim(&git_status_porcelain_excluding(cwd, exclude_paths)?).is_empty())
}

/// The same `:(exclude)` pathspec form [`git_status_porcelain_excluding`]
/// uses, but with `--untracked-files=all` so an untracked directory is
/// listed FILE BY FILE rather than collapsed to one top-level summary line.
/// Plain porcelain's collapsing is fine for the boolean dirty/clean check
/// (`is_tree_dirty_excluding`) — a collapsed line is still non-empty — but
/// it is WRONG for a refusal message that has to NAME the offending path:
/// with `docs/history/<mine>` excluded and `docs/history/<theirs>` left
/// over, plain porcelain still renders the whole thing as `?? docs/`,
/// naming neither feature. `--untracked-files=all` makes the exclude
/// pathspec's own recursion into `docs/` surface the real leftover file
/// (`?? docs/history/<theirs>/plan.md`) instead.
pub(crate) fn git_status_porcelain_excluding_untracked_all(cwd: &Path, exclude_paths: &[String]) -> Result<String, String> {
    let pathspecs: Vec<String> = exclude_paths
        .iter()
        .map(|p| format!(":(exclude){}", p.replace('\\', "/")))
        .collect();
    let mut args: Vec<&str> = vec!["status", "--porcelain", "--untracked-files=all", "--"];
    args.extend(pathspecs.iter().map(String::as_str));
    let r = run_git(cwd, &args);
    if r.status != Some(0) {
        return Err(format!(
            "\"git status --porcelain --untracked-files=all -- {}\" failed in {}: {}",
            pathspecs.join(" "),
            p(cwd),
            r.fail_text()
        ));
    }
    Ok(r.stdout.unwrap_or_default())
}

// ─── trun-4: pre-merge `.bee` (+ `docs/decisions`, `docs/knowledge`, and
//     `docs/history/<feature>`) bookkeeping auto-commit ──────────────────
//
// Closes a real deadlock: `bee worktree merge` refuses on ANY dirty path in
// main (WORKTREE_MERGE_MAIN_DIRTY, phases.rs), but the dirt is routinely
// bee's OWN bookkeeping — cell traces, .bee/decisions.jsonl,
// .bee/backlog.jsonl, docs/decisions/taxonomy.json (every `decisions log`),
// docs/knowledge/** (every capture sync), and (at close) a promote proposal
// or other artifact under the MERGING feature's own docs/history/<feature>/
// — written by the orchestrator's normal state calls during the slice.
// Committing exactly that dirt BY HAND from main is refused by the
// worktree-first guard, so a green slice could never land. This mirrors
// `bee close`'s own bee-store bookkeeping auto-commit (drivers/close.rs's
// `commit_close_bookkeeping`) — same warn-never-block contract, same
// `commit_unsigned` mechanism (git.rs, B-P2-1) — widened to the roots above
// and kept as its own implementation (not a shared call into close's
// helper) because close's version is hard-scoped to a single `.bee`
// pathspec and a different config key/message; forcing the two together
// would either weaken close's own validated config path or teach it a
// multi-pathspec, multi-root shape it has no other reason to carry. What
// DOES stay the ONE shared mechanism is the underlying `git commit
// --no-gpg-sign`, through `commit_unsigned`.

/// The pathspecs a pre-merge bookkeeping auto-commit is allowed to sweep:
/// `.bee`, `docs/decisions`, and `docs/knowledge` always; `docs/history/<feature>`
/// only when the worktree's feature is known (`resolve_worktree_feature` —
/// absent for a worktree registered without bee's own creation identity, in
/// which case the other three still apply). `docs/decisions` and
/// `docs/knowledge` join `.bee` unconditionally because bee itself writes
/// all three on every session — `decisions log` writes
/// `docs/decisions/taxonomy.json` (verbs/decisions/mod.rs), and the capture
/// chain writes `docs/knowledge/**` — exactly the bookkeeping dirt this
/// auto-commit exists to sweep, never a peer's work. Never widened to all of
/// `docs/history/`: a sibling feature's in-flight worktree can be writing
/// its own `docs/history/<other>/` at the same moment, and sweeping it into
/// an UNRELATED merge's bookkeeping commit would land a peer's uncommitted
/// work without their say-so.
pub(crate) fn main_bookkeeping_roots(feature: Option<&str>) -> Vec<String> {
    let mut roots = vec![".bee".to_string(), "docs/decisions".to_string(), "docs/knowledge".to_string()];
    if let Some(feature) = feature {
        roots.push(format!("docs/history/{feature}"));
    }
    roots
}

/// `worktree_merge_commit_bookkeeping` in the merged config (`.bee/config.json`
/// overlaid by `.bee/config.local.json`, state.rs's `read_config_raw`) —
/// absent or any non-`false` value reads as ON, the same absent-means-on
/// default `archive_on_close_enabled` (close.rs) uses for its own opt-out.
/// Only an explicit `false` turns the auto-commit off; at that point a
/// dirty main refuses exactly as it did before this cell
/// (WORKTREE_MERGE_MAIN_DIRTY, unconditionally, the original wording) — the
/// opt-out means "stop auto-committing on my behalf," not "auto-commit a
/// smaller scope."
pub(crate) fn worktree_merge_commit_bookkeeping_enabled(main_root: &Path) -> bool {
    !matches!(
        crate::state::read_config_raw(main_root).get("worktree_merge_commit_bookkeeping"),
        Some(Value::Bool(false))
    )
}

/// What the pre-merge bookkeeping auto-commit did, and why not when it
/// didn't — the same `committed` / `reason` / `index_restored` shape
/// close's own `BookkeepingCommit` (drivers/close.rs) renders, so a caller
/// reading either JSON never has to learn a second shape for the same idea.
/// `reason` is one of: `clean`, `not_a_repo`, or `git_failed:<first line>`.
/// `index_restored` is only ever `Some` on the one `git_failed` reason that
/// can leave the roots staged after `git add` already ran — `git commit`
/// itself failing — every other `Skipped` arm never staged anything.
pub(crate) enum MainBookkeepingCommit {
    Committed { sha: String },
    Skipped { reason: String, index_restored: Option<bool> },
}

impl MainBookkeepingCommit {
    fn skipped(reason: impl Into<String>) -> Self {
        MainBookkeepingCommit::Skipped { reason: reason.into(), index_restored: None }
    }

    pub(crate) fn value(&self) -> Value {
        match self {
            MainBookkeepingCommit::Committed { sha } => json!({"committed": true, "sha": sha}),
            MainBookkeepingCommit::Skipped { reason, index_restored: None } => {
                json!({"committed": false, "reason": reason})
            }
            MainBookkeepingCommit::Skipped { reason, index_restored: Some(restored) } => {
                json!({"committed": false, "reason": reason, "index_restored": restored})
            }
        }
    }
}

/// The `(stderr || stdout || '').trim() || `exit status <code>`` (or
/// `killed by signal`) fallback chain, applied to a [`GitOut`] the way
/// close.rs's own `git_fail_first_line` applies it to its local `GitRun` —
/// a silent failure (a pre-commit hook that exits non-zero without a word
/// on either stream) must never render the bare `git_failed:` prefix with
/// nothing after it.
fn git_fail_first_line(out: &GitOut) -> String {
    let src = out
        .stderr
        .as_deref()
        .filter(|s| !js_trim(s).is_empty())
        .or_else(|| out.stdout.as_deref().filter(|s| !js_trim(s).is_empty()))
        .unwrap_or("");
    let first_line = js_trim(src).lines().next().unwrap_or("").trim();
    if !first_line.is_empty() {
        return first_line.to_string();
    }
    match out.status {
        Some(code) => format!("exit status {code}"),
        None => "killed by signal".to_string(),
    }
}

/// Auto-commits whatever dirt sits under `pathspecs` in `main_root` —
/// path-scoped throughout (`git status` / `git add` both take the SAME
/// `-- <pathspecs>` tail, so unrelated dirt and unrelated staged files are
/// never swept) — using the ONE shared unsigned-commit mechanism
/// (`commit_unsigned`, git.rs, B-P2-1) for the actual `git commit`.
/// Warn-never-block: every step here is best-effort; the caller
/// (phases.rs's `merge_stage`) never turns a `Skipped` outcome into a
/// refusal — a dirty main that could not be tidied is not a reason to keep
/// refusing forever.
pub(crate) fn commit_main_bookkeeping(main_root: &Path, message: &str, pathspecs: &[String]) -> MainBookkeepingCommit {
    let probe = run_git(main_root, &["rev-parse", "--is-inside-work-tree"]);
    match (probe.status, probe.stdout.as_deref().map(js_trim)) {
        (Some(0), Some("true")) => {}
        (Some(_), _) => return MainBookkeepingCommit::skipped("not_a_repo"),
        (None, _) => return MainBookkeepingCommit::skipped("git_failed:git rev-parse could not be spawned"),
    }

    // `git add -A -- <pathspec>` (unlike `git status`) FAILS OUTRIGHT with
    // "pathspec '<p>' did not match any files" when a pathspec matches
    // nothing at all — the ordinary case for `docs/history/<feature>` on a
    // worktree whose feature never wrote anything there. Drop any root that
    // matches neither a real path on disk nor a tracked one (`git ls-files`,
    // so a root whose only content was just DELETED still counts) before it
    // ever reaches `add`/`commit` — `.bee` alone is never dropped this way in
    // practice (bootstrap always creates it), so this only ever narrows the
    // optional docs/history root.
    let specs: Vec<&str> = pathspecs
        .iter()
        .map(String::as_str)
        .filter(|spec| {
            main_root.join(spec).exists() || {
                let tracked = run_git(main_root, &["ls-files", "--", spec]);
                !js_trim(&tracked.stdout.unwrap_or_default()).is_empty()
            }
        })
        .collect();
    if specs.is_empty() {
        return MainBookkeepingCommit::skipped("clean");
    }

    let mut status_args: Vec<&str> = vec!["status", "--porcelain", "--"];
    status_args.extend_from_slice(&specs);
    let status = run_git(main_root, &status_args);
    match status.status {
        Some(0) => {}
        Some(_) => return MainBookkeepingCommit::skipped(format!("git_failed:{}", git_fail_first_line(&status))),
        None => return MainBookkeepingCommit::skipped("git_failed:git status could not be spawned"),
    }
    if js_trim(&status.stdout.clone().unwrap_or_default()).is_empty() {
        return MainBookkeepingCommit::skipped("clean");
    }

    let mut add_args: Vec<&str> = vec!["add", "-A", "--"];
    add_args.extend_from_slice(&specs);
    let add = run_git(main_root, &add_args);
    match add.status {
        Some(0) => {}
        Some(_) => return MainBookkeepingCommit::skipped(format!("git_failed:{}", git_fail_first_line(&add))),
        None => return MainBookkeepingCommit::skipped("git_failed:git add could not be spawned"),
    }

    let commit_out = commit_unsigned(main_root, message, &specs);
    if commit_out.stdout.is_none() {
        let mut reset_args: Vec<&str> = vec!["reset", "--"];
        reset_args.extend_from_slice(&specs);
        let index_restored = run_git(main_root, &reset_args).status == Some(0);
        return MainBookkeepingCommit::Skipped {
            reason: "git_failed:git commit could not be spawned".to_string(),
            index_restored: Some(index_restored),
        };
    }
    if commit_out.status != Some(0) {
        let mut reset_args: Vec<&str> = vec!["reset", "--"];
        reset_args.extend_from_slice(&specs);
        let index_restored = run_git(main_root, &reset_args).status == Some(0);
        return MainBookkeepingCommit::Skipped {
            reason: format!("git_failed:{}", git_fail_first_line(&commit_out)),
            index_restored: Some(index_restored),
        };
    }

    let sha_out = run_git(main_root, &["rev-parse", "HEAD"]);
    let sha = if sha_out.status == Some(0) {
        js_trim(&sha_out.stdout.unwrap_or_default()).to_string()
    } else {
        String::new()
    };
    MainBookkeepingCommit::Committed { sha }
}

/// The three-part "main was left byte-untouched" proof (decision D2-REVISED)
/// required after EVERY `git merge --abort` this module runs. `Ok(())` is
/// `{ok:true}`; `Err(reason)` is the `{ok:false, reason}` the caller folds
/// into a SPECIFIC typed refusal.
///
/// Both `runGit(...).stdout.trim()` sites here are un-null-guarded in Node (a
/// TypeError whose V8 text would be persisted into the queue record). They are
/// unreachable by construction: `mergeFeatureWorktreeStage`'s very first git
/// call is `isTreeDirty(mainRoot)`, which throws the deterministic
/// `"git status --porcelain" failed in <root>: exit null` message — with ZERO
/// mutations — before any merge is staged. So by the time this function can
/// run, git has provably launched at least once from this same cwd. The
/// residual (git becoming unlaunchable mid-merge) is the same race class
/// verbs/workspace_store.rs documents for a record going unreadable between
/// its probe and its in-lock read.
pub(crate) fn main_untouched_proof(main_root: &Path, pre_merge_head: &str, merge_head_file: &Path) -> Result<(), String> {
    let head_now = js_trim(
        &run_git(main_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default(),
    )
    .to_string();
    if head_now != pre_merge_head {
        return Err(format!("HEAD moved from {pre_merge_head} to {head_now}"));
    }
    if merge_head_file.exists() {
        return Err(".git/MERGE_HEAD is still present".to_string());
    }
    let status = run_git(main_root, &["status", "--porcelain", "--untracked-files=no"])
        .stdout
        .unwrap_or_default();
    if !js_trim(&status).is_empty() {
        return Err(format!(
            "\"git status --porcelain --untracked-files=no\" is not clean:\n{status}"
        ));
    }
    Ok(())
}

/// currentBranch — `null` on detached HEAD (or no HEAD ref at all).
pub(crate) fn current_branch(cwd: &Path) -> Option<String> {
    let r = run_git(cwd, &["symbolic-ref", "-q", "--short", "HEAD"]);
    if r.status != Some(0) {
        return None;
    }
    Some(js_trim(&r.stdout.unwrap_or_default()).to_string())
}

/// The two never-throwing `feature` reads behind resolveWorktreeFeature. A
/// missing/corrupt/foreign file is simply "unknown" in BOTH runtimes (the read
/// is a bare `JSON.parse(readFileSync(...))` in a `try`, never fsutil's
/// warning `readJson`), so a parse failure needs no delegation here.
pub(crate) fn read_json_feature(file: &Path) -> Option<String> {
    let raw = std::fs::read(file).ok()?;
    let parsed: Value = serde_json::from_slice(&raw).ok()?;
    match parsed.get("feature") {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

pub(crate) struct WorktreeFeature {
    pub(crate) feature: Option<String>,
    pub(crate) created: Option<String>,
    pub(crate) state_feature: Option<String>,
}

/// resolveWorktreeFeature — the IMMUTABLE creation slug wins over the mutable
/// `state.feature` (issues-46-53 D4), degrading exactly to the pre-fix
/// behavior when no creation record exists.
pub(crate) fn resolve_worktree_feature(worktree_root: &Path) -> WorktreeFeature {
    let created = read_json_feature(
        &worktree_root
            .join(".bee")
            .join("runtime")
            .join("worktree-identity.json"),
    );
    let state_feature = read_json_feature(&worktree_root.join(".bee").join("state.json"));
    WorktreeFeature {
        feature: created.clone().or_else(|| state_feature.clone()),
        created,
        state_feature,
    }
}

/// WT_BRANCH_RE = /^wt\/[a-z0-9][a-z0-9-]*$/.
pub(crate) fn wt_branch_shaped(branch: &str) -> bool {
    branch.strip_prefix("wt/").is_some_and(feature_slug_ok)
}

/// resolveWorktreeById — the same BIDIRECTIONAL gitdir validation
/// resolveRoots uses, keyed by id instead of by walking up from a cwd. `None`
/// on ANY mismatch, missing file or unreadable content, so "no such id" and
/// "id's link is broken" fold into one typed refusal.
///
/// The reverse comparison goes through path_identity's `canonical_paths_equal`
/// (windows-path-identity wpi-1), NOT a byte compare — the shared fix.
pub(crate) fn resolve_worktree_by_id(main_root: &Path, id: &str) -> Option<PathBuf> {
    let git_worktree_dir = main_root.join(".git").join("worktrees").join(id);
    if !std::fs::metadata(&git_worktree_dir).map(|m| m.is_dir()).unwrap_or(false) {
        return None;
    }
    let forward_raw = std::fs::read_to_string(git_worktree_dir.join("gitdir")).ok()?;
    let forward_raw = js_trim(&forward_raw);
    if forward_raw.is_empty() {
        return None;
    }
    let resolved_git_file = js_path_resolve(
        &git_worktree_dir,
        &forward_raw.replace('\\', &MAIN_SEPARATOR.to_string()),
    );
    let worktree_root = resolved_git_file.parent()?.to_path_buf();

    let reverse_raw = std::fs::read_to_string(worktree_root.join(".git")).ok()?;
    let captured = parse_gitdir_pointer(js_trim(&reverse_raw))?;
    let reverse_resolved = js_path_resolve(
        &worktree_root,
        &js_trim(captured).replace('\\', &MAIN_SEPARATOR.to_string()),
    );
    if !crate::path_identity::canonical_paths_equal(&reverse_resolved, &git_worktree_dir) {
        return None;
    }
    Some(worktree_root)
}

/// worktree-holds.mjs releaseAllForHolder — every unreleased hold for `id`,
/// marked released under the shared `cross-worktree-holds` lock on mainRoot.
/// BEST-EFFORT at its one call site (performCleanup wraps it in `try/catch`),
/// so every failure here is swallowed exactly as Node swallows the throw.
pub(crate) fn release_all_for_holder(main_root: &Path, id: &str) {
    let Ok(mut guard) =
        lock::acquire_store_lock(main_root, "cross-worktree-holds", lock::MAX_ATTEMPTS)
    else {
        return;
    };
    let file = main_root
        .join(".bee")
        .join("runtime")
        .join("cross-worktree-holds.json");
    let mut store: Value = match std::fs::read(&file) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        Err(_) => Value::Null,
    };
    let mut released = 0usize;
    let released_at = now_iso();
    if let Some(Value::Array(holds)) = store.get_mut("holds") {
        for hold in holds.iter_mut() {
            let unreleased = matches!(hold.get("released_at"), None | Some(Value::Null));
            if !unreleased {
                continue;
            }
            if !matches!(hold.get("holder"), Some(Value::String(s)) if s == id) {
                continue;
            }
            if let Value::Object(m) = hold {
                m.insert("released_at".into(), Value::String(released_at.clone()));
            }
            released += 1;
        }
    }
    if released > 0 {
        let _ = crate::fsutil::write_json_atomic(&file, &store);
    }
    guard.release();
}

/// The self-delete guard `teardown_worktree` runs before ever removing a
/// directory: `worktree_root` must never be `cwd`, nor an ancestor of it.
/// Pulled out as a pure predicate over an injected `cwd` so it is testable
/// without touching the real process directory — the one production call
/// site always supplies `std::env::current_dir()`.
pub(crate) fn assert_directory_removal_is_safe(cwd: Option<&Path>, worktree_root: &Path) {
    if let Some(cwd) = cwd {
        assert!(
            !cwd.starts_with(worktree_root),
            "teardown_worktree: refusing to remove {} — it contains the current directory ({})",
            p(worktree_root),
            p(cwd)
        );
    }
}

/// The two ways `teardown_worktree`'s directory half can refuse. Only
/// produced when `remove` is `Some` — the registry-only call `run_unregister`
/// makes never reaches either arm.
pub(crate) enum TeardownFailure {
    /// `git worktree remove --force` failed; nothing else ran.
    RemoveFailed(String),
    /// The directory is gone but `git branch -d` (NEVER -D) refused; the
    /// registry half below did not run.
    BranchDeleteFailed(String),
}

/// The five removal steps `performCleanup` used to run inline, lifted into
/// one shared helper (decisions D3, D3a): `git worktree remove --force`,
/// `git branch -d`, grant drop, `ws::unregister_workspace`,
/// `release_all_for_holder`.
///
/// `remove` is the explicit, non-default directory-removal parameter —
/// `Some((worktree_root, branch))` runs the directory and branch steps first
/// and only falls through to the registry half (grant, workspace record,
/// holds) on success; `None` reaches the registry half alone, which is how
/// `run_unregister` wires in without ever touching a directory.
pub(crate) fn teardown_worktree(
    main_root: &Path,
    id: &str,
    remove: Option<(&Path, &str)>,
) -> Result<(), TeardownFailure> {
    if let Some((worktree_root, branch)) = remove {
        assert_directory_removal_is_safe(std::env::current_dir().ok().as_deref(), worktree_root);

        let worktree_root_s = p(worktree_root);
        let remove_result = run_git(
            main_root,
            &["worktree", "remove", "--force", "--", &worktree_root_s],
        );
        if remove_result.status != Some(0) {
            return Err(TeardownFailure::RemoveFailed(remove_result.fail_text()));
        }

        let branch_delete = run_git(main_root, &["branch", "-d", "--", branch]);
        if branch_delete.status != Some(0) {
            return Err(TeardownFailure::BranchDeleteFailed(branch_delete.fail_text()));
        }
    }

    // The three best-effort ledger drops, in Node's order. Each is wrapped in
    // its own `try/catch` there, so every failure — including the ones this
    // port would otherwise call Exotic — is swallowed, not delegated.
    let main_store_root = main_root.join(".bee");
    if let Some(existing) = read_grants_strict(&main_store_root) {
        if existing.contains_key(id) {
            let mut next = existing;
            next.remove(id);
            let _ = write_grants_file_atomic(&main_store_root, &next);
        }
    }
    let _ = ws::unregister_workspace(main_root, id);
    release_all_for_holder(main_root, id);
    Ok(())
}

/// performCleanup (decision D8b): re-check freshness, then run the shared
/// teardown with directory removal on. Never throws — every outcome is the
/// `{ok, code?}` object folded into the merge result's `.cleanup` field, in
/// Node's exact key order. The `Map` construction stays here; only the
/// side-effect calls moved into `teardown_worktree`.
///
/// A live session's cwd must never be deleted out from under it: that
/// strands the session behind WORKTREE_LINK_INVALID denials on every write
/// it makes afterward. So this checks `live_session_holds` (mlsg-1) right
/// before `teardown_worktree`'s `git worktree remove --force`, the same
/// liveness gate `bee worktree prune` already applies.
pub(crate) fn perform_cleanup(
    main_root: &Path,
    worktree_root: &Path,
    branch: &str,
    id: &str,
    proof_unproven: bool,
) -> Map<String, Value> {
    let mut out = Map::new();
    let status = match git_status_porcelain(worktree_root) {
        Ok(s) => s,
        Err(message) => {
            out.insert("ok".into(), Value::Bool(false));
            out.insert("code".into(), json!("WORKTREE_MERGE_CLEANUP_CHECK_FAILED"));
            out.insert("reason".into(), Value::String(message));
            return out;
        }
    };
    if !js_trim(&status).is_empty() {
        out.insert("ok".into(), Value::Bool(false));
        out.insert("code".into(), json!("WORKTREE_MERGE_CLEANUP_DIRTY"));
        out.insert("reason".into(), json!(format!(
            "{} has tracked-modified or untracked files at tracked paths — cleanup refuses. Remove them (a bootstrapped, gitignored .bee store alone does not block cleanup) and retry, or clean up manually.",
            p(worktree_root)
        )));
        out.insert("status".into(), Value::String(status));
        return out;
    }

    // A live session's cwd must never be deleted out from under it — every
    // write it makes after that lands on a directory that no longer exists,
    // and gets denied as WORKTREE_LINK_INVALID with no path back but a
    // restart. Reuse prune's `live_session_holds` (D2b) with the SHORT
    // window, `HEARTBEAT_STALE_SECONDS` (15 minutes, claims.rs) — not
    // prune's own six-hour `PRUNE_LIVENESS_SECONDS` — because a merge that
    // races an active session should refuse promptly, not wait out prune's
    // much longer patience for a worktree its owner might return to.
    if let Some(reason) = live_session_holds(
        main_root,
        id,
        worktree_root,
        crate::verbs::reservations::now_ms(),
        crate::verbs::cells::HEARTBEAT_STALE_SECONDS,
    ) {
        out.insert("ok".into(), Value::Bool(false));
        out.insert("code".into(), json!("WORKTREE_MERGE_CLEANUP_LIVE_SESSION"));
        out.insert("reason".into(), json!(format!(
            "{reason} The merge itself has already landed — only directory removal is deferred. Close the session working in that worktree, then rerun `bee worktree merge --id {id}`, or let `bee worktree prune` sweep the worktree once that session goes stale."
        )));
        return out;
    }

    if let Err(failure) = teardown_worktree(main_root, id, Some((worktree_root, branch))) {
        match failure {
            TeardownFailure::RemoveFailed(reason) => {
                out.insert("ok".into(), Value::Bool(false));
                out.insert("code".into(), json!("WORKTREE_MERGE_CLEANUP_REMOVE_FAILED"));
                out.insert("reason".into(), Value::String(reason));
            }
            TeardownFailure::BranchDeleteFailed(reason) => {
                out.insert("ok".into(), Value::Bool(false));
                out.insert(
                    "code".into(),
                    json!("WORKTREE_MERGE_CLEANUP_BRANCH_DELETE_FAILED"),
                );
                out.insert("removed".into(), Value::Bool(true));
                out.insert("reason".into(), Value::String(reason));
            }
        }
        return out;
    }

    out.insert("ok".into(), Value::Bool(true));
    out.insert("removed".into(), Value::Bool(true));
    out.insert("branch_deleted".into(), Value::Bool(true));
    if proof_unproven {
        out.insert(
            "warning".into(),
            json!("proof unchecked — no capped cell for this feature recorded a valid D8 proof line; cleaned up unchecked."),
        );
    }
    out
}

/// attachCleanupOutcome — runs cleanup, or attaches the suggested command
/// (decision D8b: "never prompt"). wkm-1 (D1): `cleanup` here is already the
/// EFFECTIVE decision — KEEP by default, `--cleanup` or an explicit
/// `worktree_cleanup_on_merge: true` opt a merge in, `--no-cleanup` always
/// wins as an opt-out, and the ALREADY_UP_TO_DATE caller hardcodes it
/// false — so a `false` reaching this function means the default fired, one
/// of the opt-outs fired, or nothing was merged.
pub(crate) fn attach_cleanup_outcome(
    result: &mut Map<String, Value>,
    main_root: &Path,
    worktree_root: &Path,
    branch: &str,
    id: &str,
    cleanup: bool,
    proof_unproven: bool,
) {
    if !cleanup {
        result.insert(
            "cleanup_suggested_command".into(),
            json!(format!("bee worktree merge --id {id} --json")),
        );
        return;
    }
    result.insert(
        "cleanup".into(),
        Value::Object(perform_cleanup(
            main_root,
            worktree_root,
            branch,
            id,
            proof_unproven,
        )),
    );
}

/// D7/D8: the merge result's own `verify` field, reshaped from the retired
/// `VerifyOutcome` (ran/status/combined, a spawned command's exit) to
/// honestly report the D8 proof-check verdict instead — the field name
/// stays (existing key-order tests pin it), but nothing here ever claims a
/// test ran. `None` only when the merging feature could not even be
/// resolved (`identity.feature` was `None`, `merge_stage`'s own
/// zero-mutation posture for that case: nothing to check, so it never
/// blocks either). A `blocking` verdict never reaches this — `merge_stage`
/// already refused before `git merge` ever ran.
pub(crate) fn proof_report_field(proof: Option<&crate::verbs::cells::ProofCheck>) -> String {
    let Some(proof) = proof else {
        return "unchecked (feature unresolved)".to_string();
    };
    if proof.proven_count == 0 && proof.legacy_count == 0 {
        return "unchecked (no capped cells)".to_string();
    }
    if proof.proven_count == 0 {
        return format!("unchecked ({} legacy cap(s), no proof line)", proof.legacy_count);
    }
    if proof.legacy_count > 0 {
        return format!("proven ({} cell(s); {} legacy)", proof.proven_count, proof.legacy_count);
    }
    format!("proven ({} cell(s))", proof.proven_count)
}

/// wkm-1 (D1): the keep path's cross-check record. A green (or
/// verify-skipped) merge that does NOT tear down the worktree appends
/// exactly one `worktree-cleanup` deferred-queue entry, so the user has a
/// durable pointer back to it — `bee worktree prune` is the drain that
/// resolves the entry once it removes the worktree. `feature` prefers the
/// worktree's IMMUTABLE creation-identity slug (`resolve_worktree_feature`,
/// issues-46-53 D4) and falls back to the worktree id when no creation
/// record exists. Best-effort, the same shape close.rs's own scribe/promote
/// enqueues use: a queue-append failure here must never fail an
/// already-committed merge.
pub(crate) fn enqueue_worktree_cleanup_deferral(
    main_root: &Path,
    worktree_root: &Path,
    id: &str,
    branch: &str,
    merge_commit: &str,
) {
    let feature = resolve_worktree_feature(worktree_root)
        .created
        .unwrap_or_else(|| id.to_string());
    let reason = format!(
        "Worktree {id} (branch {branch}) merged into main at {merge_commit} and kept per default (D1) — remove it with `bee worktree prune`."
    );
    let _ = crate::verbs::deferred_queue::enqueue(
        main_root,
        "worktree-cleanup",
        &feature,
        &[],
        &[],
        &[p(worktree_root)],
        &reason,
    );
}

/// checkMergeFence — P3's SECOND line of defense (the processor-lease epoch is
/// the first). A short drift description, or `None` when the fence is clean.
/// The two `.stdout.trim()` sites here carry the same unreachability argument
/// `main_untouched_proof` documents.
pub(crate) fn check_merge_fence(
    main_root: &Path,
    id: &str,
    pre_merge_head: &str,
    merge_head_file: &Path,
    staged_tree_hash: &str,
) -> Option<String> {
    let head_now = js_trim(&run_git(main_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default())
        .to_string();
    if head_now != pre_merge_head {
        return Some(format!(
            "HEAD moved from {pre_merge_head} to {head_now} while the lock was released for verify"
        ));
    }
    if !merge_head_file.exists() {
        return Some(
            ".git/MERGE_HEAD disappeared while the lock was released for verify — the staged merge was cleared out from under this operation"
                .to_string(),
        );
    }
    let tree_now =
        js_trim(&run_git(main_root, &["write-tree"]).stdout.unwrap_or_default()).to_string();
    if tree_now != staged_tree_hash {
        return Some(format!(
            "the staged tree changed from {staged_tree_hash} to {tree_now} while the lock was released for verify — the index was mutated mid-verify"
        ));
    }
    // readGrants swallows a parse error and reads `{}`; read_grants_strict
    // delegates instead — but `run_merge` already probed the registry before
    // any lock, so `None` here is a mid-merge race, treated as "revoked"
    // exactly like an absent entry would be.
    let granted = read_grants_strict(&main_root.join(".bee"))
        .map(|g| g.get(id) == Some(&Value::Bool(true)))
        .unwrap_or(false);
    if !granted {
        return Some(format!(
            "the grant for worktree id {} was revoked while the lock was released for verify",
            jsjson::stringify(&Value::String(id.to_string()))
        ));
    }
    None
}

// ─── the companion-command shell (retired verify child, D7/D8) ────────────
//
// td-3 (docs/history/test-doctrine/CONTEXT.md, D7/D8): the P2 "verify
// child" that used to spawn `commands.test` unlocked against the
// merged-but-uncommitted tree, plus its `shell_launchable()` pre-check and
// `VerifyOutcome`/`run_verify_child`, are RETIRED — `bee worktree merge`
// never spawns `commands.test` itself anymore. `merge_stage` (phases.rs)
// now reads the merging feature's D8 proof lines instead
// (`crate::verbs::cells::feature_proof_check`), as a zero-mutation
// precondition BEFORE `git merge` ever runs, so there is no post-merge
// red to abort and no shell-launch race to pre-check. `shell_child` itself
// stays: `companion.rs`'s `commands.worktree_companion_end` spawn still
// needs it.

/// Node's `spawn(command, { shell: true })` file/args, faithfully: on win32
/// `process.env.comspec || 'cmd.exe'` with `/d /s /c "<command>"` passed
/// VERBATIM; elsewhere `/bin/sh -c <command>`. Deliberately NOT
/// verbs/cells.rs's `spawn_declared`, which prefers Git Bash on win32.
pub(crate) fn shell_child(command: &str) -> std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let file = std::env::var("comspec").unwrap_or_else(|_| "cmd.exe".to_string());
        let mut c = std::process::Command::new(file);
        c.raw_arg(format!("/d /s /c \"{command}\""));
        c
    }
    #[cfg(not(windows))]
    {
        let mut c = std::process::Command::new("/bin/sh");
        c.args(["-c", command]);
        c
    }
}
