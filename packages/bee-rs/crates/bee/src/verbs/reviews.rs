// bee reviews — native port of the reviews verb group (bee.mjs
// handleReviewsCreate/List/Show/Record/CandidateAdd/Candidates/Status +
// lib/reviews.mjs).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   reviews list          [--json]
//   reviews show          --id I [--json]
//   reviews create        (--file F | --stdin) [--json]
//   reviews record        --id I --kind K (--file F | --stdin) [--json]
//   reviews candidate add --feature F --head H --mode M [--baseline B]
//                         [--cells C] [--json]
//   reviews candidates    [--json]
//   reviews status        [--feature F] [--json]
// Within the accepted shapes, the legacy STDERR-routed refusals (DB3 —
// requireFlag misses, invalid ids/kinds/modes, frozen-field payloads,
// missing sessions/cells) are served natively byte-identical.
//
// CUTOVER (2026-08-01) — `--stdin` IS NOW NATIVE, and so is corrupt JSON.
// Both exclusions existed only to serve contract C2 (byte-identical output
// with a Node runtime that no longer exists):
//   - `--stdin` was permanently delegated because the probe had to choose
//     native-vs-Node BEFORE the pipe was consumed — a delegated Node child
//     would have re-read it and found EOF. With nowhere to delegate, that
//     constraint is void: read_json_input reads the pipe and validates it
//     here. `flags.stdin === true` stays STRICT (see its doc comment), and
//     the labelled refusal "<label>: input is not valid JSON." is unchanged.
//     `record --stdin` pre-scans the stored session id before consuming the
//     pipe (see run_record); `create --stdin` has no payload-independent
//     trigger left to pre-scan.
//   - corrupt JSON on the READ path now warns via fsutil::warn_corrupt_json
//     and takes the same readJson fallback (list skips the session with its
//     own "skipping corrupt session file" line; show reports "not found";
//     candidate rows are skipped like every other corrupt JSONL line), and
//     readReviewStrict raises its OWN loud corrupt refusal — the byte-exact
//     sentence Node's `catch` threw — instead of handing the command back.
//     A lone-surrogate escape is covered by whichever of those the site uses.
//   - the unreadable-file branches that interpolated a libuv err.code carry
//     the Rust io error in the same sentence.
//
// Delegation triggers that remain (None before any output/write):
//   - --help anywhere, unknown flags, non-flag tokens
//   - session ids whose String() form leaves the ASCII slug charset the
//     ported localeCompare model is calibrated for
//   - candidates that are strings/arrays (JS spread would explode them into
//     index keys), or git args that are not strings (spawnSync TypeError)
//   - a cell file whose JSON is an ARRAY (typeof [] === 'object' exotica)
//   - numbers outside the JS round-trip emission guard (integers > 2^53 read
//     verbatim off disk; anything through js_numberify is already f64-exact)
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

/// The review session's merge-approval field inside `decision`.
///
/// It was named `gate4` while bee had four gates. validation-diet D2 merged
/// shape and execution into Gate 2, which renumbered the review gate to 3 and
/// left `gate4` naming a gate that no longer exists. The field is now
/// `review` — named for what it approves rather than for a number that has
/// already moved once.
const DECISION_GATE_FIELD: &str = "review";
/// The pre-renumber spelling. Sessions created before the rename carry it, so
/// it is still read and still accepted on write (normalized to `review`).
const DECISION_GATE_FIELD_LEGACY: &str = "gate4";

/// Read the merge-approval field from a `decision` object, new name first.
/// Returns `None` only when neither spelling is present.
///
/// No production caller today — nothing in the CLI branches on the approval
/// payload; `is_session_open` and the candidates ledger both key off
/// `decision.status`. It exists so that the first reader to need it cannot
/// accidentally ship a `gate4`-blind lookup against a store that still holds
/// pre-rename sessions, and the back-compat test below is its exercise.
#[allow(dead_code)]
fn decision_gate<'a>(decision: &'a Map<String, Value>) -> Option<&'a Value> {
    decision.get(DECISION_GATE_FIELD).or_else(|| decision.get(DECISION_GATE_FIELD_LEGACY))
}

/// Fold a legacy `gate4` key into `review` so the store converges on one
/// spelling. A payload that already carries `review` wins outright — an
/// explicit new-name value is never overwritten by a stale legacy one, and the
/// legacy key never survives the write.
fn normalize_decision_gate_field(decision: &mut Map<String, Value>) {
    if let Some(legacy) = decision.remove(DECISION_GATE_FIELD_LEGACY) {
        decision.entry(DECISION_GATE_FIELD.to_string()).or_insert(legacy);
    }
}
const IMMUTABLE_FIELDS: [&str; 4] = ["baseline", "head", "included", "excluded"];

/// Delegate marker (a shape this port still refuses to answer).
#[derive(Debug)]
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
/// A corrupt session file warns, takes readJson's null fallback, and is then
/// skipped by the shape check with listReviews' OWN "skipping corrupt session
/// file" line — exactly Node's two-warning sequence, minus the V8 bytes.
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
            ReadJson::Corrupt => {
                crate::fsutil::warn_corrupt_json(&entry.path());
                Value::Null // readJson(file, null) — the skip below follows
            }
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

/// readReview — fail-open single read: invalid id, missing, or corrupt =>
/// null (readJson's fallback; corrupt warns first). `show` then reports the
/// same "not found" it reports for an absent session, exactly as Node did.
fn read_review(root: &Path, id: &str) -> R<Value> {
    if !id_pattern_ok(id) {
        return Ok(Value::Null);
    }
    let file = review_file(root, id);
    match read_json(&file) {
        ReadJson::Missing => Ok(Value::Null),
        ReadJson::Corrupt => {
            crate::fsutil::warn_corrupt_json(&file);
            Ok(Value::Null)
        }
        ReadJson::Parsed(v) => js_numberify(&v).map_err(|_| Delegate),
    }
}

