// bee hook prompt-context — port of hooks/bee-prompt-context.mjs
// (UserPromptSubmit). Injects the 1-3 line phase/mode/next-action/gate
// reminder plus the compaction anchor nudge, deduped via the inject cache,
// and rides the throttled heartbeat + claim/hold lease renewal (D5).
//
// TWO-PHASE SHAPE. The hook runs in two phases:
//   1. plan() — a READ-ONLY preflight that performs every read the
//      pipeline needs, in order, and computes every decision (reminder
//      text/hash, inject decisions, anchor nudge). Inputs it still cannot
//      serve natively (the linked-worktree grant topology, a non-object inject
//      cache, a shape-wrong lane record) return Outcome::Delegate BEFORE any
//      output or write.
//   2. execute() — the side-effect phase in the .mjs's exact order:
//      heartbeat touch + hold renewal (fail-open, crash-logged), then
//      markInjected + stdout. Exit 0 always.
//
// CUTOVER (2026-08-01). Corrupt JSON used to be part of that delegate set,
// because Node's fsutil.mjs readJson warning quoted V8's parse sentence. Node
// is gone, so read_json_pf now prints bee's own line via
// crate::fsutil::warn_corrupt_json and returns readJson's `null` fallback —
// which every caller already treats exactly as "absent": a corrupt state.json
// reads as defaultState(), a corrupt session/claim/cell/intent record reads as
// absent (skipped, one warning per bad file), a corrupt inject cache falls
// through to the legacy location and then to `{}`, and a corrupt lane record
// takes readLane's own second warn and becomes the LANE_CORRUPT refusal, i.e.
// the reminder falls back to readState and the nudge goes silent — the same
// continuation LANE_MISSING already had.
//
// Because plan() and execute() BOTH read the session/claim stores, the two
// pure preflight passes that existed only to spot corruption early (the
// duplicate readSession pair and preflightClaimsDir) are DELETED — keeping
// them would have warned the user twice for one bad file.
//
// plan() still has to be able to delegate having emitted nothing, so warnings
// it discovers are QUEUED and released by execute() (see PLAN_WARNINGS);
// execute-phase reads warn immediately.
//
// Ported lib functions (source file → private fn), each replicated with JS
// semantics (truthiness, `??` vs `||`, template coercion, key order):
//   state.mjs   readState/defaultState        → read_state / StateRecord
//   state.mjs   coerceLegacyPhase             → coerce_legacy_phase
//   state.mjs   resolvePipeline/readLane/
//               laneRecordFrom/lanePath       → resolve_pipeline / read_lane_record
//   state.mjs   resolveRootsCore/resolveContext
//               controlRootFor (ordinary arm) → control_root_for_state
//   state.mjs   readConfig (hooks/product_root slice) → via crate::state::read_config_raw
//   claims.mjs  readSession                   → read_session
//   claims.mjs  heartbeatStale                → heartbeat_stale
//   claims.mjs  heartbeatSession              → heartbeat_session
//   claims.mjs  renewClaimTTL/acquireGate/releaseGate → renew_claim_ttl
//   claims.mjs  heartbeatTouch                → heartbeat_touch
//   lock.mjs    acquireStoreLockOnceSync/withStoreLock(maxAttempts:1)/
//               sanitizeLockName/tryStaleTakeover/isPidAlive/
//               appendContentionTelemetry     → acquire_store_lock_once & friends
//   reservations.mjs findMainRoot/controlRootFor → control_root_for_leases
//   reservations.mjs renewHoldsBySession/leaseTtlSeconds/listPathLeaseRecords
//                                             → renew_holds_by_session
//   lease-store.mjs renewLease(File)/computeExpiresAt/lockNameForFile/
//               canonicalizePath/listAllLeaseFiles → lease helpers
//   inject.mjs  buildPromptReminder/firstOpenGate/visibleGates/gatesLine
//               (reminder slice)/stableHash   → build_prompt_reminder
//   inject.mjs  shouldInject/markInjected/injectCachePath/legacyInjectCachePath
//                                             → should_inject / mark_injected
//   compaction.mjs anchorMissing/claimedCells/sessionOwnedCells/
//               attributedToSession/claimStoreCellIds/claimedCellId
//                                             → anchor_missing & friends
//   cells.mjs   listCells (top-level arm)/readCell/ID_PATTERN
//                                             → list_cells_top / read_cell
//   intent.mjs  readIntent/intentKeyCandidates/activeFeature/
//               sanitizeIntentKey/normalizeAnchor → intent_anchor_present
//
// Deliberate divergences (all documented in the port report):
//   * Delegation triggers double coverage-gap logging never happens: the
//     preflight runs on a hand-parsed payload BEFORE read_hook_context, so
//     a delegated run performs zero native reads-with-side-effects and zero
//     log writes — Node produces the one canonical set of lines.
//   * localeCompare('en') tie-breaks inside claimedCellId are approximated
//     with a case-insensitive-then-byte comparison (only reachable when two
//     claimed cells share an identical claimed_at timestamp).
//   * Read-then-write interleaving: all reads happen in the preflight, all
//     writes after — observable only under concurrent mid-hook mutation.

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::hooks::adapter::{log_crash, now_iso, read_hook_context};
use crate::hooks::Outcome;
use crate::jsjson;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HOOK_NAME: &str = "prompt-context";

// inject.mjs INJECT_INTERVAL_MS
const INJECT_INTERVAL_MS: i64 = 30 * 60 * 1000;
// claims.mjs HEARTBEAT_TOUCH_THROTTLE_SECONDS
const HEARTBEAT_TOUCH_THROTTLE_SECONDS: i64 = 60;
// lock.mjs STALE_MS / HARD_STALE_MS
const LOCK_STALE_MS: f64 = 30_000.0;
const LOCK_HARD_STALE_MS: f64 = 3_600_000.0;
// state.mjs GATE_NAMES
const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];
// inject.mjs / compaction.mjs / intent.mjs terminal-phase set
const NO_WORK_PHASES: [&str; 2] = ["idle", "compounding-complete"];
// compaction.mjs ANCHOR_NUDGE_KEY / ANCHOR_NUDGE_COMMAND
const ANCHOR_NUDGE_KEY: &str = "anchor-missing-nudge";
const ANCHOR_NUDGE_COMMAND: &str = ".bee/bin/bee intent set --request \"<the user's VERBATIM request>\" --acceptance \"<what done means>\"";

// CUTOVER: a 21-module `LIB_CLOSURE` preflight used to live here. The .mjs
// pipeline dynamically imported that closure, so a repo missing any one of
// them made Node's `await import` throw with a V8-worded crash log, and this
// port delegated rather than invent the message. The closure is compiled into
// the binary now — it cannot be partially present, so there is nothing left to
// preflight.

/// Preflight signal: this input needs the Node runtime for byte parity.
#[derive(Debug)]
struct NeedsNode;
type Pf<T> = Result<T, NeedsNode>;

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    // Hand-parsed payload FIRST (mirrors adapter normalization) so a
    // delegated run performs no native writes at all — including the
    // adapter's own coverage-gap log lines, which Node will produce once.
    clear_plan_warnings(); // one queue per run (tests reuse the thread)
    let payload = parse_payload(stdin);
    let cwd = payload
        .get("cwd")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let roots = crate::hooks::adapter::resolve_roots(&cwd);
    let Some(root) = roots.work_root.clone() else {
        // No root: Node logs nothing and exits 0 — byte-identical natively.
        return Outcome::Done(ExitCode::SUCCESS);
    };
    if !crate::hooks::adapter::bee_installed(&root) {
        // Presence gate (bee-prompt-context.mjs:59) — still run the shared
        // adapter so coverage gaps are logged exactly as Node would.
        let _ctx = read_hook_context(HOOK_NAME, argv, stdin);
        return Outcome::Done(ExitCode::SUCCESS);
    }
    if roots.worktree_resolution != "ordinary" {
        // Linked-worktree topology pulls the grants/control-root surface —
        // delegated wholesale for now.
        return Outcome::Delegate;
    }

    let plan = match plan(&root, &payload) {
        Err(NeedsNode) => return Outcome::Delegate,
        Ok(None) => {
            // Hook disabled: run the adapter for gap-log parity, then exit 0.
            // (Reached before any warn-producing read, but the queue is
            // released here too so no path can swallow a warning.)
            flush_plan_warnings();
            let _ctx = read_hook_context(HOOK_NAME, argv, stdin);
            return Outcome::Done(ExitCode::SUCCESS);
        }
        Ok(Some(p)) => p,
    };

    // Committed to native from here: the adapter read logs coverage gaps
    // exactly once, same as the .mjs's readHookContext.
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    Outcome::Done(execute(&root, &ctx, plan))
}

// ─── plan (read-only preflight + full decision computation) ─────────────────

struct Nudge {
    message: String,
    hash: String,
    inject: bool,
}

struct Plan {
    session_id: Option<String>,
    /// D2/D4 (awaiting-human, ah-2): the feature this reminder resolved
    /// against, if any — the resolved `record.feature`, whichever pipeline
    /// arm produced it. Used only to find that feature's live workflow
    /// record (if one exists) for the waiting-mark clearing pass below;
    /// `None` when no feature is bound, matching D3's session-scoped case.
    feature: Option<String>,
    reminder_text: String,
    reminder_hash: String,
    inject_key: String,
    inject_reminder: bool,
    nudge: Option<Nudge>,
}

