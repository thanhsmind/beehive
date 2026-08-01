// bee reviews — native port of the reviews verb group (bee.mjs
// handleReviewsCreate/List/Show/Record/CandidateAdd/Candidates/Status +
// lib/reviews.mjs).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   reviews list          [--json]
//   reviews show          --id I [--json]
//   reviews create        --file F [--json]              (--stdin delegates)
//   reviews record        --id I --kind K --file F [--json]   (--stdin delegates)
//   reviews candidate add --feature F --head H --mode M [--baseline B]
//                         [--cells C] [--json]
//   reviews candidates    [--json]
//   reviews status        [--feature F] [--json]
// Within the accepted shapes, the legacy STDERR-routed refusals (DB3 —
// requireFlag misses, invalid ids/kinds/modes, frozen-field payloads,
// missing sessions/cells) are served natively byte-identical.
//
// PERMANENTLY DELEGATED SHAPES: any call carrying --stdin. The probe must
// decide native-vs-Node BEFORE consuming stdin (a delegated Node process
// re-reads the same pipe), so stdin-fed JSON can never be validated here
// first — Node owns those calls end to end.
//
// Additional delegation triggers (None before any output/write):
//   - --help anywhere, unknown flags, non-flag tokens
//   - corrupt JSON anywhere on the read path (session files, candidate rows,
//     cell files, scope/payload files — Node warns with the V8 message, or
//     V8's JSON grammar might accept what serde refused)
//   - session ids whose String() form leaves the ASCII slug charset the
//     ported localeCompare model is calibrated for
//   - candidates that are strings/arrays (JS spread would explode them into
//     index keys), or git args that are not strings (spawnSync TypeError)
//   - numbers outside the JS round-trip emission guard
//   - writeJsonAtomic/appendJsonl failures (nothing durable written)
//
// DIVERGENCE NOTES (documented, unreachable-different for real bee data):
//   - candidate ids come from the SHA-256/OS-entropy uuid generator, not
//     crypto.randomUUID (format-identical v4; random on both runtimes).
//   - the localeCompare(…, 'en', {numeric:true}) session/cell sort is the
//     probe-calibrated natural_cmp copied from verbs/cells.rs (ASCII slugs
//     exact; anything wider is delegated by the charset guard above).
//
// Provenance: bee.mjs readReviewsJsonInput/summarizeReview/
// candidateStatusLine/buildReviewsStatusSummary/renderReviewsStatusText/
// handleReviewsCreate/List/Show/Record/CandidateAdd/Candidates/Status/
// requireFlag/readFileText/splitList, lib/reviews.mjs (ID_PATTERN/
// SCOPE_ENTRY_TYPES/REVIEW_MODES/IMMUTABLE_FIELDS/RECORD_KINDS/
// DECISION_STATUSES/utcNow/reviewsDir/reviewFile/candidatesPath/
// assertValidId/readReviewStrict/readReview/listReviews/writeReview/
// normalizeScopeEntry/runPreflight/createReview/recordOnReview/addCandidate/
// listCandidates/CANDIDATE_STATUSES/sessionCoversCandidate/isSessionOpen/
// defaultRunGit/coveredByKey/commitsSinceKey/headCoveredBy/commitsSince/
// deriveCandidateStatus), lib/cells.mjs readCell/listCells (read slice),
// lib/fsutil.mjs readJson/readJsonl.

use crate::fsutil::{append_jsonl, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::verbs::feedback::{random_uuid_v4, read_jsonl, value_js_safe};
use crate::verbs::knowledge::{g_prelude, js_str_or_undefined, pre_json_scan, GCtx, GPre};
use crate::verbs::reservations::{
    js_numberify, js_strict_eq, js_trim, keys_known, now_iso, parse_flags, truthy, FlagV, Flags,
};
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const REVIEW_MODES: [&str; 6] = ["docs", "tiny", "small", "spike", "standard", "high-risk"];
const SCOPE_ENTRY_TYPES: [&str; 3] = ["cell", "feature", "commit"];
const RECORD_KINDS: [&str; 5] = ["manifest", "preflight", "finding", "uat", "decision"];
const DECISION_STATUSES: [&str; 3] = ["pending", "blocked", "approved"];
const IMMUTABLE_FIELDS: [&str; 4] = ["baseline", "head", "included", "excluded"];

/// Delegate marker (Node owns the shape).
struct Delegate;
type R<T> = Result<T, Delegate>;

/// A handler outcome: an emitted payload or a thrown-Error message.
enum Out2 {
    Emit(Value, String),
    Thrown(String),
}

fn finish(ctx: &GCtx, out: R<Out2>) -> Option<ExitCode> {
    match out {
        Ok(Out2::Emit(result, text)) => Some(ctx.emit(&result, &text, 0)),
        Ok(Out2::Thrown(msg)) => Some(ctx.fail(&msg)),
        Err(Delegate) => None,
    }
}

fn reviews_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("reviews")
}

fn review_file(root: &Path, id: &str) -> PathBuf {
    reviews_dir(root).join(format!("{id}.json"))
}

fn candidates_path(root: &Path) -> PathBuf {
    root.join(".bee").join("review-candidates.jsonl")
}

