// bee hook chain-nudge (SubagentStop). Advances the bee chain mechanically:
// when a registered bee worker stops (or the phase is swarming) it nudges the
// orchestrator; when the phase is reviewing it nudges reviewer synthesis.
// Otherwise silent. Fail-open: any miss or crash -> exit 0 (crash logged to
// .bee/logs/hooks.jsonl).
//
// A corrupt read never aborts the run — it warns once (queued until the run
// is committed to native, see below) and falls back to an absent-record
// reading: a corrupt state.json yields the default state, a corrupt
// session/lane record reads as "no record" (a corrupt lane also gets its own
// second, deterministic warn), and a corrupt cell is skipped, one warning per
// bad file.
//
// Two shapes still resolve to `Outcome::Delegate` rather than a native
// answer:
//   - readState's truthy-non-object `approved_gates` (a JS-spread-exotic
//     shape this hook does not model).
//   - resolveProductRoot's warn conditions (non-string / missing directory).

use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::hooks::adapter::{emit_hook_output, log_crash, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::state::hook_enabled;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const HOOK_NAME: &str = "chain-nudge";

/// The native path cannot serve this input; the run answers with
/// `Outcome::Delegate` instead (D6, docs/history/js-parity-cleanup/CONTEXT.md
/// — the live delegation signal itself is a separate backlog item).
///
/// pub(crate) (not the default private) so `scribing_debt`'s `Result` error
/// type is nameable from the debt-door-archive dda-2 parity test; `Debug` is
/// derived only so that test can `.unwrap()` the `Result`.
#[derive(Debug)]
pub(crate) struct Delegate;

// A run can still end in Delegate AFTER a corrupt file has been read (a bad
// product_root or an exotic approved_gates is only discovered later), and a
// delegating run must emit nothing. Warnings are therefore QUEUED here and
// released once the run is committed to native.
enum Warn {
    /// crate::fsutil::warn_corrupt_json for this file.
    CorruptJson(PathBuf),
    /// A ready-made line (readLane's own deterministic warn).
    Line(String),
}

thread_local! {
    static QUEUED_WARNINGS: std::cell::RefCell<Vec<Warn>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn queue_warning(w: Warn) {
    QUEUED_WARNINGS.with(|q| q.borrow_mut().push(w));
}

fn queue_corrupt_json(file: &Path) {
    queue_warning(Warn::CorruptJson(file.to_path_buf()));
}

fn flush_queued_warnings() {
    let queued = QUEUED_WARNINGS.with(|q| q.borrow_mut().drain(..).collect::<Vec<_>>());
    for w in queued {
        match w {
            Warn::CorruptJson(file) => warn_corrupt_json(&file),
            Warn::Line(line) => eprint!("{line}"),
        }
    }
}

fn clear_queued_warnings() {
    QUEUED_WARNINGS.with(|q| q.borrow_mut().clear());
}

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    clear_queued_warnings(); // one queue per run (tests reuse the thread)
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = ctx.root.clone() else {
        return Outcome::Done(ExitCode::SUCCESS);
    };
    // Vendored-lib presence gate: no installed harness under this root, so skip.
    if !crate::hooks::adapter::bee_installed(&root) {
        return Outcome::Done(ExitCode::SUCCESS);
    }
    match run_gated(&ctx, &root) {
        Ok(()) => {
            flush_queued_warnings();
            Outcome::Done(ExitCode::SUCCESS)
        }
        Err(Delegate) => Outcome::Delegate,
    }
}

fn run_gated(ctx: &HookContext, root: &Path) -> Result<(), Delegate> {
    if !hook_enabled(root, HOOK_NAME) {
        return Ok(());
    }

    let state = match read_state(root) {
        Ok(s) => s,
        Err(Delegate) => return Err(Delegate),
    };
    let session_id = get_session_id(&ctx.payload);

    // Resolve phase: session record → bound lane → default state, in that order.
    let phase_raw: Value = match resolve_pipeline_phase(root, session_id.as_deref()) {
        Ok(PipelinePhase::Record(v)) => v,
        Ok(PipelinePhase::FromState) => state.get("phase").cloned().unwrap_or(Value::Null),
        Err(PipelineErr::Delegate) => return Err(Delegate),
        Err(PipelineErr::Crash(msg)) => {
            // Fail-open: log the crash and exit 0 with no output.
            log_crash(Some(root), HOOK_NAME, &msg, ctx.source);
            return Ok(());
        }
    };
    // `(…phase…) || 'idle'` — JS truthiness.
    let phase = if truthy(&phase_raw) { phase_raw } else { json!("idle") };

    let agent_name = get_agent_name(&ctx.payload);
    let workers: Vec<Value> = match state.get("workers") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let is_registered_worker =
        !agent_name.is_empty() && workers.iter().any(|e| worker_name_matches(e, &agent_name));

    let mut msg: Option<String> = None;
    if phase == json!("reviewing") {
        msg = Some(
            "bee chain-nudge: a review agent finished. Collect its findings report \
and score severities independently (P1/P2/P3). When all reviewers are done, \
synthesize findings (corroboration promotes one level; disagreements go \
conservative) and present Gate 3."
                .to_string(),
        );
    } else if is_registered_worker || phase == json!("swarming") {
        let who = if !agent_name.is_empty() {
            format!("Worker \"{agent_name}\"")
        } else {
            "A bee worker".to_string()
        };
        let mut m = format!(
            "bee chain-nudge: {who} returned - collect its [STATUS] token \
([DONE]/[BLOCKED]/[HANDOFF]/[NOOP]), update the cell \
(bee cells), and check/release its reservations \
(bee reservations list --active-only). \
When the wave is clean, move to the next wave or the next chain step."
        );
        // Deferred-capture reminder: scribing_debt is compiled directly into
        // this binary, so the reminder always runs (nothing can silently
        // skip it).
        {
            let (count, cells) = scribing_debt(root)?;
            if count > 0 {
                m.push_str(&format!(
                    "\n\u{26a0} Capture pending: {count} behavior_change cell(s) uncaptured \
({}) \u{2014} run bee-capturing when you choose (batching features is fine).",
                    cells.join(", ")
                ));
            }
        }
        msg = Some(m);
    }
    // else: not a bee-managed subagent -> silent.

    if let Some(m) = msg {
        emit_hook_output(ctx, &m, "SubagentStop");
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
    Ok(())
}

// ── payload readers ─────────────────────────────────────────────────────

fn get_agent_name(payload: &Map<String, Value>) -> String {
    for key in ["agent_name", "agentName", "agent_nickname", "subagent_type", "agent_type"] {
        if let Some(Value::String(s)) = payload.get(key) {
            if !s.trim().is_empty() {
                return s.trim().to_string();
            }
        }
    }
    String::new()
}

fn get_session_id(payload: &Map<String, Value>) -> Option<String> {
    match payload.get("session_id") {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    }
}

/// Matches an entry to `agent`: the first JS-truthy of nickname|name|agent|worker
/// is compared strictly — a truthy non-string masks later keys and can never
/// strict-equal a string agent name.
fn worker_name_matches(entry: &Value, agent: &str) -> bool {
    match entry {
        Value::String(s) => s == agent,
        Value::Object(m) => {
            for key in ["nickname", "name", "agent", "worker"] {
                if let Some(v) = m.get(key) {
                    if truthy(v) {
                        return matches!(v, Value::String(s) if s == agent);
                    }
                }
            }
            false // all falsy -> "" — caller guarantees agent != ""
        }
        _ => false, // arrays/primitives carry no name field -> no match
    }
}

// ── JS semantics helpers ────────────────────────────────────────────────────

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// JS Date.parse for the shapes bee writes: RFC3339 timestamps
/// ("2026-07-30T03:35:50.326Z") and date-only "YYYY-MM-DD" (UTC midnight).
/// Anything else reads as None — no other date-parser forms appear in
/// bee-authored records.
fn date_parse_ms(v: &Value) -> Option<f64> {
    let Value::String(s) = v else { return None };
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp_millis() as f64);
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = d.and_hms_opt(0, 0, 0)?;
        return Some(dt.and_utc().timestamp_millis() as f64);
    }
    None
}

