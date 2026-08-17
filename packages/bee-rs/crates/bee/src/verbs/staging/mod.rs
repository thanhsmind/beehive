// bee staging — the mixing-ground lane (staging-lane D0/D0a).
//
//   bee staging add --feature <slug>   [--json]
//
// D0: staging is a MIXING GROUND, never a source of truth — code truth lives
// on feature branches; staging exists so several features can be exercised
// together before UAT. D0a (lifecycle): staging is created LAZILY, always
// FROM CURRENT MAIN, and it never self-updates — the only writer of its
// branch/worktree is `staging add` (this cell) and `staging rebuild`
// (sl-2, a sibling cell), never a plain commit landing on it by hand.
//
// Store: `.bee/runtime/staging.json` on the MAIN root —
//   {branch, worktree_root, created_at, base_sha,
//    staged: [{feature, branch, last_merged_sha, at}]}
// — written only through `write_json_atomic` (temp-then-rename), the same
// atomic-write shape `verbs/worktree/registry.rs`'s grant registry uses, and
// only ever from this CLI (no other writer exists).
//
// `add`'s shape mirrors `verbs/worktree/create.rs`'s `create_feature_worktree`
// and `verbs/worktree/merge.rs`'s staged transaction closely on purpose: a
// `worktree-admin`-shaped hold (here `staging-admin`, its own lock name — a
// concurrent `staging add`/`worktree new` must never contend on the same
// name for two different resources) around the lazy create + merge + record
// write, released before the (potentially slow) build hook runs.
//
// The public, cwd-INDEPENDENT core (`staging_add`) takes `main_root`
// explicitly and never touches `std::env::current_dir()` — the same split
// `create_feature_worktree`/`create_feature_worktree_locked` keep, so tests
// exercise the real logic without a process-wide cwd race. `run_add` is the
// thin CLI layer: it resolves `main_root` from the process cwd via
// `worktree::prelude` (the "am I the main checkout" question already asked
// the same way every other worktree-topology verb asks it) and then calls
// straight into `staging_add`.

use crate::fsutil::{write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::verbs::reservations::{js_trim, now_iso, parse_flags, FlagV, Flags};
use crate::verbs::workflow_store;
use crate::verbs::worktree;
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

/// bee's own lock name for this module — distinct from worktree-store.mjs's
/// `worktree-admin` (a different resource: the staging branch/worktree/store,
/// not the grant registry), so the two never contend on the same name for
/// two different things.
const STAGING_ADMIN_LOCK: &str = "staging-admin";

/// `[CODE] message` — the one observable byte of every refusal here, same
/// convention `worktree/create.rs`'s `refuse`/`merge.rs`'s `refuse_merge`
/// use.
fn refuse(code: &str, message: String) -> String {
    format!("[{code}] {message}")
}

/// staging-lane D0 teeth #2: the write-guard hook (hooks/write_guard/
/// checks.rs's `staging_worktree_commit_denial`) refuses a `git commit`
/// with cwd inside the staging worktree unless this marker is set — set it
/// around this module's OWN merge commits (the two sanctioned writers of
/// the staging branch) so `staging add`/`staging rebuild` never trip their
/// own guard. Scoped RAII, same shape `worktree/tests.rs`'s
/// `GitCeilingGuard` uses for `GIT_CEILING_DIRECTORIES`: `new` records
/// whatever value (or absence) was already there and restores it on
/// `Drop`, so a caller can never forget to unwind it, even on an early
/// return via `?`.
struct StagingMachineryGuard {
    prior: Option<std::ffi::OsString>,
}

impl StagingMachineryGuard {
    fn new() -> Self {
        let prior = std::env::var_os("BEE_STAGING_MACHINERY");
        // SAFETY: `staging add`/`staging rebuild` run their own merge
        // commits under the `staging-admin` lock, single-threaded within
        // this process — no other thread reads or writes this var while
        // this guard is live.
        unsafe { std::env::set_var("BEE_STAGING_MACHINERY", "1") };
        StagingMachineryGuard { prior }
    }
}

impl Drop for StagingMachineryGuard {
    fn drop(&mut self) {
        // SAFETY: see `new` above.
        match self.prior.take() {
            Some(v) => unsafe { std::env::set_var("BEE_STAGING_MACHINERY", v) },
            None => unsafe { std::env::remove_var("BEE_STAGING_MACHINERY") },
        }
    }
}

// ─── the staged-set store ───────────────────────────────────────────────

pub(crate) fn staging_file(main_root: &Path) -> PathBuf {
    main_root.join(".bee").join("runtime").join("staging.json")
}

#[derive(Clone, Debug)]
pub(crate) struct StagedEntry {
    pub(crate) feature: String,
    pub(crate) branch: String,
    pub(crate) last_merged_sha: String,
    pub(crate) at: String,
}

impl StagedEntry {
    fn to_value(&self) -> Value {
        json!({
            "feature": self.feature,
            "branch": self.branch,
            "last_merged_sha": self.last_merged_sha,
            "at": self.at,
        })
    }

    fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        Some(Self {
            feature: m.get("feature")?.as_str()?.to_string(),
            branch: m.get("branch")?.as_str()?.to_string(),
            last_merged_sha: m.get("last_merged_sha").and_then(Value::as_str).unwrap_or("").to_string(),
            at: m.get("at").and_then(Value::as_str).unwrap_or("").to_string(),
        })
    }
}

pub(crate) struct StagingRecord {
    pub(crate) branch: String,
    pub(crate) worktree_root: PathBuf,
    pub(crate) created_at: String,
    pub(crate) base_sha: String,
    pub(crate) staged: Vec<StagedEntry>,
}

impl StagingRecord {
    fn to_value(&self) -> Value {
        json!({
            "branch": self.branch,
            "worktree_root": worktree::p(&self.worktree_root),
            "created_at": self.created_at,
            "base_sha": self.base_sha,
            "staged": self.staged.iter().map(StagedEntry::to_value).collect::<Vec<_>>(),
        })
    }

    fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        let branch = m.get("branch")?.as_str()?.to_string();
        let worktree_root = PathBuf::from(m.get("worktree_root")?.as_str()?);
        let created_at = m.get("created_at").and_then(Value::as_str).unwrap_or("").to_string();
        let base_sha = m.get("base_sha")?.as_str()?.to_string();
        let staged = match m.get("staged") {
            Some(Value::Array(arr)) => arr.iter().filter_map(StagedEntry::from_value).collect(),
            _ => Vec::new(),
        };
        Some(Self { branch, worktree_root, created_at, base_sha, staged })
    }
}