/// Ok(None) = hook disabled (native exit 0). Err = delegate.
fn plan(root: &Path, payload: &Map<String, Value>) -> Pf<Option<Plan>> {
    // hookEnabled(root, HOOK_NAME) — state.mjs readConfig over tracked +
    // overlay; a corrupt config warns natively and reads as absent now.
    let config = crate::state::read_config_raw(root);
    if matches!(config.get("hooks").and_then(|h| h.get(HOOK_NAME)), Some(Value::Bool(false))) {
        return Ok(None);
    }

    // state.mjs controlRootFor(root): resolveContext walks resolveRootsCore
    // and reads config (product_root) at the workspace root — any branch
    // that would warn (bad product_root) delegates.
    let control_root = control_root_for_state(root)?;
    if control_root != root {
        let ctl_config = crate::state::read_config_raw(&control_root);
        if product_root_would_warn(&control_root, &ctl_config) {
            return Err(NeedsNode);
        }
    } else if product_root_would_warn(root, &config) {
        return Err(NeedsNode);
    }

    let session_id = payload
        .get("session_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    // (No session/claims pre-pass: the reads below and the heartbeat block in
    // execute() are the .mjs's own reads, and each warns for itself. A probe
    // here would double every corrupt-file warning.)

    // resolvePipeline(root, { sessionId }) — the record the reminder AND the
    // nudge read (fresh-session-handoff fsh-6 / codex-loop-p0).
    let pipeline = resolve_pipeline(root, &control_root, session_id.as_deref())?;
    let record = match &pipeline {
        Pipeline::Ok(rec) => rec.clone(),
        Pipeline::Refused => read_state(root)?,
    };
    // D2/D4 (ah-2): the resolved record's own feature, if any — carried into
    // the Plan so execute()'s waiting-mark clearing pass can find that
    // feature's live workflow record without a second pipeline resolution.
    let feature = match &record.feature {
        Value::String(s) if !s.trim().is_empty() => Some(s.clone()),
        _ => None,
    };

    // inject.mjs buildPromptReminder — fields + stableHash(sha1(JSON)).
    let (reminder_text, reminder_hash) = build_prompt_reminder(&record);
    let inject_key = match &session_id {
        Some(sid) => format!("prompt:{sid}"),
        None => "prompt".to_string(),
    };
    let cache = read_inject_cache(root)?;
    let now = now_ms();
    let inject_reminder = !reminder_text.trim().is_empty()
        && should_inject(&cache, &inject_key, &reminder_hash, now);

    // compaction.mjs anchorMissing — evaluated read-only here; the .mjs
    // computes it after the heartbeat block, but nothing the heartbeat
    // writes (session heartbeat, claim claimed_at, lease expires_at) is
    // read by the nudge predicate.
    let nudge = match anchor_missing(root, &control_root, &pipeline, session_id.as_deref())? {
        Some((message, hash)) => {
            let inject = should_inject(&cache, ANCHOR_NUDGE_KEY, &hash, now);
            Some(Nudge { message, hash, inject })
        }
        None => None,
    };

    Ok(Some(Plan {
        session_id,
        feature,
        reminder_text,
        reminder_hash,
        inject_key,
        inject_reminder,
        nudge,
    }))
}

// ─── execute (side effects in .mjs order) ───────────────────────────────────

fn execute(root: &Path, ctx: &crate::hooks::adapter::HookContext, plan: Plan) -> ExitCode {
    // Committed to native: release the warnings plan() queued. Everything read
    // from here on warns immediately (there is no delegate left to protect).
    flush_plan_warnings();

    // D5 heartbeat + hold renewal — own try/catch in the .mjs: a throw here
    // is crash-logged and never blocks the reminder.
    if let Some(sid) = &plan.session_id {
        let hb_root = ctx.control_root.clone().unwrap_or_else(|| root.to_path_buf());
        if let Err(err) = heartbeat_block(&hb_root, root, sid) {
            log_crash(Some(root), HOOK_NAME, &err, ctx.source);
        }
    }

    // D2 (awaiting-human, ah-2): the RELIABLE clearing layer — the human
    // sending anything at all clears any live waiting_on mark, because this
    // fires on a real human action rather than on the agent remembering.
    // Own try/catch per clear, exactly like heartbeat_block above: a failure
    // here is crash-logged and must NEVER fail the hook or block the
    // reminder that already printed. D4's stale-expiry reap rides the same
    // best-effort pass, on the theory that any live session's prompt is as
    // good an opportunity as any to sweep a mark left behind by a dead one.
    clear_and_reap_waiting_on_best_effort(root, ctx, plan.feature.as_deref());

    let mut parts: Vec<String> = Vec::new();
    if plan.inject_reminder {
        parts.push(plan.reminder_text.clone());
        // markInjected throw → the .mjs's OUTER catch: crash-logged, exit 0,
        // nothing printed.
        if let Err(err) = mark_injected(root, &plan.inject_key, &plan.reminder_hash) {
            log_crash(Some(root), HOOK_NAME, &err, ctx.source);
            return ExitCode::SUCCESS;
        }
    }
    if let Some(nudge) = &plan.nudge {
        if nudge.inject {
            // maybeAnchorNudge's own try/catch: a markInjected throw is
            // logged with source "anchor-nudge" and the nudge is dropped.
            match mark_injected(root, ANCHOR_NUDGE_KEY, &nudge.hash) {
                Ok(()) => parts.push(nudge.message.clone()),
                Err(err) => log_crash(Some(root), HOOK_NAME, &err, Some("anchor-nudge")),
            }
        }
    }
    if !parts.is_empty() {
        // UserPromptSubmit stdout stays PLAIN developer context — direct
        // write, never a JSON envelope, no trailing newline.
        use std::io::Write;
        let _ = std::io::stdout().write_all(parts.join("\n\n").as_bytes());
    }
    ExitCode::SUCCESS
}

// ─── waiting-mark clearing (D2/D4, ah-2) ────────────────────────────────────

/// D2's reliable clearing layer plus D4's stale-expiry reap, both best-effort
/// and both isolated per call — one failing (or the mark not existing at
/// all, which is a no-op everywhere it is checked) never skips or blocks the
/// other, and neither ever surfaces to stdout or changes the hook's exit
/// code. The mark can live in TWO places (D1/D3): the default
/// `.bee/state.json` record is always tried, and the resolved `feature`'s
/// live workflow record (if any) is tried too.
fn clear_and_reap_waiting_on_best_effort(
    root: &Path,
    ctx: &crate::hooks::adapter::HookContext,
    feature: Option<&str>,
) {
    if let Err(e) = crate::verbs::workflow_store::clear_default_state_waiting_on(root) {
        log_crash(Some(root), HOOK_NAME, &waiting_on_err_message(e), ctx.source);
    }
    if let Some(feature) = feature.filter(|f| !f.trim().is_empty()) {
        match crate::verbs::workflow_store::list_workflows(root) {
            Ok(workflows) => {
                if let Some(wf) = crate::verbs::workflow_store::find_live_workflow(&workflows, feature) {
                    let id = crate::verbs::workflow_store::wf_id(wf);
                    if let Err(e) = crate::verbs::workflow_store::clear_workflow_waiting_on(root, &id) {
                        log_crash(Some(root), HOOK_NAME, &waiting_on_err_message(e), ctx.source);
                    }
                }
            }
            Err(_) => {} // best-effort: an unreadable workflows dir is silently skipped
        }
    }
    // D4: the stale-expiry backstop for a mark left by a session that will
    // never send another prompt to clear it itself. `reap_stale_waiting_on`
    // is its own best-effort function (never returns an error to log) —
    // every record it touches is individually locked/re-verified.
    crate::verbs::workflow_store::reap_stale_waiting_on(root, now_ms() as f64);
}

fn waiting_on_err_message(e: crate::verbs::reservations::Err2) -> String {
    match e {
        crate::verbs::reservations::Err2::Msg(m) => m,
        crate::verbs::reservations::Err2::Ex => {
            "waiting_on clear: an exotic (unmodelable) record value".to_string()
        }
    }
}

// ─── payload parsing (adapter-normalization twin, read-only) ────────────────

fn parse_payload(raw: &str) -> Map<String, Value> {
    if raw.trim().is_empty() {
        return Map::new();
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(m)) => m,
        _ => Map::new(),
    }
}

// ─── JS value helpers ───────────────────────────────────────────────────────

fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// `value ?? null` — missing key and JSON null both land on Null.
fn nullish_or_null(v: Option<&Value>) -> Value {
    match v {
        None | Some(Value::Null) => Value::Null,
        Some(other) => other.clone(),
    }
}

/// compaction.mjs `norm` / the hooks' session-id trim: string && trim || null.
fn norm(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    }
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn iso_from_ms(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
        .unwrap_or_else(now_iso)
}

/// JS Date.parse for the ISO shapes bee writes (toISOString output plus the
/// date-only form). Anything else is NaN → None.
fn date_parse_ms(s: &str) -> Option<i64> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp_millis());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return d.and_hms_opt(0, 0, 0).map(|ndt| ndt.and_utc().timestamp_millis());
    }
    None
}

