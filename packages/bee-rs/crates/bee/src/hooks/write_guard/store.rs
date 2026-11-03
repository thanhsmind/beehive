// state, claims, reservations, worktree holds and the workspace store
//
// Split out of the single 5.9k-line hooks/write_guard.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── state.mjs ports ───────────────────────────────────────────────────────

/// provenance: state.mjs KNOWN_PHASES / isKnownPhase.
pub(crate) const KNOWN_PHASES: &[&str] = &[
    "idle", "exploring", "planning", "swarming", "reviewing", "scribing",
    "compounding", "grooming", "compounding-complete",
];

pub(crate) fn is_known_phase(phase: &Value) -> bool {
    matches!(phase, Value::String(s) if KNOWN_PHASES.contains(&s.as_str()))
}

/// provenance: state.mjs defaultState() — only the keys guard logic reads.
pub(crate) fn default_state() -> Map<String, Value> {
    let mut gates = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        gates.insert(g.into(), Value::Bool(false));
    }
    let mut m = Map::new();
    m.insert("schema_version".into(), Value::String("1.0".into()));
    m.insert("phase".into(), Value::String("idle".into()));
    m.insert("feature".into(), Value::Null);
    m.insert("mode".into(), Value::Null);
    m.insert("approved_gates".into(), Value::Object(gates));
    m.insert("workers".into(), Value::Array(vec![]));
    m.insert("summary".into(), Value::String(String::new()));
    m.insert(
        "next_action".into(),
        Value::String("No active bee work — awaiting a user request.".into()),
    );
    m
}

/// provenance: state.mjs readState — fail-open merge over defaultState with
/// the D13 legacy-phase coercion. A corrupt file warns and reads as absent,
/// so the guards evaluate against defaultState() exactly as Node did.
pub(crate) fn read_state(root: &Path) -> R<Map<String, Value>> {
    let file = root.join(".bee").join("state.json");
    let parsed = read_json_g(&file)?;
    let obj = match parsed {
        Some(Value::Object(m)) => m,
        _ => return Ok(default_state()),
    };
    let mut merged = default_state();
    for (k, v) in &obj {
        merged.insert(k.clone(), v.clone());
    }
    // approved_gates: { ...defaults, ...(state.approved_gates || {}) } — a
    // truthy non-object spreads only numeric/char keys, which the four gate
    // names never collide with, so defaults stand for them.
    let mut gates = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        gates.insert(g.into(), Value::Bool(false));
    }
    if let Some(Value::Object(over)) = obj.get("approved_gates") {
        for (k, v) in over {
            gates.insert(k.clone(), v.clone());
        }
    }
    merged.insert("approved_gates".into(), Value::Object(gates));
    if merged.get("phase") == Some(&Value::String("validating".into())) {
        merged.insert("phase".into(), Value::String("planning".into()));
    }
    Ok(merged)
}

/// provenance: state.mjs readConfig (merged tracked+overlay, advisor
/// stripped). Only raw pass-through keys (guards.*, worktree_first,
/// product_root, hooks) are consumed by this hook; the normalize* steps in
/// the .mjs never touch those. A corrupt file warns natively and reads as
/// absent — infallible now.
pub(crate) fn read_config(root: &Path) -> R<Map<String, Value>> {
    Ok(crate::state::read_config_raw(root))
}

// sfg-4: `check_product_root_silent` used to live here. It ported state.mjs
// resolveProductRoot's WARNING side effect and nothing else: it answered
// `Err(Nd)` for a non-string `product_root`, or one naming a directory that
// is not there — the two shapes Node printed a warning for.
//
// That error was not a warning here. It left `resolve_context`, then
// `control_root_for_state`, then `resolve_write_record`, and landed on
// `hooks/mod.rs`'s `emit_undecidable`: exit 0, "the guard did NOT run on
// it", for every path and every `.bee` mutation. Two lines of config
// therefore switched the WHOLE write guard off — the same fail-OPEN sfg-3
// closed in the claim readers, one call frame away.
//
// The honest read is "nothing at all", and it is not a close call.
// `product_root` names where PRODUCT DOCS live (docs/backlog.md,
// docs/specs/). This guard never reads it: not for containment, not for the
// acting record, not for any refusal it can utter. A product_root the reader
// cannot resolve is not evidence of containment, and it is not evidence of
// anything else the guard decides — so it decides nothing, and the guard
// goes on judging the inputs it does read. The warning it stood for is not
// lost either: the callers still perform the `read_config` beside it, which
// is what prints bee's own corrupt-config line.
//
// Pinned by `sfg4_a_product_root_the_guard_cannot_resolve_never_stops_it`.

/// provenance: worktree-store.mjs readGrants — swallow-all read of
/// <mainStoreRoot>/runtime/worktree-grants.json.
pub(crate) fn read_grants(main_bee_dir: &Path) -> Map<String, Value> {
    let file = main_bee_dir.join("runtime").join("worktree-grants.json");
    match std::fs::read_to_string(&file) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(m)) => m,
            _ => Map::new(),
        },
        Err(_) => Map::new(),
    }
}

/// provenance: state.mjs readGitdirFile.
pub(crate) fn read_gitdir_file(file: &Path, base: &str) -> R<Option<String>> {
    let raw = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    let mut raw = js_trim(&raw);
    if let Some(rest) = raw.strip_prefix("gitdir:") {
        raw = js_trim(rest);
    }
    if raw.is_empty() {
        return Ok(None);
    }
    let fixed: String = if cfg!(windows) {
        raw.to_string()
    } else {
        raw.replace('\\', "/")
    };
    Ok(Some(np_resolve2(base, &fixed)?))
}

/// The guard-relevant slice of state.mjs resolveContext(cwd).
#[derive(Clone, Default)]
pub(crate) struct JsCtx {
    pub(crate) control_root: Option<String>,
    pub(crate) workspace_root: Option<String>,
    pub(crate) workspace_id: Option<String>,
    pub(crate) worktree_id: Option<String>,
}