// ── state read / defaults / legacy-phase coercion ───────────────────────────

fn default_gates() -> Map<String, Value> {
    let mut m = Map::new();
    for name in ["context", "shape", "execution", "review"] {
        m.insert(name.into(), Value::Bool(false));
    }
    m
}

fn default_state() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("schema_version".into(), json!("1.0"));
    m.insert("phase".into(), json!("idle"));
    m.insert("feature".into(), Value::Null);
    m.insert("mode".into(), Value::Null);
    m.insert("approved_gates".into(), Value::Object(default_gates()));
    m.insert("workers".into(), json!([]));
    m.insert("summary".into(), json!(""));
    m.insert(
        "next_action".into(),
        json!("No active bee work \u{2014} awaiting a user request."),
    );
    m
}

/// Reads state.json merged over the defaults: approved_gates re-merged,
/// legacy 'validating' phase coerced to 'planning'. A corrupt file warns and
/// reads as absent, exactly the Missing arm; a truthy non-object
/// approved_gates is a JS-spread-exotic shape this hook does not model, so it
/// returns Delegate.
fn read_state(root: &Path) -> Result<Map<String, Value>, Delegate> {
    let state_file = root.join(".bee").join("state.json");
    let file_state = match read_json(&state_file) {
        ReadJson::Missing => return Ok(default_state()),
        ReadJson::Corrupt => {
            queue_corrupt_json(&state_file);
            return Ok(default_state());
        }
        ReadJson::Parsed(Value::Object(m)) => m,
        ReadJson::Parsed(_) => return Ok(default_state()),
    };
    let mut merged = default_state();
    for (k, v) in &file_state {
        merged.insert(k.clone(), v.clone());
    }
    let gates = match file_state.get("approved_gates") {
        None => default_gates(),
        Some(v) if !truthy(v) => default_gates(),
        Some(Value::Object(overlay)) => {
            let mut g = default_gates();
            for (k, v) in overlay {
                g.insert(k.clone(), v.clone());
            }
            g
        }
        // Spreading a non-empty string/array adds index keys in JS; empty
        // array / true / non-zero number spread to nothing.
        Some(Value::Array(a)) if a.is_empty() => default_gates(),
        Some(Value::Bool(_)) | Some(Value::Number(_)) => default_gates(),
        Some(_) => return Err(Delegate),
    };
    merged.insert("approved_gates".into(), Value::Object(gates));
    if merged.get("phase") == Some(&json!("validating")) {
        merged.insert("phase".into(), json!("planning")); // D13 legacy coercion
    }
    Ok(merged)
}