fn sha256_hex(input: &str) -> String {
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

// SHA-1 (inject.mjs stableHash) — implemented in-file; the workspace only
// ships sha2 and Cargo.toml is out of this cell's write scope.
fn sha1_hex(input: &str) -> String {
    let data = input.as_bytes();
    let ml = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&ml.to_be_bytes());
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let tmp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = tmp;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

// ─── preflight-flavored JSON read ───────────────────────────────────────────

// plan() must still be able to return Delegate having emitted NOTHING (a
// shape-wrong lane and a non-object inject cache both delegate AFTER other
// reads have run), so a corrupt-JSON warning discovered during plan() is
// QUEUED and released by execute(), the moment the run is committed to native.
enum PlanWarn {
    /// crate::fsutil::warn_corrupt_json for this file.
    CorruptJson(PathBuf),
    /// A ready-made line (readLane's own deterministic warn).
    Line(String),
}

thread_local! {
    static PLAN_WARNINGS: std::cell::RefCell<Vec<PlanWarn>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn queue_plan_warning(w: PlanWarn) {
    PLAN_WARNINGS.with(|q| q.borrow_mut().push(w));
}

fn flush_plan_warnings() {
    let queued = PLAN_WARNINGS.with(|q| q.borrow_mut().drain(..).collect::<Vec<_>>());
    for w in queued {
        match w {
            PlanWarn::CorruptJson(file) => crate::fsutil::warn_corrupt_json(&file),
            PlanWarn::Line(line) => eprint!("{line}"),
        }
    }
}

fn clear_plan_warnings() {
    PLAN_WARNINGS.with(|q| q.borrow_mut().clear());
}

/// fsutil.mjs readJson: Missing → None; Corrupt → queue one warning and take
/// the `null` fallback, which every caller here reads as "absent"; Parsed →
/// value. (Still `Pf` because the callers that delegate for OTHER reasons — a
/// non-object inject cache, a shape-wrong lane — thread their Err through it.)
fn read_json_pf(file: &Path) -> Pf<Option<Value>> {
    match read_json(file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => {
            queue_plan_warning(PlanWarn::CorruptJson(file.to_path_buf()));
            Ok(None)
        }
        ReadJson::Parsed(v) => Ok(Some(v)),
    }
}

/// Node path.relative for the shape readLane's warn needs (file under root).
fn path_relative(root: &Path, file: &Path) -> String {
    match file.strip_prefix(root) {
        Ok(rel) => rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(std::path::MAIN_SEPARATOR_STR),
        Err(_) => file.display().to_string(),
    }
}

/// state.mjs readLane's own (deterministic, never V8-worded) skip warn, which
/// Node prints ON TOP of readJson's when a lane record will not parse.
fn queue_corrupt_lane_warning(root: &Path, file: &Path) {
    let rel = path_relative(root, file);
    queue_plan_warning(PlanWarn::Line(format!(
        "readLane: skipping corrupt lane record \"{rel}\" for display \u{2014} mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\").\n"
    )));
}

// ─── state.mjs: readState / lanes / resolvePipeline ─────────────────────────

/// The slice of a pipeline record (state.json or a lane record) the
/// reminder and the nudge consume.
#[derive(Clone)]
struct StateRecord {
    phase: Value,
    feature: Value,
    mode: Value,
    next_action: Value,
    /// approved_gates after `{ ...defaults, ...(overlay || {}) }` for the
    /// four gate names this hook queries. A truthy non-map overlay spreads
    /// only index/char keys in JS, so the named gates read as the defaults.
    gates: Map<String, Value>,
}

fn default_gates() -> Map<String, Value> {
    let mut m = Map::new();
    for g in GATE_NAMES {
        m.insert(g.to_string(), Value::Bool(false));
    }
    m
}

fn merged_gates(overlay: Option<&Value>) -> Map<String, Value> {
    let mut gates = default_gates();
    if let Some(Value::Object(over)) = overlay {
        for (k, v) in over {
            gates.insert(k.clone(), v.clone());
        }
    }
    gates
}

/// state.mjs coerceLegacyPhase (D13): 'validating' → 'planning'.
fn coerce_legacy_phase(phase: Value) -> Value {
    if phase == Value::String("validating".into()) {
        Value::String("planning".into())
    } else {
        phase
    }
}

/// state.mjs readState — fail-open display read over .bee/state.json.
fn read_state(root: &Path) -> Pf<StateRecord> {
    let parsed = read_json_pf(&root.join(".bee").join("state.json"))?;
    let state = match parsed {
        Some(Value::Object(m)) => m,
        _ => {
            // defaultState()
            return Ok(StateRecord {
                phase: Value::String("idle".into()),
                feature: Value::Null,
                mode: Value::Null,
                next_action: Value::String("No active bee work — awaiting a user request.".into()),
                gates: default_gates(),
            });
        }
    };
    let pick = |key: &str, default: Value| state.get(key).cloned().unwrap_or(default);
    Ok(StateRecord {
        phase: coerce_legacy_phase(pick("phase", Value::String("idle".into()))),
        feature: pick("feature", Value::Null),
        mode: pick("mode", Value::Null),
        next_action: pick(
            "next_action",
            Value::String("No active bee work — awaiting a user request.".into()),
        ),
        gates: merged_gates(state.get("approved_gates").filter(|v| js_truthy(v))),
    })
}

/// claims.mjs requireId's validity predicate (plain id, no separators/..).
fn is_plain_id(id: &str) -> bool {
    let id = id.trim();
    !id.is_empty() && !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

/// claims.mjs readSession — fail-open. Corrupt record file → delegate.
fn read_session(control_root: &Path, session_id: &str) -> Pf<Option<Map<String, Value>>> {
    let sid = session_id.trim();
    if !is_plain_id(sid) {
        return Ok(None); // sessionPath throws; readSession catches → null
    }
    let file = control_root.join(".bee").join("sessions").join(format!("{sid}.json"));
    let Some(parsed) = read_json_pf(&file)? else { return Ok(None) };
    match parsed {
        Value::Object(m) if m.get("id").and_then(Value::as_str) == Some(sid) => Ok(Some(m)),
        _ => Ok(None),
    }
}

/// state.mjs laneRecordFrom + readLane. Corrupt JSON is native: readJson's
/// warn, then laneRecordFrom(null) → readLane's own warn, then null → the
/// LANE_CORRUPT refusal, which this hook already handles exactly like
/// LANE_MISSING (reminder falls back to readState, nudge silent). A record
/// that PARSES but has the wrong shape still delegates — that arm is about
/// JS-shape exotica, not V8 text.
fn read_lane_record(control_root: &Path, feature: &str) -> Pf<Option<StateRecord>> {
    let file = control_root.join(".bee").join("lanes").join(format!("{feature}.json"));
    if !file.exists() {
        return Ok(None); // LANE_MISSING — typed refusal, no warning
    }
    let parsed = match read_json(&file) {
        ReadJson::Missing => return Err(NeedsNode), // vanished mid-read: unreachable, be safe
        ReadJson::Corrupt => {
            queue_plan_warning(PlanWarn::CorruptJson(file.clone()));
            queue_corrupt_lane_warning(control_root, &file);
            return Ok(None); // LANE_CORRUPT → Pipeline::Refused
        }
        ReadJson::Parsed(v) => v,
    };
    let Value::Object(lane) = parsed else { return Err(NeedsNode) }; // readLane warns
    if lane.get("feature").and_then(Value::as_str) != Some(feature) {
        return Err(NeedsNode); // feature mismatch → readLane warns
    }
    let pick = |key: &str, default: Value| lane.get(key).cloned().unwrap_or(default);
    Ok(Some(StateRecord {
        phase: coerce_legacy_phase(pick("phase", Value::String("idle".into()))),
        feature: pick("feature", Value::String(feature.to_string())),
        mode: pick("mode", Value::Null),
        next_action: pick("next_action", Value::String(String::new())),
        gates: merged_gates(lane.get("approved_gates").filter(|v| js_truthy(v))),
    }))
}

enum Pipeline {
    Ok(StateRecord),
    /// LANE_INVALID / LANE_MISSING / LANE_CORRUPT-adjacent typed refusal:
    /// the reminder falls back to readState, the nudge goes silent.
    Refused,
}

/// state.mjs resolvePipeline — session record → bound lane → default state.
fn resolve_pipeline(root: &Path, control_root: &Path, session_id: Option<&str>) -> Pf<Pipeline> {
    let Some(sid) = session_id else {
        return Ok(Pipeline::Ok(read_state(root)?));
    };
    let Some(session) = read_session(control_root, sid)? else {
        return Ok(Pipeline::Ok(read_state(root)?));
    };
    let bound = match session.get("lane") {
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
        _ => return Ok(Pipeline::Ok(read_state(root)?)),
    };
    if !is_plain_id(&bound) {
        return Ok(Pipeline::Refused); // LANE_INVALID
    }
    match read_lane_record(control_root, &bound)? {
        Some(record) => Ok(Pipeline::Ok(record)),
        None => Ok(Pipeline::Refused), // LANE_MISSING
    }
}

/// state.mjs resolveRootsCore/resolveContext/controlRootFor — the ordinary
/// arm only; a linked-worktree `.git` FILE marker delegates (grants +
/// possibly-throwing validation live there).
fn control_root_for_state(root: &Path) -> Pf<PathBuf> {
    // Nearest-ancestor onboarding marker (fixture rule): onboarding.json
    // present AND no .git at that level wins outright.
    let mut dir = root.to_path_buf();
    loop {
        if dir.join(".bee").join("onboarding.json").exists() && !dir.join(".git").exists() {
            return Ok(dir);
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break,
        }
    }
    // locateGitRoot walk-up.
    let mut dir = root.to_path_buf();
    let located = loop {
        if dir.join(".git").exists() {
            break Some(dir.clone());
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break None,
        }
    };
    let Some(work_root) = located else {
        // No git root: fall back to any onboarding marker; else controlRoot
        // is null and controlRootFor returns `root` itself.
        let mut dir = root.to_path_buf();
        loop {
            if dir.join(".bee").join("onboarding.json").exists() {
                return Ok(dir);
            }
            match dir.parent() {
                Some(p) => dir = p.to_path_buf(),
                None => return Ok(root.to_path_buf()),
            }
        }
    };
    let marker = work_root.join(".git");
    match std::fs::metadata(&marker) {
        Ok(meta) if meta.is_file() => Err(NeedsNode), // linked worktree arm
        Ok(_) => Ok(work_root),
        Err(_) => Err(NeedsNode), // statSync throw — unreachable-ish, be safe
    }
}

/// state.mjs resolveProductRoot's warning predicate: a configured
/// product_root that is non-string, or names a missing directory, makes
/// every resolveContext call warn to stderr → delegate.
fn product_root_would_warn(base: &Path, config: &Map<String, Value>) -> bool {
    match config.get("product_root") {
        None | Some(Value::Null) => false,
        Some(Value::String(s)) if s.is_empty() => false,
        Some(Value::String(s)) => {
            let p = PathBuf::from(s);
            let resolved = if p.is_absolute() { p } else { base.join(p) };
            !resolved.is_dir()
        }
        Some(_) => true,
    }
}

/// reservations.mjs findMainRoot ?? root — fail-open, never throws; the
/// ordinary arm returns the nearest git root. A linked-worktree marker file
/// is unreachable here (the hook already delegated on that topology).
fn control_root_for_leases(root: &Path) -> PathBuf {
    let mut dir = root.to_path_buf();
    loop {
        let marker = dir.join(".git");
        if marker.exists() {
            return match std::fs::metadata(&marker) {
                Ok(meta) if !meta.is_file() => dir,
                _ => root.to_path_buf(), // file marker/unreadable → fail open to root
            };
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => return root.to_path_buf(),
        }
    }
}

// ─── inject.mjs: reminder + inject cache ────────────────────────────────────

/// inject.mjs visibleGates (codex-loop advisor #54).
fn visible_gates(record: &StateRecord) -> Vec<&'static str> {
    if matches!(&record.phase, Value::String(s) if NO_WORK_PHASES.contains(&s.as_str())) {
        return Vec::new();
    }
    if record.phase == Value::String("reviewing".into()) {
        GATE_NAMES.to_vec()
    } else {
        GATE_NAMES.iter().copied().filter(|g| *g != "review").collect()
    }
}

/// inject.mjs firstOpenGate.
fn first_open_gate(record: &StateRecord) -> Option<&'static str> {
    visible_gates(record)
        .into_iter()
        .find(|g| record.gates.get(*g) != Some(&Value::Bool(true)))
}

/// inject.mjs buildPromptReminder → (text, stableHash).
fn build_prompt_reminder(record: &StateRecord) -> (String, String) {
    let mode = nullish_or_null(Some(&record.mode));
    let next_action = nullish_or_null(Some(&record.next_action));
    let gate = first_open_gate(record);

    let mut fields = Map::new();
    fields.insert("phase".into(), record.phase.clone());
    fields.insert("mode".into(), mode.clone());
    fields.insert("next_action".into(), next_action.clone());
    fields.insert(
        "first_open_gate".into(),
        gate.map_or(Value::Null, |g| Value::String(g.to_string())),
    );
    let hash = sha1_hex(&jsjson::stringify(&Value::Object(fields)));

    let mut lines = vec![format!(
        "bee: phase={}{}",
        jsjson::js_to_string(&record.phase),
        if js_truthy(&mode) {
            format!(" mode={}", jsjson::js_to_string(&mode))
        } else {
            String::new()
        }
    )];
    if js_truthy(&next_action) {
        lines.push(format!("next: {}", jsjson::js_to_string(&next_action)));
    }
    if let Some(g) = gate {
        lines.push(format!("gate pending: {g}"));
    }
    lines.truncate(3);
    (lines.join("\n"), hash)
}

fn inject_cache_path(root: &Path) -> PathBuf {
    root.join(".bee").join("cache").join("inject-cache.json")
}

fn legacy_inject_cache_path(root: &Path) -> PathBuf {
    root.join(".bee").join(".inject-cache.json")
}

/// inject.mjs's cache read chain:
/// `readJson(new, null) || readJson(legacy, {}) || {}` — a corrupt cache warns
/// and reads as absent, so the chain falls through to the legacy location and
/// then to `{}` (every key re-injects once, which is the intended fail-open).
/// A non-object cache value still delegates: strict-mode property assignment
/// on it would THROW in markInjected, which is JS exotica, not V8 text.
fn read_inject_cache(root: &Path) -> Pf<Map<String, Value>> {
    let primary = read_json_pf(&inject_cache_path(root))?;
    let value = match primary {
        Some(v) if js_truthy(&v) => v,
        _ => match read_json_pf(&legacy_inject_cache_path(root))? {
            Some(v) if js_truthy(&v) => v,
            _ => Value::Object(Map::new()),
        },
    };
    match value {
        Value::Object(m) => Ok(m),
        _ => Err(NeedsNode),
    }
}

/// inject.mjs shouldInject: hash changed, or >30 min since last injection.
fn should_inject(cache: &Map<String, Value>, key: &str, hash: &str, now: i64) -> bool {
    let Some(entry) = cache.get(key).filter(|v| js_truthy(v)) else {
        return true;
    };
    if entry.get("hash").and_then(Value::as_str) != Some(hash) {
        return true;
    }
    let Some(last) = entry.get("at").and_then(Value::as_str).and_then(date_parse_ms) else {
        return true;
    };
    now - last > INJECT_INTERVAL_MS
}

/// inject.mjs markInjected — re-reads the chain, sets the key, writes the
/// new location atomically, prunes the legacy file. Any write error maps to
/// the .mjs's throw (caller crash-logs).
fn mark_injected(root: &Path, key: &str, hash: &str) -> Result<(), String> {
    // Re-read the same chain shouldInject uses — the .mjs reads afresh in both
    // functions, so this readJson pair warns for itself just as Node's did.
    let read_or_warn = |file: PathBuf| -> Option<Value> {
        let read = read_json(&file);
        if matches!(read, ReadJson::Corrupt) {
            crate::fsutil::warn_corrupt_json(&file);
        }
        match read {
            ReadJson::Parsed(v) if js_truthy(&v) => Some(v),
            _ => None,
        }
    };
    let primary = read_or_warn(inject_cache_path(root));
    let value = primary
        .or_else(|| read_or_warn(legacy_inject_cache_path(root)))
        .unwrap_or_else(|| Value::Object(Map::new()));
    let mut cache = match value {
        Value::Object(m) => m,
        _ => return Err("TypeError: cannot set property on non-object inject cache".to_string()),
    };
    let mut entry = Map::new();
    entry.insert("hash".into(), Value::String(hash.to_string()));
    entry.insert("at".into(), Value::String(now_iso()));
    cache.insert(key.to_string(), Value::Object(entry));
    write_json_atomic(&inject_cache_path(root), &Value::Object(cache))
        .map_err(|e| format!("Error: {e}"))?;
    let _ = std::fs::remove_file(legacy_inject_cache_path(root));
    Ok(())
}

// ─── compaction.mjs: anchorMissing + cell/claim attribution ─────────────────

/// cells.mjs listCells (top-level arm, includeArchived:false): every parsed
/// non-null object/array under .bee/cells/*.json. Corrupt → delegate.
fn list_cells_top(root: &Path) -> Pf<Vec<Value>> {
    let dir = root.join(".bee").join("cells");
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(Vec::new()) };
    let mut cells = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        if let Some(cell) = read_json_pf(&entry.path())? {
            if js_truthy(&cell) && (cell.is_object() || cell.is_array()) {
                cells.push(cell);
            }
        }
    }
    // listCells sorts by String(id).localeCompare(.., 'en', {numeric:true});
    // the order is unobservable downstream (see module header) — a natural
    // case-insensitive sort keeps map insertion deterministic.
    cells.sort_by(|a, b| {
        natural_cmp(
            &jsjson::js_to_string(a.get("id").unwrap_or(&Value::Null)),
            &jsjson::js_to_string(b.get("id").unwrap_or(&Value::Null)),
        )
    });
    Ok(cells)
}