pub(crate) enum CtxOutcome {
    Ok(JsCtx),
    /// resolveRootsCore threw (WorktreeLinkInvalidError or a raw stat error).
    Threw,
}

/// provenance: state.mjs resolveRootsCore + resolveContext (workspace-id
/// slice). Returns Threw where Node would throw.
pub(crate) fn resolve_context(cwd: &str) -> R<CtxOutcome> {
    // Nearest onboarding-marker-without-git.
    let mut nearest = np_resolve1(cwd)?;
    loop {
        let n = Path::new(&nearest);
        if n.join(".bee").join("onboarding.json").exists() && !n.join(".git").exists() {
            return finish_ordinary(&nearest);
        }
        let parent = np_dirname(&nearest);
        if parent == nearest {
            break;
        }
        nearest = parent;
    }
    // locateGitRoot.
    let mut located: Option<(String, String)> = None;
    let mut dir = np_resolve1(cwd)?;
    loop {
        if Path::new(&dir).join(".git").exists() {
            let marker = Path::new(&dir).join(".git").to_string_lossy().into_owned();
            located = Some((dir.clone(), marker));
            break;
        }
        let parent = np_dirname(&dir);
        if parent == dir {
            break;
        }
        dir = parent;
    }
    let (work_root, marker) = match located {
        Some(pair) => pair,
        None => {
            // onboarding-marker-anywhere fallback.
            let mut d = np_resolve1(cwd)?;
            loop {
                if Path::new(&d).join(".bee").join("onboarding.json").exists() {
                    return finish_ordinary(&d);
                }
                let parent = np_dirname(&d);
                if parent == d {
                    break;
                }
                d = parent;
            }
            return Ok(CtxOutcome::Ok(JsCtx::default()));
        }
    };
    let marker_stat = match std::fs::metadata(Path::new(&marker)) {
        Ok(s) => s,
        Err(_) => return Ok(CtxOutcome::Threw), // statSync throw (broken symlink .git)
    };
    if !marker_stat.is_file() {
        return finish_ordinary(&work_root);
    }
    // Linked-worktree validation.
    let gitdir = match read_gitdir_file(Path::new(&marker), &work_root)? {
        Some(g) => g,
        None => return Ok(CtxOutcome::Threw), // WorktreeLinkInvalidError
    };
    let worktrees_root = np_resolve2(&gitdir, "..")?;
    let common_git_dir = np_resolve2(&worktrees_root, "..")?;
    if np_basename(&common_git_dir) != ".git" || np_basename(&worktrees_root) != "worktrees" {
        return Ok(CtxOutcome::Threw);
    }
    let id = np_basename(&gitdir);
    if id.is_empty() || id == "." || id == ".." {
        return Ok(CtxOutcome::Threw);
    }
    let reverse = read_gitdir_file(&Path::new(&gitdir).join("gitdir"), &gitdir)?;
    let marker_resolved = np_resolve1(&marker)?;
    match reverse {
        Some(r) if np_resolve1(&r)? == marker_resolved => {}
        _ => return Ok(CtxOutcome::Threw),
    }
    let main_root = np_dirname(&common_git_dir);
    let grants = read_grants(&Path::new(&main_root).join(".bee"));
    let granted = grants.get(&id) == Some(&Value::Bool(true));
    // resolveContext tail (linked branch).
    // resolveProductRoot(workspaceRoot). sfg-4: the READ stays — it is what
    // prints bee's own corrupt-config line, and that output is pinned. The
    // product_root CHECK that followed it is gone; see the note where it used
    // to live for why a product_root the guard cannot resolve decides nothing.
    let _config = read_config(Path::new(&work_root))?;
    Ok(CtxOutcome::Ok(JsCtx {
        control_root: Some(main_root),
        workspace_root: Some(work_root),
        workspace_id: Some(if granted { id.clone() } else { "main".into() }),
        worktree_id: Some(id),
    }))
}

pub(crate) fn finish_ordinary(root: &str) -> R<CtxOutcome> {
    // resolveContext tail for an ordinary checkout: gitCommonDir stat can
    // throw only for exotic .git states — statSync inside resolveContext is
    // guarded by existsSync first; a race is Nd-irrelevant here.
    // sfg-4: read kept for its corrupt-config warning, product_root check
    // dropped — same reason as the linked branch above.
    let _config = read_config(Path::new(root))?;
    Ok(CtxOutcome::Ok(JsCtx {
        control_root: Some(root.to_string()),
        workspace_root: Some(root.to_string()),
        workspace_id: Some("main".into()),
        worktree_id: None,
    }))
}

/// provenance: state.mjs controlRootFor — resolveContext(root).controlRoot ??
/// root.
///
/// sfg-4: infallible by signature. `Err(Nd)` on `CtxOutcome::Threw` (and on
/// every error `resolve_context` could raise on the way) walked straight out
/// of `resolve_write_record` into `emit_undecidable` — one broken `.git`
/// line, or one unreadable config value, and the write guard stopped running
/// on EVERY path. That is the fail-OPEN sfg-3 closed for the claim readers;
/// this is the same class, one frame up.
///
/// The safe read is the root already in hand — the exact fallback the `??
/// root` in the Ok arm takes. A context the reader cannot resolve is no
/// evidence about WHERE the store lives, so it may not be read as "the store
/// lives somewhere else". The only consumer is `resolve_write_record`, which
/// uses this answer to LOOK UP the acting session and its claims; a root that
/// finds neither falls back to the control-root default record, which is the
/// strictest record on offer (it keeps `source == "default"`, so the intake
/// gate and the workspace-ownership deny both stay in front of this session).
/// So this fallback can only ever be as restrictive as the resolved answer,
/// never more permissive — and it never stops the guard.
///
/// Pinned by `sfg4_an_unresolvable_context_never_stops_the_guard`.
pub(crate) fn control_root_for_state(root: &str) -> String {
    match resolve_context(root) {
        Ok(CtxOutcome::Ok(ctx)) => ctx.control_root.unwrap_or_else(|| root.to_string()),
        // Threw, or a context the reader could not resolve at all.
        _ => root.to_string(),
    }
}

