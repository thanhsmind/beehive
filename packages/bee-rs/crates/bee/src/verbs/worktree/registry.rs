// the emission frame, the grant registry, and list/unregister/register
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
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::{Command, ExitCode};
use std::time::Instant;

// ─── emission frame (bee.mjs emit/emitError + the direct-run timing) ───────
// Same shape as verbs/decisions.rs's, with one difference that is the whole
// point of this module: the prelude keeps the FULL root resolution, because
// every verb here has to know whether it is standing inside a worktree.

pub(crate) struct Ctx {
    /// main()'s `root`: resolveRoots(cwd).storeRoot.
    pub(crate) root: PathBuf,
    /// resolveRoots(cwd).worktreeResolution — "ordinary" | "linked-valid".
    pub(crate) kind: &'static str,
    /// The git-verified worktree id (linked-valid only).
    pub(crate) id: Option<String>,
    /// resolveRoots(cwd).mainRoot (linked-valid only).
    pub(crate) main_root: Option<PathBuf>,
    /// resolveRoots(cwd).workRoot — the physical checkout.
    pub(crate) work_root: PathBuf,
    pub(crate) cmd: &'static str,
    pub(crate) use_json: bool,
    pub(crate) t0: Instant,
    pub(crate) drift_changed: bool,
    pub(crate) drift_hint: &'static str,
}

pub(crate) enum Pre {
    Go(Box<Ctx>),
    Emitted(ExitCode),
}

pub(crate) fn prelude(cmd: &'static str, use_json: bool, t0: Instant) -> Option<Pre> {
    let cwd = std::env::current_dir().ok()?;
    let (root, kind, id, main_root, work_root) = match resolve_roots_core(&cwd) {
        Resolution::Ordinary {
            store_root,
            work_root,
        } => (store_root, "ordinary", None, None, work_root),
        Resolution::LinkedValid {
            store_root,
            work_root,
            id,
            main_root,
        } => (store_root, "linked-valid", Some(id), Some(main_root), work_root),
        // A BROKEN link. Node's main() catches its own findRepoRoot throw and
        // emits the message like any other refusal; only the direct-run timing
        // wrapper's SECOND findRepoRoot call (which also throws, skipping the
        // timings.jsonl append) made this unreproducible through the shared
        // wrapper. crate::link_invalid owns that exact shape now — see its
        // header for the one named timing-line divergence.
        Resolution::LinkInvalid { message } => {
            return Some(Pre::Emitted(crate::link_invalid::emit_link_invalid(
                &message, cmd, use_json, t0,
            )))
        }
        Resolution::Unresolved => {
            return Some(Pre::Emitted(emit_no_root_error(&cwd, cmd, use_json, t0)))
        }
    };
    let drift = check_manifest_drift(&root);
    Some(Pre::Go(Box::new(Ctx {
        root,
        kind,
        id,
        main_root,
        work_root,
        cmd,
        use_json,
        t0,
        drift_changed: drift.manifest_changed,
        drift_hint: drift.hint,
    })))
}

impl Ctx {
    /// bee.mjs emit(): drift line (stderr) + result (stdout) + timing.
    pub(crate) fn emit(&self, result: &Value, text: &str) -> ExitCode {
        self.emit_code(result, text, 0)
    }

    /// emit() with a handler-supplied `exitCode` (only `worktree merge` uses a
    /// non-zero one). main()'s `recordTiming(!code)` means a non-zero exit
    /// logs `ok: false` even though the result still went to stdout.
    pub(crate) fn emit_code(&self, result: &Value, text: &str, exit: u8) -> ExitCode {
        if self.drift_changed {
            eprintln!("manifest_changed: true — {}", self.drift_hint);
        }
        if self.use_json {
            println!("{}", jsjson::stringify_pretty(result));
        } else {
            println!("{text}");
        }
        record_timing(&self.root, self.cmd, self.t0, exit == 0);
        if exit == 0 {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(exit)
        }
    }