/// lib/reviews.mjs + lib/cells.mjs ID_PATTERN: /^[A-Za-z0-9][A-Za-z0-9._-]*$/.
fn id_pattern_ok(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// bee.mjs requireFlag: value must be present, not '', not bare-boolean true.
fn require_flag(flags: &Flags, name: &str) -> Result<String, String> {
    match flags.get(name) {
        Some(FlagV::S(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(format!("Missing required flag --{name}.")),
    }
}

/// bee.mjs splitList.
fn split_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(js_trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

// ─── localeCompare(…, 'en', {numeric:true}) — copied from verbs/cells.rs ───
// (natural_cmp there is module-private; this is the same probe-calibrated
// model, used only after the charset guard keeps ids inside its exact range.)

fn char_rank(c: char) -> u8 {
    if c.is_whitespace() {
        0
    } else if c.is_alphabetic() {
        3
    } else if c.is_ascii_digit() {
        2
    } else {
        1
    }
}

fn punct_key(c: char) -> (u8, u32) {
    match c {
        '_' => (0, 0),
        '-' => (1, 0),
        '.' => (2, 0),
        other => (3, other as u32),
    }
}

fn primary_cmp(a: &str, b: &str) -> Ordering {
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let (mut i, mut j) = (0usize, 0usize);
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
        let ord = char_rank(x).cmp(&char_rank(y)).then_with(|| {
            if char_rank(x) == 1 {
                punct_key(x).cmp(&punct_key(y))
            } else {
                x.to_lowercase().collect::<String>().cmp(&y.to_lowercase().collect::<String>())
            }
        });
        if ord != Ordering::Equal {
            return ord;
        }
        i += 1;
        j += 1;
    }
    (av.len() - i).cmp(&(bv.len() - j))
}

fn tertiary_case_cmp(a: &str, b: &str) -> Ordering {
    let ai = a.chars().filter(|c| !c.is_ascii_digit());
    let bi = b.chars().filter(|c| !c.is_ascii_digit());
    for (x, y) in ai.zip(bi) {
        let ord = x.is_uppercase().cmp(&y.is_uppercase());
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

fn natural_cmp(a: &str, b: &str) -> Ordering {
    primary_cmp(a, b).then_with(|| tertiary_case_cmp(a, b))
}

/// The natural_cmp model is exact only over the ASCII slug charset — a
/// String()-coerced id outside it delegates the whole command.
fn slug_sortable(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

// ─── session store reads ───────────────────────────────────────────────────

/// listReviews — fail-open per file (deterministic warn), sorted by id.
/// Corrupt JSON (V8-worded warn) => Delegate.
fn list_reviews(root: &Path) -> R<Vec<Value>> {
    let dir = reviews_dir(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(Vec::new()),
    };
    let mut sessions: Vec<Value> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { return Err(Delegate) };
        if !name.ends_with(".json") {
            continue;
        }
        let session = match read_json(&entry.path()) {
            ReadJson::Missing => Value::Null, // unreadable → readJson fallback null, silently
            ReadJson::Corrupt => return Err(Delegate), // Node's readJson warns with the V8 message
            ReadJson::Parsed(v) => js_numberify(&v).map_err(|_| Delegate)?,
        };
        if !truthy(&session) || !matches!(session, Value::Object(_)) {
            eprintln!("reviews: skipping corrupt session file {name} (list stays fail-open)");
            continue;
        }
        sessions.push(session);
    }
    // sort by String(id).localeCompare(String(id), 'en', {numeric: true}).
    let keys: Vec<String> = sessions
        .iter()
        .map(|s| js_str_or_undefined(s.get("id")))
        .collect();
    if keys.iter().any(|k| !slug_sortable(k)) {
        return Err(Delegate);
    }
    let mut idx: Vec<usize> = (0..sessions.len()).collect();
    idx.sort_by(|&a, &b| natural_cmp(&keys[a], &keys[b]));
    Ok(idx.into_iter().map(|i| sessions[i].clone()).collect())
}

/// readReview — fail-open single read: invalid id or missing => null.
fn read_review(root: &Path, id: &str) -> R<Value> {
    if !id_pattern_ok(id) {
        return Ok(Value::Null);
    }
    match read_json(&review_file(root, id)) {
        ReadJson::Missing => Ok(Value::Null),
        ReadJson::Corrupt => Err(Delegate),
        ReadJson::Parsed(v) => js_numberify(&v).map_err(|_| Delegate),
    }
}

/// readReviewStrict — the write-verb sibling. Ok(Err(msg)) carries the loud
/// refusal; the outer Err delegates (io/V8-uncertain shapes).
fn read_review_strict(root: &Path, id: &str) -> R<Result<Map<String, Value>, String>> {
    if !id_pattern_ok(id) {
        return Ok(Err(format!(
            "invalid review id \"{id}\" — use letters, digits, dot, dash, underscore (e.g. \"review-2026-07-12\")."
        )));
    }
    let file = review_file(root, id);
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Err(format!(
                "readReviewStrict: review session \"{id}\" not found at {}.",
                file.display()
            )));
        }
        Err(_) => return Err(Delegate), // message embeds Node's err.code — Node owns it
    };
    let text = String::from_utf8_lossy(&bytes);
    let parsed = match serde_json::from_str::<Value>(&text) {
        Ok(v) => js_numberify(&v).map_err(|_| Delegate)?,
        Err(_) => return Err(Delegate), // V8 might parse what serde refused — Node decides
    };
    match parsed {
        Value::Object(m) => Ok(Ok(m)),
        other => Ok(Err(format!(
            "readReviewStrict: \"{}\" exists but is not a JSON object (found {}).",
            file.display(),
            if other.is_array() { "an array" } else { typeof_word(&other) }
        ))),
    }
}

fn typeof_word(v: &Value) -> &'static str {
    match v {
        Value::Null => "object",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) | Value::Object(_) => "object",
    }
}

/// listCandidates — readJsonl (skip corrupt lines; V8-maybe lines delegate).
fn list_candidates(root: &Path) -> R<Vec<Value>> {
    let read = read_jsonl(&candidates_path(root));
    if read.needs_node {
        return Err(Delegate);
    }
    read.rows.iter().map(|r| js_numberify(r).map_err(|_| Delegate)).collect()
}

// ─── lib/cells.mjs read slice (readCell / listCells) — provenance: the
// already-proved port in verbs/cells.rs (module-private there) ─────────────

fn cells_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("cells")
}

fn read_cell_json(file: &Path) -> R<Option<Value>> {
    match read_json(file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(Delegate),
        ReadJson::Parsed(Value::Null) => Ok(None),
        ReadJson::Parsed(v) => Ok(Some(js_numberify(&v).map_err(|_| Delegate)?)),
    }
}

fn read_cell(root: &Path, id: &str) -> R<Option<Value>> {
    if id.is_empty() || !id_pattern_ok(id) {
        return Ok(None);
    }
    let active = cells_dir(root).join(format!("{id}.json"));
    if let Some(v) = read_cell_json(&active)? {
        return Ok(Some(v));
    }
    let archive_root = cells_dir(root).join("archive");
    let entries = match std::fs::read_dir(&archive_root) {
        Ok(rd) => rd,
        Err(_) => return Ok(None),
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let candidate = entry.path().join(format!("{id}.json"));
        if let Some(v) = read_cell_json(&candidate)? {
            return Ok(Some(v));
        }
    }
    Ok(None)
}