// ─── claims.mjs ports ──────────────────────────────────────────────────────

pub(crate) fn sessions_dir(root: &str) -> PathBuf {
    Path::new(root).join(".bee").join("sessions")
}

pub(crate) fn plain_id_ok(id: &str) -> bool {
    let t = js_trim(id);
    !t.is_empty() && !t.contains('/') && !t.contains('\\') && !t.contains("..")
}

/// provenance: claims.mjs readSession (strict=false). Corrupt → warn once and
/// read as None; malformed id / missing / shape mismatch → None.
pub(crate) fn read_session(root: &str, session_id: &str) -> R<Option<Map<String, Value>>> {
    if !plain_id_ok(session_id) {
        return Ok(None);
    }
    let file = sessions_dir(root).join(format!("{}.json", js_trim(session_id)));
    let parsed = read_json_g(&file)?;
    match parsed {
        Some(Value::Object(m)) => {
            if m.get("id") == Some(&Value::String(js_trim(session_id).to_string())) {
                Ok(Some(m))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

/// provenance: claims.mjs readSession (strict=true): a parse error or a
/// non-ENOENT read error THROWS in Node (F1).
///
/// It used to be `Nd` here, and `Nd` on this path is a FAIL-OPEN: the only
/// caller sits on the hook's own `R<..>` path, so the error walked out to
/// `emit_undecidable` — exit 0, "the guard did NOT run on it" — on a real
/// Edit or Write. sfg-5 closed that. The `Err(Nd)` is still returned here,
/// but it is no longer a delegation: `list_session_records_strict` turns it
/// into the FILE that could not be read, and
/// `is_shared_nested_checkout_target` turns that into bee's own DENY.
///
/// DELIBERATE DEPARTURE from Node parity, recorded at cell sfg-5: Node's
/// typed detection-error deny carries a V8-worded crash log this port cannot
/// replicate, so sfg-4 left the branch delegating rather than approximate
/// that wording. Wording is not worth a hole. bee refuses in its own words
/// instead — see `unreadable_session_refusal` (hook_local.rs) — because a
/// guard never falls open on data it merely read.
pub(crate) fn read_session_strict(root: &str, session_id: &str) -> R<Option<Map<String, Value>>> {
    if !plain_id_ok(session_id) {
        return Ok(None);
    }
    let file = sessions_dir(root).join(format!("{}.json", js_trim(session_id)));
    let text = match std::fs::read(&file) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) if io_err_is_enoent(&e) => return Ok(None),
        Err(_) => return Err(Nd),
    };
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let parsed: Value = serde_json::from_str(text).map_err(|_| Nd)?;
    match parsed {
        Value::Object(m) if m.get("id") == Some(&Value::String(js_trim(session_id).to_string())) => {
            Ok(Some(m))
        }
        _ => Ok(None),
    }
}

/// The `.json` file names under `<root>/.bee/sessions`, or `Err` for a
/// non-ENOENT `readdir` failure (which THROWS in Node, F1). Shared by both
/// scans below so the directory walk lives in exactly one place.
fn session_file_names(root: &str) -> Result<Vec<String>, ()> {
    let entries = match std::fs::read_dir(sessions_dir(root)) {
        Ok(e) => e,
        Err(e) if io_err_is_enoent(&e) => return Ok(Vec::new()),
        Err(_) => return Err(()),
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".json") {
            names.push(name);
        }
    }
    // fs.readdirSync returns sorted order on most platforms; Node does not
    // re-sort, but iteration order only affects which record is seen first —
    // all our consumers are order-independent predicates (some/filter).
    Ok(names)
}

/// provenance: claims.mjs listSessionRecords (strict=false).
///
/// sfg-5: infallible by signature, and it always was in fact — the non-strict
/// reader answers `Ok` for a missing directory AND for a corrupt record
/// (`read_json_g` swallows both). Dropping the `strict` flag splits the two
/// readers apart, so the STRICT one can carry the extra fact its caller needs
/// (WHICH file failed) instead of a bare error.
pub(crate) fn list_session_records(root: &str) -> Vec<Map<String, Value>> {
    let names = session_file_names(root).unwrap_or_default();
    let mut out = Vec::new();
    for name in names {
        let stem = &name[..name.len() - ".json".len()];
        if let Ok(Some(r)) = read_session(root, stem) {
            out.push(r);
        }
    }
    out
}

/// provenance: claims.mjs listSessionRecords (strict=true).
///
/// `Err(path)` names the ONE session file that is present but could not be
/// read or parsed. sfg-5: that used to be `Err(Nd)` — a delegation, i.e. the
/// whole write guard falling open on a real write. The caller turns this path
/// into a deny that names the file; see `read_session_strict`.
pub(crate) fn list_session_records_strict(root: &str) -> Result<Vec<Map<String, Value>>, PathBuf> {
    let names = session_file_names(root).map_err(|()| sessions_dir(root))?;
    let mut out = Vec::new();
    for name in names {
        let stem = &name[..name.len() - ".json".len()];
        match read_session_strict(root, stem) {
            Ok(Some(r)) => out.push(r),
            Ok(None) => {}
            Err(Nd) => return Err(sessions_dir(root).join(&name)),
        }
    }
    Ok(out)
}

pub(crate) const HEARTBEAT_STALE_SECONDS: f64 = 900.0;

pub(crate) fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

/// provenance: claims.mjs heartbeatStale.
///
/// sfg-4: infallible by signature, like sfg-3's claim readers. It read the
/// stamp through `date_parse_ms(..)?`, and `check_workspace_ownership` `?`d
/// the result, so ONE malformed `last_heartbeat` — in the WORKSPACE OWNER's
/// session record, a record the acting session neither owns nor can see —
/// reached `emit_undecidable`: exit 0, "the guard did NOT run on it". The
/// same reader is read by `is_concurrent_mode` and
/// `active_worker_session_ids`, so any session file in the store could do it.
///
/// Three answers, and the third is the new one:
///
/// - a stamp that parses answers the honest arithmetic;
/// - NO stamp (absent, null, blank) is stale — claims.mjs `heartbeatStale`
///   says so, and it is right: a session that never beat is not beating;
/// - a stamp the reader CANNOT PARSE is NOT stale. An unreadable byte is no
///   evidence the session went away — it is evidence about the byte. Reading
///   it as staleness would invent a death.
///
/// That third answer is also the safe one at every call site, in the same
/// direction: "not stale" means live, and live is what makes
/// `check_workspace_ownership` REFUSE the write, what puts
/// `is_concurrent_mode` into concurrent mode, and what makes
/// `active_worker_session_ids` count the worker. So an unparseable heartbeat
/// costs the guard teeth nowhere — it can only ever add them, and it can
/// never again switch the guard off.
///
/// sfg-5 (the cost of that third answer, paid instead of undone): "not stale"
/// has NO time-based self-heal. A session record carrying
/// `"last_heartbeat": "just now"` reads live forever, so it can hold a
/// workspace — and refuse every other session in the checkout — until a human
/// touches the file. sfg-4's read stands (reopening the door would restore
/// the fail-open), but the refusal it causes may never be SILENT: the reader
/// names the file and the remedy, once per file per evaluation, on the same
/// buffered stderr channel the corrupt-JSON warning already uses. A human who
/// is refused sees WHY instead of guessing.
///
/// Pinned by `sfg4_a_malformed_owner_heartbeat_never_stops_the_guard`,
/// `sfg4_a_malformed_heartbeat_never_stops_the_live_worker_count` and
/// `sfg5_an_unparseable_heartbeat_lockout_is_never_silent`.
pub(crate) fn heartbeat_stale(root: &str, session: &Map<String, Value>, now: f64) -> bool {
    match date_parse_ms(session.get("last_heartbeat")) {
        Ok(None) => true, // no beat at all
        Ok(Some(ms)) => ms + HEARTBEAT_STALE_SECONDS * 1000.0 <= now,
        Err(_) => {
            // present but unreadable: not evidence of staleness — but say so.
            warn_unreadable_heartbeat(root, session);
            false
        }
    }
}

/// The one line a locked-out human needs: which record reads live, why, and
/// what clears it.
fn warn_unreadable_heartbeat(root: &str, session: &Map<String, Value>) {
    let file = match session.get("id") {
        Some(Value::String(id)) if plain_id_ok(id) => {
            sessions_dir(root).join(format!("{}.json", js_trim(id))).display().to_string()
        }
        // A record with no usable id cannot be named; the directory still is.
        _ => sessions_dir(root).display().to_string(),
    };
    queue_guard_warning_once(format!(
        "bee: the session record {} carries a last_heartbeat this reader cannot parse, so that \
session counts as LIVE and keeps its claim on this checkout — it will not time out on its own. \
FIX: repair or delete that file (`bee state session release` writes a clean record for a session \
that is finished), then retry.\n",
        file
    ));
}

/// provenance: claims.mjs isConcurrentMode (strict=false).
/// sfg-5: infallible — `list_session_records` never errors.
pub(crate) fn is_concurrent_mode(root: &str, exclude: Option<&str>) -> bool {
    concurrent_among(list_session_records(root), exclude, root)
}

/// provenance: claims.mjs isConcurrentMode (strict=true).
///
/// `Err(path)` names the session file that is present but unreadable. sfg-5:
/// this is the fail-open sfg-4 left. The one caller
/// (`is_shared_nested_checkout_target`) turns this into a deny, never a
/// delegation — an unreadable record is not permission to stop guarding.
pub(crate) fn is_concurrent_mode_strict(root: &str, exclude: Option<&str>) -> Result<bool, PathBuf> {
    Ok(concurrent_among(list_session_records_strict(root)?, exclude, root))
}

fn concurrent_among(sessions: Vec<Map<String, Value>>, exclude: Option<&str>, root: &str) -> bool {
    let exclude = exclude.map(js_trim).unwrap_or("");
    let now = now_ms();
    for session in sessions {
        if matches!(session.get("status"), Some(Value::String(s)) if s == "closed" || s == "dead") {
            continue; // a closed/dead session is never counted toward concurrent mode.
        }
        let id_matches = session.get("id") == Some(&Value::String(exclude.to_string()));
        if !id_matches && !heartbeat_stale(root, &session, now) {
            return true;
        }
    }
    false
}

/// provenance: claims.mjs activeWorkers — reduced to the live session-id
/// view resolveLiveWorkerCount consumes (lane/cell fields are dead there),
/// but the claims-directory scan is still performed so a corrupt claim file
/// gets the same one-line warning Node's readJson gave it.
pub(crate) fn active_worker_session_ids(control_root: &str, exclude: Option<&str>) -> R<Vec<String>> {
    let exclude = exclude.map(js_trim).unwrap_or("");
    let now = now_ms();
    let mut live: Vec<String> = Vec::new();
    for session in list_session_records(control_root) {
        let id = match session.get("id") {
            Some(Value::String(s)) => s.clone(),
            _ => continue,
        };
        if matches!(session.get("status"), Some(Value::String(s)) if s == "closed" || s == "dead") {
            continue; // a closed/dead session never counts as an active worker.
        }
        if id != exclude && !heartbeat_stale(control_root, &session, now) {
            live.push(id);
        }
    }
    if live.is_empty() {
        return Ok(Vec::new());
    }
    // Claims scan (side-effect parity: corrupt claim JSON warns in Node).
    let claims_dir = Path::new(control_root).join(".bee").join("claims");
    if let Ok(entries) = std::fs::read_dir(&claims_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let stem = &name[..name.len() - ".json".len()];
            if !plain_id_ok(stem) {
                continue; // requireId throw → caught/skipped in Node
            }
            read_json_g(&claims_dir.join(&name))?; // Corrupt → warn, read as no claim
        }
    }
    Ok(live)
}