/// `None` — no record yet (lazy-create path). `Some(Err(..))` — a record
/// exists but is not readable/shaped right; refused rather than silently
/// overwritten (the store is CLI-only; a hand-edited or corrupt file is a
/// human problem, never papered over by a fresh create that would orphan the
/// real staging branch/worktree it already points at).
pub(crate) fn read_staging_record(main_root: &Path) -> Result<Option<StagingRecord>, String> {
    let file = staging_file(main_root);
    match crate::fsutil::read_json(&file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(refuse(
            "STAGING_STORE_CORRUPT",
            format!("{} exists but is not valid JSON — fix or remove it by hand before retrying.", worktree::p(&file)),
        )),
        ReadJson::Parsed(v) => match StagingRecord::from_value(&v) {
            Some(r) => Ok(Some(r)),
            None => Err(refuse(
                "STAGING_STORE_CORRUPT",
                format!(
                    "{} does not match the expected staging record shape (branch/worktree_root/base_sha/staged) — fix or remove it by hand before retrying.",
                    worktree::p(&file)
                ),
            )),
        },
    }
}

fn write_staging_record(main_root: &Path, record: &StagingRecord) -> std::io::Result<()> {
    write_json_atomic(&staging_file(main_root), &record.to_value())
}

// ─── lazy create (D0a: only ever from CURRENT main) ────────────────────