    /// bee.mjs emitError(): no drift line, {"error"} or stderr, exit 1.
    pub(crate) fn fail(&self, message: &str) -> ExitCode {
        if self.use_json {
            println!("{}", jsjson::stringify(&json!({ "error": message })));
        } else {
            eprintln!("{message}");
        }
        record_timing(&self.root, self.cmd, self.t0, false);
        ExitCode::FAILURE
    }

    /// bee.mjs resolveMainRoot(root): a linked worktree's own `mainRoot`
    /// regardless of grant state; otherwise the already-resolved storeRoot.
    pub(crate) fn main_root(&self) -> PathBuf {
        self.main_root.clone().unwrap_or_else(|| self.root.clone())
    }
}

/// Node prints paths through plain string interpolation of what
/// path.resolve/path.join produced — always valid UTF-8 here because the
/// resolution walked from an existing cwd.
pub(crate) fn p(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

// ─── worktree-store.mjs grant registry ────────────────────────────────────

pub(crate) fn grants_file(main_store_root: &Path) -> PathBuf {
    main_store_root.join("runtime").join("worktree-grants.json")
}

/// A file that exists but does not parse is `None` (delegate) — a broader
/// fallback would cover "parses as JSON but not as an object" too readily.
/// Missing/unreadable file, or a parsed non-object, is `{}`. An ARRAY
/// registry (`typeof [] === 'object'` in JS) also delegates rather than
/// model Object.keys over array indices.
pub(crate) fn read_grants_strict(main_store_root: &Path) -> Option<Map<String, Value>> {
    let file = grants_file(main_store_root);
    let Ok(bytes) = std::fs::read(&file) else {
        return Some(Map::new());
    };
    let parsed: Value = serde_json::from_slice(&bytes).ok()?;
    match js_numberify(&parsed).ok()? {
        Value::Object(m) => Some(m),
        Value::Array(_) => None,
        _ => Some(Map::new()), // JSON.parse gave a non-object -> {}
    }
}

/// writeGrantsFileAtomic: mkdir -p the runtime dir, write
/// `JSON.stringify(grants, null, 2) + "\n"` to `<file>.tmp` (Node's fixed tmp
/// name, not fsutil's unique one — this module has its own writer), rename.
pub(crate) fn write_grants_file_atomic(main_store_root: &Path, grants: &Map<String, Value>) -> std::io::Result<()> {
    let file = grants_file(main_store_root);
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = {
        let mut name = file.clone().into_os_string();
        name.push(".tmp");
        PathBuf::from(name)
    };
    let body = format!("{}\n", jsjson::stringify_pretty(&Value::Object(grants.clone())));
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, &file)
}

// ─── worktree list ────────────────────────────────────────────────────────