// ─── sfg-1 (slp-followup-gaps D1): this session's OWN live claims ──────────
//
// claims.mjs readClaim/isClaimActive, reduced to the single question the
// write guard asks — "which feature is this session actually working under?"
// Every read here is fail-quiet: an unreadable, foreign, expired or
// cell-less claim contributes nothing and NEVER produces a refusal. The
// loud, typed lane refusals belong to a session that DECLARED a lane; a
// derived one gets silence and the default record.
//
// sfg-3 makes that fail-quiet STRUCTURAL: every function below is
// infallible — no `?`, no `R<..>`, nothing to propagate. These readers sit
// under `resolve_write_record`, and an error escaping one of them reached
// `hooks/mod.rs`'s `emit_undecidable`: exit 0, "the guard did NOT run on
// it", for every path including `.bee` mutations. So ONE unreadable byte in
// this session's own claim would have switched the WHOLE write guard off —
// a guard failing OPEN on malformed store data. A claim the reader cannot
// understand contributes nothing, or reads as active; it never decides the
// guard's fate.

pub(crate) fn claims_dir_g(control_root: &str) -> PathBuf {
    Path::new(control_root).join(".bee").join("claims")
}

/// provenance: claims.mjs isClaimExpired/isClaimActive — a claim carrying no
/// usable ttl or no parseable timestamp reads as ACTIVE, never as expired.
///
/// sfg-3: "no parseable timestamp" means EVERY shape `date_parse_ms` cannot
/// turn into milliseconds, its `Err(Nd)` arms included — a non-RFC3339
/// string, a numeric epoch, an object, a bool. That error used to escape
/// through `?` and take the whole guard down with it (see the module note
/// above); it is swallowed here instead, which is exactly what this
/// function's own contract already promised.
pub(crate) fn claim_active(claim: &Map<String, Value>, now: f64) -> bool {
    let ttl = match claim.get("ttl_seconds").and_then(|v| v.as_f64()) {
        Some(t) if t.is_finite() && t > 0.0 => t,
        _ => return true,
    };
    match date_parse_ms(claim.get("claimed_at")) {
        Ok(Some(ms)) => ms + ttl * 1000.0 > now,
        // Ok(None) = absent/null/blank; Err(Nd) = present but unreadable.
        // Both are "no parseable timestamp", and both read as ACTIVE.
        _ => true,
    }
}