/// listCells(root) — active dir only (the handler passes no includeArchived).
fn list_cells(root: &Path) -> R<Vec<Value>> {
    let dir = cells_dir(root);
    let mut cells: Vec<Value> = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(cells),
    };
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.ends_with(".json") {
            continue;
        }
        let cell = match read_cell_json(&entry.path())? {
            None => continue,
            Some(v) => v,
        };
        match &cell {
            Value::Object(_) => {}
            Value::Array(_) => return Err(Delegate), // typeof [] === 'object' exotica
            _ => continue,
        }
        cells.push(cell);
    }
    let keys: Vec<String> = cells.iter().map(|c| js_str_or_undefined(c.get("id"))).collect();
    if keys.iter().any(|k| !slug_sortable(k)) {
        return Err(Delegate);
    }
    let mut idx: Vec<usize> = (0..cells.len()).collect();
    idx.sort_by(|&a, &b| natural_cmp(&keys[a], &keys[b]));
    Ok(idx.into_iter().map(|i| cells[i].clone()).collect())
}

// ─── JSON input files (readReviewsJsonInput, --file branch only) ───────────

/// Ok(Err(msg)) — the deterministic refusal; outer Err — delegate.
fn read_json_input(root_flags: &Flags, label: &str) -> R<Result<Value, String>> {
    let file = match require_flag(root_flags, "file") {
        Ok(f) => f,
        Err(msg) => return Ok(Err(msg)),
    };
    let text = match std::fs::read(&file) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(_) => return Ok(Err(format!("Cannot read {label} file: {file}"))),
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(v) => Ok(Ok(js_numberify(&v).map_err(|_| Delegate)?)),
        Err(_) => Err(Delegate), // V8's grammar might accept it — Node owns the answer
    }
}

// ─── derived coverage (deriveCandidateStatus + helpers) ────────────────────

/// undefined-aware strict equality: both-absent is true (undefined ===
/// undefined), absent vs anything else false, objects/arrays always false.
fn strict_eq_opt(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => js_strict_eq(x, y),
        _ => false,
    }
}

/// SameValueZero over parsed JSON (Set.has): primitives by value, composites
/// by identity — never equal across independent parses.
fn same_value_zero(a: &Value, b: &Value) -> bool {
    js_strict_eq(a, b)
}

fn session_covers_candidate(session: &Value, candidate: &Value) -> bool {
    let Some(Value::Array(included)) = session.get("included") else { return false };
    let cand_feature = candidate.get("feature");
    let feature_match = included.iter().any(|e| {
        truthy(e)
            && matches!(e.get("type"), Some(Value::String(t)) if t == "feature")
            && strict_eq_opt(e.get("id"), cand_feature)
    });
    if feature_match {
        return true;
    }
    let cells: Vec<&Value> = match candidate.get("cells") {
        Some(Value::Array(items)) => items.iter().filter(|v| truthy(v)).collect(),
        _ => Vec::new(),
    };
    if cells.is_empty() {
        return false;
    }
    let included_cell_ids: Vec<Option<&Value>> = included
        .iter()
        .filter(|e| truthy(e) && matches!(e.get("type"), Some(Value::String(t)) if t == "cell"))
        .map(|e| e.get("id"))
        .collect();
    cells.iter().all(|id| {
        included_cell_ids
            .iter()
            .any(|set_id| matches!(set_id, Some(v) if same_value_zero(id, v)))
    })
}

fn is_session_open(session: &Value) -> bool {
    match session.get("decision") {
        Some(d) if truthy(d) => !matches!(d.get("status"), Some(Value::String(s)) if s == "approved"),
        _ => true, // absent/falsy decision → open
    }
}

struct GitAnswer {
    covered: Option<bool>,
    unresolved: bool,
}

fn run_git(root: &Path, args: &[&str]) -> Option<std::process::Output> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
}

type GitMemo = HashMap<String, (Option<bool>, bool, Option<f64>)>; // (covered, unresolved, count)