pub(crate) fn run_list(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match prelude("worktree list", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let main_root = ctx.main_root();
    let main_store_root = main_root.join(".bee");
    let grants = read_grants_strict(&main_store_root)?;

    let ids: Vec<&String> = grants
        .iter()
        .filter(|(_, v)| **v == Value::Bool(true))
        .map(|(k, _)| k)
        .collect();
    let text = if ids.is_empty() {
        "No worktree grants.".to_string()
    } else {
        ids.iter()
            .map(|id| format!("{id} (granted)"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut result = Map::new();
    result.insert("grants".into(), Value::Object(grants.clone()));
    result.insert("main_root".into(), json!(p(&main_root)));
    Some(ctx.emit(&Value::Object(result), &text))
}

// ─── worktree unregister ──────────────────────────────────────────────────

pub(crate) fn run_unregister(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["id"]) {
        return None;
    }
    // `flags.id ? String(flags.id) : null` — an empty value is falsy in JS.
    let id_flag: Option<String> = match flags.get("id") {
        Some(FlagV::S(s)) if !s.is_empty() => Some(s.clone()),
        Some(FlagV::S(_)) | None => None,
        Some(FlagV::Present) => return None, // not a flag-alone boolean
    };

    let ctx = match prelude("worktree unregister", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    let main_root = ctx.main_root();
    let main_store_root = main_root.join(".bee");

    let id = match id_flag {
        Some(id) => id,
        None => match (&ctx.kind, &ctx.id) {
            (&"linked-valid", Some(id)) => id.clone(),
            // The resolveRoots-threw branch of this message is unreachable
            // from the CLI (main() already threw); the plain one is not.
            _ => {
                return Some(ctx.fail(
                    "--id not given, and the current directory is not a linked worktree — pass --id explicitly.",
                ))
            }
        },
    };

    // Pre-checked before the lock: an unparseable registry delegates.
    read_grants_strict(&main_store_root)?;

    let mut guard = match lock::acquire_store_lock(&main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS) {
        Ok(g) => g,
        Err(busy) => return Some(ctx.fail(&busy.message())),
    };
    // The registry half of the shared teardown helper (D3, D3a): grant,
    // workspace record, holds release — never the directory. `remove: None`
    // is the whole reason `unregister` cannot self-delete a worktree: the
    // directory-removal parameter is `perform_cleanup`'s alone to pass.
    let _ = teardown_worktree(&main_root, &id, None);
    guard.release();

    let mut result = Map::new();
    result.insert("ok".into(), Value::Bool(true));
    result.insert("id".into(), json!(id));
    result.insert("main_root".into(), json!(p(&main_root)));
    Some(ctx.emit(&Value::Object(result), &format!("Removed worktree grant for id {id}.")))
}

// ─── worktree register ────────────────────────────────────────────────────

pub(crate) fn run_register(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["feature"]) {
        return None;
    }
    // requireFlag(flags, 'feature') — undefined/''/true all throw; the
    // dispatcher's validate() already covers "missing" (required), so the
    // remaining throw shapes go back to Node.
    let feature = match flags.get("feature") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };

    let ctx = match prelude("worktree register", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    // review B-P2-2: the same shape create.rs's `feature_slug_ok` enforces —
    // `worktree register` never routes through `create_feature_worktree`'s
    // own gate (only `worktree new` does), so an unvalidated `--feature`
    // value would otherwise reach `bootstrap_worktree_store`'s feature-keyed
    // path joins (`archive/<feature>`) unchecked. Checked here, before
    // `main_root`/`worktree_root` are even read off `ctx`, let alone joined
    // with `feature` — a `../../etc` or an absolute value is refused by NAME
    // rather than ever reaching a join.
    if let Some(message) = register_feature_refusal(&feature) {
        return Some(ctx.fail(&message));
    }
    let (id, main_root) = match (&ctx.kind, &ctx.id, &ctx.main_root) {
        (&"linked-valid", Some(id), Some(main_root)) => (id.clone(), main_root.clone()),
        _ => {
            return Some(ctx.fail(&format!(
                "\"bee worktree register\" must be run from inside a linked git worktree (git worktree add), not an \"{}\" checkout.",
                ctx.kind
            )))
        }
    };
    let worktree_root = ctx.work_root.clone();
    let main_store_root = main_root.join(".bee");

    // Pre-checked before the lock: an unparseable registry delegates.
    let existing = read_grants_strict(&main_store_root)?;

    // writeGrant: `{ ...readGrants(main), [id]: true }` under the lock.
    let mut guard = match lock::acquire_store_lock(&main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS) {
        Ok(g) => g,
        Err(busy) => return Some(ctx.fail(&busy.message())),
    };
    let mut next = existing;
    next.insert(id.clone(), Value::Bool(true)); // JS spread: existing key keeps its position
    let write_err = write_grants_file_atomic(&main_store_root, &next).err();
    guard.release();
    if write_err.is_some() {
        return None; // V8-worded fs throw in Node
    }

    let bootstrap = bootstrap_worktree_store(&worktree_root, &main_store_root, &feature)?;
    let created = bootstrap.get("created") == Some(&Value::Bool(true));
    let store_root_str = bootstrap
        .get("worktreeStoreRoot")
        .map(jsjson::js_to_string)
        .unwrap_or_default();

    let mut result = Map::new();
    result.insert("ok".into(), Value::Bool(true));
    result.insert("id".into(), json!(id));
    result.insert("feature".into(), json!(feature));
    result.insert("main_root".into(), json!(p(&main_root)));
    result.insert("worktree_root".into(), json!(p(&worktree_root)));
    result.insert("bootstrap".into(), Value::Object(bootstrap.clone()));

    let last = if created {
        format!("  bootstrapped {store_root_str} (phase idle, gates unapproved).")
    } else {
        format!(
            "  worktree .bee/state.json already existed — left untouched ({}).",
            bootstrap.get("reason").map(jsjson::js_to_string).unwrap_or_default()
        )
    };
    let mut lines = vec![
        format!("Registered worktree grant: id {id} (feature \"{feature}\")."),
        format!("  worktree:    {}", p(&worktree_root)),
        format!("  main store:  {}", p(&main_store_root)),
        last,
    ];
    // review B-P1-1: name the symlinked path and the reason right in the
    // CLI text, not just the JSON report.
    if let Some(sync) = bootstrap.get("cellsSync") {
        let path = sync.get("path").map(jsjson::js_to_string).unwrap_or_default();
        let reason = sync.get("reason").map(jsjson::js_to_string).unwrap_or_default();
        lines.push(format!("  cells sync skipped — {path}: {reason}"));
    }
    // review D-P3-2: name the count right in the CLI text, not just the
    // JSON report — `bootstrap`'s own `pruned` key is already omitted when
    // nothing was removed, so its mere presence here is the whole gate.
    if let Some(pruned) = bootstrap.get("pruned").and_then(Value::as_array) {
        lines.push(format!(
            "  pruned {} foreign cell file{} from the island.",
            pruned.len(),
            if pruned.len() == 1 { "" } else { "s" }
        ));
    }
    let text = lines.join("\n");
    Some(ctx.emit(&Value::Object(result), &text))
}

/// review B-P2-2: `feature_slug_ok`'s refusal, in `run_register`'s own
/// words — `create.rs`'s `refuse("WORKTREE_INVALID_SLUG", …)` carries
/// `worktree new`'s bracketed `[CODE] message` convention (bee.mjs's
/// thrown-error shape); `run_register`'s refusals are plain `ctx.fail` text,
/// so this mirrors the sentence without the code prefix. `None` means the
/// slug is fine.
pub(crate) fn register_feature_refusal(feature: &str) -> Option<String> {
    if feature_slug_ok(feature) {
        return None;
    }
    Some(format!(
        "feature slug {} must match /^[a-z0-9][a-z0-9-]*$/ (lowercase letters/digits, starting with a letter or digit, hyphens allowed after that).",
        jsjson::stringify(&Value::String(feature.to_string()))
    ))
}

/// worktree-store.mjs writeCreationIdentity — the worktree's IMMUTABLE
/// creation slug, write-if-absent, never fatal.
pub(crate) fn write_creation_identity(worktree_store_root: &Path, feature: &str) -> Map<String, Value> {
    let mut out = Map::new();
    let file = worktree_store_root
        .join("runtime")
        .join("worktree-identity.json");
    if feature.is_empty() {
        out.insert("written".into(), Value::Bool(false));
        out.insert("reason".into(), json!("no feature slug given"));
        return out;
    }
    if file.exists() {
        out.insert("written".into(), Value::Bool(false));
        out.insert(
            "reason".into(),
            json!("creation identity already recorded — never overwritten"),
        );
        return out;
    }
    let body = format!(
        "{}\n",
        jsjson::stringify_pretty(&json!({ "feature": feature, "created_at": now_iso() }))
    );
    let tmp = {
        let mut name = file.clone().into_os_string();
        name.push(".tmp");
        PathBuf::from(name)
    };
    let attempt = file
        .parent()
        .map(std::fs::create_dir_all)
        .unwrap_or(Ok(()))
        .and_then(|()| std::fs::write(&tmp, body))
        .and_then(|()| std::fs::rename(&tmp, &file));
    match attempt {
        Ok(()) => {
            out.insert("written".into(), Value::Bool(true));
            out.insert("feature".into(), json!(feature));
            out.insert("file".into(), json!(p(&file)));
        }
        Err(e) => {
            out.insert("written".into(), Value::Bool(false));
            // `error instanceof Error ? error.message : String(error)` — an
            // io error message is V8-worded, so this shape delegates instead.
            let _ = e;
            out.insert("reason".into(), Value::Null);
        }
    }
    out
}

/// Cell store layout, restated locally rather than reached-for from
/// `verbs/cells/` or `verbs/status_full/cells.rs` (both private submodules
/// outside their own parent) — same duplication pattern those two already
/// use between each other.
const CELLS_DIR_NAME: &str = "cells";
const CELLS_ARCHIVE_DIR_NAME: &str = "archive";

/// `true` only when `path` parses as a JSON object whose `"feature"` is the
/// string `feature` (JS strict-equality shape: a missing/non-string/mismatched
/// field, or an unreadable/unparsable file, is "not this feature" — never a
/// silent keep).
fn cell_file_feature_matches(path: &Path, feature: &str) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let Ok(parsed) = serde_json::from_slice::<Value>(&bytes) else {
        return false;
    };
    matches!(parsed.get("feature"), Some(Value::String(f)) if f == feature)
}

/// ips-1 P1: the tracked set behind the prune arm's safety check. `git
/// worktree add` legitimately checks out every tracked file under
/// `.bee/cells` — that directory is git-tracked — so a foreign-feature file
/// already sitting in a fresh island is not a stray, it's main's committed
/// history riding along. Deleting a TRACKED file manufactures a deletion
/// that a later `worktree merge` would replay onto main and wipe the cell
/// archive; only an UNTRACKED stray (never committed, e.g. a main-store cell
/// written after this checkout was cut) is safe to remove.
///
/// One invocation covers both `cells/*.json` and `cells/archive/**` — the
/// pathspec is a directory. `None` means git is unavailable or `worktree_root`
/// is not (yet) a git repo; the caller's response is fail-safe: prune
/// nothing. Paths come back forward-slash-relative to `.bee/cells/` (git
/// always emits `/`, regardless of OS), matching the manual `/`-joined keys
/// built at the call sites below.
fn git_tracked_cells(worktree_root: &Path) -> Option<HashSet<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_root)
        .args(["ls-files", "-z", "--", ".bee/cells"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_git_ls_files_cells_output(&output.stdout)
}

/// review D-P2-1: the pure parse half of `git_tracked_cells`, split out so
/// its fail-CLOSED shape is directly testable without a real `git` process.
/// `None` on ANY entry that does not itself `strip_prefix(".bee/cells/")`
/// cleanly — an unexpected shape used to fall through as "skip just this
/// line", which left the tracked SET silently under-populated; a
/// under-populated set is indistinguishable from "git tracks nothing here",
/// and an empty set is exactly the shape that makes the prune arm above
/// delete everything. Refusing the whole lookup keeps the caller's existing
/// `None` == "prune nothing" contract the only way an unexpected shape can
/// resolve.
pub(crate) fn parse_git_ls_files_cells_output(stdout: &[u8]) -> Option<HashSet<String>> {
    let mut set = HashSet::new();
    for raw in stdout.split(|&b| b == 0) {
        if raw.is_empty() {
            continue;
        }
        let rel = String::from_utf8_lossy(raw);
        match rel.strip_prefix(".bee/cells/") {
            Some(stripped) => {
                set.insert(stripped.to_string());
            }
            None => return None,
        }
    }
    Some(set)
}

/// PBI p-9c48a67c + ips-1: restrict the worktree island's `.bee/cells` to the
/// granted feature's cells only. `git worktree add` checks out `.bee/cells`
/// in FULL — it is git-tracked — so a freshly created worktree can already
/// hold every OTHER feature's open cells before this function ever runs;
/// pruning has to run unconditionally, not just "copy whatever's missing".
/// Two passes, main store read-only throughout:
///   1. PRUNE — drop any UNTRACKED foreign-feature cell file (or the
///      untracked leftovers of an `archive/<feature>` dir) already sitting
///      in the worktree store. A TRACKED foreign-feature file stays on disk
///      untouched (see `git_tracked_cells`); git unavailable fails safe —
///      prune nothing.
///   2. FILL — copy in the main store's matching-feature cells that are not
///      already present (a from-scratch `worktree register` adopting a bare
///      checkout, or a main-store cell not yet committed to git).
/// Feature-neutral content under `.bee` (config, expertise, prompts,
/// decisions/backlog, `cells/archive/summary.json`, …) is untouched — this
/// function only ever looks inside `cells/`.
/// review B-P1-1 / D23: whether the whole cell sync ran, or was refused
/// because one of the fixed store paths it is about to prune or fill
/// through is a SYMLINK.
enum CellsSync {
    /// review D-P3-2: `pruned` names every file the prune arm actually
    /// removed, `.bee/cells/`-relative (a bare `name.json` for a top-level
    /// removal, `archive/<feature>/name.json` for an archive one) — empty on
    /// the common case where nothing foreign was sitting in the island.
    Ran { pruned: Vec<String> },
    /// `path` is the symlink that tripped the refusal; `reason` is the
    /// one-line explanation the bootstrap report carries verbatim.
    Skipped { path: PathBuf, reason: String },
}

/// `true` only for a path that itself resolves to a symlink — never follows
/// it. A missing/unreadable path is `false` (delegate to the caller's own
/// existence handling), matching `std::fs::symlink_metadata`'s own contract.
fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn sync_worktree_cells(worktree_store_root: &Path, main_store_root: &Path, feature: &str) -> Option<CellsSync> {
    let dest_cells = worktree_store_root.join(CELLS_DIR_NAME);
    let src_cells = main_store_root.join(CELLS_DIR_NAME);
    let dest_archive = dest_cells.join(CELLS_ARCHIVE_DIR_NAME);
    let src_feature_archive = src_cells.join(CELLS_ARCHIVE_DIR_NAME).join(feature);
    // review B-P2-7 / D-P3-1: the granted feature's OWN archive subdir — the
    // exact path `fs::copy` writes into at the bottom of this function.
    // `dest_archive` (its parent, `cells/archive`) being a real directory
    // said nothing about THIS child: a symlinked `archive/<feature>` let
    // `create_dir_all` no-op on the existing link and `fs::copy` land
    // straight in the link's target, outside the store entirely.
    let dest_feature_archive = dest_archive.join(feature);

    // review B-P1-1 / D23 never-follow: a repo committing `.bee` or
    // `.bee/cells` (or either store's archive dir) as a SYMLINK defeats the
    // tracked-set shield below — git tracks the symlink OBJECT, not paths
    // under it, so `git ls-files` comes back empty for that prefix and the
    // prune arm would delete every `*.json` sitting in the symlink's TARGET,
    // which is outside the store entirely. Checked before any prune or
    // fill, against every fixed path this function is about to `read_dir`
    // or `create_dir_all` through — never followed, never deleted through.
    // A per-entry foreign-feature archive subdir stays covered by
    // `DirEntry::file_type`, which already never follows a symlink either.
    for suspect in [
        worktree_store_root,
        &dest_cells,
        &src_cells,
        &dest_archive,
        &dest_feature_archive,
        &src_feature_archive,
    ] {
        if is_symlink(suspect) {
            return Some(CellsSync::Skipped {
                path: suspect.to_path_buf(),
                reason: "refusing to sync .bee/cells through a symlinked path".to_string(),
            });
        }
    }

    std::fs::create_dir_all(&dest_cells).ok()?;

    let tracked = worktree_store_root.parent().and_then(git_tracked_cells);
    // review D-P3-2: every file this pass actually removes, `.bee/cells/`-
    // relative — carried up into the bootstrap report so `register`'s CLI
    // text can name the count rather than pruning silently.
    let mut pruned: Vec<String> = Vec::new();

    // (1) prune top-level cell files that are not the granted feature's,
    // and are not tracked — a tracked foreign-feature file is left alone.
    if let Some(tracked) = &tracked {
        if let Ok(entries) = std::fs::read_dir(&dest_cells) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue; // archive/ handled in pass (3) below
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                if !name.ends_with(".json") {
                    continue;
                }
                let path = entry.path();
                if cell_file_feature_matches(&path, feature) {
                    continue;
                }
                if tracked.contains(&name) {
                    continue; // tracked foreign cell — main's history, stays
                }
                std::fs::remove_file(&path).ok()?;
                pruned.push(name);
            }
        }
    }

    // (2) fill in the main store's matching cells not already present.
    if let Ok(entries) = std::fs::read_dir(&src_cells) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let src_path = entry.path();
            if !cell_file_feature_matches(&src_path, feature) {
                continue;
            }
            let dest_path = dest_cells.join(&name);
            if dest_path.exists() {
                continue;
            }
            std::fs::copy(&src_path, &dest_path).ok()?;
        }
    }

    // (3) archive: already partitioned by feature-name subdirectory — prune
    // every OTHER feature's subdir down to its tracked files, then fill in
    // the granted feature's. A tracked file inside a foreign dir stays; only
    // when nothing tracked is left does the now-empty dir itself go.
    if let Some(tracked) = &tracked {
        if let Ok(entries) = std::fs::read_dir(&dest_archive) {
            for entry in entries.filter_map(|e| e.ok()) {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue; // e.g. archive/summary.json is feature-neutral
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                if name == feature {
                    continue;
                }
                let dir_path = entry.path();
                if let Ok(inner) = std::fs::read_dir(&dir_path) {
                    for file in inner.filter_map(|e| e.ok()) {
                        if file.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            continue; // no nested dirs in a feature's archive
                        }
                        let file_name = file.file_name().to_string_lossy().into_owned();
                        let rel = format!("{CELLS_ARCHIVE_DIR_NAME}/{name}/{file_name}");
                        if tracked.contains(&rel) {
                            continue; // tracked foreign archive file — stays
                        }
                        std::fs::remove_file(file.path()).ok()?;
                        pruned.push(rel);
                    }
                }
                let now_empty = std::fs::read_dir(&dir_path)
                    .map(|mut it| it.next().is_none())
                    .unwrap_or(false);
                if now_empty {
                    std::fs::remove_dir(&dir_path).ok()?;
                }
            }
        }
    }
    if src_feature_archive.exists() {
        std::fs::create_dir_all(&dest_feature_archive).ok()?;
        if let Ok(entries) = std::fs::read_dir(&src_feature_archive) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                let dest_path = dest_feature_archive.join(&name);
                if dest_path.exists() {
                    continue;
                }
                std::fs::copy(entry.path(), &dest_path).ok()?;
            }
        }
    }

    Some(CellsSync::Ran { pruned })
}

