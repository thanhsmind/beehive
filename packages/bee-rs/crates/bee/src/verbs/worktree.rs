// bee worktree — native port of the `worktree` verb group.
//
// Ported argv shapes (everything else returns None BEFORE any output and the
// whole command re-runs under Node):
//   worktree list                       [--json]
//   worktree register   --feature F     [--json]
//   worktree unregister [--id ID]       [--json]
//   worktree new    --feature F [--base-ref R] [--with-companion] [--json]
//   worktree merge  --id ID [--cleanup] [--queue-wait-ms N]        [--json]
//
// `worktree new` — NATIVE (R6), including the creating path from the MAIN
// checkout: createFeatureWorktree whole, inside ONE 'worktree-admin' hold —
// the eight zero-mutation refusals, the real `git worktree add`, the post-add
// block (git-verified id, grant write, store bootstrap, `rev-parse HEAD`,
// registerWorkspace, COMPANION START, skill-tree sync) and the
// ORDER-SENSITIVE rollback ladder. The port that unblocked it is
// verbs/workspace_store.rs: the registerWorkspace/unregisterWorkspace pair the
// post-add block and rungs R1/R2 of the ladder need, with Node's
// `workspace:<id>` lock name.
//
// Both of `new`'s former gates are CLOSED (R6 cutover wave):
//   * `--with-companion` is native. runCompanionStart spawns the project's own
//     `commands.worktree_companion_start`, symlinks the declared worktreePath
//     at `<worktreeRoot>/<commands.worktree_companion_mount>` and writes the
//     `.bee/companion-session.json` marker; every failure arm folds into the
//     SAME post-add rollback ladder. It could not delegate because all of
//     those arms fire AFTER `git worktree add`, so the two that embed a
//     Node-only string are DIVERGENCES now, named below.
//   * wcg-3's shared-nested-checkout guard is native, whole, via
//     crate::nested_checkout — which imports guards.mjs's verification
//     predicates from hooks/write_guard.rs (widened to pub(crate) for exactly
//     this) rather than re-deriving them, so the two enforcement surfaces
//     cannot drift apart (C5).
// A bare `--base-ref` (no value) still delegates: Node stringifies `true` into
// a ref name, an unproven shape.
//
// `worktree merge` — NATIVE (R6), the whole three-phase staged transaction
// (mergeFeatureWorktree's P1/P2/P3), integration-queue.mjs's drainer
// (crate::integration_queue, built on the now-landed crate::lease_store) and
// bee.mjs's own text/JSON rendering. Its three recorded blockers were
// re-measured at this port and NONE of them survived:
//
//   (a) "the lease store is unported, so composing merge on it would fork
//       store-mutation logic" — DISSOLVED. src/lease_store.rs now carries
//       acquireLeases (hash-sorted batch + rollback), renewLease,
//       releaseLease, both fence checks, sweep and list, so
//       crate::integration_queue's tryBecomeProcessor / renewProcessorLease /
//       releaseProcessorLease / checkProcessorLeaseEpoch call THE port rather
//       than a second copy. This was a sequencing blocker; the sequence has
//       arrived.
//   (b) `runVerifyChild`'s libuv spawn error (`spawn cmd.exe ENOENT`)
//       concatenated into MERGE_VERIFY_RED's `output_tail` — PRE-CHECKED
//       away. That byte exists only when the SHELL ITSELF cannot start, so
//       `shell_launchable()` probes the shell BEFORE P1 stages anything: a
//       failed probe returns None with zero mutations and zero locks taken,
//       and a passing probe proves the real spawn will launch, making the arm
//       unreachable. (A delegation is only sound before a mutation — after
//       the merge is staged nothing here may fall back, which is exactly why
//       this had to become a pre-check rather than a late bail.)
//   (c) the un-null-guarded `runGit(...).stdout.trim()` sites
//       (mainUntouchedProof ×2, checkMergeFence ×2, preMergeHead,
//       stagedTreeHash, postCommitStatus) whose `TypeError: Cannot read
//       properties of null` V8 text `processAsOwner` PERSISTS into the queue
//       record — UNREACHABLE BY CONSTRUCTION, for the same reason `new`'s
//       single site is. Every one of them sits after
//       `mergeFeatureWorktreeStage`'s FIRST git call, `isTreeDirty(mainRoot)`,
//       which routes a never-launched spawn through `gitStatusPorcelain`'s
//       own throw — a DETERMINISTIC message (`"git status --porcelain" failed
//       in <root>: exit null`, reproduced natively) raised with zero
//       mutations. So a repo where git cannot be spawned refuses before any
//       merge is staged, and by the time a `.trim()` site runs git has
//       provably launched from that same cwd. The residual — git going away
//       BETWEEN two calls of one merge — is the same race class
//       verbs/workspace_store.rs documents for a record going unreadable
//       between its probe and its in-lock read.
//
// Two DOCUMENTED DIVERGENCES, both in the class state_group.rs's prune
// approximation and lease_store.rs's LEASE_CORRUPT residual already
// established:
//   * `runVerifyChild` resolves on Node's `exit` event, so output still
//     sitting in a pipe at exit can be LOST; this port joins its reader
//     threads and therefore always captures the full stream. It can only ADD
//     trailing bytes to `output_tail`, never change `status`, the red/green
//     verdict, or any `.bee/` record — and the Node side is not
//     reproducible run-to-run anyway.
//   * a verify spawn that fails DESPITE `shell_launchable()` (a race) keeps
//     Node's `status: null` verdict but carries Rust's io text instead of
//     libuv's, exactly as `node_fs_error_message` approximates elsewhere here.
//
// Both of `merge`'s former gates are CLOSED too (same wave):
//   * a COMPANION worktree (a parsed `.bee/companion-session.json` marker) is
//     native. `teardownCompanionIfPresent` spawns
//     `commands.worktree_companion_end` and unlinks the mount after every
//     zero-mutation refusal has cleared but BEFORE the merge is staged, and
//     the worktree dirty-check switches to `gitStatusPorcelainExcluding`'s
//     pathspec form (which has to be a pathspec, never text filtering — see
//     that function). On a companion worktree the TEARDOWN, not the staged
//     merge, is the first mutation, so every `MErr::Ex` probe in `run_merge`
//     is what keeps the no-fallback-after-mutation rule true.
//   * `--queue-wait-ms` is native: `js_string_to_number` is the full
//     `Number(string)` conversion, validate()'s finiteness gate is reproduced,
//     and the handler's positive-only filter decides whether the value
//     replaces DEFAULT_WAIT_BOUND_MS. Only a value validate() itself REFUSES
//     (a bare flag, an empty string, `Infinity`, a non-numeric literal) still
//     returns before any output — that is the dispatcher's own generic
//     machinery, shared by every verb, not this flag's arm.
//
// A BROKEN linked-worktree link (WorktreeLinkInvalidError) is native for all
// five verbs here, through crate::link_invalid — see that module for why the
// timing wrapper made it undelegatable-but-unreproducible before, and for the
// single named timing-line divergence.
//
// DELEGATED to Node, by design:
//
//   * A grants registry file that exists but does not parse with serde:
//     Node's readGrants swallows the parse error and reads `{}`, but V8's
//     JSON grammar is not provably identical to serde's, so "unparseable
//     here" cannot be turned into "Node saw {} too".
//   * An `Exotic` root resolution (a `.git` that vanished between existsSync
//     and statSync) — a V8-worded ENOENT.
//
// THREE MORE DOCUMENTED DIVERGENCES, all of them companion arms that fire
// after a mutation and therefore could never have been delegations:
//   * runCompanionStart's unparseable-stdout refusal interpolates serde's
//     parse message where Node interpolates V8's. Every other byte of that
//     sentence, including the 500-UTF-16-unit raw-stdout tail, is Node's.
//   * runCompanionStart's symlink failure carries an errno-CLASS-accurate
//     approximation of libuv's message (`EPERM: operation not permitted,
//     symlink '<target>' -> '<path>'` for a win32 host without
//     SeCreateSymbolicLinkPrivilege) — same class as `node_fs_error_message`.
//   * a companion marker that parses to a truthy value with no string
//     `mountPath` makes the .mjs die with a V8 TypeError from
//     `mountPath.replace(...)`; here it is an explicit typed refusal
//     (WORKTREE_MERGE_COMPANION_MARKER_INVALID) taken before any mutation.
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
// two runtimes serialize against each other mid-campaign (contract C1).
// `merge` adds two more of Node's names, unchanged: "integration-queue" on
// the CONTROL root for every queue write, and lease-store's per-resource
// `lease:<sha256(file)>` for the processor record. It also re-acquires
// 'worktree-admin' for P3 rather than holding one lock end to end — Node
// does, so a single hold would drop a `result: "acquired"` row from
// .bee/logs/contention.jsonl, which is `.bee/` state, not just telemetry. A
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
        // A `.git` that vanished between existsSync and statSync: Node's own
        // throw is V8-worded. Still delegated.
        Resolution::Exotic => return None,
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
        self.emit_code(result, text, 0)
    }

    /// emit() with a handler-supplied `exitCode` (only `worktree merge` uses a
    /// non-zero one). main()'s `recordTiming(!code)` means a non-zero exit
    /// logs `ok: false` even though the result still went to stdout.
    fn emit_code(&self, result: &Value, text: &str, exit: u8) -> ExitCode {
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

    /// `(stderr || stdout || '').trim() || '(no output)'` — the companion
    /// hooks' variant of the fallback chain below (same `||` semantics, a
    /// different tail). Fully deterministic even for a never-launched spawn:
    /// spawnSync leaves every field null there, so both halves fall through to
    /// the literal "(no output)".
    fn no_output_text(&self) -> String {
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
    let (code, text) = node_errno_class(err);
    format!("{code}: {text}, {syscall} '{}'", p(path))
}

/// The errno class behind both formatters. `EPERM` is broken out from `EACCES`
/// because it is the one a companion mount actually hits on win32: creating a
/// directory symlink without SeCreateSymbolicLinkPrivilege fails with
/// ERROR_PRIVILEGE_NOT_HELD (1314), which libuv maps to EPERM, and Rust
/// surfaces it as an uncategorized error rather than PermissionDenied.
fn node_errno_class(err: &std::io::Error) -> (&'static str, &'static str) {
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
fn node_symlink_error_message(err: &std::io::Error, target: &str, path: &Path) -> String {
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
fn js_path_resolve_from_cwd(raw: &str) -> String {
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
fn js_path_join(base: &Path, rel: &str) -> PathBuf {
    if cfg!(windows) {
        base.join(rel.replace('/', "\\"))
    } else {
        base.join(rel)
    }
}

/// `fs.symlinkSync(target, path, 'dir')` — a DIRECTORY symlink on every
/// platform (never a junction: Node only falls back to a junction for
/// `type: 'junction'`, which this call site does not pass).
fn symlink_dir(target: &str, link: &Path) -> std::io::Result<()> {
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
fn unlink_maybe_dir_symlink(path: &Path) {
    if std::fs::remove_file(path).is_err() {
        let _ = std::fs::remove_dir(path);
    }
}

/// `/^gitdir:\s*(.+)$/` applied to an ALREADY-TRIMMED pointer file: no `/m`,
/// and `.` never matches a line terminator, so a multi-line pointer simply
/// does not match. Shared by readWorktreeGitVerifiedId (the forward read) and
/// resolveWorktreeById (the reverse read) so the two can never drift.
fn parse_gitdir_pointer(trimmed: &str) -> Option<&str> {
    trimmed
        .strip_prefix("gitdir:")
        .map(str::trim_start)
        .filter(|rest| !rest.is_empty() && !rest.contains(['\n', '\r', '\u{2028}', '\u{2029}']))
}

/// readWorktreeGitVerifiedId — the AUTHORITATIVE id: git suffixes a colliding
/// basename, so the id is always re-read from the new worktree's own `.git`
/// pointer rather than assumed to be the directory name.
fn read_worktree_git_verified_id(worktree_root: &Path) -> Result<String, String> {
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
fn companion_marker_rel() -> String {
    format!(".bee{MAIN_SEPARATOR}companion-session.json")
}

fn companion_marker_file(worktree_root: &Path) -> PathBuf {
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
fn shell_sync(command: &str, cwd: &Path) -> GitOut {
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
fn js_slice_utf16(s: &str, n: usize) -> String {
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
fn js_replace_first(haystack: &str, needle: &str, replacement: &str) -> String {
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
fn validate_companion_mount_path(mount_path: &str) -> Result<String, CErr> {
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
fn js_path_is_absolute(p: &str) -> bool {
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
/// own `cd` into whatever nested tree it isolates (mirroring `commands.verify`).
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
fn run_companion_start(
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
fn read_companion_marker(worktree_root: &Path) -> Option<Value> {
    let raw = std::fs::read(companion_marker_file(worktree_root)).ok()?;
    let parsed: Value = serde_json::from_slice(&raw).ok()?;
    if js_truthy(&parsed) {
        Some(parsed)
    } else {
        None
    }
}

/// JS truthiness of a parsed JSON value (an absent key is the caller's None).
fn js_truthy(v: &Value) -> bool {
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
fn companion_mount_path(marker: &Value) -> Result<String, MErr> {
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
fn teardown_companion_if_present(
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

// ─── worktree-store.mjs createFeatureWorktree ─────────────────────────────

/// `refuse(code, message)` throws a WorktreeCreateError whose `.message` is
/// `[CODE] message` — bee.mjs's dispatcher surfaces only `.message`, so the
/// bracket prefix is part of every observable byte.
fn refuse(code: &str, message: String) -> CErr {
    CErr::Refuse(format!("[{code}] {message}"))
}

pub(crate) enum CErr {
    /// A shape whose Node bytes embed a V8 message — delegate. Only ever
    /// returned BEFORE `git worktree add` runs (nothing has mutated).
    Ex,
    Refuse(String),
}

pub(crate) struct Created {
    pub(crate) id: String,
    pub(crate) worktree_root: PathBuf,
    pub(crate) branch: String,
    base_ref: Option<String>,
    base_ref_sha: Option<String>,
    bootstrap: Map<String, Value>,
    /// `null`, or runCompanionStart's `{sessionId, worktreePath, mountPath}`.
    companion: Value,
    skills_sync: Value,
}

/// The `--with-companion` pair, both-or-neither. Node passes the two as
/// separate options and re-checks the pairing inside
/// createFeatureWorktreeLocked; this keeps that check reachable (see
/// WORKTREE_COMPANION_CONFIG_INCOMPLETE there) by carrying them as two
/// independent `Option`s rather than collapsing them into one.
#[derive(Default, Clone, Copy)]
pub(crate) struct CompanionSpec<'a> {
    pub(crate) start_command: Option<&'a str>,
    pub(crate) mount_path: Option<&'a str>,
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
/// `companion` carries `--with-companion`'s two config strings (both-or-
/// neither, re-checked below exactly as Node re-checks them): with it present
/// the post-add block also runs `runCompanionStart`, whose failure enters the
/// SAME rollback ladder as any other post-add failure.
pub(crate) fn create_feature_worktree(
    main_root: &Path,
    feature: &str,
    base_ref: Option<&str>,
    companion: CompanionSpec<'_>,
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
    let out = create_feature_worktree_locked(main_root, feature, base_ref, companion);
    guard.release();
    out
}

fn create_feature_worktree_locked(
    main_root: &Path,
    feature: &str,
    base_ref: Option<&str>,
    companion: CompanionSpec<'_>,
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

    // (2) companion both-or-neither. The CLI handler already refuses each
    // half's absence when --with-companion is passed; this is the defensive
    // invariant for any OTHER caller, and it is a zero-mutation refusal.
    // JS truthiness: an empty string counts as absent on both sides.
    let start_command = companion.start_command.filter(|s| !s.is_empty());
    let mount_path_raw = companion.mount_path.filter(|s| !s.is_empty());
    let mut companion_mount: Option<String> = None;
    if start_command.is_some() || mount_path_raw.is_some() {
        let (Some(_), Some(mount)) = (start_command, mount_path_raw) else {
            return Err(refuse(
                "WORKTREE_COMPANION_CONFIG_INCOMPLETE",
                "commands.worktree_companion_start and commands.worktree_companion_mount must both be configured to use --with-companion — only one was found."
                    .to_string(),
            ));
        };
        companion_mount = Some(validate_companion_mount_path(mount)?);
    }

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
        start_command.zip(companion_mount.as_deref()),
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
    companion: Option<(&str, &str)>,
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

    // 10.6 — companion. Deliberately INSIDE this fallible block: a companion
    // start failure folds into the exact same post-add rollback as any other
    // failure after `git worktree add` succeeded, so a worktree is never left
    // created-and-registered but silently missing the companion it asked for.
    let companion = match companion {
        Some((command, mount)) => {
            run_companion_start(main_root, worktree_root, command, mount)?
        }
        None => Value::Null,
    };

    // 10.7 — skills: best-effort, never fatal, never in the ladder.
    let skills_sync = sync_worktree_skills(main_root, worktree_root);

    Ok(Created {
        id,
        worktree_root: worktree_root.to_path_buf(),
        branch: branch.to_string(),
        base_ref: base_ref.filter(|s| !s.is_empty()).map(str::to_string),
        base_ref_sha: base_ref_sha.map(str::to_string),
        bootstrap,
        companion,
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

// ─── worktree-store.mjs mergeFeatureWorktree ──────────────────────────────
//
// The three-phase staged transaction, whole (see the module header for the
// two delegation gates that keep every V8-worded arm out of reach):
//   P1  mergeFeatureWorktreeStage   — LOCKED ('worktree-admin' on mainRoot)
//   P2  runVerifyChild              — UNLOCKED, only when a verify command
//                                     is configured (hardening-4c)
//   P3  mergeFeatureWorktreeFinish  — RE-LOCKED
// Node acquires 'worktree-admin' TWICE on every non-terminal merge (P1 then
// P3) even when no verify runs, so this port does too — a single hold would
// drop one `result: "acquired"` row from .bee/logs/contention.jsonl.

/// `WorktreeMergeError` — `[CODE] message`, the only observable byte.
fn refuse_merge(code: &str, message: String) -> MErr {
    MErr::Thrown(format!("[{code}] {message}"))
}

/// The merge's failure channel. `Thrown` is a message bee.mjs's dispatcher
/// surfaces through emitError AND processAsOwner persists into the queue
/// record's `error` field — every arm that produces one is deterministic.
enum MErr {
    Thrown(String),
    /// Only ever returned BEFORE `git merge --no-ff --no-commit` runs.
    Ex,
}

type MR<T> = Result<T, MErr>;

/// The merge's own `{ok, ...}` answer: the result object plus the exit code
/// bee.mjs derives from it and the queue status processAsOwner writes.
struct MergeAnswer {
    result: Map<String, Value>,
    ok: bool,
}

/// gitStatusPorcelain — deliberately WITHOUT `--ignored` (decision D8a). A
/// git failure is a plain (untyped) Error whose bytes are still fully
/// deterministic, including the literal "exit null" a never-launched spawn
/// renders.
fn git_status_porcelain(cwd: &Path) -> Result<String, String> {
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

fn is_tree_dirty(cwd: &Path) -> Result<bool, String> {
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
fn git_status_porcelain_excluding(cwd: &Path, exclude_paths: &[String]) -> Result<String, String> {
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

fn is_tree_dirty_excluding(cwd: &Path, exclude_paths: &[String]) -> Result<bool, String> {
    Ok(!js_trim(&git_status_porcelain_excluding(cwd, exclude_paths)?).is_empty())
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
fn main_untouched_proof(main_root: &Path, pre_merge_head: &str, merge_head_file: &Path) -> Result<(), String> {
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
fn current_branch(cwd: &Path) -> Option<String> {
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
fn read_json_feature(file: &Path) -> Option<String> {
    let raw = std::fs::read(file).ok()?;
    let parsed: Value = serde_json::from_slice(&raw).ok()?;
    match parsed.get("feature") {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

struct WorktreeFeature {
    feature: Option<String>,
    created: Option<String>,
    state_feature: Option<String>,
}

/// resolveWorktreeFeature — the IMMUTABLE creation slug wins over the mutable
/// `state.feature` (issues-46-53 D4), degrading exactly to the pre-fix
/// behavior when no creation record exists.
fn resolve_worktree_feature(worktree_root: &Path) -> WorktreeFeature {
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
fn wt_branch_shaped(branch: &str) -> bool {
    branch.strip_prefix("wt/").is_some_and(feature_slug_ok)
}

/// resolveWorktreeById — the same BIDIRECTIONAL gitdir validation
/// resolveRoots uses, keyed by id instead of by walking up from a cwd. `None`
/// on ANY mismatch, missing file or unreadable content, so "no such id" and
/// "id's link is broken" fold into one typed refusal.
///
/// The reverse comparison goes through path_identity's `canonical_paths_equal`
/// (windows-path-identity wpi-1), NOT a byte compare — the shared fix.
fn resolve_worktree_by_id(main_root: &Path, id: &str) -> Option<PathBuf> {
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
fn release_all_for_holder(main_root: &Path, id: &str) {
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

/// performCleanup (decision D8b): re-check freshness, `git worktree remove
/// --force`, `git branch -d` (NEVER -D), then the three best-effort ledger
/// drops. Never throws — every outcome is the `{ok, code?}` object folded into
/// the merge result's `.cleanup` field, in Node's exact key order.
fn perform_cleanup(
    main_root: &Path,
    worktree_root: &Path,
    branch: &str,
    id: &str,
    verify_skipped: bool,
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

    let worktree_root_s = p(worktree_root);
    let remove_result = run_git(
        main_root,
        &["worktree", "remove", "--force", "--", &worktree_root_s],
    );
    if remove_result.status != Some(0) {
        out.insert("ok".into(), Value::Bool(false));
        out.insert("code".into(), json!("WORKTREE_MERGE_CLEANUP_REMOVE_FAILED"));
        out.insert("reason".into(), Value::String(remove_result.fail_text()));
        return out;
    }

    let branch_delete = run_git(main_root, &["branch", "-d", "--", branch]);
    if branch_delete.status != Some(0) {
        out.insert("ok".into(), Value::Bool(false));
        out.insert(
            "code".into(),
            json!("WORKTREE_MERGE_CLEANUP_BRANCH_DELETE_FAILED"),
        );
        out.insert("removed".into(), Value::Bool(true));
        out.insert("reason".into(), Value::String(branch_delete.fail_text()));
        return out;
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

    out.insert("ok".into(), Value::Bool(true));
    out.insert("removed".into(), Value::Bool(true));
    out.insert("branch_deleted".into(), Value::Bool(true));
    if verify_skipped {
        out.insert(
            "warning".into(),
            json!("verify skipped — no commands.verify recorded; cleaned up unchecked."),
        );
    }
    out
}

/// attachCleanupOutcome — runs cleanup, or attaches the suggested command
/// (decision D8b: "never prompt").
fn attach_cleanup_outcome(
    result: &mut Map<String, Value>,
    main_root: &Path,
    worktree_root: &Path,
    branch: &str,
    id: &str,
    cleanup: bool,
    verify_skipped: bool,
) {
    if !cleanup {
        result.insert(
            "cleanup_suggested_command".into(),
            json!(format!("bee worktree merge --id {id} --cleanup --json")),
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
            verify_skipped,
        )),
    );
}

/// checkMergeFence — P3's SECOND line of defense (the processor-lease epoch is
/// the first). A short drift description, or `None` when the fence is clean.
/// The two `.stdout.trim()` sites here carry the same unreachability argument
/// `main_untouched_proof` documents.
fn check_merge_fence(
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

// ─── the verify child (P2) ────────────────────────────────────────────────

/// Node's `spawn(command, { shell: true })` file/args, faithfully: on win32
/// `process.env.comspec || 'cmd.exe'` with `/d /s /c "<command>"` passed
/// VERBATIM; elsewhere `/bin/sh -c <command>`. Deliberately NOT
/// verbs/cells.rs's `spawn_declared`, which prefers Git Bash on win32 — that
/// is `runDeclaredTests`' shape, not `runVerifyChild`'s.
fn shell_child(command: &str) -> std::process::Command {
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

/// The pre-check that retires blocker (b). `runVerifyChild`'s ONLY V8/libuv
/// byte is the `error` event's message (`spawn cmd.exe ENOENT`,
/// `spawn /bin/sh ENOENT`), concatenated into `verifyOutcome.combined` and
/// surfaced verbatim in MERGE_VERIFY_RED's `output_tail` — a byte reached
/// AFTER the merge is staged, where nothing can fall back. That event fires
/// only when the SHELL ITSELF cannot be started, so probing the shell BEFORE
/// P1 ever stages anything makes the arm unreachable: a failed probe returns
/// None with zero mutations and zero locks taken, and a passing probe proves
/// the real spawn will launch.
fn shell_launchable() -> bool {
    shell_child("exit 0")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

struct VerifyOutcome {
    ran: bool,
    status: Option<i32>,
    combined: String,
}

/// runVerifyChild — the async-`spawn` verify, run UNLOCKED against the
/// merged-but-UNCOMMITTED tree, with `on_tick` firing on `tick_interval_ms`
/// for as long as the child is still running (integration-queue's processor-
/// lease renewal in production). A throwing tick is swallowed.
///
/// ONE DOCUMENTED DIVERGENCE, in the same class as verbs/state_group.rs's
/// prune approximation: Node resolves on the child's `exit` event, so output
/// still sitting in a pipe when the process exits can be LOST; this port joins
/// its reader threads, so it always captures the full stream. The difference
/// is observable only in a race Node does not reproduce run-to-run, and it can
/// only ever ADD trailing bytes to `output_tail` — never change `status`, the
/// red/green verdict, or any `.bee/` record.
fn run_verify_child(
    command: &str,
    cwd: &Path,
    on_tick: &dyn Fn(),
    tick_interval_ms: f64,
) -> VerifyOutcome {
    use std::io::Read;
    use std::sync::mpsc;

    let mut child = match shell_child(command)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        // Unreachable after `shell_launchable` (see its doc comment); if a
        // race gets here anyway, Node's `status: null` verdict is reproduced
        // and the error text is Rust's — the same narrow approximation
        // node_fs_error_message already makes elsewhere in this file.
        Err(e) => {
            return VerifyOutcome {
                ran: true,
                status: None,
                combined: format!("{e}"),
            }
        }
    };

    let drain = |mut pipe: Option<std::process::ChildStdout>| {
        let (tx, rx) = mpsc::channel::<String>();
        let handle = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(pipe) = pipe.as_mut() {
                let _ = pipe.read_to_end(&mut buf);
            }
            let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
        });
        (handle, rx)
    };
    let (out_handle, out_rx) = drain(child.stdout.take());
    let (err_handle, err_rx) = {
        let mut pipe = child.stderr.take();
        let (tx, rx) = mpsc::channel::<String>();
        let handle = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(pipe) = pipe.as_mut() {
                let _ = pipe.read_to_end(&mut buf);
            }
            let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
        });
        (handle, rx)
    };

    // `setInterval(tickIntervalMs)` for as long as the child is running. The
    // .mjs unref()s the timer so it can never keep the process alive; here the
    // poll is inline, so there is nothing to unref.
    let interval = std::time::Duration::from_millis(tick_interval_ms.max(1.0) as u64);
    let mut last_tick = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s.code(),
            Ok(None) => {}
            Err(_) => break None,
        }
        if last_tick.elapsed() >= interval {
            last_tick = Instant::now();
            on_tick();
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    };
    let stdout = out_rx.recv().unwrap_or_default();
    let stderr = err_rx.recv().unwrap_or_default();
    let _ = out_handle.join();
    let _ = err_handle.join();
    VerifyOutcome {
        ran: true,
        status,
        combined: format!("{stdout}{stderr}"),
    }
}

// ─── P1 / P3 ──────────────────────────────────────────────────────────────

/// Everything P3 needs, carried across the released lock.
struct Staged {
    id: String,
    branch: String,
    worktree_root: PathBuf,
    merge_message: String,
    pre_merge_head: String,
    merge_head_file: PathBuf,
    staged_tree_hash: String,
    /// teardownCompanionIfPresent's `{ended, sessionId, warning?}`, carried
    /// across the released lock so P3's results can spread it too.
    companion: Option<Value>,
}

enum StageOut {
    /// A TERMINAL outcome already fully resolved inside the P1 lock.
    Done(MergeAnswer),
    Staged(Box<Staged>),
}

#[allow(clippy::too_many_arguments)]
fn merge_stage(
    main_root: &Path,
    id: &str,
    cleanup: bool,
    companion_end_command: Option<&str>,
) -> MR<StageOut> {
    // `typeof id !== 'string' || !id` is already enforced by run_merge's
    // requireFlag gate, so WORKTREE_MERGE_INVALID_ID is unreachable here.
    if !is_ordinary_checkout(main_root) {
        return Err(refuse_merge(
            "WORKTREE_MERGE_CALLER_NOT_ORDINARY",
            format!(
                "\"bee worktree merge\" must be run from the MAIN checkout, not a linked worktree ({} is not an ordinary checkout) — a worktree, including the one being merged, cannot merge itself.",
                p(main_root)
            ),
        ));
    }

    let main_store_root = main_root.join(".bee");
    let grants = read_grants_strict(&main_store_root).ok_or(MErr::Ex)?;
    let id_json = jsjson::stringify(&Value::String(id.to_string()));
    if grants.get(id) != Some(&Value::Bool(true)) {
        return Err(refuse_merge(
            "WORKTREE_MERGE_UNKNOWN_ID",
            format!(
                "no granted worktree found for id {id_json} — run \"bee worktree list\" to see granted ids."
            ),
        ));
    }

    let resolved = resolve_worktree_by_id(main_root, id).filter(|r| r.exists());
    let Some(worktree_root) = resolved else {
        return Err(refuse_merge(
            "WORKTREE_MERGE_UNKNOWN_ID",
            format!(
                "id {id_json} is granted but no matching, bidirectionally-valid git worktree link was found under {} (or the worktree no longer exists on disk) — run \"git worktree prune\" and \"bee worktree unregister --id {id}\" if it was removed by hand.",
                p(main_root)
            ),
        ));
    };

    // worktree-companion-hook: READ (never delete) the marker up front so its
    // mountPath can be excluded from the worktree dirty-check right below via
    // a git pathspec. Actual teardown is deferred until every zero-mutation
    // refusal in this function has cleared — see teardown_companion_if_present
    // for the full ordering rationale.
    let companion_marker = read_companion_marker(&worktree_root);

    if is_tree_dirty(main_root).map_err(MErr::Thrown)? {
        return Err(refuse_merge(
            "WORKTREE_MERGE_MAIN_DIRTY",
            format!(
                "the MAIN checkout at {} has uncommitted changes (\"git status --porcelain\" is non-empty) — commit or stash before merging.",
                p(main_root)
            ),
        ));
    }
    // A present companion mount AND its marker file are both untracked (and
    // the marker, unlike the rest of a bootstrapped `.bee` store, is not
    // gitignored either) — either alone would trip this check, so both are
    // excluded by git pathspec rather than by deletion-before-check.
    let worktree_dirty = match &companion_marker {
        Some(marker) => is_tree_dirty_excluding(
            &worktree_root,
            &[companion_mount_path(marker)?, companion_marker_rel()],
        ),
        None => is_tree_dirty(&worktree_root),
    };
    if worktree_dirty.map_err(MErr::Thrown)? {
        return Err(refuse_merge(
            "WORKTREE_MERGE_WORKTREE_DIRTY",
            format!(
                "the worktree at {} has uncommitted changes (\"git status --porcelain\" is non-empty) — commit or stash before merging. (A bootstrapped, gitignored .bee store alone is NOT dirty, per decision D8a.)",
                p(&worktree_root)
            ),
        ));
    }

    let Some(branch) = current_branch(&worktree_root) else {
        return Err(refuse_merge(
            "WORKTREE_MERGE_DETACHED_HEAD",
            format!(
                "the worktree at {} is on a detached HEAD — check out its branch before merging.",
                p(&worktree_root)
            ),
        ));
    };

    let identity = resolve_worktree_feature(&worktree_root);
    let expected_branch = identity.feature.as_ref().map(|f| format!("wt/{f}"));
    let branch_ok = match &expected_branch {
        Some(expected) => branch == *expected,
        None => wt_branch_shaped(&branch),
    };
    if !branch_ok {
        // issues-46-53 D4 (#46): name the field that actually drifted.
        let expected_disp = expected_branch.clone().unwrap_or_default();
        let drift = match (&identity.created, &identity.state_feature) {
            (Some(created), Some(state)) if created != state => format!(
                " This worktree was CREATED as feature {} (its immutable creation slug, which \"{expected_disp}\" comes from), while its .bee/state.json now records feature {} — the FEATURE FIELD drifted after creation (a rename, \"bee state set --feature\", or a new feature started in this worktree); the branch did not. Do NOT rename the branch to match: check \"{expected_disp}\" back out in the worktree, or merge the branch you actually want by hand.",
                jsjson::stringify(&Value::String(created.clone())),
                jsjson::stringify(&Value::String(state.clone())),
            ),
            (None, Some(state)) => format!(
                " \"{expected_disp}\" is derived from this worktree's MUTABLE .bee/state.json \"feature\" field ({}) because the worktree predates bee's immutable creation-slug record — if the feature was renamed after the worktree was created, that FIELD is what drifted, not the branch. The branch name is fixed at creation; do not rename it to match.",
                jsjson::stringify(&Value::String(state.clone())),
            ),
            _ => String::new(),
        };
        let expected_phrase = match &expected_branch {
            Some(e) => format!("\"{e}\""),
            None => "\"wt/<slug>\"-style".to_string(),
        };
        return Err(refuse_merge(
            "WORKTREE_MERGE_BRANCH_MISMATCH",
            format!(
                "the worktree at {} is checked out to \"{branch}\", not its expected {expected_phrase} branch — merge refuses to guess which branch to consume.{drift}",
                p(&worktree_root)
            ),
        ));
    }

    // worktree-companion-hook: every zero-mutation refusal above (both
    // dirty-tree checks, detached-HEAD, branch-mismatch) has now cleared, so
    // it is safe to tear the companion down. It cannot run any earlier (that
    // would destroy the mount even for a merge about to be refused) or any
    // later (the companion session must not outlive a merge attempt that is
    // actually proceeding). On a COMPANION worktree this — not the staged
    // merge — is the first real mutation, so NOTHING from this line on may
    // return MErr::Ex; run_merge's read-only probes are what keep that true.
    let companion = teardown_companion_if_present(
        main_root,
        &worktree_root,
        companion_end_command,
        companion_marker.as_ref(),
    );

    // ── every REFUSAL above is zero-mutation; the staged merge below is the
    // first write to MAIN. ────────────────────────────────────────────────
    let pre_merge_head =
        js_trim(&run_git(main_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
    let merge_head_file = main_root.join(".git").join("MERGE_HEAD");
    let merge_message = format!("Merge worktree {id} (branch {branch}) via bee worktree merge");

    let merge_result = run_git(main_root, &["merge", "--no-ff", "--no-commit", "--", &branch]);
    if merge_result.status != Some(0) {
        run_git(main_root, &["merge", "--abort"]);
        if let Err(reason) = main_untouched_proof(main_root, &pre_merge_head, &merge_head_file) {
            return Err(refuse_merge(
                "WORKTREE_MERGE_ABORT_FAILED",
                format!(
                    "\"git merge --no-ff --no-commit {branch}\" failed and \"git merge --abort\" did NOT fully restore {} to its pre-merge state ({reason}) — main may be left mid-merge; inspect it by hand before retrying.",
                    p(main_root)
                ),
            ));
        }
        let mut result = Map::new();
        result.insert("ok".into(), Value::Bool(false));
        result.insert("code".into(), json!("MERGE_CONFLICT"));
        result.insert("id".into(), json!(id));
        result.insert("branch".into(), json!(branch));
        result.insert("worktreeRoot".into(), json!(p(&worktree_root)));
        result.insert("message".into(), json!(format!(
            "\"git merge --no-ff {branch}\" hit a textual conflict — the merge was aborted and {} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); bee does not auto-resolve a textual conflict.",
            p(main_root)
        )));
        result.insert("output".into(), json!(format!(
            "{}{}",
            merge_result.stdout.clone().unwrap_or_default(),
            merge_result.stderr.clone().unwrap_or_default()
        )));
        // `...(companion ? { companion } : {})` — last, after `output`.
        if let Some(companion) = companion {
            result.insert("companion".into(), companion);
        }
        return Ok(StageOut::Done(MergeAnswer { result, ok: false }));
    }

    if !merge_head_file.exists() {
        // Zero exit but nothing staged: "Already up to date".
        let mut result = Map::new();
        result.insert("ok".into(), Value::Bool(true));
        result.insert("merged".into(), Value::Bool(false));
        result.insert("id".into(), json!(id));
        result.insert("branch".into(), json!(branch));
        result.insert("worktreeRoot".into(), json!(p(&worktree_root)));
        result.insert("code".into(), json!("ALREADY_UP_TO_DATE"));
        result.insert("verify".into(), json!("skipped"));
        result.insert("message".into(), json!(format!(
            "\"{branch}\" is already up to date with {} — nothing to merge.",
            p(main_root)
        )));
        // `...(companion ? { companion } : {})` — after `message`, BEFORE the
        // cleanup keys attachCleanupOutcome appends next.
        if let Some(companion) = companion {
            result.insert("companion".into(), companion);
        }
        // verifySkipped is deliberately FALSE here (see the .mjs comment).
        attach_cleanup_outcome(
            &mut result,
            main_root,
            &worktree_root,
            &branch,
            id,
            cleanup,
            false,
        );
        return Ok(StageOut::Done(MergeAnswer { result, ok: true }));
    }

    let staged_tree_hash =
        js_trim(&run_git(main_root, &["write-tree"]).stdout.unwrap_or_default()).to_string();
    Ok(StageOut::Staged(Box::new(Staged {
        id: id.to_string(),
        branch,
        worktree_root,
        merge_message,
        pre_merge_head,
        merge_head_file,
        staged_tree_hash,
        companion,
    })))
}

/// P3: verify-red first, then the two-line fence, then commit + post-commit
/// guard + cleanup. `lease_drift` is the caller-supplied FIRST fence line
/// (integration-queue's `checkProcessorLeaseEpoch`), evaluated here so it runs
/// inside the re-acquired hold exactly as the .mjs's `await
/// checkProcessorLease()` does.
fn merge_finish(
    main_root: &Path,
    state: &Staged,
    cleanup: bool,
    verify: &VerifyOutcome,
    lease_fence: &dyn Fn() -> Option<String>,
) -> MR<MergeAnswer> {
    let Staged {
        id,
        branch,
        worktree_root,
        merge_message,
        pre_merge_head,
        merge_head_file,
        staged_tree_hash,
        companion,
    } = state;

    let mut committed = false;
    let outcome = (|| -> MR<MergeAnswer> {
        if verify.ran && verify.status != Some(0) {
            let lines: Vec<&str> = verify.combined.split('\n').collect();
            let tail = lines[lines.len().saturating_sub(30)..].join("\n");
            run_git(main_root, &["merge", "--abort"]);
            if let Err(reason) = main_untouched_proof(main_root, pre_merge_head, merge_head_file) {
                return Err(refuse_merge(
                    "WORKTREE_MERGE_ABORT_FAILED",
                    format!(
                        "verify failed and \"git merge --abort\" did NOT fully restore {} to its pre-merge state ({reason}) — main may be left mid-merge; inspect it by hand before retrying.",
                        p(main_root)
                    ),
                ));
            }
            let mut result = Map::new();
            result.insert("ok".into(), Value::Bool(false));
            result.insert("code".into(), json!("MERGE_VERIFY_RED"));
            result.insert("id".into(), json!(id));
            result.insert("branch".into(), json!(branch));
            result.insert("worktreeRoot".into(), json!(p(worktree_root)));
            result.insert("merged".into(), Value::Bool(false));
            result.insert("verify".into(), json!("red"));
            result.insert("message".into(), json!(format!(
                "the merge was textually clean but the post-merge verify failed against the merged-but-uncommitted tree — this is the semantic-conflict alarm: behavior broke even though git found no textual conflict. The merge was aborted and {} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); no merge commit exists. Fix-first before release.",
                p(main_root)
            )));
            result.insert("output_tail".into(), Value::String(tail));
            // `...(companion ? { companion } : {})` — last, after output_tail.
            if let Some(companion) = companion {
                result.insert("companion".into(), companion.clone());
            }
            return Ok(MergeAnswer { result, ok: false });
        }

        // The P3 fence: the processor-lease epoch is the FIRST line, and
        // `||` short-circuits, so checkMergeFence's git reads never run when
        // the lease already drifted.
        let fence_drift = lease_fence().or_else(|| {
            check_merge_fence(main_root, id, pre_merge_head, merge_head_file, staged_tree_hash)
        });
        if let Some(fence_drift) = fence_drift {
            run_git(main_root, &["merge", "--abort"]);
            if let Err(reason) = main_untouched_proof(main_root, pre_merge_head, merge_head_file) {
                return Err(refuse_merge(
                    "WORKTREE_MERGE_ABORT_FAILED",
                    format!(
                        "the P3 fence detected drift ({fence_drift}) and \"git merge --abort\" did NOT fully restore {} to its pre-merge state ({reason}) — main may be left mid-merge; inspect it by hand before retrying.",
                        p(main_root)
                    ),
                ));
            }
            return Err(refuse_merge(
                "WORKTREE_MERGE_FENCE_DRIFT",
                format!(
                    "the staged merge was aborted before commit because the P3 re-check (advisor condition C2) detected drift while the 'worktree-admin' lock was released around the verify child: {fence_drift}. {} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); no merge commit exists.",
                    p(main_root)
                ),
            ));
        }

        let commit_result = run_git(main_root, &["commit", "-m", merge_message]);
        if commit_result.status != Some(0) {
            run_git(main_root, &["merge", "--abort"]);
            return Err(refuse_merge(
                "WORKTREE_MERGE_COMMIT_FAILED",
                format!(
                    "\"git commit\" failed for the staged merge of {branch} ({}) — the staged merge was aborted; {} was left untouched.",
                    commit_result.fail_text(),
                    p(main_root)
                ),
            ));
        }
        committed = true;

        let mut result = Map::new();
        result.insert("ok".into(), Value::Bool(true));
        result.insert("merged".into(), Value::Bool(true));
        result.insert("id".into(), json!(id));
        result.insert("branch".into(), json!(branch));
        result.insert("worktreeRoot".into(), json!(p(worktree_root)));
        let verify_field = if verify.ran { "green" } else { "skipped" };
        result.insert("verify".into(), json!(verify_field));
        // `...(companion ? { companion } : {})` — after `verify`, BEFORE the
        // post-commit `warning` and the cleanup keys.
        if let Some(companion) = companion {
            result.insert("companion".into(), companion.clone());
        }

        // Post-commit guard (D2-REVISED).
        let post_commit_status = run_git(
            main_root,
            &["status", "--porcelain", "--untracked-files=no"],
        )
        .stdout
        .unwrap_or_default();
        if !js_trim(&post_commit_status).is_empty() {
            result.insert("warning".into(), json!({
                "code": "verify_mutated_tracked_files",
                "message": "the post-merge verify command left tracked files modified after the merge commit landed (\"git status --porcelain --untracked-files=no\" is non-empty) — the merge commit itself is clean, but verify mutated the working tree afterward; inspect and commit/discard those changes separately. Recovery if a LATER independent verify goes red on this merge: \"git revert -m 1 <merge-commit>\".",
                "status": post_commit_status,
            }));
        }

        attach_cleanup_outcome(
            &mut result,
            main_root,
            worktree_root,
            branch,
            id,
            cleanup,
            verify_field == "skipped",
        );
        Ok(MergeAnswer { result, ok: true })
    })();

    // The `finally` safety net: an unexpected exit that never committed and
    // never aborted must not strand a staged merge on main.
    if !committed && merge_head_file.exists() {
        run_git(main_root, &["merge", "--abort"]);
    }
    outcome
}

/// mergeFeatureWorktree — P1 / (P2) / P3 with the lock released across the
/// verify child. `Err(MErr::Ex)` is only ever produced by P1, before any
/// mutation; the caller has already taken the queue lock by then, so it is the
/// documented late-delegation residual, never an ordinary shape.
fn merge_feature_worktree(
    main_root: &Path,
    id: &str,
    cleanup: bool,
    verify_command: Option<&str>,
    companion_end_command: Option<&str>,
    hooks: Option<&crate::integration_queue::Hooks<'_>>,
) -> MR<MergeAnswer> {
    let mut guard = lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
        .map_err(|b| MErr::Thrown(b.message()))?;
    let staged = merge_stage(main_root, id, cleanup, companion_end_command);
    guard.release();
    let staged = match staged? {
        StageOut::Done(answer) => return Ok(answer),
        StageOut::Staged(s) => s,
    };

    let no_lease_drift = || None;
    let lease_fence: &dyn Fn() -> Option<String> = match hooks {
        Some(h) => &move || h.check_processor_lease(),
        None => &no_lease_drift,
    };

    let Some(command) = verify_command else {
        // Nothing to release the lock around — stage and finish inside the
        // SAME shape as the pre-hardening-4c single-lock behavior, which is
        // still TWO acquires (the .mjs re-enters withStoreLock here).
        let mut guard = lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
            .map_err(|b| MErr::Thrown(b.message()))?;
        let out = merge_finish(
            main_root,
            &staged,
            cleanup,
            &VerifyOutcome { ran: false, status: None, combined: String::new() },
            lease_fence,
        );
        guard.release();
        return out;
    };

    // P2 — UNLOCKED.
    let tick_ms = hooks
        .map(|h| h.verify_tick_interval_ms)
        .unwrap_or(crate::integration_queue::DEFAULT_RENEW_INTERVAL_MS);
    let no_tick = || {};
    let tick: &dyn Fn() = match hooks {
        Some(h) => &move || h.on_verify_tick(),
        None => &no_tick,
    };
    let verify = run_verify_child(command, main_root, tick, tick_ms);

    // P3 — re-acquire and re-check the fence before ever committing.
    let mut guard = lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
        .map_err(|b| MErr::Thrown(b.message()))?;
    let out = merge_finish(main_root, &staged, cleanup, &verify, lease_fence);
    guard.release();
    out
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
    let with_companion = bool_flag_true(&flags, "with-companion");
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

    // wcg-3 (D1a/D3/D4): the shared-nested-checkout guard, whole. Fires BEFORE
    // any mutation, hard fail-closed with no override (D3), and teaches the
    // paved road (D4 — a NEW companion-mounted worktree, never an in-place
    // conversion). Both of its shapes now run natively: the marker-verified
    // companion mount comes from THE write guard's own
    // `resolveVerifiedCompanionMountReal` and the nested-`.git` down-scan from
    // crate::nested_checkout (which is the only thing that module holds — see
    // its header for why the predicates are imported rather than re-derived).
    //
    // `!withCompanion && …` short-circuits in the .mjs, so `--with-companion`
    // never even runs the scan: the guard exists to push a caller toward that
    // flag, and is never a refusal for someone who already passed it.
    //
    // A solo checkout stays a pure no-op (D6): nobody else live, no scan.
    let main_root_s = p(&main_root);
    let ctrl_root = crate::verbs::reservations::control_root_for(&main_root_s).ok()?;
    let session_id = crate::verbs::reservations::resolve_session_id(None, &ctrl_root).ok()?;
    let shared_nested_found = if with_companion {
        false
    } else {
        match crate::nested_checkout::has_any_shared_nested_checkout(
            &main_root,
            &ctrl_root,
            session_id.as_deref(),
        ) {
            Ok(found) => found,
            // The detection check itself errored. Node's message interpolates
            // the caught error's V8 `.message`; this port supplies its own
            // deterministic reason in the same slot (crate::nested_checkout's
            // header documents the divergence). Everything else — fail CLOSED,
            // zero mutation, same wording, same exit — is Node's.
            Err(detect) => {
                return Some(ctx.fail(&format!(
                    "refusing to create a worktree: could not determine whether {main_root_s} holds a shared nested checkout another live session could reach — the detection check itself errored ({}). This guard fails CLOSED on a detection error rather than risk silently allowing an unguarded worktree. FIX: resolve the underlying filesystem error, then retry.",
                    detect.reason
                )))
            }
        }
    };
    if shared_nested_found {
        return Some(ctx.fail(&format!(
            "refusing to create a worktree: another session is concurrently live on {main_root_s} and it contains a shared nested checkout a companion mount must cover — running unguarded is how one session silently ate another's work. Re-run with \"bee worktree new --feature {feature} --with-companion\" so the shared checkout is mounted and tracked (the paved road for concurrent shared-checkout work — AGENTS.md rule 13). This creates a NEW companion-mounted worktree; it does not retrofit the checkout you are in."
        )));
    }

    // worktree-companion-hook: resolved HERE from readConfig(mainRoot).commands
    // and passed down as plain strings (worktree-store.mjs stays zero-deps),
    // and refused HERE — before any worktree is created — rather than surfacing
    // later as a confusing symlink failure.
    let mut companion_start: Option<String> = None;
    let mut companion_mount: Option<String> = None;
    if with_companion {
        let commands = read_worktree_commands(&main_root)?; // corrupt config -> Node
        companion_start = commands.companion_start;
        companion_mount = commands.companion_mount;
        if companion_start.is_none() {
            return Some(ctx.fail(
                "--with-companion requires commands.worktree_companion_start to be set in .bee/config.json.",
            ));
        }
        if companion_mount.is_none() {
            return Some(ctx.fail(
                "--with-companion requires commands.worktree_companion_mount to be set in .bee/config.json.",
            ));
        }
    }

    let mut lock_busy: Option<String> = None;
    let created = match create_feature_worktree(
        &main_root,
        &feature,
        base_ref.as_deref(),
        CompanionSpec {
            start_command: companion_start.as_deref(),
            mount_path: companion_mount.as_deref(),
        },
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
    // `companion: created.companion || null`.
    result.insert(
        "companion".into(),
        if js_truthy(&created.companion) {
            created.companion.clone()
        } else {
            Value::Null
        },
    );
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
    // `.filter((line) => line !== null)` — the companion line is present only
    // when a companion was actually mounted.
    let mut lines = vec![
        format!(
            "Created worktree for feature \"{feature}\": {}",
            p(&created.worktree_root)
        ),
        branch_line,
        bootstrap_line,
        skills_line,
    ];
    if js_truthy(&created.companion) {
        let field = |k: &str| {
            created
                .companion
                .get(k)
                .map(jsjson::js_to_string)
                .unwrap_or_default()
        };
        let session = match created.companion.get("sessionId") {
            Some(v) if js_truthy(v) => format!(", session {}", jsjson::js_to_string(v)),
            _ => String::new(),
        };
        lines.push(format!(
            "  companion:   mounted at {} ({}{session}).",
            field("mountPath"),
            field("worktreePath")
        ));
    }
    lines.push(next_step);
    let text = lines.join("\n");
    Some(ctx.emit(&Value::Object(result), &text))
}

/// bee.mjs's `WORKTREE_MERGE_SESSIONLESS_ID` (multisession-native-22): merge
/// has never required session identity to run solo, and lease-store only needs
/// SOME non-empty session_id string.
const WORKTREE_MERGE_SESSIONLESS_ID: &str = "bee-worktree-merge-sessionless";

/// readConfig(mainRoot).commands, narrowed to the five keys the worktree verbs
/// read. normalizeCommands trims every string value, drops empties, and keeps
/// `test`'s ARRAY shape distinct from its string shape — which matters,
/// because merge's verify fallback is `typeof commands.test === 'string'`, so
/// an array `test` is never spawned as one shell command.
struct WorktreeCommands {
    verify: Option<String>,
    test_string: Option<String>,
    companion_start: Option<String>,
    companion_mount: Option<String>,
    companion_end: Option<String>,
}

fn read_worktree_commands(main_root: &Path) -> Option<WorktreeCommands> {
    let config = crate::state::read_config_raw(main_root).ok()?;
    let mut out = WorktreeCommands {
        verify: None,
        test_string: None,
        companion_start: None,
        companion_mount: None,
        companion_end: None,
    };
    let Some(Value::Object(raw)) = config.get("commands") else {
        return Some(out);
    };
    let trimmed = |key: &str| -> Option<String> {
        match raw.get(key) {
            Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
            _ => None,
        }
    };
    out.verify = trimmed("verify");
    out.test_string = trimmed("test");
    out.companion_start = trimmed("worktree_companion_start");
    out.companion_mount = trimmed("worktree_companion_mount");
    out.companion_end = trimmed("worktree_companion_end");
    Some(out)
}

/// `Number(string)` — ECMA-262 StringNumericLiteral, whole: leading/trailing
/// whitespace stripped, an empty (or all-whitespace) string is 0, `Infinity`
/// with an optional sign, the 0x/0o/0b integer literals (no sign allowed on
/// those), and the ordinary decimal grammar with an optional exponent.
/// Anything else is NaN.
///
/// This is the FULL conversion, unlike verbs/reservations.rs's
/// `js_number_flag`, which models the same grammar but returns
/// `Number.parseInt`'s answer for its own call sites.
fn js_string_to_number(raw: &str) -> f64 {
    let t = js_trim(raw);
    if t.is_empty() {
        return 0.0; // Number('') === 0, Number('   ') === 0
    }
    // Radix literals: no sign, at least one digit.
    if let Some(rest) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return radix_value(rest, 16);
    }
    if let Some(rest) = t.strip_prefix("0o").or_else(|| t.strip_prefix("0O")) {
        return radix_value(rest, 8);
    }
    if let Some(rest) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
        return radix_value(rest, 2);
    }
    let (sign, body) = match t.strip_prefix('-') {
        Some(rest) => (-1.0, rest),
        None => (1.0, t.strip_prefix('+').unwrap_or(t)),
    };
    if body == "Infinity" {
        return sign * f64::INFINITY;
    }
    // [digits][.[digits]] | .digits, then optional [eE][+-]digits.
    let bytes = body.as_bytes();
    let mut i = 0usize;
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let int_len = i - int_start;
    let mut frac_len = 0usize;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let fs = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        frac_len = i - fs;
    }
    if int_len == 0 && frac_len == 0 {
        return f64::NAN;
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        let es = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return f64::NAN;
        }
    }
    if i != bytes.len() {
        return f64::NAN;
    }
    // Rust's f64 parser accepts exactly this grammar and rounds the same way
    // (both are correctly-rounded IEEE-754); an out-of-range literal saturates
    // to ±Infinity in both.
    body.parse::<f64>().map(|v| sign * v).unwrap_or(f64::NAN)
}

fn radix_value(digits: &str, radix: u32) -> f64 {
    if digits.is_empty() {
        return f64::NAN;
    }
    let mut acc = 0.0f64;
    for c in digits.chars() {
        match c.to_digit(radix) {
            Some(d) => acc = acc * f64::from(radix) + f64::from(d),
            None => return f64::NAN,
        }
    }
    acc
}

fn run_merge(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["id", "cleanup", "queue-wait-ms"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "cleanup") {
        return None;
    }
    // `--queue-wait-ms`: a registry `type:"number"` flag. validate() runs
    // first (typeMatches: a non-empty string whose `Number(...)` is FINITE),
    // then the handler keeps the value only when it is also POSITIVE —
    // anything else silently keeps DEFAULT_WAIT_BOUND_MS rather than refusing.
    let mut queue_wait_bound_ms = crate::integration_queue::DEFAULT_WAIT_BOUND_MS;
    match flags.get("queue-wait-ms") {
        None => {}
        // A bare `--queue-wait-ms` parses to `true`, which fails
        // typeMatches('number') — the dispatcher's own generic validate()
        // refusal, shared by every verb and not this flag's arm; it is the one
        // shape here that still returns before any output.
        Some(FlagV::Present) => return None,
        Some(FlagV::S(raw)) => {
            if js_trim(raw).is_empty() {
                return None; // validate(): `value.trim() !== ''`
            }
            let n = js_string_to_number(raw);
            if !n.is_finite() {
                return None; // validate(): `Number.isFinite(Number(value))`
            }
            if n > 0.0 {
                queue_wait_bound_ms = n;
            }
        }
    }
    let id = match flags.get("id") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    let cleanup = bool_flag_true(&flags, "cleanup");

    let ctx = match prelude("worktree merge", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&format!(
            "\"bee worktree merge\" must be run from inside the main checkout, not a \"{}\" checkout — a worktree, including the one being merged, cannot merge itself.",
            ctx.kind
        )));
    }
    let main_root = ctx.work_root.clone();

    // ── every delegation gate, decided BEFORE the first lock ──────────────
    // Campaign rule 2: lock.rs appends a `result: "acquired"` row to
    // .bee/logs/contention.jsonl on EVERY successful acquire, so a delegation
    // taken after one would leave a doubled row in the `.bee/` tree — a C1
    // break, not just noisy telemetry. Each probe below is read-only.

    let commands = read_worktree_commands(&main_root)?; // corrupt config -> Node
    // test-simple (412e9b3a) + no-test-repos D1/D2: `commands.verify ||
    // (typeof commands.test === 'string' ? commands.test : undefined)`, with
    // the literal "none" sentinel mapped to "no verify configured".
    let verify_command = commands
        .verify
        .clone()
        .or_else(|| commands.test_string.clone())
        .filter(|c| c != "none");
    // worktree-companion-hook: resolved unconditionally (cheap) — there is no
    // `--with-companion` on the merge side, because the worktree's own marker
    // IS the signal. A worktree WITH a marker is torn down even when this
    // invocation opted in to nothing; there is nothing to opt in to.
    let companion_end_command = commands.companion_end.clone();

    // readGrants is consulted twice (P1's grant check, P3's fence); an
    // unparseable registry delegates here rather than from inside a hold.
    read_grants_strict(&main_root.join(".bee"))?;

    // runVerifyChild's ONLY V8/libuv byte is its spawn-`error` message, and it
    // is reached AFTER the merge is staged — see `shell_launchable`.
    if verify_command.is_some() && !shell_launchable() {
        return None;
    }

    let main_root_s = p(&main_root);
    let ctrl_root_s = crate::verbs::reservations::control_root_for(&main_root_s).ok()?;
    let ctrl_root = PathBuf::from(&ctrl_root_s);
    let session_id = crate::verbs::reservations::resolve_session_id(None, &ctrl_root_s)
        .ok()?
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| WORKTREE_MERGE_SESSIONLESS_ID.to_string());

    // listQueueRecords reads each record with fsutil's WARNING readJson.
    if !crate::integration_queue::preflight_queue_readable(&ctrl_root) {
        return None;
    }
    // performCleanup's releaseAllForHolder reads the holds ledger the same
    // way; only `--cleanup` can reach it.
    if cleanup {
        let ledger = main_root
            .join(".bee")
            .join("runtime")
            .join("cross-worktree-holds.json");
        if let Ok(bytes) = std::fs::read(&ledger) {
            serde_json::from_slice::<Value>(&bytes).ok()?;
        }
    }

    // ── the drain (multisession-native-22 D8 stage 5) ─────────────────────
    let mut thrown: Option<String> = None;
    let drained = crate::integration_queue::run_through_queue(
        &ctrl_root,
        &id,
        &session_id,
        "main",
        queue_wait_bound_ms,
        crate::integration_queue::DEFAULT_POLL_INTERVAL_MS,
        crate::integration_queue::DEFAULT_PROCESSOR_TTL_SECONDS,
        crate::integration_queue::DEFAULT_RENEW_INTERVAL_MS,
        |hooks| {
            match merge_feature_worktree(
                &main_root,
                &id,
                cleanup,
                verify_command.as_deref(),
                companion_end_command.as_deref(),
                Some(hooks),
            ) {
                Ok(answer) => {
                    let ok = answer.ok;
                    Ok((Some(answer), ok))
                }
                // processAsOwner persists `error.message` into the queue
                // record and rethrows — both are deterministic here.
                Err(MErr::Thrown(message)) => Err(message),
                // The one late-delegation residual. Reachable ONLY if the
                // grants registry stops parsing BETWEEN the probe above and
                // P1's own read — a race, never an ordinary shape. Zero
                // mutations have happened to the repo, so the Node re-run
                // reproduces the same answer; the cost is one extra queue
                // record (this one, driven to 'done') plus its contention
                // rows, the same "idempotent steps, one duplicated
                // bookkeeping artifact" residual verbs/workspace_store.rs
                // and `worktree register` already document.
                Err(MErr::Ex) => Ok((None, true)),
            }
        },
    );

    let (result, ok, message_lines) = match drained {
        Err(crate::integration_queue::QErr::Msg(m))
        | Err(crate::integration_queue::QErr::LockBusy(m)) => {
            thrown = Some(m);
            (Map::new(), false, Vec::new())
        }
        Err(crate::integration_queue::QErr::Ex) => return None,
        Ok(crate::integration_queue::Drain::TimedOut(timeout)) => {
            // Advisor condition B: this text must UNAMBIGUOUSLY say the merge
            // did NOT run — never readable as success.
            let lines = vec![format!(
                "Merge of worktree {id} did NOT run: {}",
                timeout.message
            )];
            let map = match timeout.result {
                Value::Object(m) => m,
                _ => Map::new(),
            };
            (map, false, lines)
        }
        Ok(crate::integration_queue::Drain::Ran(None)) => return None, // MErr::Ex
        Ok(crate::integration_queue::Drain::Ran(Some(answer))) => {
            let lines = merge_text_lines(&id, &main_root, &answer);
            (answer.result, answer.ok, lines)
        }
    };
    if let Some(message) = thrown {
        return Some(ctx.fail(&message));
    }
    Some(ctx.emit_code(
        &Value::Object(result),
        &message_lines.join("\n"),
        if ok { 0 } else { 1 },
    ))
}

/// bee.mjs handleWorktreeMerge's text block, for every non-timeout outcome.
fn merge_text_lines(id: &str, main_root: &Path, answer: &MergeAnswer) -> Vec<String> {
    let r = &answer.result;
    let s = |key: &str| -> String { r.get(key).map(jsjson::js_to_string).unwrap_or_default() };
    let mut lines: Vec<String> = Vec::new();
    let code = r.get("code").and_then(Value::as_str).unwrap_or("");
    if answer.ok {
        if code == "ALREADY_UP_TO_DATE" {
            lines.push(format!(
                "Worktree {id} (branch {}) is already up to date with {} — nothing to merge; no commit was made.",
                s("branch"),
                p(main_root)
            ));
        } else {
            lines.push(format!(
                "Merged worktree {id} (branch {}) into {}.",
                s("branch"),
                p(main_root)
            ));
            lines.push(format!("  verify: {}", s("verify")));
        }
        // The companion block, shared by BOTH ok outcomes (issues-46-53 D3).
        if let Some(companion) = r.get("companion").filter(|v| js_truthy(v)) {
            lines.push(match companion.get("warning").filter(|w| js_truthy(w)) {
                Some(warning) => {
                    format!("  companion: WARNING — {}", jsjson::js_to_string(warning))
                }
                None => {
                    let session = match companion.get("sessionId").filter(|v| js_truthy(v)) {
                        Some(v) => format!(" (session {})", jsjson::js_to_string(v)),
                        None => String::new(),
                    };
                    format!("  companion: ended{session}.")
                }
            });
        }
        if let Some(warning) = r.get("warning") {
            lines.push(format!(
                "  WARNING ({}): {}",
                warning.get("code").map(jsjson::js_to_string).unwrap_or_default(),
                warning.get("message").map(jsjson::js_to_string).unwrap_or_default()
            ));
        }
        if let Some(cleanup) = r.get("cleanup") {
            lines.push(if cleanup.get("ok") == Some(&Value::Bool(true)) {
                "  cleanup: worktree removed, branch deleted.".to_string()
            } else {
                format!(
                    "  cleanup: refused ({}) — {}",
                    cleanup.get("code").map(jsjson::js_to_string).unwrap_or_default(),
                    cleanup.get("reason").map(jsjson::js_to_string).unwrap_or_default()
                )
            });
            if let Some(w) = cleanup.get("warning") {
                lines.push(format!("  WARNING: {}", jsjson::js_to_string(w)));
            }
        } else if let Some(cmd) = r.get("cleanup_suggested_command") {
            lines.push(format!(
                "  cleanup: run `{}` when ready.",
                jsjson::js_to_string(cmd)
            ));
        }
    } else if code == "MERGE_VERIFY_RED" {
        lines.push(format!(
            "Merge of worktree {id} (branch {}) was TEXTUALLY CLEAN, but verify is RED (semantic-conflict alarm).",
            s("branch")
        ));
        lines.push(format!(
            "The merge was aborted — {} was left byte-untouched; no merge commit exists. Fix-first before release, then retry the merge.",
            p(main_root)
        ));
        lines.push("--- verify output tail ---".to_string());
        lines.push(s("output_tail"));
    } else {
        lines.push(format!(
            "Merge of worktree {id} hit a textual conflict — the merge was aborted and {} was left byte-untouched; bee does not auto-resolve a textual conflict. Resolve it in the worktree and retry.",
            p(main_root)
        ));
    }
    lines
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

    /// resolveWorktreeFeature's preference order (issues-46-53 D4): the
    /// IMMUTABLE creation slug wins, and its absence degrades EXACTLY to the
    /// pre-fix `state.feature` behavior rather than refusing.
    #[test]
    fn worktree_feature_prefers_the_immutable_creation_slug() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path();
        std::fs::create_dir_all(wt.join(".bee").join("runtime")).unwrap();

        // Neither file: unknown, and the branch check falls back to the shape.
        let none = resolve_worktree_feature(wt);
        assert_eq!(none.feature, None);
        assert!(wt_branch_shaped("wt/demo-2"));
        assert!(!wt_branch_shaped("wt/Demo"));
        assert!(!wt_branch_shaped("feature/demo"));

        // Only state.json: the legacy degradation path.
        std::fs::write(wt.join(".bee").join("state.json"), "{\"feature\":\"renamed\"}").unwrap();
        let legacy = resolve_worktree_feature(wt);
        assert_eq!(legacy.feature.as_deref(), Some("renamed"));
        assert_eq!(legacy.created, None);

        // Both: the creation slug wins, and BOTH are reported so the refusal
        // can name the field that drifted.
        std::fs::write(
            wt.join(".bee").join("runtime").join("worktree-identity.json"),
            "{\"feature\":\"original\"}",
        )
        .unwrap();
        let both = resolve_worktree_feature(wt);
        assert_eq!(both.feature.as_deref(), Some("original"));
        assert_eq!(both.created.as_deref(), Some("original"));
        assert_eq!(both.state_feature.as_deref(), Some("renamed"));

        // A corrupt file is "unknown", never a crash — the .mjs reads these
        // with a bare JSON.parse in a try, not fsutil's warning readJson.
        std::fs::write(
            wt.join(".bee").join("runtime").join("worktree-identity.json"),
            "{oops",
        )
        .unwrap();
        assert_eq!(resolve_worktree_feature(wt).created, None);
    }

    /// gitStatusPorcelain's failure message is deterministic even when git
    /// never launched — including the literal "exit null" `${result.status}`
    /// renders. This is what makes every downstream `.stdout.trim()` site
    /// unreachable (module header, blocker (c)).
    #[test]
    fn git_status_failure_message_is_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        // Not a git repo at all: git launches and exits non-zero.
        let err = git_status_porcelain(tmp.path()).unwrap_err();
        assert!(
            err.starts_with(&format!("\"git status --porcelain\" failed in {}: ", p(tmp.path()))),
            "{err}"
        );
        // The never-launched shape renders "exit null" through the same chain.
        let never = GitOut { status: None, stdout: None, stderr: None };
        assert_eq!(never.fail_text(), "exit null");
    }

    /// The verify child is Node's `shell: true`, so a shell builtin runs and
    /// its exit code comes back verbatim — and `output_tail` is the LAST 30
    /// lines of stdout-then-stderr concatenated.
    #[test]
    fn verify_child_captures_status_and_output() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(shell_launchable());
        let green = run_verify_child("exit 0", tmp.path(), &|| {}, 30_000.0);
        assert!(green.ran);
        assert_eq!(green.status, Some(0));

        let red = run_verify_child("echo RED-TAIL& exit 7", tmp.path(), &|| {}, 30_000.0);
        assert_eq!(red.status, Some(7));
        assert!(red.combined.contains("RED-TAIL"), "{:?}", red.combined);

        // The tick fires while a slow child runs (integration-queue's
        // processor-lease heartbeat depends on exactly this).
        let ticks = std::sync::atomic::AtomicUsize::new(0);
        let slow = if cfg!(windows) {
            "ping -n 2 127.0.0.1 > NUL"
        } else {
            "sleep 1"
        };
        let out = run_verify_child(
            slow,
            tmp.path(),
            &|| {
                ticks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            },
            60.0,
        );
        assert_eq!(out.status, Some(0));
        assert!(ticks.load(std::sync::atomic::Ordering::SeqCst) > 0, "the renewal tick must fire");
    }

    /// performCleanup's refusal shapes carry Node's exact key ORDER — the
    /// bytes `--json` prints and the twin diff pins.
    #[test]
    fn cleanup_check_failure_keeps_nodes_key_order() {
        let tmp = tempfile::tempdir().unwrap();
        let out = perform_cleanup(tmp.path(), tmp.path(), "wt/demo", "wt-demo", false);
        assert_eq!(out.keys().collect::<Vec<_>>(), vec!["ok", "code", "reason"]);
        assert_eq!(out["ok"], Value::Bool(false));
        assert_eq!(out["code"], json!("WORKTREE_MERGE_CLEANUP_CHECK_FAILED"));
    }

    /// attachCleanupOutcome without the flag NEVER runs anything — it attaches
    /// the suggestion (decision D8b: "never prompt").
    #[test]
    fn cleanup_without_the_flag_only_suggests() {
        let tmp = tempfile::tempdir().unwrap();
        let mut result = Map::new();
        attach_cleanup_outcome(&mut result, tmp.path(), tmp.path(), "wt/demo", "wt-demo", false, false);
        assert_eq!(
            result["cleanup_suggested_command"],
            json!("bee worktree merge --id wt-demo --cleanup --json")
        );
        assert!(!result.contains_key("cleanup"));
    }

    /// releaseAllForHolder marks every unreleased row for the holder and
    /// leaves everyone else's — and never rewrites the file when nothing
    /// changed (worktree-holds.mjs's own "only write when something changed").
    #[test]
    fn release_all_for_holder_is_holder_scoped() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp
            .path()
            .join(".bee")
            .join("runtime")
            .join("cross-worktree-holds.json");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            serde_json::to_string_pretty(&json!({"holds": [
                {"holder": "wt-a", "path": "src/x", "released_at": null},
                {"holder": "wt-b", "path": "src/y", "released_at": null},
                {"holder": "wt-a", "path": "src/z", "released_at": "2020-01-01T00:00:00.000Z"},
            ]}))
            .unwrap(),
        )
        .unwrap();
        release_all_for_holder(tmp.path(), "wt-a");
        let after: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        let holds = after["holds"].as_array().unwrap();
        assert!(holds[0]["released_at"].is_string(), "the holder's row is released");
        assert!(holds[1]["released_at"].is_null(), "another holder is untouched");
        assert_eq!(holds[2]["released_at"], json!("2020-01-01T00:00:00.000Z"));

        // Nothing to release: the file is left byte-identical.
        let before = std::fs::read_to_string(&file).unwrap();
        release_all_for_holder(tmp.path(), "wt-nobody");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
    }

    // ── worktree-companion-hook, over REAL `git worktree add` fixtures ─────
    //
    // The mount is a real symlink and win32 denies symlink creation without
    // SeCreateSymbolicLinkPrivilege, so every test that must CREATE one probes
    // the capability and skips LOUDLY, naming it. The tests that only need a
    // mount to EXIST use a plain untracked file instead: the dirty-check
    // exclusion and the teardown unlink are indifferent to the node type
    // (porcelain collapses a symlink-to-directory and a plain file into the
    // same "?? <top-level>/" summary line, which is the whole reason the
    // exclusion has to be a git pathspec), so the companion merge path runs
    // natively here on any host.

    const SYMLINK_CAP: &str = "symlink creation denied — needs SeCreateSymbolicLinkPrivilege \
(Developer Mode or an elevated shell)";

    fn symlink_capable() -> bool {
        use std::sync::OnceLock;
        static CAP: OnceLock<bool> = OnceLock::new();
        *CAP.get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("t");
            std::fs::create_dir(&target).unwrap();
            symlink_dir(&target.to_string_lossy(), &dir.path().join("l")).is_ok()
        })
    }

    fn git_ok(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the worktree fixtures");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// A MAIN checkout with one commit and a host-shaped `.gitignore`: the
    /// whole `.bee` store is ignored EXCEPT the companion marker, which is the
    /// real-world shape COMPANION_MARKER_REL's own comment describes (it sits
    /// outside the gitignored `.bee/runtime/` prefix, so it is untracked AND
    /// not ignored — and therefore has to be excluded by pathspec).
    fn main_repo(tmp: &Path) -> PathBuf {
        let main = tmp.join("main");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::write(main.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        std::fs::write(
            main.join(".gitignore"),
            ".bee/*\n!.bee/companion-session.json\n",
        )
        .unwrap();
        std::fs::write(main.join("f.txt"), "x").unwrap();
        git_ok(&main, &["init", "-q", "-b", "main", "."]);
        git_ok(&main, &["config", "user.email", "a@b.c"]);
        git_ok(&main, &["config", "user.name", "t"]);
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "init"]);
        main
    }

    /// A shell command that prints `file`'s bytes verbatim — the portable way
    /// to give `commands.worktree_companion_start` a fixed JSON stdout through
    /// the same `shell: true` spawn production uses.
    fn cat_command(file: &Path) -> String {
        if cfg!(windows) {
            format!("type \"{}\"", file.to_string_lossy())
        } else {
            format!("cat \"{}\"", file.to_string_lossy())
        }
    }

    /// The whole `--with-companion` creation path: the configured child runs,
    /// its declared worktreePath is mounted as a real symlink at the
    /// configured relative mount, and the marker records all three fields.
    #[test]
    fn companion_start_mounts_the_declared_path_and_writes_the_marker() {
        if !symlink_capable() {
            eprintln!("SKIP (env-limited: {SYMLINK_CAP}) — worktree new --with-companion mounts and marks");
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let companion = tmp.path().join("shared-checkout");
        std::fs::create_dir_all(&companion).unwrap();
        let payload = tmp.path().join("payload.json");
        std::fs::write(
            &payload,
            jsjson::stringify(&json!({
                "worktreePath": companion.to_string_lossy(),
                "sessionId": "sess-1",
            })),
        )
        .unwrap();

        let command = cat_command(&payload);
        let mut lock_busy = None;
        let created = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec {
                start_command: Some(&command),
                mount_path: Some("vendor/companion"),
            },
            &mut lock_busy,
        )
        .unwrap_or_else(|e| match e {
            CErr::Refuse(m) => panic!("refused: {m}"),
            CErr::Ex => panic!("delegated"),
        });

        assert_eq!(
            created.companion.get("sessionId"),
            Some(&json!("sess-1")),
            "{:?}",
            created.companion
        );
        assert_eq!(created.companion.get("mountPath"), Some(&json!("vendor/companion")));

        // The mount is a real link to the declared path.
        let mount = created.worktree_root.join("vendor").join("companion");
        assert!(std::fs::symlink_metadata(&mount).unwrap().file_type().is_symlink());
        assert_eq!(
            dunce::canonicalize(&mount).unwrap(),
            dunce::canonicalize(&companion).unwrap()
        );

        // The marker is Node's bytes: 2-space JSON + a trailing newline.
        let marker = std::fs::read_to_string(companion_marker_file(&created.worktree_root)).unwrap();
        assert!(marker.ends_with("}\n"), "{marker}");
        let parsed: Value = serde_json::from_str(&marker).unwrap();
        assert_eq!(
            parsed.as_object().unwrap().keys().collect::<Vec<_>>(),
            vec!["sessionId", "worktreePath", "mountPath"]
        );
    }

    /// A companion start failure fires AFTER `git worktree add`, so it enters
    /// the post-add rollback ladder: worktree gone, branch gone, grant gone,
    /// and the typed refusal names the child's exit. Needs no symlink — the
    /// child dies before the mount is ever created.
    #[test]
    fn a_failed_companion_start_rolls_the_worktree_back() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let err = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec {
                start_command: Some("exit 7"),
                mount_path: Some("vendor/companion"),
            },
            &mut lock_busy,
        )
        .err()
        .expect("a failing companion start must refuse");
        let CErr::Refuse(message) = err else {
            panic!("a companion start failure must never delegate")
        };
        assert!(
            message.starts_with("[WORKTREE_POST_ADD_FAILED] "),
            "{message}"
        );
        assert!(
            message.contains("commands.worktree_companion_start failed (exit 7): (no output)"),
            "{message}"
        );
        assert!(message.contains("it has been rolled back"), "{message}");

        // The ladder unwound in Node's order: nothing survives.
        assert!(!tmp.path().join("main--wt--demo").exists());
        assert_eq!(
            read_grants_strict(&main.join(".bee")).unwrap(),
            Map::new(),
            "the grant is rolled back"
        );
        let branches = run_git(&main, &["branch", "--list", "wt/demo"]);
        assert_eq!(js_trim(&branches.stdout.unwrap_or_default()), "");
    }

    /// Unparseable child stdout is a typed post-add refusal too. Node's
    /// parenthetical carries V8's JSON.parse message and this port carries
    /// serde's (the module header's documented divergence); every other byte —
    /// including the raw-stdout tail — is Node's.
    #[test]
    fn unparseable_companion_stdout_refuses_with_the_raw_tail() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let payload = tmp.path().join("payload.txt");
        std::fs::write(&payload, "not json at all").unwrap();
        let mut lock_busy = None;
        let err = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec {
                start_command: Some(&cat_command(&payload)),
                mount_path: Some("vendor/companion"),
            },
            &mut lock_busy,
        )
        .err()
        .unwrap();
        let CErr::Refuse(message) = err else { panic!("must not delegate") };
        assert!(message.contains(
            "commands.worktree_companion_start must print JSON with a \"worktreePath\" field to stdout — got unparseable output ("
        ), "{message}");
        assert!(message.contains("Raw stdout: not json at all"), "{message}");
        assert!(!tmp.path().join("main--wt--demo").exists());
    }

    /// JSON that parses but carries no usable worktreePath is a FULLY
    /// deterministic refusal — `JSON.stringify(parsed)` and nothing else.
    #[test]
    fn companion_stdout_without_a_worktree_path_refuses_deterministically() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let payload = tmp.path().join("payload.json");
        std::fs::write(&payload, "{\"sessionId\":\"s\"}").unwrap();
        let mut lock_busy = None;
        let err = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec {
                start_command: Some(&cat_command(&payload)),
                mount_path: Some("vendor/companion"),
            },
            &mut lock_busy,
        )
        .err()
        .unwrap();
        let CErr::Refuse(message) = err else { panic!("must not delegate") };
        assert!(message.contains(
            "commands.worktree_companion_start's JSON output must include a non-empty \"worktreePath\" string — got {\"sessionId\":\"s\"}."
        ), "{message}");
    }

    /// The two zero-mutation companion config refusals, with their exact
    /// `[CODE] …` bytes — neither ever reaches `git worktree add`.
    #[test]
    fn companion_config_refusals_are_zero_mutation() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;

        let only_one = create_feature_worktree(
            &main,
            "demo",
            None,
            CompanionSpec { start_command: Some("true"), mount_path: None },
            &mut lock_busy,
        )
        .err()
        .unwrap();
        let CErr::Refuse(message) = only_one else { panic!() };
        assert_eq!(
            message,
            "[WORKTREE_COMPANION_CONFIG_INCOMPLETE] commands.worktree_companion_start and commands.worktree_companion_mount must both be configured to use --with-companion — only one was found."
        );

        for bad in ["/abs/mount", "..\\escape", "a/../b"] {
            let mut lock_busy = None;
            let err = create_feature_worktree(
                &main,
                "demo",
                None,
                CompanionSpec { start_command: Some("true"), mount_path: Some(bad) },
                &mut lock_busy,
            )
            .err()
            .unwrap();
            let CErr::Refuse(message) = err else { panic!() };
            assert!(
                message.starts_with("[WORKTREE_COMPANION_CONFIG_INVALID] "),
                "{bad}: {message}"
            );
        }
        assert!(!tmp.path().join("main--wt--demo").exists(), "zero mutation");
    }

    /// The companion MERGE path, end to end, on a real worktree: the mount and
    /// the marker are both excluded from the dirty-check by pathspec (without
    /// that the merge refuses WORKTREE_MERGE_WORKTREE_DIRTY), the configured
    /// `_end` command runs, both are unlinked, and `companion` rides the result.
    #[test]
    fn a_companion_worktree_merges_and_tears_the_mount_down() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        let mut lock_busy = None;
        let created =
            create_feature_worktree(&main, "demo", None, CompanionSpec::default(), &mut lock_busy)
                .unwrap_or_else(|_| panic!("plain worktree creation must succeed"));
        let wt = created.worktree_root.clone();

        // Real work on the branch, so the merge has something to do.
        std::fs::write(wt.join("f.txt"), "y").unwrap();
        git_ok(&wt, &["config", "user.email", "a@b.c"]);
        git_ok(&wt, &["config", "user.name", "t"]);
        git_ok(&wt, &["commit", "-qam", "work"]);

        // A companion mount + marker, exactly as runCompanionStart leaves them.
        // A plain untracked file stands in for the symlink (see the block
        // comment above) so this runs on every host.
        std::fs::create_dir_all(wt.join("vendor")).unwrap();
        std::fs::write(wt.join("vendor").join("companion"), "mount").unwrap();
        std::fs::write(
            companion_marker_file(&wt),
            format!(
                "{}\n",
                jsjson::stringify_pretty(&json!({
                    "sessionId": "sess-1",
                    "worktreePath": tmp.path().join("shared").to_string_lossy(),
                    "mountPath": "vendor/companion",
                }))
            ),
        )
        .unwrap();

        // Both are genuinely dirt without the exclusion — prove it, so the
        // pathspec below is doing real work.
        assert!(is_tree_dirty(&wt).unwrap(), "the mount+marker read as dirty");

        let answer = merge_feature_worktree(&main, &created.id, false, None, Some("exit 0"), None)
            .unwrap_or_else(|e| match e {
                MErr::Thrown(m) => panic!("merge threw: {m}"),
                MErr::Ex => panic!("merge delegated"),
            });
        assert!(answer.ok, "{:?}", answer.result);
        assert_eq!(answer.result["merged"], Value::Bool(true));
        assert_eq!(
            answer.result["companion"],
            json!({ "ended": true, "sessionId": "sess-1" }),
            "an ended-cleanly companion carries no `warning` key"
        );
        // `companion` sits directly after `verify`, before the cleanup keys.
        let keys: Vec<&String> = answer.result.keys().collect();
        let vi = keys.iter().position(|k| *k == "verify").unwrap();
        assert_eq!(keys[vi + 1], "companion");

        assert!(!wt.join("vendor").join("companion").exists(), "mount unlinked");
        assert!(!companion_marker_file(&wt).exists(), "marker unlinked");

        let lines = merge_text_lines(&created.id, &main, &answer);
        assert!(
            lines.iter().any(|l| l == "  companion: ended (session sess-1)."),
            "{lines:?}"
        );
    }

    /// With no `commands.worktree_companion_end` configured, teardown still
    /// removes the mount (so the merge is never blocked) but says so LOUDLY.
    #[test]
    fn teardown_without_an_end_command_warns_and_still_unlinks() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(wt.join(".bee")).unwrap();
        std::fs::create_dir_all(wt.join("vendor")).unwrap();
        std::fs::write(wt.join("vendor").join("companion"), "mount").unwrap();
        let marker = json!({"sessionId": Value::Null, "mountPath": "vendor/companion"});
        std::fs::write(companion_marker_file(&wt), jsjson::stringify(&marker)).unwrap();

        let out = teardown_companion_if_present(tmp.path(), &wt, None, Some(&marker)).unwrap();
        assert_eq!(out["ended"], Value::Bool(false));
        assert_eq!(out["sessionId"], Value::Null);
        assert!(
            jsjson::js_to_string(&out["warning"]).starts_with(
                "a companion marker exists on this worktree but commands.worktree_companion_end is not configured"
            ),
            "{out:?}"
        );
        assert!(!wt.join("vendor").join("companion").exists());
        assert!(!companion_marker_file(&wt).exists());
    }

    /// A failing `_end` command never blocks the merge — the mount still goes,
    /// and the failure rides the result as a warning with the child's exit.
    #[test]
    fn a_failed_end_command_warns_but_never_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(wt.join(".bee")).unwrap();
        std::fs::write(wt.join("mount"), "m").unwrap();
        let marker = json!({"sessionId": "sess-9", "mountPath": "mount"});
        std::fs::write(companion_marker_file(&wt), jsjson::stringify(&marker)).unwrap();

        let out =
            teardown_companion_if_present(tmp.path(), &wt, Some("exit 3"), Some(&marker)).unwrap();
        assert_eq!(out["ended"], Value::Bool(false));
        assert_eq!(out["sessionId"], json!("sess-9"));
        assert!(
            jsjson::js_to_string(&out["warning"])
                .starts_with("commands.worktree_companion_end failed (exit 3): (no output) — "),
            "{out:?}"
        );
        assert!(!wt.join("mount").exists());
    }

    /// The `<id>` substitution is JS `String.replace` with a STRING pattern:
    /// first occurrence only, `$`-patterns honored.
    #[test]
    fn the_end_command_substitutes_the_session_id_like_js() {
        assert_eq!(js_replace_first("end <id> then <id>", "<id>", "s1"), "end s1 then <id>");
        assert_eq!(js_replace_first("end <id>", "<id>", ""), "end ");
        assert_eq!(js_replace_first("no token", "<id>", "s1"), "no token");
        // $$ -> literal $, $& -> the matched text, $` / $' -> the surroundings.
        assert_eq!(js_replace_first("a<id>b", "<id>", "$$"), "a$b");
        assert_eq!(js_replace_first("a<id>b", "<id>", "$&"), "a<id>b");
        assert_eq!(js_replace_first("a<id>b", "<id>", "$`|$'"), "aa|bb");
        // $1 has no capture group behind a string pattern — left literal.
        assert_eq!(js_replace_first("a<id>b", "<id>", "$1"), "a$1b");
    }

    /// The dirty-check exclusion has to be a git PATHSPEC: porcelain collapses
    /// an untracked nested path to its top-level directory, so text-filtering
    /// for `vendor/companion` would never match and the merge would refuse
    /// forever. This pins the behavior that makes that true.
    #[test]
    fn the_dirty_check_exclusion_is_a_pathspec_not_a_text_filter() {
        let tmp = tempfile::tempdir().unwrap();
        let main = main_repo(tmp.path());
        std::fs::create_dir_all(main.join("vendor")).unwrap();
        std::fs::write(main.join("vendor").join("companion"), "m").unwrap();

        // Porcelain names only the TOP-LEVEL dir, never the nested path.
        let plain = git_status_porcelain(&main).unwrap();
        assert!(plain.contains("?? vendor/"), "{plain:?}");
        assert!(!plain.contains("vendor/companion"), "{plain:?}");
        assert!(is_tree_dirty(&main).unwrap());

        // The pathspec removes it at the source, nested depth and all.
        assert!(!is_tree_dirty_excluding(
            &main,
            &["vendor/companion".to_string(), companion_marker_rel()]
        )
        .unwrap());
    }

    /// A truthy marker with no string `mountPath` is an EXPLICIT NATIVE
    /// REFUSAL: the .mjs dies here with a V8 TypeError, which can neither be
    /// reproduced nor (post-cutover) delegated.
    #[test]
    fn a_marker_without_a_mount_path_is_a_typed_refusal() {
        let MErr::Thrown(message) = companion_mount_path(&json!({"sessionId": "s"})).unwrap_err()
        else {
            panic!("must be a typed refusal")
        };
        assert!(
            message.starts_with(
                "[WORKTREE_MERGE_COMPANION_MARKER_INVALID] the companion marker at .bee/companion-session.json has no usable \"mountPath\" string (got null)"
            ),
            "{message}"
        );
        assert_eq!(
            companion_mount_path(&json!({"mountPath": "m"})).map_err(|_| ()),
            Ok("m".to_string())
        );
    }

    /// A marker that is missing, unparseable, or parses FALSY all read as "no
    /// companion here" — the .mjs's bare `JSON.parse` in a try, plus every
    /// consumer's `if (!marker)` guard.
    #[test]
    fn marker_reads_match_nodes_falsy_contract() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path();
        std::fs::create_dir_all(wt.join(".bee")).unwrap();
        assert_eq!(read_companion_marker(wt), None, "missing");
        for falsy in ["{oops", "null", "false", "0", "\"\""] {
            std::fs::write(companion_marker_file(wt), falsy).unwrap();
            assert_eq!(read_companion_marker(wt), None, "{falsy}");
        }
        std::fs::write(companion_marker_file(wt), "{\"mountPath\":\"m\"}").unwrap();
        assert_eq!(
            read_companion_marker(wt),
            Some(json!({"mountPath": "m"}))
        );
    }

    /// `--queue-wait-ms` is `Number(string)`, whole — the conversion
    /// validate()'s finiteness gate and the handler's positive filter both run
    /// against.
    #[test]
    fn queue_wait_ms_uses_js_number_semantics() {
        assert_eq!(js_string_to_number("5000"), 5000.0);
        assert_eq!(js_string_to_number("  5000  "), 5000.0);
        assert_eq!(js_string_to_number("5e3"), 5000.0);
        assert_eq!(js_string_to_number("1.5"), 1.5);
        assert_eq!(js_string_to_number(".5"), 0.5);
        assert_eq!(js_string_to_number("-1"), -1.0);
        assert_eq!(js_string_to_number("0x10"), 16.0);
        assert_eq!(js_string_to_number("0b101"), 5.0);
        assert_eq!(js_string_to_number("0o17"), 15.0);
        assert_eq!(js_string_to_number(""), 0.0);
        assert_eq!(js_string_to_number("   "), 0.0);
        assert!(js_string_to_number("Infinity").is_infinite());
        assert!(js_string_to_number("-Infinity").is_infinite());
        assert!(js_string_to_number("1e400").is_infinite()); // overflow, like V8
        for nan in ["abc", "5px", "0x", "1..2", "1e", "--1", "+"] {
            assert!(js_string_to_number(nan).is_nan(), "{nan}");
        }
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
