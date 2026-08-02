// bee cells — natively served slice.
//
// READ-ONLY (unchanged from R3 wave 1): `cells list`, `cells ready`,
// `cells show` (flags: --json; --feature/--status on list; --feature on
// ready; --id on show).
//
// SELECTING (R6): `cells claim-next` — the sweep, resolvePipeline, the
// cross-lane pool and the hold filters, then the shared claim half.
//
// MUTATING (this wave): add, update, claim, cap, finish, block, drop,
// unclaim, reopen, tier, judge, reset-budget, judge-record, schedule,
// archive, unarchive — each mirroring bee.mjs's dispatch frame (root resolve
// -> manifest-drift check -> handler -> emit/emitError -> timing) and the
// lib/cells.mjs mutators behind it: the same `cells:<id>` /
// `cells-archive` / `decisions` store-lock names (crate::lock — identical
// name strings, so Node and Rust serialize against each other), the same
// claims-store O_EXCL protocol (claimCellFile: fence_epoch 1 at creation,
// session resolution flag -> BEE_SESSION_ID -> CLAUDE_CODE_SESSION_ID ->
// single-live-session adoption), and atomic cell writes through
// crate::fsutil::write_json_atomic via the writeCell funnel (brief
// 'cells-archive' acquire + archived-only re-check, typed
// CELLS_ARCHIVE_BUSY on contention).
//
// R6 — `cells claim-next` IS NOW NATIVE (the last cells debt). All four
// pieces the previous header listed as missing are ported, in one piece so
// the sweep never half-runs:
//   1. sweepExpiredClaims (claims.mjs): the per-claim `.adopting` gate, the
//      `sessions` store lock around the heartbeat re-verify, the claim-file
//      removal, the claimed->open reset under `cells:<id>` (trace stamped
//      swept_at/swept_from_session), and one best-effort logDecision row per
//      actual reset.
//   2. resolvePipeline (state.mjs) — session -> bound lane -> default, with
//      the four typed LANE_INVALID/LANE_MISSING/LANE_CORRUPT refusals.
//   3. the pooling pass — readState + listLanes + listSessionRecords/
//      heartbeatStale (GH#20 live-owner skip) + featureBacklogRank
//      (verbs/backlog.rs, both the docs/backlog.md Feature-column walk and
//      the PBI fold's `a.id.localeCompare(b.id)` arm) + the created_at
//      tiebreak.
//   4. the per-candidate filters — findSessionConflicts (path leases) and
//      findForeignHolds over resolveHoldTopology's ordinary arm.
// The old "a partial port cannot fall back to Node afterwards" objection is
// answered, not ignored: the sweep removes its own trigger, so a Node re-run
// after a mid-flight delegate re-derives the identical end state and bytes.
// See the `cells claim-next` section comment for the full argument.
//
// STILL DELEGATED (file-header contract):
//   - every argv shape any ported verb cannot PROVE: unknown flags, missing
//     required flags, --help, bad enum/number values (Node's validate()
//     speaks there), non-flag tokens, non-UTF-8 argv.
//   - JS-exotic store shapes this port cannot carry: an array where a cell/
//     claim/session/lane record is expected (`typeof [] === 'object'` lets
//     them through Node's guards into index-key spreads), a string/array
//     `trace`, a non-string `feature` feeding path math. These delegate
//     BEFORE any output or write — the drift-cache write is the one
//     sanctioned pre-None write, exactly like the read-only slice.
//
// CUTOVER (2026-08-01) — CORRUPT JSON IS NATIVE. Contract C2 (byte-identical
// output with Node) is retired with the Node runtime, so the arms that used
// to hand a corrupt store back to Node — because Node's readJson warning
// interpolated V8's own `JSON.parse` message — now do the work here:
//   - readJson-backed reads (`read_cell_json` / `read_store_json`) warn via
//     crate::fsutil::warn_corrupt_json and take the SAME fallback Node's
//     readJson took (null / {} / the caller's default). Fail-open stays
//     fail-open; nothing that refused before stops refusing.
//   - the strict readers (readLaneStrict, readCellStrictForUpdate,
//     recoverArchiveJournal) keep their own deterministic refusals, and the
//     unreadable-file branches that used to embed a libuv errno now carry the
//     Rust io error in the same sentence.
//   - lone-surrogate escapes (`\uD800`-`\uDFFF`), which V8's JSON.parse
//     accepted and serde refuses, are simply CORRUPT now: there is no second
//     parser to defer to, so each site takes its own not-valid-JSON path.
//   - |n| >= 1e21 no longer diverges at all — jsjson::js_f64_to_string
//     implements the spec's exponential forms, so those arms are gone.
// The pre-scan that walked the whole cell store just to make that delegation
// decision is gone with them (it would have warned about files the command
// never reads); `warn_corrupt_json_once` keeps the surviving probes from
// double-warning about a file the real flow reads again.
//
// DOCUMENTED RESIDUAL DIVERGENCES (all pathological, none reachable from
// well-formed bee stores; each noted again at its code site):
//   - hard mid-transaction filesystem failures (a failing rename inside the
//     archive loop, a failing final writeCell): Node embedded the libuv errno
//     message; the native error text carries the Rust io message instead.
//   - a store file that turns corrupt in the window between a surviving
//     probe and a post-test re-read (cap/finish only) warns once, not twice —
//     `warn_corrupt_json_once` dedupes per path.
//   - declared test commands producing > 64 MiB of combined output: Node's
//     spawnSync maxBuffer kills the child (spawn error); the native runner
//     captures it all.

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::state as bstate;
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::{Err2, FlagV, Out, R2};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

/// Sentinel: this argv/store shape belongs to the Node runtime.
#[derive(Debug)]
struct Delegate;

#[derive(Clone, Copy, PartialEq)]
enum Verb {
    List,
    Ready,
    Show,
}

impl Verb {
    /// The dispatcher's timing label: `commandName.split('.').join(' ')`.
    fn cmd(self) -> &'static str {
        match self {
            Verb::List => "cells list",
            Verb::Ready => "cells ready",
            Verb::Show => "cells show",
        }
    }
}

#[derive(Default)]
struct Flags {
    json: bool,
    feature: Option<String>,
    status: Option<String>,
    id: Option<String>,
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "cells" {
        return None;
    }
    let verb = match args.get(1)?.to_str()? {
        "list" => Verb::List,
        "ready" => Verb::Ready,
        "show" => Verb::Show,
        other => return try_mutating(other, &args[2..], t0),
    };
    let flags = parse_flags(verb, &args[2..])?;
    if verb == Verb::Show {
        // Missing/empty --id takes Node's validate() "required, missing"
        // emission path (stdout-shaped, drift-line-bearing) — delegate it.
        if flags.id.as_deref().map(str::is_empty).unwrap_or(true) {
            return None;
        }
    }
    run(verb, flags, t0)
}

/// bee.mjs parseFlags, narrowed to the three verbs' own registry flags.
///
/// Provenance (bee.mjs parseFlags + FLAG_ALONE_BOOLEANS): `--json` is
/// flag-alone (never consumes a value; `--json=<anything>` still just sets
/// json, and matches main()'s pre-parse `--json`/`--json=` scan, so
/// parsed.json == jsonRequested for every accepted shape). `--feature`/
/// `--status`/`--id` take a value: `--flag=value` inline, or the next token.
/// A next token starting with `--` WOULD be consumed as the value by Node —
/// that shape (and a missing value token, a bare positional, any unknown
/// flag such as `--help`, and non-UTF-8 argv) delegates instead. Repeated
/// flags keep Node's last-wins overwrite.
fn parse_flags(verb: Verb, tokens: &[OsString]) -> Option<Flags> {
    let mut out = Flags::default();
    let mut i = 0usize;
    while i < tokens.len() {
        let tok = tokens[i].to_str()?;
        if !tok.starts_with("--") {
            return None; // Node: parse error "unexpected argument" — delegate
        }
        let body = &tok[2..];
        let (name, inline) = match body.find('=') {
            Some(pos) => (&body[..pos], Some(body[pos + 1..].to_string())),
            None => (body, None),
        };
        if name == "json" {
            out.json = true;
            i += 1;
            continue;
        }
        let allowed = matches!(
            (verb, name),
            (Verb::List, "feature") | (Verb::List, "status") | (Verb::Ready, "feature") | (Verb::Show, "id")
        );
        if !allowed {
            return None; // unknown flag (incl. --help) — Node owns the refusal/help
        }
        let value = match inline {
            Some(v) => v,
            None => {
                let next = tokens.get(i + 1)?.to_str()?;
                if next.starts_with("--") {
                    return None; // Node would eat a flag token as the value — not proven here
                }
                i += 1;
                next.to_string()
            }
        };
        match name {
            "feature" => out.feature = Some(value),
            "status" => out.status = Some(value),
            "id" => out.id = Some(value),
            _ => unreachable!(),
        }
        i += 1;
    }
    Some(out)
}

fn run(verb: Verb, flags: Flags, t0: Instant) -> Option<ExitCode> {
    let cwd = std::env::current_dir().ok()?;
    let use_json = flags.json;

    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, verb.cmd(), use_json, t0, &why))
        }
        Roots::None => return Some(emit_no_root_error(&cwd, verb.cmd(), use_json, t0)),
    };

    let drift = check_manifest_drift(&root).ok()?;

    // handleCellsList/Ready: `flags.feature ? String(flags.feature) : null` —
    // empty string is falsy, so it never filters.
    let feature = flags.feature.as_deref().filter(|s| !s.is_empty());
    let status = flags.status.as_deref().filter(|s| !s.is_empty());

    let outcome = match verb {
        Verb::List => handle_list(&root, feature, status),
        Verb::Ready => handle_ready(&root, feature),
        Verb::Show => handle_show(&root, flags.id.as_deref().unwrap_or("")),
    };

    match outcome {
        Err(Delegate) => None, // no output has happened — Node re-runs the command
        Ok(Handled::Emit { result, text }) => {
            // emit(): drift stderr line first, then the bare result on stdout.
            if drift.manifest_changed {
                eprintln!("manifest_changed: true — {}", drift.hint);
            }
            if use_json {
                println!("{}", jsjson::stringify_pretty(&result));
            } else {
                println!("{text}");
            }
            record_timing(&root, verb.cmd(), t0, true);
            Some(ExitCode::SUCCESS)
        }
        Ok(Handled::Error(message)) => {
            // emitError(): handler throw — NO drift line on this path (the
            // cache write above already happened, matching Node), --json gets
            // a compact {"error": ...} on stdout, text mode goes to stderr.
            if use_json {
                println!("{}", jsjson::stringify(&serde_json::json!({ "error": message })));
            } else {
                eprintln!("{message}");
            }
            record_timing(&root, verb.cmd(), t0, false);
            Some(ExitCode::FAILURE)
        }
    }
}

enum Handled {
    Emit { result: Value, text: String },
    Error(String),
}

// ─── bee.mjs handlers ──────────────────────────────────────────────────────

/// handleCellsList (bee.mjs): listCells with the two truthiness-normalized
/// filters; text is one summarizeCell line per cell or "No cells.".
fn handle_list(root: &Path, feature: Option<&str>, status: Option<&str>) -> Result<Handled, Delegate> {
    let cells = list_cells(root, feature, status)?;
    let text = if cells.is_empty() {
        "No cells.".to_string()
    } else {
        cells.iter().map(summarize_cell).collect::<Vec<_>>().join("\n")
    };
    Ok(Handled::Emit { result: Value::Array(cells), text })
}

/// handleCellsReady (bee.mjs): readyCells = listCells({status:'open'})
/// filtered to cells whose depsAllCapped list is empty.
fn handle_ready(root: &Path, feature: Option<&str>) -> Result<Handled, Delegate> {
    let mut ready = Vec::new();
    for cell in list_cells(root, feature, Some("open"))? {
        if deps_all_capped_is_empty(root, &cell)? {
            ready.push(cell);
        }
    }
    let text = if ready.is_empty() {
        "No ready cells.".to_string()
    } else {
        ready.iter().map(summarize_cell).collect::<Vec<_>>().join("\n")
    };
    Ok(Handled::Emit { result: Value::Array(ready), text })
}

/// handleCellsShow (bee.mjs): readCell -> not-found throw (byte-matched
/// message) -> withVerifyOwner -> {result: annotated, text: pretty JSON}.
/// Both output modes print the identical JSON.stringify(annotated, null, 2).
fn handle_show(root: &Path, id: &str) -> Result<Handled, Delegate> {
    let cell = match read_cell(root, id)? {
        None => return Ok(Handled::Error(format!("Cell \"{id}\" not found."))),
        Some(v) => v,
    };
    // A truthy non-object cell file (number/string/bool/array JSON) takes
    // Object.entries()/! coercion paths whose renders are JS-exotic — Node's.
    let Value::Object(map) = cell else { return Err(Delegate) };
    let annotated = Value::Object(with_verify_owner(&map));
    let text = jsjson::stringify_pretty(&annotated);
    Ok(Handled::Emit { result: annotated, text })
}

/// bee.mjs VERIFY_OWNER_ANNOTATION (vo-1, R82 main-verifies).
const VERIFY_OWNER_ANNOTATION: &str = "main (feature close) — the worker never runs this";

/// bee.mjs withVerifyOwner: re-build the object inserting `verify_owner`
/// immediately after the `verify` key; append at the end when the cell has
/// no `verify` key at all. Key order is otherwise the file's own (JS
/// insertion order == serde_json preserve_order).
fn with_verify_owner(cell: &Map<String, Value>) -> Map<String, Value> {
    let mut out = Map::new();
    let mut inserted = false;
    for (key, value) in cell {
        out.insert(key.clone(), value.clone());
        if key == "verify" {
            out.insert("verify_owner".into(), Value::String(VERIFY_OWNER_ANNOTATION.into()));
            inserted = true;
        }
    }
    if !inserted {
        out.insert("verify_owner".into(), Value::String(VERIFY_OWNER_ANNOTATION.into()));
    }
    out
}

/// bee.mjs summarizeCell: `${cell.id} [${cell.status}] (${cell.lane})
/// ${cell.title}` — template-literal coercion, so an absent field renders
/// "undefined", an object "[object Object]", an array its comma-join.
fn summarize_cell(cell: &Value) -> String {
    format!(
        "{} [{}] ({}) {}",
        js_string_or_undefined(cell.get("id")),
        js_string_or_undefined(cell.get("status")),
        js_string_or_undefined(cell.get("lane")),
        js_string_or_undefined(cell.get("title"))
    )
}

// ─── lib/cells.mjs read path ───────────────────────────────────────────────

fn cells_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("cells")
}

/// lib/cells.mjs ARCHIVE_DIR_NAME — a reserved child of cellsDir.
const ARCHIVE_DIR_NAME: &str = "archive";

/// lib/cells.mjs ID_PATTERN: /^[A-Za-z0-9][A-Za-z0-9._-]*$/.
fn id_pattern_ok(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// fsutil.mjs readJson(file, null) for a cell file, split three ways:
/// - Ok(Some(v))  — parsed, non-null (JS `!== null`)
/// - Ok(None)     — absent/unreadable (fallback), a literal JSON `null`, or
///   present-but-corrupt: readJson warns to stderr and returns the `null`
///   fallback, and so does this (CUTOVER — see the file header).
/// - Err(Delegate) is unreachable here now; the type stays so the JS-exotic
///   arms in this module's callers keep one error channel.
fn read_cell_json(file: &Path) -> Result<Option<Value>, Delegate> {
    match read_json(file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => {
            warn_corrupt_json_once(file);
            Ok(None) // readJson(file, null) fail-open — identical to Missing
        }
        ReadJson::Parsed(Value::Null) => Ok(None),
        ReadJson::Parsed(v) => Ok(Some(v)),
    }
}

/// `crate::fsutil::warn_corrupt_json`, at most once per path per process.
///
/// Node had no pre-scan: it warned once per readJson CALL, at the point the
/// real flow read the file. This port keeps a few pre-lock probes that read
/// the same file the flow reads again a moment later (they exist to bail
/// BEFORE a store lock is acquired, not merely to decide delegation), so the
/// raw helper would print the same sentence twice for one logical read.
/// Deduping per path keeps the user-visible warning count at Node's.
fn warn_corrupt_json_once(file: &Path) {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static SEEN: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    let seen = SEEN.get_or_init(|| Mutex::new(HashSet::new()));
    let mut guard = match seen.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if guard.insert(file.to_path_buf()) {
        crate::fsutil::warn_corrupt_json(file);
    }
}

/// lib/cells.mjs listCells(root, {feature, status}) — includeArchived is
/// never passed by list/ready, so only the active .bee/cells/*.json scan is
/// ported. Directory entries (the `archive` child) and non-.json names are
/// skipped; a cell parsing to a falsy or non-object value is skipped exactly
/// as `!cell || typeof cell !== 'object'` skips it. A JSON *array* cell
/// passes that JS check (typeof [] === 'object') and would flow into the
/// renderers with exotic coercions — delegated instead of approximated.
/// Filters are strict-equality against string fields (an absent field never
/// matches). Sort: String(id).localeCompare(String(id), 'en', {numeric:true})
/// via natural_cmp below; V8's Array#sort and Rust's sort_by are both stable,
/// and both runtimes enumerate the directory in the same OS order, so ids
/// comparing equal (e.g. leading-zero variants) keep identical output order.
fn list_cells(root: &Path, feature: Option<&str>, status: Option<&str>) -> Result<Vec<Value>, Delegate> {
    let dir = cells_dir(root);
    let mut cells: Vec<Value> = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(cells), // readdirSync catch -> entries = []
    };
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue; // explicit guard: `archive` (or any dir) is never a cell
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.ends_with(".json") {
            continue;
        }
        let cell = match read_cell_json(&entry.path())? {
            None => continue, // readJson fallback null -> `!cell` skips
            Some(v) => v,
        };
        let map = match &cell {
            Value::Object(m) => m,
            Value::Array(_) => return Err(Delegate), // JS-exotic: typeof [] === 'object'
            _ => continue,                           // primitives fail `typeof === 'object'`
        };
        if let Some(f) = feature {
            if !matches!(map.get("feature"), Some(Value::String(s)) if s == f) {
                continue; // strict !==; absent/non-string never equals the filter
            }
        }
        if let Some(st) = status {
            if !matches!(map.get("status"), Some(Value::String(s)) if s == st) {
                continue;
            }
        }
        cells.push(cell);
    }
    cells.sort_by(|a, b| {
        natural_cmp(
            &js_string_or_undefined(a.get("id")),
            &js_string_or_undefined(b.get("id")),
        )
    });
    Ok(cells)
}

/// lib/cells.mjs readCell(root, id): malformed/falsy id -> null; the active
/// .bee/cells/<id>.json wins when it reads non-null; otherwise every
/// .bee/cells/archive/<feature>/ directory is probed for <id>.json in
/// directory order (readdirSync error -> null). Corrupt JSON anywhere on the
/// probe path warns and reads as absent, exactly like readJson's fallback.
fn read_cell(root: &Path, id: &str) -> Result<Option<Value>, Delegate> {
    if id.is_empty() || !id_pattern_ok(id) {
        return Ok(None);
    }
    let active = cells_dir(root).join(format!("{id}.json"));
    if let Some(v) = read_cell_json(&active)? {
        return Ok(Some(v));
    }
    let archive_root = cells_dir(root).join(ARCHIVE_DIR_NAME);
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

/// lib/cells.mjs depsAllCapped(root, cell).length === 0 (the only reading
/// readyCells does). `cell.deps || []`: a falsy deps value iterates nothing;
/// an array iterates its elements; a truthy STRING would be for..of'd
/// char-by-char and any other truthy value throws a V8 TypeError — both
/// JS-exotic, both delegated. Every dep is visited (Node never early-exits,
/// so a corrupt dep file late in the list still warns there — the full scan
/// preserves that delegation trigger). A falsy dep, a malformed dep id, a
/// missing dep cell, and a dep whose status !== 'capped' all count as
/// not-capped; readCell's archive fallback means an archived capped dep
/// still satisfies readiness. String(dep) coercion feeds non-string deps
/// through the same id path Node uses.
fn deps_all_capped_is_empty(root: &Path, cell: &Value) -> Result<bool, Delegate> {
    let deps = cell.get("deps").unwrap_or(&Value::Null);
    if !js_truthy(deps) {
        return Ok(true);
    }
    let Value::Array(deps) = deps else { return Err(Delegate) };
    let mut all_capped = true;
    for dep in deps {
        let capped = if !js_truthy(dep) {
            false // readCell's `!id` guard: a falsy dep never resolves
        } else {
            let id = jsjson::js_to_string(dep);
            match read_cell(root, &id)? {
                None => false,
                Some(dep_cell) => {
                    matches!(dep_cell.get("status"), Some(Value::String(s)) if s == "capped")
                }
            }
        };
        if !capped {
            all_capped = false; // keep scanning: Node collects every miss
        }
    }
    Ok(all_capped)
}

// ─── JS value semantics ────────────────────────────────────────────────────

/// JS truthiness for a JSON value (undefined callers pass Null).
fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// Template-literal coercion for a possibly-absent field: undefined renders
/// "undefined", everything else through String() (jsjson::js_to_string).
fn js_string_or_undefined(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

// ─── String.prototype.localeCompare(b, 'en', {numeric: true}) ──────────────
//
// Faithful for the ascii slugs ID_PATTERN allows (letters, digits, '.', '_',
// '-'), calibrated against V8/ICU probes:
//   - primary: whitespace < punctuation < digits < letters; within
//     punctuation ICU orders '_' < '-' < '.'; letters case-insensitively;
//     a digit run compares as an integer (leading zeros stripped), and runs
//     of EQUAL numeric value — including "01" vs "1" — are fully equal at
//     every ICU level, so "a01" and "a1" compare 0 (stability then preserves
//     directory order, identically in both runtimes).
//   - tertiary (only when primary ties): the first case difference over the
//     non-digit characters decides, lowercase first — deferred, never
//     per-character ("Ab" < "aC" because primary b<c wins over A>a).
// Documented approximations: other punctuation falls back to code-point
// order after the three anchored slugs chars, and non-ASCII letters compare
// by code point after lowercasing (ICU accent secondaries are not modeled).
fn natural_cmp(a: &str, b: &str) -> Ordering {
    primary_cmp(a, b).then_with(|| tertiary_case_cmp(a, b))
}

/// ICU primary-strength class rank (probe-calibrated).
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

/// Punctuation key within rank 1: ICU orders '_' < '-' < '.'; anything else
/// (never produced by ID_PATTERN ids) falls back to code-point order after
/// those three — documented approximation.
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
            // Integer compare: shorter trimmed run is smaller, then lexicographic.
            let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
            if ord != Ordering::Equal {
                return ord;
            }
            continue; // equal value (leading zeros carry no weight at any level)
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

/// Tertiary case pass over the non-digit characters (digit runs that tied at
/// primary strength — including leading-zero variants — contribute nothing,
/// per the "a01B" vs "a1b" ICU probes). Primary equality guarantees the two
/// non-digit sequences pair up one-to-one.
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

// ═══════════════════════════════════════════════════════════════════════════
// MUTATING VERBS
// ═══════════════════════════════════════════════════════════════════════════

/// Handler failure: hand the whole command to Node (no output yet), or a
/// thrown-Error message (bee.mjs emitError bytes).
#[derive(Debug)]
pub(crate) enum Fail {
    Delegate,
    Thrown(String),
}
pub(crate) type MR<T> = Result<T, Fail>;

impl From<Delegate> for Fail {
    fn from(_: Delegate) -> Self {
        Fail::Delegate
    }
}
impl From<rsv::Exotic> for Fail {
    fn from(_: rsv::Exotic) -> Self {
        Fail::Delegate
    }
}

fn to_r2(out: MR<Out>) -> R2<Out> {
    match out {
        Ok(o) => Ok(o),
        Err(Fail::Delegate) => Err(Err2::Ex),
        Err(Fail::Thrown(m)) => Ok(Out::Thrown(m)),
    }
}

/// Routing for the mutating verbs. Flags parse via bee.mjs's own grammar
/// (rsv::parse_flags); anything Node's dispatcher/validate() answers itself
/// (unknown flags, missing required flags, bad enum/number values, --help)
/// returns None before any output.
fn try_mutating(verb: &str, rest: &[OsString], t0: Instant) -> Option<ExitCode> {
    let toks: Vec<&str> = rest.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // Node renders command-scoped help
    }
    let (flags, use_json) = rsv::parse_flags(&toks)?;
    match verb {
        "add" => run_add(flags, use_json, t0),
        "update" => run_update(flags, use_json, t0),
        "claim" => run_claim(flags, use_json, t0),
        "cap" => run_cap(false, flags, use_json, t0),
        "finish" => run_cap(true, flags, use_json, t0),
        "block" => run_block(flags, use_json, t0),
        "drop" => run_drop(flags, use_json, t0),
        "unclaim" => run_unclaim(flags, use_json, t0),
        "reopen" => run_reopen(flags, use_json, t0),
        "tier" => run_tier(flags, use_json, t0),
        "judge" => run_judge(flags, use_json, t0),
        "reset-budget" => run_reset_budget(flags, use_json, t0),
        "judge-record" => run_judge_record(flags, use_json, t0),
        "schedule" => run_schedule(flags, use_json, t0),
        "archive" => run_archive(flags, use_json, t0),
        "unarchive" => run_unarchive(flags, use_json, t0),
        "claim-next" => run_claim_next(flags, use_json, t0),
        _ => None, // every unknown verb stays with Node
    }
}

/// Boolean-typed flag through validate(): absent/Present pass; "true"/"false"
/// strings pass validate but read as JS non-`true`; any other value is
/// validate()'s refusal — delegate (None).
fn bool_flag(flags: &rsv::Flags, name: &str) -> Option<bool> {
    match flags.get(name) {
        None => Some(false),
        Some(FlagV::Present) => Some(true),
        Some(FlagV::S(s)) if s == "true" || s == "false" => Some(false), // string !== true
        Some(FlagV::S(_)) => None,
    }
}

/// `flags[name] !== undefined ? String(flags[name]) : undefined` for a
/// value-typed flag. Present (boolean true) is impossible for non-flag-alone
/// names by the parser's grammar.
fn opt_string_flag(flags: &rsv::Flags, name: &str) -> Option<Option<String>> {
    match flags.get(name) {
        None => Some(None),
        Some(FlagV::S(s)) => Some(Some(s.clone())),
        Some(FlagV::Present) => None,
    }
}

// ─── shared JS/string helpers ──────────────────────────────────────────────

fn js_trim(s: &str) -> &str {
    rsv::js_trim(s)
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn utc_now() -> String {
    rsv::now_iso()
}

/// Array.prototype.join(', ') with JS element coercion (null/undefined -> '').
fn js_join(values: &[Value], sep: &str) -> String {
    values
        .iter()
        .map(|v| match v {
            Value::Null => String::new(),
            other => jsjson::js_to_string(other),
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// JSON.stringify(value) rendered into a template literal; None (undefined)
/// renders "undefined" like JS does.
fn js_json_or_undefined(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => jsjson::stringify(v),
    }
}

/// JS default Array#sort comparator over strings (UTF-16 code-unit order).
fn js_default_str_sort(items: &mut [String]) {
    items.sort_by(|a, b| {
        let au: Vec<u16> = a.encode_utf16().collect();
        let bu: Vec<u16> = b.encode_utf16().collect();
        au.cmp(&bu)
    });
}

/// JSON.parse with JS number semantics. `strip_bom` mirrors readJson's BOM
/// strip (the strict readers do NOT strip). Only two outcomes now:
/// - Value: parsed and JS-number-normalized.
/// - NotJson: the text is not JSON, and every caller takes its own
///   not-valid-JSON path for it.
///
/// CUTOVER: this used to have a third `Delegate` outcome for the two shapes
/// only V8 could settle — a `\uD800`-`\uDFFF` lone-surrogate escape (V8's
/// JSON.parse accepted it, serde refuses) and a |n| >= 1e21 number (jsjson
/// printed it differently than V8). The number case is fixed upstream
/// (`js_f64_to_string` does the spec's exponential forms, `js_numberify` no
/// longer rejects), and with no Node to defer to, a lone surrogate is just
/// input this CLI cannot parse — i.e. NotJson, exactly like any other text
/// serde refuses.
enum JsParse {
    Value(Value),
    NotJson,
}

fn parse_json_js(text: &str, strip_bom: bool) -> JsParse {
    let mut t = text;
    if strip_bom {
        if let Some(stripped) = t.strip_prefix('\u{feff}') {
            t = stripped;
        }
    }
    match serde_json::from_str::<Value>(t) {
        Ok(v) => match rsv::js_numberify(&v) {
            // Unreachable from JSON text (no NaN/Infinity literals), but a
            // parse helper must not panic on it either.
            Err(_) => JsParse::NotJson,
            Ok(v) => JsParse::Value(v),
        },
        // Includes lone-surrogate escapes — see the doc comment above.
        Err(_) => JsParse::NotJson,
    }
}

// ─── store-file readers with JS-number normalization ───────────────────────

/// readJson-backed store read (BOM-stripped): Missing -> None; Corrupt ->
/// warn + None, which is exactly readJson's fail-open `fallback` for every
/// caller here (each one maps a missing file and its fallback to the same
/// branch). Parsed values are JS-number-normalized.
fn read_store_json(file: &Path) -> Result<Option<Value>, Delegate> {
    match read_json(file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => {
            warn_corrupt_json_once(file);
            Ok(None)
        }
        ReadJson::Parsed(v) => match rsv::js_numberify(&v) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Err(Delegate), // unreachable from JSON — see js_numberify
        },
    }
}

/// readCell with JS-number normalization for the mutators (the read-only
/// slice keeps its own un-normalized reader above).
fn read_cell_norm(root: &Path, id: &str) -> Result<Option<Value>, Delegate> {
    match read_cell(root, id)? {
        None => Ok(None),
        Some(v) => match rsv::js_numberify(&v) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Err(Delegate),
        },
    }
}

fn cell_file(root: &Path, id: &str) -> PathBuf {
    cells_dir(root).join(format!("{id}.json"))
}

/// lib/cells.mjs resolveCellFile — existence-only (never parses).
fn resolve_cell_file(root: &Path, id: &str) -> Option<PathBuf> {
    if id.is_empty() || !id_pattern_ok(id) {
        return None;
    }
    let active = cell_file(root, id);
    if active.exists() {
        return Some(active);
    }
    let archive_root = cells_dir(root).join(ARCHIVE_DIR_NAME);
    let entries = std::fs::read_dir(&archive_root).ok()?;
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let candidate = entry.path().join(format!("{id}.json"));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// lib/cells.mjs CellArchivedError message.
fn cell_archived_error(verb: &str, id: &str) -> String {
    format!(
        "{verb}: cell \"{id}\" is archived — unarchive its feature first (bee cells unarchive --feature <feature>)."
    )
}

/// lib/cells.mjs assertNotArchived: no-op for malformed ids, active cells,
/// and genuinely missing ids; throws CELL_ARCHIVED for archived-only ids.
fn assert_not_archived(root: &Path, verb: &str, id: &str) -> MR<()> {
    if id.is_empty() || !id_pattern_ok(id) {
        return Ok(());
    }
    if cell_file(root, id).exists() {
        return Ok(());
    }
    if resolve_cell_file(root, id).is_some() {
        return Err(Fail::Thrown(cell_archived_error(verb, id)));
    }
    Ok(())
}

/// lock-holder rendering shared by CellsArchiveBusyError and
/// DecisionsLockBusyError: `pid=<p> session=<s> since <ts>` with ?? 'unknown'.
fn holder_who(holder: &Option<Value>) -> String {
    match holder {
        Some(Value::Object(h)) => {
            let field = |k: &str| match h.get(k) {
                Some(Value::Null) | None => "unknown".to_string(),
                Some(v) => jsjson::js_to_string(v),
            };
            format!("pid={} session={} since {}", field("pid"), field("session"), field("ts"))
        }
        _ => "unknown holder".to_string(),
    }
}

/// lib/cells.mjs writeCell — the single write funnel: valid-id check, brief
/// 'cells-archive' single-attempt acquire (typed CELLS_ARCHIVE_BUSY on
/// contention), archived-only re-check under that lock, atomic write.
fn write_cell(root: &Path, cell: &Value) -> MR<()> {
    let id = match cell.get("id") {
        Some(Value::String(s)) if !s.is_empty() && id_pattern_ok(s) => s.clone(),
        other => {
            return Err(Fail::Thrown(format!(
                "writeCell: cell needs a valid id (got {}).",
                js_json_or_undefined(other)
            )))
        }
    };
    match lock::acquire_store_lock_once(root, "cells-archive") {
        lock::AcquireOnce::Busy { holder } => Err(Fail::Thrown(format!(
            "writeCell: cell \"{id}\" write refused — the \"cells-archive\" lock is held by {} (a live archive/unarchive transaction). Retry once it completes.",
            holder_who(&holder)
        ))),
        lock::AcquireOnce::Acquired(mut guard) => {
            let result = (|| -> MR<()> {
                let active = cell_file(root, &id);
                if !active.exists() && resolve_cell_file(root, &id).is_some() {
                    return Err(Fail::Thrown(cell_archived_error("writeCell", &id)));
                }
                // Node throws the raw fs error here (libuv bytes) — residual
                // divergence on a hard write failure, documented in the header.
                write_json_atomic(&active, cell)
                    .map_err(|e| Fail::Thrown(format!("writeCell: {e}")))
            })();
            guard.release();
            result
        }
    }
}

/// withStoreLock(root, name, ..) — Node's async bounded-retry acquisition;
/// a LockBusyError's message surfaces via emitError.
fn acquire_named_lock(root: &Path, name: &str) -> MR<lock::LockGuard> {
    lock::acquire_store_lock(root, name, lock::MAX_ATTEMPTS).map_err(|busy| Fail::Thrown(busy.message()))
}

// Provenance markers for later sections are appended below in order:
// claims-store protocol, trace helpers, text scanners, decision log,
// regen guards, validators, budgets, judge, schedule, test runner,
// reservations-release subset, impact registry, and the verb handlers.
// ─── claims.mjs port — sessions + per-cell O_EXCL claim files ──────────────
// All claim/session stores are CONTROL-PLANE (msn-18b): resolved through
// controlRootFor (rsv::control_root_for — the same git-common-dir walk the
// reservations port already carries). For the ordinary checkouts this native
// path serves, control root == root.

const DEFAULT_CLAIM_TTL_SECONDS: f64 = 3600.0;
const HEARTBEAT_STALE_SECONDS: f64 = 900.0;
const GATE_RETRY_ATTEMPTS: u32 = 15;
const GATE_RETRY_DELAY_MS: u64 = 20;

fn control_root(root: &Path) -> MR<PathBuf> {
    let s = root.to_str().ok_or(Fail::Delegate)?;
    Ok(PathBuf::from(rsv::control_root_for(s).map_err(|_| Fail::Delegate)?))
}

fn claims_dir(control: &Path) -> PathBuf {
    control.join(".bee").join("claims")
}

fn sessions_dir(control: &Path) -> PathBuf {
    control.join(".bee").join("sessions")
}

/// claims.mjs requireId.
fn require_id(value: &str, label: &str) -> MR<String> {
    if js_trim(value).is_empty() {
        return Err(Fail::Thrown(format!("{label} is required.")));
    }
    let id = js_trim(value).to_string();
    if id.contains('\\') || id.contains('/') || id.contains("..") {
        return Err(Fail::Thrown(format!("{label} must be a plain id (no path separators).")));
    }
    Ok(id)
}

fn claim_path(control: &Path, cell_id: &str) -> MR<PathBuf> {
    Ok(claims_dir(control).join(format!("{}.json", require_id(cell_id, "cell id")?)))
}

fn claim_gate_path(control: &Path, cell_id: &str) -> MR<PathBuf> {
    Ok(claims_dir(control).join(format!("{}.adopting", require_id(cell_id, "cell id")?)))
}

/// claims.mjs readClaim: readJson fallback null; falsy/non-object -> null
/// (a corrupt file warns and lands in that same null). A JSON-array claim
/// file would take JS property paths this port does not model — Delegate.
fn read_claim(control: &Path, cell_id: &str) -> MR<Option<Map<String, Value>>> {
    let file = claim_path(control, cell_id)?;
    match read_store_json(&file)? {
        None => Ok(None),
        Some(Value::Object(m)) => Ok(Some(m)),
        Some(Value::Array(_)) => Err(Fail::Delegate),
        Some(_) => Ok(None), // primitives fail `typeof === 'object'` (null falls earlier)
    }
}

/// claims.mjs isClaimExpired: non-finite/non-positive TTL never expires;
/// unparseable claimed_at never expires.
fn claim_expired(claim: &Map<String, Value>, now: f64) -> MR<bool> {
    let ttl = match claim.get("ttl_seconds") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        _ => return Ok(false), // Number.isFinite(non-number) -> false
    };
    if !ttl.is_finite() || ttl <= 0.0 {
        return Ok(false);
    }
    match rsv::date_parse_val(claim.get("claimed_at")).map_err(|_| Fail::Delegate)? {
        None => Ok(false),
        Some(ms) => Ok(ms + ttl * 1000.0 <= now),
    }
}

fn claim_active(claim: Option<&Map<String, Value>>, now: f64) -> MR<bool> {
    match claim {
        None => Ok(false),
        Some(c) => Ok(!claim_expired(c, now)?),
    }
}

/// claims.mjs claimExpiry: 'no expiry' | 'expires <iso>'.
fn claim_expiry(claim: Option<&Map<String, Value>>) -> MR<String> {
    let Some(claim) = claim else { return Ok("no expiry".to_string()) };
    let claimed = rsv::date_parse_val(claim.get("claimed_at")).map_err(|_| Fail::Delegate)?;
    let ttl = match claim.get("ttl_seconds") {
        Some(Value::Number(n)) => n.as_f64(),
        _ => None,
    };
    match (claimed, ttl) {
        (Some(c), Some(t)) if t.is_finite() && t > 0.0 => {
            Ok(format!("expires {}", rsv::iso_from_ms(c + t * 1000.0).map_err(|_| Fail::Delegate)?))
        }
        _ => Ok("no expiry".to_string()),
    }
}

/// claims.mjs withTransientFsRetry (EBUSY/EPERM/ENOTEMPTY/EMFILE/ENFILE class,
/// 15 x 20ms) — the same Windows sharing-violation classes lock.rs retries.
fn transient_fs_retry<T>(mut f: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    #[cfg(windows)]
    fn transient(e: &std::io::Error) -> bool {
        matches!(e.raw_os_error(), Some(4 | 5 | 32 | 33 | 145))
    }
    #[cfg(unix)]
    fn transient(e: &std::io::Error) -> bool {
        matches!(e.raw_os_error(), Some(code) if {
            code == libc::EBUSY || code == libc::EPERM || code == libc::ENOTEMPTY
                || code == libc::EMFILE || code == libc::ENFILE
        })
    }
    let mut attempt = 0u32;
    loop {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                if !transient(&e) || attempt >= GATE_RETRY_ATTEMPTS {
                    return Err(e);
                }
                std::thread::sleep(std::time::Duration::from_millis(GATE_RETRY_DELAY_MS));
            }
        }
    }
}

/// claims.mjs acquireGate: single 'wx' attempt writing {pid, at}. EEXIST ->
/// false. Any other fs error throws in Node with libuv bytes — residual
/// divergence (Rust io message), documented in the header.
fn acquire_gate(control: &Path, cell_id: &str) -> MR<bool> {
    let file = claim_gate_path(control, cell_id)?;
    let body = format!(
        "{}\n",
        jsjson::stringify(&json!({ "pid": std::process::id(), "at": utc_now() }))
    );
    let result = transient_fs_retry(|| {
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&file) {
            Ok(mut f) => {
                use std::io::Write;
                f.write_all(body.as_bytes())
            }
            Err(e) => Err(e),
        }
    });
    match result {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(Fail::Thrown(format!("{e}"))),
    }
}

fn release_gate(control: &Path, cell_id: &str) {
    if let Ok(file) = claim_gate_path(control, cell_id) {
        let _ = transient_fs_retry(|| match std::fs::remove_file(&file) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        });
    }
}

fn acquire_gate_with_retry(control: &Path, cell_id: &str) -> MR<bool> {
    for attempt in 0..GATE_RETRY_ATTEMPTS {
        if acquire_gate(control, cell_id)? {
            return Ok(true);
        }
        if attempt + 1 < GATE_RETRY_ATTEMPTS {
            std::thread::sleep(std::time::Duration::from_millis(GATE_RETRY_DELAY_MS));
        }
    }
    Ok(false)
}

/// claims.mjs clearClaim — unconditional claim-file removal for the
/// claim-clearing cell transitions (cap/unclaim/block/drop/reopen). The
/// production call sites wrap it best-effort (releaseClaimFileBestEffort
/// swallows every failure), so this returns () and never fails the caller.
fn release_claim_file_best_effort(root: &Path, id: &str) {
    let Ok(control) = control_root(root) else { return };
    let _ = (|| -> MR<()> {
        if read_claim(&control, id)?.is_none() {
            return Ok(());
        }
        if !acquire_gate(&control, id)? {
            return Ok(()); // GATE_HELD — best-effort caller never retries
        }
        let still_there = read_claim(&control, id);
        if let Ok(Some(_)) = still_there {
            if let Ok(file) = claim_path(&control, id) {
                let _ = transient_fs_retry(|| match std::fs::remove_file(&file) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    other => other,
                });
            }
        }
        release_gate(&control, id);
        Ok(())
    })();
}

// ─── claims: fencing, adoption and same-session renewal ───────────────────
//
// multisession-native-12 (D4/D9 invariant 10). `fence_epoch` is a CAS-style
// token: claimCellFile stamps 1, adoptClaim bumps it by exactly 1 in the SAME
// atomic write as the ownership change, and renewClaimTTL/releaseClaim refuse
// typed CLAIM_FENCE_STALE when a caller presents an epoch BEHIND the stored
// one. Before this cell nothing in the Rust tree consumed the token at all —
// claim_cell_file stamped it and no code path ever compared it, so a stale
// holder's write would have silently proceeded. Fence semantics are
// safety-critical: a stale fence REFUSES, never silently proceeds.
//
// SECOND-PORT NOTE: verbs/state_group.rs carries a NARROWED adopt_claim (the
// `state handoff adopt` path: no typed codes, no `now` injection). That file
// is outside this cell's touchable set, so adoptClaim is re-derived here from
// claims.mjs rather than imported, and
// `adopt_agrees_with_the_state_group_port_on_the_shared_fixture` pins the two
// against one fixture so a future divergence fails a test.

/// claims.mjs `fail(code, reason, extra)` — the typed refusal shape every
/// claims mutator returns instead of throwing.
#[allow(dead_code)]
pub(crate) struct ClaimRefusal {
    pub code: &'static str,
    pub reason: String,
    pub extra: Map<String, Value>,
}

impl ClaimRefusal {
    fn new(code: &'static str, reason: String) -> Self {
        Self { code, reason, extra: Map::new() }
    }
}

#[allow(dead_code)]
pub(crate) enum AdoptClaimOutcome {
    Ok { claim: Value, previous_owner: Option<Value> },
    Refused(ClaimRefusal),
}

#[allow(dead_code)]
pub(crate) enum RenewClaimOutcome {
    Ok { renewed: Vec<String>, skipped: Vec<String> },
    Refused(ClaimRefusal),
}

#[allow(dead_code)]
pub(crate) enum ReleaseClaimOutcome {
    Ok { released: Value },
    Refused(ClaimRefusal),
}

/// `Number.isFinite(claim.fence_epoch) ? claim.fence_epoch : 1` — a legacy
/// claim written before msn-12 reads as epoch 1, the same default a fresh
/// claimCellFile stamps.
fn current_fence_epoch(claim: &Map<String, Value>) -> f64 {
    match claim.get("fence_epoch") {
        Some(Value::Number(n)) => match n.as_f64() {
            Some(v) if v.is_finite() => v,
            _ => 1.0,
        },
        _ => 1.0,
    }
}

/// The shared `!Number.isFinite(presentedEpoch) || presentedEpoch <
/// currentEpoch` guard behind renewClaimTTL and releaseClaim. `verb` is the
/// only difference between the two refusal texts ("renew" / "release").
fn claim_fence_refusal(
    verb: &str,
    cell: &str,
    presented: &Value,
    claim: &Map<String, Value>,
) -> Option<ClaimRefusal> {
    let current = current_fence_epoch(claim);
    let presented_num = match presented {
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        _ => f64::NAN, // Number.isFinite(non-number) === false
    };
    if presented_num.is_finite() && presented_num >= current {
        return None;
    }
    let mut refusal = ClaimRefusal::new(
        "CLAIM_FENCE_STALE",
        format!(
            "cell \"{cell}\" {verb} refused: presented epoch {} is behind current fence_epoch {} — a takeover already moved ownership forward; re-adopt before writing again.",
            jsjson::stringify(presented),
            jsjson::js_f64_to_string(current)
        ),
    );
    refusal.extra.insert("cell".into(), Value::String(cell.to_string()));
    refusal.extra.insert(
        "current_epoch".into(),
        Number::from_f64(current).map(Value::Number).unwrap_or(Value::Null),
    );
    refusal.extra.insert("presented_epoch".into(), presented.clone());
    Some(refusal)
}

/// claims.mjs adoptClaim — transfer ownership IN PLACE under the exclusive
/// gate: the claim file is atomically REWRITTEN, never deleted, so concurrent
/// 'wx' claimers keep getting EEXIST throughout the adoption. Every adoption
/// bumps `fence_epoch` by exactly 1 in the same atomic write as the ownership
/// change, which is what makes a stale holder's later renew/release
/// detectable at all.
///
/// Not called from a verb in THIS module: the one native caller today is
/// `state handoff adopt`, which lives in verbs/state_group.rs (owned by
/// another in-flight cell) and drives its own narrowed twin. This is the full
/// claims-module contract, byte-pinned against the live Node oracle.
#[allow(dead_code)]
pub(crate) fn adopt_claim(
    control: &Path,
    cell_id: &str,
    new_session_id: &str,
) -> MR<AdoptClaimOutcome> {
    let cell = require_id(cell_id, "cell id")?;
    let session = require_id(new_session_id, "session id")?;
    let _ = std::fs::create_dir_all(claims_dir(control));
    if !acquire_gate(control, &cell)? {
        return Ok(AdoptClaimOutcome::Refused(ClaimRefusal::new(
            "GATE_HELD",
            format!(
                "claim \"{cell}\" is gated by another in-flight adopt/sweep — retry later, never wait on the gate."
            ),
        )));
    }
    let outcome = (|| -> MR<AdoptClaimOutcome> {
        let Some(claim) = read_claim(control, &cell)? else {
            return Ok(AdoptClaimOutcome::Refused(ClaimRefusal::new(
                "NOT_FOUND",
                format!("cell \"{cell}\" has no claim to adopt."),
            )));
        };
        let previous = claim.get("session").cloned();
        let previous_epoch = current_fence_epoch(&claim);
        let now = utc_now();
        // `{...claim, session, claimed_at, adopted_from, adopted_at,
        // fence_epoch}` — a re-assigned key keeps its ORIGINAL position, and
        // `adopted_from: undefined` (no previous owner) is dropped wholesale
        // by JSON.stringify rather than written as null.
        let mut adopted = claim.clone();
        adopted.insert("session".into(), Value::String(session));
        adopted.insert("claimed_at".into(), Value::String(now.clone())); // fresh ownership renews the TTL clock
        match &previous {
            Some(prev) => {
                adopted.insert("adopted_from".into(), prev.clone());
            }
            None => {
                adopted.shift_remove("adopted_from");
            }
        }
        adopted.insert("adopted_at".into(), Value::String(now));
        adopted.insert(
            "fence_epoch".into(),
            Value::Number(Number::from_f64(previous_epoch + 1.0).ok_or(Fail::Delegate)?),
        );
        let adopted = Value::Object(adopted);
        let file = claim_path(control, &cell)?;
        transient_fs_retry(|| write_json_atomic(&file, &adopted))
            .map_err(|e| Fail::Thrown(format!("{e}")))?;
        Ok(AdoptClaimOutcome::Ok { claim: adopted, previous_owner: previous })
    })();
    release_gate(control, &cell); // `finally` — the gate is never leaked
    outcome
}

/// claims.mjs renewClaimTTL — same-session-only TTL renewal: refreshes
/// `claimed_at` for every claim owned by `session`, never touching
/// adopted_from/adopted_at and never bumping `fence_epoch`. A claim whose gate
/// is held by another in-flight adopt/sweep is SKIPPED, never waited on. The
/// session match is RE-VERIFIED under the gate, so a renewal racing an
/// adoption can never revert ownership.
///
/// `presented_epoch` is OPTIONAL and OFF by default (the shape every
/// production caller uses today). Presented and behind a reached claim's
/// current epoch, the WHOLE call refuses typed CLAIM_FENCE_STALE rather than
/// silently completing a partial renewal of the others.
///
/// Node's production caller is `heartbeatTouch` (claims.mjs), reached from
/// the bee-state-sync hook — whose own narrowed heartbeat path lives in
/// src/hooks/state_sync.rs, outside this cell's touchable set. This is the
/// full contract including the fencing arm that hook does not present.
#[allow(dead_code)]
pub(crate) fn renew_claim_ttl(
    control: &Path,
    session_id: &str,
    presented_epoch: Option<&Value>,
) -> MR<RenewClaimOutcome> {
    let session = require_id(session_id, "session id")?;
    let Ok(entries) = std::fs::read_dir(claims_dir(control)) else {
        return Ok(RenewClaimOutcome::Ok { renewed: Vec::new(), skipped: Vec::new() });
    };
    // readdirSync yields directory order; collect first so the gate work does
    // not hold the directory handle open (a Windows sharing hazard).
    let mut names: Vec<String> =
        entries.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.retain(|n| n.ends_with(".json"));
    let mut renewed = Vec::new();
    let mut skipped = Vec::new();
    for name in names {
        let cell = name[..name.len() - ".json".len()].to_string();
        // readClaim → claimPath → requireId: a file name that is not a plain
        // id throws in Node (uncaught, so the whole call throws) — mirrored.
        let preview = read_claim(control, &cell)?;
        match preview.as_ref().and_then(|c| c.get("session")) {
            Some(Value::String(s)) if *s == session => {}
            _ => continue, // not ours (or sessionless): never touched
        }
        if !acquire_gate(control, &cell)? {
            skipped.push(cell);
            continue;
        }
        let step = (|| -> MR<RenewStep> {
            let Some(claim) = read_claim(control, &cell)? else { return Ok(RenewStep::Untouched) };
            match claim.get("session") {
                Some(Value::String(s)) if *s == session => {}
                _ => return Ok(RenewStep::Untouched), // adopted away between listing and gating
            }
            if let Some(presented) = presented_epoch {
                if let Some(refusal) = claim_fence_refusal("renew", &cell, presented, &claim) {
                    return Ok(RenewStep::Refused(refusal));
                }
            }
            // The `...claim` spread carries acquired_at AND fence_epoch
            // forward untouched — only claimed_at, the expiry clock, advances.
            let mut next = claim.clone();
            next.insert("claimed_at".into(), Value::String(utc_now()));
            let file = claim_path(control, &cell)?;
            transient_fs_retry(|| write_json_atomic(&file, &Value::Object(next.clone())))
                .map_err(|e| Fail::Thrown(format!("{e}")))?;
            Ok(RenewStep::Renewed)
        })();
        release_gate(control, &cell); // `finally`, even on the fenced refusal
        match step? {
            RenewStep::Refused(refusal) => return Ok(RenewClaimOutcome::Refused(refusal)),
            RenewStep::Renewed => renewed.push(cell),
            RenewStep::Untouched => {}
        }
    }
    Ok(RenewClaimOutcome::Ok { renewed, skipped })
}

#[allow(dead_code)] // Refused is only constructed on the fencing arm
enum RenewStep {
    Renewed,
    Untouched,
    Refused(ClaimRefusal),
}

/// claims.mjs releaseClaim — owner-matched removal under the same exclusive
/// gate as adopt/sweep, with the msn-12 fencing guard checked AFTER the owner
/// check (epoch fencing is an additional, orthogonal guard, never a
/// substitute for ownership). A fenced refusal leaves the claim file
/// untouched.
pub(crate) fn release_claim_typed(
    control: &Path,
    session: Option<&str>,
    cell_id: &str,
    presented_epoch: Option<&Value>,
) -> MR<ReleaseClaimOutcome> {
    let cell = require_id(cell_id, "cell id")?;
    let not_found = || {
        ReleaseClaimOutcome::Refused(ClaimRefusal::new(
            "NOT_FOUND",
            format!("cell \"{cell}\" has no claim to release."),
        ))
    };
    if read_claim(control, &cell)?.is_none() {
        return Ok(not_found());
    }
    if !acquire_gate_with_retry(control, &cell)? {
        return Ok(ReleaseClaimOutcome::Refused(ClaimRefusal::new(
            "GATE_HELD",
            format!(
                "claim \"{cell}\" is gated by another in-flight adopt/sweep/release after {GATE_RETRY_ATTEMPTS} bounded retries — never waited unboundedly."
            ),
        )));
    }
    let outcome = (|| -> MR<ReleaseClaimOutcome> {
        let Some(claim) = read_claim(control, &cell)? else { return Ok(not_found()) };
        let owner: Option<String> = match claim.get("session") {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Null) | None => None,
            Some(_) => return Err(Fail::Delegate), // non-string session — JS-exotic compare
        };
        if owner.as_deref() != session {
            return Ok(ReleaseClaimOutcome::Refused(ClaimRefusal::new(
                "NOT_OWNER",
                format!(
                    "cell \"{cell}\" is owned by session \"{}\", not \"{}\".",
                    owner.as_deref().unwrap_or("none (sessionless)"),
                    session.unwrap_or("none (sessionless)")
                ),
            )));
        }
        if let Some(presented) = presented_epoch {
            if let Some(refusal) = claim_fence_refusal("release", &cell, presented, &claim) {
                return Ok(ReleaseClaimOutcome::Refused(refusal));
            }
        }
        let file = claim_path(control, &cell)?;
        let _ = transient_fs_retry(|| match std::fs::remove_file(&file) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        });
        Ok(ReleaseClaimOutcome::Ok { released: Value::Object(claim) })
    })();
    release_gate(control, &cell);
    outcome
}

/// The claim-unwind path of claimCellCrossSession: the caller ignores the
/// typed result (only the disk effect must match Node), and never fences.
fn release_claim(control: &Path, session: Option<&str>, cell_id: &str) -> MR<()> {
    release_claim_typed(control, session, cell_id, None).map(|_| ())
}

// ─── sessions (claims.mjs) ─────────────────────────────────────────────────

/// claims.mjs readSession (fail-open flavor): malformed id -> None; corrupt
/// record -> warn + None (readJson's fallback); id-mismatch -> None.
fn read_session(control: &Path, session_id: &str) -> MR<Option<Map<String, Value>>> {
    let trimmed = js_trim(session_id);
    if trimmed.is_empty() || trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Ok(None); // sessionPath's requireId throw, caught -> null
    }
    let file = sessions_dir(control).join(format!("{trimmed}.json"));
    match read_store_json(&file)? {
        None => Ok(None),
        Some(Value::Object(m)) => {
            if matches!(m.get("id"), Some(Value::String(s)) if s == trimmed) {
                Ok(Some(m))
            } else {
                Ok(None)
            }
        }
        Some(Value::Array(_)) => Err(Fail::Delegate), // typeof 'object' — JS-exotic
        Some(_) => Ok(None),
    }
}

/// claims.mjs listSessionRecords (fail-open): missing dir -> empty.
fn list_session_records(control: &Path) -> MR<Vec<Map<String, Value>>> {
    let dir = sessions_dir(control);
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .collect();
    // fs.readdirSync returns names sorted per OS; both runtimes read the same
    // order — only the record set matters to every consumer here.
    names.sort();
    let mut out = Vec::new();
    for name in names {
        if !name.ends_with(".json") {
            continue;
        }
        let stem = &name[..name.len() - ".json".len()];
        if let Some(record) = read_session(control, stem)? {
            out.push(record);
        }
    }
    Ok(out)
}

/// claims.mjs heartbeatStale.
fn heartbeat_stale(session: Option<&Map<String, Value>>, now: f64) -> MR<bool> {
    let Some(session) = session else { return Ok(true) };
    match rsv::date_parse_val(session.get("last_heartbeat")).map_err(|_| Fail::Delegate)? {
        None => Ok(true),
        Some(ms) => Ok(ms + HEARTBEAT_STALE_SECONDS * 1000.0 <= now),
    }
}

/// claims.mjs isConcurrentMode (no exclusion).
fn is_concurrent_mode(control: &Path) -> MR<bool> {
    let now = rsv::now_ms();
    for record in list_session_records(control)? {
        if !heartbeat_stale(Some(&record), now)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn env_nonempty(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) if !js_trim(&v).is_empty() => Some(js_trim(&v).to_string()),
        _ => None,
    }
}

/// claims.mjs resolveSessionId flag -> BEE_SESSION_ID -> CLAUDE_CODE_SESSION_ID
/// (no `root`: the durable single-live-session fallback stays with callers
/// that pass one).
fn resolve_session_flag_env(flag: Option<&str>) -> Option<String> {
    if let Some(f) = flag {
        if !js_trim(f).is_empty() {
            return Some(js_trim(f).to_string());
        }
    }
    env_nonempty("BEE_SESSION_ID").or_else(|| env_nonempty("CLAUDE_CODE_SESSION_ID"))
}

/// resolveSessionId's durable fallback half ({root, audit}) — exactly one
/// fresh live session record adopts; anything else resolves None.
fn resolve_session_adopt(control: &Path) -> MR<Option<String>> {
    let now = rsv::now_ms();
    let mut fresh: Vec<Map<String, Value>> = Vec::new();
    for record in list_session_records(control)? {
        if !heartbeat_stale(Some(&record), now)? {
            fresh.push(record);
        }
    }
    if fresh.len() == 1 {
        if let Some(Value::String(id)) = fresh[0].get("id") {
            return Ok(Some(id.clone()));
        }
    }
    Ok(None)
}

/// claimCellFile's typed outcome. The Ok claim payload mirrors Node's
/// {ok:true, claim} return; the CLI handler never reads it (tests do).
#[allow(dead_code)]
enum ClaimFileOutcome {
    Ok { claim: Value },
    Refused { code: &'static str, reason: String },
}

/// claims.mjs claimCellFile — the O_EXCL one-winner primitive: fence_epoch
/// stamped 1 at creation, acquired_at == claimed_at, session/workspace_id/
/// adopted omitted (never null) when absent.
fn claim_cell_file(
    control: &Path,
    session_in: Option<&str>,
    cell_id: &str,
    ttl: Option<f64>,
) -> MR<ClaimFileOutcome> {
    let explicit = match session_in {
        None => None,
        Some(s) => Some(require_id(s, "session id")?),
    };
    let cell = require_id(cell_id, "cell id")?;
    let mut session = explicit;
    let mut adopted = false;
    if session.is_none() {
        if let Some(candidate) = resolve_session_adopt(control)? {
            session = Some(candidate);
            adopted = true;
        } else if is_concurrent_mode(control)? {
            return Ok(ClaimFileOutcome::Refused {
                code: "SESSION_REQUIRED",
                reason: format!(
                    "cell \"{cell}\" cannot be claimed without identifying the acting session while another session is active — pass --session-id or set BEE_SESSION_ID (CLAUDE_CODE_SESSION_ID is also honored)."
                ),
            });
        }
    }
    let _ = std::fs::create_dir_all(claims_dir(control));
    // msn-19: workspace_id auto-looked-up from the acting session's record.
    let mut workspace_id: Option<String> = None;
    if let Some(s) = &session {
        if let Some(record) = read_session(control, s)? {
            if let Some(Value::String(w)) = record.get("workspace_id") {
                if !w.is_empty() {
                    workspace_id = Some(w.clone());
                }
            }
        }
    }
    let mut claim = Map::new();
    claim.insert("cell".into(), Value::String(cell.clone()));
    if let Some(s) = &session {
        claim.insert("session".into(), Value::String(s.clone()));
    }
    if let Some(w) = &workspace_id {
        claim.insert("workspace_id".into(), Value::String(w.clone()));
    }
    let ttl_value = match ttl {
        Some(t) if t.is_finite() && t > 0.0 => t.floor(),
        _ => DEFAULT_CLAIM_TTL_SECONDS,
    };
    claim.insert(
        "ttl_seconds".into(),
        Value::Number(Number::from_f64(ttl_value).ok_or(Fail::Delegate)?),
    );
    let now = utc_now();
    claim.insert("claimed_at".into(), Value::String(now.clone()));
    claim.insert("acquired_at".into(), Value::String(now));
    claim.insert("fence_epoch".into(), Value::Number(Number::from_f64(1.0).unwrap()));
    if adopted {
        claim.insert("adopted".into(), Value::Bool(true));
    }
    let claim = Value::Object(claim);
    let file = claim_path(control, &cell)?;
    let body = format!("{}\n", jsjson::stringify_pretty(&claim));
    let write = std::fs::OpenOptions::new().write(true).create_new(true).open(&file).and_then(|mut f| {
        use std::io::Write;
        f.write_all(body.as_bytes())
    });
    match write {
        Ok(()) => Ok(ClaimFileOutcome::Ok { claim }),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let holder = read_claim(control, &cell)?;
            let owner = match holder.as_ref().and_then(|h| h.get("session")) {
                Some(Value::Null) | None => "no session (sessionless claim)".to_string(),
                Some(v) => jsjson::js_to_string(v),
            };
            Ok(ClaimFileOutcome::Refused {
                code: "CLAIMED",
                reason: format!(
                    "cell \"{cell}\" is already claimed by session \"{owner}\" ({}).",
                    claim_expiry(holder.as_ref())?
                ),
            })
        }
        Err(e) => Err(Fail::Thrown(format!("{e}"))), // Node rethrows raw fs errors
    }
}

// ─── trace helpers (lib/cells.mjs) ─────────────────────────────────────────

/// lib/cells.mjs defaultTrace() — key order is load-bearing (JSON bytes).
fn default_trace() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("worker".into(), Value::Null);
    m.insert("outcome".into(), Value::Null);
    m.insert("files_changed".into(), Value::Array(vec![]));
    m.insert("deviations".into(), Value::Array(vec![]));
    m.insert("friction".into(), Value::Null);
    m.insert("capped_at".into(), Value::Null);
    m.insert("behavior_change".into(), Value::Bool(false));
    m.insert("verification_evidence".into(), Value::Null);
    m.insert("verify_output".into(), Value::Null);
    m.insert("verify_passed".into(), Value::Null);
    m.insert("claim_session".into(), Value::Null);
    m
}

/// JS object-spread assignment: existing keys keep their position, new keys
/// append — serde_json's preserve_order Map::insert has exactly this shape.
fn spread_into(base: &mut Map<String, Value>, overlay: &Map<String, Value>) {
    for (k, v) in overlay {
        base.insert(k.clone(), v.clone());
    }
}

/// `{...defaultTrace(), ...(cell.trace || {})}` — the merge every mutator
/// opens with. Falsy trace -> defaults; plain object -> overlay; a truthy
/// number/bool spreads nothing ({...5} === {}); a non-empty string or array
/// spreads index keys (JS-exotic) -> Delegate.
fn merge_trace(trace: Option<&Value>) -> MR<Map<String, Value>> {
    let mut base = default_trace();
    match trace {
        None | Some(Value::Null) | Some(Value::Bool(false)) => {}
        Some(Value::String(s)) if s.is_empty() => {}
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => {}
        Some(Value::Object(m)) => spread_into(&mut base, m),
        Some(Value::Array(a)) if a.is_empty() => {}
        Some(Value::Number(_)) | Some(Value::Bool(true)) => {}
        Some(_) => return Err(Fail::Delegate), // string/array index spread — JS-exotic
    }
    Ok(base)
}

/// lib/cells.mjs releaseTrace: clear claim + legacy verify evidence, keep the
/// rest (assignment order pins where absent keys append).
fn release_trace(mut trace: Map<String, Value>) -> Map<String, Value> {
    for key in [
        "worker",
        "claimed_at",
        "claim_session",
        "verify_command",
        "verify_output",
        "verify_passed",
        "verified_at",
    ] {
        trace.insert(key.into(), Value::Null);
    }
    trace
}

/// lib/cells.mjs appendAttempt — one revision-ledger row, claim identity read
/// from the LIVE control-plane claim file.
fn append_attempt(
    root: &Path,
    id: &str,
    mut trace: Map<String, Value>,
    verdict: &str,
    failure_signature: Option<String>,
    note: Option<&str>,
) -> MR<Map<String, Value>> {
    let attempts: Vec<Value> = match trace.get("attempts") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let control = control_root(root)?;
    let claim = read_claim(&control, id)?;
    let claim_str = |key: &str| -> Value {
        match claim.as_ref().and_then(|c| c.get(key)) {
            Some(Value::String(s)) => Value::String(s.clone()),
            _ => Value::Null,
        }
    };
    let acquired_at = match claim.as_ref().and_then(|c| c.get("acquired_at")) {
        Some(Value::String(s)) => Value::String(s.clone()),
        _ => claim_str("claimed_at"), // legacy claims: fall back to claimed_at
    };
    let worker = match trace.get("worker") {
        Some(Value::String(s)) => Value::String(s.clone()),
        _ => Value::Null,
    };
    let mut entry = Map::new();
    entry.insert(
        "n".into(),
        Value::Number(Number::from_f64((attempts.len() + 1) as f64).unwrap()),
    );
    entry.insert("at".into(), Value::String(utc_now()));
    entry.insert("claim_session".into(), claim_str("session"));
    entry.insert("claimed_at".into(), claim_str("claimed_at"));
    entry.insert("acquired_at".into(), acquired_at);
    entry.insert("worker".into(), worker);
    entry.insert("verdict".into(), Value::String(verdict.to_string()));
    entry.insert(
        "failure_signature".into(),
        failure_signature.map(Value::String).unwrap_or(Value::Null),
    );
    entry.insert("note".into(), note.map(|n| Value::String(n.to_string())).unwrap_or(Value::Null));
    let mut next = attempts;
    next.push(Value::Object(entry));
    trace.insert("attempts".into(), Value::Array(next));
    Ok(trace)
}

/// lib/cells.mjs checkClaimOwnership (D4/msh-4).
struct Ownership {
    ok: bool,
    reason: String,
    holder: Option<Value>,
}

fn check_claim_ownership(root: &Path, id: &str, session_flag: Option<&str>) -> MR<Ownership> {
    let control = control_root(root)?;
    let claim = read_claim(&control, id)?;
    let now = rsv::now_ms();
    if !claim_active(claim.as_ref(), now)? {
        return Ok(Ownership { ok: true, reason: String::new(), holder: None });
    }
    let claim = claim.unwrap();
    let owner = match claim.get("session") {
        Some(v) if js_truthy(v) => v.clone(),
        _ => return Ok(Ownership { ok: true, reason: String::new(), holder: None }),
    };
    let caller = resolve_session_flag_env(session_flag);
    // `caller === owner` — strict; a non-string owner can never equal a
    // string caller, and a null caller (undefined session) never equals it.
    if let (Some(caller), Value::String(owner_s)) = (&caller, &owner) {
        if caller == owner_s {
            return Ok(Ownership { ok: true, reason: String::new(), holder: None });
        }
    }
    let owner_disp = jsjson::js_to_string(&owner);
    Ok(Ownership {
        ok: false,
        reason: format!(
            "cell \"{id}\" is claimed by session \"{owner_disp}\" ({}) — another session owns it. Pass --force-ownership to override (audited).",
            claim_expiry(Some(&claim))?
        ),
        holder: Some(owner),
    })
}

/// lib/cells.mjs appendOwnershipOverride (D4 Δ5).
fn append_ownership_override(
    mut trace: Map<String, Value>,
    verb: &str,
    session_flag: Option<&str>,
    ownership: &Ownership,
) -> Map<String, Value> {
    let overrides: Vec<Value> = match trace.get("ownership_overrides") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let mut entry = Map::new();
    entry.insert("verb".into(), Value::String(verb.to_string()));
    entry.insert(
        "forced_by".into(),
        resolve_session_flag_env(session_flag).map(Value::String).unwrap_or(Value::Null),
    );
    entry.insert(
        "owner_bypassed".into(),
        if ownership.ok { Value::Null } else { ownership.holder.clone().unwrap_or(Value::Null) },
    );
    entry.insert("at".into(), Value::String(utc_now()));
    let mut next = overrides;
    next.push(Value::Object(entry));
    trace.insert("ownership_overrides".into(), Value::Array(next));
    trace
}

/// lib/cells.mjs guardClaimOwnership.
fn guard_claim_ownership(
    root: &Path,
    id: &str,
    trace: Map<String, Value>,
    verb: &str,
    session_flag: Option<&str>,
    force: bool,
) -> MR<Map<String, Value>> {
    let ownership = check_claim_ownership(root, id, session_flag)?;
    if !force {
        if !ownership.ok {
            return Err(Fail::Thrown(format!("{verb}: {}", ownership.reason)));
        }
        return Ok(trace);
    }
    Ok(append_ownership_override(trace, verb, session_flag, &ownership))
}

// ─── failure-signature normalizer (lib/cells.mjs D1) ───────────────────────
// Hand-rolled ports of the four scrub regexes + the failing-line pick; the
// sha256[..12] value lands in cell files, so byte-exact parity with V8's
// regex semantics is load-bearing (pinned by Node-computed test vectors).

fn is_ascii_word(c: char) -> bool {
    is_word_char(c) // JS \w without the u flag: [A-Za-z0-9_]
}

/// /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?/g -> "<ts>"
fn scrub_iso_timestamps(chars: &[char]) -> Vec<char> {
    let d = |i: usize| i < chars.len() && chars[i].is_ascii_digit();
    let lit = |i: usize, c: char| i < chars.len() && chars[i] == c;
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        let m = (|| -> Option<usize> {
            let mut j = i;
            for _ in 0..4 {
                if !d(j) {
                    return None;
                }
                j += 1;
            }
            if !lit(j, '-') {
                return None;
            }
            j += 1;
            for _ in 0..2 {
                if !d(j) {
                    return None;
                }
                j += 1;
            }
            if !lit(j, '-') {
                return None;
            }
            j += 1;
            for _ in 0..2 {
                if !d(j) {
                    return None;
                }
                j += 1;
            }
            if !lit(j, 'T') {
                return None;
            }
            j += 1;
            for block in 0..3 {
                for _ in 0..2 {
                    if !d(j) {
                        return None;
                    }
                    j += 1;
                }
                if block < 2 {
                    if !lit(j, ':') {
                        return None;
                    }
                    j += 1;
                }
            }
            if lit(j, '.') && d(j + 1) {
                j += 1;
                while d(j) {
                    j += 1;
                }
            }
            if lit(j, 'Z') {
                j += 1;
            }
            Some(j)
        })();
        match m {
            Some(end) => {
                out.extend("<ts>".chars());
                i = end;
            }
            None => {
                out.push(chars[i]);
                i += 1;
            }
        }
    }
    out
}

/// /[A-Za-z]:\\[^\s"'<>]*/g -> "<path>"
fn scrub_win_paths(chars: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].is_ascii_alphabetic()
            && i + 2 < chars.len()
            && chars[i + 1] == ':'
            && chars[i + 2] == '\\'
        {
            let mut j = i + 3;
            while j < chars.len() {
                let c = chars[j];
                if rsv::js_is_ws(c) || c == '"' || c == '\'' || c == '<' || c == '>' {
                    break;
                }
                j += 1;
            }
            out.extend("<path>".chars());
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// /\/(?:[\w.-]+\/)+[\w.-]*/g -> "<path>" (two segments minimum).
fn scrub_unix_paths(chars: &[char]) -> Vec<char> {
    let seg = |c: char| is_ascii_word(c) || c == '.' || c == '-';
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '/' {
            let mut j = i + 1;
            let mut groups = 0usize;
            loop {
                let mut k = j;
                while k < chars.len() && seg(chars[k]) {
                    k += 1;
                }
                if k > j && k < chars.len() && chars[k] == '/' {
                    groups += 1;
                    j = k + 1;
                } else {
                    // trailing [\w.-]* from j
                    if groups > 0 {
                        j = k;
                    }
                    break;
                }
            }
            if groups > 0 {
                out.extend("<path>".chars());
                i = j;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// /\b[0-9a-fA-F]{6,}\b/g -> "<hex>" — equivalently: any maximal \w run made
/// solely of hex digits, length >= 6 (boundaries cannot fall inside a run).
fn scrub_hex_runs(chars: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0usize;
    while i < chars.len() {
        if is_ascii_word(chars[i]) {
            let mut j = i;
            while j < chars.len() && is_ascii_word(chars[j]) {
                j += 1;
            }
            let run = &chars[i..j];
            if run.len() >= 6 && run.iter().all(|c| c.is_ascii_hexdigit()) {
                out.extend("<hex>".chars());
            } else {
                out.extend_from_slice(run);
            }
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

const FAILING_LINE_NEEDLES: [&str; 4] = ["fail", "error", "refus", "denied"];

fn is_failing_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    FAILING_LINE_NEEDLES.iter().any(|n| lower.contains(n))
}

/// lib/cells.mjs normalizeFailureSignature — sha256(chosen line)[..12].
fn normalize_failure_signature(output: &str) -> String {
    let chars: Vec<char> = output.chars().collect();
    let scrubbed: String = scrub_hex_runs(&scrub_unix_paths(&scrub_win_paths(&scrub_iso_timestamps(
        &chars,
    ))))
    .into_iter()
    .collect();
    let lines: Vec<&str> = scrubbed.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    let trimmed: Vec<&str> = lines.iter().map(|l| js_trim(l)).collect();
    let chosen = trimmed
        .iter()
        .find(|l| !l.is_empty() && is_failing_line(l))
        .or_else(|| trimmed.iter().find(|l| !l.is_empty()))
        .copied()
        .unwrap_or("");
    let mut hasher = Sha256::new();
    hasher.update(chosen.as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    hex[..12].to_string()
}

// ─── decisions.mjs write-time safety patterns (exact matchers) ─────────────
// The refusal message embeds the JS regex literal's own toString, so both
// the DETECTION and the PATTERN TEXT are pinned here.

fn ci_starts_with(chars: &[char], i: usize, needle: &str) -> bool {
    let n: Vec<char> = needle.chars().collect();
    if i + n.len() > chars.len() {
        return false;
    }
    n.iter().enumerate().all(|(k, c)| chars[i + k].to_ascii_lowercase() == *c)
}

fn cs_starts_with(chars: &[char], i: usize, needle: &str) -> bool {
    let n: Vec<char> = needle.chars().collect();
    if i + n.len() > chars.len() {
        return false;
    }
    n.iter().enumerate().all(|(k, c)| chars[i + k] == *c)
}

fn ws_run(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && rsv::js_is_ws(chars[i]) {
        i += 1;
    }
    i
}

fn word_boundary_before(chars: &[char], i: usize) -> bool {
    i == 0 || !is_ascii_word(chars[i - 1])
}

fn word_at(chars: &[char], i: usize) -> bool {
    i < chars.len() && is_ascii_word(chars[i])
}

/// First SECRET_CONTENT_PATTERNS hit, as the JS regex literal string.
fn find_secret_pattern(text: &str) -> Option<&'static str> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    // /-----BEGIN [A-Z ]*PRIVATE KEY-----/
    for i in 0..len {
        if cs_starts_with(&chars, i, "-----BEGIN ") {
            let p = i + 11;
            let mut r = p;
            while r < len && (chars[r] == ' ' || chars[r].is_ascii_uppercase()) {
                r += 1;
            }
            for s in p..=r {
                if cs_starts_with(&chars, s, "PRIVATE KEY-----") {
                    return Some("/-----BEGIN [A-Z ]*PRIVATE KEY-----/");
                }
            }
        }
    }
    // /\bAKIA[0-9A-Z]{16}\b/
    for i in 0..len {
        if word_boundary_before(&chars, i) && cs_starts_with(&chars, i, "AKIA") {
            let p = i + 4;
            if p + 16 <= len
                && chars[p..p + 16].iter().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
                && !word_at(&chars, p + 16)
            {
                return Some("/\\bAKIA[0-9A-Z]{16}\\b/");
            }
        }
    }
    // /\bghp_[A-Za-z0-9]{20,}\b/
    for i in 0..len {
        if word_boundary_before(&chars, i) && cs_starts_with(&chars, i, "ghp_") {
            let p = i + 4;
            let mut r = p;
            while r < len && chars[r].is_ascii_alphanumeric() {
                r += 1;
            }
            if r - p >= 20 && !word_at(&chars, r) {
                return Some("/\\bghp_[A-Za-z0-9]{20,}\\b/");
            }
        }
    }
    // /\bsk-[A-Za-z0-9_-]{20,}\b/
    let sk_class = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    for i in 0..len {
        if word_boundary_before(&chars, i) && cs_starts_with(&chars, i, "sk-") {
            let p = i + 3;
            let mut r = p;
            while r < len && sk_class(chars[r]) {
                r += 1;
            }
            let run = r - p;
            if run >= 20 {
                for k in (20..=run).rev() {
                    let last_word = is_ascii_word(chars[p + k - 1]);
                    let next_word = word_at(&chars, p + k);
                    if last_word != next_word {
                        return Some("/\\bsk-[A-Za-z0-9_-]{20,}\\b/");
                    }
                }
            }
        }
    }
    // /\beyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}/
    for i in 0..len {
        if word_boundary_before(&chars, i) && cs_starts_with(&chars, i, "eyJ") {
            let p = i + 3;
            let mut r = p;
            while r < len && sk_class(chars[r]) {
                r += 1;
            }
            if r - p >= 20 && r < len && chars[r] == '.' {
                let q = r + 1;
                let mut r2 = q;
                while r2 < len && sk_class(chars[r2]) {
                    r2 += 1;
                }
                if r2 - q >= 10 {
                    return Some("/\\beyJ[A-Za-z0-9_-]{20,}\\.[A-Za-z0-9_-]{10,}/");
                }
            }
        }
    }
    // /\b(?:api[_-]?key|secret|token|password|passwd)\s*[:=]\s*['"]?[^\s'"]{6,}/i
    const KEYWORDS: [&str; 7] = ["api_key", "api-key", "apikey", "secret", "token", "password", "passwd"];
    for i in 0..len {
        if !word_boundary_before(&chars, i) {
            continue;
        }
        for kw in KEYWORDS {
            if !ci_starts_with(&chars, i, kw) {
                continue;
            }
            let mut j = ws_run(&chars, i + kw.chars().count());
            if !(j < len && (chars[j] == ':' || chars[j] == '=')) {
                continue;
            }
            j = ws_run(&chars, j + 1);
            if j < len && (chars[j] == '\'' || chars[j] == '"') {
                j += 1;
            }
            let mut r = j;
            while r < len && !rsv::js_is_ws(chars[r]) && chars[r] != '\'' && chars[r] != '"' {
                r += 1;
            }
            if r - j >= 6 {
                return Some("/\\b(?:api[_-]?key|secret|token|password|passwd)\\s*[:=]\\s*['\"]?[^\\s'\"]{6,}/i");
            }
        }
    }
    None
}

/// First INJECTION_PATTERNS hit, as the JS regex literal string.
fn find_injection_pattern(text: &str) -> Option<&'static str> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let qualifiers = ["previous", "prior", "above", "earlier"];
    let terminals = ["instructions", "messages", "context", "prompt"];
    let ws1 = |i: usize| -> Option<usize> {
        let j = ws_run(&chars, i);
        if j > i {
            Some(j)
        } else {
            None
        }
    };
    let match_alt = |i: usize, alts: &[&str]| -> Option<usize> {
        for alt in alts {
            if ci_starts_with(&chars, i, alt) {
                return Some(i + alt.chars().count());
            }
        }
        None
    };
    // /ignore\s+(?:all\s+)?(?:previous|prior|above|earlier)\s+(?:instructions|messages|context|prompts?)/i
    for i in 0..len {
        if !ci_starts_with(&chars, i, "ignore") {
            continue;
        }
        let Some(j) = ws1(i + 6) else { continue };
        let starts = if ci_starts_with(&chars, j, "all") {
            match ws1(j + 3) {
                Some(k) => vec![j, k],
                None => vec![j],
            }
        } else {
            vec![j]
        };
        for start in starts {
            let Some(q) = match_alt(start, &qualifiers) else { continue };
            let Some(w) = ws1(q) else { continue };
            if match_alt(w, &terminals).is_some() {
                return Some("/ignore\\s+(?:all\\s+)?(?:previous|prior|above|earlier)\\s+(?:instructions|messages|context|prompts?)/i");
            }
        }
    }
    // /disregard\s+(?:all\s+)?(?:previous|prior|above|earlier)/i
    for i in 0..len {
        if !ci_starts_with(&chars, i, "disregard") {
            continue;
        }
        let Some(j) = ws1(i + 9) else { continue };
        let starts = if ci_starts_with(&chars, j, "all") {
            match ws1(j + 3) {
                Some(k) => vec![j, k],
                None => vec![j],
            }
        } else {
            vec![j]
        };
        for start in starts {
            if match_alt(start, &qualifiers).is_some() {
                return Some("/disregard\\s+(?:all\\s+)?(?:previous|prior|above|earlier)/i");
            }
        }
    }
    // /<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/i
    let tags = ["system", "assistant", "user", "developer", "tool"];
    for i in 0..len {
        if chars[i] != '<' {
            continue;
        }
        let mut j = i + 1;
        if j < len && chars[j] == '/' {
            j += 1;
        }
        j = ws_run(&chars, j);
        let Some(k) = match_alt(j, &tags) else { continue };
        if word_at(&chars, k) {
            continue; // \b after the tag name
        }
        let mut m = k;
        while m < len && chars[m] != '>' {
            m += 1;
        }
        if m < len {
            return Some("/<\\/?\\s*(?:system|assistant|user|developer|tool)\\b[^>]*>/i");
        }
    }
    // /\[\s*(?:system|assistant|user|developer)\s*\]/i
    let btags = ["system", "assistant", "user", "developer"];
    for i in 0..len {
        if chars[i] != '[' {
            continue;
        }
        let j = ws_run(&chars, i + 1);
        let Some(k) = match_alt(j, &btags) else { continue };
        let m = ws_run(&chars, k);
        if m < len && chars[m] == ']' {
            return Some("/\\[\\s*(?:system|assistant|user|developer)\\s*\\]/i");
        }
    }
    None
}

/// decisions.mjs assertSafe over the logDecision field set.
fn assert_safe_decision_fields(fields: &[(&str, Option<&str>)]) -> MR<()> {
    for (field, value) in fields {
        let Some(value) = value else { continue }; // typeof !== 'string' skip
        if value.is_empty() {
            continue;
        }
        if let Some(pattern) = find_secret_pattern(value) {
            return Err(Fail::Thrown(format!(
                "Decision rejected: field \"{field}\" matches a secret pattern ({pattern}). Never log credentials — describe the decision without the secret."
            )));
        }
        if let Some(pattern) = find_injection_pattern(value) {
            return Err(Fail::Thrown(format!(
                "Decision rejected: field \"{field}\" contains instruction-like content ({pattern}). Decision text must be data, not instructions."
            )));
        }
    }
    Ok(())
}

// ─── decisions.mjs logDecision (the audit rows cells verbs write) ──────────

const DECISIONS_LOCK_NAME: &str = "decisions";

fn decisions_path(root: &Path) -> PathBuf {
    root.join(".bee").join("decisions.jsonl")
}

fn taxonomy_path(root: &Path) -> PathBuf {
    root.join("docs").join("decisions").join("taxonomy.json")
}

struct Taxonomy {
    schema_version: Value,
    tags: Vec<Value>,
    candidates: Vec<String>,
}

/// decisions.mjs loadTaxonomy — readJson-backed (corrupt -> warn + the same
/// absent-file fallback).
fn load_taxonomy(root: &Path) -> MR<Option<Taxonomy>> {
    match read_store_json(&taxonomy_path(root))? {
        None => Ok(None),
        Some(Value::Object(raw)) => {
            let tags = match raw.get("tags") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let candidates = match raw.get("candidates") {
                Some(Value::Array(a)) => a
                    .iter()
                    .filter_map(|c| match c {
                        Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let schema_version = match raw.get("schema_version") {
                Some(Value::Null) | None => json!(1.0),
                Some(v) => v.clone(),
            };
            Ok(Some(Taxonomy { schema_version, tags, candidates }))
        }
        Some(_) => Ok(None),
    }
}

/// decisions.mjs withDecisionsLockSync — bounded 15 x 20ms retry, typed
/// DecisionsLockBusyError on exhaustion.
fn with_decisions_lock<T>(root: &Path, f: impl FnOnce() -> MR<T>) -> MR<T> {
    let mut attempt = 0u32;
    loop {
        match lock::acquire_store_lock_once(root, DECISIONS_LOCK_NAME) {
            lock::AcquireOnce::Acquired(mut guard) => {
                let out = f();
                guard.release();
                return out;
            }
            lock::AcquireOnce::Busy { holder } => {
                attempt += 1;
                if attempt > GATE_RETRY_ATTEMPTS - 1 {
                    return Err(Fail::Thrown(format!(
                        "decisions store lock \"{DECISIONS_LOCK_NAME}\" busy: held by {}",
                        holder_who(&holder)
                    )));
                }
                std::thread::sleep(std::time::Duration::from_millis(GATE_RETRY_DELAY_MS));
            }
        }
    }
}

/// decisions.mjs classifyDecisionTags + appendTaxonomyCandidatesSync.
fn classify_decision_tags(root: &Path, tags: &[String]) -> MR<()> {
    let Some(taxonomy) = load_taxonomy(root)? else { return Ok(()) };
    if tags.is_empty() {
        return Err(Fail::Thrown(
            "decisions: docs/decisions/taxonomy.json exists — this decision event needs at least one tag. Pass --tags (e.g. \"billing,recall\").".into(),
        ));
    }
    let mut known: Vec<String> = taxonomy
        .tags
        .iter()
        .filter_map(|t| match t.get("name") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    known.extend(taxonomy.candidates.iter().cloned());
    let unknown: Vec<String> = tags.iter().filter(|t| !known.contains(t)).cloned().collect();
    if unknown.is_empty() {
        return Ok(());
    }
    with_decisions_lock(root, || {
        let Some(fresh) = load_taxonomy(root)? else { return Ok(()) };
        let mut fresh_known: Vec<String> = fresh
            .tags
            .iter()
            .filter_map(|t| match t.get("name") {
                Some(Value::String(s)) => Some(s.clone()),
                _ => None,
            })
            .collect();
        fresh_known.extend(fresh.candidates.iter().cloned());
        let mut next = fresh.candidates.clone();
        for tag in &unknown {
            if !fresh_known.contains(tag) && !next.contains(tag) {
                next.push(tag.clone());
            }
        }
        if next.len() != fresh.candidates.len() {
            let mut body = Map::new();
            body.insert("schema_version".into(), fresh.schema_version.clone());
            body.insert("tags".into(), Value::Array(fresh.tags.clone()));
            body.insert(
                "candidates".into(),
                Value::Array(next.into_iter().map(Value::String).collect()),
            );
            write_json_atomic(&taxonomy_path(root), &Value::Object(body))
                .map_err(|e| Fail::Thrown(format!("{e}")))?;
        }
        Ok(())
    })
}

/// decisions.mjs logDecision — the exact event shape/append the cells verbs
/// produce (alternatives/confidence always null here; scope 'repo', source
/// 'user', matching every cells.mjs call site).
fn log_decision(root: &Path, decision: &str, rationale: &str, tags: &[&str]) -> MR<()> {
    if js_trim(decision).is_empty() {
        return Err(Fail::Thrown("logDecision: decision text is required.".into()));
    }
    if js_trim(rationale).is_empty() {
        return Err(Fail::Thrown("logDecision: rationale is required.".into()));
    }
    assert_safe_decision_fields(&[
        ("decision", Some(decision)),
        ("rationale", Some(rationale)),
        ("alternatives", None),
        ("scope", Some("repo")),
        ("source", Some("user")),
    ])?;
    let normalized: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
    classify_decision_tags(root, &normalized)?;
    let mut event = Map::new();
    event.insert("id".into(), Value::String(rsv::pseudo_uuid_v4()));
    event.insert("type".into(), Value::String("decide".into()));
    event.insert("date".into(), Value::String(utc_now()));
    event.insert("decision".into(), Value::String(js_trim(decision).to_string()));
    event.insert("rationale".into(), Value::String(js_trim(rationale).to_string()));
    event.insert("alternatives".into(), Value::Null);
    event.insert("scope".into(), Value::String("repo".into()));
    event.insert("source".into(), Value::String("user".into()));
    event.insert("confidence".into(), Value::Null);
    if !normalized.is_empty() {
        event.insert(
            "tags".into(),
            Value::Array(normalized.into_iter().map(Value::String).collect()),
        );
    }
    with_decisions_lock(root, || {
        crate::fsutil::append_jsonl(&decisions_path(root), &Value::Object(event))
            .map_err(|e| Fail::Thrown(format!("{e}")))
    })
}

// ─── derived regen obligation (lib/cells.mjs D1/D2) ────────────────────────

const REGEN_ACK_FIELD: &str = "regen_obligation_ack";

struct RegenGuardDef {
    /// Where the covered roots come from — named in the refusal so a reader
    /// can go and check the derivation rather than trust the message.
    authority: &'static str,
    covers: &'static str,
    required: &'static str,
    command: &'static str,
    regen: &'static str,
    derive: fn() -> (Vec<String>, Vec<String>),
}

// R6 CUTOVER — WHERE THE COVERED ROOTS COME FROM NOW.
//
// Both guards used to READ A .mjs FILE AT RUNTIME and parse its source for the
// paths it operated on (`path.join(REPO_ROOT, …)` literals in
// scripts/release_manifest.mjs; `checkGroup(managed.X, "<relDir>")` calls in
// scripts/ledger_parity.mjs). That was not obfuscation for its own sake — it
// was decision D2: the obligation's scope must be DERIVED from the thing it
// guards, never pasted next to it, or the two drift and the guard quietly
// covers the wrong set.
//
// Both scripts are deleted. Parsing is therefore replaced with the strongest
// available form of the same property: the guards read the SAME CONSTANT the
// authority itself uses.
//
//   * `devtools::release_manifest::INVENTORY_ROOTS` is the list
//     `build_current_records` enumerates, pinned to it in BOTH directions by
//     `every_inventory_root_covers_what_the_builder_enumerates` and
//     `every_inventory_root_is_actually_enumerated`.
//   * `onboard::plan::LEDGER_COVERED_ROOTS` is the directory set the managed
//     ledger fingerprints, pinned to `build_managed_versions` by
//     `ledger_groups_cover_every_managed_file_group`.
//
// This is stronger than the parse it replaces: a source edit that changed the
// covered set used to be caught only if the PARSER still recognised the new
// shape (and `derive_regen_guards` threw when it did not), whereas a shared
// constant cannot be out of date without a test going red.
//
// The old failure mode is gone with it. `derive_regen_guards` used to
// `continue` — silently deactivating the guard — when the script was missing.
// Deleting the two `.mjs` files would have hit exactly that arm and switched
// BOTH obligations off with no output at all. There is no missing-file arm any
// more: the authorities are compiled in, and an empty root list is still a
// loud refusal.
const REGEN_GUARDS: [RegenGuardDef; 2] = [
    RegenGuardDef {
        authority: "devtools::release_manifest::INVENTORY_ROOTS",
        covers: "the release manifest hashes",
        required: "bee dev release-manifest --check",
        command: "bee dev release-manifest --check",
        regen: "bee dev render-skill-trees, then bee onboard --repo-root . --apply, then bee dev release-manifest --write (in that order)",
        derive: derive_manifest_scope,
    },
    RegenGuardDef {
        authority: "onboard::plan::LEDGER_COVERED_ROOTS",
        covers: "the .bee/onboarding.json managed-hash ledger covers",
        required: "bee onboard --repo-root . --json",
        command: "bee onboard --repo-root . --json",
        regen: "bee onboard --repo-root . --apply",
        derive: derive_ledger_scope,
    },
];

/// The release-manifest scope: every inventory root EXCEPT the manifest file
/// itself, which becomes the required file instead (a cell that edits a covered
/// root must also list the regenerated manifest in `files`). Same split the
/// `.mjs` parse produced from MANIFEST_PATH.
fn derive_manifest_scope() -> (Vec<String>, Vec<String>) {
    let manifest = crate::devtools::release_manifest_rel().to_string();
    let mut roots: Vec<String> = crate::devtools::release_manifest_roots()
        .iter()
        .map(|r| (*r).to_string())
        .filter(|r| *r != manifest)
        .collect();
    js_default_str_sort(&mut roots);
    (roots, vec![manifest])
}

/// The ledger scope: the host directories the managed-hash groups cover. No
/// required file — re-running onboarding rewrites `.bee/onboarding.json`
/// itself, so there is nothing for the cell to list by hand.
fn derive_ledger_scope() -> (Vec<String>, Vec<String>) {
    let mut roots: Vec<String> =
        crate::onboard::ledger_covered_roots().into_iter().map(str::to_string).collect();
    js_default_str_sort(&mut roots);
    (roots, Vec::new())
}

struct ActiveGuard {
    def: &'static RegenGuardDef,
    roots: Vec<String>,
    required_files: Vec<String>,
}

/// deriveRegenGuards: absent script -> inactive; present-but-blind -> throw.
fn derive_regen_guards() -> MR<Vec<ActiveGuard>> {
    let mut active = Vec::new();
    for guard in REGEN_GUARDS.iter() {
        let (roots, required_files) = (guard.derive)();
        // There is no "guard not installed" arm any more (see the note above
        // REGEN_GUARDS): the authorities are compiled into this binary, so the
        // only way to get an empty scope is a real defect — and a blind guard
        // refuses rather than passing everything.
        if roots.is_empty() {
            return Err(Fail::Thrown(format!(
                "regen obligation: could not derive any covered root from {} — the guard would be blind, so the write is refused rather than passed silently. FIX: that authority returned an empty root set; restore it there (never paste a literal root list in — see D2).",
                guard.authority
            )));
        }
        active.push(ActiveGuard { def: guard, roots, required_files });
    }
    Ok(active)
}

/// lib/cells.mjs normalizeCellPath.
fn normalize_cell_path(value: &str) -> String {
    let mut s = js_trim(value).replace('\\', "/");
    if let Some(rest) = s.strip_prefix("./") {
        s = rest.to_string(); // /^\.\//, one occurrence
    }
    while s.ends_with('/') {
        s.pop();
    }
    s
}

fn path_under_root(file: &str, root_path: &str) -> bool {
    file == root_path || file.starts_with(&format!("{root_path}/"))
}

/// lib/cells.mjs regenObligationRefusal — None when nothing is owed.
fn regen_obligation_refusal(cell: &Map<String, Value>, verb: &str) -> MR<Option<String>> {
    if let Some(ack) = cell.get(REGEN_ACK_FIELD) {
        if matches!(ack, Value::String(s) if !js_trim(s).is_empty()) {
            return Ok(None); // D1 escape hatch
        }
    }
    let files: Vec<String> = match cell.get("files") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|f| match f {
                Value::String(s) if !js_trim(s).is_empty() => Some(normalize_cell_path(s)),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    if files.is_empty() {
        return Ok(None);
    }
    let verify = match cell.get("verify") {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };
    let id = match cell.get("id") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => "(unknown id)".to_string(),
    };
    for guard in derive_regen_guards()? {
        let mut hit: Option<(String, String)> = None;
        for file in &files {
            if let Some(matched) = guard.roots.iter().find(|r| path_under_root(file, r)) {
                hit = Some((file.clone(), matched.clone()));
                break;
            }
        }
        let Some((hit_path, hit_root)) = hit else { continue };
        let mut missing = Vec::new();
        if !verify.contains(guard.def.required) {
            missing.push(format!("verify does not contain \"{}\"", guard.def.required));
        }
        for required_file in &guard.required_files {
            if !files.contains(required_file) {
                missing.push(format!("files does not list \"{required_file}\""));
            }
        }
        if missing.is_empty() {
            continue;
        }
        let mut fixes = Vec::new();
        if !verify.contains(guard.def.required) {
            fixes.push(format!("add `{}` to this cell's verify", guard.def.command));
        }
        for required_file in &guard.required_files {
            if !files.contains(required_file) {
                fixes.push(format!("add \"{required_file}\" to its files"));
            }
        }
        return Ok(Some(format!(
            "{verb}: REGEN_OBLIGATION — cell \"{id}\" touches \"{hit_path}\", which falls under \"{hit_root}\", a root {} (derived at runtime from {}, never a list kept here). Missing: {}. FIX: {}, and run the regen inside THIS cell — {}. To skip deliberately, set \"{REGEN_ACK_FIELD}\" on the cell to a one-line reason; it is recorded in the cell, so skipping is a named act rather than an oversight. For parallel waves, the recognized value \"wave-barrier\" defers the regen to the orchestrator, which owes the full regen chain once at wave close, in the wave-close commit, before the wave is declared clean (parallel-default D2). The write is refused; nothing was written.",
            guard.def.covers,
            guard.def.authority,
            missing.join("; "),
            fixes.join(", "),
            guard.def.regen,
        )));
    }
    Ok(None)
}

fn assert_regen_obligation(cell: &Map<String, Value>, verb: &str) -> MR<()> {
    match regen_obligation_refusal(cell, verb)? {
        Some(refusal) => Err(Fail::Thrown(refusal)),
        None => Ok(()),
    }
}

// ─── config slice (state.mjs readConfig -> commands.test/verify) ───────────

const NO_TEST_SENTINEL: &str = "none";

struct CommandsSlice {
    /// normalizeCommands' `test`: Some(list) for a declared string/array.
    test: Option<Vec<String>>,
    /// normalizeCommands' `verify` (single trimmed string).
    verify: Option<String>,
}

fn read_commands_slice(root: &Path) -> MR<CommandsSlice> {
    let config = bstate::read_config_raw(root).map_err(|_| Fail::Delegate)?;
    let raw = config.get("commands");
    let mut out = CommandsSlice { test: None, verify: None };
    let Some(Value::Object(raw)) = raw else { return Ok(out) };
    if let Some(Value::String(s)) = raw.get("verify") {
        if !js_trim(s).is_empty() {
            out.verify = Some(js_trim(s).to_string());
        }
    }
    match raw.get("test") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => {
            out.test = Some(vec![js_trim(s).to_string()]);
        }
        Some(Value::Array(items)) => {
            let list: Vec<String> = items
                .iter()
                .filter_map(|c| match c {
                    Value::String(s) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
                    _ => None,
                })
                .collect();
            if !list.is_empty() {
                out.test = Some(list);
            }
        }
        _ => {}
    }
    Ok(out)
}

/// state.mjs isNoTestRepo over the normalized commands slice.
fn is_no_test_repo(commands: &CommandsSlice) -> bool {
    commands.verify.as_deref() == Some(NO_TEST_SENTINEL)
        || matches!(&commands.test, Some(list) if list.len() == 1 && list[0] == NO_TEST_SENTINEL)
}

// ─── cell validators (lib/cells.mjs validateNewCell / updateCell) ──────────

const LANES: [&str; 5] = ["tiny", "small", "standard", "high-risk", "spike"];
const MODEL_TIERS: [&str; 3] = ["extraction", "generation", "ceiling"];
const CHANGE_CLASSES: [&str; 8] =
    ["formatting", "bugfix", "behavior", "api", "security", "migration", "refactor", "test"];
const BUDGET_KEYS: [&str; 3] = ["max_claims", "max_failed_attempts", "max_same_signature"];
const BUDGET_DEFAULTS: [f64; 3] = [3.0, 4.0, 2.0];
const BUDGET_HARD_MAX: [f64; 3] = [9.0, 12.0, 6.0];

/// assertVerifySentinelAllowed (no-test-repos D1/D2).
fn assert_verify_sentinel_allowed(root: &Path, verb: &str, verify: &Value) -> MR<()> {
    if !matches!(verify, Value::String(s) if s == NO_TEST_SENTINEL) {
        return Ok(());
    }
    if is_no_test_repo(&read_commands_slice(root)?) {
        return Ok(());
    }
    Err(Fail::Thrown(format!(
        "{verb}: verify \"{NO_TEST_SENTINEL}\" is refused — this repo has not declared itself a no-test repo. FIX: use a real, runnable verify command, or declare the repo no-test first by setting commands.verify (or commands.test) to \"{NO_TEST_SENTINEL}\" in .bee/config.json (decision 55b951e1)."
    )))
}

fn nonblank_string(v: Option<&Value>) -> bool {
    matches!(v, Some(Value::String(s)) if !js_trim(s).is_empty())
}

fn is_string_array(v: &Value) -> bool {
    matches!(v, Value::Array(items) if items.iter().all(|i| matches!(i, Value::String(_))))
}

/// JS Number.isInteger for a JSON value.
fn js_is_integer(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => {
            let f = n.as_f64()?;
            if f.is_finite() && f.fract() == 0.0 {
                Some(f)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// lib/cells.mjs validateNewCell — throws (Fail::Thrown) the FIRST problem.
fn validate_new_cell(root: &Path, cell: &Value) -> MR<()> {
    let map = match cell {
        Value::Object(m) => m,
        _ => return Err(Fail::Thrown("addCell: cell must be a JSON object.".into())),
    };
    for field in ["id", "feature", "title", "action", "verify"] {
        if !nonblank_string(map.get(field)) {
            return Err(Fail::Thrown(format!(
                "addCell: cell is missing required field \"{field}\" (non-empty string)."
            )));
        }
    }
    assert_verify_sentinel_allowed(root, "addCell", map.get("verify").unwrap_or(&Value::Null))?;
    let id = match map.get("id") {
        Some(Value::String(s)) => s.clone(),
        _ => unreachable!("checked above"),
    };
    if !id_pattern_ok(&id) {
        return Err(Fail::Thrown(format!(
            "addCell: invalid id \"{id}\" — use letters, digits, dot, dash, underscore (e.g. \"auth-3\")."
        )));
    }
    let lane_ok = matches!(map.get("lane"), Some(Value::String(s)) if LANES.contains(&s.as_str()));
    if !lane_ok {
        return Err(Fail::Thrown(format!(
            "addCell: invalid lane \"{}\" — must be one of: {}.",
            js_string_or_undefined(map.get("lane")),
            LANES.join(", ")
        )));
    }
    let lane = match map.get("lane") {
        Some(Value::String(s)) => s.clone(),
        _ => unreachable!(),
    };
    if lane == "standard" || lane == "high-risk" {
        let truths = map
            .get("must_haves")
            .filter(|m| js_truthy(m))
            .and_then(|m| m.get("truths"));
        let ok = matches!(truths, Some(Value::Array(a)) if !a.is_empty());
        if !ok {
            return Err(Fail::Thrown(format!(
                "addCell: lane \"{lane}\" requires non-empty must_haves.truths (observable truths to verify)."
            )));
        }
    }
    if let Some(pbi) = map.get("pbi") {
        if !matches!(pbi, Value::Null | Value::String(_)) {
            return Err(Fail::Thrown(
                "addCell: optional \"pbi\" must be a string backlog id when present.".into(),
            ));
        }
    }
    if let Some(tier) = map.get("tier") {
        let ok = matches!(tier, Value::Null)
            || matches!(tier, Value::String(s) if MODEL_TIERS.contains(&s.as_str()));
        if !ok {
            return Err(Fail::Thrown(format!(
                "addCell: optional \"tier\" must be one of {} when present.",
                MODEL_TIERS.join(", ")
            )));
        }
    }
    if let Some(class) = map.get("change_class") {
        let ok = matches!(class, Value::Null)
            || matches!(class, Value::String(s) if CHANGE_CLASSES.contains(&s.as_str()));
        if !ok {
            return Err(Fail::Thrown(format!(
                "addCell: optional \"change_class\" must be one of {} when present.",
                CHANGE_CLASSES.join(", ")
            )));
        }
    }
    if let Some(budgets) = map.get("budgets") {
        if !matches!(budgets, Value::Null) {
            let Value::Object(budget_map) = budgets else {
                return Err(Fail::Thrown(
                    "addCell: optional \"budgets\" must be a plain object when present.".into(),
                ));
            };
            for key in budget_map.keys() {
                if !BUDGET_KEYS.contains(&key.as_str()) {
                    return Err(Fail::Thrown(format!(
                        "addCell: unknown \"budgets\" key \"{key}\" — must be one of: {}.",
                        BUDGET_KEYS.join(", ")
                    )));
                }
            }
            for (idx, key) in BUDGET_KEYS.iter().enumerate() {
                let Some(value) = budget_map.get(*key) else { continue };
                let hard_max = BUDGET_HARD_MAX[idx];
                let ok = js_is_integer(value).map(|f| f >= 1.0 && f <= hard_max).unwrap_or(false);
                if !ok {
                    return Err(Fail::Thrown(format!(
                        "addCell: \"budgets.{key}\" must be an integer in [1, {}] when present, got {}.",
                        jsjson::js_f64_to_string(hard_max),
                        jsjson::stringify(value)
                    )));
                }
            }
        }
    }
    if let Some(ack) = map.get(REGEN_ACK_FIELD) {
        if !matches!(ack, Value::Null) && !nonblank_string(Some(ack)) {
            return Err(Fail::Thrown(format!(
                "addCell: optional \"{REGEN_ACK_FIELD}\" must be a non-empty string (the one-line reason the derived regen obligation is being skipped)."
            )));
        }
    }
    if read_cell_norm(root, &id)?.map(|v| js_truthy(&v)).unwrap_or(false) {
        return Err(Fail::Thrown(format!("addCell: cell \"{id}\" already exists.")));
    }
    assert_regen_obligation(map, "addCell")
}

/// lib/cells.mjs normalizeNewCell — key order: existing keys keep position,
/// the literal's fields (status, deps, decisions, files, read_first, trace)
/// append where absent.
fn normalize_new_cell(cell: &Value) -> MR<Value> {
    let Value::Object(map) = cell else { return Err(Fail::Delegate) };
    let mut out = map.clone();
    let status = match map.get("status") {
        Some(v) if js_truthy(v) => v.clone(),
        _ => Value::String("open".into()),
    };
    out.insert("status".into(), status);
    for key in ["deps", "decisions", "files", "read_first"] {
        let value = match map.get(key) {
            Some(Value::Array(a)) => Value::Array(a.clone()),
            _ => Value::Array(vec![]),
        };
        out.insert(key.into(), value);
    }
    out.insert("trace".into(), Value::Object(merge_trace(map.get("trace"))?));
    Ok(Value::Object(out))
}

// ─── cycle detection (lib/schedule.mjs detectCycles, Tarjan) ───────────────

fn schedule_deps_of(cell: &Value) -> Vec<String> {
    match cell.get("deps") {
        Some(Value::Array(deps)) => deps
            .iter()
            .filter_map(|d| match d {
                Value::String(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn schedule_files_of(cell: &Value) -> Vec<String> {
    match cell.get("files") {
        Some(Value::Array(files)) => files
            .iter()
            .filter_map(|f| match f {
                Value::String(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn ids_by_id(cells: &[Value]) -> Vec<(String, &Value)> {
    let mut by_id: Vec<(String, &Value)> = Vec::new();
    for cell in cells {
        if let Some(Value::String(id)) = cell.get("id") {
            if id.is_empty() {
                continue;
            }
            if let Some(slot) = by_id.iter_mut().find(|(k, _)| k == id) {
                slot.1 = cell; // Map.set: last value, first position
            } else {
                by_id.push((id.clone(), cell));
            }
        }
    }
    by_id
}

/// detectCycles — iterative Tarjan SCC; output normalized by the same sorts
/// Node applies (members sorted, cycles sorted by first member), so SCC
/// emission order never shows.
fn detect_cycles(cells: &[Value]) -> Vec<Vec<String>> {
    let by_id = ids_by_id(cells);
    let index_of: std::collections::HashMap<&str, usize> =
        by_id.iter().enumerate().map(|(i, (k, _))| (k.as_str(), i)).collect();
    let n = by_id.len();
    let adj: Vec<Vec<usize>> = by_id
        .iter()
        .map(|(_, cell)| {
            schedule_deps_of(cell)
                .into_iter()
                .filter_map(|d| index_of.get(d.as_str()).copied())
                .collect()
        })
        .collect();

    let mut indices = vec![usize::MAX; n];
    let mut lowlink = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut sccs: Vec<Vec<usize>> = Vec::new();
    let mut counter = 0usize;

    for start in 0..n {
        if indices[start] != usize::MAX {
            continue;
        }
        // Iterative DFS frame: (node, next-edge-index).
        let mut call: Vec<(usize, usize)> = vec![(start, 0)];
        indices[start] = counter;
        lowlink[start] = counter;
        counter += 1;
        stack.push(start);
        on_stack[start] = true;
        while !call.is_empty() {
            let (v, ei) = *call.last().unwrap();
            if ei < adj[v].len() {
                call.last_mut().unwrap().1 += 1;
                let w = adj[v][ei];
                if indices[w] == usize::MAX {
                    indices[w] = counter;
                    lowlink[w] = counter;
                    counter += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    call.push((w, 0));
                } else if on_stack[w] {
                    lowlink[v] = lowlink[v].min(indices[w]);
                }
            } else {
                if lowlink[v] == indices[v] {
                    let mut component = Vec::new();
                    loop {
                        let w = stack.pop().unwrap();
                        on_stack[w] = false;
                        component.push(w);
                        if w == v {
                            break;
                        }
                    }
                    sccs.push(component);
                }
                call.pop();
                if let Some(&(parent, _)) = call.last() {
                    lowlink[parent] = lowlink[parent].min(lowlink[v]);
                }
            }
        }
    }

    let mut cycles: Vec<Vec<String>> = Vec::new();
    for component in sccs {
        if component.len() > 1 {
            let mut members: Vec<String> =
                component.iter().map(|&i| by_id[i].0.clone()).collect();
            js_default_str_sort(&mut members);
            cycles.push(members);
            continue;
        }
        let idx = component[0];
        if adj[idx].contains(&idx) {
            cycles.push(vec![by_id[idx].0.clone()]);
        }
    }
    cycles.sort_by(|a, b| {
        let au: Vec<u16> = a[0].encode_utf16().collect();
        let bu: Vec<u16> = b[0].encode_utf16().collect();
        au.cmp(&bu)
    });
    cycles
}

/// lib/cells.mjs computeIncomingCycles: on-disk cells overlaid by incoming,
/// filtered to cycles touching an incoming id.
fn compute_incoming_cycles(root: &Path, incoming: &[Value]) -> MR<Vec<Vec<String>>> {
    let disk = list_cells(root, None, None).map_err(|_| Fail::Delegate)?;
    let mut union: Vec<(String, Value)> = Vec::new();
    for cell in &disk {
        if let Some(Value::String(id)) = cell.get("id") {
            if id.is_empty() {
                continue;
            }
            if let Some(slot) = union.iter_mut().find(|(k, _)| k == id) {
                slot.1 = cell.clone();
            } else {
                union.push((id.clone(), cell.clone()));
            }
        }
    }
    let mut incoming_ids: Vec<String> = Vec::new();
    for cell in incoming {
        if let Some(Value::String(id)) = cell.get("id") {
            if id.is_empty() {
                continue;
            }
            if let Some(slot) = union.iter_mut().find(|(k, _)| k == id) {
                slot.1 = cell.clone();
            } else {
                union.push((id.clone(), cell.clone()));
            }
            if !incoming_ids.contains(id) {
                incoming_ids.push(id.clone());
            }
        }
    }
    let values: Vec<Value> = union.into_iter().map(|(_, v)| v).collect();
    Ok(detect_cycles(&values)
        .into_iter()
        .filter(|cycle| cycle.iter().any(|id| incoming_ids.contains(id)))
        .collect())
}

fn format_cycle_refusal(verb: &str, cycles: &[Vec<String>]) -> String {
    let named = cycles
        .iter()
        .map(|c| c.join(" -> "))
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "{verb}: dependency cycle refused — {named}. Cycles are illegal at every dep-mutating write (D2); file overlap stays legal and is never refused."
    )
}

fn assert_no_cycle(root: &Path, verb: &str, incoming: &[Value]) -> MR<()> {
    let cycles = compute_incoming_cycles(root, incoming)?;
    if cycles.is_empty() {
        Ok(())
    } else {
        Err(Fail::Thrown(format_cycle_refusal(verb, &cycles)))
    }
}

// ─── budgets (lib/cells.mjs D2/D-GHF) ──────────────────────────────────────

struct Budgets {
    max_claims: f64,
    max_failed_attempts: f64,
    max_same_signature: f64,
}

/// resolveCellBudgets: forgiving runtime fallback + hard-max clamp.
fn resolve_cell_budgets(cell: &Map<String, Value>) -> Budgets {
    let declared = match cell.get("budgets") {
        Some(Value::Object(m)) => Some(m),
        _ => None,
    };
    let pick = |idx: usize| -> f64 {
        let value = declared.and_then(|m| m.get(BUDGET_KEYS[idx]));
        match value.and_then(js_is_integer) {
            Some(v) if v >= 1.0 => v.min(BUDGET_HARD_MAX[idx]),
            _ => BUDGET_DEFAULTS[idx],
        }
    };
    Budgets { max_claims: pick(0), max_failed_attempts: pick(1), max_same_signature: pick(2) }
}

const FAILED_ATTEMPT_VERDICTS: [&str; 3] = ["fail", "blocked", "tests-red"];

/// attemptsSinceBudgetReset — lexical ISO comparison, per the .mjs.
fn attempts_since_budget_reset(cell: &Map<String, Value>) -> MR<Vec<Value>> {
    let trace = match cell.get("trace") {
        Some(Value::Object(t)) => Some(t),
        _ => None,
    };
    let attempts: Vec<Value> = match trace.and_then(|t| t.get("attempts")) {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    // Null entries would crash Node's later `a.claim_session` access.
    if attempts.iter().any(|a| a.is_null()) {
        return Err(Fail::Delegate);
    }
    let resets = match trace.and_then(|t| t.get("budget_resets")) {
        Some(Value::Array(r)) => r.clone(),
        _ => Vec::new(),
    };
    let marker = resets
        .last()
        .and_then(|r| r.get("reset_at"))
        .and_then(|v| match v {
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            _ => None,
        });
    let Some(marker) = marker else { return Ok(attempts) };
    Ok(attempts
        .into_iter()
        .filter(|a| matches!(a.get("at"), Some(Value::String(at)) if at.as_str() > marker.as_str()))
        .collect())
}

enum BudgetCheck {
    Ok,
    Refused { code: &'static str, reason: String },
}

/// checkCellBudgets — the structural loop-safety check (never reads bypass).
fn check_cell_budgets(cell: &Map<String, Value>) -> MR<BudgetCheck> {
    let budgets = resolve_cell_budgets(cell);
    let relevant = attempts_since_budget_reset(cell)?;
    let id_disp = js_string_or_undefined(cell.get("id"));

    let coerce = |v: Option<&Value>| -> String {
        match v {
            None | Some(Value::Null) => String::new(), // ?? ''
            Some(other) => jsjson::js_to_string(other),
        }
    };
    let mut pairs: Vec<String> = Vec::new();
    for a in &relevant {
        let acquired = match a.get("acquired_at") {
            None | Some(Value::Null) => a.get("claimed_at"),
            other => other,
        };
        let key = format!("{} {}", coerce(a.get("claim_session")), coerce(acquired));
        if !pairs.contains(&key) {
            pairs.push(key);
        }
    }
    let claims_used = pairs.len() as f64 + 1.0;
    if claims_used > budgets.max_claims {
        return Ok(BudgetCheck::Refused {
            code: "CELL_BUDGET_EXHAUSTED",
            reason: format!(
                "cell \"{id_disp}\" exhausted its \"max_claims\" budget (limit {}, used {}) — the claim door is closed until an audited reset.",
                jsjson::js_f64_to_string(budgets.max_claims),
                jsjson::js_f64_to_string(claims_used)
            ),
        });
    }

    let is_failed =
        |a: &Value| matches!(a.get("verdict"), Some(Value::String(v)) if FAILED_ATTEMPT_VERDICTS.contains(&v.as_str()));
    let failed = relevant.iter().filter(|a| is_failed(a)).count() as f64;
    if failed >= budgets.max_failed_attempts {
        return Ok(BudgetCheck::Refused {
            code: "CELL_BUDGET_EXHAUSTED",
            reason: format!(
                "cell \"{id_disp}\" exhausted its \"max_failed_attempts\" budget (limit {}, used {}) — the claim door is closed until an audited reset.",
                jsjson::js_f64_to_string(budgets.max_failed_attempts),
                jsjson::js_f64_to_string(failed)
            ),
        });
    }

    // Same-signature refusal — insertion-ordered Map, first offender wins.
    let mut signature_counts: Vec<(String, f64)> = Vec::new();
    for a in &relevant {
        if !is_failed(a) {
            continue;
        }
        let Some(Value::String(sig)) = a.get("failure_signature") else { continue };
        if sig.is_empty() {
            continue;
        }
        if let Some(slot) = signature_counts.iter_mut().find(|(s, _)| s == sig) {
            slot.1 += 1.0;
        } else {
            signature_counts.push((sig.clone(), 1.0));
        }
    }
    for (signature, count) in &signature_counts {
        if *count >= budgets.max_same_signature {
            return Ok(BudgetCheck::Refused {
                code: "REPEATED_FAILURE",
                reason: format!(
                    "cell \"{id_disp}\" failed {} time(s) with the identical signature \"{signature}\" — change approach or escalate, this is not a re-run.",
                    jsjson::js_f64_to_string(*count)
                ),
            });
        }
    }
    Ok(BudgetCheck::Ok)
}

// ─── frozen judge (lib/cells.mjs P12) + judge verdict schema (judge.mjs) ───

/// FROZEN_JUDGE_PATTERNS — hand matchers (regexes are anchored/segmented, so
/// each collapses to segment/suffix checks; all case-insensitive).
fn frozen_judge_rule(file: &str) -> Option<&'static str> {
    let lower = file.to_lowercase();
    let seg_starts: Vec<usize> = std::iter::once(0)
        .chain(lower.match_indices('/').map(|(i, _)| i + 1))
        .collect();
    let seg_prefix = |names: &[&str]| -> bool {
        seg_starts
            .iter()
            .any(|&s| names.iter().any(|n| lower[s..].starts_with(n)))
    };
    let last_seg = seg_starts.last().map(|&s| &lower[s..]).unwrap_or(&lower);
    // /(^|\/)(tests?|__tests__|specs?)\//i
    if seg_prefix(&["tests/", "test/", "__tests__/", "specs/", "spec/"]) {
        return Some("test sources");
    }
    // /\.(test|spec)\.[a-z]+$/i
    for marker in [".test.", ".spec."] {
        if let Some(pos) = lower.rfind(marker) {
            let ext = &lower[pos + marker.len()..];
            if !ext.is_empty() && ext.chars().all(|c| c.is_ascii_lowercase()) {
                return Some("test file");
            }
        }
    }
    // /(^|\/)__snapshots__\/|\.snap$/i
    if seg_prefix(&["__snapshots__/"]) || lower.ends_with(".snap") {
        return Some("snapshot");
    }
    // CI config
    if seg_prefix(&[".github/workflows/", ".circleci/"])
        || [".gitlab-ci.yml", "jenkinsfile", "azure-pipelines.yml"].contains(&last_seg)
    {
        return Some("CI config");
    }
    // lockfile
    if [
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "bun.lock",
        "bun.lockb",
        "cargo.lock",
        "poetry.lock",
        "uv.lock",
        "go.sum",
        "composer.lock",
        "gemfile.lock",
    ]
    .contains(&last_seg)
    {
        return Some("lockfile");
    }
    // package manifest
    if ["package.json", "pyproject.toml", "cargo.toml", "go.mod", "composer.json", "gemfile"]
        .contains(&last_seg)
    {
        return Some("package manifest");
    }
    // test config: last segment starts with one of the prefixes
    if [
        "jest.config",
        "vitest.config",
        "playwright.config",
        "karma.conf",
        "pytest.ini",
        "tox.ini",
        "phpunit.xml",
    ]
    .iter()
    .any(|p| last_seg.starts_with(p))
    {
        return Some("test config");
    }
    // /(^|\/)\.bee\/config\.json$/i
    if lower == ".bee/config.json" || lower.ends_with("/.bee/config.json") {
        return Some("bee verify config");
    }
    None
}

/// lib/cells.mjs normalizePath (frozen-judge flavor: trim LAST).
fn frozen_normalize_path(p: &Value) -> String {
    let mut s = jsjson::js_to_string(p).replace('\\', "/");
    if let Some(rest) = s.strip_prefix("./") {
        s = rest.to_string();
    }
    js_trim(&s).to_string()
}

/// declaredCovers glob: '*' within a segment, '**' across segments; exact
/// and dir-prefix matches first.
fn declared_covers(declared: &[Value], file: &str) -> bool {
    for raw in declared {
        let entry = frozen_normalize_path(raw);
        if entry.is_empty() {
            continue;
        }
        if entry == file {
            return true;
        }
        if entry.ends_with('/') && file.starts_with(&entry) {
            return true;
        }
        if entry.contains('*') && glob_covers(&entry, file) {
            return true;
        }
    }
    false
}

/// Wildcard match: `**` -> `.*`, `*` -> `[^/]*`, everything else literal.
fn glob_covers(pattern: &str, text: &str) -> bool {
    #[derive(Clone, Copy, PartialEq)]
    enum Tok {
        Any,     // '**'  (.*)
        Seg,     // '*'   ([^/]*)
        Lit(char),
    }
    let mut toks: Vec<Tok> = Vec::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '*' {
            if i + 1 < chars.len() && chars[i + 1] == '*' {
                toks.push(Tok::Any);
                i += 2;
            } else {
                toks.push(Tok::Seg);
                i += 1;
            }
        } else {
            toks.push(Tok::Lit(chars[i]));
            i += 1;
        }
    }
    let text: Vec<char> = text.chars().collect();
    // DP over (token, text position).
    let n = toks.len();
    let m = text.len();
    let mut dp = vec![vec![false; m + 1]; n + 1];
    dp[0][0] = true;
    for ti in 0..n {
        for pos in 0..=m {
            if !dp[ti][pos] {
                continue;
            }
            match toks[ti] {
                Tok::Lit(c) => {
                    if pos < m && text[pos] == c {
                        dp[ti + 1][pos + 1] = true;
                    }
                }
                Tok::Seg => {
                    let mut k = pos;
                    dp[ti + 1][k] = true;
                    while k < m && text[k] != '/' {
                        k += 1;
                        dp[ti + 1][k] = true;
                    }
                }
                Tok::Any => {
                    for k in pos..=m {
                        dp[ti + 1][k] = true;
                    }
                }
            }
        }
    }
    dp[n][m]
}

/// frozenJudgeHits — [{file, rule}] rows.
fn frozen_judge_hits(changed: &Value, declared: &Value) -> Vec<(String, &'static str)> {
    let declared_list: Vec<Value> = match declared {
        Value::Array(a) => a.clone(),
        _ => Vec::new(),
    };
    let changed_list: Vec<Value> = match changed {
        Value::Array(a) => a.clone(),
        _ => Vec::new(),
    };
    let mut hits = Vec::new();
    for raw in &changed_list {
        let file = frozen_normalize_path(raw);
        if file.is_empty() {
            continue;
        }
        let Some(rule) = frozen_judge_rule(&file) else { continue };
        if declared_covers(&declared_list, &file) {
            continue;
        }
        hits.push((file, rule));
    }
    hits
}

// ─── judge.mjs — verdict schema validation + model independence ────────────

const JUDGE_VERDICT_SCHEMA: &str = "judge-verdict/1";
const JUDGE_VERDICTS: [&str; 2] = ["PASS", "NEEDS_REVISION"];
const CHECK_STATUSES: [&str; 2] = ["PASS", "FAIL"];
const JUDGE_FIXABILITIES: [&str; 2] = ["automatic", "authority"];
const JUDGE_CONFIDENCES: [&str; 3] = ["low", "medium", "high"];
const PINNED_MODEL_STATUS: &str = "pinned"; // dispatch-guard.mjs

fn is_nonempty_string_value(v: Option<&Value>) -> bool {
    matches!(v, Some(Value::String(s)) if !js_trim(s).is_empty())
}

/// validateJudgeVerdict -> (ok, errors). `verdict` may be any JSON value —
/// free prose (a string) is the non-object error.
fn validate_judge_verdict(obj: &Value) -> (bool, Vec<String>) {
    let mut errors = Vec::new();
    let map = match obj {
        Value::Object(m) => m,
        _ => {
            errors.push(
                "verdict must be a JSON object per schema \"judge-verdict/1\" (got free-form/non-object output) — a judge that returns free prose is a failed judge run, not a valid verdict."
                    .to_string(),
            );
            return (false, errors);
        }
    };
    if !matches!(map.get("schema"), Some(Value::String(s)) if s == JUDGE_VERDICT_SCHEMA) {
        errors.push(format!(
            "schema must be \"{JUDGE_VERDICT_SCHEMA}\", got {}.",
            js_json_or_undefined(map.get("schema"))
        ));
    }
    let verdict_ok =
        matches!(map.get("verdict"), Some(Value::String(s)) if JUDGE_VERDICTS.contains(&s.as_str()));
    if !verdict_ok {
        errors.push(format!(
            "verdict must be one of {}, got {}.",
            JUDGE_VERDICTS.join("|"),
            js_json_or_undefined(map.get("verdict"))
        ));
    }
    let mut any_fail = false;
    match map.get("checks") {
        Some(Value::Array(checks)) if !checks.is_empty() => {
            for (i, entry) in checks.iter().enumerate() {
                let entry_map = match entry {
                    Value::Object(m) => m,
                    _ => {
                        errors.push(format!("checks[{i}] must be a JSON object."));
                        continue;
                    }
                };
                if !is_nonempty_string_value(entry_map.get("id")) {
                    errors.push(format!("checks[{i}].id must be a non-empty string."));
                }
                match entry_map.get("status") {
                    Some(Value::String(s)) if CHECK_STATUSES.contains(&s.as_str()) => {
                        if s == "FAIL" {
                            any_fail = true;
                        }
                    }
                    other => errors.push(format!(
                        "checks[{i}].status must be one of {}, got {}.",
                        CHECK_STATUSES.join("|"),
                        js_json_or_undefined(other)
                    )),
                }
                if !is_nonempty_string_value(entry_map.get("evidence")) {
                    errors.push(format!("checks[{i}].evidence must be a non-empty string."));
                }
            }
            let verdict_is = |name: &str| matches!(map.get("verdict"), Some(Value::String(s)) if s == name);
            if verdict_is("PASS") && any_fail {
                errors.push(
                    "verdict must not be PASS when any check has status FAIL — a PASS verdict must not carry a FAIL check."
                        .to_string(),
                );
            }
            if verdict_is("NEEDS_REVISION") && !any_fail {
                errors.push(
                    "verdict NEEDS_REVISION requires at least one check with status FAIL — got no FAIL check among the checks."
                        .to_string(),
                );
            }
        }
        _ => errors.push("checks must be a non-empty array.".to_string()),
    }
    if !matches!(map.get("fixability"), Some(Value::String(s)) if JUDGE_FIXABILITIES.contains(&s.as_str())) {
        errors.push(format!(
            "fixability must be one of {}, got {}.",
            JUDGE_FIXABILITIES.join("|"),
            js_json_or_undefined(map.get("fixability"))
        ));
    }
    if !matches!(map.get("confidence"), Some(Value::String(s)) if JUDGE_CONFIDENCES.contains(&s.as_str())) {
        errors.push(format!(
            "confidence must be one of {}, got {}.",
            JUDGE_CONFIDENCES.join("|"),
            js_json_or_undefined(map.get("confidence"))
        ));
    }
    let fs = map.get("failure_signature");
    if any_fail && !is_nonempty_string_value(fs) {
        errors.push(
            "failure_signature is required (non-empty string) when any check has status FAIL.".to_string(),
        );
    } else if let Some(v) = fs {
        if !matches!(v, Value::Null) && !is_nonempty_string_value(Some(v)) {
            errors.push("failure_signature, when present, must be a non-empty string.".to_string());
        }
    }
    (errors.is_empty(), errors)
}

/// judge.mjs deriveModelIndependence.
fn derive_model_independence(
    builder_model: Option<&str>,
    builder_status: Option<&str>,
    judge_model: Option<&str>,
    judge_status: Option<&str>,
) -> &'static str {
    let both_pinned =
        builder_status == Some(PINNED_MODEL_STATUS) && judge_status == Some(PINNED_MODEL_STATUS);
    let named = |m: Option<&str>| m.map(|s| !js_trim(s).is_empty()).unwrap_or(false);
    if !both_pinned || !named(builder_model) || !named(judge_model) {
        return "unverified";
    }
    if builder_model == judge_model {
        "same-model"
    } else {
        "confirmed"
    }
}

// ─── schedule.mjs computeSchedule ──────────────────────────────────────────

struct Schedule {
    waves: Vec<Vec<String>>,
    cycles: Vec<Vec<String>>,
    unsatisfiable: Vec<(String, String, &'static str)>, // (cell, dep, reason)
    empty_files: Vec<String>,
}

fn compute_schedule(cells: &[Value]) -> Schedule {
    let by_id = ids_by_id(cells);
    let cycles = detect_cycles(cells);

    let mut empty_files: Vec<String> = cells
        .iter()
        .filter(|c| matches!(c.get("id"), Some(Value::String(_))))
        .filter(|c| schedule_files_of(c).is_empty())
        .map(|c| match c.get("id") {
            Some(Value::String(s)) => s.clone(),
            _ => unreachable!(),
        })
        .collect();
    js_default_str_sort(&mut empty_files);

    let status_of = |cell: &Value| -> Option<String> {
        match cell.get("status") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        }
    };
    let schedulable: Vec<&Value> = cells
        .iter()
        .filter(|c| matches!(status_of(c).as_deref(), Some("open") | Some("claimed")))
        .collect();

    let lookup = |id: &str| by_id.iter().find(|(k, _)| k == id).map(|(_, v)| *v);
    let classify = |dep: &str| -> &'static str {
        match lookup(dep) {
            None => "missing",
            Some(cell) => match status_of(cell).as_deref() {
                Some("capped") => "satisfied",
                Some("blocked") => "blocked",
                Some("dropped") => "dropped",
                _ => "pending",
            },
        }
    };

    let mut unsatisfiable: Vec<(String, String, &'static str)> = Vec::new();
    let mut excluded: Vec<String> = Vec::new();
    for cell in &schedulable {
        let Some(Value::String(cid)) = cell.get("id") else { continue };
        for dep in schedule_deps_of(cell) {
            let kind = classify(&dep);
            if matches!(kind, "missing" | "blocked" | "dropped") {
                unsatisfiable.push((cid.clone(), dep, kind));
                if !excluded.contains(cid) {
                    excluded.push(cid.clone());
                }
            }
        }
    }
    unsatisfiable.sort_by(|a, b| {
        let cell_cmp = {
            let au: Vec<u16> = a.0.encode_utf16().collect();
            let bu: Vec<u16> = b.0.encode_utf16().collect();
            au.cmp(&bu)
        };
        if cell_cmp != Ordering::Equal {
            return cell_cmp;
        }
        let au: Vec<u16> = a.1.encode_utf16().collect();
        let bu: Vec<u16> = b.1.encode_utf16().collect();
        au.cmp(&bu)
    });

    // Propagate exclusion.
    let mut changed = true;
    while changed {
        changed = false;
        for cell in &schedulable {
            let Some(Value::String(cid)) = cell.get("id") else { continue };
            if excluded.contains(cid) {
                continue;
            }
            for dep in schedule_deps_of(cell) {
                if excluded.contains(&dep) {
                    excluded.push(cid.clone());
                    changed = true;
                    break;
                }
            }
        }
    }

    let nodes: Vec<&Value> = schedulable
        .iter()
        .filter(|c| match c.get("id") {
            Some(Value::String(id)) => !excluded.contains(id),
            _ => true, // an id-less cell never entered `excluded`
        })
        .copied()
        .collect();
    let node_ids: Vec<String> = nodes
        .iter()
        .filter_map(|c| match c.get("id") {
            Some(Value::String(id)) => Some(id.clone()),
            _ => None,
        })
        .collect();

    let mut in_degree: Vec<(String, usize)> = node_ids.iter().map(|id| (id.clone(), 0)).collect();
    let mut dependents: Vec<(String, Vec<String>)> =
        node_ids.iter().map(|id| (id.clone(), Vec::new())).collect();
    for cell in &nodes {
        let Some(Value::String(cid)) = cell.get("id") else { continue };
        for dep in schedule_deps_of(cell) {
            if !node_ids.contains(&dep) {
                continue;
            }
            if let Some(slot) = in_degree.iter_mut().find(|(k, _)| k == cid) {
                slot.1 += 1;
            }
            if let Some(slot) = dependents.iter_mut().find(|(k, _)| k == &dep) {
                slot.1.push(cid.clone());
            }
        }
    }

    let mut remaining = in_degree;
    let mut placed: Vec<String> = Vec::new();
    let mut waves: Vec<Vec<String>> = Vec::new();
    loop {
        let mut ready: Vec<String> = nodes
            .iter()
            .filter_map(|c| match c.get("id") {
                Some(Value::String(id)) => Some(id.clone()),
                _ => None,
            })
            .filter(|id| {
                !placed.contains(id)
                    && remaining.iter().find(|(k, _)| k == id).map(|(_, d)| *d == 0).unwrap_or(false)
            })
            .collect();
        js_default_str_sort(&mut ready);
        if ready.is_empty() {
            break;
        }
        let mut wave: Vec<String> = Vec::new();
        for id in &ready {
            let cell_files = lookup(id).map(schedule_files_of).unwrap_or_default();
            let overlaps = wave.iter().any(|placed_id| {
                let placed_files = lookup(placed_id).map(schedule_files_of).unwrap_or_default();
                placed_files
                    .iter()
                    .any(|a| cell_files.iter().any(|b| rsv::paths_overlap(a, b)))
            });
            if !overlaps {
                wave.push(id.clone());
            }
        }
        for id in &wave {
            placed.push(id.clone());
        }
        for id in &wave {
            let deps_list = dependents
                .iter()
                .find(|(k, _)| k == id)
                .map(|(_, d)| d.clone())
                .unwrap_or_default();
            for dependent in deps_list {
                if let Some(slot) = remaining.iter_mut().find(|(k, _)| *k == dependent) {
                    slot.1 = slot.1.saturating_sub(1);
                }
            }
        }
        waves.push(wave);
    }

    Schedule { waves, cycles, unsatisfiable, empty_files }
}

// ─── state/lane gate reads (state.mjs readState / readLane*) ───────────────

/// gateApproved(readState(root), gate) over the brief state slice.
fn default_gate_approved(root: &Path, gate: &str) -> MR<bool> {
    let state = bstate::read_state_brief(root).map_err(|_| Fail::Delegate)?;
    Ok(matches!(state.gates.get(gate), Some(Value::Bool(true))))
}

fn lanes_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("lanes")
}

/// state.mjs requireLaneFeature — Ok(trimmed) | Err(()) for the throw path.
fn lane_feature_ok(feature: &str) -> Option<String> {
    let trimmed = js_trim(feature);
    if trimmed.is_empty() || trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return None;
    }
    Some(trimmed.to_string())
}

fn lane_rel_path(feature: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    format!(".bee{sep}lanes{sep}{feature}.json")
}

/// laneRecordFrom's approved_gates merge (defaults ...spread). Truthy
/// non-object approved_gates spreads exotic keys — Delegate.
fn merged_lane_gates(parsed: &Map<String, Value>) -> MR<Map<String, Value>> {
    let mut gates = Map::new();
    for g in ["context", "shape", "execution", "review"] {
        gates.insert(g.into(), Value::Bool(false));
    }
    match parsed.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => {}
        Some(Value::String(s)) if s.is_empty() => {}
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => {}
        Some(Value::Object(overlay)) => spread_into(&mut gates, overlay),
        Some(Value::Array(a)) if a.is_empty() => {}
        Some(Value::Number(_)) | Some(Value::Bool(true)) => {}
        Some(_) => return Err(Fail::Delegate),
    }
    Ok(gates)
}

/// lib/cells.mjs laneRecordForFeature — None (no lane record: default gate
/// governs) | Some(approved_gates). Both of readLaneStrict's refusals are
/// deterministic thrown messages now; the unreadable-file one carries the
/// Rust io error where Node interpolated the libuv errno.
fn lane_record_gates(root: &Path, feature: Option<&Value>) -> MR<Option<Map<String, Value>>> {
    let Some(Value::String(feature)) = feature else { return Ok(None) };
    if js_trim(feature).is_empty() {
        return Ok(None);
    }
    let Some(id) = lane_feature_ok(feature) else { return Ok(None) }; // lanePath throw, caught
    let file = lanes_dir(root).join(format!("{id}.json"));
    if !file.exists() {
        return Ok(None);
    }
    let text = match std::fs::read(&file) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        // readLaneStrict's unreadable branch. Node interpolated the libuv
        // err.code; the sentence and the refusal are otherwise unchanged.
        Err(e) => {
            return Err(Fail::Thrown(format!(
                "readLaneStrict: could not read lane record \"{}\" ({e}). The bee CLI refuses to mutate a lane it cannot read — that could silently clobber real lane state (gates, phase). FIX: inspect/restore the file (e.g. \"git checkout -- {}\"), then retry.",
                file.display(),
                lane_rel_path(&id)
            )))
        }
    };
    let corrupt = || {
        Fail::Thrown(format!(
            "readLaneStrict: lane record \"{}\" exists but is corrupt (not a JSON object naming feature \"{id}\"). The bee CLI refuses to rebuild a lane from defaults over a present-but-corrupt file — that would silently clobber real lane state (gates, phase) while reporting success. FIX: inspect/restore the file (e.g. \"git checkout -- {}\"), then retry.",
            file.display(),
            lane_rel_path(&id)
        ))
    };
    // A lone-surrogate escape lands in NotJson now and takes this same
    // deterministic corrupt refusal — the non-surrogate corrupt path.
    let parsed = match parse_json_js(&text, false) {
        JsParse::Value(v) => v,
        JsParse::NotJson => return Err(corrupt()),
    };
    let map = match parsed {
        Value::Object(m) => m,
        _ => return Err(corrupt()),
    };
    if !matches!(map.get("feature"), Some(Value::String(f)) if *f == id) {
        return Err(corrupt());
    }
    Ok(Some(merged_lane_gates(&map)?))
}

/// state.mjs readLane (fail-open display read) — only `route` truthiness is
/// consumed here (claimedFeatureHasRoute).
///
/// CUTOVER: a corrupt/mismatched record used to delegate, NOT because
/// readLane's own warning needed V8 (it is deterministic) but because it
/// would have stacked on top of readJson's V8-worded one. Both warnings are
/// ours now, so both are printed, in Node's order, and the read still fails
/// open to null.
fn read_lane_route(root: &Path, feature: &str) -> MR<Option<bool>> {
    let Some(id) = lane_feature_ok(feature) else { return Ok(None) };
    let file = lanes_dir(root).join(format!("{id}.json"));
    if !file.exists() {
        return Ok(None);
    }
    match read_store_json(&file)? {
        Some(Value::Object(m)) if matches!(m.get("feature"), Some(Value::String(f)) if *f == id) => {
            Ok(Some(m.get("route").map(js_truthy).unwrap_or(false)))
        }
        // laneRecordFrom returned falsy — a record that does not name this
        // feature, or readJson's null fallback after a corrupt file (a
        // MISSING file already returned above, so None here means corrupt).
        // readLane warns and reads as "no lane".
        _ => {
            let rel = lane_rel_path(&id);
            eprintln!(
                "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
            );
            Ok(None)
        }
    }
}

/// bee.mjs claimedFeatureHasRoute (explicit-triage D3).
fn claimed_feature_has_route(root: &Path, feature: Option<&Value>) -> MR<bool> {
    let Some(feature) = feature else { return Ok(true) };
    if !js_truthy(feature) {
        return Ok(true);
    }
    let Value::String(feature_s) = feature else { return Err(Fail::Delegate) }; // non-string feature — JS-exotic path math
    if let Some(route) = read_lane_route(root, feature_s)? {
        return Ok(route);
    }
    let state = bstate::read_state_brief(root).map_err(|_| Fail::Delegate)?;
    if matches!(&state.feature, Value::String(f) if f == feature_s) {
        return Ok(js_truthy(&state.route));
    }
    Ok(true)
}

// ─── test runner (lib/test-runner.mjs) ─────────────────────────────────────

const TEST_RESULTS_RELATIVE: &str = ".bee/logs/test-results.json";
const FAILURE_EXCERPT_MAX_CHARS: usize = 500;

fn test_results_path(root: &Path) -> PathBuf {
    root.join(".bee").join("logs").join("test-results.json")
}

struct CmdRun {
    command: String,
    exit: Option<f64>,
    duration_ms: f64,
    failure_excerpt: Option<String>,
}

struct TestsRun {
    green: bool,
    ran_at: String,
    commands: Vec<CmdRun>,
}

/// posixShell(): on win32 probe Git Bash's `bash` once per process; elsewhere
/// `shell: true` already IS a POSIX sh.
fn posix_shell() -> Option<&'static str> {
    use std::sync::OnceLock;
    static PROBE: OnceLock<bool> = OnceLock::new();
    if !cfg!(windows) {
        return None;
    }
    let has_bash = *PROBE.get_or_init(|| {
        std::process::Command::new("bash")
            .args(["-c", "exit 0"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    });
    if has_bash {
        Some("bash")
    } else {
        None
    }
}

/// spawnDeclaredCommand — POSIX sh (Git Bash) on win32 when available, the
/// platform shell otherwise. Output captured lossily (Node utf8 decode).
fn spawn_declared(command: &str, cwd: &Path) -> Result<(Option<i32>, String, String), String> {
    let mut cmd = if let Some(shell) = posix_shell() {
        let mut c = std::process::Command::new(shell);
        c.args(["-c", command]);
        c
    } else if cfg!(windows) {
        // Node shell:true on win32: cmd.exe /d /s /c "<command>".
        let mut c = std::process::Command::new("cmd.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.raw_arg(format!("/d /s /c \"{command}\""));
        }
        c
    } else {
        let mut c = std::process::Command::new("/bin/sh");
        c.args(["-c", command]);
        c
    };
    match cmd.current_dir(cwd).output() {
        Ok(out) => Ok((
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )),
        Err(e) => Err(format!("{e}")), // spawn error — Node embeds its own message (residual)
    }
}

/// String.prototype.slice(-n) over UTF-16 units (excerpt tail).
fn js_slice_tail_utf16(s: &str, n: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().collect();
    if units.len() <= n {
        return s.to_string();
    }
    String::from_utf16_lossy(&units[units.len() - n..])
}

/// runDeclaredTests over an already-normalized declared command list.
fn run_declared_tests(root: &Path, commands: &[String]) -> MR<TestsRun> {
    let ran_at = utc_now();
    let mut results: Vec<CmdRun> = Vec::new();
    let mut green = true;
    for command in commands {
        let started = std::time::Instant::now();
        let spawn = spawn_declared(command, root);
        let duration_ms = started.elapsed().as_millis() as f64;
        let (exit, mut output, passed) = match spawn {
            Ok((code, stdout, stderr)) => {
                let out = format!("{stdout}{stderr}");
                (code.map(|c| c as f64), out, code == Some(0))
            }
            Err(message) => {
                let out = format!("\n[bee test] spawn error: {message}");
                (None, out, false)
            }
        };
        if !passed {
            green = false;
        }
        let excerpt = if passed {
            None
        } else {
            output = js_trim(&output).to_string();
            let tail = js_slice_tail_utf16(&output, FAILURE_EXCERPT_MAX_CHARS);
            Some(if tail.is_empty() {
                format!(
                    "(no output; exit {})",
                    exit.map(jsjson::js_f64_to_string).unwrap_or_else(|| "null".to_string())
                )
            } else {
                tail
            })
        };
        results.push(CmdRun { command: command.clone(), exit, duration_ms, failure_excerpt: excerpt });
    }
    let record = tests_record_value(&ran_at, green, &results);
    write_json_atomic(&test_results_path(root), &record).map_err(|e| Fail::Thrown(format!("{e}")))?;
    Ok(TestsRun { green, ran_at, commands: results })
}

fn tests_record_value(ran_at: &str, green: bool, commands: &[CmdRun]) -> Value {
    let rows: Vec<Value> = commands
        .iter()
        .map(|c| {
            let mut row = Map::new();
            row.insert("command".into(), Value::String(c.command.clone()));
            row.insert(
                "exit".into(),
                c.exit.and_then(Number::from_f64).map(Value::Number).unwrap_or(Value::Null),
            );
            row.insert(
                "duration_ms".into(),
                Number::from_f64(c.duration_ms).map(Value::Number).unwrap_or(Value::Null),
            );
            row.insert(
                "failure_excerpt".into(),
                c.failure_excerpt.clone().map(Value::String).unwrap_or(Value::Null),
            );
            Value::Object(row)
        })
        .collect();
    let mut record = Map::new();
    record.insert("ran_at".into(), Value::String(ran_at.to_string()));
    record.insert("green".into(), Value::Bool(green));
    record.insert("commands".into(), Value::Array(rows));
    Value::Object(record)
}

/// firstFailureLine — first non-empty line of the first failing excerpt.
fn first_failure_line(run: &TestsRun) -> Option<String> {
    let failing = run
        .commands
        .iter()
        .find(|c| c.failure_excerpt.as_deref().map(|e| !e.is_empty()).unwrap_or(false))?;
    failing
        .failure_excerpt
        .as_deref()
        .unwrap_or("")
        .split('\n')
        .map(|l| js_trim(l.trim_end_matches('\r')))
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

// ─── reservations-release subset (finish's release half) ───────────────────
// Provenance: lib/reservations.mjs release/listReservations + bee.mjs
// releaseReservationsForAgent, mirrored from verbs/reservations.rs's own
// release_exec (those fns are module-private there; this copy keeps cells.rs
// self-contained per the one-file rule).

const CROSS_WORKTREE_HOLDS_LOCK: &str = "cross-worktree-holds";

fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn holds_ledger_path(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("cross-worktree-holds.json")
}

/// worktree-holds.mjs readStore — `readJson(path, null)` then a shape check
/// that turns anything without an array `holds` into `{holds: []}`. A corrupt
/// ledger warns and takes that same `{holds: []}` fallback (Node's `null`
/// fallback reached it through `!store`). Null hold ENTRIES still delegate:
/// that is a JS-exotic shape, not a parse failure.
fn read_holds_store(root: &Path) -> MR<Value> {
    let ledger = holds_ledger_path(root);
    let store = match read_json(&ledger) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => {
            warn_corrupt_json_once(&ledger);
            None
        }
        ReadJson::Parsed(v) => Some(rsv::js_numberify(&v).map_err(|_| Fail::Delegate)?),
    };
    let ok_shape = store
        .as_ref()
        .map(|s| matches!(s.get("holds"), Some(Value::Array(_))))
        .unwrap_or(false);
    if !ok_shape {
        return Ok(json!({ "holds": [] }));
    }
    let store = store.unwrap();
    if let Some(Value::Array(holds)) = store.get("holds") {
        if holds.iter().any(|h| h.is_null()) {
            return Err(Fail::Delegate);
        }
    }
    Ok(store)
}

fn list_path_lease_records(root: &Path) -> MR<Vec<Map<String, Value>>> {
    let control = control_root(root)?;
    let leases_root = control.join(".bee").join("runtime").join("leases");
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
                if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                    // readLeaseSafe: corrupt silently skipped (no warn in Node).
                    if let Value::Object(m) = rsv::js_numberify(&parsed).map_err(|_| Fail::Delegate)? {
                        let is_path =
                            matches!(m.get("resource"), Some(Value::String(s)) if s.starts_with("path:"));
                        if is_path {
                            out.push(m);
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

fn path_lease_file(control_root: &Path, raw_path_id: &str) -> PathBuf {
    let canonical = rsv::res_normalize_path(raw_path_id);
    let resource_key = format!("path:{canonical}");
    control_root
        .join(".bee")
        .join("runtime")
        .join("leases")
        .join("paths")
        .join(format!("{}.json", sha256_hex(&resource_key)))
}

fn lease_record_expired(rec: &Map<String, Value>, now: f64) -> MR<bool> {
    match rec.get("expires_at") {
        None | Some(Value::Null) => Ok(false),
        Some(v) => match rsv::date_parse_val(Some(v)).map_err(|_| Fail::Delegate)? {
            None => Ok(false),
            Some(ms) => Ok(ms <= now),
        },
    }
}

struct ResvLite {
    agent: Option<Value>,
    cell: Option<Value>,
    path: String,
    session: Option<Value>,
}

fn lease_to_resv_lite(rec: &Map<String, Value>) -> MR<ResvLite> {
    let resource = match rec.get("resource") {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(Fail::Delegate),
    };
    let agent = rec.get("workspace_id").map(|w| match w {
        Value::String(s) if s.starts_with("agent:") => Value::String(s["agent:".len()..].to_string()),
        other => other.clone(),
    });
    let session = match rec.get("session_id") {
        Some(v) if js_truthy(v) && !matches!(v, Value::String(s) if s == rsv::SESSIONLESS_SESSION_ID) => {
            Some(v.clone())
        }
        _ => None,
    };
    Ok(ResvLite {
        agent,
        cell: rec.get("workflow_id").cloned(),
        path: resource["path:".len()..].to_string(),
        session,
    })
}

struct ReleaseOutcome {
    paths: Vec<String>,
    /// Mirrors Node's holdsReleased (reservations-release parity); the finish
    /// text/result never surfaces it — kept for the ledger write's own count.
    #[allow(dead_code)]
    holds_released: u64,
}

/// bee.mjs releaseReservationsForAgent(root, agent, cell) — matched-rows
/// derivation, local lease release, {cell, session}-scoped ledger release.
fn release_reservations_for_agent(root: &Path, agent: &str, cell_id: &str) -> MR<ReleaseOutcome> {
    let now = rsv::now_ms();
    let records = list_path_lease_records(root)?;
    let mut matched: Vec<ResvLite> = Vec::new();
    for rec in &records {
        if lease_record_expired(rec, now)? {
            continue; // activeOnly
        }
        let resv = lease_to_resv_lite(rec)?;
        let agent_match = matches!(&resv.agent, Some(Value::String(s)) if s == agent);
        let cell_match =
            matches!(&resv.cell, Some(v) if rsv::js_strict_eq(v, &Value::String(cell_id.to_string())));
        if agent_match && cell_match {
            matched.push(resv);
        }
    }
    let mut pairs: Vec<(Value, Option<Value>)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for r in &matched {
        let Some(cell_v) = r.cell.as_ref().filter(|c| js_truthy(c)) else { continue };
        let session_v = r.session.as_ref().filter(|s| js_truthy(s)).cloned();
        let key = format!(
            "{}::{}",
            jsjson::js_to_string(cell_v),
            session_v.as_ref().map(|s| jsjson::js_to_string(s)).unwrap_or_default()
        );
        if !seen.contains(&key) {
            seen.push(key);
            pairs.push((cell_v.clone(), session_v));
        }
    }

    // reservations.mjs release(root, {agent, cell}).
    let control = control_root(root)?;
    let trimmed_agent = js_trim(agent);
    for rec in &records {
        let lease_agent = match rec.get("workspace_id") {
            Some(Value::String(s)) if s.starts_with("agent:") => {
                Value::String(s["agent:".len()..].to_string())
            }
            Some(other) => other.clone(),
            None => continue,
        };
        if !matches!(&lease_agent, Value::String(s) if s == trimmed_agent) {
            continue;
        }
        let matches_cell = matches!(
            rec.get("workflow_id"),
            Some(v) if rsv::js_strict_eq(v, &Value::String(cell_id.to_string()))
        );
        if !matches_cell {
            continue;
        }
        let resource = match rec.get("resource") {
            Some(Value::String(s)) => s.clone(),
            _ => continue,
        };
        let file = path_lease_file(&control, &resource["path:".len()..]);
        match std::fs::remove_file(&file) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Fail::Thrown(format!("{e}"))),
        }
    }

    // xwh-2/gfb-1: ledger release per {cell, session} pair (holder 'main').
    let mut holds_released: u64 = 0;
    for (cell_v, session_v) in &pairs {
        let mut guard = acquire_named_lock(root, CROSS_WORKTREE_HOLDS_LOCK)?;
        let outcome = (|| -> MR<u64> {
            let mut store = read_holds_store(root)?;
            let released_at = utc_now();
            let mut count: u64 = 0;
            if let Some(Value::Array(holds)) = store.get_mut("holds") {
                for hold in holds.iter_mut() {
                    let unreleased = matches!(hold.get("released_at"), None | Some(Value::Null));
                    if !unreleased {
                        continue;
                    }
                    if !matches!(hold.get("holder"), Some(Value::String(s)) if s == "main") {
                        continue;
                    }
                    if let Some(s) = session_v {
                        if !matches!(hold.get("session"), Some(v) if rsv::js_strict_eq(v, s)) {
                            continue;
                        }
                    }
                    if !matches!(hold.get("cell"), Some(v) if rsv::js_strict_eq(v, cell_v)) {
                        continue;
                    }
                    if let Value::Object(m) = hold {
                        m.insert("released_at".into(), Value::String(released_at.clone()));
                    }
                    count += 1;
                }
            }
            if count > 0 {
                write_json_atomic(&holds_ledger_path(root), &store)
                    .map_err(|e| Fail::Thrown(format!("{e}")))?;
            }
            Ok(count)
        })();
        guard.release();
        holds_released += outcome?;
    }

    let mut paths: Vec<String> = Vec::new();
    for r in &matched {
        if !paths.contains(&r.path) {
            paths.push(r.path.clone());
        }
    }
    Ok(ReleaseOutcome { paths, holds_released })
}

// RETIRED at the R6 cutover: the cap-time impact-registry cross-check (E1).
// It queried `scripts/impact-registry.json` — a suite-impact graph derived
// by parsing `scripts/run_verify.mjs` and the `.mjs` import closure — to warn
// when a cell's verify command missed a direct-edge Node suite. Both the
// registry and the graph it was derived from are gone with the Node tree, and
// the cargo suite that replaced them runs whole in ~20s, so there is no
// filtering left to advise about. `trace.warnings` keeps its slot (existing
// capped cells carry it) but this producer no longer exists.

// ─── delegation pre-scans ──────────────────────────────────────────────────
// The mutators must never return None after an output or a write, so the
// JS-exotic store shapes that still delegate (an array where a record is
// expected, a string/array `trace`) are probed up front; Thrown-class
// outcomes are ignored here (the real flow reproduces them at Node's own
// point in the order).
//
// CUTOVER: `prescan_cells_store` used to walk EVERY active and archived cell
// file so a corrupt one could delegate before any output. Corrupt JSON is
// native now, and that walk would have warned about cell files the command
// never reads — so it is deleted rather than kept as a second, louder read.
// The probes that survive read exactly the file the flow reads, and
// `warn_corrupt_json_once` keeps that from printing twice.

fn delegate_only<T>(result: MR<T>) -> MR<()> {
    match result {
        Err(Fail::Delegate) => Err(Fail::Delegate),
        _ => Ok(()),
    }
}

fn prescan_claim(root: &Path, id: &str) -> MR<()> {
    let control = control_root(root)?;
    if id_pattern_ok(id) {
        delegate_only(read_claim(&control, id))?;
    }
    Ok(())
}

// ─── verb handlers ─────────────────────────────────────────────────────────

fn dispatch(
    cmd: &'static str,
    use_json: bool,
    t0: Instant,
    f: impl FnOnce(&rsv::Ctx) -> MR<Out>,
) -> Option<ExitCode> {
    let ctx = match rsv::prelude(cmd, use_json, t0)? {
        rsv::Pre::Go(c) => c,
        rsv::Pre::Emitted(code) => return Some(code),
    };
    let out = f(&ctx);
    rsv::finish(&ctx, to_r2(out))
}

/// requireFlag(flags, name) — Missing/empty/boolean-true refuse with the
/// handler's own deterministic message (validate() never guards these
/// verbs' optional-at-schema flags).
fn require_flag_native(flags: &rsv::Flags, name: &str) -> MR<String> {
    match flags.get(name) {
        Some(FlagV::S(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(Fail::Thrown(format!("Missing required flag --{name}."))),
    }
}

fn read_file_text(file: &str, label: &str) -> MR<String> {
    match std::fs::read(file) {
        Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => Err(Fail::Thrown(format!("Cannot read {label} file: {file}"))),
    }
}

fn read_stdin_text() -> MR<String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    match std::io::stdin().lock().read_to_end(&mut bytes) {
        Ok(_) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(e) => Err(Fail::Thrown(format!("{e}"))),
    }
}

const RELEASE_MANIFEST_LINT_PATH: &str = "docs/history/codex-harness-hardening/release-manifest.json";

/// bee.mjs manifestLintWarning + emitManifestLintWarnings (stderr).
fn emit_manifest_lint_warnings(cells: &[Value]) {
    for cell in cells {
        let Value::Object(map) = cell else { continue };
        let verify = match map.get("verify") {
            Some(Value::String(s)) => s,
            _ => continue,
        };
        if !verify.contains("release_manifest") {
            continue;
        }
        let files = match map.get("files") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        if files.iter().any(|f| matches!(f, Value::String(s) if s == RELEASE_MANIFEST_LINT_PATH)) {
            continue;
        }
        let id = match map.get("id") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            _ => "(unknown id)".to_string(),
        };
        eprintln!(
            "WARNING: cell \"{id}\" verify mentions release_manifest but files is missing \"{RELEASE_MANIFEST_LINT_PATH}\" — a cold worker will hit red verify with no sanctioned fix. FIX: add the manifest path to files; regenerate it only via \"bee dev release-manifest --write\"."
        );
    }
}

// ── cells add ──────────────────────────────────────────────────────────────

/// buildAddCellsReport row.
struct AddReportRow {
    id: String,
    ok: bool,
    problems: Vec<String>,
}

fn build_add_cells_report(root: &Path, cells: &[Value]) -> MR<(bool, Vec<AddReportRow>, Option<Vec<Value>>)> {
    let mut seen: Vec<String> = Vec::new();
    let mut rows: Vec<AddReportRow> = Vec::new();
    for (index, cell) in cells.iter().enumerate() {
        let id = match cell.get("id") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            _ => format!("(index {index})"),
        };
        let mut problems: Vec<String> = Vec::new();
        match validate_new_cell(root, cell) {
            Ok(()) => {}
            Err(Fail::Thrown(message)) => problems.push(message),
            Err(Fail::Delegate) => return Err(Fail::Delegate),
        }
        if let Some(Value::String(cid)) = cell.get("id") {
            if !cid.is_empty() {
                if seen.contains(cid) {
                    problems.push(format!("addCells: duplicate id \"{cid}\" within the batch."));
                } else {
                    seen.push(cid.clone());
                }
            }
        }
        rows.push(AddReportRow { id, ok: problems.is_empty(), problems });
    }
    let mut normalized: Option<Vec<Value>> = None;
    if rows.iter().all(|r| r.ok) {
        let mut list = Vec::new();
        for cell in cells {
            list.push(normalize_new_cell(cell)?);
        }
        let cycles = compute_incoming_cycles(root, &list)?;
        if !cycles.is_empty() {
            let cycle_ids: Vec<String> = cycles.iter().flatten().cloned().collect();
            let message = format_cycle_refusal("addCells", &cycles);
            for row in rows.iter_mut() {
                if cycle_ids.contains(&row.id) {
                    row.problems.push(message.clone());
                    row.ok = false;
                }
            }
        } else {
            normalized = Some(list);
        }
    }
    let ok = rows.iter().all(|r| r.ok);
    Ok((ok, rows, if ok { normalized } else { None }))
}

fn add_report_rows_value(rows: &[AddReportRow]) -> Value {
    Value::Array(
        rows.iter()
            .map(|r| {
                let mut m = Map::new();
                m.insert("id".into(), Value::String(r.id.clone()));
                m.insert("ok".into(), Value::Bool(r.ok));
                m.insert(
                    "problems".into(),
                    Value::Array(r.problems.iter().map(|p| Value::String(p.clone())).collect()),
                );
                Value::Object(m)
            })
            .collect(),
    )
}

fn run_add(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["file", "stdin", "dry-run"]) {
        return None;
    }
    let stdin = bool_flag(&flags, "stdin")?;
    // `flags['dry-run'] !== undefined` — a "false" string still triggers it.
    let dry_run = match flags.get("dry-run") {
        None => false,
        Some(FlagV::Present) => true,
        Some(FlagV::S(s)) if s == "true" || s == "false" => true,
        Some(FlagV::S(_)) => return None,
    };
    dispatch("cells add", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        delegate_only(read_commands_slice(&root))?;
        let text = if stdin {
            read_stdin_text()?
        } else {
            let file = require_flag_native(&flags, "file")?;
            read_file_text(&file, "cell")?
        };
        // Lone surrogates and |n| >= 1e21 used to fork here; both are gone
        // (see parse_json_js), so every unparseable payload — file or stdin —
        // takes the one refusal Node's `catch` threw.
        let payload = match parse_json_js(&text, false) {
            JsParse::Value(v) => v,
            JsParse::NotJson => return Err(Fail::Thrown("add: input is not valid JSON.".into())),
        };
        if dry_run {
            let batch: Vec<Value> = match &payload {
                Value::Array(a) => a.clone(),
                other => vec![other.clone()],
            };
            if batch.is_empty() {
                return Err(Fail::Thrown(
                    "previewAddCells: expected a non-empty JSON array of cells.".into(),
                ));
            }
            let (ok, rows, _) = build_add_cells_report(&root, &batch)?;
            let mut lines: Vec<String> = Vec::new();
            let failing = rows.iter().filter(|r| !r.ok).count();
            lines.push(if ok {
                format!("dry-run: {} cell(s) valid — nothing written.", batch.len())
            } else {
                format!(
                    "dry-run: {failing} of {} cell(s) failed validation — nothing written.",
                    batch.len()
                )
            });
            for r in &rows {
                lines.push(format!(
                    "{} {}{}",
                    if r.ok { "OK" } else { "FAIL" },
                    r.id,
                    if r.problems.is_empty() { String::new() } else { format!(": {}", r.problems.join("; ")) }
                ));
            }
            let mut result = Map::new();
            result.insert("dry_run".into(), Value::Bool(true));
            result.insert("ok".into(), Value::Bool(ok));
            result.insert("cells".into(), add_report_rows_value(&rows));
            return Ok(Out::Emit(Value::Object(result), lines.join("\n"), if ok { 0 } else { 1 }));
        }
        if let Value::Array(batch) = &payload {
            if batch.is_empty() {
                return Err(Fail::Thrown("addCells: expected a non-empty JSON array of cells.".into()));
            }
            let (ok, rows, normalized) = build_add_cells_report(&root, batch)?;
            if !ok {
                let failing: Vec<&AddReportRow> = rows.iter().filter(|r| !r.ok).collect();
                let named = failing
                    .iter()
                    .map(|r| format!("{} ({})", r.id, r.problems.join("; ")))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Fail::Thrown(format!(
                    "addCells: {} of {} cell(s) failed validation — {named}. Nothing written.",
                    failing.len(),
                    batch.len()
                )));
            }
            let normalized = normalized.expect("ok report carries normalized cells");
            for cell in &normalized {
                write_cell(&root, cell)?;
            }
            emit_manifest_lint_warnings(&normalized);
            let text = normalized
                .iter()
                .map(|c| format!("Added {}", summarize_cell(c)))
                .collect::<Vec<_>>()
                .join("\n");
            return Ok(Out::Emit(Value::Array(normalized), text, 0));
        }
        validate_new_cell(&root, &payload)?;
        let normalized = normalize_new_cell(&payload)?;
        assert_no_cycle(&root, "addCell", std::slice::from_ref(&normalized))?;
        write_cell(&root, &normalized)?;
        emit_manifest_lint_warnings(std::slice::from_ref(&normalized));
        let text = format!("Added {}", summarize_cell(&normalized));
        Ok(Out::Emit(normalized, text, 0))
    })
}

// ── cells update ───────────────────────────────────────────────────────────

const UPDATE_FIELDS: [&str; 13] = [
    "title",
    "action",
    "verify",
    "files",
    "read_first",
    "deps",
    "decisions",
    "must_haves",
    "behavior_change",
    "lane",
    "pbi",
    "change_class",
    REGEN_ACK_FIELD,
];

fn update_field_problem(key: &str, value: &Value) -> Option<String> {
    let bad = |msg: &str| Some(msg.to_string());
    match key {
        "title" | "action" | "verify" => {
            if nonblank_string(Some(value)) {
                None
            } else {
                bad("must be a non-empty string")
            }
        }
        "files" | "read_first" | "deps" | "decisions" => {
            if is_string_array(value) {
                None
            } else {
                bad("must be an array of strings")
            }
        }
        "must_haves" => {
            if matches!(value, Value::Object(_)) {
                None
            } else {
                bad("must be a JSON object")
            }
        }
        "behavior_change" => {
            if matches!(value, Value::Bool(_)) {
                None
            } else {
                bad("must be a boolean")
            }
        }
        "lane" => {
            if matches!(value, Value::String(s) if LANES.contains(&s.as_str())) {
                None
            } else {
                Some(format!("must be one of: {}", LANES.join(", ")))
            }
        }
        "pbi" => {
            if matches!(value, Value::Null | Value::String(_)) {
                None
            } else {
                bad("must be a string or null")
            }
        }
        "change_class" => {
            if matches!(value, Value::Null)
                || matches!(value, Value::String(s) if CHANGE_CLASSES.contains(&s.as_str()))
            {
                None
            } else {
                Some(format!("must be null or one of: {}", CHANGE_CLASSES.join(", ")))
            }
        }
        _ if key == REGEN_ACK_FIELD => {
            if matches!(value, Value::Null) || nonblank_string(Some(value)) {
                None
            } else {
                bad("must be null or a non-empty string (the one-line reason for skipping the derived regen obligation)")
            }
        }
        _ => unreachable!("caller checks membership"),
    }
}

fn update_frozen_hint(key: &str) -> Option<&'static str> {
    match key {
        "id" => Some("a cell id is permanent — add a new cell instead"),
        "feature" => Some("a cell never moves between features — drop and re-add instead"),
        "status" => Some("status moves only through claim/verify/cap/block/drop"),
        "trace" => Some("the trace is the frozen audit record — claim/verify/cap own it"),
        "tier" => Some("use the tier verb (bee cells tier --id ID --tier T)"),
        _ => None,
    }
}

fn run_update(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "file", "stdin"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string(); // schema-required: missing -> validate() -> Node
    let stdin = bool_flag(&flags, "stdin")?;
    dispatch("cells update", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        delegate_only(read_commands_slice(&root))?;
        let text = if stdin {
            read_stdin_text()?
        } else {
            let file = require_flag_native(&flags, "file")?;
            read_file_text(&file, "patch")?
        };
        let patch = match parse_json_js(&text, false) {
            JsParse::Value(v) => v,
            JsParse::NotJson => {
                return Err(Fail::Thrown("update: patch input is not valid JSON.".into()))
            }
        };
        // updateCell — pure validation before the lock.
        if id.is_empty() || !id_pattern_ok(&id) {
            return Err(Fail::Thrown(format!("updateCell: invalid id \"{id}\".")));
        }
        let patch_map = match &patch {
            Value::Object(m) => m.clone(),
            _ => return Err(Fail::Thrown("updateCell: patch must be a JSON object.".into())),
        };
        if patch_map.is_empty() {
            return Err(Fail::Thrown("updateCell: patch is empty — nothing to update.".into()));
        }
        for (key, value) in &patch_map {
            if !UPDATE_FIELDS.contains(&key.as_str()) {
                let message = match update_frozen_hint(key) {
                    Some(hint) => format!(
                        "updateCell: field \"{key}\" is frozen — {hint}. The whole patch is refused; the cell is untouched."
                    ),
                    None => format!(
                        "updateCell: unknown field \"{key}\" — updatable fields: {}. The whole patch is refused; the cell is untouched.",
                        UPDATE_FIELDS.join(", ")
                    ),
                };
                return Err(Fail::Thrown(message));
            }
            if let Some(problem) = update_field_problem(key, value) {
                return Err(Fail::Thrown(format!(
                    "updateCell: field \"{key}\" {problem}. The whole patch is refused; the cell is untouched."
                )));
            }
        }
        if let Some(verify) = patch_map.get("verify") {
            assert_verify_sentinel_allowed(&root, "updateCell", verify)?;
        }

        let mut guard = acquire_named_lock(&root, &format!("cells:{id}"))?;
        let outcome = (|| -> MR<Value> {
            assert_not_archived(&root, "updateCell", &id)?;
            // readCellStrictForUpdate — raw read, no BOM strip.
            let file = cell_file(&root, &id);
            let raw = match std::fs::read(&file) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Err(Fail::Thrown(format!("updateCell: cell \"{id}\" not found.")))
                }
                // readCellStrictForUpdate's unreadable branch (lib/cells.mjs
                // :1474). Node interpolated err.code; this carries the Rust
                // io error in the same sentence, same refusal.
                Err(e) => {
                    return Err(Fail::Thrown(format!(
                        "updateCell: could not read \"{}\" ({e}) — refusing to touch it. FIX: inspect/restore the file, then retry.",
                        file.display()
                    )))
                }
            };
            let sep = std::path::MAIN_SEPARATOR;
            let rel = format!(".bee{sep}cells{sep}{id}.json");
            let cell_map = match parse_json_js(&raw, false) {
                JsParse::Value(Value::Object(m)) => m,
                JsParse::Value(_) => {
                    return Err(Fail::Thrown(format!(
                        "updateCell: \"{}\" exists but is not a JSON object — refusing to merge a patch over a corrupt cell.",
                        file.display()
                    )))
                }
                // Lone surrogates land here too — a cell file this CLI cannot
                // parse is corrupt, and the refusal is the same either way.
                JsParse::NotJson => {
                    return Err(Fail::Thrown(format!(
                        "updateCell: \"{}\" exists but is not valid JSON — refusing to merge a patch over a corrupt cell. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\"), then retry.",
                        file.display()
                    )))
                }
            };
            let status_ok = matches!(cell_map.get("status"), Some(Value::String(s)) if s == "open" || s == "blocked");
            if !status_ok {
                return Err(Fail::Thrown(format!(
                    "updateCell: cell \"{id}\" has status \"{}\" — only open or blocked cells are updatable (claimed = a live worker owns it; capped/dropped = frozen audit). The cell is untouched.",
                    js_string_or_undefined(cell_map.get("status"))
                )));
            }
            let mut merged = cell_map.clone();
            spread_into(&mut merged, &patch_map);
            let merged_lane = match merged.get("lane") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            if merged_lane == "standard" || merged_lane == "high-risk" {
                let truths = merged
                    .get("must_haves")
                    .filter(|m| js_truthy(m))
                    .and_then(|m| m.get("truths"));
                if !matches!(truths, Some(Value::Array(a)) if !a.is_empty()) {
                    return Err(Fail::Thrown(format!(
                        "updateCell: lane \"{merged_lane}\" requires non-empty must_haves.truths — the patch would leave \"{id}\" without them. The cell is untouched."
                    )));
                }
            }
            let merged_value = Value::Object(merged.clone());
            if patch_map.contains_key("deps") {
                assert_no_cycle(&root, "updateCell", std::slice::from_ref(&merged_value))?;
            }
            assert_regen_obligation(&merged, "updateCell")?;
            write_cell(&root, &merged_value)?;
            Ok(merged_value)
        })();
        guard.release();
        let updated = outcome?;
        emit_manifest_lint_warnings(std::slice::from_ref(&updated));
        let keys: Vec<String> = patch_map.keys().cloned().collect();
        let text = format!(
            "Updated {} ({}).",
            js_string_or_undefined(updated.get("id")),
            keys.join(", ")
        );
        Ok(Out::Emit(updated, text, 0))
    })
}

// ── cells claim ────────────────────────────────────────────────────────────

fn run_claim(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "worker", "session-id", "ttl", "isolate"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let worker = flags.req_str("worker")?.to_string();
    let session_flag = opt_string_flag(&flags, "session-id")?;
    let _isolate = bool_flag(&flags, "isolate")?;
    let ttl: Option<f64> = match flags.get("ttl") {
        None => None,
        Some(FlagV::Present) => return None,
        Some(FlagV::S(s)) => match rsv::js_number_flag(s) {
            Err(_) => return None, // validate() refuses the shape — Node's message
            Ok(parsed) => Some(parsed.unwrap_or(f64::NAN)),
        },
    };
    dispatch("cells claim", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let claimed = claim_cell_from_flags(
            &root,
            &id,
            &worker,
            session_flag.as_deref(),
            ttl,
        )?
        .cell;
        let worker_disp = match claimed.get("trace").and_then(|t| t.get("worker")) {
            Some(v) => jsjson::js_to_string(v),
            None => "undefined".to_string(),
        };
        let text = format!(
            "Claimed {} for {}.",
            js_string_or_undefined(claimed.get("id")),
            worker_disp
        );
        Ok(Out::Emit(claimed, text, 0))
    })
}

/// claimCellFromFlags's product — bee.mjs returns `{cell, sessionId}`.
///
/// `policy` has no variant here on purpose: `applyWritePolicy` runs with
/// `enforceIsolation: false` on this door, so its only acting arms are
/// `observe` (a no-op) and `shared-disjoint` (a refusal); the `isolated`
/// workspace-attach / auto-isolate machinery that produces `redirect: true`
/// is structurally unreachable. Node's `if (policy.redirect) return {policy}`
/// in claimCellFromFlags — and therefore `handleDispatchPrepare`'s own
/// `if (claimOutcome.policy)` early return — are both provably dead code for
/// every argv this door serves. (Same reasoning already recorded for
/// claim-next below.)
pub(crate) struct ClaimDoor {
    pub(crate) cell: Value,
    pub(crate) session_id: Option<String>,
}

/// bee.mjs's `claimCellFromFlags` — "One claim door for cells.claim and
/// dispatch.prepare --claim": the write-policy resolution, the claims.mjs
/// claim-file-first sequence, the byte-identical claim refusal and the route
/// soft-warning, all in ONE body so the door cannot diverge between the two
/// verbs. `cells claim` adds only its own emit text; `dispatch prepare
/// --claim` adds only the reserve loop that follows it.
///
/// pub(crate) since the `dispatch prepare --claim` port — previously this was
/// inlined in `run_claim`'s closure, which is exactly why that verb delegated.
///
/// Every delegate-trigger is FRONT-LOADED (the two prescans, the store reads,
/// the exotic-shape probes) because nothing after claimCellFile's O_EXCL
/// write may delegate: the claim file would already exist for the Node re-run.
pub(crate) fn claim_cell_from_flags(
    root: &Path,
    id: &str,
    worker: &str,
    session_flag: Option<&str>,
    ttl: Option<f64>,
) -> MR<ClaimDoor> {
    let root = root.to_path_buf();
    let id = id.to_string();
    {
        if let Some(t) = ttl {
            if !t.is_finite() || t <= 0.0 {
                return Err(Fail::Thrown("--ttl must be a positive integer (seconds).".into()));
            }
        }
        // Pre-scan: everything after claimCellFile's O_EXCL write must never
        // delegate (the file would already exist for a retry).
        prescan_claim(&root, &id)?;
        let control = control_root(&root)?;
        delegate_only(list_session_records(&control))?;
        delegate_only(bstate::read_state_brief(&root).map_err(|_| Fail::Delegate))?;
        let config = bstate::read_config_raw(&root).map_err(|_| Fail::Delegate)?;
        let cell_for_policy = read_cell_norm(&root, &id)?;
        if let Some(cell) = &cell_for_policy {
            if !matches!(cell, Value::Object(_)) {
                return Err(Fail::Delegate); // truthy non-object cell — JS-exotic downstream
            }
            delegate_only(merge_trace(cell.get("trace")))?;
            delegate_only(lane_record_gates(&root, cell.get("feature")))?;
            // CUTOVER: the read_lane_route probe that stood here had exactly
            // one delegating arm — a corrupt/mismatched lane record — which
            // is native now. Keeping it would print readLane's warning twice.
            match cell.get("deps") {
                None => {}
                Some(deps) if !js_truthy(deps) => {}
                Some(Value::Array(_)) => {}
                Some(_) => return Err(Fail::Delegate), // truthy non-array deps
            }
        }

        let session_id = resolve_session_flag_env(session_flag);

        // applyWritePolicy (state.mjs) with enforceIsolation:false — only the
        // observe/shared-disjoint arms can act; 'isolated' passes through.
        let mode = match config.get("guards").and_then(|g| g.get("write_policy")) {
            Some(Value::String(s)) if js_trim(s) == "observe" => "observe",
            Some(Value::String(s)) if js_trim(s) == "shared-disjoint" => "shared-disjoint",
            _ => "isolated",
        };
        if mode == "shared-disjoint" {
            let declared: Vec<String> = match cell_for_policy.as_ref().and_then(|c| c.get("files")) {
                Some(Value::Array(files)) => files
                    .iter()
                    .filter_map(|f| match f {
                        Value::String(s) if !js_trim(s).is_empty() => Some(s.clone()),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            if !declared.is_empty() {
                let now = rsv::now_ms();
                let records = list_path_lease_records(&root)?;
                let mut active: Vec<ResvLite> = Vec::new();
                for rec in &records {
                    if lease_record_expired(rec, now)? {
                        continue;
                    }
                    active.push(lease_to_resv_lite(rec)?);
                }
                let missing: Vec<String> = declared
                    .iter()
                    .filter(|p| {
                        !active.iter().any(|r| {
                            let session_match = match (&session_id, &r.session) {
                                (Some(sid), Some(Value::String(s))) => s == sid,
                                _ => false,
                            };
                            session_match && !r.path.ends_with('*') && rsv::paths_overlap(&r.path, p)
                        })
                    })
                    .cloned()
                    .collect();
                if !missing.is_empty() {
                    let session_suffix = session_id
                        .as_deref()
                        .map(|s| format!(" --session-id {s}"))
                        .unwrap_or_default();
                    return Err(Fail::Thrown(format!(
                        "bee write-policy (shared-disjoint): no exact-path lease held for: {}. A broad/glob reservation never satisfies shared-disjoint — an exact-path lease is mandatory before write. FIX: bee reservations reserve --agent <worker> --cell <id> --path <path>{session_suffix} for each path, then retry.",
                        missing.join(", ")
                    )));
                }
            }
        }

        // claimCellCrossSession (shared with claim-next — see its own comment).
        let session = session_id.clone();
        let cell_id = js_trim(&id).to_string();
        let claimed = match claim_cell_cross_session(
            &root,
            &control,
            session.as_deref(),
            worker,
            &id,
            ttl,
            cell_for_policy.as_ref(),
        )? {
            CrossClaim::Ok { cell, .. } => cell,
            CrossClaim::Refused { code, reason } => {
                return Err(Fail::Thrown(format!("claim: {code} — {reason}")));
            }
        };
        let _ = &cell_id;
        // explicit-triage D3 soft route warning (stderr, never a refusal).
        if !claimed_feature_has_route(&root, claimed.get("feature"))? {
            eprint!(
                "WARNING: cell \"{}\" claimed for feature \"{}\" with no route record — run \"bee state route --set --class <c> --lane <l> --flags <f> --files <n>\" to record the triage (D3, soft enforcement).\n",
                js_string_or_undefined(claimed.get("id")),
                js_string_or_undefined(claimed.get("feature"))
            );
        }
        Ok(ClaimDoor { cell: claimed, session_id })
    }
}

/// claimCellCrossSession's typed outcome. Node returns `{ok:false, code,
/// reason}`; each CLI caller prefixes it with its own verb word (`claim: …`
/// / `claim-next: …`), so the refusal stays typed until then.
enum CrossClaim {
    /// `{ok:true, cell, claim}` — `cells claim` reads only the cell,
    /// `cells claim-next` emits the whole envelope.
    Ok { cell: Value, claim: Value },
    Refused { code: String, reason: String },
}

/// lib/cells.mjs claimCellCrossSession — the CLAIM half shared by
/// `cells claim` and `cells claim-next`: claimCellFile's O_EXCL protocol, the
/// budget unwind (releaseClaim before surfacing, so a refused acquisition
/// never orphans a claims-store file), then claimCell under the `cells:<id>`
/// store lock with every throw unwinding into CLAIM_CELL_FAILED.
///
/// `cell_for_budget` is the caller's already-read cell record — Node re-reads
/// it here (`readCell(root, id)`); both callers pre-read it in the same
/// command, and the store cannot change under this process between the two
/// points, so the read is hoisted rather than repeated.
#[allow(clippy::too_many_arguments)]
fn claim_cell_cross_session(
    root: &Path,
    control: &Path,
    session: Option<&str>,
    worker: &str,
    cell_id_in: &str,
    ttl: Option<f64>,
    cell_for_budget: Option<&Value>,
) -> MR<CrossClaim> {
    if js_trim(worker).is_empty() {
        return Err(Fail::Thrown("claimCellCrossSession: worker is required.".into()));
    }
    if js_trim(cell_id_in).is_empty() {
        return Err(Fail::Thrown("claimCellCrossSession: cellId is required.".into()));
    }
    let cell_id = js_trim(cell_id_in).to_string();
    let file_claim = match claim_cell_file(control, session, &cell_id, ttl)? {
        ClaimFileOutcome::Refused { code, reason } => {
            return Ok(CrossClaim::Refused { code: code.to_string(), reason });
        }
        ClaimFileOutcome::Ok { claim } => claim,
    };
    // Budget check inside the O_EXCL window.
    if let Some(Value::Object(cell_map)) = cell_for_budget {
        match check_cell_budgets(cell_map) {
            Ok(BudgetCheck::Ok) => {}
            Ok(BudgetCheck::Refused { code, reason }) => {
                release_claim(control, session, &cell_id)?;
                return Ok(CrossClaim::Refused { code: code.to_string(), reason });
            }
            Err(fail) => {
                // Pre-scanned; a mid-command race lands here — unwind the
                // claim file before surfacing anything.
                release_claim(control, session, &cell_id)?;
                return Err(fail);
            }
        }
    }
    // claimCell under the per-cell store lock; every throw unwinds the
    // claim file and surfaces as CLAIM_CELL_FAILED.
    let claim_result = (|| -> MR<Value> {
        let mut guard = acquire_named_lock(root, &format!("cells:{cell_id}"))?;
        let outcome = (|| -> MR<Value> {
            let root = root;
            let worker = worker;
            {
                assert_not_archived(root, "claimCell", &cell_id)?;
                let cell = read_cell_norm(root, &cell_id)?;
                let lane_gates = match &cell {
                    Some(c) if js_truthy(c) => lane_record_gates(&root, c.get("feature"))?,
                    _ => None,
                };
                let approved = match &lane_gates {
                    Some(gates) => matches!(gates.get("execution"), Some(Value::Bool(true))),
                    None => default_gate_approved(&root, "execution")?,
                };
                if !approved {
                    let message = match (&lane_gates, &cell) {
                        (Some(_), Some(c)) => format!(
                            "claimCell: lane \"{}\" gate \"execution\" is not approved — cells of this feature cannot be claimed before ITS lane passes Gate 3 (D2: only the lane's own approvals authorize its cells — the default pipeline's gate never does). Surface Gate 3 to the user for lane \"{}\" and set its approved_gates.execution once approved.",
                            js_string_or_undefined(c.get("feature")),
                            js_string_or_undefined(c.get("feature"))
                        ),
                        _ => "claimCell: gate \"execution\" is not approved — cells cannot be claimed before execution is approved. Surface Gate 3 to the user (\"Feasibility validated. Approve execution?\") and set approved_gates.execution once approved. The opt-in gate_bypass switch may self-approve: level \"normal\" covers tiny/small/standard non-hard-gate work only; levels \"full\" and \"total\" also self-approve high-risk/hard-gate execution (decision 0010, total-autopilot dcf01d7b).".to_string(),
                    };
                    return Err(Fail::Thrown(message));
                }
                let Some(cell) = cell else {
                    return Err(Fail::Thrown(format!("claimCell: cell \"{cell_id}\" not found.")));
                };
                let status_open = matches!(cell.get("status"), Some(Value::String(s)) if s == "open");
                if !status_open {
                    return Err(Fail::Thrown(format!(
                        "claimCell: cell \"{cell_id}\" is \"{}\", not \"open\" — only open cells can be claimed. Run bee cells ready to list claimable cells.",
                        js_string_or_undefined(cell.get("status"))
                    )));
                }
                // depsAllCapped (cells.mjs flavor — collects misses).
                let mut uncapped: Vec<Value> = Vec::new();
                if let Some(deps) = cell.get("deps") {
                    if js_truthy(deps) {
                        let Value::Array(deps) = deps else { return Err(Fail::Delegate) };
                        for dep in deps {
                            let dep_id = jsjson::js_to_string(dep);
                            let capped = match read_cell_norm(&root, &dep_id)? {
                                Some(dep_cell) => {
                                    matches!(dep_cell.get("status"), Some(Value::String(s)) if s == "capped")
                                }
                                None => false,
                            };
                            if !capped {
                                uncapped.push(dep.clone());
                            }
                        }
                    }
                }
                if !uncapped.is_empty() {
                    return Err(Fail::Thrown(format!(
                        "claimCell: cell \"{cell_id}\" has uncapped deps: {} — deps must be capped first.",
                        js_join(&uncapped, ", ")
                    )));
                }
                let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
                cell_map.insert("status".into(), Value::String("claimed".into()));
                let mut trace = merge_trace(cell_map.get("trace"))?;
                trace.insert("worker".into(), Value::String(js_trim(worker).to_string()));
                trace.insert(
                    "claim_session".into(),
                    session.map(|s| Value::String(s.to_string())).unwrap_or(Value::Null),
                );
                trace.insert("claimed_at".into(), Value::String(utc_now()));
                cell_map.insert("trace".into(), Value::Object(trace));
                let cell_value = Value::Object(cell_map);
                write_cell(root, &cell_value)?;
                Ok(cell_value)
            }
        })();
        guard.release();
        outcome
    })();
    match claim_result {
        Ok(cell) => Ok(CrossClaim::Ok { cell, claim: file_claim }),
        Err(Fail::Thrown(message)) => {
            release_claim(control, session, &cell_id)?;
            Ok(CrossClaim::Refused { code: "CLAIM_CELL_FAILED".into(), reason: message })
        }
        Err(Fail::Delegate) => {
            // Pre-scanned; only a mid-command race lands here. Unwind so the
            // Node re-run isn't refused by our own claim file.
            release_claim(control, session, &cell_id)?;
            Err(Fail::Delegate)
        }
    }
}

// ─── cells claim-next (R6 coverage debt — the SELECTION half + the sweep) ──
//
// Provenance: bee.mjs handleCellsClaimNext, lib/cells.mjs claimNextCell /
// resolveHoldTopology, lib/claims.mjs sweepExpiredClaims, lib/state.mjs
// resolvePipeline / applyWritePolicy, lib/reservations.mjs
// findSessionConflicts, lib/worktree-holds.mjs findForeignHolds /
// isActive / isExpired, lib/backlog.mjs featureBacklogRank (ported in
// verbs/backlog.rs and imported here).
//
// WHY THE SWEEP IS SAFE TO RUN NATIVELY. sweepExpiredClaims mutates (claim
// files removed, claimed->open cell resets, one decision row per reset)
// BEFORE selection reads anything, so the usual "return None and let Node
// re-run" escape would ordinarily double-write. It does not here, because the
// sweep removes its own trigger: every row it writes is gated on a claim FILE
// that it then deletes, so a Node re-run finds `readClaim` null for exactly
// those cells and writes nothing a second time. The pre-scan below still
// front-loads every delegation trigger it can, so a mid-flight delegate is a
// concurrent-writer race, not a routine path — but when one does happen the
// end state and the emitted bytes are Node's own.
//
// applyWritePolicy is a NO-OP for this verb by construction: claim-next
// passes `paths: []` and `enforceIsolation: false`, so 'observe' returns
// immediately, 'shared-disjoint' short-circuits on the empty declared list,
// and 'isolated' takes the `!enforceIsolation` passthrough. `policy.redirect`
// can never be true, so the redirect branch of handleCellsClaimNext is
// unreachable — only readConfig's own corrupt-file delegation survives.
//
// Root topology: rsv::prelude serves ORDINARY checkouts only, so
// resolveHoldTopology(root) is the constant `{mainRoot: root, holder:'main'}`
// (the same constant verbs/reservations.rs already documents), and
// controlRootFor(root) === root.

/// lib/claims.mjs sweepExpiredClaims (hardening-4b sweep-reset, rel180-2).
/// TTL expired AND owner heartbeat stale, both re-verified under the claim's
/// exclusive `<cell>.adopting` gate and — for a session-owned claim — under
/// the same `sessions` store lock heartbeatSession itself holds. Every
/// removal is followed by the claimed->open cell reset under
/// `cells:<id>` and one best-effort decision row.
fn sweep_expired_claims(control: &Path, now: f64) -> MR<()> {
    let dir = claims_dir(control);
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(()) };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    for entry in names {
        let Some(cell) = entry.strip_suffix(".json") else { continue };
        let Some(preview) = read_claim(control, cell)? else { continue }; // corrupt: never touch
        if !claim_expired(&preview, now)? {
            continue;
        }
        if !heartbeat_stale(read_session_of_claim(control, &preview)?.as_ref(), now)? {
            continue;
        }
        if !acquire_gate(control, cell)? {
            continue; // gate held by another in-flight adopt/sweep — skipped
        }
        let mut swept_claim: Option<Map<String, Value>> = None;
        let gated = (|| -> MR<()> {
            let Some(claim) = read_claim(control, cell)? else { return Ok(()) };
            if !claim_expired(&claim, now)? {
                return Ok(());
            }
            // `claim.session ?? null`; a sessionless claim has no heartbeat to
            // race against and skips the lock entirely (rel180-2).
            let owner_session = nullish(claim.get("session"));
            let _sessions_lock = if js_truthy(&owner_session) {
                match acquire_sessions_lock_bounded(control) {
                    Some(guard) => Some(guard),
                    None => return Ok(()), // never steal on contention — skipped
                }
            } else {
                None
            };
            if heartbeat_stale(read_session_of_claim(control, &claim)?.as_ref(), now)? {
                let file = claim_path(control, cell)?;
                let _ = transient_fs_retry(|| match std::fs::remove_file(&file) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    other => other,
                });
                swept_claim = Some(claim);
            }
            Ok(())
        })();
        release_gate(control, cell);
        gated?;
        let Some(swept) = swept_claim else { continue };
        let swept_session = nullish(swept.get("session"));
        let was_reset = sweep_reset_cell(control, cell, &swept_session, now)?;
        if was_reset {
            // Best-effort: the cell write above already committed, so a
            // decision-log failure must never read as the reset having failed.
            let owner_disp = if swept_session.is_null() {
                "none (sessionless)".to_string()
            } else {
                jsjson::js_to_string(&swept_session)
            };
            let _ = log_decision(
                control,
                &format!(
                    "\u{ab}sweep: cell \"{cell}\" reset claimed -> open \u{2014} swept session \"{owner_disp}\"'s expired, stale claim\u{bb}"
                ),
                "sweepExpiredClaims (hardening-4b) removed the abandoned claim file; the cell was still \"claimed\" by that exact session (trace.claim_session matched), so it is returned to open rather than left claimed-but-unclaimable forever.",
                &["claims", "sweep"],
            );
        }
    }
    Ok(())
}

/// `value ?? null` for an optional JSON field (undefined AND null collapse).
fn nullish(v: Option<&Value>) -> Value {
    match v {
        None | Some(Value::Null) => Value::Null,
        Some(other) => other.clone(),
    }
}

/// `readSession(root, claim.session)` — a non-string session makes
/// sessionPath's requireId throw, which readSession catches as "no session".
fn read_session_of_claim(
    control: &Path,
    claim: &Map<String, Value>,
) -> MR<Option<Map<String, Value>>> {
    match claim.get("session") {
        Some(Value::String(s)) => read_session(control, s),
        _ => Ok(None),
    }
}

/// claims.mjs SESSIONS_LOCK_NAME bounded acquire (15 × 20ms, acquire-once) —
/// the exact `sessions` lock heartbeatSession/bindSessionLane hold.
fn acquire_sessions_lock_bounded(root: &Path) -> Option<lock::LockGuard> {
    for attempt in 0..15u32 {
        match lock::acquire_store_lock_once(root, "sessions") {
            lock::AcquireOnce::Acquired(guard) => return Some(guard),
            lock::AcquireOnce::Busy { .. } => {
                if attempt + 1 < 15 {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            }
        }
    }
    None
}

/// The sweep's claimed->open reset, under the SAME `cells:<id>` store lock
/// every other cells.mjs mutator uses. `readCellForSweepReset` is claims.mjs's
/// own minimal `.bee/cells/<id>.json` read/write (never cells.mjs's readCell —
/// that would cycle), so it never consults the archive.
fn sweep_reset_cell(
    control: &Path,
    cell: &str,
    swept_session: &Value,
    now: f64,
) -> MR<bool> {
    let mut guard = acquire_named_lock(control, &format!("cells:{cell}"))?;
    let outcome = (|| -> MR<bool> {
        let file = cells_dir(control).join(format!("{cell}.json"));
        let record = match read_store_json(&file)? {
            Some(Value::Object(m)) => m,
            _ => return Ok(false), // !cellRecord (or a non-object: .status is undefined)
        };
        if !matches!(record.get("status"), Some(Value::String(s)) if s == "claimed") {
            return Ok(false);
        }
        // `(cellRecord.trace && cellRecord.trace.claim_session) ?? null`
        let current_session = match record.get("trace") {
            None => Value::Null,
            Some(t) if !js_truthy(t) => nullish(Some(t)),
            Some(Value::Object(t)) => nullish(t.get("claim_session")),
            Some(_) => Value::Null, // truthy non-object: .claim_session is undefined
        };
        if !rsv::js_strict_eq(&current_session, swept_session) {
            return Ok(false); // a fresher claim already owns it
        }
        let mut record = record;
        record.insert("status".into(), Value::String("open".into()));
        // `{ ...(cellRecord.trace || {}), worker: null, claimed_at: null,
        //    claim_session: null, swept_at, swept_from_session }`
        let mut trace = Map::new();
        if let Some(Value::Object(old)) = record.get("trace") {
            spread_into(&mut trace, old);
        }
        trace.insert("worker".into(), Value::Null);
        trace.insert("claimed_at".into(), Value::Null);
        trace.insert("claim_session".into(), Value::Null);
        trace.insert(
            "swept_at".into(),
            Value::String(rsv::iso_from_ms(now).map_err(|_| Fail::Delegate)?),
        );
        trace.insert("swept_from_session".into(), swept_session.clone());
        record.insert("trace".into(), Value::Object(trace));
        let value = Value::Object(record);
        transient_fs_retry(|| crate::fsutil::write_json_atomic(&file, &value))
            .map_err(|e| Fail::Thrown(format!("{e}")))?;
        Ok(true)
    })();
    guard.release();
    outcome
}

/// lib/state.mjs resolvePipeline's answer, reduced to the two fields
/// claimNextCell consumes: `resolved.record.feature || null` and
/// `gateApproved(resolved.record, 'execution')`.
enum Pipeline {
    Ok { feature: Option<String>, execution_approved: bool },
    Refused { code: &'static str, reason: String },
}

/// lib/state.mjs resolvePipeline — session record → bound lane → default
/// state.json, with the four typed refusals. Sessions and lanes are
/// control-plane (msn-18a); the default record stays on the caller's own root.
fn resolve_pipeline(root: &Path, control: &Path, session_id: &str) -> MR<Pipeline> {
    let defaults = || -> MR<Pipeline> {
        let state = bstate::read_state_brief(root).map_err(|_| Fail::Delegate)?;
        Ok(Pipeline::Ok {
            feature: match &state.feature {
                v if js_truthy(v) => Some(jsjson::js_to_string(v)),
                _ => None,
            },
            execution_approved: matches!(state.gates.get("execution"), Some(Value::Bool(true))),
        })
    };
    if js_trim(session_id).is_empty() {
        return defaults();
    }
    let Some(session) = read_session(control, session_id)? else { return defaults() };
    let bound = match session.get("lane") {
        Some(Value::String(l)) => js_trim(l).to_string(),
        _ => String::new(),
    };
    if bound.is_empty() {
        return defaults();
    }
    let session_disp = js_string_or_undefined(session.get("id"));
    // lanePath's requireLaneFeature throw → LANE_INVALID, message embedded.
    let Some(lane_id) = lane_feature_ok(&bound) else {
        let detail = if js_trim(&bound).is_empty() {
            "lane feature is required."
        } else {
            "lane feature must be a plain id (no path separators)."
        };
        return Ok(Pipeline::Refused {
            code: "LANE_INVALID",
            reason: format!(
                "session \"{session_disp}\" is bound to lane \"{bound}\", which is not a valid lane name ({detail}) \u{2014} never guessed back to the default pipeline. FIX: rebind or unbind the session (claims bindSessionLane/unbindSessionLane)."
            ),
        });
    };
    let file = lanes_dir(control).join(format!("{lane_id}.json"));
    let rel = lane_rel_path(&lane_id);
    if !file.exists() {
        return Ok(Pipeline::Refused {
            code: "LANE_MISSING",
            reason: format!(
                "session \"{session_disp}\" is bound to lane \"{bound}\" but {rel} does not exist \u{2014} resolution never guesses back to the default pipeline. FIX: start the lane (startFeature with lane mode) or unbind the session."
            ),
        });
    }
    let record = crate::verbs::workflow_store::read_lane_display(control, &bound)?;
    let Some(record) = record else {
        return Ok(Pipeline::Refused {
            code: "LANE_CORRUPT",
            reason: format!(
                "session \"{session_disp}\" is bound to lane \"{bound}\" but its record is corrupt \u{2014} display never guesses and mutations must refuse. FIX: inspect/restore {rel}, then retry."
            ),
        });
    };
    let approved = record
        .get("approved_gates")
        .and_then(|g| g.get("execution"))
        .map(|v| matches!(v, Value::Bool(true)))
        .unwrap_or(false);
    Ok(Pipeline::Ok {
        feature: match record.get("feature") {
            Some(v) if js_truthy(v) => Some(jsjson::js_to_string(v)),
            _ => None,
        },
        execution_approved: approved,
    })
}

/// lib/reservations.mjs findSessionConflicts — active path leases owned by a
/// DIFFERENT session overlapping any requested path. `true` = at least one.
fn has_session_conflict(root: &Path, acting: &str, requested: &[String], now: f64) -> MR<bool> {
    if requested.is_empty() {
        return Ok(false);
    }
    let acting = js_trim(acting);
    for rec in list_path_lease_records(root)? {
        if lease_record_expired(&rec, now)? {
            continue;
        }
        let resv = lease_to_resv_lite(&rec)?;
        let owner = match &resv.session {
            Some(Value::String(s)) if !js_trim(s).is_empty() => s.clone(),
            _ => continue, // a legacy/sessionless row never conflicts here
        };
        if owner == acting {
            continue;
        }
        if requested.iter().any(|p| rsv::paths_overlap(&resv.path, p)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// lib/worktree-holds.mjs findForeignHolds over resolveHoldTopology's
/// ORDINARY arm (`{mainRoot: root, holder: 'main'}` — see the section header).
fn has_foreign_hold(root: &Path, holder: &str, requested: &[String], now: f64) -> MR<bool> {
    if requested.is_empty() {
        return Ok(false);
    }
    let acting = js_trim(holder);
    let store = read_holds_store(root)?;
    let Some(Value::Array(holds)) = store.get("holds") else { return Ok(false) };
    for hold in holds {
        // isActive: released_at == null && !isExpired
        if !matches!(hold.get("released_at"), None | Some(Value::Null)) {
            continue;
        }
        let expired = match hold.get("ttl_seconds") {
            Some(Value::Number(n)) => {
                let ttl = n.as_f64().unwrap_or(f64::NAN);
                if !ttl.is_finite() || ttl <= 0.0 {
                    false
                } else {
                    match rsv::date_parse_val(hold.get("mirrored_at")).map_err(|_| Fail::Delegate)? {
                        None => false,
                        Some(m) => m + ttl * 1000.0 <= now,
                    }
                }
            }
            _ => false,
        };
        if expired {
            continue;
        }
        if matches!(hold.get("holder"), Some(Value::String(s)) if s == acting) {
            continue;
        }
        let hold_path = match hold.get("path") {
            Some(v) if js_truthy(v) => jsjson::js_to_string(v),
            _ => String::new(),
        };
        if requested.iter().any(|p| rsv::paths_overlap(&hold_path, p)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `Array.isArray(cell.files) ? cell.files : []`, then `.filter(Boolean)` —
/// the request list both hold checks take.
fn declared_files(cell: &Value) -> (usize, Vec<String>) {
    let Some(Value::Array(files)) = cell.get("files") else { return (0, Vec::new()) };
    let requested = files
        .iter()
        .filter(|f| js_truthy(f))
        .map(jsjson::js_to_string)
        .collect();
    (files.len(), requested)
}

/// claimNextCell's `candidateOk` = holdFree && checkCellBudgets().ok.
fn candidate_ok(root: &Path, control: &Path, session: &str, cell: &Value, now: f64) -> MR<bool> {
    let (raw_len, requested) = declared_files(cell);
    if raw_len > 0 {
        if has_session_conflict(control, session, &requested, now)? {
            return Ok(false);
        }
        // resolveHoldTopology(root) is the ordinary constant here.
        if has_foreign_hold(root, "main", &requested, now)? {
            return Ok(false);
        }
    }
    let Value::Object(map) = cell else { return Err(Fail::Delegate) };
    Ok(matches!(check_cell_budgets(map)?, BudgetCheck::Ok))
}

/// readyCells(root, feature) — listCells({feature, status:'open'}) filtered to
/// cells whose depsAllCapped list is empty (lib/cells.mjs).
fn ready_cells(root: &Path, feature: Option<&str>) -> MR<Vec<Value>> {
    let mut out = Vec::new();
    for cell in list_cells(root, feature, Some("open"))? {
        if deps_all_capped_is_empty(root, &cell)? {
            out.push(cell);
        }
    }
    Ok(out)
}

/// Everything claim-next reads that could route the command back to Node,
/// probed BEFORE the sweep's first write. See the section header for why a
/// residual post-sweep delegate is still byte-safe.
fn prescan_claim_next(root: &Path, control: &Path) -> MR<()> {
    delegate_only(bstate::read_state_brief(root).map_err(|_| Fail::Delegate))?;
    delegate_only(list_session_records(control))?;
    delegate_only(read_holds_store(root))?;
    // CUTOVER: the lane-record walk that used to live here probed for exactly
    // two things — corrupt JSON and |n| >= 1e21 numbers. Both are native now,
    // so the walk has nothing left to decide, and keeping it would warn about
    // lane files this command never reads. Deleted; resolvePipeline warns at
    // its own read, once, like Node did.
    if crate::verbs::backlog::feature_backlog_rank(root).is_none() {
        return Err(Fail::Delegate);
    }
    for rec in list_path_lease_records(root)? {
        delegate_only(lease_to_resv_lite(&rec))?;
    }
    if let Ok(entries) = std::fs::read_dir(claims_dir(control)) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(cell) = name.strip_suffix(".json") else { continue };
            let claim = match read_claim(control, cell) {
                Err(Fail::Delegate) => return Err(Fail::Delegate),
                Err(_) => continue, // requireId throws are Node's own, per-cell
                Ok(c) => c,
            };
            if let Some(claim) = claim {
                delegate_only(read_session_of_claim(control, &claim))?;
            }
        }
    }
    // The sweep's reset spreads `...(cellRecord.trace || {})`; a truthy
    // NON-object trace would spread JS-exotic index keys. Delegate up front
    // rather than guess (no bee-written cell has that shape). Read RAW here,
    // not through read_store_json: this walks every cell file, while the
    // sweep only reads the ones it resets, so a corrupt file must not warn
    // from the probe — its own read will warn if the sweep gets there.
    if let Ok(entries) = std::fs::read_dir(cells_dir(control)) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            if let ReadJson::Parsed(Value::Object(m)) = read_json(&entry.path()) {
                match m.get("trace") {
                    None | Some(Value::Null) | Some(Value::Object(_)) => {}
                    Some(t) if !js_truthy(t) => {}
                    Some(_) => return Err(Fail::Delegate),
                }
            }
        }
    }
    Ok(())
}

/// bee.mjs handleCellsClaimNext + lib/cells.mjs claimNextCell.
fn run_claim_next(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["worker", "session-id", "ttl", "isolate"]) {
        return None;
    }
    let worker = flags.req_str("worker")?.to_string();
    let session_flag = opt_string_flag(&flags, "session-id")?;
    let _isolate = bool_flag(&flags, "isolate")?;
    let ttl: Option<f64> = match flags.get("ttl") {
        None => None,
        Some(FlagV::Present) => return None,
        Some(FlagV::S(s)) => match rsv::js_number_flag(s) {
            Err(_) => return None, // validate() refuses the shape — Node's message
            Ok(parsed) => Some(parsed.unwrap_or(f64::NAN)),
        },
    };
    dispatch("cells claim-next", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let control = control_root(&root)?;
        // resolveSessionId({flag, root: controlRootFor(root)}) — flag ->
        // BEE_SESSION_ID -> CLAUDE_CODE_SESSION_ID -> the durable
        // single-live-session adoption (hardening-1-7-10 D5/1710-10).
        let session_id = match resolve_session_flag_env(session_flag.as_deref()) {
            Some(s) => Some(s),
            None => resolve_session_adopt(&control)?,
        };
        let Some(session_id) = session_id else {
            return Err(Fail::Thrown(
                "claim-next: --session-id or CLAUDE_CODE_SESSION_ID env is required.".into(),
            ));
        };
        if let Some(t) = ttl {
            if !t.is_finite() || t <= 0.0 {
                return Err(Fail::Thrown("--ttl must be a positive integer (seconds).".into()));
            }
        }
        // applyWritePolicy — a no-op for this verb (see the section header);
        // only readConfig's own corrupt-file delegation survives.
        delegate_only(bstate::read_config_raw(&root).map_err(|_| Fail::Delegate))?;

        prescan_claim_next(&root, &control)?;

        // ── claimNextCell ──────────────────────────────────────────────────
        let session = js_trim(&session_id).to_string();
        // Unconditional, first thing — the production sweep trigger (C10).
        sweep_expired_claims(&control, rsv::now_ms())?;

        let (own_feature, own_approved) = match resolve_pipeline(&root, &control, &session)? {
            Pipeline::Refused { code, reason } => {
                return Err(Fail::Thrown(format!("claim-next: {code} — {reason}")));
            }
            Pipeline::Ok { feature, execution_approved } => (feature, execution_approved),
        };

        let now = rsv::now_ms();
        let mut candidate: Option<Value> = None;
        if let Some(feature) = &own_feature {
            if own_approved {
                for cell in ready_cells(&root, Some(feature))? {
                    if candidate_ok(&root, &control, &session, &cell, now)? {
                        candidate = Some(cell);
                        break;
                    }
                }
            }
        }

        if candidate.is_none() {
            let state = bstate::read_state_brief(&root).map_err(|_| Fail::Delegate)?;
            // feature -> (approved, created_at); insertion-ordered like the Map.
            let mut pipelines: Vec<(String, bool, Value)> = Vec::new();
            let state_feature = match &state.feature {
                v if js_truthy(v) => Some(jsjson::js_to_string(v)),
                _ => None,
            };
            if let Some(f) = &state_feature {
                if own_feature.as_deref() != Some(f.as_str()) {
                    pipelines.push((
                        f.clone(),
                        matches!(state.gates.get("execution"), Some(Value::Bool(true))),
                        Value::Null,
                    ));
                }
            }
            // GH#20: lanes actively owned by ANOTHER live session are never pooled.
            let mut live_owned: Vec<String> = Vec::new();
            for record in list_session_records(&control)? {
                if matches!(record.get("id"), Some(Value::String(s)) if *s == session) {
                    continue;
                }
                let bound = match record.get("lane") {
                    Some(Value::String(l)) => js_trim(l).to_string(),
                    _ => String::new(),
                };
                if bound.is_empty() || heartbeat_stale(Some(&record), now)? {
                    continue;
                }
                live_owned.push(bound);
            }
            for lane in crate::verbs::workflow_store::list_lanes(&root)? {
                let feature = match lane.get("feature") {
                    Some(v) if js_truthy(v) => jsjson::js_to_string(v),
                    _ => continue,
                };
                if own_feature.as_deref() == Some(feature.as_str())
                    || pipelines.iter().any(|(f, _, _)| *f == feature)
                {
                    continue;
                }
                if live_owned.iter().any(|l| *l == feature) {
                    continue;
                }
                let approved = lane
                    .get("approved_gates")
                    .and_then(|g| g.get("execution"))
                    .map(|v| matches!(v, Value::Bool(true)))
                    .unwrap_or(false);
                let created_at = match lane.get("created_at") {
                    Some(v) if js_truthy(v) => v.clone(),
                    _ => Value::Null, // `lane.created_at || null`
                };
                pipelines.push((feature, approved, created_at));
            }

            let rank = crate::verbs::backlog::feature_backlog_rank(&root)
                .ok_or(Fail::Delegate)?;
            // (cell, rank, created_at_ms) — the sort keys, built in pool order.
            let mut pool: Vec<(Value, f64, Option<f64>)> = Vec::new();
            for (feature, approved, created_at) in &pipelines {
                if !approved {
                    continue; // D2: an unapproved lane is never touched
                }
                let rank_of = rank.get(feature).map(|r| *r as f64).unwrap_or(f64::INFINITY);
                let created = match created_at {
                    v if js_truthy(v) => rsv::date_parse_val(Some(v))
                        .map_err(|_| Fail::Delegate)?
                        .filter(|ms| ms.is_finite()),
                    _ => None, // `a.meta.created_at ? Date.parse(...) : NaN`
                };
                for cell in ready_cells(&root, Some(feature))? {
                    if candidate_ok(&root, &control, &session, &cell, now)? {
                        pool.push((cell, rank_of, created));
                    }
                }
            }
            // rank asc, then a KNOWN created_at asc, then a known one before an
            // unknown one; V8's sort is stable and so is Rust's.
            pool.sort_by(|a, b| {
                if a.1 != b.1 {
                    return a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal);
                }
                match (a.2, b.2) {
                    (Some(x), Some(y)) if x != y => {
                        x.partial_cmp(&y).unwrap_or(Ordering::Equal)
                    }
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            });
            candidate = pool.into_iter().next().map(|(cell, _, _)| cell);
        }

        let Some(candidate) = candidate else {
            return Err(Fail::Thrown(
                "claim-next: NO_APPROVED_WORK \u{2014} no claimable cell: the acting session's own pipeline has none ready, and no other execution-approved pipeline has a ready cell free of another session's hold.".into(),
            ));
        };
        let cell_id = js_string_or_undefined(candidate.get("id"));
        let (cell, claim) = match claim_cell_cross_session(
            &root,
            &control,
            Some(session.as_str()),
            &worker,
            &cell_id,
            ttl,
            Some(&candidate),
        )? {
            CrossClaim::Ok { cell, claim } => (cell, claim),
            CrossClaim::Refused { code, reason } => {
                return Err(Fail::Thrown(format!("claim-next: {code} — {reason}")));
            }
        };
        let text = format!(
            "Claimed {} for {worker} (session {session}).",
            js_string_or_undefined(cell.get("id"))
        );
        let mut result = Map::new();
        result.insert("ok".into(), Value::Bool(true));
        result.insert("cell".into(), cell);
        result.insert("claim".into(), claim);
        Ok(Out::Emit(Value::Object(result), text, 0))
    })
}

// ── cells cap / cells finish ───────────────────────────────────────────────

const CAP_FLAGS: [&str; 8] = [
    "id",
    "outcome",
    "files",
    "deviations-file",
    "friction",
    "override-judge",
    "session-id",
    "force-ownership",
];

/// resolveDeclaredBehaviorChange (E6).
fn resolve_declared_behavior_change(cell: &Map<String, Value>) -> bool {
    match cell.get("behavior_change") {
        Some(Value::Bool(true)) => true,
        Some(Value::Bool(false)) => false,
        _ => {
            let trace = cell.get("trace");
            matches!(trace, Some(t) if js_truthy(t))
                && matches!(
                    trace.and_then(|t| t.get("behavior_change")),
                    Some(Value::Bool(true))
                )
        }
    }
}

/// parseDeviationsFile.
fn parse_deviations_file(file: &str) -> MR<Vec<Value>> {
    let raw = read_file_text(file, "deviations")?;
    match parse_json_js(&raw, false) {
        JsParse::Value(Value::Array(a)) => Ok(a),
        JsParse::Value(other) => Ok(vec![Value::String(jsjson::js_to_string(&other))]),
        // Node's `catch`: not JSON -> one deviation per non-blank line. A
        // lone-surrogate escape now takes that same branch.
        JsParse::NotJson => Ok(raw
            .split('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .filter(|l| !js_trim(l).is_empty())
            .map(|l| Value::String(l.to_string()))
            .collect()),
    }
}

struct CapFlags {
    id: String,
    outcome: Option<String>,       // flags.outcome ? String : undefined
    friction: Option<String>,      // flags.friction ? String : null
    files_changed: Vec<Value>,
    deviations: Vec<Value>,
    override_reason: String,       // trimmed; '' = none
    session_flag: Option<String>,
    force_ownership: bool,
}

/// capCellFromFlags — the ONE cap door cap and finish share.
fn cap_cell_from_flags(root: &Path, f: &CapFlags, finish: bool) -> MR<Value> {
    let id = &f.id;
    // Pre-scan (see the pre-scan section header).
    prescan_claim(root, id)?;
    let commands = read_commands_slice(root)?;
    if !f.override_reason.is_empty() {
        delegate_only(load_taxonomy(root))?;
    }
    if finish {
        delegate_only(list_path_lease_records(root))?;
        delegate_only(read_holds_store(root))?;
    }
    // Cheap pre-checks BEFORE the (possibly long) test run.
    let existing = read_cell_norm(root, id)?;
    let Some(existing) = existing else {
        return Err(Fail::Thrown(format!("capCell: cell \"{id}\" not found.")));
    };
    let Value::Object(existing_map) = &existing else { return Err(Fail::Delegate) };
    match existing_map.get("status") {
        Some(Value::String(s)) if s == "capped" => {
            return Err(Fail::Thrown(format!("capCell: cell \"{id}\" is already capped.")))
        }
        Some(Value::String(s)) if s == "dropped" => {
            return Err(Fail::Thrown(format!("capCell: cell \"{id}\" was dropped.")))
        }
        _ => {}
    }
    delegate_only(merge_trace(existing_map.get("trace")))?;

    // The one test door (test-simple, decision 412e9b3a).
    let declared = commands
        .test
        .as_ref()
        .map(|list| list.iter().filter(|c| *c != NO_TEST_SENTINEL).cloned().collect::<Vec<_>>())
        .filter(|l| !l.is_empty());
    let tests_run: Option<TestsRun> = match &declared {
        Some(list) => Some(run_declared_tests(root, list)?),
        None => None,
    };
    if let Some(run) = &tests_run {
        if !run.green {
            let excerpt_line = first_failure_line(run);
            // Record the tests-red attempt BEFORE refusing.
            let mut guard = acquire_named_lock(root, &format!("cells:{id}"))?;
            let recorded = (|| -> MR<()> {
                assert_not_archived(root, "recordTestsRedAttempt", id)?;
                let cell = read_cell_norm(root, id)?;
                let Some(cell) = cell else {
                    return Err(Fail::Thrown(format!(
                        "recordTestsRedAttempt: cell \"{id}\" not found."
                    )));
                };
                let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
                let trace = merge_trace(cell_map.get("trace"))?;
                let trace = append_attempt(
                    root,
                    id,
                    trace,
                    "tests-red",
                    excerpt_line.as_deref().map(normalize_failure_signature),
                    excerpt_line.as_deref(),
                )?;
                cell_map.insert("trace".into(), Value::Object(trace));
                write_cell(root, &Value::Object(cell_map))
            })();
            guard.release();
            recorded?;
            let failing: Vec<&CmdRun> = run
                .commands
                .iter()
                .filter(|c| c.failure_excerpt.as_deref().map(|e| !e.is_empty()).unwrap_or(false))
                .collect();
            let mut lines = vec![format!(
                "refusing to cap \"{id}\" — the declared test run is RED ({} of {} command(s) failed; record: {TEST_RESULTS_RELATIVE}).",
                failing.len(),
                run.commands.len()
            )];
            for c in &failing {
                lines.push(format!(
                    "--- {} (exit {}) ---\n{}",
                    c.command,
                    c.exit.map(jsjson::js_f64_to_string).unwrap_or_else(|| "spawn-failed".to_string()),
                    c.failure_excerpt.as_deref().unwrap_or("")
                ));
            }
            lines.push(format!(
                "The red is the work: fix what the failing output names, then re-run bee cells finish --id {id}. Never build on a red base."
            ));
            return Err(Fail::Thrown(lines.join("\n")));
        }
    }

    // capCell (lib/cells.mjs) under the per-cell lock.
    let mut guard = acquire_named_lock(root, &format!("cells:{id}"))?;
    let saved = (|| -> MR<Value> {
        assert_not_archived(root, "capCell", id)?;
        let cell = read_cell_norm(root, id)?;
        let Some(cell) = cell else {
            return Err(Fail::Thrown(format!("capCell: cell \"{id}\" not found.")));
        };
        let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
        let bc = resolve_declared_behavior_change(&cell_map);
        match cell_map.get("status") {
            Some(Value::String(s)) if s == "capped" => {
                return Err(Fail::Thrown(format!("capCell: cell \"{id}\" is already capped.")))
            }
            Some(Value::String(s)) if s == "dropped" => {
                return Err(Fail::Thrown(format!("capCell: cell \"{id}\" was dropped.")))
            }
            _ => {}
        }
        let mut trace = merge_trace(cell_map.get("trace"))?;
        trace = guard_claim_ownership(
            root,
            id,
            trace,
            "capCell",
            f.session_flag.as_deref(),
            f.force_ownership,
        )?;
        let judge_entries: Vec<Value> = match trace.get("semantic_judge") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        let latest_judge = judge_entries.last().cloned();
        let latest_needs_revision = matches!(
            latest_judge.as_ref().filter(|l| js_truthy(l)).and_then(|l| l.get("verdict")),
            Some(Value::String(s)) if s == "NEEDS_REVISION"
        );
        if latest_needs_revision && f.override_reason.is_empty() {
            return Err(Fail::Thrown(format!(
                "capCell: cell \"{id}\" has a NEEDS_REVISION semantic-judge verdict — rework the cell and record a PASS verdict (bee cells judge-record), or cap with an audited override (bee cells cap --id {id} --override-judge \"<reason>\")."
            )));
        }
        if !f.override_reason.is_empty() {
            let overrides: Vec<Value> = match trace.get("judge_overrides") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let mut entry = Map::new();
            entry.insert("overridden_at".into(), Value::String(utc_now()));
            entry.insert("reason".into(), Value::String(f.override_reason.clone()));
            match &latest_judge {
                None => {
                    entry.insert("last_verdict".into(), Value::Null);
                }
                Some(l) => match l.get("verdict") {
                    Some(v) => {
                        entry.insert("last_verdict".into(), v.clone());
                    }
                    None => {} // {last_verdict: undefined} — JSON.stringify drops the key
                },
            }
            let worker_disp = match trace.get("worker") {
                Some(w) if js_truthy(w) => jsjson::js_to_string(w),
                _ => "unknown".to_string(),
            };
            log_decision(
                root,
                &format!(
                    "«cells cap: cell \"{id}\" judge override by {worker_disp} — {}»",
                    f.override_reason
                ),
                "Audited cap over a NEEDS_REVISION (or absent) semantic-judge verdict (D-GHF-C, GH #27.5) — the verdict itself is never rewritten, only a judge_overrides marker appended.",
                &["cells", "judge"],
            )?;
            let mut next = overrides;
            next.push(Value::Object(entry));
            trace.insert("judge_overrides".into(), Value::Array(next));
        }
        let lane = match cell_map.get("lane") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        if lane == "small" || lane == "standard" || lane == "high-risk" {
            if f.files_changed.is_empty() {
                return Err(Fail::Thrown(format!(
                    "capCell: lane \"{lane}\" cell \"{id}\" requires non-empty files_changed (--files a.js,b.js) — record what the worker actually touched. A cell that changed nothing is a drop or a NOOP, not a cap."
                )));
            }
        }
        if lane == "high-risk" && f.outcome.as_deref().map(|o| js_trim(o).is_empty()).unwrap_or(true) {
            return Err(Fail::Thrown(format!(
                "capCell: high-risk cell \"{id}\" requires an outcome summary."
            )));
        }
        cell_map.insert("status".into(), Value::String("capped".into()));
        trace.insert("files_changed".into(), Value::Array(f.files_changed.clone()));
        trace.insert("deviations".into(), Value::Array(f.deviations.clone()));
        trace.insert(
            "friction".into(),
            f.friction.clone().map(Value::String).unwrap_or(Value::Null),
        );
        trace.insert("behavior_change".into(), Value::Bool(bc));
        let outcome_value = match &f.outcome {
            Some(o) if !js_trim(o).is_empty() => Value::String(o.clone()),
            _ => trace.get("outcome").cloned().unwrap_or(Value::Null),
        };
        trace.insert("outcome".into(), outcome_value);
        trace.insert("capped_at".into(), Value::String(utc_now()));
        // No producer since the E1 impact-registry check retired; the key
        // stays so a capped cell's trace shape is unchanged.
        trace.insert("warnings".into(), Value::Array(Vec::new()));
        match &tests_run {
            None => {
                trace.insert("tests".into(), Value::String("undeclared".into()));
            }
            Some(run) => {
                trace.insert("tests".into(), Value::String("green".into()));
                trace.insert("results".into(), Value::String(TEST_RESULTS_RELATIVE.into()));
                trace.insert("ran_at".into(), Value::String(run.ran_at.clone()));
            }
        }
        cell_map.insert("trace".into(), Value::Object(trace));
        let cell_value = Value::Object(cell_map);
        write_cell(root, &cell_value)?;
        Ok(cell_value)
    })();
    guard.release();
    let saved = saved?;
    release_claim_file_best_effort(root, id); // D1 Δ2: cap clears the claim
    Ok(saved)
}

fn cap_flags_from(flags: &rsv::Flags) -> Option<CapFlags> {
    let id = flags.req_str("id")?.to_string();
    let outcome = flags.truthy_str("outcome").map(str::to_string);
    let friction = flags.truthy_str("friction").map(str::to_string);
    let files_changed: Vec<Value> = flags
        .truthy_str("files")
        .map(|s| {
            s.split(',')
                .map(js_trim)
                .filter(|p| !p.is_empty())
                .map(|p| Value::String(p.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let override_reason = match opt_string_flag(flags, "override-judge")? {
        Some(s) => js_trim(&s).to_string(),
        None => String::new(),
    };
    let session_flag = opt_string_flag(flags, "session-id")?;
    let force_ownership = bool_flag(flags, "force-ownership")?;
    Some(CapFlags {
        id,
        outcome,
        friction,
        files_changed,
        deviations: Vec::new(), // filled inside dispatch (file read may throw)
        override_reason,
        session_flag,
        force_ownership,
    })
}

fn cap_text(cell: &Value) -> String {
    let trace = cell.get("trace");
    let capped_at = trace.and_then(|t| t.get("capped_at"));
    let tests = trace.and_then(|t| t.get("tests"));
    format!(
        "Capped {} at {} (tests: {}).",
        js_string_or_undefined(cell.get("id")),
        js_string_or_undefined(capped_at),
        match tests {
            None | Some(Value::Null) => "not run".to_string(), // ?? 'not run'
            Some(v) => jsjson::js_to_string(v),
        }
    )
}

fn run_cap(finish: bool, flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &CAP_FLAGS) {
        return None;
    }
    let cap_flags = cap_flags_from(&flags)?;
    let deviations_file = opt_string_flag(&flags, "deviations-file")?;
    let cmd: &'static str = if finish { "cells finish" } else { "cells cap" };
    let cap_flags_owned = cap_flags;
    dispatch(cmd, use_json, t0, move |ctx| {
        let mut cap_flags = cap_flags_owned;
        let root = ctx.root.clone();
        if let Some(file) = &deviations_file {
            if !file.is_empty() {
                // `flags['deviations-file'] ? parse : []` — truthy only.
                cap_flags.deviations = parse_deviations_file(file)?;
            }
        }
        let cell = cap_cell_from_flags(&root, &cap_flags, finish)?;
        if !finish {
            let text = cap_text(&cell);
            return Ok(Out::Emit(cell, text, 0));
        }
        // cells.finish: release every reservation the claiming agent holds.
        let agent = match cell.get("trace").and_then(|t| t.get("worker")) {
            Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        };
        let cell_id = js_string_or_undefined(cell.get("id"));
        let mut released: Vec<String> = Vec::new();
        let mut release_failure: Option<(String, String)> = None;
        if let Some(agent) = &agent {
            match release_reservations_for_agent(&root, agent, &cell_id) {
                Ok(outcome) => released = outcome.paths,
                Err(Fail::Thrown(message)) => {
                    release_failure = Some((
                        message,
                        format!("bee reservations release --agent {agent} --cell {cell_id} --json"),
                    ));
                }
                Err(Fail::Delegate) => {
                    // Pre-scanned; a mid-command race lands here (header
                    // residual): report it as a release failure, never a
                    // rollback of the already-committed cap.
                    release_failure = Some((
                        "reservation store shape changed mid-command (unrepresentable natively)".to_string(),
                        format!("bee reservations release --agent {agent} --cell {cell_id} --json"),
                    ));
                }
            }
        }
        let Value::Object(cell_map) = &cell else { return Err(Fail::Delegate) };
        let mut result = cell_map.clone();
        result.insert(
            "released".into(),
            Value::Array(released.iter().map(|p| Value::String(p.clone())).collect()),
        );
        if let Some((error, fix)) = &release_failure {
            let mut rf = Map::new();
            rf.insert("error".into(), Value::String(error.clone()));
            rf.insert("fix".into(), Value::String(fix.clone()));
            result.insert("release_failed".into(), Value::Object(rf));
        }
        let mut lines = vec![cap_text(&cell)];
        lines.push(match (&release_failure, released.len()) {
            (Some((error, fix)), _) => {
                format!("Cap stands, but releasing reservations FAILED ({error}) — run: {fix}")
            }
            (None, 0) => "No active reservations to release.".to_string(),
            (None, n) => format!("Released {n} reservation(s): {}.", released.join(", ")),
        });
        lines.push("next: reply [DONE] with the one-line outcome, files touched, and the commit hash.".to_string());
        Ok(Out::Emit(Value::Object(result), lines.join("\n"), 0))
    })
}

// ── block / drop / unclaim / reopen / tier ─────────────────────────────────

/// Shared frame: pre-scan, per-cell lock, read (Delegate on non-object),
/// mutate, write, optional claim clear.
fn mutate_cell(
    root: &Path,
    id: &str,
    verb_not_found: &str,
    archived_verb: Option<&str>,
    clear_claim_after: bool,
    mutate: impl FnOnce(&mut Map<String, Value>) -> MR<()>,
) -> MR<Value> {
    prescan_claim(root, id)?;
    let mut guard = acquire_named_lock(root, &format!("cells:{id}"))?;
    let saved = (|| -> MR<Value> {
        if let Some(verb) = archived_verb {
            assert_not_archived(root, verb, id)?;
        }
        let cell = read_cell_norm(root, id)?;
        let Some(cell) = cell else {
            return Err(Fail::Thrown(format!("{verb_not_found}: cell \"{id}\" not found.")));
        };
        let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
        mutate(&mut cell_map)?;
        let value = Value::Object(cell_map);
        write_cell(root, &value)?;
        Ok(value)
    })();
    guard.release();
    let saved = saved?;
    if clear_claim_after {
        release_claim_file_best_effort(root, id);
    }
    Ok(saved)
}

fn ownership_args(flags: &rsv::Flags) -> Option<(Option<String>, bool)> {
    Some((opt_string_flag(flags, "session-id")?, bool_flag(flags, "force-ownership")?))
}

fn run_block(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells block", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("blockCell: a reason is required.".into()));
        }
        let root2 = root.clone();
        let id2 = id.clone();
        let reason2 = reason.clone();
        let cell = mutate_cell(&root, &id, "blockCell", Some("blockCell"), true, move |cell_map| {
            let trace = merge_trace(cell_map.get("trace"))?;
            let trace = guard_claim_ownership(
                &root2,
                &id2,
                trace,
                "blockCell",
                session_flag.as_deref(),
                force,
            )?;
            let mut trace = append_attempt(
                &root2,
                &id2,
                trace,
                "blocked",
                Some(normalize_failure_signature(&reason2)),
                Some(&reason2),
            )?;
            cell_map.insert("status".into(), Value::String("blocked".into()));
            trace.insert("blocked_reason".into(), Value::String(reason2.clone()));
            cell_map.insert("trace".into(), Value::Object(trace));
            Ok(())
        })?;
        let text = format!("Blocked {}.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

fn run_drop(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    dispatch("cells drop", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("dropCell: a reason is required.".into()));
        }
        let reason2 = reason.clone();
        let cell = mutate_cell(&root, &id, "dropCell", Some("dropCell"), true, move |cell_map| {
            let mut trace = merge_trace(cell_map.get("trace"))?;
            cell_map.insert("status".into(), Value::String("dropped".into()));
            trace.insert("dropped_reason".into(), Value::String(reason2.clone()));
            cell_map.insert("trace".into(), Value::Object(trace));
            Ok(())
        })?;
        let text = format!("Dropped {}.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

fn run_unclaim(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells unclaim", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let cell = unclaim_cell(&root, &id, session_flag.as_deref(), force)?;
        let text = format!("Unclaimed {} — back to open.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

/// cells.mjs `unclaimCell(root, id, {sessionId, forceOwnership})`.
///
/// pub(crate) since the `dispatch prepare --claim` port: bee.mjs's
/// claimAndReserveForDispatch calls this as the SECOND rung of its unwind
/// ladder when a reservation conflicts, so the conflict refusal can promise
/// "the claim was unwound and state restored as found".
pub(crate) fn unclaim_cell(
    root: &Path,
    id: &str,
    session_flag: Option<&str>,
    force: bool,
) -> MR<Value> {
    let root = root.to_path_buf();
    let id = id.to_string();
    {
        let root2 = root.clone();
        let id2 = id.clone();
        let session_flag = session_flag.map(str::to_string);
        // unclaimCell has NO assertNotArchived (an archived cell reads as
        // capped/dropped and takes the not-claimed refusal instead).
        let cell = mutate_cell(&root, &id, "unclaimCell", None, true, move |cell_map| {
            let claimed = matches!(cell_map.get("status"), Some(Value::String(s)) if s == "claimed");
            if !claimed {
                return Err(Fail::Thrown(format!(
                    "unclaimCell: cell \"{id2}\" is \"{}\", not \"claimed\" — only a claimed cell can be unclaimed (returned to open). For a capped/blocked/dropped cell use bee cells reopen.",
                    js_string_or_undefined(cell_map.get("status"))
                )));
            }
            let trace = merge_trace(cell_map.get("trace"))?;
            let trace = guard_claim_ownership(
                &root2,
                &id2,
                trace,
                "unclaimCell",
                session_flag.as_deref(),
                force,
            )?;
            cell_map.insert("status".into(), Value::String("open".into()));
            cell_map.insert("trace".into(), Value::Object(release_trace(trace)));
            Ok(())
        })?;
        Ok(cell)
    }
}

fn run_reopen(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason", "session-id", "force-ownership"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells reopen", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("reopenCell: a reason is required.".into()));
        }
        let root2 = root.clone();
        let id2 = id.clone();
        let reason2 = reason.clone();
        let cell = mutate_cell(&root, &id, "reopenCell", Some("reopenCell"), true, move |cell_map| {
            match cell_map.get("status") {
                Some(Value::String(s)) if s == "open" => {
                    return Err(Fail::Thrown(format!(
                        "reopenCell: cell \"{id2}\" is already \"open\"."
                    )))
                }
                Some(Value::String(s)) if s == "claimed" => {
                    return Err(Fail::Thrown(format!(
                        "reopenCell: cell \"{id2}\" is \"claimed\" — use bee cells unclaim to release the claim back to open."
                    )))
                }
                _ => {}
            }
            let trace = merge_trace(cell_map.get("trace"))?;
            let trace = guard_claim_ownership(
                &root2,
                &id2,
                trace,
                "reopenCell",
                session_flag.as_deref(),
                force,
            )?;
            cell_map.insert("status".into(), Value::String("open".into()));
            let mut trace = release_trace(trace);
            trace.insert("blocked_reason".into(), Value::Null);
            trace.insert("dropped_reason".into(), Value::Null);
            trace.insert("reopened_at".into(), Value::String(utc_now()));
            trace.insert("reopened_reason".into(), Value::String(reason2.clone()));
            cell_map.insert("trace".into(), Value::Object(trace));
            Ok(())
        })?;
        let text = format!("Reopened {} — back to open.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
}

fn run_tier(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "tier"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let tier = flags.req_str("tier")?.to_string();
    if !MODEL_TIERS.contains(&tier.as_str()) {
        return None; // validate()'s required-field enum refusal — Node's bytes
    }
    dispatch("cells tier", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let tier2 = tier.clone();
        let cell = mutate_cell(&root, &id, "setTier", Some("setTier"), false, move |cell_map| {
            cell_map.insert("tier".into(), Value::String(tier2.clone()));
            Ok(())
        })?;
        let text = format!(
            "Cell {} tier set to {}.",
            js_string_or_undefined(cell.get("id")),
            js_string_or_undefined(cell.get("tier"))
        );
        Ok(Out::Emit(cell, text, 0))
    })
}

// ── cells judge (read-only frozen-judge check) ─────────────────────────────

fn run_judge(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    dispatch("cells judge", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let cell = read_cell_norm(&root, &id)?;
        let Some(cell) = cell else {
            return Err(Fail::Thrown(format!("judgeCell: cell \"{id}\" not found.")));
        };
        let Value::Object(cell_map) = &cell else { return Err(Fail::Delegate) };
        let changed = cell_map
            .get("trace")
            .filter(|t| js_truthy(t))
            .and_then(|t| t.get("files_changed"))
            .cloned()
            .unwrap_or(Value::Null);
        let declared = cell_map.get("files").cloned().unwrap_or(Value::Null);
        let hits = frozen_judge_hits(&changed, &declared);
        let id_disp = js_string_or_undefined(cell_map.get("id"));
        let text = if hits.is_empty() {
            format!("Judge intact for {id_disp}: no undeclared test/CI/lockfile changes.")
        } else {
            format!(
                "FROZEN-JUDGE HITS for {id_disp}: {} — do not count this cell toward a clean wave; flag it for review (decision 0018).",
                hits.iter()
                    .map(|(file, rule)| format!("{file} ({rule})"))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        };
        let mut result = Map::new();
        result.insert("id".into(), cell_map.get("id").cloned().unwrap_or(Value::Null));
        // NOTE: judgeCell returns {id: cell.id, hits}; an absent id would be
        // dropped by JSON.stringify — cells with a string id (the only shape
        // this native path serves) always carry one.
        result.insert(
            "hits".into(),
            Value::Array(
                hits.iter()
                    .map(|(file, rule)| {
                        let mut h = Map::new();
                        h.insert("file".into(), Value::String(file.clone()));
                        h.insert("rule".into(), Value::String(rule.to_string()));
                        Value::Object(h)
                    })
                    .collect(),
            ),
        );
        if !matches!(cell_map.get("id"), Some(Value::String(_))) {
            return Err(Fail::Delegate); // undefined-id JSON shape — Node's
        }
        Ok(Out::Emit(Value::Object(result), text, 0))
    })
}

// ── cells reset-budget ─────────────────────────────────────────────────────

fn run_reset_budget(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["id", "reason", "operator", "session-id"]) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let reason = flags.req_str("reason")?.to_string();
    let operator = opt_string_flag(&flags, "operator")?;
    let session_flag = opt_string_flag(&flags, "session-id")?;
    dispatch("cells reset-budget", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        if js_trim(&reason).is_empty() {
            return Err(Fail::Thrown("resetCellBudget: a reason is required.".into()));
        }
        let reason_text = js_trim(&reason).to_string();
        let by_session = resolve_session_flag_env(session_flag.as_deref());
        let actor = operator
            .as_deref()
            .filter(|o| !js_trim(o).is_empty())
            .map(|o| js_trim(o).to_string())
            .or_else(|| env_nonempty("BEE_AGENT_NAME"));
        delegate_only(load_taxonomy(&root))?;
        let mut guard = acquire_named_lock(&root, &format!("cells:{id}"))?;
        let saved = (|| -> MR<Value> {
            assert_not_archived(&root, "resetCellBudget", &id)?;
            let cell = read_cell_norm(&root, &id)?;
            let Some(cell) = cell else {
                return Err(Fail::Thrown(format!("resetCellBudget: cell \"{id}\" not found.")));
            };
            let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
            let Some(actor) = actor.clone() else {
                return Err(Fail::Thrown(format!(
                    "resetCellBudget: an actor is required — pass --operator \"<name>\" or set BEE_AGENT_NAME in the environment before resetting cell \"{id}\"'s budget."
                )));
            };
            match check_cell_budgets(&cell_map)? {
                BudgetCheck::Refused { .. } => {}
                BudgetCheck::Ok => {
                    return Err(Fail::Thrown(format!(
                        "resetCellBudget: cell \"{id}\" is not budget-blocked (checkCellBudgets reports ok) — a reset is only needed once the claim door is actually closed by CELL_BUDGET_EXHAUSTED or REPEATED_FAILURE."
                    )));
                }
            }
            let mut trace = merge_trace(cell_map.get("trace"))?;
            let resets: Vec<Value> = match trace.get("budget_resets") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let mut entry = Map::new();
            entry.insert("reset_at".into(), Value::String(utc_now()));
            entry.insert("reason".into(), Value::String(reason_text.clone()));
            entry.insert(
                "by_session".into(),
                by_session.clone().map(Value::String).unwrap_or(Value::Null),
            );
            entry.insert("by_actor".into(), Value::String(actor.clone()));
            // Audit BEFORE write (D-GHF-C).
            log_decision(
                &root,
                &format!(
                    "«cells reset-budget: cell \"{id}\" claim-lifetime budget reset by {actor} — {reason_text}»"
                ),
                "Audited reopening of a D2 loop-safety door (self-correcting-loop); the attempt ledger itself is never rewritten, only a budget_resets marker appended.",
                &["cells"],
            )?;
            let mut next = resets;
            next.push(Value::Object(entry));
            trace.insert("budget_resets".into(), Value::Array(next));
            cell_map.insert("trace".into(), Value::Object(trace));
            let value = Value::Object(cell_map);
            write_cell(&root, &value)?;
            Ok(value)
        })();
        guard.release();
        let cell = saved?;
        let text = format!(
            "Reset the claim-lifetime budget door for {}.",
            js_string_or_undefined(cell.get("id"))
        );
        Ok(Out::Emit(cell, text, 0))
    })
}

// ── cells judge-record ─────────────────────────────────────────────────────

fn run_judge_record(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(
        &flags,
        &["id", "file", "builder-model", "judge-model", "session-id", "force-ownership"],
    ) {
        return None;
    }
    let id = flags.req_str("id")?.to_string();
    let file = flags.req_str("file")?.to_string();
    let builder_model = opt_string_flag(&flags, "builder-model")?;
    let judge_model = opt_string_flag(&flags, "judge-model")?;
    let (session_flag, force) = ownership_args(&flags)?;
    dispatch("cells judge-record", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let raw = read_file_text(&file, "judge verdict")?;
        let verdict = match parse_json_js(&raw, false) {
            JsParse::Value(v) => v,
            // free prose — validator rejects it; a lone-surrogate escape is
            // "not JSON this CLI can parse" and takes the same branch.
            JsParse::NotJson => Value::String(raw.clone()),
        };
        let (ok, errors) = validate_judge_verdict(&verdict);
        if !ok {
            return Err(Fail::Thrown(format!(
                "recordJudgeVerdict: cell \"{id}\" verdict rejected against schema \"judge-verdict/1\" — {} FIX: the judge dispatch must return the schema verbatim (never free prose); re-dispatch once, then record model_independence \"unverified\" if it fails again (D5).",
                errors.join(" ")
            )));
        }
        let verdict_map = match &verdict {
            Value::Object(m) => m.clone(),
            _ => unreachable!("validated object"),
        };
        let independence = derive_model_independence(
            builder_model.as_deref(),
            builder_model.as_deref().map(|_| PINNED_MODEL_STATUS),
            judge_model.as_deref(),
            judge_model.as_deref().map(|_| PINNED_MODEL_STATUS),
        );
        prescan_claim(&root, &id)?;
        delegate_only(load_taxonomy(&root))?;
        let mut reopened = false;
        let mut guard = acquire_named_lock(&root, &format!("cells:{id}"))?;
        let saved = (|| -> MR<Value> {
            assert_not_archived(&root, "recordJudgeVerdict", &id)?;
            let cell = read_cell_norm(&root, &id)?;
            let Some(cell) = cell else {
                return Err(Fail::Thrown(format!("recordJudgeVerdict: cell \"{id}\" not found.")));
            };
            let Value::Object(mut cell_map) = cell else { return Err(Fail::Delegate) };
            let mut entry = Map::new();
            entry.insert("schema".into(), verdict_map.get("schema").cloned().unwrap_or(Value::Null));
            entry.insert("verdict".into(), verdict_map.get("verdict").cloned().unwrap_or(Value::Null));
            entry.insert("checks".into(), verdict_map.get("checks").cloned().unwrap_or(Value::Null));
            entry.insert(
                "failure_signature".into(),
                match verdict_map.get("failure_signature") {
                    None | Some(Value::Null) => Value::Null, // ?? null
                    Some(v) => v.clone(),
                },
            );
            entry.insert(
                "fixability".into(),
                verdict_map.get("fixability").cloned().unwrap_or(Value::Null),
            );
            entry.insert(
                "confidence".into(),
                verdict_map.get("confidence").cloned().unwrap_or(Value::Null),
            );
            let model_or_null = |m: &Option<String>| match m {
                Some(s) if !js_trim(s).is_empty() => Value::String(s.clone()),
                _ => Value::Null,
            };
            entry.insert("builder_model".into(), model_or_null(&builder_model));
            entry.insert("judge_model".into(), model_or_null(&judge_model));
            entry.insert("model_independence".into(), Value::String(independence.to_string()));
            entry.insert("recorded_at".into(), Value::String(utc_now()));
            let mut trace = merge_trace(cell_map.get("trace"))?;
            trace = guard_claim_ownership(
                &root,
                &id,
                trace,
                "recordJudgeVerdict",
                session_flag.as_deref(),
                force,
            )?;
            let existing: Vec<Value> = match trace.get("semantic_judge") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            };
            let mut next = existing;
            next.push(Value::Object(entry));
            trace.insert("semantic_judge".into(), Value::Array(next));
            let needs_revision =
                matches!(verdict_map.get("verdict"), Some(Value::String(s)) if s == "NEEDS_REVISION");
            let capped = matches!(cell_map.get("status"), Some(Value::String(s)) if s == "capped");
            if needs_revision && capped {
                cell_map.insert("status".into(), Value::String("open".into()));
                let mut rework = Map::new();
                rework.insert("at".into(), Value::String(utc_now()));
                rework.insert(
                    "reason".into(),
                    Value::String("NEEDS_REVISION semantic-judge verdict recorded after cap".into()),
                );
                trace.insert("reopened_for_rework".into(), Value::Object(rework));
                trace = release_trace(trace);
                reopened = true;
                log_decision(
                    &root,
                    &format!(
                        "«cells judge-record: cell \"{id}\" reopened capped->open by a NEEDS_REVISION semantic-judge verdict»"
                    ),
                    "A NEEDS_REVISION verdict recorded after cap must have teeth: the cell is reopened to open (clean slate) for rework, with claim + verify evidence cleared, instead of being silently logged into an inert trace entry (hardening-3) or left falsely \"claimed\" with stale verify_passed that a later PASS verdict could re-cap on with zero fresh verify (hardening-1-7-10 D7).",
                    &["cells", "judge"],
                )?;
            }
            cell_map.insert("trace".into(), Value::Object(trace));
            let value = Value::Object(cell_map);
            write_cell(&root, &value)?;
            Ok(value)
        })();
        guard.release();
        let cell = saved?;
        if reopened {
            release_claim_file_best_effort(&root, &id);
        }
        let entries = cell.get("trace").and_then(|t| t.get("semantic_judge"));
        let latest = match entries {
            Some(Value::Array(a)) => a.last().cloned().unwrap_or(Value::Null),
            _ => Value::Null,
        };
        let text = format!(
            "Recorded judge verdict on {}: {} (model_independence={}).",
            js_string_or_undefined(cell.get("id")),
            js_string_or_undefined(latest.get("verdict")),
            js_string_or_undefined(latest.get("model_independence"))
        );
        Ok(Out::Emit(cell, text, 0))
    })
}

// ── cells schedule (read-only computed schedule) ───────────────────────────

fn run_schedule(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["feature"]) {
        return None;
    }
    let feature = flags.truthy_str("feature").map(str::to_string);
    dispatch("cells schedule", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let cells = list_cells(&root, feature.as_deref(), None).map_err(|_| Fail::Delegate)?;
        // A schedulable cell with a non-string id takes JS-exotic Map-key
        // paths (undefined keys, undefined-in-sort) — Node's.
        for cell in &cells {
            let schedulable = matches!(cell.get("status"), Some(Value::String(s)) if s == "open" || s == "claimed");
            if schedulable && !matches!(cell.get("id"), Some(Value::String(_))) {
                return Err(Fail::Delegate);
            }
        }
        let schedule = compute_schedule(&cells);
        let mut lines: Vec<String> = Vec::new();
        if schedule.waves.is_empty() {
            lines.push("No schedulable cells.".to_string());
        } else {
            for (index, wave) in schedule.waves.iter().enumerate() {
                lines.push(format!("Wave {}: {}", index + 1, wave.join(", ")));
            }
        }
        if !schedule.cycles.is_empty() {
            lines.push("Cycles:".to_string());
            for cycle in &schedule.cycles {
                lines.push(format!("- {}", cycle.join(" -> ")));
            }
        }
        if !schedule.unsatisfiable.is_empty() {
            lines.push("Unsatisfiable deps:".to_string());
            for (cell, dep, reason) in &schedule.unsatisfiable {
                lines.push(format!("- {cell} -> {dep} ({reason})"));
            }
        }
        if !schedule.empty_files.is_empty() {
            lines.push(format!("Empty files: {}", schedule.empty_files.join(", ")));
        }
        let result = json!({
            "waves": schedule.waves,
            "diagnostics": {
                "cycles": schedule.cycles,
                "unsatisfiable_deps": schedule
                    .unsatisfiable
                    .iter()
                    .map(|(cell, dep, reason)| json!({"cell": cell, "dep": dep, "reason": reason}))
                    .collect::<Vec<_>>(),
                "empty_files": schedule.empty_files,
            }
        });
        Ok(Out::Emit(result, lines.join("\n"), 0))
    })
}

// ── cells archive / unarchive ──────────────────────────────────────────────

const ARCHIVE_JOURNAL_FILE: &str = ".journal.json";

fn cells_archive_dir(root: &Path, feature: &str) -> PathBuf {
    cells_dir(root).join(ARCHIVE_DIR_NAME).join(feature)
}

fn archive_journal_path(root: &Path, feature: &str) -> PathBuf {
    cells_archive_dir(root, feature).join(ARCHIVE_JOURNAL_FILE)
}

fn archive_summary_file(root: &Path) -> PathBuf {
    cells_dir(root).join(ARCHIVE_DIR_NAME).join("summary.json")
}

/// assertValidFeatureSlug (hardening-1).
fn assert_valid_feature_slug(verb: &str, feature: &str) -> MR<String> {
    if js_trim(feature).is_empty() {
        return Err(Fail::Thrown(format!("{verb}: feature is required.")));
    }
    let pattern_ok = !feature.is_empty()
        && feature
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    let all_dots = !feature.is_empty() && feature.chars().all(|c| c == '.');
    if !pattern_ok || all_dots {
        return Err(Fail::Thrown(format!(
            "{verb}: invalid feature \"{feature}\" — use letters, digits, dot, dash, underscore only (no path separators, and never \".\" or \"..\"). Refusing before any file is touched."
        )));
    }
    Ok(feature.to_string())
}

/// recoverArchiveJournal (hardening-1-7-10 D4) — direction-agnostic repair.
fn recover_archive_journal(root: &Path, feature: &str) -> MR<()> {
    let journal_path = archive_journal_path(root, feature);
    let journal = match read_json(&journal_path) {
        ReadJson::Missing => return Ok(()), // removeFileIfExists on nothing
        // readJson(journalPath, null) fails open to null, and `!journal`
        // then DELETES the journal and returns — the file is present here,
        // so the removal is what makes this arm equal to Node's.
        ReadJson::Corrupt => {
            warn_corrupt_json_once(&journal_path);
            crate::fsutil::remove_file_if_exists(&journal_path);
            return Ok(());
        }
        ReadJson::Parsed(v) => v,
    };
    let planned = journal.get("planned");
    let Some(Value::Array(planned)) = planned else {
        crate::fsutil::remove_file_if_exists(&journal_path);
        return Ok(());
    };
    for m in planned {
        let (Some(Value::String(from)), Some(Value::String(to))) = (m.get("from"), m.get("to")) else {
            continue;
        };
        let from_p = Path::new(from);
        let to_p = Path::new(to);
        if to_p.exists() && !from_p.exists() {
            let _ = std::fs::rename(to_p, from_p); // best-effort
        }
    }
    crate::fsutil::remove_file_if_exists(&journal_path);
    Ok(())
}

/// archivedSummary — {} on absent/shape-less, and on corrupt too (warn +
/// readJson's `{}` fallback).
fn archived_summary(root: &Path) -> MR<Map<String, Value>> {
    match read_store_json(&archive_summary_file(root))? {
        Some(Value::Object(m)) => Ok(m),
        _ => Ok(Map::new()),
    }
}

fn assert_archive_dir_contained(verb: &str, root: &Path, archive_dir: &Path) -> MR<()> {
    let base = cells_dir(root).join(ARCHIVE_DIR_NAME);
    let base_s = base.to_string_lossy().into_owned();
    let resolved_s = archive_dir.to_string_lossy().into_owned();
    let sep = std::path::MAIN_SEPARATOR;
    if resolved_s == base_s || resolved_s.starts_with(&format!("{base_s}{sep}")) {
        return Ok(());
    }
    Err(Fail::Thrown(format!(
        "{verb}: resolved archive path \"{resolved_s}\" escapes the archive root \"{base_s}\" — refusing before any file is touched."
    )))
}

fn run_archive(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["feature"]) {
        return None;
    }
    let feature = flags.req_str("feature")?.to_string();
    dispatch("cells archive", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        // handleCellsArchive: active-feature guard from state.json.
        let state = bstate::read_state_brief(&root).map_err(|_| Fail::Delegate)?;
        if matches!(&state.feature, Value::String(f) if js_truthy(&state.feature) && *f == feature) {
            return Err(Fail::Thrown(format!(
                "cells archive: feature \"{feature}\" is the active feature (state.feature) — only a closed/inactive feature can be archived. Switch or clear state.feature first, or archive a different feature."
            )));
        }
        let feature = assert_valid_feature_slug("archiveFeature", &feature)?;
        let mut guard = acquire_named_lock(&root, "cells-archive")?;
        let outcome = (|| -> MR<(Vec<Value>, f64, f64)> {
            recover_archive_journal(&root, &feature)?;
            let cells = list_cells(&root, Some(&feature), None).map_err(|_| Fail::Delegate)?;
            if cells.is_empty() {
                return Err(Fail::Thrown(format!(
                    "archiveFeature: no cells found for feature \"{feature}\" — nothing to archive."
                )));
            }
            let terminal = |c: &Value| {
                matches!(c.get("status"), Some(Value::String(s)) if s == "capped" || s == "dropped")
            };
            let non_terminal: Vec<&Value> = cells.iter().filter(|c| !terminal(c)).collect();
            if !non_terminal.is_empty() {
                let named = non_terminal
                    .iter()
                    .map(|c| {
                        format!(
                            "{} ({})",
                            js_string_or_undefined(c.get("id")),
                            js_string_or_undefined(c.get("status"))
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Fail::Thrown(format!(
                    "archiveFeature: feature \"{feature}\" has non-terminal cell(s) — {named} — only a feature whose cells are ALL capped/dropped can be archived."
                )));
            }
            let archive_dir = cells_archive_dir(&root, &feature);
            assert_archive_dir_contained("archiveFeature", &root, &archive_dir)?;
            std::fs::create_dir_all(&archive_dir).map_err(|e| Fail::Thrown(format!("{e}")))?;
            // Every cell.id must be a plain string for path.join — anything
            // else takes a V8 TypeError in Node.
            let mut planned: Vec<(Value, String, PathBuf, PathBuf)> = Vec::new();
            for cell in &cells {
                let Some(Value::String(cid)) = cell.get("id") else { return Err(Fail::Delegate) };
                let status = js_string_or_undefined(cell.get("status"));
                planned.push((
                    cell.get("id").cloned().unwrap_or(Value::Null),
                    status,
                    cell_file(&root, cid),
                    archive_dir.join(format!("{cid}.json")),
                ));
            }
            let collisions: Vec<String> = planned
                .iter()
                .filter(|(_, _, _, to)| to.exists())
                .map(|(id, _, _, _)| jsjson::js_to_string(id))
                .collect();
            if !collisions.is_empty() {
                return Err(Fail::Thrown(format!(
                    "archiveFeature: feature \"{feature}\" refused — a archived file already exists for {}. Refusing before any file is touched (never overwrite existing data).",
                    collisions.join(", ")
                )));
            }
            let planned_value = Value::Array(
                planned
                    .iter()
                    .map(|(id, _, from, to)| {
                        json!({
                            "id": id,
                            "from": from.to_string_lossy(),
                            "to": to.to_string_lossy(),
                        })
                    })
                    .collect(),
            );
            write_json_atomic(
                &archive_journal_path(&root, &feature),
                &json!({"op": "archive", "feature": feature, "planned": planned_value, "started_at": utc_now()}),
            )
            .map_err(|e| Fail::Thrown(format!("{e}")))?;
            let mut moved: Vec<(PathBuf, PathBuf, Value)> = Vec::new();
            let mut capped = 0f64;
            let mut dropped = 0f64;
            for (id, status, from, to) in &planned {
                if let Err(e) = std::fs::rename(from, to) {
                    for (from_r, to_r, _) in moved.iter().rev() {
                        let _ = std::fs::rename(to_r, from_r); // best-effort rollback
                    }
                    crate::fsutil::remove_file_if_exists(&archive_journal_path(&root, &feature));
                    return Err(Fail::Thrown(format!("{e}"))); // residual: libuv message in Node
                }
                moved.push((from.clone(), to.clone(), id.clone()));
                if status == "capped" {
                    capped += 1.0;
                } else if status == "dropped" {
                    dropped += 1.0;
                }
            }
            let mut summary = archived_summary(&root)?;
            let mut entry = Map::new();
            entry.insert("capped".into(), Value::Number(Number::from_f64(capped).unwrap()));
            entry.insert("dropped".into(), Value::Number(Number::from_f64(dropped).unwrap()));
            entry.insert("archived_at".into(), Value::String(utc_now()));
            summary.insert(feature.clone(), Value::Object(entry));
            write_json_atomic(&archive_summary_file(&root), &Value::Object(summary))
                .map_err(|e| Fail::Thrown(format!("{e}")))?;
            crate::fsutil::remove_file_if_exists(&archive_journal_path(&root, &feature));
            Ok((moved.into_iter().map(|(_, _, id)| id).collect(), capped, dropped))
        })();
        guard.release();
        let (moved, capped, dropped) = outcome?;
        let result = json!({
            "feature": feature,
            "moved": moved,
            "counts": {"capped": capped, "dropped": dropped},
        });
        let text = format!(
            "Archived feature \"{feature}\": {} cell(s) moved (capped={} dropped={}).",
            moved.len(),
            jsjson::js_f64_to_string(capped),
            jsjson::js_f64_to_string(dropped)
        );
        Ok(Out::Emit(result, text, 0))
    })
}

fn run_unarchive(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !rsv::keys_known(&flags, &["feature"]) {
        return None;
    }
    let feature = flags.req_str("feature")?.to_string();
    dispatch("cells unarchive", use_json, t0, move |ctx| {
        let root = ctx.root.clone();
        let feature = assert_valid_feature_slug("unarchiveFeature", &feature)?;
        let mut guard = acquire_named_lock(&root, "cells-archive")?;
        let outcome = (|| -> MR<Vec<String>> {
            recover_archive_journal(&root, &feature)?;
            let archive_dir = cells_archive_dir(&root, &feature);
            assert_archive_dir_contained("unarchiveFeature", &root, &archive_dir)?;
            let entries = match std::fs::read_dir(&archive_dir) {
                Ok(e) => e,
                Err(_) => {
                    return Err(Fail::Thrown(format!(
                        "unarchiveFeature: no archived cells found for feature \"{feature}\"."
                    )))
                }
            };
            let names: Vec<String> = entries
                .flatten()
                .filter_map(|e| e.file_name().to_str().map(str::to_string))
                .collect();
            let json_files: Vec<String> = names
                .into_iter()
                .filter(|f| f.ends_with(".json") && f != ARCHIVE_JOURNAL_FILE)
                .collect();
            if json_files.is_empty() {
                return Err(Fail::Thrown(format!(
                    "unarchiveFeature: no archived cells found for feature \"{feature}\"."
                )));
            }
            let planned: Vec<(String, PathBuf, PathBuf)> = json_files
                .iter()
                .map(|f| {
                    (
                        f[..f.len() - ".json".len()].to_string(),
                        archive_dir.join(f),
                        cells_dir(&root).join(f),
                    )
                })
                .collect();
            let collisions: Vec<String> = planned
                .iter()
                .filter(|(_, _, to)| to.exists())
                .map(|(id, _, _)| id.clone())
                .collect();
            if !collisions.is_empty() {
                return Err(Fail::Thrown(format!(
                    "unarchiveFeature: feature \"{feature}\" refused — a active file already exists for {}. Refusing before any file is touched (never overwrite existing data).",
                    collisions.join(", ")
                )));
            }
            let planned_value = Value::Array(
                planned
                    .iter()
                    .map(|(id, from, to)| {
                        json!({"id": id, "from": from.to_string_lossy(), "to": to.to_string_lossy()})
                    })
                    .collect(),
            );
            write_json_atomic(
                &archive_journal_path(&root, &feature),
                &json!({"op": "unarchive", "feature": feature, "planned": planned_value, "started_at": utc_now()}),
            )
            .map_err(|e| Fail::Thrown(format!("{e}")))?;
            let mut moved: Vec<(PathBuf, PathBuf, String)> = Vec::new();
            for (id, from, to) in &planned {
                if let Err(e) = std::fs::rename(from, to) {
                    for (from_r, to_r, _) in moved.iter().rev() {
                        let _ = std::fs::rename(to_r, from_r);
                    }
                    crate::fsutil::remove_file_if_exists(&archive_journal_path(&root, &feature));
                    return Err(Fail::Thrown(format!("{e}")));
                }
                moved.push((from.clone(), to.clone(), id.clone()));
            }
            crate::fsutil::remove_file_if_exists(&archive_journal_path(&root, &feature));
            let _ = std::fs::remove_dir(&archive_dir); // best-effort
            let mut summary = archived_summary(&root)?;
            summary.shift_remove(&feature);
            write_json_atomic(&archive_summary_file(&root), &Value::Object(summary))
                .map_err(|e| Fail::Thrown(format!("{e}")))?;
            Ok(moved.into_iter().map(|(_, _, id)| id).collect())
        })();
        guard.release();
        let moved = outcome?;
        let result = json!({"feature": feature, "moved": moved});
        let text = format!(
            "Unarchived feature \"{feature}\": {} cell(s) restored to .bee/cells/.",
            moved.len()
        );
        Ok(Out::Emit(result, text, 0))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_cell_fixture(root: &Path, id: &str, body: &Value) {
        let dir = cells_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), jsjson::stringify_pretty(body)).unwrap();
    }

    fn cell(id: &str, status: &str, feature: &str, deps: Value) -> Value {
        json!({
            "id": id,
            "title": format!("title {id}"),
            "status": status,
            "lane": "tiny",
            "feature": feature,
            "deps": deps,
            "verify": "echo ok",
        })
    }

    // ── natural sort: every pair below is pinned to a live V8
    //    `a.localeCompare(b, 'en', {numeric: true})` probe result. ──────────
    #[test]
    fn natural_cmp_matches_v8_locale_compare_probes() {
        use Ordering::{Equal, Greater, Less};
        let probes: &[(&str, &str, Ordering)] = &[
            ("a01", "a1", Equal),
            ("01", "1", Equal),
            ("a00", "a0", Equal),
            ("a001b", "a1b", Equal),
            ("a2", "a10", Less),
            ("f1-2", "f1-10", Less),
            ("w-2", "w-10", Less),
            ("a1b2", "a1b10", Less),
            ("a10b", "a9c", Greater),
            ("a0", "a", Greater),
            ("a1", "a1a", Less),
            ("a", "a-", Less),
            ("x", "xx", Less),
            ("a-1", "a.1", Less),
            ("a-1", "a_1", Greater),
            ("a.1", "a_1", Greater),
            ("-", ".", Less),
            (".", "_", Greater),
            ("_", "-", Less),
            ("a", "1", Greater),
            ("0", "-", Greater),
            ("aa", "a-a", Greater),
            ("abc", "ab-c", Greater),
            ("a-2", "a2", Less),
            ("x-1", "x1", Less),
            ("x.y", "xy", Less),
            ("x_y", "x-y", Less),
            ("a b", "a_b", Less),
            ("a 1", "a-1", Less),
            ("a", "A", Less),
            ("A", "a", Greater),
            ("aB", "ab", Greater),
            ("A1", "a1", Greater),
            ("Ab", "aC", Less),     // primary (b<c) beats the earlier case diff
            ("a01B", "a1b", Greater), // digits tie; tertiary B>b
            ("a01b", "A1b", Less),  // tertiary a<A, digits carry no case weight
            ("a01x", "a1X", Less),
            ("ABC", "abd", Less),
            ("demo-1", "demo-1", Equal),
        ];
        for (a, b, want) in probes {
            assert_eq!(natural_cmp(a, b), *want, "natural_cmp({a:?}, {b:?})");
        }
    }

    #[test]
    fn list_cells_sorts_naturally_and_skips_non_cells() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "w-10", &cell("w-10", "open", "f", json!([])));
        write_cell_fixture(root, "w-2", &cell("w-2", "open", "f", json!([])));
        write_cell_fixture(root, "a-1", &cell("a-1", "capped", "g", json!([])));
        // Non-.json and directory entries are never cells.
        std::fs::write(cells_dir(root).join("notes.txt"), "x").unwrap();
        std::fs::create_dir_all(cells_dir(root).join(ARCHIVE_DIR_NAME).join("old")).unwrap();
        // A literal-null cell file and a primitive cell file are skipped.
        std::fs::write(cells_dir(root).join("nul.json"), "null").unwrap();
        std::fs::write(cells_dir(root).join("num.json"), "5").unwrap();
        let ids: Vec<String> = list_cells(root, None, None)
            .unwrap_or_else(|_| panic!("no delegate expected"))
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["a-1", "w-2", "w-10"]);
    }

    #[test]
    fn list_cells_filters_by_feature_and_status_strictly() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "f-1", &cell("f-1", "open", "feat", json!([])));
        write_cell_fixture(root, "f-2", &cell("f-2", "capped", "feat", json!([])));
        write_cell_fixture(root, "g-1", &cell("g-1", "open", "other", json!([])));
        // A cell with NO feature field never matches a truthy filter.
        write_cell_fixture(root, "h-1", &json!({"id": "h-1", "status": "open"}));
        let feat_open: Vec<String> = list_cells(root, Some("feat"), Some("open"))
            .unwrap_or_else(|_| panic!())
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(feat_open, vec!["f-1"]);
        let all_open: Vec<String> = list_cells(root, None, Some("open"))
            .unwrap_or_else(|_| panic!())
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(all_open, vec!["f-1", "g-1", "h-1"]);
    }

    #[test]
    fn list_cells_skips_corrupt_cell_and_delegates_on_array_cell() {
        // CUTOVER: a corrupt cell file is no longer a delegation. readJson
        // warns and returns null, `!cell` skips it, and the rest of the store
        // still lists — exactly Node's fail-open.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        std::fs::write(cells_dir(root).join("bad.json"), "{nope").unwrap();
        write_cell_fixture(root, "good-1", &cell("good-1", "open", "f", json!([])));
        let listed = list_cells(root, None, None).expect("corrupt JSON must not delegate");
        let ids: Vec<String> = listed.iter().map(|c| js_string_or_undefined(c.get("id"))).collect();
        assert_eq!(ids, vec!["good-1"], "the corrupt file is skipped, the good one survives");

        // A lone-surrogate escape (V8's JSON.parse took it; serde never can)
        // is just corrupt input now — same skip, no delegation.
        let tmp3 = tempfile::tempdir().unwrap();
        let root3 = tmp3.path();
        std::fs::create_dir_all(cells_dir(root3)).unwrap();
        std::fs::write(cells_dir(root3).join("sur.json"), r#"{"id":"sur-1","title":"\ud800"}"#).unwrap();
        write_cell_fixture(root3, "good-2", &cell("good-2", "open", "f", json!([])));
        let listed = list_cells(root3, None, None).expect("lone surrogate must not delegate");
        let ids: Vec<String> = listed.iter().map(|c| js_string_or_undefined(c.get("id"))).collect();
        assert_eq!(ids, vec!["good-2"]);

        // JS-exotic shapes are NOT in that class and still delegate.
        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        std::fs::create_dir_all(cells_dir(root2)).unwrap();
        std::fs::write(cells_dir(root2).join("arr.json"), "[1,2]").unwrap();
        assert!(list_cells(root2, None, None).is_err(), "array cell (typeof 'object') must delegate");
    }

    // ── readiness (depsAllCapped) ──────────────────────────────────────────
    #[test]
    fn ready_requires_every_dep_capped() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "base-1", &cell("base-1", "capped", "f", json!([])));
        write_cell_fixture(root, "base-2", &cell("base-2", "open", "f", json!([])));
        write_cell_fixture(root, "ok-1", &cell("ok-1", "open", "f", json!(["base-1"])));
        write_cell_fixture(root, "wait-1", &cell("wait-1", "open", "f", json!(["base-1", "base-2"])));
        write_cell_fixture(root, "ghost-1", &cell("ghost-1", "open", "f", json!(["missing-9"])));
        write_cell_fixture(root, "free-1", &cell("free-1", "open", "f", json!([])));
        // deps: falsy value behaves as [] (readiness unconditional).
        write_cell_fixture(root, "nul-1", &json!({"id": "nul-1", "status": "open", "deps": null}));
        let Handled::Emit { result, text } = handle_ready(root, None).unwrap() else {
            panic!("ready never errors")
        };
        let ids: Vec<String> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["base-2", "free-1", "nul-1", "ok-1"]);
        assert!(text.contains("ok-1 [open] (tiny) title ok-1"));
    }

    #[test]
    fn ready_counts_archived_capped_dep_via_read_cell_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let arch = cells_dir(root).join(ARCHIVE_DIR_NAME).join("done-feature");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(
            arch.join("old-1.json"),
            jsjson::stringify_pretty(&cell("old-1", "capped", "done-feature", json!([]))),
        )
        .unwrap();
        write_cell_fixture(root, "next-1", &cell("next-1", "open", "f", json!(["old-1"])));
        let Handled::Emit { result, .. } = handle_ready(root, None).unwrap() else { panic!() };
        let ids: Vec<String> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["next-1"], "archived capped dep satisfies readiness");
    }

    #[test]
    fn ready_delegates_on_truthy_non_array_deps_and_falsy_dep_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "s-1", &json!({"id": "s-1", "status": "open", "deps": "x-1"}));
        assert!(handle_ready(root, None).is_err(), "string deps (char iteration) delegates");

        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        write_cell_fixture(root2, "z-1", &json!({"id": "z-1", "status": "open", "deps": [""]}));
        let Handled::Emit { result, text } = handle_ready(root2, None).unwrap() else { panic!() };
        assert_eq!(result.as_array().unwrap().len(), 0, "falsy dep never resolves -> not ready");
        assert_eq!(text, "No ready cells.");
    }

    // ── renderers ──────────────────────────────────────────────────────────
    #[test]
    fn renderers_match_node_templates_and_empty_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let Handled::Emit { text, .. } = handle_list(root, None, None).unwrap() else { panic!() };
        assert_eq!(text, "No cells.");
        let Handled::Emit { text, .. } = handle_ready(root, None).unwrap() else { panic!() };
        assert_eq!(text, "No ready cells.");

        // Missing fields coerce like template literals: "undefined".
        write_cell_fixture(root, "bare-1", &json!({"id": "bare-1"}));
        let Handled::Emit { text, .. } = handle_list(root, None, None).unwrap() else { panic!() };
        assert_eq!(text, "bare-1 [undefined] (undefined) undefined");
    }

    // ── show: trace assembly + verify_owner placement + error path ─────────
    #[test]
    fn show_inserts_verify_owner_after_verify_preserving_trace_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let body = json!({
            "id": "t-1",
            "title": "t",
            "status": "capped",
            "verify": "run checks",
            "trace": {
                "claimed_by": "w1",
                "attempts": [{"n": 1, "ok": false}, {"n": 2, "ok": true}],
                "verify_passed": true
            }
        });
        write_cell_fixture(root, "t-1", &body);
        let Handled::Emit { result, text } = handle_show(root, "t-1").unwrap() else { panic!() };
        let keys: Vec<&String> = result.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["id", "title", "status", "verify", "verify_owner", "trace"]);
        assert_eq!(
            result.get("verify_owner"),
            Some(&Value::String(VERIFY_OWNER_ANNOTATION.into()))
        );
        // text is the pretty render of the SAME annotated object, trace intact.
        assert_eq!(text, jsjson::stringify_pretty(&result));
        assert!(text.contains("\"attempts\": ["));
    }

    #[test]
    fn show_appends_verify_owner_when_cell_has_no_verify_key() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "nv-1", &json!({"id": "nv-1", "status": "open"}));
        let Handled::Emit { result, .. } = handle_show(root, "nv-1").unwrap() else { panic!() };
        let keys: Vec<&String> = result.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["id", "status", "verify_owner"]);
    }

    #[test]
    fn show_not_found_message_matches_node_and_reads_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        match handle_show(root, "nope-1").unwrap() {
            Handled::Error(msg) => assert_eq!(msg, "Cell \"nope-1\" not found."),
            _ => panic!("expected the not-found error"),
        }
        // Malformed id short-circuits to the same message (ID_PATTERN).
        match handle_show(root, "../evil").unwrap() {
            Handled::Error(msg) => assert_eq!(msg, "Cell \"../evil\" not found."),
            _ => panic!("expected the not-found error"),
        }
        // Archived cell resolves through the archive fallback.
        let arch = cells_dir(root).join(ARCHIVE_DIR_NAME).join("f");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(
            arch.join("arc-1.json"),
            jsjson::stringify_pretty(&cell("arc-1", "capped", "f", json!([]))),
        )
        .unwrap();
        match handle_show(root, "arc-1").unwrap() {
            Handled::Emit { result, .. } => {
                assert_eq!(result.get("id"), Some(&Value::String("arc-1".into())))
            }
            _ => panic!("archived cell must resolve"),
        }
    }

    #[test]
    fn show_reports_not_found_on_corrupt_cell_and_delegates_on_non_object() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        // CUTOVER: readCell warns and falls back to null, so `show` reaches
        // the SAME not-found refusal a missing cell reaches — no delegation.
        std::fs::write(cells_dir(root).join("bad-1.json"), "{nope").unwrap();
        match handle_show(root, "bad-1").expect("corrupt cell must not delegate") {
            Handled::Error(msg) => assert_eq!(msg, "Cell \"bad-1\" not found."),
            _ => panic!("corrupt cell must take readCell's null fallback"),
        }
        // Same for a lone-surrogate escape.
        std::fs::write(cells_dir(root).join("sur-1.json"), r#"{"id":"sur-1","t":"\udfff"}"#).unwrap();
        match handle_show(root, "sur-1").expect("lone surrogate must not delegate") {
            Handled::Error(msg) => assert_eq!(msg, "Cell \"sur-1\" not found."),
            _ => panic!("lone-surrogate cell must take readCell's null fallback"),
        }
        std::fs::write(cells_dir(root).join("num-1.json"), "5").unwrap();
        assert!(handle_show(root, "num-1").is_err(), "truthy non-object cell delegates");
    }

    // ── argv routing ───────────────────────────────────────────────────────
    #[test]
    fn parse_flags_accepts_only_provable_shapes() {
        let os = |v: &[&str]| v.iter().map(OsString::from).collect::<Vec<_>>();
        // list: --json --feature f --status open (both flag forms).
        let f = parse_flags(Verb::List, &os(&["--json", "--feature", "f", "--status=open"])).unwrap();
        assert!(f.json);
        assert_eq!(f.feature.as_deref(), Some("f"));
        assert_eq!(f.status.as_deref(), Some("open"));
        // last-wins overwrite, like Node's flags[name] = value.
        let f = parse_flags(Verb::List, &os(&["--feature=a", "--feature=b"])).unwrap();
        assert_eq!(f.feature.as_deref(), Some("b"));
        // Delegations: bare positional, unknown flag, --help, missing value,
        // a `--`-shaped value token, and per-verb flag sets.
        assert!(parse_flags(Verb::List, &os(&["foo"])).is_none());
        assert!(parse_flags(Verb::List, &os(&["--bogus", "x"])).is_none());
        assert!(parse_flags(Verb::List, &os(&["--help"])).is_none());
        assert!(parse_flags(Verb::List, &os(&["--feature"])).is_none());
        assert!(parse_flags(Verb::List, &os(&["--feature", "--json"])).is_none());
        assert!(parse_flags(Verb::Ready, &os(&["--status", "open"])).is_none());
        assert!(parse_flags(Verb::Show, &os(&["--feature", "f"])).is_none());
        // show requires --id at try_native level; parse itself allows any value.
        let f = parse_flags(Verb::Show, &os(&["--id", "-weird"])).unwrap();
        assert_eq!(f.id.as_deref(), Some("-weird"));
    }

    // ═══ mutating-verb building blocks ════════════════════════════════════

    fn thrown<T>(r: MR<T>) -> String {
        match r {
            Err(Fail::Thrown(m)) => m,
            Err(Fail::Delegate) => panic!("unexpected delegate"),
            Ok(_) => panic!("expected a thrown refusal"),
        }
    }

    // ── failure-signature normalizer: pinned against live Node runs of
    //    lib/cells.mjs normalizeFailureSignature. ──────────────────────────
    #[test]
    fn failure_signature_matches_node_vectors() {
        let vectors: &[(&str, &str)] = &[
            ("boom", "81f52337ebb4"),
            ("", "e3b0c44298fc"),
            ("ok line\nError: deadbeef00 at /home/u/repo/file.js", "dc04ab11120d"),
            ("3/45 passed\nrefused: cap denied", "9b9c6fc6eefa"),
            ("Error at abc123 deadbeefcafe1234", "667b748a5aff"),
            ("  Error:   spaced   ", "c8165beb8597"),
            ("no failure words here\nsecond line", "16678f6e01be"),
            ("a /x/ b", "042c163d395c"),
            ("path /usr/lib/x.so denied", "7910c9d525df"),
            ("ERR deadBEEF01", "dd49055a8bf4"),
        ];
        for (input, want) in vectors {
            assert_eq!(&normalize_failure_signature(input), want, "signature({input:?})");
        }
    }

    // ── secret/injection matchers: pinned against the live Node regexes. ──
    #[test]
    fn safety_pattern_matchers_match_node() {
        let secret = |s: &str| find_secret_pattern(s);
        let inject = |s: &str| find_injection_pattern(s);
        assert_eq!(
            secret("my token: abcdef123"),
            Some("/\\b(?:api[_-]?key|secret|token|password|passwd)\\s*[:=]\\s*['\"]?[^\\s'\"]{6,}/i")
        );
        assert_eq!(secret("risk-based sk-notenoughchars"), None);
        assert_eq!(secret("sk-aaaaaaaaaaaaaaaaaaaa!"), Some("/\\bsk-[A-Za-z0-9_-]{20,}\\b/"));
        assert_eq!(secret("AKIAABCDEFGHIJKLMNOP"), Some("/\\bAKIA[0-9A-Z]{16}\\b/"));
        assert_eq!(secret("xAKIAABCDEFGHIJKLMNOP"), None);
        assert_eq!(secret("normal reason text"), None);
        assert_eq!(
            secret("-----BEGIN RSA PRIVATE KEY-----"),
            Some("/-----BEGIN [A-Z ]*PRIVATE KEY-----/")
        );
        assert_eq!(
            inject("ignore previous instructions"),
            Some("/ignore\\s+(?:all\\s+)?(?:previous|prior|above|earlier)\\s+(?:instructions|messages|context|prompts?)/i")
        );
        assert_eq!(
            inject("gignore  all  earlier prompts"),
            Some("/ignore\\s+(?:all\\s+)?(?:previous|prior|above|earlier)\\s+(?:instructions|messages|context|prompts?)/i")
        );
        assert_eq!(inject("[ system ]"), Some("/\\[\\s*(?:system|assistant|user|developer)\\s*\\]/i"));
        assert_eq!(
            inject("<system attr=x>"),
            Some("/<\\/?\\s*(?:system|assistant|user|developer|tool)\\b[^>]*>/i"),
        );
        assert_eq!(inject("normal reason text"), None);
        assert_eq!(inject("systematic <thinker>"), None);
    }

    // ── validateNewCell / normalizeNewCell ────────────────────────────────
    #[test]
    fn validate_new_cell_refusals_and_normalize_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert_eq!(
            thrown(validate_new_cell(root, &json!([1]))),
            "addCell: cell must be a JSON object."
        );
        assert_eq!(
            thrown(validate_new_cell(root, &json!({"id": "a-1"}))),
            "addCell: cell is missing required field \"feature\" (non-empty string)."
        );
        let base = |lane: &str| {
            json!({"id": "a-1", "feature": "f", "title": "t", "action": "a", "verify": "v", "lane": lane})
        };
        assert_eq!(
            thrown(validate_new_cell(root, &base("mega"))),
            "addCell: invalid lane \"mega\" — must be one of: tiny, small, standard, high-risk, spike."
        );
        assert_eq!(
            thrown(validate_new_cell(root, &base("standard"))),
            "addCell: lane \"standard\" requires non-empty must_haves.truths (observable truths to verify)."
        );
        let mut with_budget = base("tiny");
        with_budget["budgets"] = json!({"max_claims": 99});
        assert_eq!(
            thrown(validate_new_cell(root, &with_budget)),
            "addCell: \"budgets.max_claims\" must be an integer in [1, 9] when present, got 99."
        );
        let mut bad_key = base("tiny");
        bad_key["budgets"] = json!({"nope": 1});
        assert_eq!(
            thrown(validate_new_cell(root, &bad_key)),
            "addCell: unknown \"budgets\" key \"nope\" — must be one of: max_claims, max_failed_attempts, max_same_signature."
        );
        // no-test sentinel refused outside a declared no-test repo
        let mut sentinel = base("tiny");
        sentinel["verify"] = json!("none");
        assert!(thrown(validate_new_cell(root, &sentinel)).starts_with("addCell: verify \"none\" is refused"));
        // duplicate id
        write_cell_fixture(root, "a-1", &cell("a-1", "open", "f", json!([])));
        assert_eq!(thrown(validate_new_cell(root, &base("tiny"))), "addCell: cell \"a-1\" already exists.");
        // normalize: literal-order appends + trace defaults
        let normalized = normalize_new_cell(&json!({"id": "n-1", "title": "t"})).unwrap();
        let keys: Vec<&String> = normalized.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["id", "title", "status", "deps", "decisions", "files", "read_first", "trace"]);
        assert_eq!(normalized["status"], json!("open"));
        let trace_keys: Vec<&String> = normalized["trace"].as_object().unwrap().keys().collect();
        assert_eq!(
            trace_keys,
            vec![
                "worker",
                "outcome",
                "files_changed",
                "deviations",
                "friction",
                "capped_at",
                "behavior_change",
                "verification_evidence",
                "verify_output",
                "verify_passed",
                "claim_session"
            ]
        );
    }

    #[test]
    fn cycle_detection_and_refusal_message() {
        let cells = vec![
            json!({"id": "a", "deps": ["b"]}),
            json!({"id": "b", "deps": ["a"]}),
            json!({"id": "c", "deps": ["c"]}),
            json!({"id": "d", "deps": ["missing"]}),
        ];
        let cycles = detect_cycles(&cells);
        assert_eq!(cycles, vec![vec!["a".to_string(), "b".to_string()], vec!["c".to_string()]]);
        assert_eq!(
            format_cycle_refusal("addCell", &cycles),
            "addCell: dependency cycle refused — a -> b; c. Cycles are illegal at every dep-mutating write (D2); file overlap stays legal and is never refused."
        );
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "x-1", &cell("x-1", "open", "f", json!(["y-1"])));
        let incoming = vec![json!({"id": "y-1", "deps": ["x-1"]})];
        assert!(assert_no_cycle(root, "addCell", &incoming).is_err());
    }

    // ── regen guards ──────────────────────────────────────────────────────
    //
    // R6 CUTOVER: this used to build a fake `scripts/release_manifest.mjs` in a
    // tempdir and assert the PARSE of it. The guards read compiled-in
    // authorities now, so the fixture is gone and the test asserts the thing
    // that actually matters: the derived scope is real, the obligation fires on
    // it, and every escape hatch still works.
    #[test]
    fn regen_guard_derives_real_roots_from_the_compiled_authorities() {
        let guards = derive_regen_guards().unwrap();
        assert_eq!(guards.len(), 2, "both guards are always active — there is no absent arm");

        let manifest = &guards[0];
        assert!(
            manifest.roots.contains(&"packages/bee".to_string())
                && manifest.roots.contains(&"skills".to_string()),
            "the manifest guard must cover the shipped frame: {:?}",
            manifest.roots
        );
        assert_eq!(
            manifest.required_files,
            vec!["docs/history/codex-harness-hardening/release-manifest.json".to_string()],
            "the manifest file itself is the required file, never a covered root"
        );
        assert!(
            !manifest.roots.contains(&manifest.required_files[0]),
            "the manifest must not be both a covered root and its own required file"
        );

        let ledger = &guards[1];
        assert!(
            ledger.roots.contains(&".bee/bin/lib".to_string())
                && ledger.roots.contains(&".bee/expertise".to_string()),
            "the ledger guard must cover the vendored trees: {:?}",
            ledger.roots
        );
    }

    #[test]
    fn regen_obligation_fires_refuses_and_can_be_acked() {
        let manifest_rel = "docs/history/codex-harness-hardening/release-manifest.json";

        // A cell touching a covered root without the check refuses…
        let cell = json!({"id": "r-1", "files": ["skills/bee-hive/SKILL.md"], "verify": "echo ok"});
        let refusal = regen_obligation_refusal(cell.as_object().unwrap(), "addCell")
            .unwrap()
            .expect("must refuse");
        assert!(
            refusal.starts_with(
                "addCell: REGEN_OBLIGATION — cell \"r-1\" touches \"skills/bee-hive/SKILL.md\""
            ),
            "{refusal}"
        );
        assert!(refusal.contains("verify does not contain \"bee dev release-manifest --check\""));
        assert!(refusal.contains(&format!("files does not list \"{manifest_rel}\"")));
        // The refusal names WHERE the scope came from, so it can be checked.
        assert!(refusal.contains("devtools::release_manifest::INVENTORY_ROOTS"), "{refusal}");

        // …the ack skips it…
        let acked = json!({
            "id": "r-1",
            "files": ["skills/bee-hive/SKILL.md"],
            "verify": "x",
            "regen_obligation_ack": "wave-barrier"
        });
        assert!(regen_obligation_refusal(acked.as_object().unwrap(), "addCell")
            .unwrap()
            .is_none());

        // …and a compliant cell passes.
        let ok = json!({
            "id": "r-1",
            "files": ["skills/bee-hive/SKILL.md", manifest_rel],
            "verify": "bee dev release-manifest --check"
        });
        assert!(regen_obligation_refusal(ok.as_object().unwrap(), "addCell").unwrap().is_none());

        // The LEDGER guard fires on its own roots, with its own fix.
        let vendored = json!({"id": "r-2", "files": [".bee/bin/lib/state.mjs"], "verify": "echo ok"});
        let refusal = regen_obligation_refusal(vendored.as_object().unwrap(), "addCell")
            .unwrap()
            .expect("the ledger guard must fire on a vendored path");
        assert!(refusal.contains("bee onboard --repo-root . --json"), "{refusal}");
        assert!(refusal.contains("onboard::plan::LEDGER_COVERED_ROOTS"), "{refusal}");

        // A cell that touches nothing covered is silent.
        let unrelated = json!({"id": "r-3", "files": ["src/main.rs"], "verify": "echo ok"});
        assert!(regen_obligation_refusal(unrelated.as_object().unwrap(), "addCell")
            .unwrap()
            .is_none());
    }

    // ── claims-store protocol ─────────────────────────────────────────────
    #[test]
    fn claim_cell_file_protocol_and_release() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        // Sessionless claim: session key omitted, fence_epoch 1, floor(ttl).
        let outcome = claim_cell_file(control, None, "c-1", Some(120.9)).unwrap();
        let claim = match outcome {
            ClaimFileOutcome::Ok { claim } => claim,
            _ => panic!("first claim must win"),
        };
        let keys: Vec<&String> = claim.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["cell", "ttl_seconds", "claimed_at", "acquired_at", "fence_epoch"]);
        assert_eq!(claim["ttl_seconds"], json!(120.0));
        assert_eq!(claim["fence_epoch"], json!(1.0));
        // Second claim loses with the typed CLAIMED reason.
        match claim_cell_file(control, Some("s2"), "c-1", None).unwrap() {
            ClaimFileOutcome::Refused { code, reason } => {
                assert_eq!(code, "CLAIMED");
                assert!(reason.starts_with(
                    "cell \"c-1\" is already claimed by session \"no session (sessionless claim)\""
                ));
                assert!(reason.contains("expires "));
            }
            _ => panic!("second claim must refuse"),
        }
        // Owner-matched release removes the file; a mismatched owner leaves it.
        release_claim(control, Some("someone-else"), "c-1").unwrap();
        assert!(claims_dir(control).join("c-1.json").exists());
        release_claim(control, None, "c-1").unwrap();
        assert!(!claims_dir(control).join("c-1.json").exists());
        // Sessioned claim carries the session key before ttl_seconds.
        let claim = match claim_cell_file(control, Some("sess-9"), "c-2", None).unwrap() {
            ClaimFileOutcome::Ok { claim } => claim,
            _ => panic!(),
        };
        let keys: Vec<&String> = claim.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["cell", "session", "ttl_seconds", "claimed_at", "acquired_at", "fence_epoch"]);
        assert_eq!(claim["ttl_seconds"], json!(3600.0));
        // Bad ids throw claims.mjs requireId's exact messages.
        assert_eq!(
            thrown(claim_cell_file(control, Some("a/b"), "c-3", None).map(|_| ())),
            "session id must be a plain id (no path separators)."
        );
        assert_eq!(thrown(claim_path(control, "  ").map(|_| ())), "cell id is required.");
    }

    #[test]
    fn ownership_guard_refuses_foreign_live_claim_and_audits_force() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(claims_dir(root)).unwrap();
        // A live claim owned by another session.
        match claim_cell_file(root, Some("owner-1"), "g-1", None).unwrap() {
            ClaimFileOutcome::Ok { .. } => {}
            _ => panic!(),
        }
        let refusal = thrown(guard_claim_ownership(
            root,
            "g-1",
            default_trace(),
            "blockCell",
            Some("intruder-2"),
            false,
        ));
        assert!(refusal.starts_with("blockCell: cell \"g-1\" is claimed by session \"owner-1\""));
        assert!(refusal.ends_with("Pass --force-ownership to override (audited)."));
        // Force appends the audit row instead.
        let audited = guard_claim_ownership(
            root,
            "g-1",
            default_trace(),
            "blockCell",
            Some("intruder-2"),
            true,
        )
        .unwrap();
        let overrides = audited.get("ownership_overrides").unwrap().as_array().unwrap();
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0]["verb"], json!("blockCell"));
        assert_eq!(overrides[0]["forced_by"], json!("intruder-2"));
        assert_eq!(overrides[0]["owner_bypassed"], json!("owner-1"));
        // The owner itself passes untouched.
        let own = guard_claim_ownership(root, "g-1", default_trace(), "capCell", Some("owner-1"), false);
        assert!(own.is_ok());
    }

    // ── budgets ───────────────────────────────────────────────────────────
    fn attempt(session: &str, acquired: &str, verdict: &str, sig: Option<&str>) -> Value {
        json!({
            "n": 1, "at": format!("{acquired}x"), "claim_session": session,
            "claimed_at": acquired, "acquired_at": acquired, "worker": "w",
            "verdict": verdict, "failure_signature": sig, "note": null
        })
    }

    #[test]
    fn budget_checks_close_and_reopen_the_claim_door() {
        let mut cell = json!({"id": "b-1", "trace": {"attempts": [
            attempt("s1", "2026-01-01T00:00:00.000Z", "blocked", None),
            attempt("s2", "2026-01-02T00:00:00.000Z", "tests-red", None),
            attempt("s3", "2026-01-03T00:00:00.000Z", "fail", None),
        ]}});
        // 3 distinct acquisition pairs + the attempt being made = 4 > 3.
        match check_cell_budgets(cell.as_object().unwrap()).unwrap() {
            BudgetCheck::Refused { code, reason } => {
                assert_eq!(code, "CELL_BUDGET_EXHAUSTED");
                assert_eq!(
                    reason,
                    "cell \"b-1\" exhausted its \"max_claims\" budget (limit 3, used 4) — the claim door is closed until an audited reset."
                );
            }
            _ => panic!("must refuse"),
        }
        // A budget_resets marker restarts the counters (lexical ISO compare).
        cell["trace"]["budget_resets"] = json!([{"reset_at": "2026-01-04T00:00:00.000Z"}]);
        assert!(matches!(check_cell_budgets(cell.as_object().unwrap()).unwrap(), BudgetCheck::Ok));
        // Same-signature repeats refuse independently.
        let cell = json!({"id": "b-2", "trace": {"attempts": [
            attempt("s1", "2026-01-01T00:00:00.000Z", "fail", Some("deadbeef0000")),
            attempt("s1", "2026-01-01T00:00:00.000Z", "fail", Some("deadbeef0000")),
        ]}});
        match check_cell_budgets(cell.as_object().unwrap()).unwrap() {
            BudgetCheck::Refused { code, reason } => {
                assert_eq!(code, "REPEATED_FAILURE");
                assert!(reason.contains("failed 2 time(s) with the identical signature \"deadbeef0000\""));
            }
            _ => panic!("must refuse"),
        }
        // Declared budgets are clamped to the hard max; junk falls back.
        let cell = json!({"id": "b-3", "budgets": {"max_claims": 99, "max_failed_attempts": 0.5}});
        let budgets = resolve_cell_budgets(cell.as_object().unwrap());
        assert_eq!(budgets.max_claims, 9.0);
        assert_eq!(budgets.max_failed_attempts, 4.0);
    }

    // ── frozen judge + glob covers ────────────────────────────────────────
    #[test]
    fn frozen_judge_rules_and_declared_covers() {
        assert_eq!(frozen_judge_rule("tests/a.mjs"), Some("test sources"));
        assert_eq!(frozen_judge_rule("src/__tests__/a.mjs"), Some("test sources"));
        assert_eq!(frozen_judge_rule("src/a.test.js"), Some("test file"));
        assert_eq!(frozen_judge_rule("x/__snapshots__/a.snap"), Some("snapshot"));
        assert_eq!(frozen_judge_rule(".github/workflows/ci.yml"), Some("CI config"));
        assert_eq!(frozen_judge_rule("package-lock.json"), Some("lockfile"));
        assert_eq!(frozen_judge_rule("sub/Cargo.toml"), Some("package manifest"));
        assert_eq!(frozen_judge_rule("jest.config.mjs"), Some("test config"));
        assert_eq!(frozen_judge_rule(".bee/config.json"), Some("bee verify config"));
        assert_eq!(frozen_judge_rule("src/lib.rs"), None);
        assert_eq!(frozen_judge_rule("attestation/a.js"), None);

        let declared = vec![json!("tests/"), json!("src/*.test.js"), json!("docs/**/x.md")];
        assert!(declared_covers(&declared, "tests/anything.mjs"));
        assert!(declared_covers(&declared, "src/a.test.js"));
        assert!(!declared_covers(&declared, "src/deep/a.test.js")); // '*' never crosses '/'
        assert!(declared_covers(&declared, "docs/a/b/x.md")); // '**' does
        let hits = frozen_judge_hits(&json!(["tests/a.mjs", "src/x.js", "yarn.lock"]), &json!(["tests/"]));
        assert_eq!(hits, vec![("yarn.lock".to_string(), "lockfile")]);
    }

    // ── judge verdict schema ──────────────────────────────────────────────
    #[test]
    fn judge_verdict_validation_matches_node_errors() {
        let (ok, errors) = validate_judge_verdict(&json!("free prose"));
        assert!(!ok);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("verdict must be a JSON object per schema \"judge-verdict/1\""));

        let good = json!({
            "schema": "judge-verdict/1", "verdict": "NEEDS_REVISION",
            "checks": [{"id": "c1", "status": "FAIL", "evidence": "boom"}],
            "failure_signature": "sig-1", "fixability": "automatic", "confidence": "high"
        });
        assert!(validate_judge_verdict(&good).0);

        let bad = json!({
            "schema": "judge-verdict/2", "verdict": "PASS",
            "checks": [{"id": "c1", "status": "FAIL", "evidence": "boom"}],
            "fixability": "automatic", "confidence": "high"
        });
        let (ok, errors) = validate_judge_verdict(&bad);
        assert!(!ok);
        assert!(errors.contains(&"schema must be \"judge-verdict/1\", got \"judge-verdict/2\".".to_string()));
        assert!(errors.contains(&"verdict must not be PASS when any check has status FAIL — a PASS verdict must not carry a FAIL check.".to_string()));
        assert!(errors.contains(&"failure_signature is required (non-empty string) when any check has status FAIL.".to_string()));

        assert_eq!(derive_model_independence(Some("a"), Some("pinned"), Some("b"), Some("pinned")), "confirmed");
        assert_eq!(derive_model_independence(Some("a"), Some("pinned"), Some("a"), Some("pinned")), "same-model");
        assert_eq!(derive_model_independence(Some("a"), Some("pinned"), None, None), "unverified");
    }

    // ── schedule ──────────────────────────────────────────────────────────
    #[test]
    fn compute_schedule_waves_and_diagnostics() {
        let cells = vec![
            json!({"id": "a", "status": "open", "deps": [], "files": ["x.js"]}),
            json!({"id": "b", "status": "open", "deps": [], "files": ["x.js"]}),
            json!({"id": "c", "status": "open", "deps": ["a"], "files": ["y.js"]}),
            json!({"id": "d", "status": "capped", "deps": [], "files": ["z.js"]}),
            json!({"id": "e", "status": "open", "deps": ["ghost"], "files": []}),
            json!({"id": "f", "status": "open", "deps": ["e"], "files": ["w.js"]}),
        ];
        let s = compute_schedule(&cells);
        // a/b overlap on x.js -> b defers; c waits for a; e/f unsatisfiable.
        assert_eq!(s.waves, vec![vec!["a".to_string()], vec!["b".to_string(), "c".to_string()]]);
        assert_eq!(s.unsatisfiable, vec![("e".to_string(), "ghost".to_string(), "missing")]);
        assert_eq!(s.empty_files, vec!["e".to_string()]);
        assert!(s.cycles.is_empty());
    }

    // ── test runner ───────────────────────────────────────────────────────
    #[test]
    fn test_runner_green_and_red_record_shapes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Green run.
        let run = run_declared_tests(root, &["exit 0".to_string()]).unwrap();
        assert!(run.green);
        assert!(run.commands[0].failure_excerpt.is_none());
        let record: Value =
            serde_json::from_str(&std::fs::read_to_string(test_results_path(root)).unwrap()).unwrap();
        assert_eq!(record["green"], json!(true));
        assert_eq!(record["commands"][0]["command"], json!("exit 0"));
        assert_eq!(record["commands"][0]["exit"], json!(0));
        assert_eq!(record["commands"][0]["failure_excerpt"], Value::Null);
        // Red run: excerpt carries the tail, firstFailureLine picks line 1.
        let run = run_declared_tests(root, &["echo boom && exit 3".to_string()]).unwrap();
        assert!(!run.green);
        let excerpt = run.commands[0].failure_excerpt.as_deref().unwrap();
        assert_eq!(js_trim(excerpt), "boom");
        assert_eq!(run.commands[0].exit, Some(3.0));
        assert_eq!(first_failure_line(&run).as_deref(), Some("boom"));
        let record: Value =
            serde_json::from_str(&std::fs::read_to_string(test_results_path(root)).unwrap()).unwrap();
        assert_eq!(record["green"], json!(false));
        // Silent red: the "(no output; exit N)" placeholder.
        let run = run_declared_tests(root, &["exit 7".to_string()]).unwrap();
        assert_eq!(run.commands[0].failure_excerpt.as_deref(), Some("(no output; exit 7)"));
    }

    // ── decision log ──────────────────────────────────────────────────────
    #[test]
    fn log_decision_appends_event_and_taxonomy_candidates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // No taxonomy: bootstrap-safe append.
        log_decision(root, "«x»", "because", &["cells"]).unwrap();
        let text = std::fs::read_to_string(decisions_path(root)).unwrap();
        let event: Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
        assert_eq!(event["type"], json!("decide"));
        assert_eq!(event["decision"], json!("«x»"));
        assert_eq!(event["tags"], json!(["cells"]));
        assert_eq!(event["scope"], json!("repo"));
        // Taxonomy present: unknown tag lands in candidates[].
        std::fs::create_dir_all(root.join("docs").join("decisions")).unwrap();
        std::fs::write(
            taxonomy_path(root),
            r#"{"schema_version": 1, "tags": [{"name": "cells"}], "candidates": []}"#,
        )
        .unwrap();
        log_decision(root, "«y»", "because", &["cells", "brand-new"]).unwrap();
        let taxonomy: Value = serde_json::from_str(&std::fs::read_to_string(taxonomy_path(root)).unwrap()).unwrap();
        assert_eq!(taxonomy["candidates"], json!(["brand-new"]));
        // Safety refusal embeds the JS pattern literal.
        let refusal = thrown(log_decision(root, "token: supersecret1", "r", &["cells"]));
        assert!(refusal.starts_with("Decision rejected: field \"decision\" matches a secret pattern (/\\b(?:api[_-]?key"));
    }

    // ── writeCell funnel + archive txn helpers ────────────────────────────
    #[test]
    fn write_cell_funnel_refuses_archived_and_busy() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        // invalid id
        assert_eq!(
            thrown(write_cell(root, &json!({"id": "../evil"}))),
            "writeCell: cell needs a valid id (got \"../evil\")."
        );
        assert_eq!(
            thrown(write_cell(root, &json!({"title": "no id"}))),
            "writeCell: cell needs a valid id (got undefined)."
        );
        // archived-only id refuses CELL_ARCHIVED
        let arch = cells_dir(root).join(ARCHIVE_DIR_NAME).join("f");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(arch.join("z-1.json"), "{\"id\":\"z-1\"}").unwrap();
        assert_eq!(
            thrown(write_cell(root, &json!({"id": "z-1"}))),
            "writeCell: cell \"z-1\" is archived — unarchive its feature first (bee cells unarchive --feature <feature>)."
        );
        // live archive lock -> CELLS_ARCHIVE_BUSY
        let _held = lock::acquire_store_lock(root, "cells-archive", 1).ok().unwrap();
        let busy = thrown(write_cell(root, &json!({"id": "w-1"})));
        assert!(busy.starts_with("writeCell: cell \"w-1\" write refused — the \"cells-archive\" lock is held by pid="));
        assert!(busy.ends_with("(a live archive/unarchive transaction). Retry once it completes."));
    }

    #[test]
    fn archive_slug_journal_and_summary_helpers() {
        assert_eq!(
            thrown(assert_valid_feature_slug("archiveFeature", "../up")),
            "archiveFeature: invalid feature \"../up\" — use letters, digits, dot, dash, underscore only (no path separators, and never \".\" or \"..\"). Refusing before any file is touched."
        );
        assert_eq!(
            thrown(assert_valid_feature_slug("unarchiveFeature", "  ")),
            "unarchiveFeature: feature is required."
        );
        assert!(assert_valid_feature_slug("archiveFeature", "demo-1").is_ok());
        // Journal recovery reverses completed moves and drops the journal.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let arch = cells_archive_dir(root, "f");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        let from = cell_file(root, "j-1");
        let to = arch.join("j-1.json");
        std::fs::write(&to, "{\"id\":\"j-1\"}").unwrap(); // move completed pre-crash
        std::fs::write(
            archive_journal_path(root, "f"),
            jsjson::stringify(&json!({"op": "archive", "feature": "f", "planned": [
                {"id": "j-1", "from": from.to_string_lossy(), "to": to.to_string_lossy()}
            ]})),
        )
        .unwrap();
        recover_archive_journal(root, "f").unwrap();
        assert!(from.exists(), "completed move must be reversed");
        assert!(!archive_journal_path(root, "f").exists());
        // CUTOVER: a corrupt journal warns and takes readJson's null
        // fallback, which is the `!journal` branch — delete the journal and
        // return, leaving nothing to recover. Same as Node, minus the V8 text.
        std::fs::write(archive_journal_path(root, "f"), "{nope").unwrap();
        recover_archive_journal(root, "f").expect("corrupt journal must not delegate");
        assert!(
            !archive_journal_path(root, "f").exists(),
            "the unusable journal must be removed, exactly as `!journal` did"
        );
    }

    // ── CUTOVER: corrupt JSON is served natively ──────────────────────────
    #[test]
    fn corrupt_store_reads_fail_open_to_the_same_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();

        // read_store_json: corrupt reads exactly like missing (readJson's
        // `fallback`), with a warning instead of a delegation.
        let f = root.join(".bee").join("whatever.json");
        std::fs::write(&f, "{ nope").unwrap();
        assert!(read_store_json(&f).expect("corrupt must not delegate").is_none());
        // …and so does a lone-surrogate escape.
        std::fs::write(&f, r#"{"a":"\uD83D"}"#).unwrap();
        assert!(read_store_json(&f).expect("lone surrogate must not delegate").is_none());

        // archivedSummary: readJson(file, {}) — corrupt yields the same {}.
        std::fs::create_dir_all(cells_dir(root).join(ARCHIVE_DIR_NAME)).unwrap();
        std::fs::write(archive_summary_file(root), "not json at all {").unwrap();
        assert!(archived_summary(root).expect("corrupt summary must not delegate").is_empty());

        // worktree-holds readStore: corrupt falls into the `{holds: []}` shape
        // fallback, so claim-next still runs with no cross-worktree holds.
        std::fs::create_dir_all(root.join(".bee").join("runtime")).unwrap();
        std::fs::write(holds_ledger_path(root), "{\"holds\": [").unwrap();
        assert_eq!(
            read_holds_store(root).expect("corrupt ledger must not delegate"),
            json!({ "holds": [] })
        );
        // A null hold ENTRY is JS-exotic, not a parse failure — still delegates.
        std::fs::write(holds_ledger_path(root), "{\"holds\": [null]}").unwrap();
        assert!(matches!(read_holds_store(root), Err(Fail::Delegate)));
    }

    #[test]
    fn corrupt_lane_record_throws_instead_of_delegating() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(lanes_dir(root)).unwrap();
        let file = lanes_dir(root).join("f.json");

        // readLaneStrict's own deterministic corrupt refusal — reached now by
        // BOTH plain garbage and a lone-surrogate escape.
        std::fs::write(&file, "{ nope").unwrap();
        let feature = json!("f");
        match lane_record_gates(root, Some(&feature)) {
            Err(Fail::Thrown(msg)) => {
                assert!(msg.starts_with("readLaneStrict: lane record "), "{msg}");
                assert!(msg.contains("exists but is corrupt"), "{msg}");
            }
            other => panic!("expected a thrown corrupt refusal, got {other:?}"),
        }
        std::fs::write(&file, r#"{"feature":"f","x":"\ud800"}"#).unwrap();
        match lane_record_gates(root, Some(&feature)) {
            Err(Fail::Thrown(msg)) => assert!(msg.contains("exists but is corrupt"), "{msg}"),
            other => panic!("lone surrogate must refuse, not delegate: {other:?}"),
        }
        // readLane (fail-open display read) takes readJson's null fallback.
        assert_eq!(read_lane_route(root, "f").expect("must not delegate"), None);
    }

    #[test]
    fn parse_json_js_treats_lone_surrogates_as_not_json() {
        assert!(matches!(parse_json_js(r#"{"a":1}"#, false), JsParse::Value(_)));
        assert!(matches!(parse_json_js(r#"{"a":"\ud800"}"#, false), JsParse::NotJson));
        assert!(matches!(parse_json_js("nope", false), JsParse::NotJson));
        // |n| >= 1e21 round-trips now — no delegation, no loss.
        match parse_json_js("[1e21,1e-7]", false) {
            JsParse::Value(v) => assert_eq!(jsjson::stringify(&v), "[1e+21,1e-7]"),
            _ => panic!("large/small magnitudes must parse"),
        }
    }

    #[test]
    fn deviations_file_lone_surrogate_takes_the_free_prose_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("dev.json");
        // Node's `catch`: not JSON -> one deviation per non-blank line.
        std::fs::write(&file, "[\"\\ud800\"]").unwrap();
        let out = parse_deviations_file(file.to_str().unwrap()).expect("must not delegate");
        assert_eq!(out, vec![json!("[\"\\ud800\"]")]);
    }

    // ── trace helpers ─────────────────────────────────────────────────────
    #[test]
    fn trace_merge_release_and_attempt_append() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // merge: object overlays defaults in place, exotics delegate.
        let merged = merge_trace(Some(&json!({"worker": "w9", "extra": 1}))).unwrap();
        assert_eq!(merged.get("worker"), Some(&json!("w9")));
        assert_eq!(merged.keys().last().unwrap(), "extra");
        assert!(matches!(merge_trace(Some(&json!("abc"))), Err(Fail::Delegate)));
        assert!(matches!(merge_trace(Some(&json!([1]))), Err(Fail::Delegate)));
        assert!(merge_trace(Some(&json!(5))).is_ok()); // {...5} === {}
        // releaseTrace clears claim + verify evidence, appends absent keys.
        let released = release_trace(merged);
        assert_eq!(released.get("worker"), Some(&Value::Null));
        assert_eq!(released.get("verify_passed"), Some(&Value::Null));
        assert!(released.contains_key("verify_command"));
        assert!(released.contains_key("verified_at"));
        // appendAttempt reads the LIVE claim for its session identity.
        std::fs::create_dir_all(claims_dir(root)).unwrap();
        match claim_cell_file(root, Some("live-sess"), "t-9", None).unwrap() {
            ClaimFileOutcome::Ok { .. } => {}
            _ => panic!(),
        }
        let trace = append_attempt(root, "t-9", default_trace(), "blocked", Some("cafe00".into()), Some("why"))
            .unwrap();
        let attempts = trace.get("attempts").unwrap().as_array().unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0]["n"], json!(1.0));
        assert_eq!(attempts[0]["claim_session"], json!("live-sess"));
        assert_eq!(attempts[0]["verdict"], json!("blocked"));
        assert_eq!(attempts[0]["failure_signature"], json!("cafe00"));
        assert_eq!(attempts[0]["note"], json!("why"));
        let keys: Vec<&String> = attempts[0].as_object().unwrap().keys().collect();
        assert_eq!(
            keys,
            vec!["n", "at", "claim_session", "claimed_at", "acquired_at", "worker", "verdict", "failure_signature", "note"]
        );
    }

    // ── update validators ─────────────────────────────────────────────────
    #[test]
    fn update_field_validators_and_frozen_hints() {
        assert_eq!(update_field_problem("title", &json!("ok")), None);
        assert_eq!(
            update_field_problem("title", &json!("  ")),
            Some("must be a non-empty string".to_string())
        );
        assert_eq!(
            update_field_problem("deps", &json!(["a", 1])),
            Some("must be an array of strings".to_string())
        );
        assert_eq!(
            update_field_problem("lane", &json!("mega")),
            Some("must be one of: tiny, small, standard, high-risk, spike".to_string())
        );
        assert_eq!(update_field_problem("change_class", &Value::Null), None);
        assert_eq!(update_field_problem("behavior_change", &json!(true)), None);
        assert_eq!(
            update_frozen_hint("tier"),
            Some("use the tier verb (bee cells tier --id ID --tier T)")
        );
        assert_eq!(update_frozen_hint("status"), Some("status moves only through claim/verify/cap/block/drop"));
        assert_eq!(update_frozen_hint("nonsense"), None);
    }

    // ── cells claim-next (R6): the sweep + the selection filters ──────────

    fn cn_root() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        tmp
    }

    fn write_claim_fixture(root: &Path, id: &str, session: Option<&str>, ttl: f64, at: &str) {
        let dir = claims_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let mut claim = Map::new();
        claim.insert("cell".into(), json!(id));
        if let Some(s) = session {
            claim.insert("session".into(), json!(s));
        }
        claim.insert("ttl_seconds".into(), json!(ttl));
        claim.insert("claimed_at".into(), json!(at));
        claim.insert("acquired_at".into(), json!(at));
        std::fs::write(
            dir.join(format!("{id}.json")),
            jsjson::stringify_pretty(&Value::Object(claim)),
        )
        .unwrap();
    }

    fn write_session_fixture(root: &Path, id: &str, heartbeat: &str, lane: Option<&str>) {
        let dir = sessions_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let mut rec = Map::new();
        rec.insert("id".into(), json!(id));
        rec.insert("started_at".into(), json!(heartbeat));
        rec.insert("last_heartbeat".into(), json!(heartbeat));
        rec.insert("lane".into(), lane.map(|l| json!(l)).unwrap_or(Value::Null));
        std::fs::write(
            dir.join(format!("{id}.json")),
            jsjson::stringify_pretty(&Value::Object(rec)),
        )
        .unwrap();
    }

    const OLD: &str = "2020-01-01T00:00:00.000Z";

    #[test]
    fn sweep_resets_only_the_claim_it_actually_removed() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        let fresh = rsv::iso_from_ms(now).ok().unwrap();

        // (a) expired claim, dead owner, cell still claimed BY THAT SESSION.
        write_cell_fixture(
            root,
            "a1",
            &json!({"id":"a1","status":"claimed","feature":"f","trace":{"worker":"w","claim_session":"dead"}}),
        );
        write_claim_fixture(root, "a1", Some("dead"), 60.0, OLD);
        write_session_fixture(root, "dead", OLD, None);
        // (b) expired claim, but the cell was RE-claimed by another session.
        write_cell_fixture(
            root,
            "b1",
            &json!({"id":"b1","status":"claimed","feature":"f","trace":{"worker":"w2","claim_session":"someone-else"}}),
        );
        write_claim_fixture(root, "b1", Some("dead"), 60.0, OLD);
        // (c) expired claim whose owner is LIVE — never swept.
        write_cell_fixture(
            root,
            "c1",
            &json!({"id":"c1","status":"claimed","feature":"f","trace":{"worker":"w3","claim_session":"live"}}),
        );
        write_claim_fixture(root, "c1", Some("live"), 60.0, OLD);
        write_session_fixture(root, "live", &fresh, None);
        // (d) an UNEXPIRED claim — never swept.
        write_cell_fixture(
            root,
            "d1",
            &json!({"id":"d1","status":"claimed","feature":"f","trace":{"worker":"w4","claim_session":"dead"}}),
        );
        write_claim_fixture(root, "d1", Some("dead"), 3600.0, &fresh);

        sweep_expired_claims(root, now).ok().unwrap();

        let gone = |id: &str| !claims_dir(root).join(format!("{id}.json")).exists();
        assert!(gone("a1"), "expired + stale owner is swept");
        assert!(gone("b1"), "the claim file goes even when the reset is skipped");
        assert!(!gone("c1"), "a live owner is never swept");
        assert!(!gone("d1"), "an unexpired claim is never swept");

        let status = |id: &str| match read_cell_norm(root, id).ok().unwrap() {
            Some(Value::Object(m)) => js_string_or_undefined(m.get("status")),
            _ => panic!("cell {id}"),
        };
        assert_eq!(status("a1"), "open", "claimed -> open reset");
        assert_eq!(status("b1"), "claimed", "claim_session mismatch: never overwritten");
        assert_eq!(status("c1"), "claimed");
        assert_eq!(status("d1"), "claimed");

        // The reset's trace carries the sweep stamps and clears the claim.
        let a1 = read_cell_norm(root, "a1").ok().unwrap().unwrap();
        let trace = a1.get("trace").unwrap();
        assert_eq!(trace.get("worker"), Some(&Value::Null));
        assert_eq!(trace.get("claimed_at"), Some(&Value::Null));
        assert_eq!(trace.get("claim_session"), Some(&Value::Null));
        assert_eq!(trace.get("swept_from_session"), Some(&json!("dead")));
        assert_eq!(
            trace.get("swept_at"),
            Some(&json!(rsv::iso_from_ms(now).ok().unwrap()))
        );

        // Exactly ONE decision row — b1's skipped reset logs nothing.
        let rows = std::fs::read_to_string(decisions_path(root)).unwrap();
        let lines: Vec<&str> = rows.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("sweep: cell \\\"a1\\\" reset claimed -> open"));
        assert!(lines[0].contains("swept session \\\"dead\\\""));

        // Idempotent: a second pass has nothing left to trigger on.
        sweep_expired_claims(root, now).ok().unwrap();
        let rows2 = std::fs::read_to_string(decisions_path(root)).unwrap();
        assert_eq!(rows2.lines().filter(|l| !l.trim().is_empty()).count(), 1);
    }

    #[test]
    fn sweep_of_a_sessionless_claim_names_none_in_its_decision_row() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        write_cell_fixture(
            root,
            "s1",
            &json!({"id":"s1","status":"claimed","feature":"f","trace":{"worker":"w"}}),
        );
        write_claim_fixture(root, "s1", None, 60.0, OLD);
        sweep_expired_claims(root, now).ok().unwrap();
        let s1 = read_cell_norm(root, "s1").ok().unwrap().unwrap();
        assert_eq!(s1.get("status"), Some(&json!("open")));
        assert_eq!(
            s1.get("trace").and_then(|t| t.get("swept_from_session")),
            Some(&Value::Null)
        );
        let rows = std::fs::read_to_string(decisions_path(root)).unwrap();
        assert!(rows.contains("swept session \\\"none (sessionless)\\\""));
    }

    #[test]
    fn resolve_pipeline_refuses_a_bound_but_broken_lane() {
        let tmp = cn_root();
        let root = tmp.path();
        let fresh = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();

        // No session record at all → the default pipeline.
        match resolve_pipeline(root, root, "nobody").ok().unwrap() {
            Pipeline::Ok { feature, execution_approved } => {
                assert!(feature.is_none() && !execution_approved);
            }
            Pipeline::Refused { .. } => panic!("default expected"),
        }

        // Bound to a lane with no record → LANE_MISSING.
        write_session_fixture(root, "s1", &fresh, Some("nope"));
        match resolve_pipeline(root, root, "s1").ok().unwrap() {
            Pipeline::Refused { code, reason } => {
                assert_eq!(code, "LANE_MISSING");
                assert!(reason.contains("session \"s1\" is bound to lane \"nope\" but"));
                assert!(reason.contains("does not exist"));
            }
            Pipeline::Ok { .. } => panic!("LANE_MISSING expected"),
        }

        // Bound to an invalid lane NAME → LANE_INVALID (lanePath's throw).
        write_session_fixture(root, "s2", &fresh, Some("a/b"));
        match resolve_pipeline(root, root, "s2").ok().unwrap() {
            Pipeline::Refused { code, reason } => {
                assert_eq!(code, "LANE_INVALID");
                assert!(reason.contains("lane feature must be a plain id (no path separators)"));
            }
            Pipeline::Ok { .. } => panic!("LANE_INVALID expected"),
        }

        // Bound to a lane file that is not a record for THAT feature → LANE_CORRUPT.
        let lanes = root.join(".bee").join("lanes");
        std::fs::create_dir_all(&lanes).unwrap();
        std::fs::write(lanes.join("broken.json"), r#"{"feature":"other"}"#).unwrap();
        write_session_fixture(root, "s3", &fresh, Some("broken"));
        match resolve_pipeline(root, root, "s3").ok().unwrap() {
            Pipeline::Refused { code, .. } => assert_eq!(code, "LANE_CORRUPT"),
            Pipeline::Ok { .. } => panic!("LANE_CORRUPT expected"),
        }

        // A healthy bound lane resolves to ITS OWN feature and gate.
        std::fs::write(
            lanes.join("good.json"),
            r#"{"feature":"good","approved_gates":{"execution":true}}"#,
        )
        .unwrap();
        write_session_fixture(root, "s4", &fresh, Some("good"));
        match resolve_pipeline(root, root, "s4").ok().unwrap() {
            Pipeline::Ok { feature, execution_approved } => {
                assert_eq!(feature.as_deref(), Some("good"));
                assert!(execution_approved);
            }
            Pipeline::Refused { .. } => panic!("lane expected"),
        }
    }

    #[test]
    fn candidate_filters_skip_foreign_session_holds_and_foreign_worktree_holds() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        let cell = json!({"id":"x1","status":"open","feature":"f","files":["src/a.ts"]});
        // No holds anywhere → claimable.
        assert!(candidate_ok(root, root, "mine", &cell, now).ok().unwrap());

        // A cross-worktree hold owned by a DIFFERENT checkout blocks it.
        let runtime = root.join(".bee").join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        let stamp = rsv::iso_from_ms(now).ok().unwrap();
        std::fs::write(
            runtime.join("cross-worktree-holds.json"),
            format!(
                r#"{{"holds":[{{"holder":"wt-other","path":"src/a.ts","mirrored_at":"{stamp}","ttl_seconds":3600,"released_at":null}}]}}"#
            ),
        )
        .unwrap();
        assert!(!candidate_ok(root, root, "mine", &cell, now).ok().unwrap());
        // Our OWN holder never blocks us.
        std::fs::write(
            runtime.join("cross-worktree-holds.json"),
            format!(
                r#"{{"holds":[{{"holder":"main","path":"src/a.ts","mirrored_at":"{stamp}","ttl_seconds":3600,"released_at":null}}]}}"#
            ),
        )
        .unwrap();
        assert!(candidate_ok(root, root, "mine", &cell, now).ok().unwrap());
        // A RELEASED hold never blocks.
        std::fs::write(
            runtime.join("cross-worktree-holds.json"),
            format!(
                r#"{{"holds":[{{"holder":"wt-other","path":"src/a.ts","mirrored_at":"{stamp}","ttl_seconds":3600,"released_at":"{stamp}"}}]}}"#
            ),
        )
        .unwrap();
        assert!(candidate_ok(root, root, "mine", &cell, now).ok().unwrap());
        // A cell with NO declared files skips both hold checks entirely.
        let bare = json!({"id":"x2","status":"open","feature":"f"});
        assert!(candidate_ok(root, root, "mine", &bare, now).ok().unwrap());
    }

    // ── claims (R5): the sweep's gate discipline ──────────────────────────
    // Oracle: test_claims.mjs "sweep: TTL expired AND heartbeat stale IS
    // reclaimed; no gate file leaks", "sweep: TTL expired but heartbeat FRESH
    // is never reclaimed (20260710 — no steal on a stall signal)", and the
    // sweep half of "sweep and adopt skip/refuse while the per-claim gate is
    // held — typed GATE_HELD, never wait". (The adopt half of that row, and
    // the whole msn-12 fencing surface, are covered in § adoption + fencing
    // at the end of this module.)
    #[test]
    fn sweep_skips_a_gated_claim_and_leaks_no_gate_file() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        let fresh = rsv::iso_from_ms(now).ok().unwrap();
        let claimed = |session: &str| {
            json!({"id":"x","status":"claimed","feature":"f","trace":{"worker":"w","claim_session":session}})
        };

        // (a) expired + stale owner, but another process holds the per-claim
        //     gate: skipped on the spot (GATE_HELD), never waited out.
        write_cell_fixture(root, "held", &claimed("dead"));
        write_claim_fixture(root, "held", Some("dead"), 60.0, OLD);
        let held_gate = claim_gate_path(root, "held").unwrap();
        std::fs::write(&held_gate, "{}").unwrap(); // another process mid-adopt

        // (b) the SAME shape with a free gate — the control that IS reclaimed.
        write_cell_fixture(root, "free", &claimed("dead"));
        write_claim_fixture(root, "free", Some("dead"), 60.0, OLD);

        // (c) expired TTL but a FRESH owner heartbeat: never reclaimed, and
        //     the gate is never even taken (the heartbeat test precedes
        //     acquireGate).
        write_cell_fixture(root, "alive", &claimed("live"));
        write_claim_fixture(root, "alive", Some("live"), 60.0, OLD);

        write_session_fixture(root, "dead", OLD, None);
        write_session_fixture(root, "live", &fresh, None);

        sweep_expired_claims(root, now).ok().unwrap();

        let claim_file = |id: &str| claims_dir(root).join(format!("{id}.json"));
        let status = |id: &str| match read_cell_norm(root, id).ok().unwrap() {
            Some(Value::Object(m)) => js_string_or_undefined(m.get("status")),
            _ => panic!("cell {id}"),
        };

        assert!(claim_file("held").exists(), "a gated claim is skipped, never stolen");
        assert_eq!(status("held"), "claimed", "a skipped claim's cell is never reset");
        assert_eq!(
            std::fs::read_to_string(&held_gate).unwrap(),
            "{}",
            "the other process's gate file is left exactly as found"
        );

        assert!(!claim_file("free").exists(), "expired + stale owner IS reclaimed");
        assert_eq!(status("free"), "open");
        assert!(
            !claim_gate_path(root, "free").unwrap().exists(),
            "a completed sweep leaves no gate file behind"
        );

        assert!(claim_file("alive").exists(), "a fresh heartbeat is never swept");
        assert_eq!(status("alive"), "claimed");
        assert!(
            !claim_gate_path(root, "alive").unwrap().exists(),
            "the heartbeat check runs before the gate is ever taken"
        );
    }

    // ── claims (R5): releaseClaim's owner ladder ──────────────────────────
    // Oracle: test_claims.mjs "releaseClaim: NOT_OWNER for the old session
    // after adoption, owner release removes the file, NOT_FOUND after". This
    // port returns () — the typed codes are the half the unwind caller
    // ignores — so the ladder is asserted through its disk effect.
    #[test]
    fn release_claim_owner_ladder_and_gate_hygiene() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        match claim_cell_file(control, Some("owner-a"), "r-1", None).unwrap() {
            ClaimFileOutcome::Ok { .. } => {}
            _ => panic!("precondition: r-1 claimed by owner-a"),
        }
        let file = claims_dir(control).join("r-1.json");
        let gate = claim_gate_path(control, "r-1").unwrap();
        let parse = |p: &Path| -> Value {
            serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
        };
        let before = parse(&file);

        // NOT_OWNER: a foreign session's release changes nothing.
        release_claim(control, Some("owner-b"), "r-1").unwrap();
        assert!(file.exists(), "a denied release never removes the claim");
        assert_eq!(parse(&file), before, "a denied release never rewrites the claim");
        assert!(!gate.exists(), "a denied release leaks no gate file");

        // A sessionless release is not the owner here either.
        release_claim(control, None, "r-1").unwrap();
        assert!(file.exists(), "sessionless != owner \"owner-a\"");
        assert_eq!(parse(&file), before);

        // The owner's release removes it, gate-clean.
        release_claim(control, Some("owner-a"), "r-1").unwrap();
        assert!(!file.exists(), "the owner's release removes the claim file");
        assert!(!gate.exists(), "the owner's release leaks no gate file");

        // NOT_FOUND: releasing again is a no-op that never takes the gate.
        release_claim(control, Some("owner-a"), "r-1").unwrap();
        assert!(!file.exists());
        assert!(!gate.exists());
    }

    // ── claims D5 (R5): the Codex session bridge ──────────────────────────
    // Oracle: test_claims.mjs "claimCellFile (hardening-1-7-10 D5 — Codex
    // session bridge): a sessionless claim with EXACTLY ONE fresh live session
    // auto-adopts that session's identity", and its twin "…with TWO OR MORE
    // fresh live sessions still refuses typed SESSION_REQUIRED".
    #[test]
    fn sessionless_claim_adopts_one_live_session_and_refuses_two() {
        let tmp = cn_root();
        let root = tmp.path();
        let fresh = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();
        let claim_ok = |id: &str, session: Option<&str>| -> Value {
            match claim_cell_file(root, session, id, Some(60.0)).unwrap() {
                ClaimFileOutcome::Ok { claim } => claim,
                ClaimFileOutcome::Refused { code, reason } => {
                    panic!("{id}: expected a claim, got {code}: {reason}")
                }
            }
        };

        // Zero live sessions: a genuinely solo claim stays sessionless and is
        // never marked adopted.
        let solo = claim_ok("d5-solo", None);
        assert!(solo.get("session").is_none(), "a solo sessionless claim omits the session key");
        assert!(solo.get("adopted").is_none(), "nothing was adopted");

        // Exactly one fresh session: its identity is adopted rather than refused.
        write_session_fixture(root, "only-live", &fresh, None);
        let adopted = claim_ok("d5-one", None);
        assert_eq!(adopted.get("session"), Some(&json!("only-live")));
        assert_eq!(adopted.get("adopted"), Some(&json!(true)));
        let on_disk = read_claim(root, "d5-one").unwrap().unwrap();
        assert_eq!(on_disk.get("session"), Some(&json!("only-live")));
        assert_eq!(
            on_disk.get("adopted"),
            Some(&json!(true)),
            "the on-disk record carries the audit marker too"
        );

        // A STALE second session is not ambiguity — one FRESH is still one.
        write_session_fixture(root, "long-gone", OLD, None);
        assert_eq!(claim_ok("d5-still-one", None).get("session"), Some(&json!("only-live")));

        // Two fresh sessions: real ambiguity is refused, never guessed.
        write_session_fixture(root, "second-live", &fresh, None);
        match claim_cell_file(root, None, "d5-two", Some(60.0)).unwrap() {
            ClaimFileOutcome::Refused { code, reason } => {
                assert_eq!(code, "SESSION_REQUIRED");
                // Both identity routes are a pinned part of this refusal —
                // the message is what tells a stuck agent how to proceed.
                assert!(reason.contains("--session-id"), "reason: {reason}");
                assert!(reason.contains("BEE_SESSION_ID"), "reason: {reason}");
            }
            ClaimFileOutcome::Ok { claim } => panic!("two live sessions must refuse, got {claim}"),
        }
        assert!(
            !claims_dir(root).join("d5-two.json").exists(),
            "the refusal leaves no claim file behind"
        );

        // Control: an explicit session id still claims fine, and is never
        // marked adopted (adoption is only ever an inference).
        let explicit = claim_ok("d5-two", Some("second-live"));
        assert_eq!(explicit.get("session"), Some(&json!("second-live")));
        assert!(explicit.get("adopted").is_none());
    }

    // ── claims (R5): resolveSessionId's ordered chain ─────────────────────
    // Oracle: test_claims.mjs "resolveSessionId: explicit flag wins over env;
    // a blank flag falls through to env" + "(hardening-4a): BEE_SESSION_ID
    // wins over legacy CLAUDE_CODE_SESSION_ID".
    const SESSION_CHAIN_CHILD: &str = "verbs::cells::tests::session_id_env_chain_child";

    /// Runs ONLY as a child of the test below, which hands it a controlled
    /// environment. `#[ignore]` keeps it out of the normal pass: this
    /// process's env is shared with every other test in the binary (
    /// state_group's fixtures resolve BEE_SESSION_ID / CLAUDE_CODE_SESSION_ID
    /// live, and the CI runner really does export the latter), so the ordered
    /// chain is exercised out-of-process instead of by mutating env under them.
    #[test]
    #[ignore = "spawned by resolve_session_id_precedence_flag_beats_bee_beats_legacy"]
    fn session_id_env_chain_child() {
        let want = std::env::var("BEE_TEST_EXPECT").unwrap_or_default();
        assert_eq!(
            resolve_session_flag_env(None).unwrap_or_default(),
            want,
            "no-flag resolution"
        );
        // An explicit flag outranks whatever the env says, in every combination.
        assert_eq!(
            resolve_session_flag_env(Some("sess-from-flag")).as_deref(),
            Some("sess-from-flag")
        );
        // A blank / whitespace-only flag is NOT an explicit empty session: it
        // falls through to the same answer the env alone gives.
        assert_eq!(resolve_session_flag_env(Some("")).unwrap_or_default(), want);
        assert_eq!(resolve_session_flag_env(Some("   ")).unwrap_or_default(), want);
    }

    #[test]
    fn resolve_session_id_precedence_flag_beats_bee_beats_legacy() {
        // The env-free half runs in-process: whatever this machine's ambient
        // env says, an explicit flag wins and a blank one falls through to it.
        assert_eq!(
            resolve_session_flag_env(Some("sess-from-flag")).as_deref(),
            Some("sess-from-flag")
        );
        let ambient = resolve_session_flag_env(None);
        assert_eq!(resolve_session_flag_env(Some("")), ambient, "blank flag falls through");
        assert_eq!(resolve_session_flag_env(Some("   ")), ambient);

        // The ordered env half needs a controlled environment — child process.
        // (bee, legacy, expected)
        let cases: &[(Option<&str>, Option<&str>, &str)] = &[
            (None, Some("sess-legacy"), "sess-legacy"),
            (Some("sess-bee"), Some("sess-legacy"), "sess-bee"),
            (Some("   "), Some("sess-legacy"), "sess-legacy"),
            (Some("sess-bee"), None, "sess-bee"),
            (None, None, ""),
        ];
        let exe = std::env::current_exe().expect("test binary path");
        for (bee, legacy, want) in cases {
            let mut cmd = std::process::Command::new(&exe);
            cmd.args(["--exact", SESSION_CHAIN_CHILD, "--ignored", "--test-threads", "1"]);
            cmd.env_remove("BEE_SESSION_ID").env_remove("CLAUDE_CODE_SESSION_ID");
            if let Some(v) = bee {
                cmd.env("BEE_SESSION_ID", v);
            }
            if let Some(v) = legacy {
                cmd.env("CLAUDE_CODE_SESSION_ID", v);
            }
            cmd.env("BEE_TEST_EXPECT", want);
            let out = cmd.output().expect("spawn the test binary");
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            // Tripwire: a filter that matched nothing ALSO exits 0, so the
            // pass count — not the status — is what proves the case ran.
            assert!(
                text.contains("1 passed"),
                "child never ran the case (bee={bee:?} legacy={legacy:?}):\n{text}"
            );
            assert!(
                out.status.success(),
                "chain case failed (bee={bee:?} legacy={legacy:?} want={want:?}):\n{text}"
            );
        }
    }

    // ── cells add (R5): whole-batch report ────────────────────────────────
    // Oracle: test_cells.mjs "addCells aggregates EVERY failing cell in one
    // refusal", "addCells refuses a duplicate id within the batch",
    // "addCells refuses an in-batch cycle", "previewAddCells: a clean batch
    // reports ok:true …and writes nothing", "previewAddCells: a dirty batch
    // names EVERY failing cell", "previewAddCells folds a batch-wide cycle
    // into the cells it touches (ce-2)". buildAddCellsReport is the one
    // engine `cells add` and `cells add --dry-run` share.
    fn addable(id: &str) -> Value {
        json!({
            "id": id, "feature": "batch", "title": format!("title {id}"),
            "action": "do the thing", "verify": "echo ok", "lane": "tiny",
        })
    }

    #[test]
    fn add_cells_report_aggregates_every_failure_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let unwritten = |ids: &[&str]| {
            for id in ids {
                assert!(
                    read_cell_norm(root, id).ok().unwrap().is_none(),
                    "{id} must not exist — the report never writes"
                );
            }
        };

        // A clean batch: every verdict ok, normalized cells handed back, and
        // still nothing on disk (the --dry-run "nothing written" contract).
        let clean = vec![addable("b-1"), addable("b-2")];
        let (ok, rows, normalized) = build_add_cells_report(root, &clean).unwrap();
        assert!(ok);
        assert_eq!(rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), vec!["b-1", "b-2"]);
        assert!(rows.iter().all(|r| r.ok && r.problems.is_empty()));
        assert_eq!(normalized.as_ref().map(Vec::len), Some(2));
        unwritten(&["b-1", "b-2"]);

        // A dirty batch does NOT stop at the first bad cell: both bad cells
        // carry their own problem, the good one still verdicts ok, and no
        // normalized list comes back — that absence is the all-or-nothing
        // mechanism the writer loop depends on.
        let mut bad_lane = addable("d-2");
        bad_lane["lane"] = json!("huge");
        let mut blank_title = addable("d-3");
        blank_title["title"] = json!("");
        let dirty = vec![addable("d-1"), bad_lane, blank_title];
        let (ok, rows, normalized) = build_add_cells_report(root, &dirty).unwrap();
        assert!(!ok);
        assert!(rows[0].ok && rows[0].problems.is_empty(), "the valid cell still verdicts ok");
        assert!(!rows[1].ok);
        assert!(rows[1].problems.iter().any(|p| p.contains("lane")), "{:?}", rows[1].problems);
        assert!(!rows[2].ok);
        assert!(
            rows[2].problems.iter().any(|p| p.contains("\"title\"")),
            "the SECOND bad cell is named too, never swallowed by the first: {:?}",
            rows[2].problems
        );
        assert!(normalized.is_none(), "a dirty batch yields nothing to write");
        unwritten(&["d-1", "d-2", "d-3"]);

        // The per-cell verdict shape the dry-run payload renders.
        let payload = add_report_rows_value(&rows);
        let cells = payload.as_array().unwrap();
        assert_eq!(cells.len(), 3);
        assert_eq!(
            cells[0].as_object().unwrap().keys().collect::<Vec<_>>(),
            vec!["id", "ok", "problems"]
        );
        assert_eq!(cells[0]["id"], json!("d-1"));
        assert_eq!(cells[0]["ok"], json!(true));
        assert_eq!(cells[0]["problems"], json!([]));
        assert_eq!(cells[1]["ok"], json!(false));

        // An in-batch duplicate id: the first occurrence is clean, the repeat
        // carries the duplicate problem.
        let (ok, rows, normalized) = build_add_cells_report(root, &[addable("dup"), addable("dup")]).unwrap();
        assert!(!ok);
        assert!(rows[0].ok, "the first occurrence of the id is not the duplicate");
        assert_eq!(rows[1].problems, vec!["addCells: duplicate id \"dup\" within the batch."]);
        assert!(normalized.is_none());
        unwritten(&["dup"]);

        // A cell with no usable id is still reported, under its index.
        let mut anonymous = addable("x");
        anonymous.as_object_mut().unwrap().shift_remove("id");
        let (ok, rows, _) = build_add_cells_report(root, &[addable("keep"), anonymous]).unwrap();
        assert!(!ok);
        assert_eq!(rows[1].id, "(index 1)");
        assert!(!rows[1].ok);
    }

    #[test]
    fn add_cells_report_folds_a_batch_wide_cycle_onto_every_cell_it_touches() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Control first: the same two cells WITHOUT the back-edge are clean,
        // so the refusal below is the cycle and nothing else.
        let mut a = addable("cyc-a");
        a["deps"] = json!(["cyc-b"]);
        let b = addable("cyc-b");
        let (ok, _, normalized) = build_add_cells_report(root, &[a.clone(), b]).unwrap();
        assert!(ok, "a plain in-batch dependency is legal");
        assert_eq!(normalized.as_ref().map(Vec::len), Some(2));

        let mut b = addable("cyc-b");
        b["deps"] = json!(["cyc-a"]);
        let (ok, rows, normalized) = build_add_cells_report(root, &[a, b]).unwrap();
        assert!(!ok, "a <-> b is a cycle");
        for row in &rows {
            assert!(!row.ok, "{} must fail", row.id);
            assert!(
                row.problems.iter().any(|p| p.contains("dependency cycle refused")),
                "the cycle folds onto {} too: {:?}",
                row.id,
                row.problems
            );
        }
        assert!(normalized.is_none());
        assert!(read_cell_norm(root, "cyc-a").ok().unwrap().is_none());
        assert!(read_cell_norm(root, "cyc-b").ok().unwrap().is_none());
    }

    // ── verify:"none" (R5): the no-test-repo sentinel ─────────────────────
    // Oracle: lib/cells.mjs assertVerifySentinelAllowed (decision 55b951e1) —
    // the sentinel is accepted only where the repo has declared itself.
    fn write_bee_config(root: &Path, config: &Value) {
        let dir = root.join(".bee");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.json"), jsjson::stringify_pretty(config)).unwrap();
    }

    #[test]
    fn verify_none_is_accepted_only_in_a_declared_no_test_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut sentinel = addable("v-1");
        sentinel["verify"] = json!("none");

        // Undeclared repo: refused on add AND on update; a real verify passes
        // either way (the control that nothing else in the cell is at fault).
        assert!(
            thrown(validate_new_cell(root, &sentinel)).starts_with("addCell: verify \"none\" is refused"),
        );
        assert!(validate_new_cell(root, &addable("v-1")).is_ok());
        assert!(
            thrown(assert_verify_sentinel_allowed(root, "updateCell", &json!("none")))
                .starts_with("updateCell: verify \"none\" is refused"),
        );
        assert!(assert_verify_sentinel_allowed(root, "updateCell", &json!("npm test")).is_ok());

        // Declared no-test repo: the same sentinel is accepted on both doors.
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        assert!(validate_new_cell(root, &sentinel).is_ok(), "a declared no-test repo accepts it");
        assert!(assert_verify_sentinel_allowed(root, "updateCell", &json!("none")).is_ok());

        // isNoTestRepo's own matrix, read back through the real config reader.
        let declares = |config: Value| -> bool {
            write_bee_config(root, &config);
            is_no_test_repo(&read_commands_slice(root).unwrap())
        };
        assert!(declares(json!({"commands": {"verify": "none"}})));
        assert!(declares(json!({"commands": {"test": ["none"]}})));
        assert!(
            !declares(json!({"commands": {"test": ["none", "npm test"]}})),
            "a list with a real command beside the sentinel is NOT a no-test repo"
        );
        assert!(!declares(json!({"commands": {"test": "npm test"}})));
        assert!(!declares(json!({"commands": {}})));
    }

    #[test]
    fn capping_in_a_no_test_repo_runs_no_tests_but_a_declared_red_still_refuses() {
        let cap_flags = |id: &str| CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: Vec::new(),
            deviations: Vec::new(),
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
        };
        let cell_body = |id: &str| {
            json!({
                "id": id, "feature": "f", "title": "t", "action": "a",
                "verify": "none", "lane": "tiny", "status": "claimed",
                "deps": [], "files": [], "trace": {},
            })
        };

        // A repo that declares itself no-test: the sentinel is filtered out of
        // commands.test, the test door never opens, and the cap lands.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "nt-1", &cell_body("nt-1"));
        let capped = cap_cell_from_flags(root, &cap_flags("nt-1"), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
        assert_eq!(
            capped["trace"]["tests"],
            json!("undeclared"),
            "\"none\" is not a command to run"
        );
        assert!(!test_results_path(root).exists(), "nothing ran, so nothing was recorded");

        // Control: the same cell shape in a repo declaring a real, RED command
        // refuses the cap — proving the door above was genuinely closed by the
        // sentinel rather than by an absent test runner.
        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        write_bee_config(root2, &json!({"commands": {"test": "exit 3"}}));
        write_cell_fixture(root2, "nt-2", &cell_body("nt-2"));
        let refusal = thrown(cap_cell_from_flags(root2, &cap_flags("nt-2"), false));
        assert!(
            refusal.starts_with("refusing to cap \"nt-2\" — the declared test run is RED"),
            "{refusal}"
        );
        let after = read_cell_norm(root2, "nt-2").ok().unwrap().unwrap();
        assert_eq!(after.get("status"), Some(&json!("claimed")), "a red run never caps");
        assert!(test_results_path(root2).exists(), "the red run IS recorded");
    }

    // ══ adoption + fencing (claims.mjs, msn-12 D4/D9 invariant 10) ═════════
    //
    // Ported from test_claims.mjs. Before this cell nothing in the Rust tree
    // consumed `fence_epoch`: claim_cell_file stamped it and no code path ever
    // compared it, so a stale holder's renew/release would have proceeded
    // silently. Each negative below CONSTRUCTS the stale state and pins the
    // exact refusal bytes, with a firing control beside it.

    fn adopt(root: &Path, cell: &str, session: &str) -> AdoptClaimOutcome {
        adopt_claim(root, cell, session).expect("adopt must not throw")
    }

    fn refused(outcome: AdoptClaimOutcome) -> ClaimRefusal {
        match outcome {
            AdoptClaimOutcome::Refused(r) => r,
            AdoptClaimOutcome::Ok { .. } => panic!("expected a typed refusal"),
        }
    }

    fn claim_on_disk(root: &Path, cell: &str) -> Value {
        let raw = std::fs::read_to_string(claims_dir(root).join(format!("{cell}.json"))).unwrap();
        serde_json::from_str(&raw).unwrap()
    }

    /// Oracle: "adoptClaim rewrites the owner IN PLACE: old owner loses, new
    /// owner holds, the claim file is present throughout" and "adoptClaim
    /// bumps fence_epoch by exactly 1, atomically with the ownership rewrite".
    #[test]
    fn adopt_rewrites_ownership_in_place_and_bumps_the_fence_by_exactly_one() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "c-1", Some("sess-a"), 600.0, OLD);
        // A pre-msn-12 claim carries no fence_epoch on disk at all.
        assert!(claim_on_disk(root, "c-1").get("fence_epoch").is_none());

        let AdoptClaimOutcome::Ok { claim, previous_owner } = adopt(root, "c-1", "sess-b") else {
            panic!("expected an adoption");
        };
        assert_eq!(previous_owner, Some(json!("sess-a")));
        assert_eq!(claim["session"], json!("sess-b"));
        assert_eq!(claim["adopted_from"], json!("sess-a"));
        assert_eq!(claim["fence_epoch"], json!(2.0), "a legacy claim reads as epoch 1, so +1 == 2");
        assert_ne!(claim["claimed_at"], json!(OLD), "fresh ownership renews the TTL clock");
        assert_eq!(claim["acquired_at"], json!(OLD), "the acquisition stamp is immutable");
        assert_eq!(claim["adopted_at"], claim["claimed_at"]);
        // Compared as RENDERED bytes: JS writes 2 and 2.0 identically, so a
        // JSON number-kind difference on the read-back is not a difference.
        assert_eq!(
            jsjson::stringify(&claim_on_disk(root, "c-1")),
            jsjson::stringify(&claim),
            "written atomically, never deleted first"
        );
        // ORACLE-PINNED BYTES: captured from a live `node` run of claims.mjs
        // adoptClaim over this exact fixture, not from a reading of the source.
        let on_disk = std::fs::read_to_string(claims_dir(root).join("c-1.json"))
            .unwrap()
            .replace(claim["claimed_at"].as_str().unwrap(), "<now>");
        assert_eq!(
            on_disk,
            "{\n  \"cell\": \"c-1\",\n  \"session\": \"sess-b\",\n  \"ttl_seconds\": 600,\n  \"claimed_at\": \"<now>\",\n  \"acquired_at\": \"2020-01-01T00:00:00.000Z\",\n  \"adopted_from\": \"sess-a\",\n  \"adopted_at\": \"<now>\",\n  \"fence_epoch\": 2\n}\n"
        );
        // Key order: a re-assigned key keeps its position; the three new ones
        // append in declaration order.
        let keys: Vec<&str> =
            claim.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec!["cell", "session", "ttl_seconds", "claimed_at", "acquired_at", "adopted_from", "adopted_at", "fence_epoch"]
        );

        // A second adoption bumps again, from the STORED epoch.
        let AdoptClaimOutcome::Ok { claim, previous_owner } = adopt(root, "c-1", "sess-c") else {
            panic!("expected an adoption");
        };
        assert_eq!(previous_owner, Some(json!("sess-b")));
        assert_eq!(claim["fence_epoch"], json!(3.0));

        // Adopting a SESSIONLESS claim drops `adopted_from` entirely rather
        // than writing null (`{...claim, adopted_from: undefined}`).
        write_claim_fixture(root, "c-2", None, 600.0, OLD);
        let AdoptClaimOutcome::Ok { claim, previous_owner } = adopt(root, "c-2", "sess-b") else {
            panic!("expected an adoption");
        };
        assert_eq!(previous_owner, None);
        assert!(claim.get("adopted_from").is_none(), "undefined is dropped, never null: {claim}");
        assert!(!claim_on_disk(root, "c-2").as_object().unwrap().contains_key("adopted_from"));

        // The gate file never leaks.
        assert!(!claim_gate_path(root, "c-1").unwrap().exists());
        assert!(!claim_gate_path(root, "c-2").unwrap().exists());
    }

    /// Oracle: "adoptClaim on a cell with no claim is a typed NOT_FOUND" and
    /// "sweep and adopt skip/refuse while the per-claim gate is held — typed
    /// GATE_HELD, never wait".
    #[test]
    fn adopt_refuses_not_found_and_gate_held_without_ever_waiting() {
        let tmp = cn_root();
        let root = tmp.path();
        let ghost = refused(adopt(root, "no-such-cell", "sess-b"));
        assert_eq!(ghost.code, "NOT_FOUND");
        assert_eq!(ghost.reason, "cell \"no-such-cell\" has no claim to adopt.");

        write_claim_fixture(root, "gated", Some("sess-a"), 600.0, OLD);
        let gate = claim_gate_path(root, "gated").unwrap();
        std::fs::write(&gate, "{}").unwrap(); // another process mid-adopt
        let before = claim_on_disk(root, "gated");
        let held = refused(adopt(root, "gated", "sess-b"));
        assert_eq!(held.code, "GATE_HELD");
        assert_eq!(
            held.reason,
            "claim \"gated\" is gated by another in-flight adopt/sweep — retry later, never wait on the gate."
        );
        assert_eq!(claim_on_disk(root, "gated"), before, "a gated adopt changes nothing");
        assert!(gate.exists(), "someone else's gate is never released by the loser");

        // Control: with the gate free the very same adopt succeeds.
        std::fs::remove_file(&gate).unwrap();
        assert!(matches!(adopt(root, "gated", "sess-b"), AdoptClaimOutcome::Ok { .. }));

        // requireId still guards both arguments.
        assert!(matches!(adopt_claim(root, "  ", "s"), Err(Fail::Thrown(m)) if m == "cell id is required."));
        assert!(matches!(adopt_claim(root, "c", " "), Err(Fail::Thrown(m)) if m == "session id is required."));
    }

    /// The second-port pin named in the fencing section header: this module's
    /// adoptClaim must leave the SAME bytes on disk as verbs/state_group.rs's
    /// narrowed twin (the `state handoff adopt` path). Re-derived rather than
    /// imported because that file is outside this cell's touchable set.
    #[test]
    fn adopt_agrees_with_the_state_group_port_on_the_shared_fixture() {
        let mine = cn_root();
        let theirs = cn_root();
        for root in [mine.path(), theirs.path()] {
            write_claim_fixture(root, "shared", Some("sess-a"), 600.0, OLD);
        }
        let AdoptClaimOutcome::Ok { .. } = adopt(mine.path(), "shared", "sess-b") else {
            panic!("expected an adoption");
        };
        let other = crate::verbs::state_group::adopt_claim(theirs.path(), "shared", "sess-b")
            .unwrap_or_else(|_| panic!("the state_group twin must also adopt"));
        let crate::verbs::state_group::AdoptOutcome::Adopted { claim, previous_owner } = other else {
            panic!("expected an adoption from the twin");
        };
        assert_eq!(previous_owner, Some(json!("sess-a")));
        let mut a = claim_on_disk(mine.path(), "shared");
        let mut b = Value::Object(claim);
        // The two ports stamp their own `now`; every other byte must agree.
        for v in [&mut a, &mut b] {
            let m = v.as_object_mut().unwrap();
            m.insert("claimed_at".into(), json!("<now>"));
            m.insert("adopted_at".into(), json!("<now>"));
        }
        assert_eq!(jsjson::stringify(&a), jsjson::stringify(&b));
        assert_eq!(a["fence_epoch"].as_f64(), Some(2.0));
    }

    /// Oracle: "renewClaimTTL refreshes claimed_at for this session's claims
    /// only, never touching adopted_from/adopted_at or fence_epoch", and "a
    /// claim whose gate is held is SKIPPED, never waited on".
    #[test]
    fn renew_touches_only_this_sessions_claims_and_never_the_fence() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "mine", Some("sess-a"), 600.0, OLD);
        write_claim_fixture(root, "theirs", Some("sess-b"), 600.0, OLD);
        write_claim_fixture(root, "nobodys", None, 600.0, OLD);
        write_claim_fixture(root, "gated", Some("sess-a"), 600.0, OLD);
        std::fs::write(claim_gate_path(root, "gated").unwrap(), "{}").unwrap();
        // Give `mine` a fence so "renewal never bumps it" is not vacuous.
        let mut with_fence = claim_on_disk(root, "mine");
        with_fence["fence_epoch"] = json!(4);
        std::fs::write(
            claims_dir(root).join("mine.json"),
            jsjson::stringify_pretty(&with_fence),
        )
        .unwrap();

        let RenewClaimOutcome::Ok { renewed, skipped } =
            renew_claim_ttl(root, "sess-a", None).unwrap()
        else {
            panic!("expected a renewal");
        };
        assert_eq!(renewed, vec!["mine".to_string()]);
        assert_eq!(skipped, vec!["gated".to_string()], "a held gate is skipped, never waited on");

        let renewed_claim = claim_on_disk(root, "mine");
        assert_ne!(renewed_claim["claimed_at"], json!(OLD), "the expiry clock advanced");
        assert_eq!(renewed_claim["acquired_at"], json!(OLD), "acquired_at never moves");
        assert_eq!(renewed_claim["fence_epoch"], json!(4), "renewal never bumps the fence");
        assert_eq!(claim_on_disk(root, "theirs")["claimed_at"], json!(OLD));
        assert_eq!(claim_on_disk(root, "nobodys")["claimed_at"], json!(OLD));
        assert_eq!(claim_on_disk(root, "gated")["claimed_at"], json!(OLD));
        assert!(!claim_gate_path(root, "mine").unwrap().exists(), "no gate leak");

        // An absent claims directory is an empty, non-throwing answer.
        let empty = cn_root();
        let RenewClaimOutcome::Ok { renewed, skipped } =
            renew_claim_ttl(empty.path(), "sess-a", None).unwrap()
        else {
            panic!("expected the empty answer");
        };
        assert!(renewed.is_empty() && skipped.is_empty());
    }

    /// Oracle: "renewClaimTTL refuses typed CLAIM_FENCE_STALE when the
    /// presented epoch is behind the claim's current fence_epoch, and renews
    /// NOTHING". NEGATIVE test: the stale state is constructed and the
    /// refusal bytes are pinned exactly.
    #[test]
    fn a_stale_presented_epoch_refuses_the_renew_and_writes_nothing() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "c-1", Some("sess-a"), 600.0, OLD);
        // A takeover already moved ownership forward: stored epoch is 3.
        let mut bumped = claim_on_disk(root, "c-1");
        bumped["fence_epoch"] = json!(3);
        std::fs::write(claims_dir(root).join("c-1.json"), jsjson::stringify_pretty(&bumped))
            .unwrap();
        let before = std::fs::read_to_string(claims_dir(root).join("c-1.json")).unwrap();

        for (presented, rendered) in
            [(json!(2), "2"), (json!(0), "0"), (json!(-1), "-1"), (json!(null), "null")]
        {
            let RenewClaimOutcome::Refused(r) =
                renew_claim_ttl(root, "sess-a", Some(&presented)).unwrap()
            else {
                panic!("{presented} must be refused");
            };
            assert_eq!(r.code, "CLAIM_FENCE_STALE");
            assert_eq!(
                r.reason,
                format!(
                    "cell \"c-1\" renew refused: presented epoch {rendered} is behind current fence_epoch 3 — a takeover already moved ownership forward; re-adopt before writing again."
                )
            );
            assert_eq!(r.extra["cell"], json!("c-1"));
            assert_eq!(r.extra["current_epoch"], json!(3.0));
            assert_eq!(
                std::fs::read_to_string(claims_dir(root).join("c-1.json")).unwrap(),
                before,
                "a fenced refusal renews nothing at all"
            );
            assert!(!claim_gate_path(root, "c-1").unwrap().exists(), "the gate is released in finally");
        }

        // Controls: the CURRENT epoch and an AHEAD epoch both renew, and
        // omitting the presentation is the legacy unfenced arm.
        for fresh in [json!(3), json!(4)] {
            let RenewClaimOutcome::Ok { renewed, .. } =
                renew_claim_ttl(root, "sess-a", Some(&fresh)).unwrap()
            else {
                panic!("presenting {fresh} must renew");
            };
            assert_eq!(renewed, vec!["c-1".to_string()]);
        }
        assert!(matches!(
            renew_claim_ttl(root, "sess-a", None).unwrap(),
            RenewClaimOutcome::Ok { .. }
        ));

        // A legacy claim with NO fence_epoch reads as 1: presenting 0 is
        // stale, presenting 1 renews.
        write_claim_fixture(root, "legacy", Some("sess-b"), 600.0, OLD);
        let RenewClaimOutcome::Refused(r) =
            renew_claim_ttl(root, "sess-b", Some(&json!(0))).unwrap()
        else {
            panic!("a legacy claim must fence at 1");
        };
        assert!(r.reason.contains("behind current fence_epoch 1"), "{}", r.reason);
        assert!(matches!(
            renew_claim_ttl(root, "sess-b", Some(&json!(1))).unwrap(),
            RenewClaimOutcome::Ok { .. }
        ));
    }

    /// Oracle: "releaseClaim refuses typed CLAIM_FENCE_STALE on a stale
    /// presentation and the claim file is left untouched" — the
    /// safety-critical half. Also pins the refusal ORDER: ownership is
    /// checked BEFORE fencing (fencing is orthogonal, never a substitute).
    #[test]
    fn a_stale_presented_epoch_refuses_the_release_and_never_removes_the_file() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "c-1", Some("sess-a"), 600.0, OLD);
        let mut bumped = claim_on_disk(root, "c-1");
        bumped["fence_epoch"] = json!(3);
        std::fs::write(claims_dir(root).join("c-1.json"), jsjson::stringify_pretty(&bumped))
            .unwrap();
        let file = claims_dir(root).join("c-1.json");
        let before = std::fs::read_to_string(&file).unwrap();

        let stale = json!(2);
        let ReleaseClaimOutcome::Refused(r) =
            release_claim_typed(root, Some("sess-a"), "c-1", Some(&stale)).unwrap()
        else {
            panic!("a stale release must refuse");
        };
        assert_eq!(r.code, "CLAIM_FENCE_STALE");
        assert_eq!(
            r.reason,
            "cell \"c-1\" release refused: presented epoch 2 is behind current fence_epoch 3 — a takeover already moved ownership forward; re-adopt before writing again."
        );
        assert!(file.exists(), "a fenced release must NEVER remove the claim file");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
        assert!(!claim_gate_path(root, "c-1").unwrap().exists());

        // Refusal ORDER: a foreign session presenting a FRESH epoch still gets
        // NOT_OWNER, not a fence answer.
        let ReleaseClaimOutcome::Refused(r) =
            release_claim_typed(root, Some("sess-x"), "c-1", Some(&json!(99))).unwrap()
        else {
            panic!("a foreign release must refuse");
        };
        assert_eq!(r.code, "NOT_OWNER");
        assert_eq!(r.reason, "cell \"c-1\" is owned by session \"sess-a\", not \"sess-x\".");
        assert!(file.exists());

        // Control: the owner presenting the current epoch releases for real.
        let ReleaseClaimOutcome::Ok { released } =
            release_claim_typed(root, Some("sess-a"), "c-1", Some(&json!(3))).unwrap()
        else {
            panic!("the owner must be able to release");
        };
        assert_eq!(released["cell"], json!("c-1"));
        assert!(!file.exists());
        // …and the NOT_FOUND rung is unchanged.
        let ReleaseClaimOutcome::Refused(r) =
            release_claim_typed(root, Some("sess-a"), "c-1", None).unwrap()
        else {
            panic!("a released claim is NOT_FOUND");
        };
        assert_eq!(r.code, "NOT_FOUND");
        assert_eq!(r.reason, "cell \"c-1\" has no claim to release.");
    }

    /// The whole point of the fence, end to end: an adoption moves ownership
    /// forward, and the STALE holder's later renew AND release are both
    /// refused with the epoch it no longer has — never silently applied.
    #[test]
    fn an_adoption_fences_out_the_previous_holders_later_writes() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "c-1", Some("sess-a"), 600.0, OLD);
        // sess-a's in-memory copy says epoch 1 (a fresh claimCellFile stamp).
        let held_epoch = json!(1);
        // A takeover happens behind its back.
        let AdoptClaimOutcome::Ok { claim, .. } = adopt(root, "c-1", "sess-b") else {
            panic!("expected an adoption");
        };
        assert_eq!(claim["fence_epoch"], json!(2.0));

        // The stale holder's renew is refused — and it is no longer the owner
        // either, so nothing is renewed on any path.
        let RenewClaimOutcome::Ok { renewed, .. } =
            renew_claim_ttl(root, "sess-a", Some(&held_epoch)).unwrap()
        else {
            panic!("session ownership alone already excludes sess-a");
        };
        assert!(renewed.is_empty());

        // The edge case session identity alone would MISS: the same session
        // re-adopts (so it owns the claim again) while a stale in-memory copy
        // still presents the pre-adoption epoch.
        let AdoptClaimOutcome::Ok { .. } = adopt(root, "c-1", "sess-a") else {
            panic!("expected a re-adoption");
        };
        assert_eq!(claim_on_disk(root, "c-1")["fence_epoch"].as_f64(), Some(3.0));
        let RenewClaimOutcome::Refused(r) =
            renew_claim_ttl(root, "sess-a", Some(&held_epoch)).unwrap()
        else {
            panic!("a stale epoch from the CURRENT owner must still fence");
        };
        assert_eq!(r.code, "CLAIM_FENCE_STALE");
        let ReleaseClaimOutcome::Refused(r) =
            release_claim_typed(root, Some("sess-a"), "c-1", Some(&held_epoch)).unwrap()
        else {
            panic!("a stale epoch must fence the release too");
        };
        assert_eq!(r.code, "CLAIM_FENCE_STALE");
        assert!(claims_dir(root).join("c-1.json").exists(), "still there — a stale fence never proceeds");
    }
}