// ── root resolution / control-root lookup ───────────────────────────────────

enum RootsCore {
    None,
    Ordinary { work_root: PathBuf },
    LinkedValid { work_root: PathBuf, main_root: PathBuf },
}

fn js_absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

/// Trim, strip a leading "gitdir:", backslash-normalize, resolve against
/// `base`.
fn read_gitdir_file(file: &Path, base: &Path) -> Option<PathBuf> {
    let raw = std::fs::read_to_string(file).ok()?;
    let mut raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Some(rest) = raw.strip_prefix("gitdir:") {
        raw = rest.trim();
    }
    let sep_fixed = if cfg!(windows) { raw.to_string() } else { raw.replace('\\', "/") };
    Some(js_absolute(&base.join(sep_fixed)))
}

/// Onboarding-marker preference, git-root walk, linked-worktree bidirectional
/// validation. Err(reason) carries the WorktreeLinkInvalidError message,
/// formatted as `{reason} ({marker})`.
fn resolve_roots_core(start: &Path) -> Result<RootsCore, String> {
    let mut nearest = js_absolute(start);
    loop {
        if nearest.join(".bee").join("onboarding.json").exists() && !nearest.join(".git").exists() {
            return Ok(RootsCore::Ordinary { work_root: nearest });
        }
        match nearest.parent() {
            Some(p) => nearest = p.to_path_buf(),
            None => break,
        }
    }
    // locateGitRoot walk-up.
    let mut dir = js_absolute(start);
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
        let mut dir = js_absolute(start);
        loop {
            if dir.join(".bee").join("onboarding.json").exists() {
                return Ok(RootsCore::Ordinary { work_root: dir });
            }
            match dir.parent() {
                Some(p) => dir = p.to_path_buf(),
                None => return Ok(RootsCore::None),
            }
        }
    };
    let marker = work_root.join(".git");
    let marker_str = marker.display().to_string();
    let stat = std::fs::metadata(&marker)
        .map_err(|e| format!("Error: {e}, stat '{marker_str}'"))?; // statSync throw (race only)
    if !stat.is_file() {
        return Ok(RootsCore::Ordinary { work_root });
    }
    let invalid = |reason: &str| Err(format!("{reason} ({marker_str})"));
    let Some(gitdir) = read_gitdir_file(&marker, &work_root) else {
        return invalid("linked worktree gitdir is missing or malformed");
    };
    let worktrees_root = gitdir.parent().map(Path::to_path_buf).unwrap_or_default();
    let common_git_dir = worktrees_root.parent().map(Path::to_path_buf).unwrap_or_default();
    if !(common_git_dir.file_name().is_some_and(|n| n == ".git")
        && worktrees_root.file_name().is_some_and(|n| n == "worktrees"))
    {
        return invalid("linked worktree gitdir is outside the expected .git/worktrees namespace");
    }
    let id = gitdir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if id.is_empty() || id == "." || id == ".." {
        return invalid("linked worktree id is empty");
    }
    let reverse = read_gitdir_file(&gitdir.join("gitdir"), &gitdir);
    if reverse.as_deref() != Some(js_absolute(&marker).as_path()) {
        return invalid("linked worktree reverse gitdir pointer is missing or mismatched");
    }
    let Some(main_root) = common_git_dir.parent().map(Path::to_path_buf) else {
        return invalid("linked worktree gitdir is outside the expected .git/worktrees namespace");
    };
    Ok(RootsCore::LinkedValid { work_root, main_root })
}

enum CtrlErr {
    Delegate,
    /// A crash that gets logged, then exits 0 (WorktreeLinkInvalidError).
    Crash(String),
}

/// resolveProductRoot's warn conditions, as a delegate gate: unset/null/""
/// and existing-dir strings are silent (native path proceeds); a non-string
/// value or a missing directory would warn — both are rare, so they delegate
/// rather than reproducing the warn text.
fn check_product_root(work_root: &Path) -> Result<(), CtrlErr> {
    let config = crate::state::read_config_raw(work_root);
    match config.get("product_root") {
        None | Some(Value::Null) => Ok(()),
        Some(Value::String(s)) if s.is_empty() => Ok(()),
        Some(Value::String(s)) => {
            let resolved = work_root.join(s);
            match std::fs::metadata(&resolved) {
                Ok(m) if m.is_dir() => Ok(()),
                _ => Err(CtrlErr::Delegate),
            }
        }
        Some(_) => Err(CtrlErr::Delegate),
    }
}

/// The control root for `root`, falling back to `root` itself when there is
/// none. Only the returned value and the possible product-root warn
/// (delegate-gated above) are observable.
fn control_root_for(root: &Path) -> Result<PathBuf, CtrlErr> {
    match resolve_roots_core(root) {
        Err(msg) => Err(CtrlErr::Crash(format!("WorktreeLinkInvalidError: {msg}"))),
        Ok(RootsCore::None) => Ok(root.to_path_buf()),
        Ok(RootsCore::Ordinary { work_root }) => {
            check_product_root(&work_root)?;
            Ok(work_root)
        }
        Ok(RootsCore::LinkedValid { work_root, main_root }) => {
            check_product_root(&work_root)?;
            Ok(main_root)
        }
    }
}