/// Approximation of localeCompare('en', {numeric: true}) for cell ids.
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let (la, lb) = (a.to_lowercase(), b.to_lowercase());
    let mut ia = la.chars().peekable();
    let mut ib = lb.chars().peekable();
    loop {
        match (ia.peek().copied(), ib.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(ca), Some(cb)) => {
                if ca.is_ascii_digit() && cb.is_ascii_digit() {
                    let mut na = String::new();
                    while let Some(c) = ia.peek().copied() {
                        if c.is_ascii_digit() {
                            na.push(c);
                            ia.next();
                        } else {
                            break;
                        }
                    }
                    let mut nb = String::new();
                    while let Some(c) = ib.peek().copied() {
                        if c.is_ascii_digit() {
                            nb.push(c);
                            ib.next();
                        } else {
                            break;
                        }
                    }
                    let va: u128 = na.parse().unwrap_or(u128::MAX);
                    let vb: u128 = nb.parse().unwrap_or(u128::MAX);
                    match va.cmp(&vb) {
                        std::cmp::Ordering::Equal => {}
                        other => return other,
                    }
                } else {
                    match ca.cmp(&cb) {
                        std::cmp::Ordering::Equal => {
                            ia.next();
                            ib.next();
                        }
                        other => return other,
                    }
                }
            }
        }
    }
}

/// claims.mjs readClaim — a corrupt claim warns and reads as "no claim". An
/// invalid id (requireId would throw) reads as None here; callers that need
/// the throw handle it themselves.
fn read_claim(control_root: &Path, cell_id: &str) -> Pf<Option<Value>> {
    if !is_plain_id(cell_id) {
        return Ok(None);
    }
    let file = control_root.join(".bee").join("claims").join(format!("{cell_id}.json"));
    match read_json_pf(&file)? {
        Some(v) if js_truthy(&v) && (v.is_object() || v.is_array()) => Ok(Some(v)),
        _ => Ok(None),
    }
}

/// cells.mjs readCell — active file first, then the archive fallback.
/// ID_PATTERN: /^[A-Za-z0-9][A-Za-z0-9._-]*$/.
fn read_cell(root: &Path, id: &str) -> Pf<Option<Value>> {
    let mut chars = id.chars();
    let head_ok = chars.next().map(|c| c.is_ascii_alphanumeric()).unwrap_or(false);
    let rest_ok = id.chars().skip(1).all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
    if !head_ok || !rest_ok {
        return Ok(None);
    }
    let cells_dir = root.join(".bee").join("cells");
    if let Some(v) = read_json_pf(&cells_dir.join(format!("{id}.json")))? {
        if !v.is_null() {
            return Ok(Some(v));
        }
    }
    let archive = cells_dir.join("archive");
    let Ok(entries) = std::fs::read_dir(&archive) else { return Ok(None) };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        if let Some(v) = read_json_pf(&entry.path().join(format!("{id}.json")))? {
            if !v.is_null() {
                return Ok(Some(v));
            }
        }
    }
    Ok(None)
}

fn claim_session_of(claim: &Value) -> Option<String> {
    norm(claim.get("session"))
}

/// compaction.mjs attributedToSession.
fn attributed_to_session(
    root: &Path,
    control_root: &Path,
    cell: &Value,
    session: Option<&str>,
) -> Pf<bool> {
    let _ = root;
    let trace_session = norm(cell.get("trace").and_then(|t| t.get("claim_session")));
    let cell_id = cell.get("id").and_then(Value::as_str).unwrap_or("");
    if let Some(s) = session {
        if trace_session.as_deref() == Some(s) {
            return Ok(true);
        }
        let claim = read_claim(control_root, cell_id)?;
        return Ok(claim
            .as_ref()
            .and_then(claim_session_of)
            .as_deref()
            == Some(s));
    }
    if trace_session.is_some() {
        return Ok(false);
    }
    if cell.get("status").and_then(Value::as_str) != Some("claimed") {
        return Ok(false);
    }
    let claim = read_claim(control_root, cell_id)?;
    Ok(match claim {
        None => true,
        Some(c) => claim_session_of(&c).is_none(),
    })
}

/// compaction.mjs claimStoreCellIds.
fn claim_store_cell_ids(control_root: &Path, session: &str) -> Pf<Vec<String>> {
    let dir = control_root.join(".bee").join("claims");
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(Vec::new()) };
    let mut ids = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(id) = name.strip_suffix(".json") else { continue };
        if let Some(claim) = read_claim(control_root, id)? {
            if claim_session_of(&claim).as_deref() == Some(session) {
                ids.push(id.to_string());
            }
        }
    }
    Ok(ids)
}

/// compaction.mjs sessionOwnedCells + claimedCells (status == 'claimed').
fn claimed_cells(root: &Path, control_root: &Path, session: Option<&str>) -> Pf<Vec<Value>> {
    let mut owned: Vec<(String, Value)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for cell in list_cells_top(root)? {
        let Some(id) = cell.get("id").and_then(Value::as_str).map(str::to_string) else {
            continue;
        };
        if attributed_to_session(root, control_root, &cell, session)? {
            if seen.insert(id.clone()) {
                owned.push((id, cell));
            }
        }
    }
    if let Some(s) = session {
        for id in claim_store_cell_ids(control_root, s)? {
            if seen.contains(&id) {
                continue;
            }
            let cell = read_cell(root, &id)?.unwrap_or_else(|| {
                let mut m = Map::new();
                m.insert("id".into(), Value::String(id.clone()));
                m.insert("status".into(), Value::Null);
                m.insert("missing".into(), Value::Bool(true));
                Value::Object(m)
            });
            seen.insert(id.clone());
            owned.push((id, cell));
        }
    }
    Ok(owned
        .into_iter()
        .map(|(_, c)| c)
        .filter(|c| js_truthy(c) && c.get("status").and_then(Value::as_str) == Some("claimed"))
        .collect())
}

/// compaction.mjs claimedCellId — most recently claimed first, id tiebreak.
fn claimed_cell_id(cells: &[Value]) -> Option<String> {
    if cells.is_empty() {
        return None;
    }
    // String(x ?? '') — undefined/null render as '', everything else via
    // JS String() coercion.
    fn claimed_at_str(cell: &Value) -> String {
        match cell.get("trace").and_then(|t| t.get("claimed_at")) {
            None | Some(Value::Null) => String::new(),
            Some(v) => jsjson::js_to_string(v),
        }
    }
    let mut sorted: Vec<&Value> = cells.iter().collect();
    sorted.sort_by(|a, b| {
        let left = claimed_at_str(b);
        let right = claimed_at_str(a);
        if left != right {
            left.cmp(&right)
        } else {
            let ida = jsjson::js_to_string(a.get("id").unwrap_or(&Value::Null));
            let idb = jsjson::js_to_string(b.get("id").unwrap_or(&Value::Null));
            // localeCompare tiebreak — approximated case-insensitively.
            let (la, lb) = (ida.to_lowercase(), idb.to_lowercase());
            if la != lb { la.cmp(&lb) } else { ida.cmp(&idb) }
        }
    });
    sorted
        .first()
        .and_then(|c| c.get("id"))
        .map(|v| jsjson::js_to_string(v))
}

// ─── intent.mjs: anchor presence ────────────────────────────────────────────

/// intent.mjs sanitizeIntentKey.
fn sanitize_intent_key(key: &str) -> String {
    let raw = key.trim();
    if raw.is_empty() {
        return "default".to_string();
    }
    // /[^A-Za-z0-9._-]+/g → '-'
    let mut safe = String::new();
    let mut in_bad = false;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || "._-".contains(c) {
            safe.push(c);
            in_bad = false;
        } else if !in_bad {
            safe.push('-');
            in_bad = true;
        }
    }
    // ^[-.]+ stripped
    let safe = safe.trim_start_matches(['-', '.']).to_string();
    // -+$ stripped
    let safe = safe.trim_end_matches('-').to_string();
    let safe: String = safe.chars().take(120).collect();
    if safe.is_empty() { "default".to_string() } else { safe }
}

/// intent.mjs readIntent — presence only (normalizeAnchor's one hard
/// requirement: a non-array object whose `request` is a non-blank string).
fn intent_anchor_present(
    root: &Path,
    state: &StateRecord,
    session_id: Option<&str>,
) -> Pf<bool> {
    // intentKeyCandidates: active feature (phase-gated) → session → default.
    let mut candidates: Vec<String> = Vec::new();
    let no_work = matches!(&state.phase, Value::String(s) if NO_WORK_PHASES.contains(&s.as_str()));
    let feature = if no_work { None } else { norm(Some(&state.feature)) };
    if let Some(f) = feature {
        candidates.push(sanitize_intent_key(&f));
    }
    if let Some(sid) = session_id {
        if !sid.trim().is_empty() {
            candidates.push(sanitize_intent_key(sid));
        }
    }
    candidates.push("default".to_string());
    let mut uniq: Vec<String> = Vec::new();
    for c in candidates {
        if !uniq.contains(&c) {
            uniq.push(c);
        }
    }
    for key in uniq {
        let file = root.join(".bee").join("intent").join(format!("{key}.json"));
        if let Some(Value::Object(raw)) = read_json_pf(&file)? {
            if matches!(raw.get("request"), Some(Value::String(s)) if !s.trim().is_empty()) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// compaction.mjs anchorMissing → Some((message, dedup hash)) | None.
fn anchor_missing(
    root: &Path,
    control_root: &Path,
    pipeline: &Pipeline,
    session_id: Option<&str>,
) -> Pf<Option<(String, String)>> {
    let session = session_id.map(str::trim).filter(|s| !s.is_empty());
    let Pipeline::Ok(record) = pipeline else { return Ok(None) };
    let phase = nullish_or_null(Some(&record.phase));
    if !js_truthy(&phase) {
        return Ok(None);
    }
    if matches!(&phase, Value::String(s) if NO_WORK_PHASES.contains(&s.as_str())) {
        return Ok(None);
    }
    let cells = claimed_cells(root, control_root, session)?;
    let execution_approved = record.gates.get("execution") == Some(&Value::Bool(true));
    if cells.is_empty() && !execution_approved {
        return Ok(None);
    }
    // readIntent uses readState(root) for the active feature, NOT the lane
    // record (intent.mjs activeFeature).
    let state = read_state(root)?;
    if intent_anchor_present(root, &state, session)? {
        return Ok(None);
    }
    let cell = if cells.is_empty() { None } else { claimed_cell_id(&cells) };
    let feature = nullish_or_null(Some(&record.feature));
    let feature_truthy = js_truthy(&feature);
    let hash = format!(
        "{}:{}:{}",
        session.unwrap_or(""),
        if feature.is_null() { String::new() } else { jsjson::js_to_string(&feature) },
        cell.clone().unwrap_or_default(),
    );
    let mut message = String::from("bee: work is active and NO INTENT ANCHOR is stored");
    if feature_truthy {
        message.push_str(&format!(" (feature={}", jsjson::js_to_string(&feature)));
        if let Some(c) = &cell {
            if !c.is_empty() {
                message.push_str(&format!(" cell={c}"));
            }
        }
        message.push(')');
    }
    message.push_str(". ");
    message.push_str(
        "The objective currently lives only in this conversation — the least durable place in the session, and the \
first thing a compaction compresses, while every workflow detail on disk comes back at full strength. \
Write it down VERBATIM now: ",
    );
    message.push_str(ANCHOR_NUDGE_COMMAND);
    Ok(Some((message, hash)))
}

// ─── lock.mjs: store locks (once-sync + maxAttempts:1 flavors) ──────────────

/// lock.mjs sanitizeLockName: unsafe chars → '_', plus sha256(name)[0..8].
fn sanitize_lock_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || (c as u32) < 0x20 {
                '_'
            } else {
                c
            }
        })
        .collect();
    format!("{sanitized}-{}", &sha256_hex(name)[..8])
}