/// readReviewStrict — the write-verb sibling. Ok(Err(msg)) carries the loud
/// refusal — including, since the cutover, the corrupt-JSON one.
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
        // Node interpolated err.code here; the Rust io error stands in its
        // place and the refusal is otherwise unchanged.
        Err(e) => {
            return Ok(Err(format!(
                "readReviewStrict: could not read \"{}\" ({e}).",
                file.display()
            )));
        }
    };
    let text = String::from_utf8_lossy(&bytes);
    let parsed = match serde_json::from_str::<Value>(&text) {
        Ok(v) => js_numberify(&v).map_err(|_| Delegate)?,
        // CUTOVER: this used to delegate because V8's JSON grammar might have
        // accepted what serde refused (a lone-surrogate escape). Nothing else
        // parses it now, so it takes readReviewStrict's OWN loud corrupt
        // refusal — the same one Node's `catch` threw, byte for byte.
        Err(_) => {
            return Ok(Err(format!(
                "readReviewStrict: \"{0}\" exists but is not valid JSON. The bee CLI refuses to mutate a present-but-corrupt review session — that could silently clobber real review state (findings, decision, scope). FIX: inspect/restore the file (e.g. \"git checkout -- {0}\"), then retry.",
                file.display()
            )));
        }
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

/// listCandidates — readJsonl: every corrupt line is skipped, including the
/// lone-surrogate lines that used to delegate the whole command. Fail-open
/// stays fail-open, and nothing new is printed (Node's readJsonl was silent).
fn list_candidates(root: &Path) -> R<Vec<Value>> {
    let read = read_jsonl(&candidates_path(root));
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
        // readJson(file, null) fail-open: warn, then the null fallback, which
        // every caller here already treats exactly like a missing file.
        ReadJson::Corrupt => {
            crate::fsutil::warn_corrupt_json(file);
            Ok(None)
        }
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

// ─── JSON input (readReviewsJsonInput — BOTH branches) ─────────────────────

/// bee.mjs readReviewsJsonInput (:5328):
///
/// ```js
/// const text = flags.stdin === true ? fs.readFileSync(0, 'utf8')
///                                   : readFileText(requireFlag(flags,'file'), label);
/// try { return JSON.parse(text); }
/// catch { throw new Error(`${label}: input is not valid JSON.`); }
/// ```
///
/// Two details are load-bearing and preserved exactly:
///   1. `flags.stdin === true` is STRICT. A bare `--stdin` parses to the
///      boolean `true` (FlagV::Present) and reads the pipe; `--stdin=x` is a
///      STRING, never `=== true`, so it falls through to the --file branch —
///      and `requireFlag` then raises its own "Missing required flag --file."
///      when no --file was given. `--stdin` together with `--file` reads
///      stdin and never looks at --file (the ternary short-circuits, so
///      requireFlag is not even evaluated).
///   2. the parse failure is the caller's LABELLED refusal — "scope: input is
///      not valid JSON." / "payload: input is not valid JSON." — not a
///      readJson-style fail-open.
///
/// CUTOVER (2026-08-01): `--stdin` was permanently delegated because the
/// native probe had to choose Node-vs-native BEFORE the pipe was consumed (a
/// delegated Node child would have read EOF). With no runtime to delegate to,
/// that constraint is gone: stdin is read and validated here.
///
/// Which side of readReviewsJsonInput's ternary this argv selects. Split out
/// so the STRICT `=== true` rule is testable without touching a real pipe.
#[derive(Debug, PartialEq)]
enum JsonInput {
    Stdin,
    File(String),
}

fn json_input_source(flags: &Flags) -> Result<JsonInput, String> {
    // `flags.stdin === true`: only a BARE --stdin is the boolean true.
    if matches!(flags.get("stdin"), Some(FlagV::Present)) {
        return Ok(JsonInput::Stdin);
    }
    // `--stdin=x` is a string, so the ternary takes the --file branch and
    // requireFlag speaks for a missing --file exactly as it always did.
    require_flag(flags, "file").map(JsonInput::File)
}

/// The `try { JSON.parse(text) } catch { throw \`${label}: …\` }` half.
fn parse_json_input_text(text: &str, label: &str) -> R<Result<Value, String>> {
    match serde_json::from_str::<Value>(text) {
        Ok(v) => Ok(Ok(js_numberify(&v).map_err(|_| Delegate)?)),
        // Includes the lone-surrogate escapes that used to delegate: with one
        // parser left, "serde refused it" IS "input is not valid JSON".
        Err(_) => Ok(Err(format!("{label}: input is not valid JSON."))),
    }
}

/// Ok(Err(msg)) — the deterministic refusal; outer Err — delegate.
fn read_json_input(root_flags: &Flags, label: &str) -> R<Result<Value, String>> {
    let text = match json_input_source(root_flags) {
        Err(msg) => return Ok(Err(msg)),
        Ok(JsonInput::Stdin) => match read_stdin_text() {
            Ok(t) => t,
            // readFileSync(0) throwing is an unhandled error in Node too; it
            // surfaces through the same emitError path this refusal takes.
            Err(msg) => return Ok(Err(msg)),
        },
        Ok(JsonInput::File(file)) => match std::fs::read(&file) {
            Ok(b) => String::from_utf8_lossy(&b).into_owned(),
            Err(_) => return Ok(Err(format!("Cannot read {label} file: {file}"))),
        },
    };
    parse_json_input_text(&text, label)
}

/// `fs.readFileSync(0, 'utf8')` — the whole pipe, lossy-decoded like Node's
/// utf8 read. Same shape as verbs/cells.rs read_stdin_text (module-private
/// there; copied rather than re-exported, per the one-file rule).
fn read_stdin_text() -> Result<String, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    match std::io::stdin().lock().read_to_end(&mut bytes) {
        Ok(_) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(e) => Err(format!("{e}")),
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

    // CUTOVER: `--stdin` no longer delegates. The blanket bail that used to
    // stand here existed for one reason — the probe had to decide before the
    // pipe was consumed — and `keys_known` below already confines the flag to
    // create/record, the only two verbs whose Node handler reads stdin at all.

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
    decision.insert(DECISION_GATE_FIELD.into(), Value::Null);
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
    // Pre-scan, BEFORE the pipe is consumed, the one remaining bail trigger
    // that does not depend on the payload: writeReview writes to the STORED
    // id, and an id outside the slug charset makes that target path
    // unprovable (the check below, after the write decision, would be too
    // late once stdin is gone). The Ok(Err(..)) refusal is deliberately
    // IGNORED here — Node raises a missing/corrupt session only AFTER the
    // payload is read, and that order is part of the contract.
    if matches!(flags.get("stdin"), Some(FlagV::Present)) {
        if let Ok(Ok(existing)) = read_review_strict(&ctx.root, &id) {
            match existing.get("id") {
                Some(Value::String(s)) if id_pattern_ok(s) => {}
                _ => return Err(Delegate),
            }
        }
    }
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
            let mut decision = payload_map.clone();
            normalize_decision_gate_field(&mut decision);
            session.insert("decision".into(), Value::Object(decision));
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
        // CUTOVER: corrupt JSON no longer delegates. readJson warns, returns
        // null, and the shape check skips it with listReviews' own line —
        // the listing stays fail-open and the good sessions still come back.
        std::fs::write(reviews_dir(tmp.path()).join("bad.json"), "{broken").unwrap();
        std::fs::write(reviews_dir(tmp.path()).join("sur.json"), r#"{"id":"\ud800"}"#).unwrap();
        let sessions = list_reviews(tmp.path()).expect("corrupt session must not delegate");
        let ids: Vec<&str> = sessions.iter().map(|s| s["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["rev-2", "rev-10"]);
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

    // ─── write-path fixtures (R5 test migration; oracle: packages/bee/tests/
    // test_reviews.mjs makeReviewRepo/reviewCell/baseScope) ────────────────

    /// The dispatch frame the write verbs take. Its non-root fields are
    /// private to `verbs::knowledge`, so the only way to mint one is
    /// `g_prelude` — which resolves the AMBIENT checkout (and refreshes its
    /// manifest-drift cache, exactly as any `bee` invocation does). The store
    /// root is then retargeted at the fixture, so nothing under test ever
    /// reads or writes the ambient repo: every run_* below uses `ctx.root`
    /// only.
    /// None when the host checkout cannot mint one — `g_prelude` answers
    /// NeedsNode inside a LINKED WORKTREE (a normal way to run this repo's
    /// suite) and Emitted when there is no `.bee` root at all.
    fn ctx_at(root: &Path) -> Option<GCtx> {
        match g_prelude("reviews create", false, false, Instant::now())? {
            GPre::Go(mut ctx) => {
                ctx.root = root.to_path_buf();
                Some(ctx)
            }
            GPre::Emitted(_) => None,
        }
    }

    /// Probe the capability (a mintable dispatch frame), never the platform,
    /// and name what is missing when it is absent.
    macro_rules! ctx_or_skip {
        ($root:expr, $name:literal) => {
            match ctx_at($root) {
                Some(c) => c,
                None => {
                    eprintln!(
                        "SKIP (env: no ordinary bee checkout around the test host — g_prelude \
                         answers NeedsNode in a linked worktree and no-root elsewhere; run the \
                         suite from the main checkout) {}",
                        $name
                    );
                    return;
                }
            }
        };
    }

    /// Real parseFlags over a real argv tail — never a hand-built Flags.
    fn flags_of(toks: &[&str]) -> Flags {
        parse_flags(toks).expect("parse_flags accepts the shape").0
    }

    fn thrown(out: R<Out2>) -> String {
        match out.ok().expect("native, not delegated") {
            Out2::Thrown(m) => m,
            Out2::Emit(v, t) => panic!("expected a refusal, got Emit({}, {t})", jsjson::stringify(&v)),
        }
    }

    fn emitted(out: R<Out2>) -> (Value, String) {
        match out.ok().expect("native, not delegated") {
            Out2::Emit(v, t) => (v, t),
            Out2::Thrown(m) => panic!("expected success, got refusal: {m}"),
        }
    }

    fn write_cell(root: &Path, id: &str, body: Value) {
        let dir = cells_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), jsjson::stringify(&body)).unwrap();
    }

    /// test_reviews.mjs seedCappedCellWithEvidence — a capped behavior_change
    /// cell, which runPreflight keeps included AND counts in cells_checked.
    fn capped_cell(root: &Path, id: &str) {
        write_cell(
            root,
            id,
            json!({"id": id, "feature": "demo", "status": "capped", "trace": {"behavior_change": true}}),
        );
    }

    /// Writes the scope/payload JSON the --file flag points at; returns the path.
    fn json_file(root: &Path, name: &str, body: &Value) -> String {
        let file = root.join(name);
        std::fs::write(&file, jsjson::stringify(body)).unwrap();
        file.to_string_lossy().into_owned()
    }

    /// test_reviews.mjs baseScope.
    fn base_scope() -> Value {
        json!({
            "id": "rev-1",
            "requested_by": "user",
            "scope_description": "review the demo feature",
            "included": [{"type": "cell", "id": "ok-1"}],
            "baseline": "sha-base",
            "head": "sha-head",
        })
    }

    fn create_with(ctx: &GCtx, root: &Path, scope: &Value) -> R<Out2> {
        let file = json_file(root, "scope.json", scope);
        run_create(ctx, &flags_of(&["--file", &file]))
    }

    fn record_with(ctx: &GCtx, root: &Path, id: &str, kind: &str, payload: &Value) -> R<Out2> {
        let file = json_file(root, "payload.json", payload);
        run_record(ctx, &flags_of(&["--id", id, "--kind", kind, "--file", &file]))
    }

    // ─── CUTOVER: readReviewsJsonInput, both branches ──────────────────────

    /// `--stdin` used to be permanently delegated. These pin the two halves
    /// of readReviewsJsonInput (bee.mjs:5328) without touching a real pipe —
    /// a unit test that read fd 0 would block on an interactive runner.
    #[test]
    fn stdin_is_selected_only_by_a_bare_flag_and_wins_over_file() {
        // A valid payload parses and is handed back as-is.
        let ok = parse_json_input_text(r#"{"id":"rev-1"}"#, "scope").unwrap();
        assert_eq!(ok.unwrap(), json!({"id": "rev-1"}));
        // An invalid payload is the LABELLED refusal, per label.
        assert_eq!(
            parse_json_input_text("{nope", "scope").unwrap().unwrap_err(),
            "scope: input is not valid JSON."
        );
        assert_eq!(
            parse_json_input_text("{nope", "payload").unwrap().unwrap_err(),
            "payload: input is not valid JSON."
        );
        // A lone-surrogate escape — the shape V8 accepted and this CLI cannot
        // — is that same refusal now, not a delegation.
        assert_eq!(
            parse_json_input_text(r#"{"a":"\ud800"}"#, "scope").unwrap().unwrap_err(),
            "scope: input is not valid JSON."
        );

        // --stdin alone: the pipe.
        assert_eq!(json_input_source(&flags_of(&["--stdin"])), Ok(JsonInput::Stdin));
        // --stdin WITH --file: the ternary short-circuits, so stdin wins and
        // requireFlag(flags,'file') is never even evaluated.
        assert_eq!(
            json_input_source(&flags_of(&["--stdin", "--file", "scope.json"])),
            Ok(JsonInput::Stdin)
        );
        // `--stdin=x` is a STRING, never `=== true` — it falls through to the
        // --file branch, which then raises its own required-flag refusal.
        assert_eq!(
            json_input_source(&flags_of(&["--stdin=yes"])),
            Err("Missing required flag --file.".to_string())
        );
        assert_eq!(
            json_input_source(&flags_of(&["--stdin=true", "--file", "s.json"])),
            Ok(JsonInput::File("s.json".to_string()))
        );
        // No --stdin at all: unchanged.
        assert_eq!(
            json_input_source(&flags_of(&["--file", "s.json"])),
            Ok(JsonInput::File("s.json".to_string()))
        );
        assert_eq!(
            json_input_source(&flags_of(&[])),
            Err("Missing required flag --file.".to_string())
        );
    }

    /// The router must let a --stdin call through to the handler now (it used
    /// to bail before g_prelude), and only for the two verbs that read it.
    #[test]
    fn stdin_is_accepted_by_create_and_record_and_rejected_elsewhere() {
        let known = |cmd: &str| -> &[&str] {
            match cmd {
                "reviews create" => &["file", "stdin"],
                "reviews record" => &["id", "kind", "file", "stdin"],
                "reviews show" => &["id"],
                _ => &[],
            }
        };
        assert!(keys_known(&flags_of(&["--stdin"]), known("reviews create")));
        assert!(keys_known(
            &flags_of(&["--id", "r", "--kind", "finding", "--stdin"]),
            known("reviews record")
        ));
        // list/show/candidates never took --stdin — still delegated there.
        assert!(!keys_known(&flags_of(&["--id", "r", "--stdin"]), known("reviews show")));
    }

    #[test]
    fn read_review_strict_refuses_corrupt_json_instead_of_delegating() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(reviews_dir(root)).unwrap();
        std::fs::write(review_file(root, "rev-1"), "{ broken").unwrap();
        let msg = read_review_strict(root, "rev-1")
            .expect("corrupt session must not delegate")
            .unwrap_err();
        assert!(msg.starts_with("readReviewStrict: "), "{msg}");
        assert!(msg.contains("exists but is not valid JSON."), "{msg}");
        assert!(msg.contains("refuses to mutate a present-but-corrupt review session"), "{msg}");
        // A lone-surrogate escape takes the identical refusal.
        std::fs::write(review_file(root, "rev-2"), r#"{"id":"rev-2","t":"\udfff"}"#).unwrap();
        let msg2 = read_review_strict(root, "rev-2")
            .expect("lone surrogate must not delegate")
            .unwrap_err();
        assert!(msg2.contains("exists but is not valid JSON."), "{msg2}");
        // …and the fail-open sibling reports the plain not-found instead.
        assert_eq!(read_review(root, "rev-1").expect("must not delegate"), Value::Null);
    }

    // ─── createReview write path ───────────────────────────────────────────

    /// Oracle: "createReview: session roundtrip carries every SPEC §8 field,
    /// and show/readReview round-trips it".
    #[test]
    fn create_writes_every_spec_8_field_and_read_review_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "create_writes_every_spec_8_field_and_read_review_round_trips");
        capped_cell(root, "ok-1");

        let (session, text) = emitted(create_with(&ctx, root, &base_scope()));
        for field in [
            "id",
            "requested_by",
            "requested_at",
            "scope_description",
            "included",
            "excluded",
            "baseline",
            "head",
            "reviewer_manifest",
            "verification_preflight",
            "findings",
            "uat",
            "decision",
            "created_at",
            "updated_at",
        ] {
            assert!(session.get(field).is_some(), "session is missing SPEC §8 field {field}");
        }
        assert_eq!(session["decision"], json!({"status": "pending", "review": null}));
        assert_eq!(session["included"], json!([{"type": "cell", "id": "ok-1"}]));
        assert_eq!(session["excluded"], json!([]));
        assert_eq!(session["reviewer_manifest"], json!([]));
        assert_eq!(session["findings"], json!([]));
        assert_eq!(session["uat"], json!([]));
        assert_eq!(session["baseline"], json!("sha-base"));
        assert_eq!(session["head"], json!("sha-head"));
        // A capped behavior_change cell is counted by the stored preflight.
        assert_eq!(session["verification_preflight"]["cells_checked"], json!(["ok-1"]));
        assert_eq!(session["verification_preflight"]["passed"], json!(true));
        // One utcNow() stamps every timestamp in a single create.
        let now = &session["created_at"];
        assert_eq!(&session["updated_at"], now);
        assert_eq!(&session["requested_at"], now);
        assert_eq!(&session["verification_preflight"]["checked_at"], now);
        assert_eq!(text, "Created review session rev-1.");

        // Written to .bee/reviews/<id>.json, and the read side round-trips it.
        let stored: Value =
            serde_json::from_str(&std::fs::read_to_string(review_file(root, "rev-1")).unwrap()).unwrap();
        assert_eq!(stored, session);
        assert_eq!(read_review(root, "rev-1").ok().unwrap(), session);
        let listed = list_reviews(root).ok().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["id"], json!("rev-1"));
    }

    /// Oracle: "createReview: A6 auto-excludes an open/claimed included cell
    /// with reason \"in progress\", never silently reviewed-in" (plus the
    /// pre-declared-exclusion row that follows it).
    #[test]
    fn create_auto_excludes_open_and_claimed_cells_and_keeps_pre_declared_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "create_auto_excludes_open_and_claimed_cells_and_keeps_pre_declared_ones");
        capped_cell(root, "ok-1");
        write_cell(root, "open-1", json!({"id": "open-1", "feature": "demo", "status": "open"}));
        write_cell(root, "claimed-1", json!({"id": "claimed-1", "feature": "demo", "status": "claimed"}));
        let open_before = std::fs::read(cells_dir(root).join("open-1.json")).unwrap();
        let claimed_before = std::fs::read(cells_dir(root).join("claimed-1.json")).unwrap();

        let scope = json!({
            "id": "rev-1",
            "requested_by": "user",
            "scope_description": "d",
            "included": [
                {"type": "cell", "id": "ok-1"},
                {"type": "cell", "id": "open-1"},
                {"type": "cell", "id": "claimed-1"},
            ],
            "excluded": [{"type": "cell", "id": "pre-1", "reason": "unrelated hotfix"}],
            "baseline": "b",
            "head": "h",
        });
        let (session, _) = emitted(create_with(&ctx, root, &scope));

        // The control that must happen: the capped cell stays included and
        // counted — so the exclusions below are not a blanket drop.
        assert_eq!(session["included"], json!([{"type": "cell", "id": "ok-1"}]));
        assert_eq!(session["verification_preflight"]["cells_checked"], json!(["ok-1"]));
        // The pre-declared exclusion keeps its verbatim reason and leads;
        // auto-exclusions are appended after it.
        assert_eq!(
            session["excluded"],
            json!([
                {"type": "cell", "id": "pre-1", "reason": "unrelated hotfix"},
                {"type": "cell", "id": "open-1", "reason": "in progress"},
                {"type": "cell", "id": "claimed-1", "reason": "in progress"},
            ])
        );
        // Excluding from review scope never touches the cells themselves.
        assert_eq!(std::fs::read(cells_dir(root).join("open-1.json")).unwrap(), open_before);
        assert_eq!(std::fs::read(cells_dir(root).join("claimed-1.json")).unwrap(), claimed_before);
    }

    /// Oracle: "bee.mjs reviews create exits non-zero and writes nothing when
    /// the preflight cannot resolve an included cell".
    #[test]
    fn create_refuses_an_unresolvable_included_cell_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "create_refuses_an_unresolvable_included_cell_and_writes_nothing");
        let msg = thrown(create_with(&ctx, root, &base_scope())); // ok-1 never seeded
        // Refusal wording is a pinned contract here: bee.mjs serves these
        // legacy stderr refusals byte-identical to Node (see file header).
        assert_eq!(
            msg,
            "create: preflight cannot resolve included cell \"ok-1\" — no such cell. FIX: fix the scope input or drop the entry."
        );
        assert!(!reviews_dir(root).exists(), "a refused create writes no session dir");
        // Control: the same scope with the cell present does write.
        capped_cell(root, "ok-1");
        emitted(create_with(&ctx, root, &base_scope()));
        assert!(review_file(root, "rev-1").exists());
    }

    /// Oracle: "createReview: refuses an already-existing session id … and
    /// leaves the file byte-unchanged (id non-reuse, §8)".
    #[test]
    fn create_refuses_a_duplicate_id_and_leaves_the_file_byte_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "create_refuses_a_duplicate_id_and_leaves_the_file_byte_unchanged");
        capped_cell(root, "ok-1");
        emitted(create_with(&ctx, root, &base_scope()));
        // Bytes, not a parsed value: a refused duplicate must not REWRITE the
        // file at all, so any serializer touch is itself the defect.
        let before = std::fs::read(review_file(root, "rev-1")).unwrap();

        let mut second = base_scope();
        second["scope_description"] = json!("a different description");
        let msg = thrown(create_with(&ctx, root, &second));
        assert_eq!(
            msg,
            "create: review session \"rev-1\" already exists — review ids are never reused. FIX: pick a new id."
        );
        assert_eq!(std::fs::read(review_file(root, "rev-1")).unwrap(), before);
        // Control: a fresh id under the same fixture does get written.
        let mut third = base_scope();
        third["id"] = json!("rev-2");
        emitted(create_with(&ctx, root, &third));
        assert!(review_file(root, "rev-2").exists());
    }

    /// Oracle: "createReview: rejects missing required scope fields and an
    /// empty \"included\" array before any write".
    #[test]
    fn create_rejects_missing_scope_fields_before_any_write() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "create_rejects_missing_scope_fields_before_any_write");
        capped_cell(root, "ok-1");

        for field in ["id", "requested_by", "scope_description", "baseline", "head"] {
            // Blank-but-present and absent take the same branch.
            for bad in [json!("   "), Value::Null] {
                let mut scope = base_scope();
                if bad.is_null() {
                    scope.as_object_mut().unwrap().remove(field);
                } else {
                    scope[field] = bad.clone();
                }
                assert_eq!(
                    thrown(create_with(&ctx, root, &scope)),
                    format!("create: scope is missing required field \"{field}\" (non-empty string).")
                );
            }
        }
        let mut empty_included = base_scope();
        empty_included["included"] = json!([]);
        assert_eq!(
            thrown(create_with(&ctx, root, &empty_included)),
            "create: scope requires a non-empty \"included\" array."
        );
        let mut bad_excluded = base_scope();
        bad_excluded["excluded"] = json!("nope");
        assert_eq!(
            thrown(create_with(&ctx, root, &bad_excluded)),
            "create: scope \"excluded\" must be an array when present."
        );
        let mut bad_id = base_scope();
        bad_id["id"] = json!("../escape");
        assert_eq!(
            thrown(create_with(&ctx, root, &bad_id)),
            "invalid review id \"../escape\" — use letters, digits, dot, dash, underscore (e.g. \"review-2026-07-12\")."
        );
        assert_eq!(
            thrown(create_with(&ctx, root, &json!(["not", "an", "object"]))),
            "create: scope input must be a JSON object."
        );
        assert!(!reviews_dir(root).exists(), "no session dir created by any rejected create");
        // Control: the untouched base scope does create one.
        emitted(create_with(&ctx, root, &base_scope()));
        assert!(review_file(root, "rev-1").exists());
    }

    // ─── recordOnReview ────────────────────────────────────────────────────

    fn seeded_session(root: &Path, ctx: &GCtx) -> Vec<u8> {
        capped_cell(root, "ok-1");
        emitted(create_with(ctx, root, &base_scope()));
        std::fs::read(review_file(root, "rev-1")).unwrap()
    }

    /// Oracle: "recordOnReview: refuses any payload touching baseline/head/
    /// included/excluded — exits via throw, file byte-unchanged (R5
    /// immutability)".
    #[test]
    fn record_refuses_immutable_scope_fields_and_leaves_the_file_byte_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "record_refuses_immutable_scope_fields_and_leaves_the_file_byte_unchanged");
        let before = seeded_session(root, &ctx);

        for field in IMMUTABLE_FIELDS {
            let payload = json!({ field: "whatever", "note": "n" });
            let msg = thrown(record_with(&ctx, root, "rev-1", "manifest", &payload));
            assert!(
                msg.starts_with(&format!(
                    "record: refused — payload attempts to touch immutable scope field(s): {field}."
                )),
                "unexpected refusal for {field}: {msg}"
            );
            assert_eq!(std::fs::read(review_file(root, "rev-1")).unwrap(), before);
        }
        // Several at once are listed in IMMUTABLE_FIELDS order, not payload order.
        let msg = thrown(record_with(
            &ctx,
            root,
            "rev-1",
            "finding",
            &json!({"head": "h", "excluded": [], "baseline": "b"}),
        ));
        assert!(
            msg.starts_with("record: refused — payload attempts to touch immutable scope field(s): baseline, head, excluded."),
            "{msg}"
        );
        assert_eq!(std::fs::read(review_file(root, "rev-1")).unwrap(), before);
        // Control: a payload with none of them DOES rewrite the file.
        emitted(record_with(&ctx, root, "rev-1", "manifest", &json!({"reviewers": ["a"]})));
        assert_ne!(std::fs::read(review_file(root, "rev-1")).unwrap(), before);
    }

    /// Oracle: "recordOnReview: manifest/preflight/decision SET the field;
    /// finding/uat APPEND one entry per call".
    #[test]
    fn record_sets_manifest_preflight_decision_and_appends_finding_and_uat() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "record_sets_manifest_preflight_decision_and_appends_finding_and_uat");
        seeded_session(root, &ctx);

        emitted(record_with(&ctx, root, "rev-1", "manifest", &json!({"reviewers": ["a", "b"]})));
        let (session, _) = emitted(record_with(&ctx, root, "rev-1", "manifest", &json!({"reviewers": ["c"]})));
        assert_eq!(session["reviewer_manifest"], json!({"reviewers": ["c"]}), "manifest SETs");

        let (session, _) = emitted(record_with(
            &ctx,
            root,
            "rev-1",
            "preflight",
            &json!({"checked_at": "t", "cells_checked": ["x"], "passed": false}),
        ));
        assert_eq!(session["verification_preflight"]["passed"], json!(false), "preflight SETs");

        emitted(record_with(&ctx, root, "rev-1", "finding", &json!({"severity": "P1"})));
        let (session, text) = emitted(record_with(&ctx, root, "rev-1", "finding", &json!({"severity": "P2"})));
        assert_eq!(
            session["findings"],
            json!([{"severity": "P1"}, {"severity": "P2"}]),
            "findings APPEND in call order"
        );
        assert_eq!(
            text,
            format!("Recorded finding on rev-1 (updated_at {}).", session["updated_at"].as_str().unwrap())
        );

        emitted(record_with(&ctx, root, "rev-1", "uat", &json!({"item": "login flow"})));
        let (session, _) = emitted(record_with(&ctx, root, "rev-1", "uat", &json!({"item": "logout"})));
        assert_eq!(session["uat"], json!([{"item": "login flow"}, {"item": "logout"}]));

        let (session, _) = emitted(record_with(
            &ctx,
            root,
            "rev-1",
            "decision",
            &json!({"status": "approved", "review": {"approved_by": "user"}}),
        ));
        assert_eq!(session["decision"], json!({"status": "approved", "review": {"approved_by": "user"}}));
        // The immutable half is still exactly what create froze.
        assert_eq!(session["baseline"], json!("sha-base"));
        assert_eq!(session["head"], json!("sha-head"));
        assert_eq!(session["included"], json!([{"type": "cell", "id": "ok-1"}]));
        assert_eq!(session["excluded"], json!([]));
        // …and the file on disk carries the same record.
        assert_eq!(read_review(root, "rev-1").ok().unwrap(), session);

        // A decision status outside DECISION_STATUSES is refused, and the
        // stored decision keeps the last accepted value.
        assert_eq!(
            thrown(record_with(&ctx, root, "rev-1", "decision", &json!({"status": "maybe"}))),
            "record: decision.status must be one of pending, blocked, approved, got \"maybe\"."
        );
        assert_eq!(read_review(root, "rev-1").ok().unwrap()["decision"]["status"], json!("approved"));
    }

    /// The merge-approval field was `gate4` before the review gate was
    /// renumbered from 4 to 3. Both halves of the back-compat contract are
    /// proven here: a decision payload written with the LEGACY key is still
    /// accepted and converges on the new `review` spelling (the legacy key
    /// never survives the write), and `decision_gate` reads either spelling
    /// off a session that has not been rewritten since the rename.
    #[test]
    fn legacy_gate4_decision_key_is_read_and_normalized_to_review() {
        // Read side: an untouched pre-rename session still yields its approval.
        let legacy = json!({"status": "approved", "gate4": {"approved_by": "user"}});
        assert_eq!(
            decision_gate(legacy.as_object().unwrap()),
            Some(&json!({"approved_by": "user"}))
        );
        // …and the new spelling reads through the same accessor.
        let current = json!({"status": "approved", "review": {"approved_by": "user"}});
        assert_eq!(
            decision_gate(current.as_object().unwrap()),
            Some(&json!({"approved_by": "user"}))
        );
        assert_eq!(decision_gate(json!({"status": "pending"}).as_object().unwrap()), None);

        // Write side: a legacy payload is folded onto the new key.
        let mut d = legacy.as_object().unwrap().clone();
        normalize_decision_gate_field(&mut d);
        assert_eq!(Value::Object(d), json!({"status": "approved", "review": {"approved_by": "user"}}));

        // An explicit new-name value wins over a stale legacy one in the same
        // payload, and the legacy key is dropped either way.
        let mut both = json!({"status": "approved", "review": "new", "gate4": "stale"})
            .as_object()
            .unwrap()
            .clone();
        normalize_decision_gate_field(&mut both);
        assert_eq!(Value::Object(both), json!({"status": "approved", "review": "new"}));
    }

    /// An unknown kind is refused BEFORE the session is read or written — the
    /// proof is that a nonexistent id still yields the kind refusal, and the
    /// same call with a legal kind reaches the not-found refusal instead.
    /// Oracle: recordOnReview's check order in lib/reviews.mjs.
    #[test]
    fn record_refuses_an_unknown_kind_before_the_session_is_touched() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "record_refuses_an_unknown_kind_before_the_session_is_touched");
        let before = seeded_session(root, &ctx);

        assert_eq!(
            thrown(record_with(&ctx, root, "no-such-review", "bogus", &json!({"a": 1}))),
            "record: invalid kind \"bogus\" — must be one of manifest, preflight, finding, uat, decision."
        );
        assert!(!review_file(root, "no-such-review").exists());
        assert_eq!(std::fs::read(review_file(root, "rev-1")).unwrap(), before);
        // Control: same id, legal kind → the read refusal, i.e. the kind gate
        // really did short-circuit ahead of the read above.
        let file = review_file(root, "no-such-review");
        assert_eq!(
            thrown(record_with(&ctx, root, "no-such-review", "manifest", &json!({"a": 1}))),
            format!("readReviewStrict: review session \"no-such-review\" not found at {}.", file.display())
        );
        assert!(!file.exists(), "a refused record never creates the session file");
        // An unknown kind also outranks a payload that is not an object.
        assert_eq!(
            thrown(record_with(&ctx, root, "rev-1", "bogus", &json!(["arr"]))),
            "record: invalid kind \"bogus\" — must be one of manifest, preflight, finding, uat, decision."
        );
        assert_eq!(
            thrown(record_with(&ctx, root, "rev-1", "manifest", &json!(["arr"]))),
            "record: payload for kind \"manifest\" must be a JSON object."
        );
        assert_eq!(std::fs::read(review_file(root, "rev-1")).unwrap(), before);
    }

    /// An id that is not a legal slug is refused by readReviewStrict's own
    /// gate, before any path is built. Oracle: readReviewStrict/assertValidId.
    #[test]
    fn record_refuses_an_unknown_or_malformed_session_id() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "record_refuses_an_unknown_or_malformed_session_id");
        seeded_session(root, &ctx);
        assert_eq!(
            thrown(record_with(&ctx, root, "../escape", "manifest", &json!({"a": 1}))),
            "invalid review id \"../escape\" — use letters, digits, dot, dash, underscore (e.g. \"review-2026-07-12\")."
        );
        // A session file that parses to a non-object refuses loudly rather
        // than being treated as absent (the strict read's whole point).
        write_session(root, "rev-arr", &json!(["not", "an", "object"]));
        let msg = thrown(record_with(&ctx, root, "rev-arr", "manifest", &json!({"a": 1})));
        assert!(msg.ends_with("exists but is not a JSON object (found an array)."), "{msg}");
        // Control: the well-formed session accepts the identical payload.
        emitted(record_with(&ctx, root, "rev-1", "manifest", &json!({"a": 1})));
    }

    // ─── reviews candidate add ─────────────────────────────────────────────

    /// Oracle: "bee.mjs reviews candidate add requires --mode and rejects an
    /// unrecognized mode, leaving the ledger untouched".
    #[test]
    fn candidate_add_requires_mode_and_rejects_an_unrecognized_one() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "candidate_add_requires_mode_and_rejects_an_unrecognized_one");
        let ledger = candidates_path(root);

        let missing = run_candidate_add(&ctx, &flags_of(&["--feature", "demo", "--head", "sha-1"]));
        assert_eq!(thrown(missing), "Missing required flag --mode.");
        assert!(!ledger.exists(), "a refused candidate add creates no ledger");

        let bad = run_candidate_add(
            &ctx,
            &flags_of(&["--feature", "demo", "--head", "sha-1", "--mode", "gigantic"]),
        );
        assert_eq!(
            thrown(bad),
            "candidate add: --mode is required and must be one of docs, tiny, small, spike, standard, high-risk (the closing feature's lane)."
        );
        assert!(!ledger.exists(), "an unrecognized mode leaves the ledger untouched");

        // Control: a legal mode appends exactly one row with the SPEC shape.
        let (entry, text) = emitted(run_candidate_add(
            &ctx,
            &flags_of(&["--feature", " demo ", "--head", " sha-1 ", "--mode", "standard", "--cells", "c1, ,c2"]),
        ));
        assert_eq!(entry["type"], json!("candidate"));
        assert_eq!(entry["feature"], json!("demo"));
        assert_eq!(entry["head"], json!("sha-1"));
        assert_eq!(entry["mode"], json!("standard"));
        assert_eq!(entry["baseline"], Value::Null);
        assert_eq!(entry["cells"], json!(["c1", "c2"]));
        assert!(text.ends_with("(mode standard, 2 cell(s))."), "{text}");
        let rows = list_candidates(root).ok().unwrap();
        assert_eq!(rows.len(), 1, "exactly one row survived the two refusals");
        assert_eq!(rows[0], entry);

        // A refusal AFTER a good row still leaves the ledger byte-unchanged.
        let before = std::fs::read(&ledger).unwrap();
        thrown(run_candidate_add(&ctx, &flags_of(&["--feature", "demo", "--head", "s", "--mode", "nope"])));
        assert_eq!(std::fs::read(&ledger).unwrap(), before);
    }

    // ─── fail-open read path ───────────────────────────────────────────────

    /// A corrupt session entry and an unreadable candidates ledger degrade the
    /// review block instead of failing it. Oracle: "bee.mjs status: a corrupt
    /// .bee/reviews entry and an unreadable candidates ledger degrade the
    /// review block but leave bee_status exiting 0".
    #[test]
    fn a_corrupt_session_entry_and_an_unreadable_ledger_degrade_rather_than_fail() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let ctx = ctx_or_skip!(root, "a_corrupt_session_entry_and_an_unreadable_ledger_degrade_rather_than_fail");
        capped_cell(root, "ok-1");
        emitted(create_with(&ctx, root, &base_scope()));
        // A session file that is valid JSON but not an object is skipped.
        write_session(root, "rev-corrupt", &json!("just a string"));
        let sessions = list_reviews(root).ok().unwrap();
        assert_eq!(sessions.len(), 1, "the good session survives the corrupt neighbour");
        assert_eq!(sessions[0]["id"], json!("rev-1"));

        // A ledger line that is not JSON at all is dropped, the rest kept.
        let ledger = candidates_path(root);
        std::fs::create_dir_all(ledger.parent().unwrap()).unwrap();
        std::fs::write(
            &ledger,
            "{\"id\":\"c1\",\"feature\":\"demo\"}\n{oops not json\n{\"id\":\"c2\",\"feature\":\"demo\"}\n",
        )
        .unwrap();
        let rows = list_candidates(root).ok().unwrap();
        assert_eq!(rows.len(), 2, "corrupt JSONL line skipped, the readable rows kept");
        assert_eq!(rows[0]["id"], json!("c1"));
        assert_eq!(rows[1]["id"], json!("c2"));

        // An UNREADABLE ledger (a directory where the file should be) reads as
        // empty rather than erroring — and reviews status still renders.
        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        let ctx2 = ctx_or_skip!(root2, "a_corrupt_session_entry_and_an_unreadable_ledger_degrade_rather_than_fail");
        std::fs::create_dir_all(candidates_path(root2)).unwrap();
        assert!(list_candidates(root2).ok().unwrap().is_empty());
        let (summary, text) = emitted(run_status(&ctx2, &flags_of(&[])));
        assert_eq!(summary["counts"]["verified"], json!(0));
        assert_eq!(summary["candidates"], json!([]));
        assert!(text.ends_with("No review candidates."), "{text}");
    }

    // ─── deriveCandidateStatus git-degradation arms ────────────────────────

    static GIT_CAPABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

    /// Probe the capability (a runnable `git`), never the platform.
    fn git_capable() -> bool {
        *GIT_CAPABLE.get_or_init(|| {
            std::process::Command::new("git")
                .arg("--version")
                .output()
                .map(|o| o.status.success() && !o.stdout.is_empty())
                .unwrap_or(false)
        })
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    fn git_out(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// test_reviews.mjs makeReviewGitRepo — bee scaffolding plus a real repo,
    /// because coverage/staleness is defined over actual commit ancestry.
    fn git_repo(dir: &Path) -> String {
        git(dir, &["init", "-q"]);
        git(dir, &["config", "user.email", "bee-review@example.com"]);
        git(dir, &["config", "user.name", "bee review tests"]);
        git(dir, &["config", "commit.gpgsign", "false"]);
        git_commit(dir, "seed.txt", "seed\n", "seed")
    }

    fn git_commit(dir: &Path, file: &str, content: &str, message: &str) -> String {
        std::fs::write(dir.join(file), content).unwrap();
        git(dir, &["add", file]);
        git(dir, &["commit", "-q", "-m", message]);
        git_out(dir, &["rev-parse", "HEAD"])
    }

    /// Deliberately shaped with the pre-rename `gate4` key: candidate
    /// derivation keys off `decision.status`, so a session written before the
    /// review gate was renumbered must keep deriving exactly the same way.
    fn approved_session(id: &str, head: &str) -> Value {
        json!({
            "id": id,
            "included": [{"type": "feature", "id": "demo"}],
            "head": head,
            "decision": {"status": "approved", "gate4": {"approved_by": "user"}},
        })
    }

    /// Oracle: "an approved session covers the candidate's exact head as
    /// \"reviewed\"; one extra commit after that head flips the SAME candidate
    /// to \"review stale\" … (A8)".
    #[test]
    fn derive_flips_reviewed_to_review_stale_when_a_commit_lands_after_the_session_head() {
        if !git_capable() {
            eprintln!(
                "SKIP (env: no runnable `git` on PATH — install git and re-run) \
                 derive_flips_reviewed_to_review_stale_when_a_commit_lands_after_the_session_head"
            );
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sha1 = git_repo(root);
        let sessions = vec![approved_session("rev-reviewed", &sha1)];
        let cand = json!({"feature": "demo", "head": sha1, "mode": "standard", "cells": []});

        let d = derive_candidate_status(root, &cand, &sessions, &mut GitMemo::new()).ok().unwrap();
        assert_eq!(d.status, "reviewed");
        assert_eq!(d.session, Some(json!("rev-reviewed")));
        assert_eq!(d.note, None);

        git_commit(root, "unrelated.txt", "unrelated\n", "unrelated commit after review head");
        // A fresh memo — the flip must come from git, not a cached answer.
        let d = derive_candidate_status(root, &cand, &sessions, &mut GitMemo::new()).ok().unwrap();
        assert_eq!(d.status, "review stale");
        assert_eq!(d.session, Some(json!("rev-reviewed")));
        assert_eq!(d.note, None, "a RESOLVABLE stale range carries no note");
    }

    /// Oracle: "an unresolvable candidate head (unknown sha, simulating
    /// rebase/amend) … degrades to \"review stale\" with a \"range
    /// unresolvable\" note, never throws".
    #[test]
    fn derive_degrades_when_the_candidate_head_cannot_be_resolved() {
        if !git_capable() {
            eprintln!(
                "SKIP (env: no runnable `git` on PATH — install git and re-run) \
                 derive_degrades_when_the_candidate_head_cannot_be_resolved"
            );
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sha1 = git_repo(root);
        let sessions = vec![approved_session("rev-unresolvable", &sha1)];
        let fake = "a".repeat(40);
        let cand = json!({"feature": "demo", "head": fake, "cells": []});

        let d = derive_candidate_status(root, &cand, &sessions, &mut GitMemo::new()).ok().unwrap();
        assert_eq!(d.status, "review stale");
        assert_eq!(d.note, Some("range unresolvable"));
        assert_eq!(d.session, Some(json!("rev-unresolvable")));
        // Control: a resolvable head in the same repo answers "reviewed", so
        // the degradation above is the unknown sha and not the fixture.
        let ok = json!({"feature": "demo", "head": sha1, "cells": []});
        let d = derive_candidate_status(root, &ok, &sessions, &mut GitMemo::new()).ok().unwrap();
        assert_eq!(d.status, "reviewed");
    }

    /// Oracle: "git binary unavailable (PATH stripped) never throws — a
    /// covering session degrades to \"review stale\"/\"range unresolvable\"".
    ///
    /// The oracle strips PATH in-process; Rust's threaded test harness makes
    /// `env::set_var` unsound (unsafe since edition 2024) while sibling tests
    /// run, so the identical branch is reached the other way: `run_git`
    /// returns None whenever the child cannot be spawned at all — here because
    /// its cwd does not exist. Same arm, no global mutation.
    #[test]
    fn derive_degrades_when_git_cannot_be_spawned_at_all() {
        if !git_capable() {
            eprintln!(
                "SKIP (env: no runnable `git` on PATH — install git and re-run) \
                 derive_degrades_when_git_cannot_be_spawned_at_all"
            );
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sha1 = git_repo(root);
        let sessions = vec![approved_session("rev-nogit", &sha1)];
        let cand = json!({"feature": "demo", "head": sha1, "cells": []});

        let unspawnable = root.join("does-not-exist");
        let d = derive_candidate_status(&unspawnable, &cand, &sessions, &mut GitMemo::new())
            .ok()
            .expect("a missing git child never delegates and never panics");
        assert_eq!(d.status, "review stale");
        assert_eq!(d.note, Some("range unresolvable"));
        // Control: the very same inputs against the real repo answer cleanly.
        let d = derive_candidate_status(root, &cand, &sessions, &mut GitMemo::new()).ok().unwrap();
        assert_eq!(d.status, "reviewed");
        assert_eq!(d.note, None);
    }

    /// Oracle: "a candidate whose head postdates the covering approved
    /// session's frozen head … derives \"unreviewed\" — not a stale
    /// re-labelling of unrelated new work".
    #[test]
    fn derive_returns_unreviewed_when_the_candidate_head_postdates_the_session() {
        if !git_capable() {
            eprintln!(
                "SKIP (env: no runnable `git` on PATH — install git and re-run) \
                 derive_returns_unreviewed_when_the_candidate_head_postdates_the_session"
            );
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sha1 = git_repo(root);
        let sessions = vec![approved_session("rev-old", &sha1)];
        let sha2 = git_commit(root, "more.txt", "more work\n", "new delta after review head");
        assert_ne!(sha1, sha2);

        let newer = json!({"feature": "demo", "head": sha2, "cells": []});
        let d = derive_candidate_status(root, &newer, &sessions, &mut GitMemo::new()).ok().unwrap();
        assert_eq!(d.status, "unreviewed", "new work is not a stale re-label");
        assert_eq!(d.session, None, "unreviewed carries no session reference");
        // Control: the OLD head against the same session is still covered —
        // so "unreviewed" above is the ancestry answer, not a broken fixture.
        let older = json!({"feature": "demo", "head": sha1, "cells": []});
        let d = derive_candidate_status(root, &older, &sessions, &mut GitMemo::new()).ok().unwrap();
        assert_eq!(d.status, "review stale", "sha1 is covered but one commit behind HEAD");
        assert_eq!(d.session, Some(json!("rev-old")));
    }

    /// Oracle: "a pass-local gitMemo dedupes repeated git invocations when
    /// multiple candidates share a covering session's (head,ref)/(ref) pair
    /// (D2) — derived statuses stay byte-identical to the unmemoized path".
    ///
    /// Proof that the memo — and not git — answered the second candidate: the
    /// second call runs against a cwd no child can be spawned in, where an
    /// unmemoized run degrades (the control at the end).
    #[test]
    fn git_memo_answers_a_second_candidate_sharing_the_same_head_pair() {
        if !git_capable() {
            eprintln!(
                "SKIP (env: no runnable `git` on PATH — install git and re-run) \
                 git_memo_answers_a_second_candidate_sharing_the_same_head_pair"
            );
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sha0 = git_repo(root);
        let sessions = vec![approved_session("rev-memo", &sha0)];
        let c1 = json!({"feature": "demo", "head": sha0, "cells": []});
        let c2 = json!({"feature": "demo", "head": sha0, "mode": "tiny", "cells": []});

        let mut memo = GitMemo::new();
        let first = derive_candidate_status(root, &c1, &sessions, &mut memo).ok().unwrap();
        assert_eq!(first.status, "reviewed");
        // The exact-head fast path never spawns merge-base; only the
        // rev-list range is memoized.
        assert!(memo.contains_key(&format!("since {sha0}")), "commitsSince memoized: {:?}", memo.keys());

        let unspawnable = root.join("does-not-exist");
        let second = derive_candidate_status(&unspawnable, &c2, &sessions, &mut memo).ok().unwrap();
        assert_eq!(second.status, first.status, "memoized status is byte-identical");
        assert_eq!(second.session, first.session);
        assert_eq!(second.note, first.note);

        // Control: without the memo the same unspawnable call degrades — so
        // the agreement above could only have come from the memo.
        let cold = derive_candidate_status(&unspawnable, &c2, &sessions, &mut GitMemo::new()).ok().unwrap();
        assert_eq!(cold.status, "review stale");
        assert_eq!(cold.note, Some("range unresolvable"));

        // The unresolvable answer is memoized too (cp-2): a second candidate
        // sharing an unresolvable head reuses it rather than re-spawning.
        let mut cold_memo = GitMemo::new();
        derive_candidate_status(&unspawnable, &c1, &sessions, &mut cold_memo).ok().unwrap();
        let reused = derive_candidate_status(&unspawnable, &c2, &sessions, &mut cold_memo).ok().unwrap();
        assert_eq!(reused.status, "review stale");
        assert_eq!(reused.note, Some("range unresolvable"));
        assert!(cold_memo.contains_key(&format!("since {sha0}")));
    }
}