// ── session read (fail-open display read) ───────────────────────────────────

fn well_formed_id(id: &str) -> bool {
    !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

fn read_session(control_root: &Path, session_id: &str) -> Option<Map<String, Value>> {
    if !well_formed_id(session_id) {
        return None; // a malformed id reads as "no session"
    }
    let file = control_root.join(".bee").join("sessions").join(format!("{session_id}.json"));
    let session = match read_json(&file) {
        ReadJson::Missing => return None,
        // readJson's null fallback: warn, then the record reads as absent.
        ReadJson::Corrupt => {
            queue_corrupt_json(&file);
            return None;
        }
        ReadJson::Parsed(Value::Object(m)) => m,
        ReadJson::Parsed(_) => return None,
    };
    if session.get("id") != Some(&Value::String(session_id.to_string())) {
        return None;
    }
    Some(session)
}

// ── lane read (phase + last_scribing_run only) ──────────────────────────────

/// Relative path for the only shape this hook needs (file under root); falls
/// back to the absolute path otherwise.
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

fn warn_corrupt_lane(root: &Path, file: &Path) {
    let rel = path_relative(root, file);
    queue_warning(Warn::Line(format!(
        "readLane: skipping corrupt lane record \"{rel}\" for display \u{2014} mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\").\n"
    )));
}

/// Reads the lane record for a plain string feature whose lane file exists.
/// Returns the merged lane record (defaults + parsed, phase coerced), or None
/// on the corrupt-shape path (deterministic warn). Corrupt JSON gets BOTH
/// warns: the generic corrupt-JSON line, then the lane-specific corrupt-lane
/// line.
fn read_lane_record(
    root: &Path,
    feature: &str,
    file: &Path,
) -> Option<Map<String, Value>> {
    let parsed = match read_json(file) {
        ReadJson::Missing => return None, // race: vanished between exists() and read
        ReadJson::Corrupt => {
            queue_corrupt_json(file);
            warn_corrupt_lane(root, file);
            return None;
        }
        ReadJson::Parsed(v) => v,
    };
    let Value::Object(parsed) = parsed else {
        warn_corrupt_lane(root, file);
        return None;
    };
    if parsed.get("feature") != Some(&Value::String(feature.to_string())) {
        warn_corrupt_lane(root, file);
        return None;
    }
    // defaultLaneRecord + parsed spread; only phase (coerced) is consumed by
    // this hook, but the merge is kept whole for fidelity.
    let mut merged = Map::new();
    merged.insert("schema_version".into(), json!("1.0"));
    merged.insert("feature".into(), Value::String(feature.to_string()));
    merged.insert("mode".into(), Value::Null);
    merged.insert("phase".into(), json!("idle"));
    merged.insert("approved_gates".into(), Value::Object(default_gates()));
    merged.insert("summary".into(), json!(""));
    merged.insert("next_action".into(), json!(""));
    merged.insert("created_at".into(), Value::Null);
    for (k, v) in &parsed {
        merged.insert(k.clone(), v.clone());
    }
    if merged.get("phase") == Some(&json!("validating")) {
        merged.insert("phase".into(), json!("planning"));
    }
    Some(merged)
}

// ── pipeline-phase resolution (phase consumer view) ─────────────────────────

enum PipelinePhase {
    /// ok:true — the resolved record's phase.
    Record(Value),
    /// ok:false refusal — chain-nudge falls back to state.phase.
    FromState,
}

enum PipelineErr {
    Delegate,
    Crash(String),
}

fn resolve_pipeline_phase(
    root: &Path,
    session_id: Option<&str>,
) -> Result<PipelinePhase, PipelineErr> {
    let defaults = |root: &Path| -> Result<PipelinePhase, PipelineErr> {
        // Falls back to a fresh state read.
        let record = read_state(root).map_err(|_| PipelineErr::Delegate)?;
        Ok(PipelinePhase::Record(record.get("phase").cloned().unwrap_or(Value::Null)))
    };
    let Some(sid) = session_id else {
        return defaults(root);
    };
    let control_root = match control_root_for(root) {
        Ok(p) => p,
        Err(CtrlErr::Delegate) => return Err(PipelineErr::Delegate),
        Err(CtrlErr::Crash(msg)) => return Err(PipelineErr::Crash(msg)),
    };
    let Some(session) = read_session(&control_root, sid) else {
        return defaults(root);
    };
    let bound = match session.get("lane") {
        Some(Value::String(s)) => s.trim().to_string(),
        _ => String::new(),
    };
    if bound.is_empty() {
        return defaults(root);
    }
    if !well_formed_id(&bound) {
        return Ok(PipelinePhase::FromState); // LANE_INVALID typed refusal
    }
    let file = control_root.join(".bee").join("lanes").join(format!("{bound}.json"));
    if !file.exists() {
        return Ok(PipelinePhase::FromState); // LANE_MISSING typed refusal
    }
    match read_lane_record(&control_root, &bound, &file) {
        Some(record) => Ok(PipelinePhase::Record(record.get("phase").cloned().unwrap_or(Value::Null))),
        None => Ok(PipelinePhase::FromState), // LANE_CORRUPT typed refusal
    }
}

// ── scribing debt and its readers ───────────────────────────────────────────

/// Splits on \r?\n, skips blank and corrupt lines.
fn read_jsonl(file: &Path) -> Vec<Value> {
    let Ok(bytes) = std::fs::read(file) else { return Vec::new() };
    let text = String::from_utf8_lossy(&bytes);
    let mut out = Vec::new();
    for line in text.split('\n') {
        let trimmed = line.trim_end_matches('\r').trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            out.push(v);
        }
    }
    out
}

