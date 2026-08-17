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
    let staged = staging_add_locked(main_root, feature);
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
}
