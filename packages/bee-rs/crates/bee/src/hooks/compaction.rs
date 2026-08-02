// compaction — native port of packages/bee/lib/compaction.mjs's two
// SessionStart(compact) entry points: `buildCompactCapsule` (the trimmed
// orientation a compacted session gets instead of the full preamble) and
// `appendCompactionRecord` (the fail-open resume half of the PreCompact
// record), plus the pieces of compaction.mjs they stand on
// (readCompactionCounts, survivalWarning, claimedCellId, compactCheck).
//
// CUTOVER MODULE — same reason as hooks/session_preamble.rs: the capsule was
// behind session-init's `Outcome::Delegate`, and there is no Node left to
// delegate to.
//
// ONE TRUTH FOR THE SHARED BLOCKS. compaction.mjs deliberately imports
// inject.mjs's onboardingLine / bypassBannerLines / handoffBlockLines /
// firstOpenGate rather than re-rendering them ("ONE truth for each of these
// three blocks, never two copies of it"). This port keeps that: those four
// renderers come from hooks/session_preamble.rs and are never re-implemented
// here.
//
// VENDORED-LIB READERS (the second half of this file). compaction.mjs pulls
// state.mjs (readState/readConfig/readOnboarding/resolvePipeline/
// controlRootFor/bypassLevel/gateApproved), claims.mjs (readClaim/sessionPath),
// cells.mjs (listCells/readCell), intent.mjs (readIntent/resumeBlock),
// knowledge.mjs (bundleMode) and reservations.mjs (listReservations). Where a
// port already exists as `pub(crate)` it is REUSED (state_group's
// default_state/spread_gates/read_claim, workflow_store's lane store,
// reservations' listReservations, state.rs's config merge). The rest is
// ported here once and shared with hooks/session_init.rs, which owns the same
// closure — see each function's provenance comment. Nothing in this file
// delegates: every corrupt-JSON arm warns through fsutil::warn_corrupt_json
// and takes Node's readJson fallback.
//
// WORDING: the one divergence taken for the cutover is that ANCHOR_NUDGE_COMMAND
// named the Node entry point (`node .bee/bin/bee.mjs intent set …`); it now
// names the binary, the only spelling that still exists. No emitted byte in
// this module contains ".mjs".

use crate::fsutil::{append_jsonl, read_json, warn_corrupt_json, ReadJson};
use crate::hooks::session_preamble::{
    bypass_banner_lines, first_open_gate, handoff_block_lines, onboarding_line, read_handoff,
    HandoffOutcome,
};
use crate::verbs::reservations::{js_disp, js_trim};
use crate::verbs::state_group::{coerce_legacy_phase, default_state, spread_gates};
use crate::verbs::workflow_store::read_lane_display;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// THE release version, single-sourced from `.claude-plugin/plugin.json`
/// at build time (R6 cutover — see src/version.rs). This used to be a
/// hand-maintained copy of state.mjs's `export const BEE_VERSION`.
use crate::version::BEE_VERSION;

/// state.mjs COMMAND_KEYS / WORKTREE_COMPANION_COMMAND_KEYS.
const COMMAND_KEYS: [&str; 3] = ["setup", "start", "test"];
const WORKTREE_COMPANION_COMMAND_KEYS: [&str; 3] = [
    "worktree_companion_start",
    "worktree_companion_end",
    "worktree_companion_mount",
];

/// The two events, and the only two (D5).
const COMPACT_EVENTS: [&str; 2] = ["precompact", "resume"];

/// D9 — a unit that has survived this many compactions gets the advisory.
const SURVIVAL_WARNING_THRESHOLD: i64 = 2;

/// D10 — the exact command the nudge names. Reworded for the cutover: the
/// Node entry point it used to spell no longer exists.
const ANCHOR_NUDGE_COMMAND: &str =
    ".bee/bin/bee intent set --request \"<the user's VERBATIM request>\" --acceptance \"<what done means>\"";

/// The terminal phases (state.mjs's set, kept local exactly as the .mjs does).
const TERMINAL_PHASES: [&str; 2] = ["idle", "compounding-complete"];

/// D19 — the sweep checks the capsule NEVER renders. Muting `anchor` is what
/// makes every byte of the capsule independent of whether `.bee/intent/` holds
/// anything; the hook already prefixes the anchor when one exists, so "no
/// anchor" stays visible from the ABSENCE of a lead block.
const CAPSULE_MUTED_CHECKS: [&str; 1] = ["anchor"];

// ───────────────────────── small JS-shaped helpers ─────────────────────────

/// compaction.mjs `norm` — a trimmed non-empty string, or null.
fn norm(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
        _ => None,
    }
}

fn norm_str(value: Option<&str>) -> Option<String> {
    value.map(js_trim).filter(|s| !s.is_empty()).map(str::to_string)
}

/// JS `value ?? fallback` for a display slot (null/absent take the fallback,
/// every other value stringifies).
fn nullish_disp(value: Option<&Value>, fallback: &str) -> String {
    match value {
        None | Some(Value::Null) => fallback.to_string(),
        Some(v) => js_disp(v),
    }
}

/// JS truthiness for a property read.
fn truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) | Some(Value::Bool(false)) => false,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        _ => true,
    }
}

fn vget<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_object().and_then(|m| m.get(key))
}

// ───────────────────── vendored-lib readers (shared) ───────────────────────

/// state.mjs readConfig's merge, fail-open. Node's readJson warned on a
/// corrupt file and returned the fallback (`{}`); so does this — never a
/// delegate, never a throw. Shared with hooks/session_init.rs.
pub(crate) fn read_config_failopen(root: &Path) -> Map<String, Value> {
    let read_obj = |file: PathBuf| -> Map<String, Value> {
        match read_json(&file) {
            ReadJson::Missing => Map::new(),
            ReadJson::Corrupt => {
                warn_corrupt_json(&file);
                Map::new()
            }
            ReadJson::Parsed(Value::Object(m)) => m,
            ReadJson::Parsed(_) => Map::new(),
        }
    };
    let tracked = read_obj(root.join(".bee").join("config.json"));
    let overlay = read_obj(root.join(".bee").join("config.local.json"));
    if overlay.is_empty() {
        return tracked;
    }
    match crate::state::merge_config_overlay(&Value::Object(tracked), &Value::Object(overlay)) {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

/// state.mjs normalizeCommands — the `commands` slice readConfig exposes.
fn config_commands(config: &Map<String, Value>) -> Map<String, Value> {
    let raw = match config.get("commands") {
        Some(Value::Object(m)) => m,
        _ => return Map::new(),
    };
    let mut commands = Map::new();
    for key in COMMAND_KEYS.iter().chain(WORKTREE_COMPANION_COMMAND_KEYS.iter()) {
        match raw.get(*key) {
            Some(Value::String(s)) if !js_trim(s).is_empty() => {
                commands.insert((*key).to_string(), json!(js_trim(s)));
            }
            // test-simple: only `test` accepts the array shape.
            Some(Value::Array(items)) if *key == "test" => {
                let list: Vec<Value> = items
                    .iter()
                    .filter_map(|c| match c {
                        Value::String(s) if !js_trim(s).is_empty() => Some(json!(js_trim(s))),
                        _ => None,
                    })
                    .collect();
                if !list.is_empty() {
                    commands.insert((*key).to_string(), Value::Array(list));
                }
            }
            _ => {}
        }
    }
    commands
}

/// state.mjs bypassLevel(root).
fn bypass_level(root: &Path) -> &'static str {
    crate::state::bypass_level(&read_config_failopen(root))
}

/// state.mjs readState — the fail-open read. A corrupt file warns and falls
/// back to defaultState(), exactly as Node's readJson + `!state` guard did.
/// Shared with hooks/session_init.rs.
pub(crate) fn read_state_failopen(root: &Path) -> Map<String, Value> {
    let file = root.join(".bee").join("state.json");
    let parsed = match read_json(&file) {
        ReadJson::Missing => return default_state(),
        ReadJson::Corrupt => {
            warn_corrupt_json(&file);
            return default_state();
        }
        ReadJson::Parsed(Value::Object(m)) => m,
        ReadJson::Parsed(_) => return default_state(),
    };
    let mut merged = default_state();
    for (k, v) in &parsed {
        merged.insert(k.clone(), v.clone());
    }
    let gates = spread_gates(parsed.get("approved_gates"))
        .unwrap_or_else(|_| match default_state().remove("approved_gates") {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        });
    merged.insert("approved_gates".into(), Value::Object(gates));
    let _ = coerce_legacy_phase(&mut merged);
    merged
}

/// state.mjs readOnboarding — fail-open readJson.
fn read_onboarding(root: &Path) -> Option<Value> {
    let file = root.join(".bee").join("onboarding.json");
    match read_json(&file) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => {
            warn_corrupt_json(&file);
            None
        }
        ReadJson::Parsed(v) => Some(v),
    }
}

