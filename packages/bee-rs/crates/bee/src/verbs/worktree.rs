// bee worktree — native port of the `worktree` verb group.
//
// Ported argv shapes (everything else returns None BEFORE any output and the
// whole command re-runs under Node):
//   worktree list                       [--json]
//   worktree register   --feature F     [--json]
//   worktree unregister [--id ID]       [--json]
//   worktree new    --feature F [--base-ref R]  [--json]   see the gates below
//   worktree merge  --id ID     ...             [--json]   ONLY the
//                                                          linked-worktree refusal
//
// `worktree new` — NATIVE (R6), including the creating path from the MAIN
// checkout: createFeatureWorktree whole, inside ONE 'worktree-admin' hold —
// the eight zero-mutation refusals, the real `git worktree add`, the post-add
// block (git-verified id, grant write, store bootstrap, `rev-parse HEAD`,
// registerWorkspace, skill-tree sync) and the ORDER-SENSITIVE rollback
// ladder. The port that unblocked it is verbs/workspace_store.rs: the
// registerWorkspace/unregisterWorkspace pair the post-add block and rungs
// R1/R2 of the ladder need, with Node's `workspace:<id>` lock name.
//
// Two gates return None BEFORE any lock, git spawn or write:
//   * `--with-companion` — runCompanionStart's failure arms all fire AFTER
//     `git worktree add`, and two of them embed V8 bytes (JSON.parse's own
//     message, plus up to 500 raw bytes of the child's stdout). Once the
//     worktree exists nothing can delegate, so the shape never starts here.
//   * ANY other live session (wcg-3's shared-nested-checkout guard). The
//     guard short-circuits to `false` — no filesystem scan at all — when no
//     other session is concurrently live, which is the only shape served
//     natively. With a second live session both remaining arms are
//     unprovable from this crate: guards.mjs's `scanForNestedCheckout` (a
//     depth-limited tree walk with no Rust counterpart) and
//     `resolveVerifiedCompanionMountReal` (private to hooks/write_guard.rs —
//     re-deriving it here would fork the guard, the exact drift C5 exists to
//     prevent), and the detection-failure refusal interpolates a V8 message.
// A bare `--base-ref` (no value) also delegates: Node stringifies `true` into
// a ref name, an unproven shape.
//
// DELEGATED to Node, by design:
//
//   * `worktree merge` from the MAIN checkout. RE-MEASURED at R6 alongside
//     the `new` port, and it stays delegated for reasons `new` does not
//     share:
//       - It runs through integration-queue.mjs's drainer: enqueue, poll for
//         front-of-line, then take a PROCESSOR LEASE (lease-store.mjs
//         acquireLeases/renewLease/releaseLease with `epoch` fencing) held
//         across the host's `commands.verify` child. lease-store.mjs's
//         renewLease/LEASE_FENCE_STALE contracts are a SEPARATE, concurrent
//         R6 workstream (crates/bee/src/lease_store.rs); composing merge on
//         top of a half-landed lease store would fork exactly the
//         store-mutation logic C1 forbids. This is a sequencing blocker, not
//         a shape blocker — merge should land ON that port, after it.
//       - `runVerifyChild` is Node's async `spawn(command, {shell:true})`
//         with an unref'd renewal timer, and it resolves on `exit` (not
//         `close`), so trailing pipe output can be lost; its libuv spawn
//         error (`spawn /bin/sh ENOENT`, `spawn cmd.exe ENOENT`) is
//         concatenated into `verifyOutcome.combined` and surfaces verbatim
//         in MERGE_VERIFY_RED's `output_tail` — a V8/libuv-worded byte
//         reached after the merge is already staged.
//       - Eight `runGit(...).stdout.trim()` sites in the merge path are NOT
//         null-guarded (mainUntouchedProof ×2, checkMergeFence ×2,
//         preMergeHead, stagedTreeHash, postCommitStatus), so a spawn
//         failure raises `TypeError: Cannot read properties of null
//         (reading 'trim')` whose V8 text is then PERSISTED into the queue
//         record's `error` field by processAsOwner. `new` has exactly one
//         such site and it is unreachable (see post_add step 10.4); merge's
//         are reachable and load-bearing.
//     What IS already reusable when that work starts: verbs/cells.rs's
//     `run_declared_tests` (the POSIX-sh runner with the explicit-PATH fix)
//     for the host verify, this file's own `run_git`/grant registry/store
//     bootstrap/`node_fs_error_message`, verbs/workspace_store.rs for
//     performCleanup's unregisterWorkspace rung, and
//     crate::roots::resolve_roots_core for every classification.
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
use crate::verbs::reservations::{js_numberify, js_trim, now_iso, parse_flags, FlagV, Flags};
use crate::verbs::workspace_store as ws;
use crate::verbs::{emit_no_root_error, record_timing};
use crate::{jsjson, lock};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
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
struct GitOut {
    status: Option<i32>,
    stdout: Option<String>,
    stderr: Option<String>,
}