/// String(value) for a possibly-absent JSON value (undefined → "undefined").
fn js_string_or_undefined(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => crate::jsjson::js_to_string(v),
    }
}

/// Approximation of `String.localeCompare('en', {numeric: true})` for cell
/// ids: digit runs compare numerically; other characters by class
/// (punctuation < digits < letters), then case-insensitively, lowercase
/// before uppercase — matching ICU order for the ascii-slug ids bee uses.
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let (mut i, mut j) = (0usize, 0usize);
    let char_rank = |c: char| -> u8 {
        if c.is_whitespace() {
            0
        } else if c.is_alphabetic() {
            3
        } else if c.is_ascii_digit() {
            2
        } else {
            1
        }
    };
    while i < av.len() && j < bv.len() {
        let (x, y) = (av[i], bv[j]);
        if x.is_ascii_digit() && y.is_ascii_digit() {
            let si = i;
            while i < av.len() && av[i].is_ascii_digit() {
                i += 1;
            }
            let sj = j;
            while j < bv.len() && bv[j].is_ascii_digit() {
                j += 1;
            }
            let da: String = av[si..i].iter().collect();
            let db: String = bv[sj..j].iter().collect();
            let ta = da.trim_start_matches('0');
            let tb = db.trim_start_matches('0');
            let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
            if ord != Ordering::Equal {
                return ord;
            }
            continue;
        }
        let ord = char_rank(x)
            .cmp(&char_rank(y))
            .then_with(|| {
                x.to_lowercase().collect::<String>().cmp(&y.to_lowercase().collect::<String>())
            })
            .then_with(|| x.is_uppercase().cmp(&y.is_uppercase()));
        if ord != Ordering::Equal {
            return ord;
        }
        i += 1;
        j += 1;
    }
    (av.len() - i).cmp(&(bv.len() - j)).then_with(|| a.cmp(b))
}

/// Lists cells matching `feature`/`status` — active-dir scan only (this hook
/// never looks at the archive), sorted by numeric-aware id compare. A corrupt
/// cell warns and is skipped, once per bad file — the rest of the scan is
/// unaffected.
fn list_cells_filtered(
    root: &Path,
    feature: &Value,
    status: &str,
) -> Vec<Map<String, Value>> {
    let dir = root.join(".bee").join("cells");
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut cells: Vec<Map<String, Value>> = Vec::new();
    for entry in rd.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue; // the `archive` child (or any dir) is never a cell
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        let cell = match read_json(&entry.path()) {
            ReadJson::Missing => continue,
            ReadJson::Corrupt => {
                queue_corrupt_json(&entry.path());
                continue;
            }
            ReadJson::Parsed(Value::Object(m)) => m,
            ReadJson::Parsed(_) => continue, // `typeof cell === 'object'` fails only for primitives; null is falsy
        };
        // JS `if (feature && cell.feature !== feature) continue` — an absent
        // cell.feature (undefined) never strict-equals a truthy filter.
        if truthy(feature)
            && cell.get("feature").unwrap_or(&Value::Null) != feature
        {
            continue;
        }
        if !matches!(cell.get("status"), Some(Value::String(s)) if s == status) {
            continue;
        }
        cells.push(cell);
    }
    cells.sort_by(|a, b| {
        natural_cmp(
            &js_string_or_undefined(a.get("id")),
            &js_string_or_undefined(b.get("id")),
        )
    });
    cells
}

/// Archive-aware sibling of `list_cells_filtered`, for debt-door-archive
/// dda-2: `bee close` archives a feature's cells on a green close
/// (`.bee/cells/archive/<feature>/*.json`), so a debt counter that only
/// walks the live store the way `list_cells_filtered` does goes
/// structurally silent the moment its own feature closes. This reads the
/// live store (exactly as `list_cells_filtered` does) THEN every archived
/// `.json` directly under `.bee/cells/archive/<feature>/`, deduplicating by
/// id with the LIVE copy winning on a duplicate — the same live-copy-wins
/// pattern `verbs/drivers/guard.rs:218 list_cells_including_archive`
/// already uses for `bee close`'s door. `list_cells_filtered` itself is
/// untouched and stays active-only. Only `scribing_debt` below calls this
/// variant; `feature` here is always the already-truthy state feature (this
/// hook has no global/cross-feature sweep).
fn list_cells_filtered_including_archive(
    root: &Path,
    feature: &Value,
    status: &str,
) -> Vec<Map<String, Value>> {
    let mut cells = list_cells_filtered(root, feature, status);
    let mut seen_ids: std::collections::HashSet<String> =
        cells.iter().map(|c| js_string_or_undefined(c.get("id"))).collect();
    let feature_str = crate::jsjson::js_to_string(feature);
    let archive_dir = root.join(".bee").join("cells").join("archive").join(&feature_str);
    let Ok(rd) = std::fs::read_dir(&archive_dir) else { return cells };
    for entry in rd.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        let cell = match read_json(&entry.path()) {
            ReadJson::Missing => continue,
            ReadJson::Corrupt => {
                queue_corrupt_json(&entry.path());
                continue;
            }
            ReadJson::Parsed(Value::Object(m)) => m,
            ReadJson::Parsed(_) => continue, // `typeof cell === 'object'` fails only for primitives; null is falsy
        };
        if truthy(feature) && cell.get("feature").unwrap_or(&Value::Null) != feature {
            continue;
        }
        if !matches!(cell.get("status"), Some(Value::String(s)) if s == status) {
            continue;
        }
        let id = js_string_or_undefined(cell.get("id"));
        if !seen_ids.insert(id) {
            continue; // the live copy above already claimed this id
        }
        cells.push(cell);
    }
    cells.sort_by(|a, b| {
        natural_cmp(
            &js_string_or_undefined(a.get("id")),
            &js_string_or_undefined(b.get("id")),
        )
    });
    cells
}