/// The DISTINCT features named by the live claims THIS session owns, in
/// sorted claim-file order. A claim owned by another session is never read;
/// an expired one, a corrupt one, one whose cell record is missing, and one
/// whose cell names no feature each contribute nothing.
pub(crate) fn session_claimed_features(control_root: &str, session_id: &str) -> Vec<String> {
    let sid = js_trim(session_id);
    if !plain_id_ok(sid) {
        return Vec::new();
    }
    let dir = claims_dir_g(control_root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(), // no claims directory at all
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".json") {
            names.push(name);
        }
    }
    names.sort(); // read_dir order is platform-dependent; the answer must not be
    let now = now_ms();
    let mut features: Vec<String> = Vec::new();
    for name in names {
        let stem = &name[..name.len() - ".json".len()];
        if !plain_id_ok(stem) {
            continue; // requireId throw → skipped, exactly like activeWorkers
        }
        // sfg-3: `unwrap_or(None)` rather than `?`. `read_json_g` answers
        // `Ok(None)` for missing and corrupt today, but this reader must not
        // depend on a distant function staying infallible — an unreadable
        // claim contributes nothing, and can never reach the caller as an
        // error.
        let claim = match read_json_g(&dir.join(&name)).unwrap_or(None) {
            Some(Value::Object(m)) => m,
            _ => continue, // missing, corrupt, or not an object
        };
        if claim.get("session") != Some(&Value::String(sid.to_string())) {
            continue; // another session's claim is never this session's lane
        }
        if !claim_active(&claim, now) {
            continue;
        }
        // The record's own `cell` field is the authority; the filename stem is
        // the fallback the store's own writers keep in step with it.
        let cell_id = match claim.get("cell") {
            Some(Value::String(c)) if plain_id_ok(c) => js_trim(c).to_string(),
            _ => stem.to_string(),
        };
        let cell_file = Path::new(control_root)
            .join(".bee")
            .join("cells")
            .join(format!("{}.json", cell_id));
        let feature = match read_json_g(&cell_file).unwrap_or(None) {
            Some(Value::Object(m)) => match m.get("feature") {
                Some(Value::String(f)) if !js_trim(f).is_empty() => js_trim(f).to_string(),
                _ => continue,
            },
            _ => continue, // sfg-3: an unreadable cell record names no feature
        };
        if !features.contains(&feature) {
            features.push(feature);
        }
    }
    features
}

// ─── reservations.mjs + lease-store.mjs read ports ─────────────────────────

pub(crate) const SESSIONLESS_SESSION_ID: &str = "\u{0}bee-reservation-sessionless\u{0}";

/// provenance: reservations.mjs normalizePath (== lease-store
/// canonicalizePath).
pub(crate) fn res_normalize_path(v: &str) -> String {
    let mut s = v.replace('\\', "/");
    // strip ONE leading "./" run: /^\.\/+/
    if s.starts_with("./") {
        let rest = s[1..].trim_start_matches('/');
        s = rest.to_string();
    }
    // strip trailing slashes: /\/+$/
    while s.ends_with('/') {
        s.pop();
    }
    s
}

pub(crate) fn res_normalize_value(v: Option<&Value>) -> String {
    // String(value || '') — falsy → ''.
    match v {
        Some(val) if truthy(val) => res_normalize_path(&js_disp(val)),
        _ => String::new(),
    }
}

