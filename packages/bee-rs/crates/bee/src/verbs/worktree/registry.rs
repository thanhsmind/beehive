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
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
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