/// state.mjs gateApproved.
fn gate_approved(record: &Map<String, Value>, gate: &str) -> bool {
    matches!(
        record.get("approved_gates").and_then(|g| g.get(gate)),
        Some(Value::Bool(true))
    )
}

/// claims.mjs requireId's shape test — a malformed id makes every path
/// helper throw, which every caller here treats as "no record".
pub(crate) fn well_formed_id(id: &str) -> bool {
    let id = js_trim(id);
    !id.is_empty() && !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

/// claims.mjs readSession's underlying record read (compactCheck reads the
/// file directly through readJson, not through readSession).
pub(crate) fn read_session_record(
    control_root: &Path,
    session_id: &str,
) -> Option<Map<String, Value>> {
    if !well_formed_id(session_id) {
        return None; // sessionPath's requireId throw → `file = null` → no record
    }
    let file = control_root.join(".bee").join("sessions").join(format!("{session_id}.json"));
    match read_json(&file) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => {
            warn_corrupt_json(&file);
            None
        }
        ReadJson::Parsed(Value::Object(m)) => Some(m),
        ReadJson::Parsed(_) => None,
    }
}

/// claims.mjs readClaim, resolved through controlRootFor (msn-18b PLANE RULE:
/// claims are control-plane). compaction.mjs's safeReadClaim swallows every
/// failure — so does this.
fn safe_read_claim(root: &Path, cell_id: &str) -> Option<Value> {
    let control = crate::hooks::session_init::control_root_for(root);
    if !well_formed_id(cell_id) {
        return None;
    }
    let file = control.join(".bee").join("claims").join(format!("{cell_id}.json"));
    match read_json(&file) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => {
            warn_corrupt_json(&file);
            None
        }
        ReadJson::Parsed(v @ (Value::Object(_) | Value::Array(_))) => Some(v),
        ReadJson::Parsed(_) => None,
    }
}

/// cells.mjs listCells(root, {}) — the active directory only.
fn list_cells(root: &Path) -> Vec<Value> {
    let dir = root.join(".bee").join("cells");
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue; // `archive` (or any dir) is never a cell
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".json") {
            names.push(name);
        }
    }
    names.sort();
    let mut cells = Vec::new();
    for name in names {
        let file = dir.join(&name);
        match read_json(&file) {
            ReadJson::Corrupt => warn_corrupt_json(&file),
            ReadJson::Parsed(v @ (Value::Object(_) | Value::Array(_))) => cells.push(v),
            _ => {}
        }
    }
    cells
}

/// cells.mjs readCell — active record first, then every archive feature dir.
fn read_cell(root: &Path, id: &str) -> Option<Value> {
    if id.is_empty() || !well_formed_id(id) {
        return None;
    }
    let cells_dir = root.join(".bee").join("cells");
    let read = |file: PathBuf| -> Option<Value> {
        match read_json(&file) {
            ReadJson::Missing => None,
            ReadJson::Corrupt => {
                warn_corrupt_json(&file);
                None
            }
            ReadJson::Parsed(v) => Some(v),
        }
    };
    if let Some(v) = read(cells_dir.join(format!("{id}.json"))) {
        return Some(v);
    }
    let archive = cells_dir.join("archive");
    let Ok(entries) = std::fs::read_dir(&archive) else { return None };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    dirs.sort();
    for dir in dirs {
        if let Some(v) = read(dir.join(format!("{id}.json"))) {
            return Some(v);
        }
    }
    None
}

// ── state.mjs resolvePipeline ─────────────────────────────────────────────