fn head_covered_by(root: &Path, head: Option<&Value>, r#ref: Option<&Value>, memo: &mut GitMemo) -> R<GitAnswer> {
    if strict_eq_opt(head, r#ref) {
        return Ok(GitAnswer { covered: Some(true), unresolved: false });
    }
    // spawnSync requires string args — anything else is a V8 TypeError.
    let (Some(Value::String(head)), Some(Value::String(r#ref))) = (head, r#ref) else {
        return Err(Delegate);
    };
    let key = format!("covered {head} {ref}", r#ref = r#ref);
    if let Some((covered, unresolved, _)) = memo.get(&key) {
        return Ok(GitAnswer { covered: *covered, unresolved: *unresolved });
    }
    let out = run_git(root, &["merge-base", "--is-ancestor", head, r#ref]);
    let answer = match out.and_then(|o| o.status.code()) {
        Some(0) => GitAnswer { covered: Some(true), unresolved: false },
        Some(1) => GitAnswer { covered: Some(false), unresolved: false },
        _ => GitAnswer { covered: None, unresolved: true },
    };
    memo.insert(key, (answer.covered, answer.unresolved, None));
    Ok(answer)
}

fn commits_since(root: &Path, r#ref: &str, memo: &mut GitMemo) -> (Option<f64>, bool) {
    let key = format!("since {ref}", r#ref = r#ref);
    if let Some((_, unresolved, count)) = memo.get(&key) {
        return (*count, *unresolved);
    }
    let out = run_git(root, &["rev-list", &format!("{ref}..HEAD", r#ref = r#ref), "--count"]);
    let value = match out {
        Some(o) if o.status.code() == Some(0) => {
            let stdout = String::from_utf8_lossy(&o.stdout).into_owned();
            match parse_int_prefix(js_trim(&stdout)) {
                Some(n) => (Some(n), false),
                None => (None, true),
            }
        }
        _ => (None, true),
    };
    memo.insert(key, (None, value.1, value.0));
    value
}

/// Number.parseInt(s, 10) restricted to finite results.
fn parse_int_prefix(s: &str) -> Option<f64> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let mut sign = 1.0f64;
    if matches!(bytes.first(), Some(b'+')) {
        i = 1;
    } else if matches!(bytes.first(), Some(b'-')) {
        sign = -1.0;
        i = 1;
    }
    let start = i;
    let mut value = 0.0f64;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        value = value * 10.0 + (bytes[i] - b'0') as f64;
        i += 1;
    }
    if i == start {
        return None; // NaN
    }
    Some(sign * value)
}

struct Derived {
    status: &'static str,
    /// None = undefined (session.id key absent).
    session: Option<Value>,
    note: Option<&'static str>,
}

fn derive_candidate_status(
    root: &Path,
    candidate: &Value,
    sessions: &[Value],
    memo: &mut GitMemo,
) -> R<Derived> {
    let covering: Vec<&Value> = sessions
        .iter()
        .filter(|s| session_covers_candidate(s, candidate))
        .collect();

    let open: Vec<&&Value> = covering.iter().filter(|s| is_session_open(s)).collect();
    if let Some(session) = open.last() {
        return Ok(Derived {
            status: "in review",
            session: session.get("id").cloned(),
            note: None,
        });
    }

    let approved: Vec<&&Value> = covering.iter().filter(|s| !is_session_open(s)).collect();
    let mut unresolved_session: Option<&Value> = None;
    for session in approved {
        let coverage = head_covered_by(root, candidate.get("head"), session.get("head"), memo)?;
        if coverage.unresolved {
            if unresolved_session.is_none() {
                unresolved_session = Some(session);
            }
            continue;
        }
        if coverage.covered != Some(true) {
            continue;
        }
        // commitsSince(root, session.head) — a non-string head is a spawnSync
        // TypeError in Node (reachable when the === head fast-path matched).
        let Some(Value::String(session_head)) = session.get("head") else {
            return Err(Delegate);
        };
        let (count, unresolved) = commits_since(root, session_head, memo);
        if unresolved {
            return Ok(Derived {
                status: "review stale",
                session: session.get("id").cloned(),
                note: Some("range unresolvable"),
            });
        }
        if count.unwrap_or(0.0) > 0.0 {
            return Ok(Derived {
                status: "review stale",
                session: session.get("id").cloned(),
                note: None,
            });
        }
        return Ok(Derived {
            status: "reviewed",
            session: session.get("id").cloned(),
            note: None,
        });
    }
    if let Some(session) = unresolved_session {
        return Ok(Derived {
            status: "review stale",
            session: session.get("id").cloned(),
            note: Some("range unresolvable"),
        });
    }
    Ok(Derived { status: "unreviewed", session: None, note: None })
}

// ─── text renderers ────────────────────────────────────────────────────────

/// summarizeReview: `${id} [${decision && decision.status}] ${scope_description}`.
fn summarize_review(session: &Value) -> String {
    let decision_disp = match session.get("decision") {
        None => "undefined".to_string(),
        Some(d) if !truthy(d) => jsjson::js_to_string(d), // null/false/0/'' print themselves
        Some(d) => js_str_or_undefined(d.get("status")),
    };
    format!(
        "{} [{}] {}",
        js_str_or_undefined(session.get("id")),
        decision_disp,
        js_str_or_undefined(session.get("scope_description"))
    )
}

fn candidate_status_line(row: &Value, status: &str, session: Option<&Value>, note: Option<&Value>) -> String {
    let target = format!(
        "{}@{} ({})",
        js_str_or_undefined(row.get("feature")),
        js_str_or_undefined(row.get("head")),
        js_str_or_undefined(row.get("mode"))
    );
    let session_disp = match session {
        Some(v) => jsjson::js_to_string(v),
        None => "undefined".to_string(),
    };
    match status {
        "reviewed" => format!("{target} — reviewed (covered by {session_disp})"),
        "review stale" => {
            let note_part = match note {
                Some(n) if truthy(n) => format!(", {}", jsjson::js_to_string(n)),
                _ => String::new(),
            };
            format!("{target} — review stale (was covered by {session_disp}{note_part})")
        }
        "in review" => format!("{target} — in review (session {session_disp})"),
        _ => format!("{target} — unreviewed"),
    }
}

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "reviews" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    // reviews.candidate.add is a NESTED 3-segment name (du-1 longest-prefix).
    let (cmd, rest_from): (&'static str, usize) = match verb {
        "list" => ("reviews list", 2),
        "show" => ("reviews show", 2),
        "create" => ("reviews create", 2),
        "record" => ("reviews record", 2),
        "candidates" => ("reviews candidates", 2),
        "status" => ("reviews status", 2),
        "candidate" => {
            if args.get(2)?.to_str()? != "add" {
                return None; // unknown nested action → group-usage fallback stays Node's
            }
            ("reviews candidate add", 3)
        }
        _ => return None,
    };
    let toks: Vec<&str> = args[rest_from..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None;
    }
    let pre_json = pre_json_scan(&toks);
    let (flags, json) = parse_flags(&toks)?;

    // --stdin (any shape/value) delegates BEFORE stdin could be consumed.
    if flags.get("stdin").is_some() {
        return None;
    }

    let known: &[&str] = match cmd {
        "reviews list" | "reviews candidates" => &[],
        "reviews show" => &["id"],
        "reviews create" => &["file", "stdin"],
        "reviews record" => &["id", "kind", "file", "stdin"],
        "reviews candidate add" => &["feature", "head", "mode", "baseline", "cells"],
        "reviews status" => &["feature"],
        _ => unreachable!(),
    };
    if !keys_known(&flags, known) {
        return None;
    }

    let ctx = match g_prelude(cmd, json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let out: R<Out2> = match cmd {
        "reviews list" => run_list(&ctx),
        "reviews show" => run_show(&ctx, &flags),
        "reviews create" => run_create(&ctx, &flags),
        "reviews record" => run_record(&ctx, &flags),
        "reviews candidate add" => run_candidate_add(&ctx, &flags),
        "reviews candidates" => run_candidates(&ctx),
        "reviews status" => run_status(&ctx, &flags),
        _ => unreachable!(),
    };
    finish(&ctx, out)
}

fn run_list(ctx: &GCtx) -> R<Out2> {
    let sessions = list_reviews(&ctx.root)?;
    let result = Value::Array(sessions.clone());
    if !value_js_safe(&result) {
        return Err(Delegate);
    }
    let text = if sessions.is_empty() {
        "No review sessions.".to_string()
    } else {
        sessions.iter().map(summarize_review).collect::<Vec<_>>().join("\n")
    };
    Ok(Out2::Emit(result, text))
}

fn run_show(ctx: &GCtx, flags: &Flags) -> R<Out2> {
    let id = match require_flag(flags, "id") {
        Ok(id) => id,
        Err(msg) => return Ok(Out2::Thrown(msg)),
    };
    let session = read_review(&ctx.root, &id)?;
    if !truthy(&session) {
        return Ok(Out2::Thrown(format!("Review session \"{id}\" not found.")));
    }
    if !value_js_safe(&session) {
        return Err(Delegate);
    }
    let text = jsjson::stringify_pretty(&session);
    Ok(Out2::Emit(session, text))
}

fn run_create(ctx: &GCtx, flags: &Flags) -> R<Out2> {
    let scope = match read_json_input(flags, "scope")? {
        Ok(v) => v,
        Err(msg) => return Ok(Out2::Thrown(msg)),
    };
    // createReview(root, scope)
    let Value::Object(scope) = &scope else {
        return Ok(Out2::Thrown("create: scope input must be a JSON object.".to_string()));
    };
    for field in ["id", "requested_by", "scope_description", "baseline", "head"] {
        let ok = matches!(scope.get(field), Some(Value::String(s)) if !js_trim(s).is_empty());
        if !ok {
            return Ok(Out2::Thrown(format!(
                "create: scope is missing required field \"{field}\" (non-empty string)."
            )));
        }
    }
    let id = scope["id"].as_str().unwrap();
    if !id_pattern_ok(id) {
        return Ok(Out2::Thrown(format!(
            "invalid review id \"{id}\" — use letters, digits, dot, dash, underscore (e.g. \"review-2026-07-12\")."
        )));
    }
    let included_raw = match scope.get("included") {
        Some(Value::Array(items)) if !items.is_empty() => items.clone(),
        _ => {
            return Ok(Out2::Thrown("create: scope requires a non-empty \"included\" array.".to_string()))
        }
    };
    let excluded_raw = match scope.get("excluded") {
        None => Vec::new(),
        Some(Value::Array(items)) => items.clone(),
        Some(_) => {
            return Ok(Out2::Thrown("create: scope \"excluded\" must be an array when present.".to_string()))
        }
    };
    if review_file(&ctx.root, id).exists() {
        return Ok(Out2::Thrown(format!(
            "create: review session \"{id}\" already exists — review ids are never reused. FIX: pick a new id."
        )));
    }

    let normalize_entry = |raw: &Value| -> Result<Map<String, Value>, String> {
        let Value::Object(obj) = raw else {
            return Err(format!("create: scope entry must be an object, got {}.", jsjson::stringify(raw)));
        };
        let entry_type = match obj.get("type") {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        };
        if !SCOPE_ENTRY_TYPES.contains(&entry_type.as_str()) {
            return Err(format!(
                "create: scope entry has invalid type \"{}\" — must be one of {}.",
                js_str_or_undefined(obj.get("type")),
                SCOPE_ENTRY_TYPES.join(", ")
            ));
        }
        let entry_id = match obj.get("id") {
            Some(Value::String(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
            _ => return Err("create: scope entry is missing a non-empty \"id\".".to_string()),
        };
        let mut entry = Map::new();
        entry.insert("type".into(), Value::String(entry_type));
        entry.insert("id".into(), Value::String(entry_id));
        if let Some(Value::String(reason)) = obj.get("reason") {
            if !js_trim(reason).is_empty() {
                entry.insert("reason".into(), Value::String(js_trim(reason).to_string()));
            }
        }
        Ok(entry)
    };

    let mut included_entries: Vec<Map<String, Value>> = Vec::new();
    for raw in &included_raw {
        match normalize_entry(raw) {
            Ok(e) => included_entries.push(e),
            Err(msg) => return Ok(Out2::Thrown(msg)),
        }
    }
    let mut pre_excluded: Vec<Map<String, Value>> = Vec::new();
    for raw in &excluded_raw {
        match normalize_entry(raw) {
            Ok(mut e) => {
                if !e.contains_key("reason") {
                    e.insert("reason".into(), Value::String("excluded at request".to_string()));
                }
                pre_excluded.push(e);
            }
            Err(msg) => return Ok(Out2::Thrown(msg)),
        }
    }

    // runPreflight (A6 auto-exclusion; behavior_change caps counted).
    let mut still_included: Vec<Map<String, Value>> = Vec::new();
    let mut auto_excluded: Vec<Map<String, Value>> = Vec::new();
    let mut checked: Vec<Value> = Vec::new();
    for entry in included_entries {
        if entry.get("type").and_then(Value::as_str) != Some("cell") {
            still_included.push(entry);
            continue;
        }
        let entry_id = entry.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
        let cell = read_cell(&ctx.root, &entry_id)?;
        let Some(cell) = cell else {
            return Ok(Out2::Thrown(format!(
                "create: preflight cannot resolve included cell \"{entry_id}\" — no such cell. FIX: fix the scope input or drop the entry."
            )));
        };
        let status = cell.get("status").and_then(Value::as_str);
        if status == Some("open") || status == Some("claimed") {
            let mut excluded_entry = entry.clone();
            excluded_entry.insert("reason".into(), Value::String("in progress".to_string()));
            auto_excluded.push(excluded_entry);
            continue;
        }
        still_included.push(entry);
        let behavior_change = cell
            .get("trace")
            .filter(|t| truthy(t))
            .map(|t| matches!(t.get("behavior_change"), Some(Value::Bool(true))))
            .unwrap_or(false);
        if behavior_change {
            checked.push(Value::String(entry_id));
        }
    }

    let now = now_iso();
    let str_trim = |field: &str| Value::String(js_trim(scope[field].as_str().unwrap()).to_string());
    let mut session = Map::new();
    session.insert("id".into(), str_trim("id"));
    session.insert("requested_by".into(), str_trim("requested_by"));
    session.insert("requested_at".into(), Value::String(now.clone()));
    session.insert("scope_description".into(), str_trim("scope_description"));
    session.insert(
        "included".into(),
        Value::Array(still_included.into_iter().map(Value::Object).collect()),
    );
    session.insert(
        "excluded".into(),
        Value::Array(pre_excluded.into_iter().chain(auto_excluded).map(Value::Object).collect()),
    );
    session.insert("baseline".into(), str_trim("baseline"));
    session.insert("head".into(), str_trim("head"));
    session.insert("reviewer_manifest".into(), Value::Array(Vec::new()));
    let mut preflight = Map::new();
    preflight.insert("checked_at".into(), Value::String(now.clone()));
    preflight.insert("cells_checked".into(), Value::Array(checked));
    preflight.insert("passed".into(), Value::Bool(true));
    session.insert("verification_preflight".into(), Value::Object(preflight));
    session.insert("findings".into(), Value::Array(Vec::new()));
    session.insert("uat".into(), Value::Array(Vec::new()));
    let mut decision = Map::new();
    decision.insert("status".into(), Value::String("pending".to_string()));
    decision.insert("gate4".into(), Value::Null);
    session.insert("decision".into(), Value::Object(decision));
    session.insert("created_at".into(), Value::String(now.clone()));
    session.insert("updated_at".into(), Value::String(now));

    let session = Value::Object(session);
    if !value_js_safe(&session) {
        return Err(Delegate);
    }
    let session_id = session.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
    if write_json_atomic(&review_file(&ctx.root, &session_id), &session).is_err() {
        return Err(Delegate); // nothing durable written — Node owns the io error
    }
    let text = format!("Created review session {session_id}.");
    Ok(Out2::Emit(session, text))
}

fn run_record(ctx: &GCtx, flags: &Flags) -> R<Out2> {
    let id = match require_flag(flags, "id") {
        Ok(v) => v,
        Err(msg) => return Ok(Out2::Thrown(msg)),
    };
    let kind = match require_flag(flags, "kind") {
        Ok(v) => v,
        Err(msg) => return Ok(Out2::Thrown(msg)),
    };
    let payload = match read_json_input(flags, "payload")? {
        Ok(v) => v,
        Err(msg) => return Ok(Out2::Thrown(msg)),
    };
    // recordOnReview(root, id, {kind, payload})
    if !RECORD_KINDS.contains(&kind.as_str()) {
        return Ok(Out2::Thrown(format!(
            "record: invalid kind \"{kind}\" — must be one of {}.",
            RECORD_KINDS.join(", ")
        )));
    }
    if payload.is_null() {
        return Ok(Out2::Thrown("record: payload is required.".to_string()));
    }
    let Value::Object(payload_map) = &payload else {
        return Ok(Out2::Thrown(format!(
            "record: payload for kind \"{kind}\" must be a JSON object."
        )));
    };
    let forbidden: Vec<&str> = IMMUTABLE_FIELDS
        .iter()
        .copied()
        .filter(|f| payload_map.contains_key(*f))
        .collect();
    if !forbidden.is_empty() {
        return Ok(Out2::Thrown(format!(
            "record: refused — payload attempts to touch immutable scope field(s): {}. baseline/head/included/excluded are frozen at create (R5) and cannot change afterward.",
            forbidden.join(", ")
        )));
    }
    let mut session = match read_review_strict(&ctx.root, &id)? {
        Ok(s) => s,
        Err(msg) => return Ok(Out2::Thrown(msg)),
    };

    match kind.as_str() {
        "decision" => {
            let status_ok = matches!(payload_map.get("status"), Some(Value::String(s)) if DECISION_STATUSES.contains(&s.as_str()));
            if !status_ok {
                return Ok(Out2::Thrown(format!(
                    "record: decision.status must be one of {}, got \"{}\".",
                    DECISION_STATUSES.join(", "),
                    js_str_or_undefined(payload_map.get("status"))
                )));
            }
            session.insert("decision".into(), Value::Object(payload_map.clone()));
        }
        "manifest" => {
            session.insert("reviewer_manifest".into(), payload.clone());
        }
        "preflight" => {
            session.insert("verification_preflight".into(), payload.clone());
        }
        "finding" | "uat" => {
            let field = if kind == "finding" { "findings" } else { "uat" };
            let mut list = match session.get(field) {
                Some(Value::Array(items)) => items.clone(),
                _ => Vec::new(),
            };
            list.push(payload.clone());
            session.insert(field.into(), Value::Array(list));
        }
        _ => unreachable!(),
    }
    session.insert("updated_at".into(), Value::String(now_iso()));

    let session = Value::Object(session);
    if !value_js_safe(&session) {
        return Err(Delegate);
    }
    // writeReview writes at reviewFile(root, session.id) — the STORED id.
    let stored_id = match session.get("id") {
        Some(Value::String(s)) if id_pattern_ok(s) => s.clone(),
        _ => return Err(Delegate), // a non-slug stored id makes the target path Node's problem
    };
    if write_json_atomic(&review_file(&ctx.root, &stored_id), &session).is_err() {
        return Err(Delegate);
    }
    let text = format!(
        "Recorded {kind} on {} (updated_at {}).",
        js_str_or_undefined(session.get("id")),
        js_str_or_undefined(session.get("updated_at"))
    );
    Ok(Out2::Emit(session, text))
}

fn run_candidate_add(ctx: &GCtx, flags: &Flags) -> R<Out2> {
    let feature = match require_flag(flags, "feature") {
        Ok(v) => v,
        Err(msg) => return Ok(Out2::Thrown(msg)),
    };
    // GitHub #16: --cells omitted (or '') auto-fills from capped cells.
    let cells: Vec<Value> = match flags.get("cells") {
        Some(FlagV::S(s)) if !s.is_empty() => split_list(s).into_iter().map(Value::String).collect(),
        _ => list_cells(&ctx.root)?
            .iter()
            .filter(|c| {
                matches!(c.get("feature"), Some(Value::String(f)) if *f == feature)
                    && matches!(c.get("status"), Some(Value::String(st)) if st == "capped")
            })
            .map(|c| c.get("id").cloned().unwrap_or(Value::Null))
            .collect(),
    };
    let head = match require_flag(flags, "head") {
        Ok(v) => v,
        Err(msg) => return Ok(Out2::Thrown(msg)),
    };
    let mode = match require_flag(flags, "mode") {
        Ok(v) => v,
        Err(msg) => return Ok(Out2::Thrown(msg)),
    };
    let baseline = match flags.get("baseline") {
        Some(FlagV::S(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    };

    // addCandidate validation
    if js_trim(&feature).is_empty() {
        return Ok(Out2::Thrown("candidate add: feature is required.".to_string()));
    }
    if js_trim(&head).is_empty() {
        return Ok(Out2::Thrown("candidate add: head (commit sha) is required.".to_string()));
    }
    let mode_trim = js_trim(&mode);
    if mode_trim.is_empty() || !REVIEW_MODES.contains(&mode_trim) {
        return Ok(Out2::Thrown(format!(
            "candidate add: --mode is required and must be one of {} (the closing feature's lane).",
            REVIEW_MODES.join(", ")
        )));
    }

    let mut entry = Map::new();
    entry.insert("id".into(), Value::String(random_uuid_v4()));
    entry.insert("type".into(), Value::String("candidate".to_string()));
    entry.insert("date".into(), Value::String(now_iso()));
    entry.insert("feature".into(), Value::String(js_trim(&feature).to_string()));
    entry.insert("head".into(), Value::String(js_trim(&head).to_string()));
    entry.insert("mode".into(), Value::String(mode_trim.to_string()));
    entry.insert(
        "baseline".into(),
        match baseline {
            Some(b) if !js_trim(&b).is_empty() => Value::String(js_trim(&b).to_string()),
            _ => Value::Null,
        },
    );
    entry.insert(
        "cells".into(),
        Value::Array(
            cells
                .into_iter()
                .filter(|c| matches!(c, Value::String(s) if !js_trim(s).is_empty()))
                .collect(),
        ),
    );
    let entry = Value::Object(entry);
    if append_jsonl(&candidates_path(&ctx.root), &entry).is_err() {
        return Err(Delegate);
    }
    let cell_count = entry.get("cells").and_then(Value::as_array).map(Vec::len).unwrap_or(0);
    let text = format!(
        "Added candidate {} for feature \"{}\" (mode {}{}).",
        js_str_or_undefined(entry.get("id")),
        js_str_or_undefined(entry.get("feature")),
        js_str_or_undefined(entry.get("mode")),
        if cell_count > 0 { format!(", {cell_count} cell(s)") } else { String::new() }
    );
    Ok(Out2::Emit(entry, text))
}

fn run_candidates(ctx: &GCtx) -> R<Out2> {
    let entries = list_candidates(&ctx.root)?;
    let result = Value::Array(entries.clone());
    if !value_js_safe(&result) {
        return Err(Delegate);
    }
    let text = if entries.is_empty() {
        "No review candidates.".to_string()
    } else {
        entries
            .iter()
            .map(|e| {
                format!(
                    "{} {} @{} ({})",
                    js_str_or_undefined(e.get("date")),
                    js_str_or_undefined(e.get("feature")),
                    js_str_or_undefined(e.get("head")),
                    js_str_or_undefined(e.get("mode"))
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    Ok(Out2::Emit(result, text))
}

fn run_status(ctx: &GCtx, flags: &Flags) -> R<Out2> {
    let feature = match flags.get("feature") {
        Some(FlagV::S(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    };
    // buildReviewsStatusSummary
    let candidates: Vec<Value> = list_candidates(&ctx.root)?
        .into_iter()
        .filter(|c| match &feature {
            None => true,
            Some(f) => strict_eq_opt(c.get("feature"), Some(&Value::String(f.clone()))),
        })
        .collect();
    let sessions = list_reviews(&ctx.root)?;

    let mut counts: Vec<(&'static str, u64)> = vec![
        ("verified", candidates.len() as u64),
        ("unreviewed", 0),
        ("in review", 0),
        ("reviewed", 0),
        ("review stale", 0),
    ];
    let mut git_memo: GitMemo = HashMap::new();
    let mut rows: Vec<Value> = Vec::new();
    for candidate in &candidates {
        let derived = derive_candidate_status(&ctx.root, candidate, &sessions, &mut git_memo)?;
        for (label, n) in counts.iter_mut() {
            if *label == derived.status {
                *n += 1;
            }
        }
        // {...candidate, review_status, review_session, note} — spread only
        // works cleanly for objects; strings/arrays explode into index keys
        // (delegated), other primitives spread to {}.
        let mut row = match candidate {
            Value::Object(m) => m.clone(),
            Value::String(_) | Value::Array(_) => return Err(Delegate),
            _ => Map::new(),
        };
        row.insert("review_status".into(), Value::String(derived.status.to_string()));
        row.insert(
            "review_session".into(),
            match &derived.session {
                Some(v) if truthy(v) => v.clone(),
                _ => Value::Null, // `derived.session || null`
            },
        );
        row.insert(
            "note".into(),
            match derived.note {
                Some(n) => Value::String(n.to_string()), // `derived.note || null`
                None => Value::Null,
            },
        );
        rows.push(Value::Object(row));
    }

    let count_of = |label: &str| counts.iter().find(|(l, _)| *l == label).map(|(_, n)| *n).unwrap_or(0);
    let headline = format!(
        "verified: {}  unreviewed: {}  in review: {}  reviewed: {}  review stale: {}",
        count_of("verified"),
        count_of("unreviewed"),
        count_of("in review"),
        count_of("reviewed"),
        count_of("review stale")
    );
    let text = if rows.is_empty() {
        format!("{headline}\nNo review candidates.")
    } else {
        let mut lines = vec![headline];
        for row in &rows {
            let status = row.get("review_status").and_then(Value::as_str).unwrap_or("");
            lines.push(candidate_status_line(
                row,
                status,
                row.get("review_session"),
                row.get("note"),
            ));
        }
        lines.join("\n")
    };

    let mut counts_map = Map::new();
    for (label, n) in counts {
        counts_map.insert(label.to_string(), Value::from(n));
    }
    let mut summary = Map::new();
    summary.insert("counts".into(), Value::Object(counts_map));
    summary.insert("candidates".into(), Value::Array(rows));
    let summary = Value::Object(summary);
    if !value_js_safe(&summary) {
        return Err(Delegate);
    }
    Ok(Out2::Emit(summary, text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_session(root: &Path, id: &str, body: &Value) {
        let dir = reviews_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), jsjson::stringify(body)).unwrap();
    }

    #[test]
    fn list_reviews_sorts_naturally_and_skips_non_objects_with_warn() {
        let tmp = tempfile::tempdir().unwrap();
        write_session(tmp.path(), "rev-10", &json!({"id": "rev-10", "scope_description": "later"}));
        write_session(tmp.path(), "rev-2", &json!({"id": "rev-2", "scope_description": "earlier"}));
        write_session(tmp.path(), "junk", &json!(["not", "an", "object"]));
        let sessions = list_reviews(tmp.path()).ok().unwrap();
        let ids: Vec<&str> = sessions.iter().map(|s| s["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["rev-2", "rev-10"]); // numeric-aware order
        // Corrupt JSON delegates.
        std::fs::write(reviews_dir(tmp.path()).join("bad.json"), "{broken").unwrap();
        assert!(list_reviews(tmp.path()).is_err());
    }

    #[test]
    fn summarize_review_coerces_like_js_templates() {
        assert_eq!(
            summarize_review(&json!({"id": "r1", "decision": {"status": "approved"}, "scope_description": "d"})),
            "r1 [approved] d"
        );
        assert_eq!(summarize_review(&json!({"id": "r1"})), "r1 [undefined] undefined");
        assert_eq!(
            summarize_review(&json!({"id": "r1", "decision": null, "scope_description": "d"})),
            "r1 [null] d"
        );
        assert_eq!(
            summarize_review(&json!({"id": "r1", "decision": "odd", "scope_description": "d"})),
            "r1 [undefined] d"
        );
    }

    #[test]
    fn coverage_matching_is_feature_or_all_cells() {
        let by_feature = json!({"included": [{"type": "feature", "id": "f1"}]});
        let by_cells = json!({"included": [
            {"type": "cell", "id": "c1"}, {"type": "cell", "id": "c2"}
        ]});
        let cand = json!({"feature": "f1", "head": "h", "cells": ["c1", "c2"]});
        assert!(session_covers_candidate(&by_feature, &cand));
        assert!(session_covers_candidate(&by_cells, &cand));
        let partial = json!({"included": [{"type": "cell", "id": "c1"}]});
        assert!(!session_covers_candidate(&partial, &cand));
        // A cells-less candidate never cell-matches.
        let no_cells = json!({"feature": "f2", "cells": []});
        assert!(!session_covers_candidate(&by_cells, &no_cells));
        // Numeric ids stay distinct from string ids (SameValueZero).
        let n = json!({"included": [{"type": "cell", "id": 5}]});
        assert!(!session_covers_candidate(&n, &json!({"cells": ["5"]})));
        assert!(session_covers_candidate(&n, &json!({"cells": [5]})));
    }

    #[test]
    fn open_session_priority_and_status_lines() {
        assert!(is_session_open(&json!({})));
        assert!(is_session_open(&json!({"decision": {"status": "blocked"}})));
        assert!(!is_session_open(&json!({"decision": {"status": "approved"}})));
        let row = json!({"feature": "f", "head": "h", "mode": "standard"});
        assert_eq!(
            candidate_status_line(&row, "reviewed", Some(&json!("rev-1")), None),
            "f@h (standard) — reviewed (covered by rev-1)"
        );
        assert_eq!(
            candidate_status_line(&row, "review stale", Some(&json!("rev-1")), Some(&json!("range unresolvable"))),
            "f@h (standard) — review stale (was covered by rev-1, range unresolvable)"
        );
        assert_eq!(candidate_status_line(&row, "unreviewed", None, None), "f@h (standard) — unreviewed");
    }

    #[test]
    fn derive_in_review_wins_over_stale_approval() {
        let tmp = tempfile::tempdir().unwrap();
        let sessions = vec![
            json!({"id": "old", "included": [{"type": "feature", "id": "f1"}], "head": "x", "decision": {"status": "approved"}}),
            json!({"id": "live", "included": [{"type": "feature", "id": "f1"}], "head": "y", "decision": {"status": "pending"}}),
        ];
        let cand = json!({"feature": "f1", "head": "z", "cells": []});
        let mut memo = GitMemo::new();
        let d = derive_candidate_status(tmp.path(), &cand, &sessions, &mut memo).ok().unwrap();
        assert_eq!(d.status, "in review");
        assert_eq!(d.session, Some(json!("live")));
        // head === session.head fast path: covered without git; rev-list in a
        // non-repo tmp dir is unresolvable → 'review stale' + note.
        let sessions = vec![json!({"id": "s", "included": [{"type": "feature", "id": "f1"}], "head": "z", "decision": {"status": "approved"}})];
        let d = derive_candidate_status(tmp.path(), &cand, &sessions, &mut memo).ok().unwrap();
        assert_eq!(d.status, "review stale");
        assert_eq!(d.note, Some("range unresolvable"));
        // No covering session at all → unreviewed.
        let d = derive_candidate_status(tmp.path(), &cand, &[], &mut memo).ok().unwrap();
        assert_eq!(d.status, "unreviewed");
    }

    #[test]
    fn parse_int_prefix_matches_parseint() {
        assert_eq!(parse_int_prefix("42"), Some(42.0));
        assert_eq!(parse_int_prefix("3 apples"), Some(3.0));
        assert_eq!(parse_int_prefix("-7"), Some(-7.0));
        assert_eq!(parse_int_prefix(""), None);
        assert_eq!(parse_int_prefix("x1"), None);
    }

    #[test]
    fn split_list_trims_and_filters() {
        assert_eq!(split_list("a, b ,,c "), vec!["a", "b", "c"]);
        assert_eq!(split_list("  "), Vec::<String>::new());
    }
}
