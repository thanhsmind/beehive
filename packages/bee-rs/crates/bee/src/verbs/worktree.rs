// bee worktree — native port of the `worktree` verb group.
//
// Ported argv shapes (everything else returns None BEFORE any output and the
// whole command re-runs under Node):
//   worktree list                       [--json]
//   worktree register   --feature F     [--json]
//   worktree unregister [--id ID]       [--json]
//   worktree new    --feature F ...     [--json]   ONLY the linked-worktree
//   worktree merge  --id ID     ...     [--json]   refusal (see below)
//
// DELEGATED to Node, by design:
//
//   * `worktree new` from the MAIN checkout — the creating path spawns
//     `git worktree add` / `git rev-parse`, syncs skill trees, optionally
//     runs the project's companion-start command and symlinks its result,
//     and rolls all of that back best-effort on any post-creation failure
//     (worktree-store.mjs createFeatureWorktree, ~450 lines of spawn-shaped
//     behavior). Nothing about that is provable byte-for-byte here yet.
//   * `worktree merge` from the MAIN checkout — enqueues through the shared
//     integration queue, takes a processor lease, spawns `git merge --no-ff`,
//     runs the host's `commands.verify` under the 'worktree-admin' store
//     lock, tears down companions and optionally removes worktree+branch+
//     grant. Same reason.
//   * Anything reached through a WorktreeLinkInvalidError: main()'s own
//     findRepoRoot throws before dispatch, and that throw ALSO escapes the
//     timings.jsonl append inside bee.mjs's recordTiming try-block. The
//     shared timing wrapper here always appends, so the whole command goes
//     back to Node rather than emit a subtly different side effect.
//   * A grants registry file that exists but does not parse with serde:
//     Node's readGrants swallows the parse error and reads `{}`, but V8's
//     JSON grammar is not provably identical to serde's, so "unparseable
//     here" cannot be turned into "Node saw {} too".
//
// What IS native is the part the linked-worktree root port unlocked: all
// three grant-registry verbs, plus the two "you are inside a worktree"
// refusals, work from INSIDE a linked worktree — the classification, the
// bidirectional gitdir validation and the grant lookup all come from
// crate::roots::resolve_roots_core (state.mjs resolveRootsCore), not from a
// second copy of the walk.
//
// Provenance: bee.mjs handleWorktreeList / handleWorktreeRegister /
// handleWorktreeUnregister / handleWorktreeNew / handleWorktreeMerge /
// resolveMainRoot / requireFlag / emit / emitError, plus lib/worktree-store.mjs
// readGrants / listGrants / writeGrant / writeGrantCore / removeGrant /
// removeGrantCore / grantsFile / writeGrantsFileAtomic /
// bootstrapWorktreeStore / writeCreationIdentity, and lib/lock.mjs
// withStoreLock.
//
// Locking: writeGrant/removeGrant serialize on the SAME cross-process lock
// file Node uses — lock name "worktree-admin" on the MAIN root — through
// crate::lock::acquire_store_lock with lock.mjs's own MAX_ATTEMPTS, so the
// two runtimes serialize against each other mid-campaign (contract C1). A
// LOCK_BUSY refusal is reproduced natively (never delegated: it is reached
// AFTER a lock attempt, and delegating would double the contention
// telemetry).
//
// One accepted residual, mirroring verbs/reservations.rs: `register` can
// still return a late None if a file copy fails mid-bootstrap. Every step it
// has taken by then is idempotent (writeGrant merges, mkdir is recursive,
// the copies are copy-if-absent), so the Node re-run reproduces the same
// error message over the same store, exactly like two sequential CLI calls.

use crate::registry::check_manifest_drift;
use crate::roots::{resolve_roots_core, Resolution};
use crate::verbs::reservations::{js_numberify, now_iso, parse_flags, FlagV, Flags};
use crate::verbs::{emit_no_root_error, record_timing};
use crate::{jsjson, lock};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

/// worktree-store.mjs's lock name, byte-identical (hardening-4b).
const WORKTREE_ADMIN_LOCK: &str = "worktree-admin";
/// worktree-store.mjs FRESH_STATE_SCHEMA_VERSION.
const FRESH_STATE_SCHEMA_VERSION: &str = "1.0";