/// provenance: reservations.mjs pathsOverlap.
pub(crate) fn paths_overlap(a: &str, b: &str) -> bool {
    let left = res_normalize_path(a);
    let right = res_normalize_path(b);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left == right {
        return true;
    }
    let strip = |s: &str| -> String {
        if s.ends_with('*') {
            let mut t = s.trim_end_matches('*').to_string();
            while t.ends_with('/') {
                t.pop();
            }
            t
        } else {
            s.to_string()
        }
    };
    let lb = strip(&left);
    let rb = strip(&right);
    if lb == rb {
        return true;
    }
    if lb.is_empty() || rb.is_empty() {
        return true;
    }
    lb.starts_with(&format!("{}/", rb)) || rb.starts_with(&format!("{}/", lb))
}

/// provenance: reservations.mjs findMainRoot/controlRootFor — the
/// self-contained, never-throwing main-root walk.
pub(crate) fn control_root_for_res(root: &str) -> String {
    (|| -> Option<String> {
        // locateGitRootForRoot
        let mut dir = np_resolve1(root).ok()?;
        let (work_root, marker) = loop {
            let m = Path::new(&dir).join(".git");
            if m.exists() {
                break (dir.clone(), m);
            }
            let parent = np_dirname(&dir);
            if parent == dir {
                return None;
            }
            dir = parent;
        };
        let is_file = std::fs::metadata(&marker).ok()?.is_file();
        if !is_file {
            return Some(work_root);
        }
        let read_ptr = |file: &Path, base: &str| -> Option<String> {
            let raw = std::fs::read_to_string(file).ok()?;
            let mut raw = js_trim(&raw);
            if let Some(rest) = raw.strip_prefix("gitdir:") {
                raw = js_trim(rest);
            }
            if raw.is_empty() {
                return None;
            }
            let fixed = if cfg!(windows) { raw.to_string() } else { raw.replace('\\', "/") };
            np_resolve2(base, &fixed).ok()
        };
        let gitdir = read_ptr(&marker, &work_root)?;
        let worktrees_root = np_resolve2(&gitdir, "..").ok()?;
        let common_git_dir = np_resolve2(&worktrees_root, "..").ok()?;
        if np_basename(&common_git_dir) != ".git" || np_basename(&worktrees_root) != "worktrees" {
            return None;
        }
        let id = np_basename(&gitdir);
        if id.is_empty() || id == "." || id == ".." {
            return None;
        }
        let reverse = read_ptr(&Path::new(&gitdir).join("gitdir"), &gitdir)?;
        let marker_s = marker.to_string_lossy().into_owned();
        if np_resolve1(&reverse).ok()? != np_resolve1(&marker_s).ok()? {
            return None;
        }
        Some(np_dirname(&common_git_dir))
    })()
    .unwrap_or_else(|| root.to_string())
}

/// provenance: lease-store.mjs listAllLeaseFiles + readLeaseSafe (silent
/// skip on corrupt — no warn, so no Nd) filtered to path-type leases
/// (reservations.mjs listPathLeaseRecords).
pub(crate) fn list_path_lease_records(root: &str) -> Vec<Map<String, Value>> {
    let control = control_root_for_res(root);
    let leases_root = Path::new(&control).join(".bee").join("runtime").join("leases");
    let mut out = Vec::new();
    for dir in [leases_root.join("cells"), leases_root.join("paths")] {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            if !is_file {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(dir.join(&name)) {
                if let Ok(Value::Object(m)) = serde_json::from_str::<Value>(&text) {
                    let is_path = matches!(m.get("resource"), Some(Value::String(s)) if s.starts_with("path:"));
                    if is_path {
                        out.push(m);
                    }
                }
            }
        }
    }
    out
}

/// The reservation-shape view of a path lease. provenance: reservations.mjs
/// leaseToReservation / leaseAgent / leaseTtlSeconds.
pub(crate) struct Resv {
    pub(crate) agent: Option<Value>,       // workspace_id minus "agent:" prefix (any JSON type)
    pub(crate) cell: Option<Value>,        // workflow_id
    pub(crate) path: String,               // resource minus "path:"
    pub(crate) ttl_seconds: Option<f64>,   // None = NaN
    pub(crate) reserved_at: Option<Value>, // acquired_at
    pub(crate) session: Option<Value>,     // present only when non-sentinel truthy
    pub(crate) kind: Value,                // kind || 'lease'
}

/// sfg-5: infallible by signature, like sfg-3's claim readers and sfg-4's
/// heartbeat reader. It read BOTH `expires_at` and `acquired_at` through
/// `date_parse_ms(..)?`, and `list_active_reservations` `?`d the result, so
/// ONE lease file carrying `"expires_at": "tomorrow"` reached
/// `emit_undecidable` — exit 0, "the guard did NOT run on it" — on every path
/// `check_write` judges. `list_path_lease_records` validates no timestamp at
/// all, so any well-formed JSON object under
/// `.bee/runtime/leases/{cells,paths}/` could do it.
///
/// A stamp the reader cannot parse contributes `None` — the same NaN JS
/// already produced when `Math.round`/`Math.max` were handed a NaN, and the
/// same value an absent stamp produces. `ttl_seconds` is read in exactly one
/// place, `hold_expiry`'s DISPLAY clause; it decides no verdict, so an
/// unreadable stamp costs the guard no teeth here. Whether the lease is live
/// at all is `lease_record_expired`'s question, and that one IS restrictive.
pub(crate) fn lease_to_reservation(rec: &Map<String, Value>) -> Resv {
    let resource = match rec.get("resource") {
        Some(Value::String(s)) => s.clone(),
        _ => unreachable!("filtered to path leases"),
    };
    let ttl = match rec.get("expires_at") {
        None | Some(Value::Null) => Some(0.0),
        Some(exp) => match (date_parse_ms(Some(exp)), date_parse_ms(rec.get("acquired_at"))) {
            (Ok(Some(e)), Ok(Some(a))) => Some(js_round((e - a) / 1000.0).max(0.0)),
            // NaN through Math.max/round — absent, blank, or unreadable.
            _ => None,
        },
    };
    let agent = rec.get("workspace_id").map(|w| match w {
        Value::String(s) if s.starts_with("agent:") => Value::String(s["agent:".len()..].to_string()),
        other => other.clone(),
    });
    let session = match rec.get("session_id") {
        Some(v) if truthy(v) && v != &Value::String(SESSIONLESS_SESSION_ID.to_string()) => {
            Some(v.clone())
        }
        _ => None,
    };
    let kind = match rec.get("kind") {
        Some(v) if truthy(v) => v.clone(),
        _ => Value::String("lease".into()),
    };
    Resv {
        agent,
        cell: rec.get("workflow_id").cloned(),
        path: resource["path:".len()..].to_string(),
        ttl_seconds: ttl,
        reserved_at: rec.get("acquired_at").cloned(),
        session,
        kind,
    }
}