fn create_staging(main_root: &Path) -> Result<StagingRecord, String> {
    let branch = "staging";
    if worktree::branch_exists(main_root, branch) {
        return Err(refuse(
            "STAGING_BRANCH_EXISTS",
            format!(
                "branch \"staging\" already exists in {} but no staging record was found at {} — remove the branch (and its worktree, if any, via \"git worktree list\"/\"git worktree remove\") or restore the record by hand before retrying.",
                worktree::p(main_root),
                worktree::p(&staging_file(main_root))
            ),
        ));
    }
    let repo_basename = main_root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let sibling_dir_name = format!("{repo_basename}--wt--staging");
    let worktree_root = worktree::js_path_resolve(&worktree::js_path_resolve(main_root, ".."), &sibling_dir_name);
    if worktree_root.exists() {
        return Err(refuse(
            "STAGING_TARGET_EXISTS",
            format!("{} already exists.", worktree::p(&worktree_root)),
        ));
    }
    // The RESOLVED HEAD sha, read BEFORE `git worktree add` — the base_sha
    // this cell records is exactly the commit staging was cut from, never a
    // ref name that could later move.
    let base_sha = js_trim(&worktree::run_git(main_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
    if base_sha.is_empty() {
        return Err(refuse(
            "STAGING_BASE_NOT_FOUND",
            format!(
                "could not resolve HEAD in {} (\"git rev-parse HEAD\" found nothing) — is this a git repo with at least one commit?",
                worktree::p(main_root)
            ),
        ));
    }
    // No explicit base ref: `git worktree add -b staging <path>` bases the
    // new branch on the CURRENT HEAD of the checkout it runs in — main's,
    // because `staging_add`'s caller has already proven `main_root` is an
    // ordinary checkout, never a feature branch's.
    let worktree_root_s = worktree::p(&worktree_root);
    let add_result = worktree::run_git(main_root, &["worktree", "add", "-b", branch, "--", &worktree_root_s]);
    if add_result.status != Some(0) {
        return Err(refuse(
            "STAGING_WORKTREE_CREATE_FAILED",
            format!("\"git worktree add -b staging {worktree_root_s}\" failed: {}", add_result.fail_text()),
        ));
    }
    Ok(StagingRecord {
        branch: branch.to_string(),
        worktree_root,
        created_at: now_iso(),
        base_sha,
        staged: Vec::new(),
    })
}

// ─── resolving --feature's branch (the same order worktree merge trusts) ──

/// Creation identity / registered worktree first (the SAME source
/// `verbs/worktree/merge.rs`'s own branch-mismatch check trusts —
/// `resolve_worktree_by_id` + `resolve_worktree_feature`), a plain
/// `wt/<slug>` fallback second — the repo's own worktree-new convention,
/// for a feature branch that exists without (or outside) a live grant.
/// `None` when neither resolves to a real branch.
fn resolve_feature_branch(main_root: &Path, feature: &str) -> Option<String> {
    let main_store_root = main_root.join(".bee");
    if let Some(grants) = worktree::read_grants_strict(&main_store_root) {
        for (id, granted) in grants.iter() {
            if *granted != Value::Bool(true) {
                continue;
            }
            let Some(root) = worktree::resolve_worktree_by_id(main_root, id) else {
                continue;
            };
            let resolved = worktree::resolve_worktree_feature(&root);
            if resolved.feature.as_deref() != Some(feature) {
                continue;
            }
            if let Some(branch) = worktree::current_branch(&root) {
                return Some(branch);
            }
        }
    }
    let fallback = format!("wt/{feature}");
    if worktree::branch_exists(main_root, &fallback) {
        Some(fallback)
    } else {
        None
    }
}

// ─── the build hook (commands.staging_build) ───────────────────────────

/// `Ok(note)` on both "ran green" and "not configured, skipped" — a skip is
/// success, always visible in the returned line, never an error. `Err`
/// carries the tail (last 30 lines, the same shape `worktree/phases.rs`'s
/// MERGE_VERIFY_RED already uses) only on a real non-zero exit / launch
/// failure.
fn run_staging_build(main_root: &Path, staging_worktree_root: &Path) -> Result<String, String> {
    let config = crate::state::read_config_raw(main_root);
    let command = config
        .get("commands")
        .and_then(|c| c.get("staging_build"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(str::to_string);
    let Some(command) = command else {
        return Ok("commands.staging_build not configured — build step skipped.".to_string());
    };
    let output = worktree::shell_child(&command)
        .current_dir(staging_worktree_root)
        .stdin(std::process::Stdio::null())
        .output();
    let out = match output {
        Ok(o) => o,
        Err(e) => {
            return Err(format!(
                "could not launch commands.staging_build ({command:?}) in {}: {e}",
                worktree::p(staging_worktree_root)
            ))
        }
    };
    if !out.status.success() {
        let combined = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        let lines: Vec<&str> = combined.split('\n').collect();
        let tail = lines[lines.len().saturating_sub(30)..].join("\n");
        let code_disp = out.status.code().map_or_else(|| "signal".to_string(), |c| c.to_string());
        return Err(format!(
            "commands.staging_build ({command:?}) exited {code_disp} in {}:\n{tail}",
            worktree::p(staging_worktree_root)
        ));
    }
    Ok(format!("commands.staging_build ({command:?}) ran green in {}.", worktree::p(staging_worktree_root)))
}

// ─── the whole `add`, cwd-independent ──────────────────────────────────

#[derive(Debug)]
pub(crate) struct AddOutcome {
    pub(crate) feature: String,
    pub(crate) branch: String,
    pub(crate) staging_worktree_root: PathBuf,
    pub(crate) last_merged_sha: String,
    pub(crate) staged: Vec<StagedEntry>,
    pub(crate) build_note: String,
}

/// The whole `staging add`, given an already-resolved `main_root` — never
/// touches `std::env::current_dir()`, so tests call it directly with an
/// explicit path (including a NON-main one, to prove the D0a refusal, the
/// same belt-and-braces shape `create_feature_worktree_locked` keeps its own
/// `is_ordinary_checkout` check even though its CLI caller already asked the
/// cwd-based question).
pub(crate) fn staging_add(main_root: &Path, feature: &str) -> Result<AddOutcome, String> {
    if !worktree::feature_slug_ok(feature) {
        return Err(refuse(
            "STAGING_INVALID_SLUG",
            format!(
                "feature slug {} must match /^[a-z0-9][a-z0-9-]*$/ (lowercase letters/digits, starting with a letter or digit, hyphens allowed after that).",
                jsjson::stringify(&Value::String(feature.to_string()))
            ),
        ));
    }
    if !worktree::is_ordinary_checkout(main_root) {
        return Err(refuse(
            "STAGING_NOT_MAIN_CHECKOUT",
            format!(
                "\"bee staging add\" must be run from the main checkout — {} is not an ordinary checkout. Staging is lazily created FROM MAIN and can only ever be triggered from there (staging-lane D0a).",
                worktree::p(main_root)
            ),
        ));
    }

    let mut guard = match lock::acquire_store_lock(main_root, STAGING_ADMIN_LOCK, lock::MAX_ATTEMPTS) {
        Ok(g) => g,
        Err(busy) => return Err(busy.message()),
    };
    let staged = {
        let _machinery = StagingMachineryGuard::new();
        staging_add_locked(main_root, feature)
    };
    guard.release();
    staged
}

fn staging_add_locked(main_root: &Path, feature: &str) -> Result<AddOutcome, String> {
    let mut record = match read_staging_record(main_root)? {
        Some(r) => r,
        None => create_staging(main_root)?,
    };

    let branch = resolve_feature_branch(main_root, feature).ok_or_else(|| {
        refuse(
            "STAGING_FEATURE_BRANCH_NOT_FOUND",
            format!(
                "could not resolve a branch for feature \"{feature}\" — neither a registered worktree nor a \"wt/{feature}\" branch was found in {}. Create the feature first (\"bee worktree new --feature {feature}\"), or check the slug.",
                worktree::p(main_root)
            ),
        )
    })?;

    let merge_out = worktree::run_git(&record.worktree_root, &["merge", "--no-ff", "--", &branch]);
    if merge_out.status != Some(0) {
        // Conflicting paths, read BEFORE `merge --abort` clears the index —
        // the whole point of naming them in the refusal.
        let conflicts = worktree::run_git(&record.worktree_root, &["diff", "--name-only", "--diff-filter=U"]);
        let files = js_trim(&conflicts.stdout.unwrap_or_default()).replace('\n', ", ");
        worktree::run_git(&record.worktree_root, &["merge", "--abort"]);
        let files_disp = if files.is_empty() { "(no files listed)".to_string() } else { files };
        return Err(refuse(
            "STAGING_MERGE_CONFLICT",
            format!(
                "merging feature \"{feature}\" (branch \"{branch}\") into staging at {} hit a conflict — the merge was aborted and staging was left usable. Conflicting file(s): {files_disp}. Fix on the feature branch, then \"bee staging add --feature {feature}\" again.",
                worktree::p(&record.worktree_root)
            ),
        ));
    }

    let last_merged_sha =
        js_trim(&worktree::run_git(&record.worktree_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();

    let at = now_iso();
    if let Some(existing) = record.staged.iter_mut().find(|e| e.feature == feature) {
        existing.branch = branch.clone();
        existing.last_merged_sha = last_merged_sha.clone();
        existing.at = at.clone();
    } else {
        record.staged.push(StagedEntry {
            feature: feature.to_string(),
            branch: branch.clone(),
            last_merged_sha: last_merged_sha.clone(),
            at,
        });
    }

    write_staging_record(main_root, &record).map_err(|_| {
        refuse(
            "STAGING_STORE_WRITE_FAILED",
            format!(
                "\"{feature}\" merged into staging at {} (sha {last_merged_sha}) but writing {} failed — re-run \"bee staging add --feature {feature}\" to repair the record (git only replays new commits, so this is safe).",
                worktree::p(&record.worktree_root),
                worktree::p(&staging_file(main_root))
            ),
        )
    })?;

    // The build hook runs UNLOCKED (the store write above already landed):
    // a slow build must never hold `staging-admin` against a sibling
    // `staging add` for a different feature. Its own failure is still typed
    // (STAGING_BUILD_FAILED) and still leaves the just-recorded merge in
    // place — the merge genuinely landed; only the build step is red.
    let build_note = run_staging_build(main_root, &record.worktree_root).map_err(|msg| refuse("STAGING_BUILD_FAILED", msg))?;

    Ok(AddOutcome {
        feature: feature.to_string(),
        branch,
        staging_worktree_root: record.worktree_root.clone(),
        last_merged_sha,
        staged: record.staged.clone(),
        build_note,
    })
}

// ─── the uat gate read (uat-gate-before-merge D1's signal) ─────────────────

/// D0a trigger 3's "awaiting UAT" question: is a staged feature's uat gate
/// approved? Read from the feature's OWN workflow record on `main_root` —
/// never staging's (staging never carries workflow state, D0). `Unknown`
/// covers both "no live workflow record names this feature" and "found one
/// but its gate state is not a recognized value" — either way `rebuild`
/// fails TOWARD keeping the feature staged (a gate this code cannot prove
/// approved is never treated as approved).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum UatGate {
    Approved,
    Pending,
    Unknown,
}

impl UatGate {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            UatGate::Approved => "approved",
            UatGate::Pending => "pending",
            UatGate::Unknown => "unknown",
        }
    }
}

pub(crate) fn feature_uat_gate(main_root: &Path, feature: &str) -> UatGate {
    let workflows = match workflow_store::list_workflows(main_root) {
        Ok(w) => w,
        Err(_) => return UatGate::Unknown,
    };
    let Some(wf) = workflow_store::find_live_workflow(&workflows, feature) else {
        return UatGate::Unknown;
    };
    let state = wf
        .get("gates")
        .and_then(|g| g.get("uat"))
        .and_then(|u| u.get("state"))
        .and_then(Value::as_str);
    match state {
        Some("approved") => UatGate::Approved,
        Some("pending") | Some("rejected") => UatGate::Pending,
        _ => UatGate::Unknown,
    }
}

// ─── rebuild (D0a trigger 3): reset --hard main, re-derive the invariant ───

#[derive(Debug)]
pub(crate) struct RebuildOutcome {
    pub(crate) staging_worktree_root: PathBuf,
    pub(crate) base_sha: String,
    pub(crate) staged: Vec<StagedEntry>,
    pub(crate) merged: Vec<String>,
    pub(crate) dropped: Vec<String>,
    pub(crate) without: Vec<String>,
    pub(crate) conflicts: Vec<String>,
    pub(crate) build_note: String,
}

/// The whole `staging rebuild`, cwd-INDEPENDENT (same split `staging_add`
/// keeps) — `main_root` explicit, `without` the already-parsed exclusion
/// slugs. Locked the same way `staging_add` is (STAGING_ADMIN_LOCK around
/// the reset+recompute+merges+record write; the build hook runs unlocked
/// afterward, same reasoning as `staging_add`'s).
pub(crate) fn staging_rebuild(main_root: &Path, without: &[String]) -> Result<RebuildOutcome, String> {
    if !worktree::is_ordinary_checkout(main_root) {
        return Err(refuse(
            "STAGING_NOT_MAIN_CHECKOUT",
            format!(
                "\"bee staging rebuild\" must be run from the main checkout — {} is not an ordinary checkout. Staging is only ever mutated from there (staging-lane D0a).",
                worktree::p(main_root)
            ),
        ));
    }

    let mut guard = match lock::acquire_store_lock(main_root, STAGING_ADMIN_LOCK, lock::MAX_ATTEMPTS) {
        Ok(g) => g,
        Err(busy) => return Err(busy.message()),
    };
    let result = {
        let _machinery = StagingMachineryGuard::new();
        staging_rebuild_locked(main_root, without)
    };
    guard.release();
    let mut outcome = result?;

    // Unlocked, same reasoning as `staging_add`'s: a slow build must never
    // hold `staging-admin` against a sibling `staging add`/`rebuild`. The
    // record above already reflects the recomputed staged set — a build
    // failure here is typed but never un-recomputes it.
    outcome.build_note =
        run_staging_build(main_root, &outcome.staging_worktree_root).map_err(|msg| refuse("STAGING_BUILD_FAILED", msg))?;
    Ok(outcome)
}

fn staging_rebuild_locked(main_root: &Path, without: &[String]) -> Result<RebuildOutcome, String> {
    let mut record = match read_staging_record(main_root)? {
        Some(r) => r,
        None => {
            return Err(refuse(
                "STAGING_NO_RECORD",
                format!(
                    "no staging record found at {} — nothing to rebuild yet. Run \"bee staging add --feature <slug>\" first.",
                    worktree::p(&staging_file(main_root))
                ),
            ))
        }
    };

    let reset = worktree::run_git(&record.worktree_root, &["reset", "--hard", "main"]);
    if reset.status != Some(0) {
        return Err(refuse(
            "STAGING_RESET_FAILED",
            format!(
                "\"git reset --hard main\" failed in {}: {}",
                worktree::p(&record.worktree_root),
                reset.fail_text()
            ),
        ));
    }
    let base_sha =
        js_trim(&worktree::run_git(&record.worktree_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();

    let old_staged = std::mem::take(&mut record.staged);
    let mut new_staged: Vec<StagedEntry> = Vec::new();
    let mut merged: Vec<String> = Vec::new();
    let mut dropped: Vec<String> = Vec::new();
    let mut skipped_without: Vec<String> = Vec::new();
    let mut conflicts: Vec<String> = Vec::new();

    for entry in old_staged {
        // Auto-drop FIRST, regardless of --without: an approved-uat or
        // branch-gone feature is no longer "awaiting UAT" — the invariant
        // (staging = main + Σ features awaiting UAT) re-derives on every
        // rebuild, never accumulates history (D0a).
        let branch_alive = worktree::branch_exists(main_root, &entry.branch);
        let gate = feature_uat_gate(main_root, &entry.feature);
        if !branch_alive || gate == UatGate::Approved {
            dropped.push(entry.feature.clone());
            continue;
        }
        if without.iter().any(|f| f == &entry.feature) {
            skipped_without.push(entry.feature.clone());
            new_staged.push(entry);
            continue;
        }
        let merge_out = worktree::run_git(&record.worktree_root, &["merge", "--no-ff", "--", &entry.branch]);
        if merge_out.status != Some(0) {
            // Same conflict-abort shape `staging_add_locked` uses: conflicting
            // paths read BEFORE `merge --abort` clears the index, then abort,
            // then CONTINUE with the rest — a single broken feature never
            // blocks testing the others (D0a conflict policy).
            let conflict_files = worktree::run_git(&record.worktree_root, &["diff", "--name-only", "--diff-filter=U"]);
            let files = js_trim(&conflict_files.stdout.unwrap_or_default()).replace('\n', ", ");
            worktree::run_git(&record.worktree_root, &["merge", "--abort"]);
            let files_disp = if files.is_empty() { "(no files listed)".to_string() } else { files };
            conflicts.push(refuse(
                "STAGING_MERGE_CONFLICT",
                format!(
                    "rebuild: merging feature \"{}\" (branch \"{}\") into staging at {} hit a conflict — the merge was aborted; \"{}\" stays staged at its last successful merge. Conflicting file(s): {}. Fix on the feature branch, then \"bee staging rebuild\" again (or \"bee staging add --feature {}\").",
                    entry.feature,
                    entry.branch,
                    worktree::p(&record.worktree_root),
                    entry.feature,
                    files_disp,
                    entry.feature
                ),
            ));
            new_staged.push(entry);
            continue;
        }
        let last_merged_sha =
            js_trim(&worktree::run_git(&record.worktree_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        let mut updated = entry.clone();
        updated.last_merged_sha = last_merged_sha;
        updated.at = now_iso();
        merged.push(updated.feature.clone());
        new_staged.push(updated);
    }

    record.base_sha = base_sha.clone();
    record.staged = new_staged.clone();
    write_staging_record(main_root, &record).map_err(|_| {
        refuse(
            "STAGING_STORE_WRITE_FAILED",
            format!(
                "rebuild recomputed the staged set (base_sha {base_sha}) but writing {} failed — re-run \"bee staging rebuild\" to repair the record.",
                worktree::p(&staging_file(main_root))
            ),
        )
    })?;

    Ok(RebuildOutcome {
        staging_worktree_root: record.worktree_root.clone(),
        base_sha,
        staged: new_staged,
        merged,
        dropped,
        without: skipped_without,
        conflicts,
        build_note: String::new(),
    })
}

// ─── status (zero mutation): the staged set, gates, staleness ─────────────

pub(crate) struct StatusEntry {
    pub(crate) feature: String,
    pub(crate) branch: String,
    pub(crate) uat_gate: &'static str,
    pub(crate) last_merged_sha: String,
    pub(crate) at: String,
}

pub(crate) struct StatusOutcome {
    pub(crate) branch: String,
    pub(crate) worktree_root: PathBuf,
    pub(crate) base_sha: String,
    pub(crate) current_main_sha: String,
    pub(crate) stale_base: bool,
    pub(crate) staged: Vec<StatusEntry>,
}

/// `Ok(None)` — no staging record yet (never an error: `status` is a plain
/// query, it never refuses on the same absence `rebuild` does). Zero
/// mutation: no lock, no write, no `git reset`/`merge`.
pub(crate) fn staging_status(main_root: &Path) -> Result<Option<StatusOutcome>, String> {
    let Some(record) = read_staging_record(main_root)? else {
        return Ok(None);
    };
    let current_main_sha =
        js_trim(&worktree::run_git(main_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
    // The trigger-3 reminder: staging drifts behind main the moment main
    // moves after staging was cut/rebuilt — this is the only signal that
    // says so, never derived from a file timestamp.
    let stale_base = !record.base_sha.is_empty() && current_main_sha != record.base_sha;
    let staged = record
        .staged
        .iter()
        .map(|e| StatusEntry {
            feature: e.feature.clone(),
            branch: e.branch.clone(),
            uat_gate: feature_uat_gate(main_root, &e.feature).as_str(),
            last_merged_sha: e.last_merged_sha.clone(),
            at: e.at.clone(),
        })
        .collect();
    Ok(Some(StatusOutcome {
        branch: record.branch.clone(),
        worktree_root: record.worktree_root.clone(),
        base_sha: record.base_sha.clone(),
        current_main_sha,
        stale_base,
        staged,
    }))
}

// ─── CLI layer ──────────────────────────────────────────────────────────

pub(crate) fn run_add(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["feature"]) {
        return None;
    }
    let feature = match flags.get("feature") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };

    let ctx = match worktree::prelude("staging add", use_json, t0)? {
        worktree::Pre::Go(c) => c,
        worktree::Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&refuse(
            "STAGING_NOT_MAIN_CHECKOUT",
            format!(
                "\"bee staging add\" must be run from inside the main checkout, not a \"{}\" checkout — staging is lazily created FROM MAIN and can only ever be triggered from there (staging-lane D0a).",
                ctx.kind
            ),
        )));
    }
    let main_root = ctx.work_root.clone();

    match staging_add(&main_root, &feature) {
        Ok(outcome) => {
            let mut result = Map::new();
            result.insert("ok".into(), Value::Bool(true));
            result.insert("feature".into(), json!(outcome.feature));
            result.insert("branch".into(), json!(outcome.branch));
            result.insert("staging_worktree_root".into(), json!(worktree::p(&outcome.staging_worktree_root)));
            result.insert("last_merged_sha".into(), json!(outcome.last_merged_sha));
            result.insert("staged".into(), Value::Array(outcome.staged.iter().map(StagedEntry::to_value).collect()));
            result.insert("build".into(), json!(outcome.build_note));
            let text = format!(
                "Merged \"{}\" (branch \"{}\") into staging at {}.\n  last_merged_sha: {}\n  {}",
                outcome.feature,
                outcome.branch,
                worktree::p(&outcome.staging_worktree_root),
                outcome.last_merged_sha,
                outcome.build_note
            );
            Some(ctx.emit(&Value::Object(result), &text))
        }
        Err(message) => Some(ctx.fail(&message)),
    }
}

pub(crate) fn run_rebuild(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["without"]) {
        return None;
    }
    let without: Vec<String> = match flags.get("without") {
        Some(FlagV::S(s)) => s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect(),
        Some(FlagV::Present) => return None, // --without requires a value
        None => Vec::new(),
    };

    let ctx = match worktree::prelude("staging rebuild", use_json, t0)? {
        worktree::Pre::Go(c) => c,
        worktree::Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&refuse(
            "STAGING_NOT_MAIN_CHECKOUT",
            format!(
                "\"bee staging rebuild\" must be run from inside the main checkout, not a \"{}\" checkout — staging is only ever mutated from there (staging-lane D0a).",
                ctx.kind
            ),
        )));
    }
    let main_root = ctx.work_root.clone();

    match staging_rebuild(&main_root, &without) {
        Ok(outcome) => {
            let mut result = Map::new();
            result.insert("ok".into(), Value::Bool(true));
            result.insert("base_sha".into(), json!(outcome.base_sha));
            result.insert("staging_worktree_root".into(), json!(worktree::p(&outcome.staging_worktree_root)));
            result.insert("merged".into(), json!(outcome.merged));
            result.insert("dropped".into(), json!(outcome.dropped));
            result.insert("without".into(), json!(outcome.without));
            result.insert("conflicts".into(), json!(outcome.conflicts));
            result.insert("staged".into(), Value::Array(outcome.staged.iter().map(StagedEntry::to_value).collect()));
            result.insert("build".into(), json!(outcome.build_note));
            let mut lines = vec![format!(
                "Rebuilt staging at {} (base_sha {}).",
                worktree::p(&outcome.staging_worktree_root),
                outcome.base_sha
            )];
            if !outcome.merged.is_empty() {
                lines.push(format!("  merged: {}", outcome.merged.join(", ")));
            }
            if !outcome.dropped.is_empty() {
                lines.push(format!("  dropped (uat approved or branch gone): {}", outcome.dropped.join(", ")));
            }
            if !outcome.without.is_empty() {
                lines.push(format!("  skipped (--without): {}", outcome.without.join(", ")));
            }
            if !outcome.conflicts.is_empty() {
                lines.push("  conflicts:".to_string());
                for c in &outcome.conflicts {
                    lines.push(format!("    {c}"));
                }
            }
            lines.push(format!("  {}", outcome.build_note));
            let text = lines.join("\n");
            Some(ctx.emit(&Value::Object(result), &text))
        }
        Err(message) => Some(ctx.fail(&message)),
    }
}

pub(crate) fn run_status(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match worktree::prelude("staging status", use_json, t0)? {
        worktree::Pre::Go(c) => c,
        worktree::Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&refuse(
            "STAGING_NOT_MAIN_CHECKOUT",
            format!(
                "\"bee staging status\" must be run from inside the main checkout, not a \"{}\" checkout — staging is only ever mutated from there (staging-lane D0a).",
                ctx.kind
            ),
        )));
    }
    let main_root = ctx.work_root.clone();

    match staging_status(&main_root) {
        Ok(Some(outcome)) => {
            let mut result = Map::new();
            result.insert("ok".into(), Value::Bool(true));
            result.insert("exists".into(), Value::Bool(true));
            result.insert("branch".into(), json!(outcome.branch));
            result.insert("worktree_root".into(), json!(worktree::p(&outcome.worktree_root)));
            result.insert("base_sha".into(), json!(outcome.base_sha));
            result.insert("current_main_sha".into(), json!(outcome.current_main_sha));
            result.insert("stale_base".into(), Value::Bool(outcome.stale_base));
            result.insert(
                "staged".into(),
                Value::Array(
                    outcome
                        .staged
                        .iter()
                        .map(|e| {
                            json!({
                                "feature": e.feature,
                                "branch": e.branch,
                                "uat_gate": e.uat_gate,
                                "last_merged_sha": e.last_merged_sha,
                                "at": e.at,
                            })
                        })
                        .collect::<Vec<_>>(),
                ),
            );
            let mut lines = vec![format!("staging at {} (base_sha {}).", worktree::p(&outcome.worktree_root), outcome.base_sha)];
            if outcome.stale_base {
                lines.push(format!(
                    "  stale base: main is now at {} — run \"bee staging rebuild\".",
                    outcome.current_main_sha
                ));
            }
            if outcome.staged.is_empty() {
                lines.push("  staged: (none)".to_string());
            } else {
                for e in &outcome.staged {
                    lines.push(format!("  - {} (branch \"{}\", uat: {})", e.feature, e.branch, e.uat_gate));
                }
            }
            let text = lines.join("\n");
            Some(ctx.emit(&Value::Object(result), &text))
        }
        Ok(None) => {
            let mut result = Map::new();
            result.insert("ok".into(), Value::Bool(true));
            result.insert("exists".into(), Value::Bool(false));
            result.insert("staged".into(), Value::Array(Vec::new()));
            Some(ctx.emit(
                &Value::Object(result),
                "no staging environment yet — run \"bee staging add --feature <slug>\" to create one.",
            ))
        }
        Err(message) => Some(ctx.fail(&message)),
    }
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "staging" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // command-scoped help falls through
    }
    let (flags, use_json) = parse_flags(&toks)?;
    match verb {
        "add" => run_add(flags, use_json, t0),
        "rebuild" => run_rebuild(flags, use_json, t0),
        "status" => run_status(flags, use_json, t0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_ok(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the staging fixtures");
        assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    fn main_repo(tmp: &Path) -> PathBuf {
        let main = tmp.join("main");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::write(main.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        // Same host-shaped `.gitignore` `worktree/tests.rs`'s own `main_repo`
        // fixture uses: without it, `commit_in`'s `git add -A` inside a
        // freshly bootstrapped feature worktree would track that worktree's
        // OWN `.bee/state.json`/`worktree-identity.json` — different bytes
        // per feature — and turn every later staging merge into a spurious
        // `.bee/`-only conflict that has nothing to do with the fixture's
        // actual (intentional) `shared.txt` conflict.
        std::fs::write(main.join(".gitignore"), ".bee/*\n").unwrap();
        std::fs::write(main.join("f.txt"), "start\n").unwrap();
        git_ok(&main, &["init", "-q", "-b", "main", "."]);
        // Windows runners ship git with core.autocrlf=true; the fixtures
        // assert exact LF bytes after checkout, so pin conversion off.
        git_ok(&main, &["config", "core.autocrlf", "false"]);
        git_ok(&main, &["config", "user.email", "a@b.c"]);
        git_ok(&main, &["config", "user.name", "t"]);
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "init"]);
        main
    }

    /// A real registered feature worktree (creation identity, grant, branch
    /// `wt/<feature>`) — exercises `resolve_feature_branch`'s FIRST source,
    /// not just the `wt/<slug>` fallback.
    fn feature_worktree(main: &Path, feature: &str) -> PathBuf {
        let mut lock_busy = None;
        let created = worktree::create_feature_worktree(main, feature, None, worktree::CompanionSpec::default(), &mut lock_busy)
            .unwrap_or_else(|e| match e {
                worktree::CErr::Refuse(m) => panic!("refused creating {feature}: {m}"),
                worktree::CErr::Ex => panic!("delegated creating {feature}"),
            });
        created.worktree_root
    }

    fn commit_in(worktree_root: &Path, file: &str, content: &str, message: &str) {
        std::fs::write(worktree_root.join(file), content).unwrap();
        git_ok(worktree_root, &["add", "-A"]);
        git_ok(worktree_root, &["commit", "-qm", message]);
    }

    fn touch_command(rel: &str) -> String {
        if cfg!(windows) {
            format!("type nul > \"{rel}\"")
        } else {
            format!("touch \"{rel}\"")
        }
    }

    #[test]
    fn staging_is_created_only_from_main() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let main_head = js_trim(&worktree::run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        let feat = feature_worktree(&main, "demo");
        commit_in(&feat, "demo.txt", "hi\n", "demo work");

        // From the feature worktree: refused, zero mutation.
        let err = staging_add(&feat, "demo").unwrap_err();
        assert!(err.contains("STAGING_NOT_MAIN_CHECKOUT"), "{err}");
        assert!(!worktree::branch_exists(&main, "staging"), "no staging branch must exist yet");
        assert!(!staging_file(&main).exists(), "no staging record must exist yet");

        // From main: created, and based on main's CURRENT head — not the
        // feature branch's (which is one commit ahead by now).
        let outcome = staging_add(&main, "demo").unwrap();
        assert_eq!(outcome.branch, "wt/demo");
        let record = read_staging_record(&main).unwrap().unwrap();
        assert_eq!(record.base_sha, main_head, "staging must be cut from CURRENT main, never the feature branch (D0a)");
        assert!(record.worktree_root.exists());
    }

    #[test]
    fn add_merges_the_feature_and_records_it() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let feat = feature_worktree(&main, "demo");
        commit_in(&feat, "demo.txt", "hello\n", "demo work");

        let outcome = staging_add(&main, "demo").unwrap_or_else(|e| panic!("staging add failed: {e}"));
        assert_eq!(outcome.feature, "demo");
        assert_eq!(outcome.branch, "wt/demo");
        assert_eq!(outcome.staged.len(), 1);
        assert_eq!(outcome.staged[0].last_merged_sha, outcome.last_merged_sha);

        let record = read_staging_record(&main).unwrap().unwrap();
        assert_eq!(record.branch, "staging");
        assert_eq!(record.staged.len(), 1);
        assert_eq!(record.staged[0].feature, "demo");
        assert!(record.worktree_root.join("demo.txt").exists(), "the merged content must land in the staging worktree");
    }

    #[test]
    fn conflict_aborts_clean_and_leaves_staging_usable() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let a = feature_worktree(&main, "a");
        commit_in(&a, "shared.txt", "start\naaa\n", "a edits shared");
        let b = feature_worktree(&main, "b");
        commit_in(&b, "shared.txt", "start\nbbb\n", "b edits shared");

        staging_add(&main, "a").expect("a must merge cleanly");

        let err = staging_add(&main, "b").unwrap_err();
        assert!(err.contains("STAGING_MERGE_CONFLICT"), "{err}");
        assert!(err.contains('b'), "{err}");
        assert!(err.contains("shared.txt"), "{err}");

        // the staged set is unchanged, and staging is byte-clean.
        let record = read_staging_record(&main).unwrap().unwrap();
        assert_eq!(record.staged.len(), 1, "the failed merge must never be recorded");
        assert_eq!(record.staged[0].feature, "a");
        let status = worktree::run_git(&record.worktree_root, &["status", "--porcelain"]);
        assert_eq!(js_trim(&status.stdout.unwrap_or_default()), "", "staging must be left clean after the abort");

        // staging stays usable: a later, non-conflicting feature still merges.
        let c = feature_worktree(&main, "c");
        commit_in(&c, "c.txt", "hi\n", "c adds a file");
        let outcome = staging_add(&main, "c").expect("staging must stay usable after an aborted conflict");
        assert_eq!(outcome.staged.len(), 2);
    }

    #[test]
    fn readd_after_a_new_commit_updates_last_merged_sha() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let feat = feature_worktree(&main, "demo");
        commit_in(&feat, "demo.txt", "v1\n", "v1");

        let first = staging_add(&main, "demo").unwrap();

        commit_in(&feat, "demo2.txt", "v2\n", "v2");
        let second = staging_add(&main, "demo").unwrap();
        assert_ne!(second.last_merged_sha, first.last_merged_sha, "re-add must pick up the new commit");
        assert_eq!(second.staged.len(), 1, "re-add upserts, never duplicates");

        let record = read_staging_record(&main).unwrap().unwrap();
        assert!(record.worktree_root.join("demo2.txt").exists());
    }

    #[test]
    fn build_hook_runs_when_configured() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let feat = feature_worktree(&main, "demo");
        commit_in(&feat, "demo.txt", "hi\n", "demo work");

        let marker_rel = "build-marker.txt";
        std::fs::write(
            main.join(".bee").join("config.json"),
            jsjson::stringify(&json!({"commands": {"staging_build": touch_command(marker_rel)}})),
        )
        .unwrap();

        let outcome = staging_add(&main, "demo").unwrap();
        assert!(outcome.build_note.contains("staging_build"), "{}", outcome.build_note);
        assert!(
            outcome.staging_worktree_root.join(marker_rel).exists(),
            "commands.staging_build must run IN the staging worktree"
        );
    }

    #[test]
    fn build_hook_skips_visibly_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let feat = feature_worktree(&main, "demo");
        commit_in(&feat, "demo.txt", "hi\n", "demo work");

        let outcome = staging_add(&main, "demo").unwrap();
        assert!(
            outcome.build_note.to_lowercase().contains("skip"),
            "an absent build hook must be a VISIBLE skip note, never silence: {}",
            outcome.build_note
        );
    }

    #[test]
    fn build_failure_is_typed_but_the_recorded_merge_stands() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let feat = feature_worktree(&main, "demo");
        commit_in(&feat, "demo.txt", "hi\n", "demo work");

        let fail_cmd = if cfg!(windows) { "exit /b 1" } else { "exit 1" };
        std::fs::write(
            main.join(".bee").join("config.json"),
            jsjson::stringify(&json!({"commands": {"staging_build": fail_cmd}})),
        )
        .unwrap();

        let err = staging_add(&main, "demo").unwrap_err();
        assert!(err.contains("STAGING_BUILD_FAILED"), "{err}");

        let record = read_staging_record(&main).unwrap().unwrap();
        assert_eq!(record.staged.len(), 1, "the merge already landed — a red build must not un-record it");
    }

    // ─── rebuild / status (sl-2, D0a trigger 3) ────────────────────────────

    /// A minimal, directly-written workflow record naming `feature` with its
    /// uat gate at `uat_state` — the same shape `read_workflow_record`
    /// expects (`id` == the directory name, `feature`, `gates.uat.state`).
    /// Written straight to disk (never through `bee state gate`): sl-2 only
    /// READS this store, so a hand-built fixture exercises exactly that read
    /// path without dragging in the whole gate-mutation machinery.
    fn write_workflow_gate(main: &Path, feature: &str, uat_state: &str) {
        let dir = main.join(".bee").join("runtime").join("workflows").join(feature);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("state.json"),
            jsjson::stringify(&json!({
                "id": feature,
                "feature": feature,
                "gates": {"uat": {"state": uat_state}},
            })),
        )
        .unwrap();
    }

    #[test]
    fn rebuild_refuses_when_no_staging_record_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());

        let err = staging_rebuild(&main, &[]).unwrap_err();
        assert!(err.contains("STAGING_NO_RECORD"), "{err}");
        assert!(!worktree::branch_exists(&main, "staging"), "rebuild with no record must create nothing");
    }

    #[test]
    fn rebuild_resets_to_moved_main_and_remerges_pending_features() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let feat = feature_worktree(&main, "demo");
        commit_in(&feat, "demo.txt", "hi\n", "demo work");

        staging_add(&main, "demo").expect("initial add must merge cleanly");
        let old_base = read_staging_record(&main).unwrap().unwrap().base_sha;

        // main moves AFTER staging was cut.
        commit_in(&main, "main2.txt", "later\n", "main moves on");
        let new_main_head = js_trim(&worktree::run_git(&main, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
        assert_ne!(old_base, new_main_head);

        let outcome = staging_rebuild(&main, &[]).expect("rebuild must succeed");
        assert_eq!(outcome.base_sha, new_main_head, "rebuild must reset to CURRENT main, not the old base");
        assert_eq!(outcome.merged, vec!["demo".to_string()], "the pending-gate feature must be re-merged");
        assert!(outcome.dropped.is_empty());
        assert!(outcome.conflicts.is_empty());
        assert!(outcome.staging_worktree_root.join("main2.txt").exists(), "the reset must pick up main's new commit");
        assert!(outcome.staging_worktree_root.join("demo.txt").exists(), "the pending feature must be re-merged in");

        let record = read_staging_record(&main).unwrap().unwrap();
        assert_eq!(record.base_sha, new_main_head);
        assert_eq!(record.staged.len(), 1);
        assert_eq!(record.staged[0].feature, "demo");
    }

    #[test]
    fn uat_approved_feature_auto_drops_on_rebuild() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let feat = feature_worktree(&main, "demo");
        commit_in(&feat, "demo.txt", "hi\n", "demo work");
        staging_add(&main, "demo").expect("initial add must merge cleanly");

        write_workflow_gate(&main, "demo", "approved");

        let outcome = staging_rebuild(&main, &[]).expect("rebuild must succeed");
        assert_eq!(outcome.dropped, vec!["demo".to_string()], "an approved-uat feature must auto-drop");
        assert!(outcome.merged.is_empty());
        assert!(
            !outcome.staging_worktree_root.join("demo.txt").exists(),
            "a dropped feature must not be re-merged after the reset"
        );

        let record = read_staging_record(&main).unwrap().unwrap();
        assert!(record.staged.is_empty(), "the invariant re-derives: staging = main + Σ features awaiting UAT");
    }

    #[test]
    fn without_flag_skips_a_merge_but_keeps_membership() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let a = feature_worktree(&main, "a");
        commit_in(&a, "a.txt", "hi\n", "a work");
        let b = feature_worktree(&main, "b");
        commit_in(&b, "b.txt", "hi\n", "b work");
        staging_add(&main, "a").expect("a must merge cleanly");
        staging_add(&main, "b").expect("b must merge cleanly");

        let outcome = staging_rebuild(&main, &["b".to_string()]).expect("rebuild must succeed");
        assert_eq!(outcome.merged, vec!["a".to_string()]);
        assert_eq!(outcome.without, vec!["b".to_string()]);
        assert!(outcome.dropped.is_empty());
        assert!(outcome.staging_worktree_root.join("a.txt").exists(), "a was re-merged");
        assert!(
            !outcome.staging_worktree_root.join("b.txt").exists(),
            "b was excluded via --without — the reset dropped it and it was never re-merged"
        );

        let record = read_staging_record(&main).unwrap().unwrap();
        assert_eq!(record.staged.len(), 2, "--without excludes the MERGE, never the membership");
        assert!(record.staged.iter().any(|e| e.feature == "b"), "b stays staged even though it was skipped");
    }

    #[test]
    fn one_conflicting_feature_never_blocks_the_others_on_rebuild() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        std::fs::write(main.join("shared.txt"), "start\n").unwrap();
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "add shared.txt"]);

        let a = feature_worktree(&main, "a");
        commit_in(&a, "a.txt", "hi\n", "a work");
        let b = feature_worktree(&main, "b");
        commit_in(&b, "shared.txt", "start\nbbb\n", "b edits shared");

        staging_add(&main, "a").expect("a must merge cleanly");
        staging_add(&main, "b").expect("b must merge cleanly (staging's shared.txt is still untouched)");

        // main moves in a way that conflicts with b's branch only.
        commit_in(&main, "shared.txt", "start\nmmm\n", "main edits shared");

        let outcome = staging_rebuild(&main, &[]).expect("rebuild must succeed overall despite one conflict");
        assert_eq!(outcome.merged, vec!["a".to_string()], "the non-conflicting feature must still merge");
        assert_eq!(outcome.conflicts.len(), 1, "{:?}", outcome.conflicts);
        assert!(outcome.conflicts[0].contains("STAGING_MERGE_CONFLICT"), "{}", outcome.conflicts[0]);
        assert!(outcome.conflicts[0].contains('b'), "{}", outcome.conflicts[0]);
        assert!(outcome.conflicts[0].contains("shared.txt"), "{}", outcome.conflicts[0]);
        assert!(outcome.dropped.is_empty());

        assert!(outcome.staging_worktree_root.join("a.txt").exists());
        assert_eq!(
            std::fs::read_to_string(outcome.staging_worktree_root.join("shared.txt")).unwrap(),
            "start\nmmm\n",
            "b's conflicting change must never land — main's own version stands"
        );
        let status = worktree::run_git(&outcome.staging_worktree_root, &["status", "--porcelain"]);
        assert_eq!(js_trim(&status.stdout.unwrap_or_default()), "", "staging must be left clean after the abort");

        let record = read_staging_record(&main).unwrap().unwrap();
        assert_eq!(record.staged.len(), 2, "b stays staged at its last successful merge — the conflict never drops it");
    }

    #[test]
    fn status_flags_a_stale_base_and_reports_uat_gate_state() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let feat = feature_worktree(&main, "demo");
        commit_in(&feat, "demo.txt", "hi\n", "demo work");
        staging_add(&main, "demo").expect("initial add must merge cleanly");

        let fresh = staging_status(&main).unwrap().unwrap();
        assert!(!fresh.stale_base, "base_sha must equal current main right after add");
        assert_eq!(fresh.staged.len(), 1);
        assert_eq!(fresh.staged[0].uat_gate, "unknown", "no workflow record exists yet for \"demo\"");

        write_workflow_gate(&main, "demo", "approved");
        let approved = staging_status(&main).unwrap().unwrap();
        assert_eq!(approved.staged[0].uat_gate, "approved");

        commit_in(&main, "main2.txt", "later\n", "main moves on");
        let stale = staging_status(&main).unwrap().unwrap();
        assert!(stale.stale_base, "main moved after staging was cut — status must flag it (trigger-3 reminder)");
        assert_ne!(stale.current_main_sha, stale.base_sha);
    }

    #[test]
    fn status_over_an_absent_staging_record_is_a_plain_none_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        assert!(staging_status(&main).unwrap().is_none());
    }
}