pub(crate) enum Pipeline {
    /// `{ok: true, source: 'default', record}`
    Default { record: Map<String, Value> },
    /// `{ok: true, source: 'lane', feature, record}`
    Lane { feature: String, record: Map<String, Value> },
    /// `{ok: false, code, feature, reason}`
    Refusal { code: &'static str, feature: String, reason: String },
}

impl Pipeline {
    /// `pipeline.ok && pipeline.record ? pipeline.record : readState(root)`.
    fn record(&self, root: &Path) -> Map<String, Value> {
        match self {
            Pipeline::Default { record } | Pipeline::Lane { record, .. } => record.clone(),
            Pipeline::Refusal { .. } => read_state_failopen(root),
        }
    }
    /// `pipeline.source === 'lane' ? norm(pipeline.feature) : null`.
    fn lane(&self) -> Option<String> {
        match self {
            Pipeline::Lane { feature, .. } => norm_str(Some(feature)),
            _ => None,
        }
    }
}

/// state.mjs resolvePipeline(root, {sessionId}). Sessions and lanes are
/// control-plane, so both reads go through controlRootFor.
pub(crate) fn resolve_pipeline(root: &Path, session_id: Option<&str>) -> Pipeline {
    let defaults = || Pipeline::Default { record: read_state_failopen(root) };
    let Some(session_id) = norm_str(session_id) else { return defaults() };
    let control_root = crate::hooks::session_init::control_root_for(root);
    // readSession: the record must exist AND self-identify.
    let Some(session) = read_session_record(&control_root, &session_id) else { return defaults() };
    if session.get("id") != Some(&Value::String(session_id.clone())) {
        return defaults();
    }
    let Some(bound) = norm(session.get("lane")) else { return defaults() };
    let sid = js_disp(session.get("id").unwrap_or(&Value::Null));
    // lanePath's requireLaneFeature throw → LANE_INVALID.
    if bound.contains('/') || bound.contains('\\') || bound.contains("..") {
        return Pipeline::Refusal {
            code: "LANE_INVALID",
            feature: bound.clone(),
            reason: format!(
                "session \"{sid}\" is bound to lane \"{bound}\", which is not a valid lane name (lane feature must be a plain id (no path separators).) — never guessed back to the default pipeline. FIX: rebind or unbind the session (claims bindSessionLane/unbindSessionLane)."
            ),
        };
    }
    let rel = format!(".bee{0}lanes{0}{bound}.json", std::path::MAIN_SEPARATOR);
    let file = control_root.join(".bee").join("lanes").join(format!("{bound}.json"));
    if !file.exists() {
        return Pipeline::Refusal {
            code: "LANE_MISSING",
            feature: bound.clone(),
            reason: format!(
                "session \"{sid}\" is bound to lane \"{bound}\" but {rel} does not exist — resolution never guesses back to the default pipeline. FIX: start the lane (startFeature with lane mode) or unbind the session."
            ),
        };
    }
    let record = match read_lane_display(&control_root, &bound) {
        Ok(record) => record,
        Err(_) => {
            // readJson's corrupt arm: Node warned, then readLane warned again
            // and returned null. Both warnings, then LANE_CORRUPT.
            warn_corrupt_json(&file);
            eprintln!(
                "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
            );
            None
        }
    };
    let Some(record) = record else {
        return Pipeline::Refusal {
            code: "LANE_CORRUPT",
            feature: bound.clone(),
            reason: format!(
                "session \"{sid}\" is bound to lane \"{bound}\" but its record is corrupt — display never guesses and mutations must refuse. FIX: inspect/restore {rel}, then retry."
            ),
        };
    };
    Pipeline::Lane { feature: bound, record }
}

// ── intent.mjs readIntent / resumeBlock ───────────────────────────────────
//
// COPIED LOGIC, minimally: verbs/intent_group.rs owns the same port but every
// piece of it is private and that file is owned by another agent right now.
// Provenance for everything below: lib/intent.mjs (sanitizeIntentKey /
// activeFeature / intentKeyCandidates / normalizeAnchor / readIntent /
// contextLines / resumeBlock) and its Rust twin verbs/intent_group.rs:51-313.
// One deliberate divergence: intent_group.rs delegates on a corrupt anchor
// file; there is nothing to delegate to, so this warns and treats the file as
// absent — which is Node's own readJson fallback.

const INTENT_SCHEMA_VERSION: &str = "1.0";
const DEFAULT_INTENT_KEY: &str = "default";
const RESUME_HEADER: &str =
    "## INTENT ANCHOR — read this FIRST (the objective; bee workflow state follows below)";

fn sanitize_intent_key(key: &str) -> String {
    let raw = js_trim(key);
    if raw.is_empty() {
        return DEFAULT_INTENT_KEY.to_string();
    }
    let mut safe = String::new();
    let mut in_run = false;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            safe.push(c);
            in_run = false;
        } else if !in_run {
            safe.push('-');
            in_run = true;
        }
    }
    let safe = safe.trim_start_matches(['-', '.']);
    let safe = safe.trim_end_matches('-');
    let safe: String = safe.chars().take(120).collect();
    if safe.is_empty() {
        DEFAULT_INTENT_KEY.to_string()
    } else {
        safe
    }
}

fn active_feature(root: &Path) -> Option<String> {
    let state = read_state_failopen(root);
    let phase = state.get("phase");
    if matches!(phase, Some(Value::String(s)) if TERMINAL_PHASES.contains(&s.as_str())) {
        return None;
    }
    norm(state.get("feature"))
}

fn intent_key_candidates(root: &Path, session: Option<&str>) -> Vec<String> {
    let mut candidates: Vec<String> = Vec::new();
    let push = |k: String, out: &mut Vec<String>| {
        if !out.contains(&k) {
            out.push(k);
        }
    };
    if let Some(feature) = active_feature(root) {
        push(sanitize_intent_key(&feature), &mut candidates);
    }
    if let Some(s) = norm_str(session) {
        push(sanitize_intent_key(&s), &mut candidates);
    }
    push(DEFAULT_INTENT_KEY.to_string(), &mut candidates);
    candidates
}

fn optional_string(v: Option<&Value>) -> Value {
    match v {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Value::String(js_trim(s).to_string()),
        _ => Value::Null,
    }
}

fn normalize_list(v: Option<&Value>) -> Vec<Value> {
    match v {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| js_trim(&js_disp(item)).to_string())
            .filter(|s| !s.is_empty())
            .map(Value::String)
            .collect(),
        Some(Value::String(s)) if !js_trim(s).is_empty() => s
            .split(',')
            .map(|p| js_trim(p).to_string())
            .filter(|p| !p.is_empty())
            .map(Value::String)
            .collect(),
        _ => Vec::new(),
    }
}

/// intent.mjs normalizeAnchor — a corrupt/half record reads as absent (D5).
fn normalize_anchor(raw: &Value, key: &str) -> Option<Map<String, Value>> {
    let Value::Object(raw) = raw else { return None };
    let request = match raw.get("request") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => s.clone(),
        _ => return None,
    };
    let str_or = |name: &str, fallback: Value| match raw.get(name) {
        Some(Value::String(s)) => Value::String(s.clone()),
        _ => fallback,
    };
    let mut anchor = Map::new();
    anchor.insert(
        "schema_version".into(),
        str_or("schema_version", Value::String(INTENT_SCHEMA_VERSION.into())),
    );
    anchor.insert(
        "key".into(),
        match raw.get("key") {
            Some(Value::String(s)) if !s.is_empty() => Value::String(s.clone()),
            _ => Value::String(key.to_string()),
        },
    );
    anchor.insert("written_at".into(), str_or("written_at", Value::Null));
    anchor.insert("request".into(), Value::String(request)); // VERBATIM
    anchor.insert("acceptance".into(), str_or("acceptance", Value::String(String::new())));
    for name in ["next_action", "feature", "lane", "cell"] {
        anchor.insert(name.into(), optional_string(raw.get(name)));
    }
    anchor.insert("do_not_reverse".into(), Value::Array(normalize_list(raw.get("do_not_reverse"))));
    anchor.insert("stop_conditions".into(), Value::Array(normalize_list(raw.get("stop_conditions"))));
    if let Some(Value::String(s)) = raw.get("advanced_at") {
        anchor.insert("advanced_at".into(), Value::String(s.clone()));
    }
    Some(anchor)
}

/// intent.mjs readIntent(root, {sessionId}) — first candidate key holding a
/// usable anchor. Fail-open on every arm.
pub(crate) fn read_intent(root: &Path, session: Option<&str>) -> Option<Map<String, Value>> {
    for key in intent_key_candidates(root, session) {
        let file = root.join(".bee").join("intent").join(format!("{key}.json"));
        match read_json(&file) {
            ReadJson::Missing => {}
            ReadJson::Corrupt => warn_corrupt_json(&file),
            ReadJson::Parsed(v) => {
                if let Some(anchor) = normalize_anchor(&v, &key) {
                    return Some(anchor);
                }
            }
        }
    }
    None
}

fn field_str<'a>(anchor: &'a Map<String, Value>, name: &str) -> Option<&'a str> {
    match anchor.get(name) {
        Some(Value::String(s)) => Some(s),
        _ => None,
    }
}

fn context_lines(anchor: &Map<String, Value>) -> Vec<String> {
    let mut lines = Vec::new();
    let join_list = |name: &str| -> Option<String> {
        match anchor.get(name) {
            Some(Value::Array(items)) if !items.is_empty() => {
                Some(items.iter().map(js_disp).collect::<Vec<_>>().join(" | "))
            }
            _ => None,
        }
    };
    if let Some(j) = join_list("do_not_reverse") {
        lines.push(format!("DO NOT REVERSE: {j}"));
    }
    if let Some(j) = join_list("stop_conditions") {
        lines.push(format!("STOP IF: {j}"));
    }
    let mut wheres = Vec::new();
    for name in ["feature", "lane", "cell"] {
        if let Some(s) = field_str(anchor, name) {
            if !s.is_empty() {
                wheres.push(format!("{name}={s}"));
            }
        }
    }
    if !wheres.is_empty() {
        lines.push(format!("CONTEXT: {}", wheres.join(" ")));
    }
    lines
}