impl GitOut {
    /// `${result.status}` — JS renders a null status as "null".
    fn status_disp(&self) -> String {
        self.status.map_or_else(|| "null".to_string(), |c| c.to_string())
    }
    /// The `(stderr || stdout || '').trim() || `exit ${status}`` fallback
    /// chain three refusals share, byte-for-byte (JS `||` on '' is falsy).
    fn fail_text(&self) -> String {
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

fn run_git(cwd: &Path, args: &[&str]) -> GitOut {
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
fn is_ordinary_checkout(root: &Path) -> bool {
    std::fs::metadata(root.join(".git")).map(|m| m.is_dir()).unwrap_or(false)
}

/// resolveBaseRefCommit: `git rev-parse --verify --end-of-options <ref>^{commit}`.
fn resolve_base_ref_commit(cwd: &Path, r#ref: &str) -> Option<String> {
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
fn branch_exists(main_root: &Path, branch: &str) -> bool {
    let spec = format!("refs/heads/{branch}");
    run_git(main_root, &["rev-parse", "--verify", "--quiet", &spec]).status == Some(0)
}

/// `path.resolve` semantics for a single (possibly relative) segment:
/// absolute wins outright, otherwise join, then lexically normalize `.`/`..`.
fn js_path_resolve(base: &Path, segment: &str) -> PathBuf {
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
fn node_fs_error_message(err: &std::io::Error, syscall: &str, path: &Path) -> String {
    use std::io::ErrorKind::*;
    let (code, text) = match err.kind() {
        NotFound => ("ENOENT", "no such file or directory"),
        PermissionDenied => ("EACCES", "permission denied"),
        IsADirectory => ("EISDIR", "illegal operation on a directory"),
        _ => ("EIO", "i/o error"),
    };
    format!("{code}: {text}, {syscall} '{}'", p(path))
}

/// readWorktreeGitVerifiedId — the AUTHORITATIVE id: git suffixes a colliding
/// basename, so the id is always re-read from the new worktree's own `.git`
/// pointer rather than assumed to be the directory name.
fn read_worktree_git_verified_id(worktree_root: &Path) -> Result<String, String> {
    let git_file = worktree_root.join(".git");
    let raw = std::fs::read_to_string(&git_file)
        .map_err(|e| node_fs_error_message(&e, "open", &git_file))?;
    let raw = js_trim(&raw);
    // /^gitdir:\s*(.+)$/ over the TRIMMED text: no /m, and `.` never matches a
    // line terminator, so a multi-line pointer simply does not match.
    let captured = raw.strip_prefix("gitdir:").map(|rest| rest.trim_start()).filter(|rest| {
        !rest.is_empty() && !rest.contains(['\n', '\r', '\u{2028}', '\u{2029}'])
    });
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
fn skill_sync_roots() -> [String; 2] {
    [
        format!(".claude{MAIN_SEPARATOR}skills"),
        format!(".agents{MAIN_SEPARATOR}skills"),
    ]
}

/// `fs.cpSync(src, dest, {recursive: true})` for the plain dir/file trees a
/// `bee-*` skill directory is. Anything else (symlink, device) is an error,
/// which lands in `skipped[].reason` exactly as a cpSync throw would.
fn cp_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
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
fn sync_worktree_skills(main_root: &Path, worktree_root: &Path) -> Value {
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

// ─── worktree-store.mjs createFeatureWorktree ─────────────────────────────

/// `refuse(code, message)` throws a WorktreeCreateError whose `.message` is
/// `[CODE] message` — bee.mjs's dispatcher surfaces only `.message`, so the
/// bracket prefix is part of every observable byte.
fn refuse(code: &str, message: String) -> CErr {
    CErr::Refuse(format!("[{code}] {message}"))
}

enum CErr {
    /// A shape whose Node bytes embed a V8 message — delegate. Only ever
    /// returned BEFORE `git worktree add` runs (nothing has mutated).
    Ex,
    Refuse(String),
}

struct Created {
    id: String,
    worktree_root: PathBuf,
    branch: String,
    base_ref: Option<String>,
    base_ref_sha: Option<String>,
    bootstrap: Map<String, Value>,
    skills_sync: Value,
}

/// FEATURE_SLUG_RE = /^[a-z0-9][a-z0-9-]*$/.
fn feature_slug_ok(feature: &str) -> bool {
    let mut chars = feature.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// createFeatureWorktree, whole. The entire body — every validation, every
/// refusal, the `git worktree add`, the post-add block and the rollback
/// ladder — runs inside ONE `worktree-admin` hold on mainRoot, exactly as
/// Node's `withStoreLock(mainRoot, 'worktree-admin', ...)` wrapper does.
///
/// `companionStartCommand`/`companionMountPath` are structurally absent here:
/// `run_new` delegates every `--with-companion` invocation before reaching
/// this function, so the WORKTREE_COMPANION_CONFIG_INCOMPLETE refusal, the
/// mount validation and runCompanionStart are all unreachable — and the
/// returned `companion` is always `null`, which is what bee.mjs prints.
fn create_feature_worktree(
    main_root: &Path,
    feature: &str,
    base_ref: Option<&str>,
    lock_busy: &mut Option<String>,
) -> Result<Created, CErr> {
    let main_store_root = main_root.join(".bee");
    // Pre-probe BEFORE the lock: an unparseable grants registry delegates
    // here rather than from inside the hold (campaign rule 2 — a delegation
    // after an acquire would double contention.jsonl's telemetry).
    read_grants_strict(&main_store_root).ok_or(CErr::Ex)?;

    let mut guard = match lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
    {
        Ok(g) => g,
        Err(busy) => {
            *lock_busy = Some(busy.message());
            return Err(CErr::Ex); // signalled to the caller through lock_busy
        }
    };
    let out = create_feature_worktree_locked(main_root, feature, base_ref);
    guard.release();
    out
}

fn create_feature_worktree_locked(
    main_root: &Path,
    feature: &str,
    base_ref: Option<&str>,
) -> Result<Created, CErr> {
    // (1) slug.
    if !feature_slug_ok(feature) {
        return Err(refuse(
            "WORKTREE_INVALID_SLUG",
            format!(
                "feature slug {} must match /^[a-z0-9][a-z0-9-]*$/ (lowercase letters/digits, starting with a letter or digit, hyphens allowed after that).",
                jsjson::stringify(&Value::String(feature.to_string()))
            ),
        ));
    }

    // (2) companion both-or-neither — unreachable, see the doc comment.

    // (3) base ref. `baseRef !== undefined && !== null && !== ''`, so an
    // empty --base-ref is treated as absent, exactly like Node.
    let mut base_ref_sha: Option<String> = None;
    if let Some(r) = base_ref.filter(|s| !s.is_empty()) {
        match resolve_base_ref_commit(main_root, r) {
            Some(sha) => base_ref_sha = Some(sha),
            None => {
                return Err(refuse(
                    "WORKTREE_BASE_NOT_FOUND",
                    format!(
                        "--base-ref {} does not resolve to a commit in {} (\"git rev-parse --verify\" found nothing) — check the ref/sha/tag exists (and isn't just a syntax typo).",
                        jsjson::stringify(&Value::String(r.to_string())),
                        p(main_root)
                    ),
                ))
            }
        }
    }

    // (4) the belt-and-braces ordinary-checkout guard.
    if !is_ordinary_checkout(main_root) {
        return Err(refuse(
            "WORKTREE_CALLER_NOT_ORDINARY",
            format!(
                "\"bee worktree new\" must be run from the main checkout, not a linked worktree ({} is not an ordinary checkout).",
                p(main_root)
            ),
        ));
    }

    // (5) derivation.
    let repo_basename = main_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sibling_dir_name = format!("{repo_basename}--wt--{feature}");
    let worktree_root = js_path_resolve(&js_path_resolve(main_root, ".."), &sibling_dir_name);
    let branch = format!("wt/{feature}");
    let main_store_root = main_root.join(".bee");

    // (6) target path.
    if worktree_root.exists() {
        return Err(refuse(
            "WORKTREE_TARGET_EXISTS",
            format!("{} already exists.", p(&worktree_root)),
        ));
    }

    // (7) branch.
    if branch_exists(main_root, &branch) {
        return Err(refuse(
            "WORKTREE_BRANCH_EXISTS",
            format!("branch \"{branch}\" already exists in {}.", p(main_root)),
        ));
    }

    // (8) advisory grant collision (strict `=== true`).
    let grants = read_grants_strict(&main_store_root).ok_or(CErr::Ex)?;
    let likely_id = sibling_dir_name;
    if grants.get(&likely_id) == Some(&Value::Bool(true)) {
        return Err(refuse(
            "WORKTREE_GRANT_EXISTS",
            format!(
                "a worktree grant already exists for id \"{likely_id}\" — run \"bee worktree unregister --id {likely_id}\" (or \"git worktree prune\") before retrying."
            ),
        ));
    }

    // (9) THE MUTATION. The RESOLVED SHA is what git receives, never the
    // original ref string.
    let worktree_root_s = p(&worktree_root);
    let mut add_args: Vec<&str> = vec!["worktree", "add", "-b", &branch, "--", &worktree_root_s];
    if let Some(sha) = &base_ref_sha {
        add_args.push(sha);
    }
    let add_result = run_git(main_root, &add_args);
    if add_result.status != Some(0) {
        return Err(refuse(
            "WORKTREE_ADD_FAILED",
            format!("git worktree add failed: {}", add_result.fail_text()),
        ));
    }

    // (10) the post-add block. Every failure below enters the rollback
    // ladder; NONE of them may delegate (the worktree already exists).
    let mut id: Option<String> = None;
    let attempt = post_add(
        main_root,
        &main_store_root,
        &worktree_root,
        feature,
        &branch,
        base_ref,
        base_ref_sha.as_deref(),
        grants,
        &mut id,
    );
    match attempt {
        Ok(created) => Ok(created),
        Err(post_add_message) => Err(rollback(
            main_root,
            &main_store_root,
            &worktree_root,
            feature,
            &branch,
            id.as_deref(),
            &post_add_message,
        )),
    }
}

/// Steps 10.1-10.7, in Node's exact order. `Err(String)` carries the message
/// the ladder interpolates as `postAddMessage`.
#[allow(clippy::too_many_arguments)]
fn post_add(
    main_root: &Path,
    main_store_root: &Path,
    worktree_root: &Path,
    feature: &str,
    branch: &str,
    base_ref: Option<&str>,
    base_ref_sha: Option<&str>,
    grants: Map<String, Value>,
    id_out: &mut Option<String>,
) -> Result<Created, String> {
    // 10.1 — the authoritative id. `id` stays None if this throws, so the
    // ladder's `if (id)` rungs are skipped, exactly as in Node.
    let id = read_worktree_git_verified_id(worktree_root)?;
    *id_out = Some(id.clone());

    // 10.2 — writeGrantCore (the UNLOCKED core: withStoreLock is not
    // reentrant and this whole body already holds 'worktree-admin').
    let mut next = grants;
    next.insert(id.clone(), Value::Bool(true));
    write_grants_file_atomic(main_store_root, &next)
        .map_err(|e| node_fs_error_message(&e, "open", &grants_file(main_store_root)))?;

    // 10.3 — bootstrap the worktree's own store.
    let bootstrap = bootstrap_worktree_store(worktree_root, main_store_root, feature)
        .ok_or_else(|| "EIO: i/o error, open".to_string())?;

    // 10.4 — the workspace base sha. `git worktree add` has just succeeded,
    // so git is provably launchable and `.stdout` is never null here — the
    // one Node shape (`TypeError: Cannot read properties of null`) that this
    // line can raise is unreachable by construction.
    let workspace_base_sha: Option<String> = match base_ref_sha {
        Some(sha) => Some(sha.to_string()),
        None => {
            let head = run_git(worktree_root, &["rev-parse", "HEAD"]);
            let trimmed = js_trim(head.stdout.as_deref().unwrap_or("")).to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
    };

    // 10.5 — registerWorkspace: the WRITE-OWNERSHIP ledger, alongside the
    // grant's STORE-TOPOLOGY ledger. Takes its own `workspace:<id>` lock
    // NESTED inside the 'worktree-admin' hold — a different lock name, so
    // never a self-deadlock, exactly as Node nests it.
    ws::register_workspace(
        main_root,
        ws::RegisterSpec {
            id: &id,
            kind: "worktree",
            root: &p(worktree_root),
            branch: Some(branch),
            base_sha: workspace_base_sha.as_deref(),
        },
        &now_iso(),
    )
    .map_err(|e| match e {
        // A WorkspaceStoreError / LockBusyError message, reproduced natively
        // by verbs/workspace_store.rs — this is the arm that used to be the
        // reason the whole verb delegated.
        ws::WsErr::Err { message, .. } => message,
        ws::WsErr::Ex => "EIO: i/o error, open".to_string(),
    })?;

    // 10.6 — companion: structurally absent (run_new delegates --with-companion).

    // 10.7 — skills: best-effort, never fatal, never in the ladder.
    let skills_sync = sync_worktree_skills(main_root, worktree_root);

    Ok(Created {
        id,
        worktree_root: worktree_root.to_path_buf(),
        branch: branch.to_string(),
        base_ref: base_ref.filter(|s| !s.is_empty()).map(str::to_string),
        base_ref_sha: base_ref_sha.map(str::to_string),
        bootstrap,
        skills_sync,
    })
}

/// The ROLLBACK LADDER, in Node's exact order. Order is load-bearing: a
/// different unwind leaves a different tree behind, which is the C1 breach
/// the campaign forbids.
///
///   R1  removeGrantCore(mainStoreRoot, id)        only if id, best-effort
///   R2  unregisterWorkspace(mainRoot, id)         only if id, best-effort
///   R3  git worktree remove --force <worktreeRoot>   unconditional
///   R4  fs.existsSync(worktreeRoot) -> stillPresent
///   R5  git branch -D <branch>                    only if R3 ok && !R4
///   R6  refuse WORKTREE_POST_ADD_FAILED           same gate as R5
///   R7  refuse WORKTREE_POST_ADD_ROLLBACK_FAILED  otherwise
fn rollback(
    main_root: &Path,
    main_store_root: &Path,
    worktree_root: &Path,
    feature: &str,
    branch: &str,
    id: Option<&str>,
    post_add_message: &str,
) -> CErr {
    if let Some(id) = id {
        // R1 — removeGrantCore: a no-op (no write at all) when the id is
        // absent. Best-effort; the typed refusal below fires either way.
        if let Some(existing) = read_grants_strict(main_store_root) {
            if existing.contains_key(id) {
                let mut next = existing;
                next.remove(id);
                let _ = write_grants_file_atomic(main_store_root, &next);
            }
        }
        // R2 — best-effort workspace unregister, always after R1.
        let _ = ws::unregister_workspace(main_root, id);
    }
    // R3 — unconditional; its status is the branch point.
    let remove_result = run_git(main_root, &["worktree", "remove", "--force", &p(worktree_root)]);
    // R4.
    let still_present = worktree_root.exists();
    if remove_result.status == Some(0) && !still_present {
        // R5 — only once the worktree is confirmed gone (git refuses to
        // delete a branch a live worktree still has checked out).
        let _ = run_git(main_root, &["branch", "-D", branch]);
        // R6.
        return refuse(
            "WORKTREE_POST_ADD_FAILED",
            format!(
                "{} was created but could not be registered ({post_add_message}); it has been rolled back (worktree and branch \"{branch}\" removed).",
                p(worktree_root)
            ),
        );
    }
    // R7.
    refuse(
        "WORKTREE_POST_ADD_ROLLBACK_FAILED",
        format!(
            "{} was created but could not be registered ({post_add_message}), and the rollback itself failed — the tree still exists on disk; run \"bee worktree register --feature {feature}\" from inside it to adopt it.",
            p(worktree_root)
        ),
    )
}

// ─── worktree new / merge ─────────────────────────────────────────────────

fn bool_flag_ok(flags: &Flags, name: &str) -> bool {
    match flags.get(name) {
        None | Some(FlagV::Present) => true,
        Some(FlagV::S(s)) => s == "true" || s == "false",
    }
}

/// `flags.x === true` — a bare `--x` or an explicit `--x=true`.
fn bool_flag_true(flags: &Flags, name: &str) -> bool {
    match flags.get(name) {
        Some(FlagV::Present) => true,
        Some(FlagV::S(s)) => s == "true",
        None => false,
    }
}

fn run_new(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["feature", "base-ref", "with-companion"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "with-companion") {
        return None; // validate() rejects a non-boolean value first
    }
    // --with-companion delegates WHOLE (see the module header): every arm of
    // runCompanionStart that can fail does so AFTER `git worktree add`, and
    // two of them embed V8 bytes (JSON.parse's message, up to 500 raw child
    // bytes) — unreachable-by-delegation once the worktree exists, so the
    // shape never starts natively.
    if bool_flag_true(&flags, "with-companion") {
        return None;
    }
    // requireFlag(flags, 'feature') runs before the resolution check.
    let feature = match flags.get("feature") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    // `flags['base-ref'] !== undefined ? String(flags['base-ref']) : undefined`
    // — a bare `--base-ref` stringifies `true` into a ref name; unproven here.
    let base_ref: Option<String> = match flags.get("base-ref") {
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(FlagV::Present) => return None,
        None => None,
    };

    let ctx = match prelude("worktree new", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&format!(
            "\"bee worktree new\" must be run from inside the main checkout, not a \"{}\" checkout — run it from the main repo root, then open your next session inside the created worktree.",
            ctx.kind
        )));
    }
    let main_root = ctx.work_root.clone();

    // wcg-3 (D1a/D3/D4): the shared-nested-checkout guard. It short-circuits
    // to `false` — a pure no-op, no filesystem scan at all — whenever no
    // OTHER session is concurrently live, which is the only shape served
    // natively here. The instant a second session IS live the whole verb goes
    // back to Node, because both remaining arms are unprovable from this
    // crate: guards.mjs's `scanForNestedCheckout` (a depth-limited walk with
    // no Rust counterpart) and `resolveVerifiedCompanionMountReal` (private
    // to hooks/write_guard.rs — re-deriving it here would fork the guard, the
    // exact drift C5 exists to prevent), plus the detection-failure refusal,
    // which interpolates the caught error's V8 message.
    let main_root_s = p(&main_root);
    let ctrl_root = crate::verbs::reservations::control_root_for(&main_root_s).ok()?;
    let session_id = crate::verbs::reservations::resolve_session_id(None, &ctrl_root).ok()?;
    if crate::verbs::reservations::is_concurrent_mode_excluding(&ctrl_root, session_id.as_deref())
        .ok()?
    {
        return None;
    }

    let mut lock_busy: Option<String> = None;
    let created = match create_feature_worktree(
        &main_root,
        &feature,
        base_ref.as_deref(),
        &mut lock_busy,
    ) {
        Ok(c) => c,
        Err(CErr::Refuse(message)) => return Some(ctx.fail(&message)),
        Err(CErr::Ex) => match lock_busy {
            // LockBusyError is reached AFTER a lock attempt, so it is native
            // (campaign rule 2 — delegating would double the telemetry).
            Some(message) => return Some(ctx.fail(&message)),
            None => return None,
        },
    };

    // GH #31 (wux-1): the explicit session-boundary next step.
    let next_step = format!(
        "Open a new session with cwd={} to work the \"{feature}\" feature there — this session stays on main. Merge back later with \"bee worktree merge --id {}\".",
        p(&created.worktree_root),
        created.id
    );
    let mut result = Map::new();
    result.insert("id".into(), json!(created.id));
    result.insert("worktreeRoot".into(), json!(p(&created.worktree_root)));
    result.insert("branch".into(), json!(created.branch));
    result.insert(
        "baseRef".into(),
        created.base_ref.clone().map_or(Value::Null, Value::String),
    );
    result.insert(
        "baseRefSha".into(),
        created.base_ref_sha.clone().map_or(Value::Null, Value::String),
    );
    result.insert("skillsSync".into(), created.skills_sync.clone());
    result.insert("companion".into(), Value::Null);
    result.insert("next_step".into(), json!(next_step));

    let skills_line = if created.skills_sync.get("applied") == Some(&Value::Bool(true)) {
        "  skills:      bee-* skill trees synced into the worktree.".to_string()
    } else {
        format!(
            "  skills:      NOT synced ({}) — bee* skills may be missing in a session opened there.",
            created
                .skills_sync
                .get("reason")
                .map(jsjson::js_to_string)
                .unwrap_or_default()
        )
    };
    let branch_line = match &created.base_ref_sha {
        Some(sha) => format!(
            "  branch:      {} (based on {}, resolved to {sha})",
            created.branch,
            jsjson::stringify(&created.base_ref.clone().map_or(Value::Null, Value::String))
        ),
        None => format!("  branch:      {}", created.branch),
    };
    let bootstrap_line = if created.bootstrap.get("created") == Some(&Value::Bool(true)) {
        format!(
            "  bootstrapped {} (phase idle, gates unapproved).",
            created
                .bootstrap
                .get("worktreeStoreRoot")
                .map(jsjson::js_to_string)
                .unwrap_or_default()
        )
    } else {
        format!(
            "  worktree .bee/state.json already existed — left untouched ({}).",
            created.bootstrap.get("reason").map(jsjson::js_to_string).unwrap_or_default()
        )
    };
    let text = [
        format!(
            "Created worktree for feature \"{feature}\": {}",
            p(&created.worktree_root)
        ),
        branch_line,
        bootstrap_line,
        skills_line,
        next_step,
    ]
    .join("\n");
    Some(ctx.emit(&Value::Object(result), &text))
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