/// worktree-store.mjs bootstrapWorktreeStore. `None` = an fs failure Node
/// would surface with a V8 message (delegate; every step so far is
/// idempotent, see the module header).
pub(crate) fn bootstrap_worktree_store(
    worktree_root: &Path,
    main_store_root: &Path,
    feature: &str,
) -> Option<Map<String, Value>> {
    let worktree_store_root = worktree_root.join(".bee");
    std::fs::create_dir_all(&worktree_store_root).ok()?;

    let copy_if_absent = |name: &str| -> Option<Map<String, Value>> {
        let mut m = Map::new();
        let dest = worktree_store_root.join(name);
        if dest.exists() {
            m.insert("copied".into(), Value::Bool(false));
            m.insert("reason".into(), json!(format!("{name} already exists")));
            return Some(m);
        }
        let src = main_store_root.join(name);
        if !src.exists() {
            m.insert("copied".into(), Value::Bool(false));
            m.insert("reason".into(), json!(format!("main store has no {name}")));
            return Some(m);
        }
        std::fs::copy(&src, &dest).ok()?;
        m.insert("copied".into(), Value::Bool(true));
        Some(m)
    };

    let onboarding = copy_if_absent("onboarding.json")?;
    let config = copy_if_absent("config.json")?;
    let identity = write_creation_identity(&worktree_store_root, feature);
    if identity.get("reason") == Some(&Value::Null) {
        return None; // V8-worded fs error in the identity write
    }

    // p-9c48a67c: prune/fill `.bee/cells` down to the granted feature — runs
    // every bootstrap (not just a fresh one), so a re-run `register` on an
    // already-adopted worktree also loses any foreign cell that leaked in.
    // An empty `feature` (write_creation_identity's own "no feature slug
    // given" case) skips this rather than pruning everything as "no match".
    let mut out = Map::new();
    if !feature.is_empty() {
        // review B-P1-1: a symlinked store path skips the WHOLE cell sync —
        // never a partial prune/fill — and the refusal rides the report
        // (both the map and, via `run_register`'s text, the CLI line) so the
        // symlink and the reason are visible rather than silently no-op'd.
        match sync_worktree_cells(&worktree_store_root, main_store_root, feature)? {
            CellsSync::Skipped { path, reason } => {
                out.insert(
                    "cellsSync".into(),
                    json!({ "skipped": true, "path": p(&path), "reason": reason }),
                );
            }
            // review D-P3-2: an empty prune omits the key entirely rather
            // than reporting `[]` — the common, nothing-happened case stays
            // silent in the report the same way `cellsSync` already does.
            CellsSync::Ran { pruned } => {
                if !pruned.is_empty() {
                    out.insert("pruned".into(), json!(pruned));
                }
            }
        }
    }

    let state_file = worktree_store_root.join("state.json");
    if state_file.exists() {
        out.insert("created".into(), Value::Bool(false));
        out.insert("reason".into(), json!("state.json already exists"));
        out.insert("worktreeStoreRoot".into(), json!(p(&worktree_store_root)));
        out.insert("onboarding".into(), Value::Object(onboarding));
        out.insert("config".into(), Value::Object(config));
        out.insert("identity".into(), Value::Object(identity));
        return Some(out);
    }

    // A FRESH state.json: main's live phase/gates/workers are deliberately
    // NOT copied, so a worktree can never inherit a gate it never earned.
    let fresh_state = json!({
        "schema_version": FRESH_STATE_SCHEMA_VERSION,
        "phase": "idle",
        "feature": feature,
        "mode": Value::Null,
        "approved_gates": {
            "context": false,
            "shape": false,
            "execution": false,
            "review": false,
        },
        "workers": [],
        "summary": "",
        "next_action": "Invoke bee-hive.",
    });
    let tmp = {
        let mut name = state_file.clone().into_os_string();
        name.push(".tmp");
        PathBuf::from(name)
    };
    std::fs::write(&tmp, format!("{}\n", jsjson::stringify_pretty(&fresh_state))).ok()?;
    std::fs::rename(&tmp, &state_file).ok()?;

    out.insert("created".into(), Value::Bool(true));
    out.insert("worktreeStoreRoot".into(), json!(p(&worktree_store_root)));
    out.insert("onboarding".into(), Value::Object(onboarding));
    out.insert("config".into(), Value::Object(config));
    out.insert("identity".into(), Value::Object(identity));
    out.insert("state".into(), fresh_state);
    Some(out)
}