/// Parses whichever of `run.at`/`run.date` is truthy.
fn scribing_run_stamp_ms(run: Option<&Value>) -> Option<f64> {
    let run = run?;
    if !truthy(run) {
        return None;
    }
    let Value::Object(m) = run else { return None }; // property access on primitives yields undefined
    let at = m.get("at").filter(|v| truthy(v));
    let candidate = match at {
        Some(v) => v.clone(),
        None => m.get("date").cloned().unwrap_or(Value::Null),
    };
    date_parse_ms(&candidate)
}

/// The most recent scribing stamp for `feature`: ledger max → lane stamp →
/// the state record's own-feature stamp.
fn best_scribing_stamp_ms(
    root: &Path,
    feature: &Value,
    state: &Map<String, Value>,
) -> Option<f64> {
    let ledger = read_jsonl(&root.join(".bee").join("logs").join("scribing-runs.jsonl"));
    let mut best: Option<f64> = None;
    let mut consider = |ms: Option<f64>| {
        if let Some(v) = ms {
            if best.is_none() || v > best.unwrap() {
                best = Some(v);
            }
        }
    };
    for entry in &ledger {
        // JS `if (!entry || entry.feature !== feature) continue` — a truthy
        // `feature` never strict-equals an absent (undefined) field.
        let Value::Object(e) = entry else { continue };
        if e.get("feature").unwrap_or(&Value::Null) != feature {
            continue;
        }
        consider(e.get("ts").and_then(date_parse_ms));
    }
    // readLane(root, feature): a non-string or malformed feature name reads as
    // "no lane" (lanePath's requireLaneFeature throw is caught by readLane).
    if let Value::String(f) = feature {
        let name = f.trim();
        if !name.is_empty() && well_formed_id(name) {
            let file = root.join(".bee").join("lanes").join(format!("{name}.json"));
            if file.exists() {
                if let Some(lane) = read_lane_record(root, name, &file) {
                    consider(scribing_run_stamp_ms(lane.get("last_scribing_run")));
                }
            }
        }
    }
    if let Some(run) = state.get("last_scribing_run") {
        if truthy(run) {
            let run_feature = match run {
                Value::Object(m) => m.get("feature"),
                _ => None,
            };
            if run_feature.is_some() && run_feature.unwrap() == feature {
                consider(scribing_run_stamp_ms(Some(run)));
            }
        }
    }
    best
}