/// intent.mjs resumeBlock.
pub(crate) fn resume_block(anchor: &Map<String, Value>) -> String {
    let mut lines = vec![
        RESUME_HEADER.to_string(),
        "ORIGINAL REQUEST (verbatim):".to_string(),
        field_str(anchor, "request").unwrap_or_default().to_string(),
        format!(
            "DONE MEANS: {}",
            match anchor.get("acceptance") {
                None => "undefined".to_string(),
                Some(v) => js_disp(v),
            }
        ),
    ];
    if let Some(n) = field_str(anchor, "next_action") {
        lines.push(format!("NEXT ACTION: {n}"));
    }
    lines.extend(context_lines(anchor));
    lines.push(
        "Everything below is workflow state — it serves the request above, it never replaces it."
            .to_string(),
    );
    lines.join("\n")
}

// ── knowledge.mjs bundleMode ──────────────────────────────────────────────
//
// COPIED LOGIC, minimally: verbs/knowledge.rs and verbs/drivers.rs both carry
// the full frontmatter parser and both files are owned by other agents.
// Provenance: lib/knowledge.mjs bundleDir/listBundleMarkdown/parseFrontmatter
// and their Rust twin verbs/drivers.rs:2849+ (`mod kctx`). Only the ONE
// question bundleMode asks is answered here — "does any non-reserved bundle
// markdown carry a non-empty `type` in a well-formed frontmatter block?" — so
// the parser reports pass/fail rather than the typed failure codes; every
// failure arm is "skip this file", exactly as bundleMode's own try/catch is.

fn product_root(root: &Path) -> PathBuf {
    let config = read_config_failopen(root);
    match config.get("product_root") {
        None | Some(Value::Null) => root.to_path_buf(),
        Some(Value::String(s)) if s.is_empty() => root.to_path_buf(),
        Some(Value::String(s)) => {
            let configured = Path::new(s);
            let resolved =
                if configured.is_absolute() { configured.to_path_buf() } else { root.join(configured) };
            if !resolved.is_dir() {
                eprintln!(
                    "bee: config product_root \"{s}\" -> \"{}\" is not an existing directory; product-doc reads (docs/backlog.md, docs/specs/) will find nothing until you fix .bee/config.json product_root. (GitHub #14)",
                    resolved.display()
                );
            }
            resolved
        }
        Some(other) => {
            let kind = match other {
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                _ => "object",
            };
            eprintln!(
                "bee: .bee/config.json product_root must be a string path (got {kind}); ignoring it and using the bee root."
            );
            root.to_path_buf()
        }
    }
}

fn list_bundle_markdown(dir: &Path) -> Vec<String> {
    fn walk(abs: &Path, rel: &str, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(abs) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            // a symlink could escape the bundle — never follow (D23)
            if std::fs::symlink_metadata(&path).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                walk(&path, &child_rel, out);
            } else if ft.is_file() && name.ends_with(".md") {
                out.push(child_rel);
            }
        }
    }
    let mut out = Vec::new();
    if dir.exists() {
        walk(dir, "", &mut out);
    }
    out.sort();
    out
}

fn key_re_ok(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// The emitted subset's scalar forms, reduced to "is this a legal scalar, and
/// what string does it carry". `None` = outside the subset (a parse failure).
fn parse_scalar_token(raw: &str) -> Option<Value> {
    if raw == "true" {
        return Some(Value::Bool(true));
    }
    if raw == "false" {
        return Some(Value::Bool(false));
    }
    if raw.starts_with('"') {
        return match serde_json::from_str::<Value>(raw) {
            Ok(v @ Value::String(_)) => Some(v),
            _ => None,
        };
    }
    if raw.starts_with('\'') {
        return None; // single-quoted scalars are outside the emitted subset
    }
    if matches!(raw.chars().next(), Some('&' | '*' | '!' | '|' | '>' | '%' | '@' | '`' | '{' | '}')) {
        return None;
    }
    Some(Value::String(raw.to_string()))
}

/// The top-level `type` of a well-formed frontmatter block, or None when the
/// file has no frontmatter / the block is outside the emitted subset.
fn frontmatter_type(text: &str) -> Option<String> {
    let open_len = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return None; // present: false
    };
    let mut cursor = open_len;
    let mut inner_end: Option<usize> = None;
    while cursor <= text.len() {
        let nl = text[cursor..].find('\n').map(|p| p + cursor);
        let line_end = nl.unwrap_or(text.len());
        let line = text[cursor..line_end].strip_suffix('\r').unwrap_or(&text[cursor..line_end]);
        if line == "---" {
            inner_end = Some(cursor);
            break;
        }
        let Some(nl) = nl else { break };
        cursor = nl + 1;
    }
    let inner_end = inner_end?; // unclosed_frontmatter
    let inner_raw = &text[open_len..inner_end];
    let mut inner_lines: Vec<&str> = if inner_raw.is_empty() {
        Vec::new()
    } else {
        inner_raw.split('\n').collect()
    };
    if !inner_lines.is_empty() {
        inner_lines.pop();
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut found: Option<String> = None;
    let mut in_bee_map = false;
    for raw_line in inner_lines {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() || line.contains('\t') {
            return None;
        }
        if let Some(inner) = line.strip_prefix("  ") {
            if !in_bee_map || inner.starts_with(' ') {
                return None;
            }
            // bee.* entries are parsed for legality but never carry `type`.
            let sep = inner.find(": ")?;
            if !key_re_ok(&inner[..sep]) || inner[sep + 2..].is_empty() {
                return None;
            }
            continue;
        }
        if line.starts_with(' ') {
            return None;
        }
        in_bee_map = false;
        if let Some(key) = line
            .strip_suffix(':')
            .filter(|k| !k.is_empty() && k.chars().all(|c| c != ':' && !c.is_whitespace()))
        {
            if !key_re_ok(key) || key != "bee" || !seen.insert("bee".to_string()) {
                return None;
            }
            in_bee_map = true;
            continue;
        }
        let sep = line.find(": ")?;
        let key = &line[..sep];
        if !key_re_ok(key) || !seen.insert(key.to_string()) {
            return None;
        }
        let raw = &line[sep + 2..];
        if raw.is_empty() {
            return None;
        }
        let parsed = if raw.starts_with('[') {
            if !raw.ends_with(']') {
                return None;
            }
            Value::Array(Vec::new()) // flow lists never carry `type`
        } else {
            parse_scalar_token(raw)?
        };
        if key == "type" {
            if let Value::String(s) = &parsed {
                if !s.is_empty() {
                    found = Some(s.clone());
                }
            }
        }
    }
    found
}

/// knowledge.mjs bundleMode(root).
fn bundle_mode(root: &Path) -> bool {
    let dir = product_root(root).join("docs").join("knowledge");
    if !dir.is_dir() {
        return false;
    }
    for rel in list_bundle_markdown(&dir) {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if base == "index.md" || base == "log.md" {
            continue;
        }
        let path = rel.split('/').fold(dir.clone(), |acc, seg| acc.join(seg));
        let Ok(bytes) = std::fs::read(&path) else { continue };
        let text = String::from_utf8_lossy(&bytes);
        if frontmatter_type(&text).is_some() {
            return true;
        }
    }
    false
}

// ─────────────────────────── cell ownership ────────────────────────────────

/// Cell ids the cross-session claims store attributes to this session.
fn claim_store_cell_ids(root: &Path, session: Option<&str>) -> Vec<String> {
    let Some(session) = session else { return Vec::new() };
    let control = crate::hooks::session_init::control_root_for(root);
    let Ok(entries) = std::fs::read_dir(control.join(".bee").join("claims")) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".json"))
        .collect();
    names.sort();
    let mut ids = Vec::new();
    for name in names {
        let id = name[..name.len() - ".json".len()].to_string();
        if let Some(claim) = safe_read_claim(root, &id) {
            if norm(vget(&claim, "session")).as_deref() == Some(session) {
                ids.push(id);
            }
        }
    }
    ids
}