// ─── emission frame (bee.mjs emit/emitError + the direct-run timing) ───────
// Same shape as verbs/decisions.rs's, with one difference that is the whole
// point of this module: the prelude keeps the FULL root resolution, because
// every verb here has to know whether it is standing inside a worktree.

struct Ctx {
    /// main()'s `root`: resolveRoots(cwd).storeRoot.
    root: PathBuf,
    /// resolveRoots(cwd).worktreeResolution — "ordinary" | "linked-valid".
    kind: &'static str,
    /// The git-verified worktree id (linked-valid only).
    id: Option<String>,
    /// resolveRoots(cwd).mainRoot (linked-valid only).
    main_root: Option<PathBuf>,
    /// resolveRoots(cwd).workRoot — the physical checkout.
    work_root: PathBuf,
    cmd: &'static str,
    use_json: bool,
    t0: Instant,
    drift_changed: bool,
    drift_hint: &'static str,
}

enum Pre {
    Go(Box<Ctx>),
    Emitted(ExitCode),
}

fn prelude(cmd: &'static str, use_json: bool, t0: Instant) -> Option<Pre> {
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
        // main()'s findRepoRoot throws these; see the module header.
        Resolution::LinkInvalid { .. } | Resolution::Exotic => return None,
        Resolution::Unresolved => {
            return Some(Pre::Emitted(emit_no_root_error(&cwd, cmd, use_json, t0)))
        }
    };
    let drift = check_manifest_drift(&root).ok()?;
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
    fn emit(&self, result: &Value, text: &str) -> ExitCode {
        if self.drift_changed {
            eprintln!("manifest_changed: true — {}", self.drift_hint);
        }
        if self.use_json {
            println!("{}", jsjson::stringify_pretty(result));
        } else {
            println!("{text}");
        }
        record_timing(&self.root, self.cmd, self.t0, true);
        ExitCode::SUCCESS
    }

    /// bee.mjs emitError(): no drift line, {"error"} or stderr, exit 1.
    fn fail(&self, message: &str) -> ExitCode {
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
    fn main_root(&self) -> PathBuf {
        self.main_root.clone().unwrap_or_else(|| self.root.clone())
    }
}