/// Counts behavior_change cells capped for the active feature since the last
/// scribing run. Returns (count, ids). No reachable branch here panics —
/// corrupt JSON warns and reads as absent, and only read_state's
/// exotic-gates case still delegates.
///
/// pub(crate) (not the default private) only so the debt-door-archive dda-2
/// parity test can reach it alongside the other three counters.
///
/// trun-9 rework (D5): the deferred-capture nudge's OWN copy of the
/// scribing-debt scan — a third one, alongside `drivers/close.rs` (close's
/// door) and `hooks/session_preamble/store.rs` (the preamble line). The
/// semantic judge caught this one having the same gap as the preamble's:
/// completing a `scribe` deferred-queue record cleared close's door and
/// left this nudge still firing. It now reads the same queue fold
/// (`state_group::scribe_queue_cells`) and decides with the same OR rule
/// (`state_group::deferred_debt_cleared`) as the other two copies, so all
/// three answer identically for the same cell.
pub(crate) fn scribing_debt(root: &Path) -> Result<(usize, Vec<String>), Delegate> {
    let state = read_state(root)?;
    let feature = state.get("feature").cloned().unwrap_or(Value::Null);
    if !truthy(&feature) {
        return Ok((0, Vec::new()));
    }
    let threshold = best_scribing_stamp_ms(root, &feature, &state).unwrap_or(0.0);
    let feature_str = crate::jsjson::js_to_string(&feature);
    let completed_cells = crate::verbs::state_group::scribe_queue_cells(root, &feature_str).completed;
    // dda-2: archive-aware, so a feature that just closed (and got archived)
    // still shows its unpaid debt instead of going structurally silent.
    let cells = list_cells_filtered_including_archive(root, &feature, "capped");
    let mut ids = Vec::new();
    for cell in &cells {
        let trace = match cell.get("trace") {
            Some(v) if truthy(v) => v.clone(),
            _ => Value::Object(Map::new()),
        };
        let Value::Object(trace) = trace else { continue }; // primitive trace: property reads yield undefined
        if trace.get("behavior_change") != Some(&Value::Bool(true)) {
            continue;
        }
        let capped_at = trace.get("capped_at").and_then(date_parse_ms);
        let legacy_cleared = !matches!(capped_at, Some(ms) if ms > threshold);
        let id_str = match cell.get("id") {
            None | Some(Value::Null) => String::new(),
            Some(v) => crate::jsjson::js_to_string(v),
        };
        let queue_completed = !id_str.is_empty() && completed_cells.contains(&id_str);
        if crate::verbs::state_group::deferred_debt_cleared(legacy_cleared, queue_completed) {
            continue;
        }
        // Array.prototype.join semantics: null/undefined render empty.
        ids.push(id_str);
    }
    Ok((ids.len(), ids))
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_root() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee").join("cells")).unwrap();
        tmp
    }

    fn write_state_file(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("state.json"), content).unwrap();
    }

    #[test]
    fn natural_sort_matches_numeric_locale_compare() {
        let mut ids = vec!["es-10", "es-2", "es-1", "aa", "es-2b"];
        ids.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(ids, vec!["aa", "es-1", "es-2", "es-2b", "es-10"]);
    }

    #[test]
    fn worker_matching_follows_js_truthy_masking() {
        let agent = "alpha";
        assert!(worker_name_matches(&json!("alpha"), agent));
        assert!(worker_name_matches(&json!({"nickname": "alpha"}), agent));
        assert!(worker_name_matches(&json!({"name": "alpha"}), agent));
        // A truthy non-string nickname masks the later name key (JS ||).
        assert!(!worker_name_matches(&json!({"nickname": 5, "name": "alpha"}), agent));
        // Falsy nickname falls through to name.
        assert!(worker_name_matches(&json!({"nickname": "", "name": "alpha"}), agent));
        assert!(!worker_name_matches(&json!(["alpha"]), agent));
        assert!(!worker_name_matches(&json!(42), agent));
    }

    #[test]
    fn read_state_defaults_and_legacy_phase() {
        let tmp = setup_root();
        // Missing file -> defaults.
        let s = read_state(tmp.path()).ok().unwrap();
        assert_eq!(s.get("phase"), Some(&json!("idle")));
        // Legacy 'validating' coerces to 'planning'; workers pass through.
        write_state_file(tmp.path(), r#"{"phase":"validating","workers":[{"nickname":"w1"}]}"#);
        let s = read_state(tmp.path()).ok().unwrap();
        assert_eq!(s.get("phase"), Some(&json!("planning")));
        assert_eq!(s.get("workers"), Some(&json!([{"nickname":"w1"}])));
        // Corrupt state warns and reads as absent — same fallback as Missing.
        write_state_file(tmp.path(), "{broken");
        let s = read_state(tmp.path()).ok().unwrap();
        assert_eq!(s, default_state());
        // Exotic approved_gates still delegates (JS spread of a string).
        write_state_file(tmp.path(), r#"{"approved_gates":"ab"}"#);
        assert!(read_state(tmp.path()).is_err());
    }

    #[test]
    fn scribing_debt_filters_by_feature_stamp_and_flag() {
        let tmp = setup_root();
        write_state_file(
            tmp.path(),
            r#"{"phase":"swarming","feature":"f1","last_scribing_run":{"feature":"f1","at":"2026-01-01T00:00:00.000Z"}}"#,
        );
        let cells = tmp.path().join(".bee").join("cells");
        let cell = |id: &str, feature: &str, capped_at: &str, behavior: bool| {
            std::fs::write(
                cells.join(format!("{id}.json")),
                serde_json::to_string(&json!({
                    "id": id, "feature": feature, "status": "capped",
                    "trace": {"behavior_change": behavior, "capped_at": capped_at}
                }))
                .unwrap(),
            )
            .unwrap();
        };
        cell("f1-10", "f1", "2026-02-01T00:00:00.000Z", true); // after stamp — debt
        cell("f1-2", "f1", "2026-02-01T00:00:00.000Z", true); // after stamp — debt
        cell("f1-1", "f1", "2025-12-01T00:00:00.000Z", true); // before stamp — clear
        cell("f1-3", "f1", "2026-02-01T00:00:00.000Z", false); // not behavior_change
        cell("g-1", "other", "2026-02-01T00:00:00.000Z", true); // other feature
        let (count, ids) = scribing_debt(tmp.path()).ok().unwrap();
        assert_eq!(count, 2);
        assert_eq!(ids, vec!["f1-2", "f1-10"]); // numeric-aware sort order
    }

    /// trun-9 rework (D5): this nudge's own copy of the scan is the THIRD
    /// place the semantic judge found unreconciled with the deferred queue
    /// (alongside `drivers/close.rs`'s door and the preamble line) —
    /// completing a `scribe` record for a debtor cell must silence the
    /// nudge too, per-cell, not just clear close's door.
    #[test]
    fn scribing_debt_clears_per_cell_once_the_queue_record_completes() {
        let tmp = setup_root();
        write_state_file(tmp.path(), r#"{"phase":"swarming","feature":"f1"}"#);
        let cells = tmp.path().join(".bee").join("cells");
        let cell = |id: &str, capped_at: &str| {
            std::fs::write(
                cells.join(format!("{id}.json")),
                serde_json::to_string(&json!({
                    "id": id, "feature": "f1", "status": "capped",
                    "trace": {"behavior_change": true, "capped_at": capped_at}
                }))
                .unwrap(),
            )
            .unwrap();
        };
        cell("f1-1", "2026-02-01T00:00:00.000Z");
        cell("f1-2", "2026-02-01T00:00:00.000Z");
        let (count, ids) = scribing_debt(tmp.path()).ok().unwrap();
        assert_eq!(count, 2);
        assert_eq!(ids, vec!["f1-1", "f1-2"]);

        // A `scribe` record naming ONLY f1-1, completed.
        std::fs::write(
            tmp.path().join(".bee").join("deferred-queue.jsonl"),
            concat!(
                "{\"ts\":\"2026-02-02T00:00:00.000Z\",\"event\":\"add\",\"id\":\"q1\",\"kind\":\"scribe\",\"feature\":\"f1\",\"cells\":[\"f1-1\"],\"areas\":[],\"files\":[],\"reason\":\"r\"}\n",
                "{\"ts\":\"2026-02-02T00:00:01.000Z\",\"event\":\"complete\",\"id\":\"q1\"}\n",
            ),
        )
        .unwrap();
        let (count, ids) = scribing_debt(tmp.path()).ok().unwrap();
        assert_eq!(count, 1, "only f1-1's record completed");
        assert_eq!(ids, vec!["f1-2"]);
    }

    #[test]
    fn scribing_debt_empty_when_idle_and_skips_corrupt_cell() {
        let tmp = setup_root();
        write_state_file(tmp.path(), r#"{"phase":"idle","feature":null}"#);
        assert_eq!(scribing_debt(tmp.path()).ok().unwrap(), (0, vec![]));
        // Corrupt cell file: only reached when a feature is active. It warns
        // and is skipped; the readable cells still count.
        write_state_file(tmp.path(), r#"{"phase":"swarming","feature":"f1"}"#);
        let cells = tmp.path().join(".bee").join("cells");
        std::fs::write(cells.join("bad.json"), "{nope").unwrap();
        std::fs::write(
            cells.join("f1-1.json"),
            serde_json::to_string(&json!({
                "id":"f1-1","feature":"f1","status":"capped",
                "trace":{"behavior_change":true,"capped_at":"2026-02-01T00:00:00.000Z"}
            }))
            .unwrap(),
        )
        .unwrap();
        let (count, ids) = scribing_debt(tmp.path()).ok().unwrap();
        assert_eq!((count, ids), (1, vec!["f1-1".to_string()]));
    }

    #[test]
    fn corrupt_session_and_lane_read_as_absent() {
        let tmp = setup_root();
        let sessions = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        std::fs::write(sessions.join("s1.json"), "{nope").unwrap();
        assert!(read_session(tmp.path(), "s1").is_none());

        // A corrupt lane record reads as "no lane" (both warns emitted).
        let lanes = tmp.path().join(".bee").join("lanes");
        std::fs::create_dir_all(&lanes).unwrap();
        let lane_file = lanes.join("l1.json");
        std::fs::write(&lane_file, "{nope").unwrap();
        assert!(read_lane_record(tmp.path(), "l1", &lane_file).is_none());

        // The whole pipeline still resolves: a corrupt session record leaves
        // the hook on the default (state.json) phase, exit 0.
        write_state_file(tmp.path(), r#"{"phase":"reviewing"}"#);
        match resolve_pipeline_phase(tmp.path(), Some("s1")) {
            Ok(PipelinePhase::Record(v)) => assert_eq!(v, json!("reviewing")),
            _ => panic!("corrupt session must fall back to defaults(), not delegate"),
        }
    }

    #[test]
    fn corrupt_state_still_runs_the_hook_to_exit_zero() {
        let tmp = setup_root();
        // Vendored-lib presence gate + a corrupt state.json.
        std::fs::create_dir_all(tmp.path().join(".bee").join("bin").join("lib")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
        std::fs::write(
            tmp.path().join(".bee").join("bin").join("lib").join("state.mjs"),
            "// stub",
        )
        .unwrap();
        write_state_file(tmp.path(), "{broken");
        let payload = json!({
            "hook_event_name": "SubagentStop",
            "cwd": tmp.path().to_string_lossy(),
        })
        .to_string();
        // run() returns ExitCode::SUCCESS on every Ok arm — Done is exit 0.
        assert!(matches!(run(&[], &payload), Outcome::Done(_)));
    }

    #[test]
    fn pipeline_defaults_without_session_and_refusal_on_missing_lane() {
        let tmp = setup_root();
        write_state_file(tmp.path(), r#"{"phase":"reviewing"}"#);
        match resolve_pipeline_phase(tmp.path(), None) {
            Ok(PipelinePhase::Record(v)) => assert_eq!(v, json!("reviewing")),
            _ => panic!("expected default-record phase"),
        }
        // Bound session naming a missing lane -> typed refusal -> FromState.
        let sessions = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        std::fs::write(
            sessions.join("s1.json"),
            r#"{"id":"s1","lane":"ghost-lane"}"#,
        )
        .unwrap();
        match resolve_pipeline_phase(tmp.path(), Some("s1")) {
            Ok(PipelinePhase::FromState) => {}
            _ => panic!("expected LANE_MISSING refusal to fall back to state.phase"),
        }
    }
}