fn attributed_to_session(root: &Path, cell: &Value, session: Option<&str>) -> bool {
    let trace_session = vget(cell, "trace").and_then(|t| norm(vget(t, "claim_session")));
    let id = match vget(cell, "id") {
        Some(Value::String(s)) => s.clone(),
        _ => return false,
    };
    if let Some(session) = session {
        if trace_session.as_deref() == Some(session) {
            return true;
        }
        return matches!(
            safe_read_claim(root, &id),
            Some(claim) if norm(vget(&claim, "session")).as_deref() == Some(session)
        );
    }
    // A caller with no session id owns only genuinely session-less claims.
    if trace_session.is_some() {
        return false;
    }
    if !matches!(vget(cell, "status"), Some(Value::String(s)) if s == "claimed") {
        return false;
    }
    match safe_read_claim(root, &id) {
        None => true,
        Some(claim) => norm(vget(&claim, "session")).is_none(),
    }
}

/// Every cell this session is on record as having claimed, in ANY status.
/// Missing cell records surface as `{id, status: null, missing: true}`.
fn session_owned_cells(root: &Path, session: Option<&str>) -> Vec<Value> {
    let mut order: Vec<String> = Vec::new();
    let mut owned: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for cell in list_cells(root) {
        let Some(Value::String(id)) = vget(&cell, "id").cloned() else { continue };
        if attributed_to_session(root, &cell, session) {
            if !owned.contains_key(&id) {
                order.push(id.clone());
            }
            owned.insert(id, cell);
        }
    }
    for id in claim_store_cell_ids(root, session) {
        if owned.contains_key(&id) {
            continue;
        }
        let cell = read_cell(root, &id)
            .unwrap_or_else(|| json!({ "id": id, "status": Value::Null, "missing": true }));
        order.push(id.clone());
        owned.insert(id, cell);
    }
    order.into_iter().filter_map(|id| owned.remove(&id)).collect()
}

fn claimed_cells(root: &Path, session: Option<&str>) -> Vec<Value> {
    session_owned_cells(root, session)
        .into_iter()
        .filter(|cell| matches!(vget(cell, "status"), Some(Value::String(s)) if s == "claimed"))
        .collect()
}

/// The single cell a compaction record names: most recently claimed first,
/// id as tiebreak.
fn claimed_cell_id(root: &Path, session: Option<&str>) -> Option<String> {
    let mut cells = claimed_cells(root, session);
    if cells.is_empty() {
        return None;
    }
    let claimed_at = |cell: &Value| -> String {
        match vget(cell, "trace").and_then(|t| vget(t, "claimed_at")) {
            None | Some(Value::Null) => String::new(),
            Some(v) => js_disp(v),
        }
    };
    let id_of = |cell: &Value| -> String { js_disp(vget(cell, "id").unwrap_or(&Value::Null)) };
    cells.sort_by(|a, b| {
        claimed_at(b).cmp(&claimed_at(a)).then_with(|| id_of(a).cmp(&id_of(b)))
    });
    Some(id_of(&cells[0]))
}

// ───────────────────────── the record (D4/D5) ──────────────────────────────

fn compaction_log_path(root: &Path) -> PathBuf {
    root.join(".bee").join("logs").join("compaction.jsonl")
}

/// fsutil.mjs readJsonl — corrupt lines are skipped, never fatal.
fn read_compaction_records(root: &Path) -> Vec<Value> {
    let Ok(bytes) = std::fs::read(compaction_log_path(root)) else { return Vec::new() };
    let text = String::from_utf8_lossy(&bytes);
    let mut events = Vec::new();
    for line in text.split('\n') {
        let trimmed = js_trim(line.strip_suffix('\r').unwrap_or(line));
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            events.push(v);
        }
    }
    events
}

/// D5 — the PLAIN prior counts. `resume` records are never counted, at all.
fn read_compaction_counts(root: &Path, session: Option<&str>, cell: Option<&str>) -> (i64, i64) {
    let mut compact_index = 0i64;
    let mut cell_compact_count = 0i64;
    for record in read_compaction_records(root) {
        if !record.is_object() {
            continue;
        }
        if !matches!(vget(&record, "event"), Some(Value::String(s)) if s == "precompact") {
            continue;
        }
        if norm(vget(&record, "session")).as_deref() != session {
            continue;
        }
        compact_index += 1;
        if let Some(unit) = cell {
            if norm(vget(&record, "cell")).as_deref() == Some(unit) {
                cell_compact_count += 1;
            }
        }
    }
    (compact_index, cell_compact_count)
}

/// D9 — the survival advisory, or None. Advice, never a verdict.
fn survival_warning(count: i64) -> Option<String> {
    if count < SURVIVAL_WARNING_THRESHOLD {
        return None;
    }
    Some(format!(
        "this unit has now survived {count} compactions — it may be oversized; consider capping at the next green verify and handing off"
    ))
}

/// compaction.mjs pipelineFields — lane / feature / phase for a session. On a
/// typed refusal the lane NAME is still recorded; feature and phase stay null.
fn pipeline_fields(root: &Path, session: Option<&str>) -> (Value, Value, Value) {
    let pipeline = resolve_pipeline(root, session);
    if let Pipeline::Refusal { feature, .. } = &pipeline {
        return (
            norm_str(Some(feature)).map(Value::String).unwrap_or(Value::Null),
            Value::Null,
            Value::Null,
        );
    }
    let record = pipeline.record(root);
    let lane = pipeline.lane().map(Value::String).unwrap_or(Value::Null);
    (
        lane,
        record.get("feature").cloned().unwrap_or(Value::Null),
        record.get("phase").cloned().unwrap_or(Value::Null),
    )
}

/// compaction.mjs `appendCompactionRecord(root, { event, sessionId })`.
/// Fail-open inside the module (D4): a log failure never affects what renders.
///
/// DIVERGENCE, named: the .mjs throws on an event outside COMPACT_EVENTS
/// (an argument error at the one call site that could corrupt every later
/// count). The only caller left is the SessionStart hook's `resume` append,
/// and a hook must never crash, so an unknown event writes nothing instead.
pub fn append_compaction_record(root: &Path, event: &str, session_id: Option<&str>) {
    if !COMPACT_EVENTS.contains(&event) {
        return;
    }
    let session = norm_str(session_id);
    let (lane, feature, phase) = pipeline_fields(root, session.as_deref());
    let cell = claimed_cell_id(root, session.as_deref());
    let (prior_index, prior_cell) =
        read_compaction_counts(root, session.as_deref(), cell.as_deref());
    // Only a precompact counts itself; a resume carries the plain prior count.
    let increment = if event == "precompact" { 1 } else { 0 };

    let mut record = Map::new();
    record.insert("ts".into(), json!(crate::hooks::adapter::now_iso()));
    record.insert("event".into(), json!(event));
    record.insert(
        "session".into(),
        session.clone().map(Value::String).unwrap_or(Value::Null),
    );
    record.insert("lane".into(), lane);
    record.insert("feature".into(), feature);
    record.insert("phase".into(), phase);
    record.insert("cell".into(), cell.clone().map(Value::String).unwrap_or(Value::Null));
    record.insert("compact_index".into(), json!(prior_index + increment));
    record.insert(
        "cell_compact_count".into(),
        json!(if cell.is_some() { prior_cell + increment } else { 0 }),
    );
    record.insert(
        "anchor_present".into(),
        json!(read_intent(root, session.as_deref()).is_some()),
    );

    // D4 — a log failure never changes the caller's return value or exit code.
    let _ = append_jsonl(&compaction_log_path(root), &Value::Object(record));
}