fn lock_file_path(root: &Path, name: &str) -> PathBuf {
    root.join(".bee").join("locks").join(format!("{}.lock", sanitize_lock_name(name)))
}

fn env_session_id() -> Option<String> {
    for var in ["BEE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"] {
        if let Ok(v) = std::env::var(var) {
            if !v.trim().is_empty() {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

fn read_holder(lock_path: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(lock_path).ok()?;
    serde_json::from_str::<Value>(&text).ok()
}

/// lock.mjs isPidAlive — null-signal probe.
fn is_pid_alive(pid: &Value) -> bool {
    let n = match pid {
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => s.trim().parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    };
    if !n.is_finite() || n.fract() != 0.0 || n <= 0.0 {
        return false;
    }
    let pid = n as u32;
    pid_alive_os(pid)
}

#[cfg(windows)]
fn pid_alive_os(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ACCESS_DENIED};
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !handle.is_null() {
            CloseHandle(handle);
            true
        } else {
            GetLastError() == ERROR_ACCESS_DENIED
        }
    }
}

#[cfg(unix)]
fn pid_alive_os(pid: u32) -> bool {
    let ok = unsafe { libc::kill(pid as i32, 0) } == 0;
    ok || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

fn random_token(bytes: usize) -> String {
    // No rand crate in-tree: hash pid + monotonic + wall clock + a counter.
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seed = format!(
        "{}-{}-{}-{:?}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        nanos,
        std::time::Instant::now(),
    );
    sha256_hex(&seed)[..bytes * 2].to_string()
}

fn try_acquire(lock_path: &Path, body: &Value) -> Result<bool, String> {
    let content = format!("{}\n", jsjson::stringify(body));
    match std::fs::OpenOptions::new().write(true).create_new(true).open(lock_path) {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(content.as_bytes()).map_err(|e| format!("Error: {e}"))?;
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(format!("Error: {e}")),
    }
}

fn same_holder_identity(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) if js_truthy(a) && js_truthy(b) => {
            a.get("pid") == b.get("pid") && a.get("token") == b.get("token") && a.get("ts") == b.get("ts")
        }
        _ => false,
    }
}

/// lock.mjs tryStaleTakeover (sync flavor): mtime + pid-liveness rule, then
/// the rename-verify-settle dance.
fn try_stale_takeover(lock_path: &Path, now_ms: i64) -> bool {
    let Ok(meta) = std::fs::metadata(lock_path) else { return false };
    let mtime_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as f64)
        .unwrap_or(f64::MAX);
    let age = now_ms as f64 - mtime_ms;
    if age <= LOCK_STALE_MS {
        return false;
    }
    let holder_before = read_holder(lock_path);
    if age <= LOCK_HARD_STALE_MS {
        let pid = holder_before
            .as_ref()
            .filter(|h| h.is_object())
            .and_then(|h| h.get("pid"))
            .cloned()
            .unwrap_or(Value::Null);
        if is_pid_alive(&pid) {
            return false;
        }
    }
    // performTakeoverClaim
    if !same_holder_identity(read_holder(lock_path).as_ref(), holder_before.as_ref()) {
        return false;
    }
    let stale_path = PathBuf::from(format!(
        "{}.stale-{}-{}-{}",
        lock_path.display(),
        std::process::id(),
        now_ms,
        random_token(4),
    ));
    if std::fs::rename(lock_path, &stale_path).is_err() {
        return false; // ENOENT: another racer consumed it — normal loss
    }
    let holder_after = read_holder(&stale_path);
    // settleTakeover
    if same_holder_identity(holder_after.as_ref(), holder_before.as_ref()) {
        let _ = std::fs::remove_file(&stale_path);
        return true;
    }
    if std::fs::metadata(lock_path).is_err() {
        if std::fs::rename(&stale_path, lock_path).is_ok() {
            return false;
        }
    }
    let _ = std::fs::remove_file(&stale_path);
    false
}

fn append_contention_telemetry(root: &Path, record: &Value) {
    let logs = root.join(".bee").join("logs");
    if std::fs::create_dir_all(&logs).is_err() {
        return;
    }
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs.join("contention.jsonl"))
    {
        let _ = f.write_all(format!("{}\n", jsjson::stringify(record)).as_bytes());
    }
}

fn contention_record(
    name: &str,
    session: &Option<String>,
    wait_start_ms: i64,
    holder_session: Option<Value>,
    result: &str,
) -> Value {
    let mut m = Map::new();
    m.insert("ts".into(), Value::String(now_iso()));
    m.insert("lock_name".into(), Value::String(name.to_string()));
    m.insert(
        "lock_wait_ms".into(),
        Value::Number(serde_json::Number::from((now_ms() - wait_start_ms).max(0))),
    );
    m.insert("holder_session".into(), holder_session.unwrap_or(Value::Null));
    m.insert(
        "caller_session".into(),
        session.clone().map_or(Value::Null, Value::String),
    );
    m.insert("workflow_id".into(), Value::Null);
    m.insert("workspace_id".into(), Value::Null);
    m.insert("resource".into(), Value::Null);
    m.insert("result".into(), Value::String(result.to_string()));
    Value::Object(m)
}

struct LockGuard {
    path: PathBuf,
    token: String,
    released: bool,
}