/// provenance: reservations.mjs isLeaseRecordExpired.
///
/// sfg-5: infallible, and RESTRICTIVE where it now has a choice. The three
/// answers:
///
/// - an expiry that parses answers the honest comparison;
/// - NO expiry (absent, null, blank) is not expired — the lease is open-ended,
///   as reservations.mjs always said;
/// - an expiry the reader CANNOT PARSE is NOT expired either. That is the
///   deliberate call at this site. An unreadable byte is evidence about the
///   byte, never that the hold went away, and "expired" is the answer that
///   REMOVES a reservation from `list_active_reservations` — i.e. the answer
///   that silently drops another agent's or another session's claim on the
///   path. Reading it as "still held" keeps the conflict, keeps the deny, and
///   costs the guard no teeth: the worst case is one refusal a human can
///   clear by fixing the file the refusal names.
pub(crate) fn lease_record_expired(rec: &Map<String, Value>, now: f64) -> bool {
    match rec.get("expires_at") {
        None | Some(Value::Null) => false,
        Some(v) => match date_parse_ms(Some(v)) {
            Ok(Some(ms)) => ms <= now,
            Ok(None) => false, // blank/null stamp: no expiry
            Err(_) => false,   // unreadable: not evidence the hold lapsed
        },
    }
}

/// provenance: reservations.mjs listReservations({activeOnly:true}).
/// sfg-5: infallible — see `lease_record_expired` / `lease_to_reservation`.
pub(crate) fn list_active_reservations(root: &str) -> Vec<Resv> {
    let now = now_ms();
    let mut out = Vec::new();
    for rec in list_path_lease_records(root) {
        if !lease_record_expired(&rec, now) {
            out.push(lease_to_reservation(&rec));
        }
    }
    out
}

/// provenance: reservations.mjs findConflicts.
pub(crate) fn find_conflicts(root: &str, agent: &str, paths: &[String]) -> Vec<Resv> {
    if paths.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for resv in list_active_reservations(root) {
        let same_agent = matches!(&resv.agent, Some(Value::String(s)) if s == agent);
        if !same_agent && paths.iter().any(|p| paths_overlap(&resv.path, p)) {
            out.push(resv);
        }
    }
    out
}

/// provenance: reservations.mjs findSessionConflicts.
pub(crate) fn find_session_conflicts(root: &str, session_id: &str, paths: &[String]) -> Vec<Resv> {
    if paths.is_empty() {
        return Vec::new();
    }
    let acting = js_trim(session_id);
    let mut out = Vec::new();
    for resv in list_active_reservations(root) {
        let sess_ok = match &resv.session {
            Some(Value::String(s)) if !js_trim(s).is_empty() && s != acting => true,
            _ => false,
        };
        if sess_ok && paths.iter().any(|p| paths_overlap(&resv.path, p)) {
            out.push(resv);
        }
    }
    out
}

/// provenance: reservations.mjs isHardConflict.
pub(crate) fn is_hard_conflict(resv: &Resv, target: &str) -> bool {
    !(resv.kind == Value::String("intent".into())
        && res_normalize_path(&resv.path) != res_normalize_path(target))
}

/// provenance: guards.mjs reservationStoreCorrupt.
pub(crate) fn reservation_store_corrupt(root: &str) -> bool {
    let file = Path::new(root).join(".bee").join("reservations.json");
    if !file.exists() {
        return false;
    }
    match std::fs::read_to_string(&file) {
        Ok(text) => serde_json::from_str::<Value>(&text).is_err(),
        Err(_) => true, // readFileSync throw is caught → corrupt
    }
}

// ─── worktree-holds.mjs ports ──────────────────────────────────────────────

pub(crate) fn holds_ledger_path(main_root: &str) -> PathBuf {
    Path::new(main_root).join(".bee").join("runtime").join("cross-worktree-holds.json")
}

/// provenance: worktree-holds.mjs holdsStoreCorrupt.
pub(crate) fn holds_store_corrupt(main_root: &str) -> bool {
    let file = holds_ledger_path(main_root);
    if !file.exists() {
        return false;
    }
    match std::fs::read_to_string(&file) {
        Ok(text) => serde_json::from_str::<Value>(&text).is_err(),
        Err(_) => true,
    }
}