// ──────────────────── the D12/D13 integrity sweep ──────────────────────────

struct Check {
    name: &'static str,
    ok: bool,
    detail: String,
    code: Option<&'static str>,
}

/// D12/D13 — the read-only integrity sweep. It REPORTS: it never repairs,
/// never releases, never blocks, and writes nothing at all.
fn compact_check(root: &Path, session: Option<&str>) -> Vec<Check> {
    let mut checks: Vec<Check> = Vec::new();
    let mut add = |name: &'static str, ok: bool, detail: String, code: Option<&'static str>| {
        checks.push(Check { name, ok, detail, code });
    };

    // 1. The session record exists and its STORED id matches.
    match session {
        None => add(
            "session_record",
            true,
            "no session id supplied — session-scoped checks are skipped, not failed.".to_string(),
            None,
        ),
        Some(session) => {
            let control = crate::hooks::session_init::control_root_for(root);
            match read_session_record(&control, session) {
                None => add(
                    "session_record",
                    false,
                    format!("no readable session record for \"{session}\" under .bee/sessions/."),
                    Some("SESSION_MISSING"),
                ),
                Some(stored) if norm(stored.get("id")).as_deref() != Some(session) => add(
                    "session_record",
                    false,
                    format!(
                        ".bee/sessions/{session}.json stores id \"{}\" — the record does not describe this session.",
                        stored.get("id").map(js_disp).unwrap_or_else(|| "undefined".into())
                    ),
                    Some("SESSION_ID_MISMATCH"),
                ),
                Some(_) => add(
                    "session_record",
                    true,
                    format!("session record present and self-consistent ({session})."),
                    None,
                ),
            }
        }
    }

    // 2. The lane binding resolves — the typed refusal is SURFACED.
    let pipeline = resolve_pipeline(root, session);
    match &pipeline {
        Pipeline::Refusal { code, reason, .. } => {
            add("lane_binding", false, reason.clone(), Some(code))
        }
        Pipeline::Lane { feature, .. } => add(
            "lane_binding",
            true,
            format!("bound to lane \"{feature}\" and it resolves."),
            None,
        ),
        Pipeline::Default { .. } => add(
            "lane_binding",
            true,
            "no lane binding — the default pipeline applies.".to_string(),
            None,
        ),
    }
    let record = pipeline.record(root);

    // 3. Every cell this session claimed is still claimed and still owned.
    let owned = session_owned_cells(root, session);
    let mut ownership_problems: Vec<String> = Vec::new();
    for cell in &owned {
        let id = match vget(cell, "id") {
            Some(Value::String(s)) => s.clone(),
            _ => "unknown".to_string(),
        };
        if truthy(vget(cell, "missing")) {
            ownership_problems.push(format!("{id}: no cell record found"));
            continue;
        }
        let status = nullish_disp(vget(cell, "status"), "undefined");
        if !matches!(vget(cell, "status"), Some(Value::String(s)) if s == "claimed") {
            ownership_problems.push(format!("{id}: no longer claimed (status={status})"));
            continue;
        }
        let trace_session = vget(cell, "trace").and_then(|t| norm(vget(t, "claim_session")));
        if let (Some(session), Some(trace)) = (session, trace_session.as_deref()) {
            if trace != session {
                ownership_problems.push(format!("{id}: cell trace names session \"{trace}\""));
            }
        }
        let claim_session = safe_read_claim(root, &id).and_then(|c| norm(vget(&c, "session")));
        if let (Some(session), Some(owner)) = (session, claim_session.as_deref()) {
            if owner != session {
                ownership_problems
                    .push(format!("{id}: claim record is owned by session \"{owner}\""));
            }
        }
    }
    let claimed: Vec<&Value> = owned
        .iter()
        .filter(|c| matches!(vget(c, "status"), Some(Value::String(s)) if s == "claimed"))
        .collect();
    let claimed_ids: Vec<String> = claimed
        .iter()
        .map(|c| js_disp(vget(c, "id").unwrap_or(&Value::Null)))
        .collect();
    if owned.is_empty() {
        add("claimed_cells", true, "this session holds no cell claims.".to_string(), None);
    } else if !ownership_problems.is_empty() {
        add("claimed_cells", false, ownership_problems.join("; "), Some("CLAIM_DRIFT"));
    } else {
        add(
            "claimed_cells",
            true,
            format!("still claimed and still owned: {}.", claimed_ids.join(", ")),
            None,
        );
    }

    // 4. approved_gates.execution is still true whenever a cell is claimed.
    if claimed.is_empty() {
        add(
            "execution_gate",
            true,
            "no cell claimed — the execution gate is not in question.".to_string(),
            None,
        );
    } else if gate_approved(&record, "execution") {
        add("execution_gate", true, "execution gate is approved.".to_string(), None);
    } else {
        add(
            "execution_gate",
            false,
            format!(
                "execution gate is NOT approved while {} {} claimed — the claim outlived its authorization.",
                claimed_ids.join(", "),
                if claimed.len() == 1 { "is" } else { "are" }
            ),
            Some("GATE_REVOKED"),
        );
    }

    // 5. The claimed cell's dependencies are still capped.
    let mut dep_problems: Vec<String> = Vec::new();
    for cell in &claimed {
        let id = js_disp(vget(cell, "id").unwrap_or(&Value::Null));
        let deps = match vget(cell, "deps") {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        for dep in deps {
            let dep_name = js_disp(&dep);
            match dep.as_str().and_then(|d| read_cell(root, d)) {
                None => dep_problems.push(format!("{id}: dep \"{dep_name}\" has no cell record")),
                Some(dep_cell) => {
                    if !matches!(vget(&dep_cell, "status"), Some(Value::String(s)) if s == "capped") {
                        dep_problems.push(format!(
                            "{id}: dep \"{dep_name}\" is {}, not capped",
                            nullish_disp(vget(&dep_cell, "status"), "undefined")
                        ));
                    }
                }
            }
        }
    }
    if claimed.is_empty() {
        add("deps_capped", true, "no cell claimed — no dependencies to re-check.".to_string(), None);
    } else if !dep_problems.is_empty() {
        add("deps_capped", false, dep_problems.join("; "), Some("DEP_UNCAPPED"));
    } else {
        add(
            "deps_capped",
            true,
            "every dependency of every claimed cell is still capped.".to_string(),
            None,
        );
    }

    // 6. This session's reservations are still held by it.
    let (held, unbound, expired) = reservation_tally(root, session, &claimed_ids);
    let mut reservation_detail = format!(
        "{held} active hold(s) still owned by this session; {unbound} session-less (unbound) row(s) on this session's cells — legacy/intra-swarm rows, never a mismatch (D13)"
    );
    if expired.is_empty() {
        reservation_detail.push('.');
    } else {
        reservation_detail
            .push_str(&format!("; EXPIRED and no longer held: {}", expired.join(", ")));
    }
    if expired.is_empty() {
        add("reservations", true, reservation_detail, None);
    } else {
        add("reservations", false, reservation_detail, Some("HOLD_EXPIRED"));
    }

    // 7. An anchor exists. (Muted in the capsule — D19.)
    if read_intent(root, session).is_some() {
        add("anchor", true, "an intent anchor is stored.".to_string(), None);
    } else {
        add(
            "anchor",
            false,
            format!("no intent anchor is stored — write it verbatim: {ANCHOR_NUDGE_COMMAND}"),
            Some("ANCHOR_MISSING"),
        );
    }

    checks
}

/// The reservation half of check 6. reservations.mjs's own listReservations is
/// reused (it self-resolves the control root); the expired set is the
/// difference between the full listing and the active-only one, because the
/// TTL fields of the ported `Resv` are private to that module.
fn reservation_tally(
    root: &Path,
    session: Option<&str>,
    claimed_ids: &[String],
) -> (usize, usize, Vec<String>) {
    let root_str = root.to_string_lossy().into_owned();
    let now = crate::verbs::reservations::now_ms();
    let all = match crate::verbs::reservations::list_reservations(&root_str, false, now) {
        Ok(rows) => rows,
        Err(_) => return (0, 0, Vec::new()),
    };
    let active: BTreeSet<String> =
        match crate::verbs::reservations::list_reservations(&root_str, true, now) {
            Ok(rows) => rows.into_iter().map(|r| r.path).collect(),
            Err(_) => all.iter().map(|r| r.path.clone()).collect(),
        };
    let claimed: BTreeSet<&str> = claimed_ids.iter().map(String::as_str).collect();
    let mut held = 0usize;
    let mut unbound = 0usize;
    let mut expired: Vec<String> = Vec::new();
    for row in &all {
        let row_session = norm(row.session.as_ref());
        match (session, row_session.as_deref()) {
            (Some(session), Some(row_session)) if row_session == session => {
                if active.contains(&row.path) {
                    held += 1;
                } else {
                    expired.push(row.path.clone());
                }
            }
            (_, None) => {
                let cell = row.cell.as_ref().map(js_disp).unwrap_or_default();
                if claimed.contains(cell.as_str()) {
                    unbound += 1;
                }
            }
            _ => {}
        }
    }
    (held, unbound, expired)
}

// ───────────────────────── the D6 compact capsule ──────────────────────────

/// compaction.mjs `buildCompactCapsule(root, { sessionId, handoffOutcome })`.
///
/// D6 item order, verbatim, items 2-12:
///   2 STATE MISMATCH (the D12 sweep, anchor check muted)
///   3 the onboarding-MISSING line
///   4 the HANDOFF block, wait-instruction verbatim, WITH its refusal reason
///   5 the gate-bypass banner
///   6 phase / mode / feature / lane
///   7 the claimed cell, its verify command, its dependency status
///   8 the first open gate
///   9 next_action
///  10 the recorded standard commands
///  11 the compaction survival count + the D9 advisory
///  12 a POINTER to the critical patterns, never the digest (D7)
///
/// IT NEVER RENDERS THE ANCHOR (D19) — the hook prefixes it.
/// `handoff_outcome` is MANDATORY at the call site (D27): without it a
/// compacted session silently loses the line saying WHY it must wait.
pub fn build_compact_capsule(
    root: &Path,
    session_id: Option<&str>,
    handoff_outcome: Option<&HandoffOutcome>,
) -> String {
    let session = norm_str(session_id);
    let session = session.as_deref();
    let mut sections: Vec<Vec<String>> = Vec::new();

    sections.push(vec![format!("## bee v{BEE_VERSION} — compact capsule (source=compact)")]);

    // ── item 2: the integrity sweep, reported and never repaired.
    let mismatches: Vec<Check> = compact_check(root, session)
        .into_iter()
        .filter(|c| !c.ok && !CAPSULE_MUTED_CHECKS.contains(&c.name))
        .collect();
    if !mismatches.is_empty() {
        let mut block =
            vec!["⚠ STATE MISMATCH — disk state overrides conversational recollection:".to_string()];
        for entry in &mismatches {
            block.push(format!(
                "- {}{}: {}",
                entry.name,
                entry.code.map(|c| format!(" ({c})")).unwrap_or_default(),
                entry.detail
            ));
        }
        sections.push(block);
    }

    // ── item 3: onboarding, but only the MISSING arm.
    let onboarding = read_onboarding(root).filter(|v| truthy(Some(v)));
    if onboarding.is_none() {
        sections.push(vec![onboarding_line(None)]);
    }

    // ── item 4: the HANDOFF block, refusal reason included (D26/D27).
    if let Some(handoff) = read_handoff(root) {
        let lines = handoff_block_lines(&handoff, handoff_outcome);
        if !lines.is_empty() {
            sections.push(lines);
        }
    }

    // ── item 5: the bypass banner — mandatory wherever orientation renders.
    let banner = bypass_banner_lines(bypass_level(root));
    if !banner.is_empty() {
        sections.push(banner);
    }

    // ── items 6-9: which pipeline, which cell, which gate, what next.
    let pipeline = resolve_pipeline(root, session);
    let record = pipeline.record(root);
    let lane = pipeline.lane();

    let mut status = vec![format!(
        "- Phase: {} | Mode: {} | Feature: {} | Lane: {}",
        nullish_disp(record.get("phase"), "unknown"),
        nullish_disp(record.get("mode"), "none"),
        nullish_disp(record.get("feature"), "none"),
        lane.as_deref().unwrap_or("none"),
    )];
    let cell_id = claimed_cell_id(root, session);
    match &cell_id {
        Some(cell_id) => {
            let cell = read_cell(root, cell_id);
            let title = cell
                .as_ref()
                .and_then(|c| vget(c, "title").filter(|v| truthy(Some(v))))
                .map(|v| format!(" — {}", js_disp(v)))
                .unwrap_or_default();
            let cell_lane = cell
                .as_ref()
                .and_then(|c| vget(c, "lane").filter(|v| truthy(Some(v))))
                .map(|v| format!(", {} lane", js_disp(v)))
                .unwrap_or_default();
            let cell_status = cell
                .as_ref()
                .map(|c| nullish_disp(vget(c, "status"), "unknown"))
                .unwrap_or_else(|| "unknown".to_string());
            status.push(format!("- Cell: {cell_id}{title} ({cell_status}{cell_lane})"));
            let verify = cell.as_ref().and_then(|c| vget(c, "verify").filter(|v| truthy(Some(v))));
            status.push(match verify {
                Some(v) => format!("- Verify: `{}`", js_disp(v)),
                None => "- Verify: NONE RECORDED — a cell with no runnable verify cannot be capped"
                    .to_string(),
            });
            let deps = match cell.as_ref().and_then(|c| vget(c, "deps")) {
                Some(Value::Array(items)) => items.clone(),
                _ => Vec::new(),
            };
            status.push(if deps.is_empty() {
                "- Deps: none".to_string()
            } else {
                format!(
                    "- Deps: {}",
                    deps.iter()
                        .map(|dep| {
                            let name = js_disp(dep);
                            let dep_status = dep
                                .as_str()
                                .and_then(|d| read_cell(root, d))
                                .map(|c| nullish_disp(vget(&c, "status"), "no cell record"))
                                .unwrap_or_else(|| "no cell record".to_string());
                            format!("{name} ({dep_status})")
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            });
        }
        None => status.push("- Cell: none claimed by this session".to_string()),
    }
    if let Some(gate) = first_open_gate(&record) {
        status.push(format!("- Gate pending: {gate}"));
    }
    if let Some(next_action) = norm(record.get("next_action")) {
        status.push(format!("- Next action: {next_action}"));
    }
    sections.push(status);

    // ── item 10: the recorded standard commands.
    let commands = config_commands(&read_config_failopen(root));
    let recorded: Vec<&str> =
        COMMAND_KEYS.iter().copied().filter(|key| truthy(commands.get(*key))).collect();
    if !recorded.is_empty() {
        let mut block = vec!["### Standard commands (host project)".to_string()];
        for key in recorded {
            block.push(format!("- {key}: `{}`", js_disp(&commands[key])));
        }
        sections.push(block);
    }

    // ── item 11: the survival count and, when it applies, the D9 advisory.
    // Silent on a repo with no records at all (D15).
    let (compact_index, cell_compact_count) =
        read_compaction_counts(root, session, cell_id.as_deref());
    if compact_index > 0 || cell_compact_count > 0 {
        let per_cell = cell_id
            .as_deref()
            .map(|id| format!(" | {cell_compact_count} on {id}"))
            .unwrap_or_default();
        let mut block =
            vec![format!("- Compactions survived: {compact_index} in this session{per_cell}")];
        if let Some(warning) = survival_warning(cell_compact_count) {
            block.push(format!("- ⚠ {warning}."));
        }
        sections.push(block);
    }

    // ── item 12: the POINTER (D7). Never dropped, never expanded.
    let pointer = if bundle_mode(root) {
        "docs/knowledge/index.md (\"## Critical patterns\")"
    } else {
        "docs/history/learnings/critical-patterns.md"
    };
    sections.push(vec![format!(
        "- Critical patterns: {pointer} — re-read the doctrine after a compaction (AGENTS.md startup step 1); this capsule carries the pointer, never the digest (D7)."
    )]);

    sections
        .into_iter()
        .filter(|block| !block.is_empty())
        .map(|block| block.join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(dir: &Path) {
        std::fs::create_dir_all(dir.join(".bee").join("cells")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
    }

    #[test]
    fn capsule_opens_with_the_compact_heading_and_never_names_the_node_runtime() {
        let tmp = tempfile::tempdir().unwrap();
        repo(tmp.path());
        let text = build_compact_capsule(tmp.path(), Some("s1"), None);
        assert!(text.starts_with("## bee v"), "{text}");
        assert!(text.contains("compact capsule (source=compact)"), "{text}");
        assert!(!text.contains(".mjs"), "the capsule must not name the Node runtime: {text}");
        // item 12 is never dropped.
        assert!(text.contains("- Critical patterns: docs/history/learnings/critical-patterns.md"));
    }

    #[test]
    fn the_resume_record_carries_the_plain_prior_count() {
        let tmp = tempfile::tempdir().unwrap();
        repo(tmp.path());
        append_compaction_record(tmp.path(), "resume", Some("s1"));
        let rows = read_compaction_records(tmp.path());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["event"], json!("resume"));
        assert_eq!(rows[0]["session"], json!("s1"));
        // A resume never counts itself (D5).
        assert_eq!(rows[0]["compact_index"], json!(0));
        assert_eq!(rows[0]["cell_compact_count"], json!(0));
        assert_eq!(rows[0]["anchor_present"], json!(false));
        // An unknown event writes nothing at all.
        append_compaction_record(tmp.path(), "nonsense", Some("s1"));
        assert_eq!(read_compaction_records(tmp.path()).len(), 1);
    }

    #[test]
    fn counts_only_precompacts_for_this_session() {
        let tmp = tempfile::tempdir().unwrap();
        repo(tmp.path());
        let log = compaction_log_path(tmp.path());
        for row in [
            json!({"event": "precompact", "session": "s1", "cell": "c1"}),
            json!({"event": "resume", "session": "s1", "cell": "c1"}),
            json!({"event": "precompact", "session": "s2", "cell": "c1"}),
            json!({"event": "precompact", "session": "s1", "cell": "c2"}),
        ] {
            append_jsonl(&log, &row).unwrap();
        }
        assert_eq!(read_compaction_counts(tmp.path(), Some("s1"), Some("c1")), (2, 1));
        assert_eq!(read_compaction_counts(tmp.path(), Some("s2"), None), (1, 0));
    }

    #[test]
    fn survival_advisory_fires_only_at_the_threshold() {
        assert!(survival_warning(1).is_none());
        let warning = survival_warning(2).unwrap();
        assert!(warning.starts_with("this unit has now survived 2 compactions"), "{warning}");
    }

    #[test]
    fn survival_block_is_silent_until_a_precompact_exists() {
        let tmp = tempfile::tempdir().unwrap();
        repo(tmp.path());
        assert!(!build_compact_capsule(tmp.path(), Some("s1"), None)
            .contains("Compactions survived"));
        append_jsonl(
            &compaction_log_path(tmp.path()),
            &json!({"event": "precompact", "session": "s1"}),
        )
        .unwrap();
        assert!(build_compact_capsule(tmp.path(), Some("s1"), None)
            .contains("- Compactions survived: 1 in this session"));
    }

    #[test]
    fn the_capsule_renders_the_recorded_standard_commands() {
        let tmp = tempfile::tempdir().unwrap();
        repo(tmp.path());
        std::fs::write(
            tmp.path().join(".bee").join("config.json"),
            r#"{"commands":{"test":"npm test","nope":"x"}}"#,
        )
        .unwrap();
        let text = build_compact_capsule(tmp.path(), Some("s1"), None);
        assert!(text.contains("### Standard commands (host project)"), "{text}");
        assert!(text.contains("- test: `npm test`"), "{text}");
        assert!(!text.contains("nope"), "{text}");
    }

    #[test]
    fn a_corrupt_config_warns_and_falls_back_instead_of_failing() {
        let tmp = tempfile::tempdir().unwrap();
        repo(tmp.path());
        std::fs::write(tmp.path().join(".bee").join("config.json"), "{broken").unwrap();
        assert!(read_config_failopen(tmp.path()).is_empty());
        // and the capsule still renders.
        assert!(build_compact_capsule(tmp.path(), Some("s1"), None).contains("compact capsule"));
    }

    #[test]
    fn frontmatter_type_accepts_the_emitted_subset_and_rejects_the_rest() {
        assert_eq!(frontmatter_type("---\ntype: concept\n---\nbody\n").as_deref(), Some("concept"));
        assert_eq!(
            frontmatter_type("---\ntype: \"concept\"\nbee:\n  id: x\n---\n").as_deref(),
            Some("concept")
        );
        assert!(frontmatter_type("no frontmatter\n").is_none());
        assert!(frontmatter_type("---\ntype: concept\n").is_none(), "unclosed");
        assert!(frontmatter_type("---\ntype: 'concept'\n---\n").is_none(), "single-quoted");
        assert!(frontmatter_type("---\n  indented: x\n---\n").is_none(), "stray indent");
        assert!(frontmatter_type("---\nnote: x\n---\n").is_none(), "no type key");
    }
}