impl LockGuard {
    /// lock.mjs release(): only removes a lock this acquisition created.
    fn release(&mut self) {
        if self.released {
            return;
        }
        self.released = true;
        if let Some(holder) = read_holder(&self.path) {
            let pid_match = holder.get("pid").and_then(Value::as_u64)
                == Some(std::process::id() as u64);
            if holder.get("token").and_then(Value::as_str) == Some(self.token.as_str()) && pid_match
            {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}

/// lock.mjs acquireStoreLockOnceSync / withStoreLock({maxAttempts:1}) —
/// both perform exactly one acquire attempt plus one post-takeover retry;
/// they differ only in how the caller surfaces failure.
fn acquire_store_lock_once(root: &Path, name: &str) -> Result<Option<LockGuard>, String> {
    std::fs::create_dir_all(root.join(".bee").join("locks")).map_err(|e| format!("Error: {e}"))?;
    let lock_path = lock_file_path(root, name);
    let token = random_token(8);
    let session = env_session_id();
    let now = now_ms();
    let wait_start = now;
    let mut body = Map::new();
    body.insert(
        "pid".into(),
        Value::Number(serde_json::Number::from(std::process::id())),
    );
    body.insert("session".into(), session.clone().map_or(Value::Null, Value::String));
    body.insert("ts".into(), Value::String(iso_from_ms(now)));
    body.insert("token".into(), Value::String(token.clone()));

    let mut acquired = try_acquire(&lock_path, &Value::Object(body.clone()))?;
    let mut contended_holder_session: Option<Value> = None;
    if !acquired {
        if let Some(contending) = read_holder(&lock_path) {
            if contending.is_object() {
                if let Some(s) = contending.get("session").filter(|v| js_truthy(v)) {
                    contended_holder_session = Some(s.clone());
                }
            }
        }
        if try_stale_takeover(&lock_path, now) {
            body.insert("ts".into(), Value::String(now_iso()));
            acquired = try_acquire(&lock_path, &Value::Object(body))?;
        }
    }
    if !acquired {
        let holder = read_holder(&lock_path);
        let holder_session = holder
            .as_ref()
            .and_then(|h| h.get("session"))
            .filter(|v| js_truthy(v))
            .cloned()
            .or(contended_holder_session);
        append_contention_telemetry(
            root,
            &contention_record(name, &session, wait_start, holder_session, "busy"),
        );
        return Ok(None);
    }
    append_contention_telemetry(
        root,
        &contention_record(name, &session, wait_start, contended_holder_session, "acquired"),
    );
    Ok(Some(LockGuard { path: lock_path, token, released: false }))
}

// ─── claims.mjs: heartbeat + claim renewal ──────────────────────────────────

fn sessions_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("sessions")
}

fn claims_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("claims")
}

/// claims.mjs heartbeatStale over the touch throttle window.
fn heartbeat_stale(session: Option<&Map<String, Value>>, now: i64, stale_seconds: i64) -> bool {
    let Some(session) = session else { return true };
    let Some(beat) = session
        .get("last_heartbeat")
        .and_then(Value::as_str)
        .and_then(date_parse_ms)
    else {
        return true;
    };
    beat + stale_seconds * 1000 <= now
}

/// Session read for the execute phase (claims.mjs heartbeatTouch's own
/// readSession — a SECOND readJson in the .mjs, so it warns for itself).
fn read_session_lenient(control_root: &Path, sid: &str) -> Option<Map<String, Value>> {
    if !is_plain_id(sid) {
        return None;
    }
    let file = sessions_dir(control_root).join(format!("{}.json", sid.trim()));
    let read = read_json(&file);
    if matches!(read, ReadJson::Corrupt) {
        crate::fsutil::warn_corrupt_json(&file);
    }
    match read {
        ReadJson::Parsed(Value::Object(m))
            if m.get("id").and_then(Value::as_str) == Some(sid.trim()) =>
        {
            Some(m)
        }
        _ => None, // corrupt / shape-wrong / id-mismatch all read as "no session"
    }
}

fn write_json_retry(file: &Path, value: &Value) -> Result<(), String> {
    // claims.mjs withTransientFsRetry: bounded retry over Windows-transient
    // errors, then the original error propagates (as a hook-level "throw").
    let mut last: Option<String> = None;
    for attempt in 0..15 {
        match write_json_atomic(file, value) {
            Ok(()) => return Ok(()),
            Err(e) => {
                let transient = matches!(
                    e.kind(),
                    std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::WouldBlock
                );
                last = Some(format!("Error: {e}"));
                if !transient || attempt + 1 >= 15 {
                    return Err(last.unwrap());
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
    }
    Err(last.unwrap_or_else(|| "Error: write failed".into()))
}

/// claims.mjs heartbeatSession (lockAttempts: 1) — typed failures
/// (LOCK_BUSY / SESSION_MISSING) are silent no-ops for the touch path; only
/// real I/O errors propagate as Err.
fn heartbeat_session(root: &Path, sid: &str, now: i64) -> Result<(), String> {
    let Some(mut guard) = acquire_store_lock_once(root, "sessions")? else {
        return Ok(()); // LOCK_BUSY — typed fail, never a throw
    };
    let result = (|| -> Result<(), String> {
        let Some(mut session) = read_session_lenient(root, sid) else {
            return Ok(()); // SESSION_MISSING — typed fail
        };
        session.insert("last_heartbeat".into(), Value::String(iso_from_ms(now)));
        // D8: a session that renews its heartbeat is no longer dead
        // (crash-swept) or closed (clean SessionEnd) — clear the mark and
        // stamp the revival (most recent only). ser-3: UserPromptSubmit is
        // the ONE heartbeat path that also revives an explicit
        // `state session release` (`released: true`) — the user speaking
        // again is the re-engage signal, unlike the PostToolUse heartbeat
        // (state_sync.rs), which deliberately leaves a released mark alone.
        let status = session.get("status").and_then(Value::as_str);
        if status == Some("dead") || status == Some("closed") {
            session.remove("status");
            session.remove("dead_at");
            session.remove("closed_at");
            session.remove("released");
            session.insert("revived_at".into(), Value::String(iso_from_ms(now)));
        }
        write_json_retry(
            &sessions_dir(root).join(format!("{}.json", sid.trim())),
            &Value::Object(session),
        )
    })();
    guard.release();
    result
}

/// claims.mjs acquireGate — single non-retrying 'wx' attempt.
fn acquire_gate(root: &Path, cell: &str, now: i64) -> Result<bool, String> {
    let gate = claims_dir(root).join(format!("{cell}.adopting"));
    let mut body = Map::new();
    body.insert(
        "pid".into(),
        Value::Number(serde_json::Number::from(std::process::id())),
    );
    body.insert("at".into(), Value::String(iso_from_ms(now)));
    match std::fs::OpenOptions::new().write(true).create_new(true).open(&gate) {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(format!("{}\n", jsjson::stringify(&Value::Object(body))).as_bytes())
                .map_err(|e| format!("Error: {e}"))?;
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(format!("Error: {e}")),
    }
}

fn release_gate(root: &Path, cell: &str) {
    let _ = std::fs::remove_file(claims_dir(root).join(format!("{cell}.adopting")));
}

/// claims.mjs readClaim as renewClaimTTL calls it (its own readJson, so it
/// warns for itself). A corrupt claim reads as "no claim" → never renewed.
fn read_claim_lenient(root: &Path, cell: &str) -> Option<Value> {
    let file = claims_dir(root).join(format!("{cell}.json"));
    let read = read_json(&file);
    if matches!(read, ReadJson::Corrupt) {
        crate::fsutil::warn_corrupt_json(&file);
    }
    match read {
        ReadJson::Parsed(v) if js_truthy(&v) && (v.is_object() || v.is_array()) => Some(v),
        _ => None,
    }
}

/// claims.mjs renewClaimTTL — same-session-only claimed_at refresh under
/// the per-claim gate. An id requireId would refuse mirrors the Node throw.
fn renew_claim_ttl(root: &Path, sid: &str, now: i64) -> Result<(), String> {
    if !is_plain_id(sid) {
        return Err("Error: session id must be a plain id (no path separators).".to_string());
    }
    let Ok(entries) = std::fs::read_dir(claims_dir(root)) else { return Ok(()) };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(cell) = name.strip_suffix(".json") else { continue };
        if !is_plain_id(cell) {
            // readClaim → claimPath → requireId throws, unwrapped here.
            return Err("Error: cell id must be a plain id (no path separators).".to_string());
        }
        let preview = read_claim_lenient(root, cell);
        if preview.as_ref().and_then(claim_session_of).as_deref() != Some(sid) {
            continue;
        }
        if !acquire_gate(root, cell, now)? {
            continue; // skipped — never waited on
        }
        let result = (|| -> Result<(), String> {
            if let Some(claim) = read_claim_lenient(root, cell) {
                if claim_session_of(&claim).as_deref() == Some(sid) {
                    if let Value::Object(mut m) = claim {
                        m.insert("claimed_at".into(), Value::String(iso_from_ms(now)));
                        write_json_retry(
                            &claims_dir(root).join(format!("{cell}.json")),
                            &Value::Object(m),
                        )?;
                    }
                }
            }
            Ok(())
        })();
        release_gate(root, cell);
        result?;
    }
    Ok(())
}

/// claims.mjs heartbeatTouch — Ok(touched). Err mirrors a JS throw.
fn heartbeat_touch(hb_root: &Path, sid: &str, now: i64) -> Result<bool, String> {
    let session = sid.trim();
    if session.is_empty() {
        return Ok(false);
    }
    let record = read_session_lenient(hb_root, session);
    if !heartbeat_stale(record.as_ref(), now, HEARTBEAT_TOUCH_THROTTLE_SECONDS) {
        return Ok(false); // throttled
    }
    heartbeat_session(hb_root, session, now)?;
    renew_claim_ttl(hb_root, session, now)?;
    Ok(true)
}

// ─── reservations.mjs / lease-store.mjs: hold renewal ───────────────────────

/// lease-store.mjs canonicalizePath.
fn canonicalize_lease_path(value: &str) -> String {
    let s = value.replace('\\', "/");
    // /^\.\/+/ — a leading "." followed by one-or-more "/" stripped once.
    let s = match s.strip_prefix("./") {
        Some(rest) => rest.trim_start_matches('/'),
        None => s.as_str(),
    };
    s.trim_end_matches('/').to_string()
}

/// lease-store.mjs computeExpiresAt.
fn compute_expires_at(ttl: f64, now: i64) -> Value {
    if ttl.is_finite() && ttl > 0.0 {
        Value::String(iso_from_ms(now + (ttl * 1000.0) as i64))
    } else {
        Value::Null
    }
}

/// reservations.mjs leaseTtlSeconds — NaN propagates (JS Math.max(0, NaN)).
fn lease_ttl_seconds(record: &Value) -> f64 {
    match record.get("expires_at") {
        None | Some(Value::Null) => 0.0,
        Some(expires) => {
            let exp = expires.as_str().and_then(date_parse_ms).map(|v| v as f64);
            let acq = record
                .get("acquired_at")
                .and_then(Value::as_str)
                .and_then(date_parse_ms)
                .map(|v| v as f64);
            match (exp, acq) {
                (Some(e), Some(a)) => (((e - a) / 1000.0).round()).max(0.0),
                _ => f64::NAN,
            }
        }
    }
}

/// reservations.mjs renewHoldsBySession over lease-store.mjs renewLease —
/// per-file {maxAttempts: 1} store locks; LockBusy/LEASE_CORRUPT propagate
/// as Err (the .mjs throws), LEASE_MISSING is skipped.
fn renew_holds_by_session(root: &Path, sid: &str, now: i64) -> Result<(), String> {
    let session = sid.trim();
    if session.is_empty() {
        return Ok(());
    }
    let ctl = control_root_for_leases(root);
    let leases_root = ctl.join(".bee").join("runtime").join("leases");
    let mut records: Vec<Value> = Vec::new();
    for sub in ["cells", "paths"] {
        let dir = leases_root.join(sub);
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            // readLeaseSafe: corrupt → silently treated as "no info".
            if let Ok(text) = std::fs::read_to_string(entry.path()) {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    records.push(v);
                }
            }
        }
    }
    for record in records {
        let is_path_lease = record
            .get("resource")
            .and_then(Value::as_str)
            .map(|r| r.starts_with("path:"))
            .unwrap_or(false);
        if !is_path_lease {
            continue;
        }
        if record.get("session_id").and_then(Value::as_str) != Some(session) {
            continue;
        }
        let ttl = lease_ttl_seconds(&record);
        // renewLease resolves the CANONICAL file from the resource id, not
        // the listed filename (lease-store.mjs resolveResourceFile).
        let id = record.get("resource").and_then(Value::as_str).unwrap()["path:".len()..].to_string();
        let canonical = canonicalize_lease_path(&id);
        let resource_key = format!("path:{canonical}");
        let file = leases_root.join("paths").join(format!("{}.json", sha256_hex(&resource_key)));
        let lock_name = format!("lease:{}", sha256_hex(&file.display().to_string()));
        let Some(mut guard) = acquire_store_lock_once(&ctl, &lock_name)? else {
            // withStoreLock exhausted its single attempt → LockBusyError.
            return Err(format!("lock \"{lock_name}\" busy"));
        };
        let result = (|| -> Result<(), String> {
            let text = match std::fs::read_to_string(&file) {
                Ok(t) => t,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()), // LEASE_MISSING → skip
                Err(e) => {
                    return Err(format!(
                        "LeaseStoreError: renewLease: could not read/parse lease record at \"{}\" ({e}).",
                        file.display()
                    ))
                }
            };
            let current = serde_json::from_str::<Value>(&text).map_err(|e| {
                format!(
                    "LeaseStoreError: renewLease: could not read/parse lease record at \"{}\" ({e}).",
                    file.display()
                )
            })?;
            let mut next = match current {
                Value::Object(m) => m,
                other => {
                    // {...primitive} yields an object; expires_at is set fresh.
                    let mut m = Map::new();
                    if let Value::Array(items) = other {
                        for (i, v) in items.into_iter().enumerate() {
                            m.insert(i.to_string(), v);
                        }
                    }
                    m
                }
            };
            next.insert("expires_at".into(), compute_expires_at(ttl, now));
            write_json_retry(&file, &Value::Object(next))
        })();
        guard.release();
        result?;
    }
    Ok(())
}