/// provenance: worktree-holds.mjs findForeignHolds (+ isActive/isExpired).
///
/// sfg-5: infallible. It read `mirrored_at` through `date_parse_ms(..)?`, and
/// `check_write` `?`d the result, so one mirrored hold row with an unreadable
/// stamp switched the WHOLE write guard off — the lease defect's twin, in the
/// cross-worktree ledger. Same restrictive read as `lease_record_expired`: a
/// `mirrored_at` the reader cannot parse is NOT expired, so the hold stays
/// active and keeps denying.
pub(crate) fn find_foreign_holds(main_root: &str, holder: &str, paths: &[String]) -> Vec<Map<String, Value>> {
    if paths.is_empty() {
        return Vec::new();
    }
    // `read_json_g` has no Err arm (missing AND corrupt both read as `None`);
    // flattening it keeps that fact local instead of re-exporting an error
    // this reader can never produce.
    let store = read_json_g(&holds_ledger_path(main_root)).ok().flatten();
    let holds = match store {
        Some(Value::Object(m)) => match m.get("holds") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    let acting = js_trim(holder);
    let now = now_ms();
    let mut out = Vec::new();
    for hold in holds {
        let hold = match hold {
            Value::Object(m) => m,
            _ => continue, // property reads on a non-object entry — a non-
                           // object array element would throw in Node only on
                           // null property access; entry.released_at on a
                           // string is undefined (== null → active)… model:
        };
        // released_at == null (JS loose) → active half
        let released_null = matches!(hold.get("released_at"), None | Some(Value::Null));
        if !released_null {
            continue;
        }
        // isExpired
        let ttl = hold.get("ttl_seconds").and_then(Value::as_f64);
        let expired = match ttl {
            Some(t) if t > 0.0 => match date_parse_ms(hold.get("mirrored_at")) {
                Ok(Some(m)) => m + t * 1000.0 <= now,
                // Absent OR unreadable: not evidence the hold lapsed.
                Ok(None) | Err(_) => false,
            },
            _ => false,
        };
        if expired {
            continue;
        }
        let holder_matches = matches!(hold.get("holder"), Some(Value::String(s)) if s == acting);
        if holder_matches {
            continue;
        }
        let hold_path = res_normalize_value(hold.get("path"));
        let _ = hold_path; // pathsOverlap normalizes again; use raw coercion:
        let hp = match hold.get("path") {
            Some(v) => js_disp(v),
            None => String::new(),
        };
        if paths.iter().any(|p| paths_overlap(&hp, p)) {
            out.push(hold);
        }
    }
    out
}

/// The expiry clause for a record whose timestamp the reader cannot turn into
/// a date. sfg-5: this clause is DISPLAY, never a verdict, so it decides
/// nothing — but it must not lie either. "no expiry" is a claim about the
/// record ("this hold never lapses"); an unreadable stamp supports no claim,
/// so it says exactly that, and the deny that carries it already names the
/// path and the holder a human needs to go look at.
const UNREADABLE_EXPIRY: &str = "expiry unknown — the timestamp on the record could not be read";

/// provenance: guards.mjs holdExpiry (reservation flavor).
/// sfg-5: infallible — one more `date_parse_ms(..)?` that reached
/// `emit_undecidable` straight from a deny message's own text.
pub(crate) fn hold_expiry(resv: &Resv) -> String {
    let ttl = match resv.ttl_seconds {
        Some(t) if t > 0.0 => t,
        _ => return "no expiry".to_string(),
    };
    match date_parse_ms(resv.reserved_at.as_ref()) {
        Ok(Some(r)) => match ms_to_iso(r + ttl * 1000.0) {
            Ok(iso) => format!("expires {iso}"),
            Err(_) => UNREADABLE_EXPIRY.to_string(), // beyond the JS Date range
        },
        Ok(None) => "no expiry".to_string(), // absent/blank stamp, as before
        Err(_) => UNREADABLE_EXPIRY.to_string(),
    }
}

/// provenance: guards.mjs foreignHoldExpiry.
/// sfg-5: infallible, same shape and same reason as `hold_expiry`.
pub(crate) fn foreign_hold_expiry(hold: &Map<String, Value>) -> String {
    let ttl = match hold.get("ttl_seconds").and_then(Value::as_f64) {
        Some(t) if t > 0.0 => t,
        _ => return "no expiry".to_string(),
    };
    match date_parse_ms(hold.get("mirrored_at")) {
        Ok(Some(m)) => match ms_to_iso(m + ttl * 1000.0) {
            Ok(iso) => format!("expires {iso}"),
            Err(_) => UNREADABLE_EXPIRY.to_string(),
        },
        Ok(None) => "no expiry".to_string(),
        Err(_) => UNREADABLE_EXPIRY.to_string(),
    }
}

// ─── workspace-store.mjs ports ─────────────────────────────────────────────

pub(crate) enum WorkspaceRead {
    Missing,
    Corrupt,
    Ok(Map<String, Value>),
}

/// provenance: workspace-store.mjs readWorkspaceRecord (read-only slice; the
/// guard only consumes write_owner_session).
pub(crate) fn read_workspace(control_root: &str, id: &str) -> WorkspaceRead {
    if !plain_id_ok(id) {
        // requireWorkspaceId throws WORKSPACE_INVALID_ID — checkWorkspace-
        // Ownership's catch treats any non-MISSING error as corrupt.
        return WorkspaceRead::Corrupt;
    }
    let file = Path::new(control_root)
        .join(".bee")
        .join("runtime")
        .join("workspaces")
        .join(format!("{}.json", js_trim(id)));
    let text = match std::fs::read(&file) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) if io_err_is_enoent(&e) => return WorkspaceRead::Missing,
        Err(_) => return WorkspaceRead::Corrupt,
    };
    let parsed: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return WorkspaceRead::Corrupt,
    };
    let obj = match parsed {
        Value::Object(m) => m,
        _ => return WorkspaceRead::Corrupt,
    };
    if obj.get("id") != Some(&Value::String(js_trim(id).to_string())) {
        return WorkspaceRead::Corrupt;
    }
    let mut merged = Map::new();
    merged.insert("write_owner_session".into(), Value::Null);
    merged.insert("fence_epoch".into(), Value::Number(0.into()));
    merged.insert("attached_sessions".into(), Value::Array(vec![]));
    merged.insert("branch".into(), Value::Null);
    merged.insert("base_sha".into(), Value::Null);
    for (k, v) in obj {
        merged.insert(k, v);
    }
    WorkspaceRead::Ok(merged)
}