/// Node prints paths through plain string interpolation of what
/// path.resolve/path.join produced — always valid UTF-8 here because the
/// resolution walked from an existing cwd.
fn p(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

// ─── worktree-store.mjs grant registry ────────────────────────────────────

fn grants_file(main_store_root: &Path) -> PathBuf {
    main_store_root.join("runtime").join("worktree-grants.json")
}

/// readGrants with the ONE difference the strangler needs: a file that
/// exists but does not parse HERE is `None` (delegate), because Node's `{}`
/// fallback also covers "V8 parsed it fine and serde would not".
/// Missing/unreadable file, or a parsed non-object, is `{}` exactly like
/// Node. An ARRAY registry (JS `typeof [] === 'object'`, so Node keeps it)
/// also delegates rather than model Object.keys over array indices.
fn read_grants_strict(main_store_root: &Path) -> Option<Map<String, Value>> {
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
fn write_grants_file_atomic(main_store_root: &Path, grants: &Map<String, Value>) -> std::io::Result<()> {
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

fn run_list(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
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

fn run_unregister(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
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
    let existing = read_grants_strict(&main_store_root)?;

    let mut guard = match lock::acquire_store_lock(&main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS) {
        Ok(g) => g,
        Err(busy) => return Some(ctx.fail(&busy.message())),
    };
    // removeGrantCore: a no-op (no write at all) when the id was never there.
    let write_result = if existing.contains_key(&id) {
        let mut next = existing.clone();
        next.remove(&id);
        write_grants_file_atomic(&main_store_root, &next).err()
    } else {
        None
    };
    guard.release();
    if write_result.is_some() {
        // An fs error here is a V8-worded throw in Node; nothing was emitted
        // yet, and the failed write left the registry untouched.
        return None;
    }

    let mut result = Map::new();
    result.insert("ok".into(), Value::Bool(true));
    result.insert("id".into(), json!(id));
    result.insert("main_root".into(), json!(p(&main_root)));
    Some(ctx.emit(&Value::Object(result), &format!("Removed worktree grant for id {id}.")))
}

// ─── worktree register ────────────────────────────────────────────────────

fn run_register(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
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
    let text = [
        format!("Registered worktree grant: id {id} (feature \"{feature}\")."),
        format!("  worktree:    {}", p(&worktree_root)),
        format!("  main store:  {}", p(&main_store_root)),
        last,
    ]
    .join("\n");
    Some(ctx.emit(&Value::Object(result), &text))
}

/// worktree-store.mjs writeCreationIdentity — the worktree's IMMUTABLE
/// creation slug, write-if-absent, never fatal.
fn write_creation_identity(worktree_store_root: &Path, feature: &str) -> Map<String, Value> {
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

/// worktree-store.mjs bootstrapWorktreeStore. `None` = an fs failure Node
/// would surface with a V8 message (delegate; every step so far is
/// idempotent, see the module header).
fn bootstrap_worktree_store(
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

    let mut out = Map::new();
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

// ─── worktree new / merge: the "not from a worktree" refusals ─────────────
// Both handlers refuse before ANY mutation, before any lock, and before any
// git spawn — the one part of these two verbs that is deterministic and
// therefore native. Everything else in them delegates.

fn bool_flag_ok(flags: &Flags, name: &str) -> bool {
    match flags.get(name) {
        None | Some(FlagV::Present) => true,
        Some(FlagV::S(s)) => s == "true" || s == "false",
    }
}

fn run_new(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["feature", "base-ref", "with-companion"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "with-companion") {
        return None; // validate() rejects a non-boolean value first
    }
    // requireFlag(flags, 'feature') runs before the resolution check.
    match flags.get("feature") {
        Some(FlagV::S(s)) if !s.is_empty() => {}
        _ => return None,
    }
    let ctx = match prelude("worktree new", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind == "ordinary" {
        return None; // the creating path — Node's
    }
    Some(ctx.fail(&format!(
        "\"bee worktree new\" must be run from inside the main checkout, not a \"{}\" checkout — run it from the main repo root, then open your next session inside the created worktree.",
        ctx.kind
    )))
}

fn run_merge(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["id", "cleanup", "queue-wait-ms"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "cleanup") {
        return None;
    }
    if flags.get("queue-wait-ms").is_some() {
        return None; // a type:"number" flag through validate() — Node's
    }
    match flags.get("id") {
        Some(FlagV::S(s)) if !s.is_empty() => {}
        _ => return None,
    }
    let ctx = match prelude("worktree merge", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind == "ordinary" {
        return None; // the merging path — Node's
    }
    Some(ctx.fail(&format!(
        "\"bee worktree merge\" must be run from inside the main checkout, not a \"{}\" checkout — a worktree, including the one being merged, cannot merge itself.",
        ctx.kind
    )))
}

// ─── routing ──────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "worktree" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> = args[2..]
        .iter()
        .map(|a| a.to_str())
        .collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // Node renders command-scoped help
    }
    let (flags, use_json) = parse_flags(&toks)?;
    match verb {
        "list" => run_list(flags, use_json, t0),
        "register" => run_register(flags, use_json, t0),
        "unregister" => run_unregister(flags, use_json, t0),
        "new" => run_new(flags, use_json, t0),
        "merge" => run_merge(flags, use_json, t0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_of(pairs: &[(&str, Value)]) -> Map<String, Value> {
        let mut m = Map::new();
        for (k, v) in pairs {
            m.insert((*k).to_string(), v.clone());
        }
        m
    }

    /// The registry file Node writes: 2-space JSON + a trailing newline,
    /// insertion order preserved (pinned against writeGrantsFileAtomic).
    #[test]
    fn grants_file_bytes_match_node() {
        let tmp = tempfile::tempdir().unwrap();
        let store = tmp.path().join(".bee");
        let grants = map_of(&[("wt-b", json!(true)), ("wt-a", json!(true))]);
        write_grants_file_atomic(&store, &grants).unwrap();
        let text = std::fs::read_to_string(grants_file(&store)).unwrap();
        assert_eq!(text, "{\n  \"wt-b\": true,\n  \"wt-a\": true\n}\n");
        // The tmp file never survives a successful write.
        assert!(!store.join("runtime").join("worktree-grants.json.tmp").exists());
    }

    #[test]
    fn read_grants_strict_matches_node_for_the_shapes_it_serves() {
        let tmp = tempfile::tempdir().unwrap();
        let store = tmp.path().join(".bee");
        // Missing file -> {}.
        assert_eq!(read_grants_strict(&store), Some(Map::new()));
        std::fs::create_dir_all(store.join("runtime")).unwrap();
        // A parsed non-object -> {} (Node's `typeof parsed === 'object'`).
        std::fs::write(grants_file(&store), "5").unwrap();
        assert_eq!(read_grants_strict(&store), Some(Map::new()));
        std::fs::write(grants_file(&store), "null").unwrap();
        assert_eq!(read_grants_strict(&store), Some(Map::new()));
        // A real registry round-trips in file order.
        std::fs::write(grants_file(&store), "{\"b\":true,\"a\":false}").unwrap();
        let got = read_grants_strict(&store).unwrap();
        assert_eq!(got.keys().collect::<Vec<_>>(), vec!["b", "a"]);
        // Unparseable / array -> delegate.
        std::fs::write(grants_file(&store), "{oops").unwrap();
        assert_eq!(read_grants_strict(&store), None);
        std::fs::write(grants_file(&store), "[true]").unwrap();
        assert_eq!(read_grants_strict(&store), None);
    }

    /// bootstrapWorktreeStore's two shapes, including the idempotence rule:
    /// an existing state.json is never overwritten, and the creation identity
    /// is written BEFORE that early return (so an adopted worktree gets one).
    #[test]
    fn bootstrap_shapes_match_node() {
        let tmp = tempfile::tempdir().unwrap();
        let main_store = tmp.path().join("main").join(".bee");
        std::fs::create_dir_all(&main_store).unwrap();
        std::fs::write(main_store.join("onboarding.json"), "{\"bee_version\":\"x\"}").unwrap();
        let wt = tmp.path().join("wt-a");
        std::fs::create_dir_all(&wt).unwrap();

        let first = bootstrap_worktree_store(&wt, &main_store, "demo").unwrap();
        assert_eq!(first.get("created"), Some(&Value::Bool(true)));
        assert_eq!(
            first.keys().collect::<Vec<_>>(),
            vec!["created", "worktreeStoreRoot", "onboarding", "config", "identity", "state"]
        );
        assert_eq!(first["onboarding"]["copied"], Value::Bool(true));
        assert_eq!(first["config"]["copied"], Value::Bool(false));
        assert_eq!(first["config"]["reason"], json!("main store has no config.json"));
        assert_eq!(first["identity"]["written"], Value::Bool(true));
        let state = std::fs::read_to_string(wt.join(".bee").join("state.json")).unwrap();
        assert!(state.starts_with("{\n  \"schema_version\": \"1.0\",\n  \"phase\": \"idle\","));
        assert!(state.ends_with("}\n"));

        // Re-running never clobbers state.json or the creation identity.
        std::fs::write(wt.join(".bee").join("state.json"), "{\"phase\":\"swarming\"}").unwrap();
        let second = bootstrap_worktree_store(&wt, &main_store, "renamed").unwrap();
        assert_eq!(second.get("created"), Some(&Value::Bool(false)));
        assert_eq!(second["reason"], json!("state.json already exists"));
        assert_eq!(
            second["identity"]["reason"],
            json!("creation identity already recorded — never overwritten")
        );
        assert_eq!(second["onboarding"]["reason"], json!("onboarding.json already exists"));
        let identity =
            std::fs::read_to_string(wt.join(".bee").join("runtime").join("worktree-identity.json"))
                .unwrap();
        assert!(identity.contains("\"feature\": \"demo\""));
        assert_eq!(
            std::fs::read_to_string(wt.join(".bee").join("state.json")).unwrap(),
            "{\"phase\":\"swarming\"}"
        );
    }

    /// The lock file this module contends on is Node's, by name.
    #[test]
    fn grant_writes_use_nodes_lock_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let Ok(mut guard) = lock::acquire_store_lock(root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
        else {
            panic!("a fresh root's worktree-admin lock must be free")
        };
        assert!(lock::lock_file_path(root, "worktree-admin").exists());
        guard.release();
    }
}