/// The hook's D5 block: throttled heartbeat + hold renewal composed at the
/// call site exactly as bee-prompt-context.mjs:84-95 does.
fn heartbeat_block(hb_root: &Path, root: &Path, sid: &str) -> Result<(), String> {
    let touched = heartbeat_touch(hb_root, sid, now_ms())?;
    if touched {
        renew_holds_by_session(root, sid, now_ms())?;
    }
    Ok(())
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bee_repo(dir: &Path) {
        std::fs::create_dir_all(dir.join(".bee")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join(".bee").join("onboarding.json"), "{}
").unwrap();
    }

    fn write(dir: &Path, rel: &str, content: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    #[test]
    fn sha1_known_vectors() {
        assert_eq!(sha1_hex("abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(sha1_hex(""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(
            sha1_hex("The quick brown fox jumps over the lazy dog"),
            "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12"
        );
    }

    #[test]
    fn reminder_matches_js_shape() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(
            tmp.path(),
            ".bee/state.json",
            r#"{"phase":"swarming","feature":"f1","mode":"standard","next_action":"do x","approved_gates":{"context":true,"shape":true,"execution":false,"review":false}}"#,
        );
        let record = read_state(tmp.path()).ok().unwrap();
        let (text, hash) = build_prompt_reminder(&record);
        assert_eq!(text, "bee: phase=swarming mode=standard\nnext: do x\ngate pending: execution");
        // stableHash = sha1(JSON.stringify({phase, mode, next_action, first_open_gate}))
        let expected = sha1_hex(
            r#"{"phase":"swarming","mode":"standard","next_action":"do x","first_open_gate":"execution"}"#,
        );
        assert_eq!(hash, expected);
    }

    #[test]
    fn reminder_terminal_phase_reports_no_gate() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        // Missing state.json → defaultState(): idle, default next_action.
        let record = read_state(tmp.path()).ok().unwrap();
        let (text, _) = build_prompt_reminder(&record);
        assert_eq!(text, "bee: phase=idle\nnext: No active bee work — awaiting a user request.");
    }

    #[test]
    fn reviewing_phase_walks_all_four_gates() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(
            tmp.path(),
            ".bee/state.json",
            r#"{"phase":"reviewing","approved_gates":{"context":true,"shape":true,"execution":true}}"#,
        );
        let record = read_state(tmp.path()).ok().unwrap();
        assert_eq!(first_open_gate(&record), Some("review"));
    }

    #[test]
    fn legacy_validating_phase_coerces() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(tmp.path(), ".bee/state.json", r#"{"phase":"validating"}"#);
        let record = read_state(tmp.path()).ok().unwrap();
        assert_eq!(record.phase, json!("planning"));
    }

    #[test]
    fn should_inject_dedup_rules() {
        let mut cache = Map::new();
        let now = now_ms();
        assert!(should_inject(&cache, "prompt", "h1", now)); // no entry
        cache.insert("prompt".into(), json!({"hash": "h1", "at": iso_from_ms(now - 60_000)}));
        assert!(!should_inject(&cache, "prompt", "h1", now)); // fresh same hash
        assert!(should_inject(&cache, "prompt", "h2", now)); // hash changed
        cache.insert(
            "prompt".into(),
            json!({"hash": "h1", "at": iso_from_ms(now - INJECT_INTERVAL_MS - 1000)}),
        );
        assert!(should_inject(&cache, "prompt", "h1", now)); // interval elapsed
        cache.insert("prompt".into(), json!({"hash": "h1", "at": "not-a-date"}));
        assert!(should_inject(&cache, "prompt", "h1", now)); // unparseable at
    }

    #[test]
    fn mark_injected_writes_cache_and_prunes_legacy() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(tmp.path(), ".bee/.inject-cache.json", r#"{"old":{"hash":"x","at":"t"}}"#);
        mark_injected(tmp.path(), "prompt", "h9").unwrap();
        let cache = std::fs::read_to_string(inject_cache_path(tmp.path())).unwrap();
        assert!(cache.contains("\"old\""), "legacy entries migrate: {cache}");
        assert!(cache.contains("\"prompt\""));
        assert!(!legacy_inject_cache_path(tmp.path()).exists());
    }

    #[test]
    fn corrupt_state_and_config_read_as_defaults_instead_of_delegating() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(tmp.path(), ".bee/state.json", "{broken");
        clear_plan_warnings();
        // readJson's null fallback → defaultState(): idle, no feature, no
        // gates — and one queued warning for the one bad file.
        let record = read_state(tmp.path()).expect("must not delegate");
        assert_eq!(record.phase, Value::String("idle".into()));
        assert_eq!(record.feature, Value::Null);
        assert_eq!(record.gates, default_gates());
        assert_eq!(PLAN_WARNINGS.with(|q| q.borrow().len()), 1);
        // The whole plan still runs natively: idle phase → no reminder text
        // worth injecting is required, but crucially it is not a delegate.
        let planned = plan(tmp.path(), &Map::new()).expect("corrupt state must stay native");
        assert!(planned.is_some());
        clear_plan_warnings();

        // A corrupt CONFIG is native too (crate::state::read_config_raw warns
        // and reads it as absent, so hookEnabled defaults to enabled).
        let tmp2 = tempfile::tempdir().unwrap();
        bee_repo(tmp2.path());
        write(tmp2.path(), ".bee/config.json", "{broken");
        assert!(plan(tmp2.path(), &Map::new()).expect("native").is_some());
        clear_plan_warnings();
    }

    #[test]
    fn corrupt_session_lane_and_cell_all_read_as_absent() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        clear_plan_warnings();
        write(tmp.path(), ".bee/state.json", r#"{"phase":"swarming","feature":"f1"}"#);
        write(tmp.path(), ".bee/sessions/s1.json", "{broken");
        // A corrupt session record reads as "no session": resolvePipeline
        // falls back to readState instead of delegating.
        assert!(read_session(tmp.path(), "s1").expect("native").is_none());
        match resolve_pipeline(tmp.path(), tmp.path(), Some("s1")) {
            Ok(Pipeline::Ok(rec)) => assert_eq!(rec.phase, Value::String("swarming".into())),
            _ => panic!("corrupt session must fall back to readState"),
        }
        // A corrupt lane record is the LANE_CORRUPT refusal (both warns), not
        // a delegate: same continuation LANE_MISSING already had.
        write(tmp.path(), ".bee/sessions/s2.json", r#"{"id":"s2","lane":"l1"}"#);
        write(tmp.path(), ".bee/lanes/l1.json", "{broken");
        assert!(matches!(
            resolve_pipeline(tmp.path(), tmp.path(), Some("s2")),
            Ok(Pipeline::Refused)
        ));
        // A corrupt cell is skipped from listCells; readable ones survive.
        write(tmp.path(), ".bee/cells/bad.json", "{broken");
        write(tmp.path(), ".bee/cells/c1.json", r#"{"id":"c1","status":"claimed"}"#);
        let cells = list_cells_top(tmp.path()).expect("native");
        assert_eq!(cells.len(), 1);
        clear_plan_warnings();
    }

    #[test]
    fn corrupt_inject_cache_reads_as_empty_and_reinjects() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        clear_plan_warnings();
        write(tmp.path(), ".bee/cache/inject-cache.json", "{broken");
        let cache = read_inject_cache(tmp.path()).expect("native");
        assert!(cache.is_empty());
        assert!(should_inject(&cache, "prompt", "h1", now_ms()));
        // A non-object cache is still a delegate (markInjected would throw).
        write(tmp.path(), ".bee/cache/inject-cache.json", "[1,2]");
        assert!(read_inject_cache(tmp.path()).is_err());
        clear_plan_warnings();
    }

    #[test]
    fn disabled_hook_plans_none() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(tmp.path(), ".bee/config.json", r#"{"hooks":{"prompt-context":false}}"#);
        assert!(matches!(plan(tmp.path(), &Map::new()), Ok(None)));
    }

    #[test]
    fn plan_reminds_and_nudges_on_active_execution_without_anchor() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(
            tmp.path(),
            ".bee/state.json",
            r#"{"phase":"swarming","feature":"feat-x","approved_gates":{"context":true,"shape":true,"execution":true}}"#,
        );
        let plan = plan(tmp.path(), &Map::new()).ok().flatten().unwrap();
        assert!(plan.inject_reminder);
        let nudge = plan.nudge.expect("anchor nudge fires");
        assert!(nudge.inject);
        assert_eq!(nudge.hash, ":feat-x:");
        assert!(nudge.message.starts_with(
            "bee: work is active and NO INTENT ANCHOR is stored (feature=feat-x). "
        ));
        assert!(nudge.message.ends_with(ANCHOR_NUDGE_COMMAND));
    }

    #[test]
    fn anchor_nudge_suppressed_by_present_anchor_or_terminal_phase() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(
            tmp.path(),
            ".bee/state.json",
            r#"{"phase":"swarming","feature":"feat-x","approved_gates":{"execution":true}}"#,
        );
        write(
            tmp.path(),
            ".bee/intent/feat-x.json",
            r#"{"request":"do the thing","acceptance":"it works"}"#,
        );
        let p = plan(tmp.path(), &Map::new()).ok().flatten().unwrap();
        assert!(p.nudge.is_none(), "anchor present suppresses the nudge");

        let tmp2 = tempfile::tempdir().unwrap();
        bee_repo(tmp2.path());
        write(tmp2.path(), ".bee/state.json", r#"{"phase":"idle","feature":"feat-x"}"#);
        let p2 = plan(tmp2.path(), &Map::new()).ok().flatten().unwrap();
        assert!(p2.nudge.is_none(), "terminal phase suppresses the nudge");
    }

    #[test]
    fn session_bound_lane_record_wins_over_default_state() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(tmp.path(), ".bee/state.json", r#"{"phase":"idle"}"#);
        write(
            tmp.path(),
            ".bee/sessions/s1.json",
            r#"{"id":"s1","started_at":"t","last_heartbeat":"t","lane":"lane-a"}"#,
        );
        write(
            tmp.path(),
            ".bee/lanes/lane-a.json",
            r#"{"feature":"lane-a","phase":"swarming","mode":"lane","next_action":"lane work"}"#,
        );
        let mut payload = Map::new();
        payload.insert("session_id".into(), json!("s1"));
        let p = plan(tmp.path(), &payload).ok().flatten().unwrap();
        assert!(p.reminder_text.starts_with("bee: phase=swarming mode=lane"));
        assert_eq!(p.inject_key, "prompt:s1");
    }

    #[test]
    fn heartbeat_touch_throttles_and_refreshes() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        let now = now_ms();
        write(
            tmp.path(),
            ".bee/sessions/s1.json",
            &format!(
                "{{\"id\":\"s1\",\"started_at\":\"t\",\"last_heartbeat\":{}}}",
                serde_json::to_string(&iso_from_ms(now - 5_000)).unwrap()
            ),
        );
        // Fresh heartbeat (5s old, under the 60s throttle) → no touch.
        assert!(!heartbeat_touch(tmp.path(), "s1", now).unwrap());
        // Stale heartbeat → touch rewrites last_heartbeat.
        write(
            tmp.path(),
            ".bee/sessions/s1.json",
            &format!(
                "{{\"id\":\"s1\",\"started_at\":\"t\",\"last_heartbeat\":{}}}",
                serde_json::to_string(&iso_from_ms(now - 120_000)).unwrap()
            ),
        );
        assert!(heartbeat_touch(tmp.path(), "s1", now).unwrap());
        let rewritten = std::fs::read_to_string(tmp.path().join(".bee/sessions/s1.json")).unwrap();
        assert!(rewritten.contains(&iso_from_ms(now)));
        // Missing session record: still "touched" (heartbeatStale(null)).
        assert!(heartbeat_touch(tmp.path(), "nope", now).unwrap());
        // Lock released.
        assert!(!lock_file_path(tmp.path(), "sessions").exists());
    }

    #[test]
    fn heartbeat_session_clears_dead_mark_and_stamps_revived_at() {
        // D8: UserPromptSubmit heartbeat path clears a dead mark too.
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(
            tmp.path(),
            ".bee/sessions/s1.json",
            &serde_json::to_string(&json!({
                "id": "s1",
                "last_heartbeat": "2020-01-01T00:00:00.000Z",
                "status": "dead",
                "dead_at": "2020-01-01T00:05:00.000Z",
                "lane": "l1"
            }))
            .unwrap(),
        );
        let now = now_ms();
        heartbeat_session(tmp.path(), "s1", now).unwrap();
        let after: Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".bee/sessions/s1.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(after["lane"], json!("l1")); // untouched fields survive
        assert!(after.get("status").is_none(), "{after:?}");
        assert!(after.get("dead_at").is_none(), "{after:?}");
        assert!(after["revived_at"].as_str().is_some(), "{after:?}");
        assert_ne!(after["last_heartbeat"], json!("2020-01-01T00:00:00.000Z"));
    }

    #[test]
    fn heartbeat_session_clears_closed_mark_and_stamps_revived_at() {
        // ser-2: UserPromptSubmit's heartbeat path clears a "closed" mark
        // (SessionEnd's clean exit) too, mirroring the "dead" revival above.
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(
            tmp.path(),
            ".bee/sessions/s1.json",
            &serde_json::to_string(&json!({
                "id": "s1",
                "last_heartbeat": "2020-01-01T00:00:00.000Z",
                "status": "closed",
                "closed_at": "2020-01-01T00:05:00.000Z",
                "lane": "l1"
            }))
            .unwrap(),
        );
        let now = now_ms();
        heartbeat_session(tmp.path(), "s1", now).unwrap();
        let after: Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".bee/sessions/s1.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(after["lane"], json!("l1")); // untouched fields survive
        assert!(after.get("status").is_none(), "{after:?}");
        assert!(after.get("closed_at").is_none(), "{after:?}");
        assert!(after["revived_at"].as_str().is_some(), "{after:?}");
        assert_ne!(after["last_heartbeat"], json!("2020-01-01T00:00:00.000Z"));
    }

    #[test]
    fn heartbeat_session_revives_a_released_mark_and_clears_it() {
        // ser-3: unlike state_sync.rs's PostToolUse heartbeat (which
        // deliberately leaves `released: true` intact), UserPromptSubmit IS
        // the re-engage signal for an explicit `state session release` — the
        // user speaking again clears status/closed_at/released exactly like
        // the plain "closed" revival above, and stamps revived_at.
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(
            tmp.path(),
            ".bee/sessions/s1.json",
            &serde_json::to_string(&json!({
                "id": "s1",
                "last_heartbeat": "2020-01-01T00:00:00.000Z",
                "status": "closed",
                "closed_at": "2020-01-01T00:05:00.000Z",
                "released": true,
                "lane": "l1"
            }))
            .unwrap(),
        );
        let now = now_ms();
        heartbeat_session(tmp.path(), "s1", now).unwrap();
        let after: Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".bee/sessions/s1.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(after["lane"], json!("l1")); // untouched fields survive
        assert!(after.get("status").is_none(), "{after:?}");
        assert!(after.get("closed_at").is_none(), "{after:?}");
        assert!(after.get("released").is_none(), "{after:?}");
        assert!(after["revived_at"].as_str().is_some(), "{after:?}");
        assert_ne!(after["last_heartbeat"], json!("2020-01-01T00:00:00.000Z"));
    }

    #[test]
    fn renew_claim_ttl_refreshes_only_own_claims() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        write(
            tmp.path(),
            ".bee/claims/c1.json",
            r#"{"cell":"c1","session":"s1","claimed_at":"2020-01-01T00:00:00.000Z","ttl_seconds":3600}"#,
        );
        write(
            tmp.path(),
            ".bee/claims/c2.json",
            r#"{"cell":"c2","session":"other","claimed_at":"2020-01-01T00:00:00.000Z","ttl_seconds":3600}"#,
        );
        let now = now_ms();
        renew_claim_ttl(tmp.path(), "s1", now).unwrap();
        let c1 = std::fs::read_to_string(tmp.path().join(".bee/claims/c1.json")).unwrap();
        let c2 = std::fs::read_to_string(tmp.path().join(".bee/claims/c2.json")).unwrap();
        assert!(c1.contains(&iso_from_ms(now)), "own claim renewed: {c1}");
        assert!(c2.contains("2020-01-01"), "other session's claim untouched");
        assert!(!tmp.path().join(".bee/claims/c1.adopting").exists(), "gate released");
    }

    #[test]
    fn renew_holds_updates_own_path_leases() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        let resource = "path:src/api";
        let hash = sha256_hex(resource);
        let acquired = iso_from_ms(now_ms() - 10_000);
        let expires = iso_from_ms(now_ms() + 50_000);
        write(
            tmp.path(),
            &format!(".bee/runtime/leases/paths/{hash}.json"),
            &format!(
                "{{\"resource\":\"path:src/api\",\"mode\":\"write\",\"workflow_id\":\"c1\",\"session_id\":\"s1\",\"workspace_id\":\"agent:a\",\"epoch\":0,\"acquired_at\":\"{acquired}\",\"expires_at\":\"{expires}\",\"kind\":\"lease\"}}"
            ),
        );
        let now = now_ms();
        renew_holds_by_session(tmp.path(), "s1", now).unwrap();
        let lease = std::fs::read_to_string(
            tmp.path().join(format!(".bee/runtime/leases/paths/{hash}.json")),
        )
        .unwrap();
        // Renewed by the lease's own original ttl (~60s) from `now`.
        let parsed: Value = serde_json::from_str(&lease).unwrap();
        let new_exp = parsed.get("expires_at").and_then(Value::as_str).unwrap();
        let new_ms = date_parse_ms(new_exp).unwrap();
        assert!((new_ms - now - 60_000).abs() <= 1500, "renewed ~60s ahead, got {new_exp}");
    }

    #[test]
    fn store_lock_acquire_and_contend() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        let mut g = acquire_store_lock_once(tmp.path(), "sessions").unwrap().unwrap();
        // Second acquisition while held: busy (fresh, live holder).
        assert!(acquire_store_lock_once(tmp.path(), "sessions").unwrap().is_none());
        g.release();
        assert!(!lock_file_path(tmp.path(), "sessions").exists());
        // Telemetry recorded both outcomes.
        let telemetry =
            std::fs::read_to_string(tmp.path().join(".bee/logs/contention.jsonl")).unwrap();
        assert!(telemetry.contains("\"result\":\"acquired\""));
        assert!(telemetry.contains("\"result\":\"busy\""));
    }

    #[test]
    fn sanitize_intent_key_matches_js() {
        assert_eq!(sanitize_intent_key(""), "default");
        assert_eq!(sanitize_intent_key("  "), "default");
        assert_eq!(sanitize_intent_key("feat/x!!y"), "feat-x-y");
        assert_eq!(sanitize_intent_key("..-abc-"), "abc");
        assert_eq!(sanitize_intent_key("ok_name.1-2"), "ok_name.1-2");
    }

    #[test]
    fn lock_name_sanitization_shape() {
        // sanitizeLockName('sessions') = 'sessions-' + sha256[0..8]
        let name = sanitize_lock_name("sessions");
        assert!(name.starts_with("sessions-"));
        assert_eq!(name.len(), "sessions-".len() + 8);
        // lease lock names carry ':' → '_'
        let lease = sanitize_lock_name("lease:abc");
        assert!(lease.starts_with("lease_abc-"));
    }

    // ─── D2 path ONE: the UserPromptSubmit hook clears the mark (ah-2 rework) ──
    //
    // Both tests below drive `run()` — the exact `pub fn` the `bee hook
    // prompt-context` dispatch calls — rather than `clear_and_reap_waiting_on_
    // best_effort` directly. That inner function was already reachable and
    // correct; what had zero coverage was the WIRING: that a real UserPromptSubmit
    // payload, run through plan()+execute() exactly as the host invokes it,
    // actually reaches the clear and can never let a failing clear take the
    // turn down with it.

    /// `Err2`'s `Exotic` payload has no `Debug` impl (by design — see
    /// jsval.rs), so a plain `.unwrap()`/`.expect()` on a `Result<_, Err2>`
    /// does not compile. Same shape as workflow_store/tests.rs's own `ok()`.
    fn ok<T, E>(r: Result<T, E>) -> T {
        match r {
            Ok(v) => v,
            Err(_) => panic!("unexpected error result"),
        }
    }

    /// TRUTH: "the human sending a message clears a live mark through the
    /// UserPromptSubmit hook." A mark set through the real setter, on disk,
    /// before `run()` — and gone after — with the outcome proving the native
    /// path decided (not `Outcome::Delegate`, which would mean the clear never
    /// ran at all).
    #[test]
    fn run_clears_a_live_default_waiting_on_mark_through_the_real_hook_entry_point() {
        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        ok(crate::verbs::state_group::set_default_state_waiting_on(
            tmp.path(),
            "question",
            "should we ship the v3 index?",
            "sess-asker",
        ));
        let before = ok(crate::verbs::state_group::read_state_peek(tmp.path()));
        assert!(
            crate::verbs::workflow_store::waiting_on_is_live(before.get("waiting_on")),
            "fixture must start with a live mark: {before:?}"
        );

        let payload = json!({ "cwd": tmp.path().to_string_lossy() }).to_string();
        let outcome = run(&[], &payload);
        assert!(
            matches!(outcome, Outcome::Done(_)),
            "a plain repo with no linked-worktree/lane edge case must decide natively, \
             not delegate — otherwise this proves nothing about the Rust clear path"
        );

        let after = ok(crate::verbs::state_group::read_state_peek(tmp.path()));
        assert!(
            !crate::verbs::workflow_store::waiting_on_is_live(after.get("waiting_on")),
            "the human's message must clear the live mark through the REAL hook entry \
             point, not merely through the inner clear function: {after:?}"
        );
    }

    /// TRUTH: "a failing clear inside the hook never fails the hook or the
    /// turn." `.bee/` itself (the direct parent of `state.json`) is made
    /// unwritable AFTER `.bee/locks/`, `.bee/cache/`, and `.bee/logs/` are
    /// pre-created and left writable — so lock acquisition, the reminder's
    /// inject-cache write, and crash logging are unaffected, and only the
    /// clear's own `write_json_atomic(state.json, ..)` tmp-file creation
    /// fails (a real, first-attempt failure — not a 100-retry lock-contention
    /// stall). By inspection `clear_and_reap_waiting_on_best_effort` only ever
    /// logs and swallows; this proves it end to end: the hook still reaches
    /// its normal completion (the inject cache is written, exactly as a
    /// successful run would) and still returns the same native `Done`
    /// outcome — never a crash, never a delegate.
    #[cfg(unix)]
    #[test]
    fn run_survives_a_failing_clear_without_failing_the_hook_or_skipping_its_output() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        bee_repo(tmp.path());
        ok(crate::verbs::state_group::set_default_state_waiting_on(
            tmp.path(),
            "question",
            "still waiting on you",
            "sess-asker",
        ));

        let bee_dir = tmp.path().join(".bee");
        // Pre-create every subdirectory the rest of the hook writes into so
        // ONLY the clear's write (a new tmp file directly inside `.bee/`)
        // fails — the lock, the inject cache, and crash logging all live in
        // their own subdirectories and stay writable.
        std::fs::create_dir_all(bee_dir.join("locks")).unwrap();
        std::fs::create_dir_all(bee_dir.join("cache")).unwrap();
        std::fs::create_dir_all(bee_dir.join("logs")).unwrap();
        let writable = std::fs::metadata(&bee_dir).unwrap().permissions();
        let mut readonly = writable.clone();
        readonly.set_mode(0o555); // r-x, no write: no new file can be created directly in .bee/
        std::fs::set_permissions(&bee_dir, readonly).unwrap();

        let payload = json!({ "cwd": tmp.path().to_string_lossy() }).to_string();
        let outcome = run(&[], &payload);

        // Restore write access before any assertion can panic and before the
        // tempdir's own Drop tries to remove a directory it can no longer
        // write into.
        std::fs::set_permissions(&bee_dir, writable).unwrap();

        assert!(
            matches!(outcome, Outcome::Done(_)),
            "a clear that fails on disk must still resolve natively — never Delegate, \
             and every path `execute()` can return here is ExitCode::SUCCESS by \
             construction, so `Done` at all is the hook surviving intact"
        );

        // The clear genuinely failed (proves the injected failure was real,
        // not accidentally a no-op): the mark is still live.
        let after = ok(crate::verbs::state_group::read_state_peek(tmp.path()));
        assert!(
            crate::verbs::workflow_store::waiting_on_is_live(after.get("waiting_on")),
            "the mark must still be live — this test's whole point is a clear that \
             fails, not one that quietly no-ops: {after:?}"
        );

        // And yet the hook ran all the way to its normal completion: the
        // reminder pipeline still wrote the inject cache exactly as it would
        // on a clean run, proving the failing clear never short-circuited the
        // turn's actual output.
        let cache = bee_dir.join("cache").join("inject-cache.json");
        assert!(
            cache.is_file(),
            "the hook must still reach and complete its normal reminder/inject step \
             after a failing clear — a failing clear that also skipped the reminder \
             would still be breaking the turn, just less visibly"
        );

        // And the failure was not silently dropped either: best-effort still
        // means "logged", never "swallowed with no trace".
        let crash_log = std::fs::read_to_string(bee_dir.join("logs").join("hooks.jsonl"))
            .unwrap_or_default();
        assert!(
            crash_log.contains("prompt-context"),
            "a failing clear must be crash-logged (log_crash), not silently eaten: {crash_log}"
        );
    }
}
