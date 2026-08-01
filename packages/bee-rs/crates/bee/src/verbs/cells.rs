// bee cells — natively served slice.
//
// READ-ONLY (unchanged from R3 wave 1): `cells list`, `cells ready`,
// `cells show` (flags: --json; --feature/--status on list; --feature on
// ready; --id on show).
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
// STILL DELEGATED (file-header contract):
//   - `cells claim-next` — heavy cross-module side effects this port cannot
//     close byte-for-byte: sweepExpiredClaims (cell resets + decision log
//     rows inside the sweep), resolvePipeline's session->lane binding with
//     typed LANE_* refusals, docs/backlog.md rank parsing (backlog.mjs),
//     live-session lane-ownership pooling, reservation-conflict +
//     cross-worktree foreign-hold selection filters (findSessionConflicts /
//     findForeignHolds over the full worktree topology). Every argv shape
//     for claim-next returns None.
//   - every argv shape any ported verb cannot PROVE: unknown flags, missing
//     required flags, --help, bad enum/number values (Node's validate()
//     speaks there), non-flag tokens, non-UTF-8 argv.
//   - any store shape whose Node rendering embeds V8 text: corrupt JSON
//     reached through readJson (fsutil warns with the V8 parse message),
//     JS-exotic spreads (string/array `trace`), non-f64-exact numbers.
//     These delegate BEFORE any output or write — the drift-cache write is
//     the one sanctioned pre-None write, exactly like the read-only slice.
//
// DOCUMENTED RESIDUAL DIVERGENCES (all pathological, none reachable from
// well-formed bee stores; each noted again at its code site):
//   - `--stdin` payloads: once stdin is consumed the probe can no longer
//     delegate (the Node child would read EOF), so a lone-surrogate escape
//     that V8's JSON.parse accepts, or a |n| >= 1e21 number, refuses
//     natively instead of being written the way Node would write it.
//   - hard mid-transaction filesystem failures (a failing rename inside the
//     archive loop, a failing final writeCell): Node embeds the libuv errno
//     message; the native error text carries the Rust io message instead.
//   - a store file that turns corrupt in the window between this port's
//     pre-scan and its post-test re-read (cap/finish only): Node would print
//     a readJson warning with V8 bytes; the native path takes the same
//     control-flow fallback without that warning line.
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
use crate::verbs::{emit_no_root_error, record_timing};
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
        Roots::NeedsNode => return None,
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
/// - Ok(None)     — absent/unreadable (fallback), or a literal JSON `null`
/// - Err(Delegate) — present but unparseable: Node warns to stderr with the
///   embedded V8 message, so the native path must hand the whole command back.
fn read_cell_json(file: &Path) -> Result<Option<Value>, Delegate> {
    match read_json(file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(Delegate),
        ReadJson::Parsed(Value::Null) => Ok(None),
        ReadJson::Parsed(v) => Ok(Some(v)),
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
/// probe path -> Delegate (Node's readJson warning).
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
enum Fail {
    Delegate,
    Thrown(String),
}
type MR<T> = Result<T, Fail>;

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
        // claim-next (see file header) and every unknown verb stay with Node.
        _ => None,
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
/// strip (the strict readers do NOT strip). Errors:
/// - NotJson: both engines refuse (native refusal is faithful) — unless the
///   text carries a \uD800-\uDFFF escape (V8 accepts lone surrogates where
///   serde refuses) in which case Delegate.
/// - Delegate: shapes whose JS value this port cannot carry (numbers with
///   |n| >= 1e21 — jsjson would print them differently than V8).
enum JsParse {
    Value(Value),
    NotJson,
    Delegate,
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
            Ok(v) => JsParse::Value(v),
            Err(_) => JsParse::Delegate,
        },
        Err(_) => {
            if has_lone_surrogate_escape(t) {
                JsParse::Delegate // V8's JSON.parse would accept it
            } else {
                JsParse::NotJson
            }
        }
    }
}

/// Detects `\uD800`..`\uDFFF` escapes (case-insensitive hex) — the one JSON
/// grammar corner where V8 accepts what serde_json refuses.
fn has_lone_surrogate_escape(text: &str) -> bool {
    let b = text.as_bytes();
    let mut i = 0usize;
    while i + 5 < b.len() {
        if b[i] == b'\\' && (b[i + 1] == b'u' || b[i + 1] == b'U') {
            let hex = &b[i + 2..i + 6];
            if hex.iter().all(|c| c.is_ascii_hexdigit()) {
                let v = u16::from_str_radix(std::str::from_utf8(hex).unwrap_or("0"), 16).unwrap_or(0);
                if (0xd800..=0xdfff).contains(&v) {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

// ─── store-file readers with JS-number normalization ───────────────────────

/// readJson-backed store read (BOM-stripped, warn-on-corrupt in Node):
/// Missing -> None; Corrupt -> Delegate; parsed value JS-number-normalized
/// (Delegate on non-representable numbers).
fn read_store_json(file: &Path) -> Result<Option<Value>, Delegate> {
    match read_json(file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(Delegate),
        ReadJson::Parsed(v) => match rsv::js_numberify(&v) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Err(Delegate),
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
        "{verb}: cell \"{id}\" is archived — unarchive its feature first (bee.mjs cells unarchive --feature <feature>)."
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

/// claims.mjs readClaim: readJson fallback null; falsy/non-object -> null.
/// A JSON-array claim file would take JS property paths this port does not
/// model — Delegate; corrupt JSON delegates (readJson's V8 warning).
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

/// claims.mjs releaseClaim — owner-matched removal under the gate (the
/// claim-unwind path of claimCellCrossSession). The caller ignores the typed
/// result; only the disk effect must match Node.
fn release_claim(control: &Path, session: Option<&str>, cell_id: &str) -> MR<()> {
    if read_claim(control, cell_id)?.is_none() {
        return Ok(()); // NOT_FOUND, ignored by the unwind caller
    }
    if !acquire_gate_with_retry(control, cell_id)? {
        return Ok(()); // GATE_HELD, ignored
    }
    let outcome = (|| -> MR<()> {
        let Some(claim) = read_claim(control, cell_id)? else { return Ok(()) };
        let owner: Option<String> = match claim.get("session") {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Null) | None => None,
            Some(_) => return Err(Fail::Delegate), // non-string session — JS-exotic compare
        };
        if owner.as_deref() != session {
            return Ok(()); // NOT_OWNER, ignored
        }
        let file = claim_path(control, cell_id)?;
        let _ = transient_fs_retry(|| match std::fs::remove_file(&file) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        });
        Ok(())
    })();
    release_gate(control, cell_id);
    outcome
}

// ─── sessions (claims.mjs) ─────────────────────────────────────────────────

/// claims.mjs readSession (fail-open flavor): malformed id -> None; corrupt
/// record -> Delegate (readJson warning); id-mismatch -> None.
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

/// decisions.mjs loadTaxonomy — readJson-backed (corrupt -> Delegate).
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
    script: &'static str,
    covers: &'static str,
    required: &'static str,
    command: &'static str,
    regen: &'static str,
    derive: fn(&str) -> (Vec<String>, Vec<String>),
}

const REGEN_GUARDS: [RegenGuardDef; 2] = [
    RegenGuardDef {
        script: "scripts/release_manifest.mjs",
        covers: "the release manifest hashes",
        required: "release_manifest.mjs --check",
        command: "node scripts/release_manifest.mjs --check",
        regen: "node scripts/render_plugin_skill_trees.mjs, then node packages/bee/scripts/onboard_bee.mjs --repo-root . --apply, then node scripts/release_manifest.mjs --write (in that order)",
        derive: derive_manifest_scope,
    },
    RegenGuardDef {
        script: "scripts/ledger_parity.mjs",
        covers: "the .bee/onboarding.json managed-hash ledger covers",
        required: "ledger_parity.mjs --check",
        command: "node scripts/ledger_parity.mjs --check",
        regen: "node packages/bee/scripts/onboard_bee.mjs --repo-root . --apply",
        derive: derive_ledger_scope,
    },
];

/// literalJoinSegments: leading string-literal args of one path.join arg list.
fn literal_join_segments(arg_text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    for raw in arg_text.split(',') {
        let arg = js_trim(raw);
        if arg.is_empty() {
            continue;
        }
        let chars: Vec<char> = arg.chars().collect();
        let quoted = chars.len() >= 2
            && (chars[0] == '"' || chars[0] == '\'')
            && chars[chars.len() - 1] == chars[0]
            && chars[1..chars.len() - 1].iter().all(|c| *c != '\\' && *c != chars[0]);
        if !quoted {
            break;
        }
        segments.push(chars[1..chars.len() - 1].iter().collect());
    }
    segments
}

/// Scan `path.join(\s*<ident>\s*,<capture-to-')'>)` occurrences of `source`.
fn joined_literal_arg_lists(source: &str, base_ident: &str) -> Vec<String> {
    let chars: Vec<char> = source.chars().collect();
    let ident: Vec<char> = base_ident.chars().collect();
    let mut found = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if !cs_starts_with(&chars, i, "path.join(") {
            i += 1;
            continue;
        }
        let mut j = ws_run(&chars, i + "path.join(".len());
        if j + ident.len() <= chars.len() && chars[j..j + ident.len()] == ident[..] {
            j += ident.len();
            j = ws_run(&chars, j);
            if j < chars.len() && chars[j] == ',' {
                let start = j + 1;
                let mut k = start;
                while k < chars.len() && chars[k] != ')' {
                    k += 1;
                }
                if k < chars.len() {
                    found.push(chars[start..k].iter().collect());
                }
            }
        }
        i += 1; // JS lastIndex resumes after the match; overlaps don't occur in practice
    }
    found
}

fn joined_literal_paths(source: &str, base_ident: &str) -> Vec<String> {
    joined_literal_arg_lists(source, base_ident)
        .into_iter()
        .filter_map(|args| {
            let segments = literal_join_segments(&args);
            if segments.is_empty() {
                None
            } else {
                Some(segments.join("/"))
            }
        })
        .collect()
}

/// deriveManifestScope: every path.join(REPO_ROOT, ...) literal, minus the
/// MANIFEST_PATH-derived path (which becomes the required file instead).
fn derive_manifest_scope(source: &str) -> (Vec<String>, Vec<String>) {
    let chars: Vec<char> = source.chars().collect();
    let mut manifest_path: Option<String> = None;
    for i in 0..chars.len() {
        if !word_boundary_before(&chars, i) || !cs_starts_with(&chars, i, "MANIFEST_PATH") {
            continue;
        }
        let mut j = ws_run(&chars, i + "MANIFEST_PATH".len());
        if !(j < chars.len() && chars[j] == '=') {
            continue;
        }
        j = ws_run(&chars, j + 1);
        if !cs_starts_with(&chars, j, "path.join(") {
            continue;
        }
        j = ws_run(&chars, j + "path.join(".len());
        if !cs_starts_with(&chars, j, "REPO_ROOT") {
            continue;
        }
        j = ws_run(&chars, j + "REPO_ROOT".len());
        if !(j < chars.len() && chars[j] == ',') {
            continue;
        }
        let start = j + 1;
        let mut k = start;
        while k < chars.len() && chars[k] != ')' {
            k += 1;
        }
        if k < chars.len() {
            let args: String = chars[start..k].iter().collect();
            manifest_path = Some(literal_join_segments(&args).join("/"));
            break; // non-global exec: first match only
        }
    }
    let mut roots: Vec<String> = Vec::new();
    for candidate in joined_literal_paths(source, "REPO_ROOT") {
        if !roots.contains(&candidate) {
            roots.push(candidate);
        }
    }
    let manifest = manifest_path.clone().filter(|m| !m.is_empty());
    let mut roots: Vec<String> = roots
        .into_iter()
        .filter(|c| !c.is_empty() && Some(c) != manifest.as_ref())
        .collect();
    js_default_str_sort(&mut roots);
    let required = match manifest {
        Some(m) => vec![m],
        None => Vec::new(),
    };
    (roots, required)
}

/// deriveLedgerScope: base from `path.join(root, "..", relDir)`, roots from
/// every `checkGroup(managed.X, "<relDir>")` call.
fn derive_ledger_scope(source: &str) -> (Vec<String>, Vec<String>) {
    let chars: Vec<char> = source.chars().collect();
    // /path\.join\(\s*root\s*,((?:\s*(?:"..."|'...')\s*,)+)\s*relDir\b/
    let mut base: Option<String> = None;
    'outer: for i in 0..chars.len() {
        if !cs_starts_with(&chars, i, "path.join(") {
            continue;
        }
        let mut j = ws_run(&chars, i + "path.join(".len());
        if !cs_starts_with(&chars, j, "root") {
            continue;
        }
        j = ws_run(&chars, j + 4);
        if !(j < chars.len() && chars[j] == ',') {
            continue;
        }
        j += 1;
        let mut literals: Vec<String> = Vec::new();
        loop {
            let mut k = ws_run(&chars, j);
            if k < chars.len() && (chars[k] == '"' || chars[k] == '\'') {
                let quote = chars[k];
                let start = k + 1;
                let mut m = start;
                while m < chars.len() && chars[m] != quote && chars[m] != '\\' {
                    m += 1;
                }
                if m >= chars.len() || chars[m] != quote {
                    continue 'outer;
                }
                let lit: String = chars[start..m].iter().collect();
                k = ws_run(&chars, m + 1);
                if !(k < chars.len() && chars[k] == ',') {
                    continue 'outer;
                }
                literals.push(lit);
                j = k + 1;
                continue;
            }
            // No further quoted literal: need \s*relDir\b with >= 1 literal.
            let k = ws_run(&chars, j);
            if !literals.is_empty()
                && cs_starts_with(&chars, k, "relDir")
                && !word_at(&chars, k + "relDir".len())
            {
                base = Some(literals.join("/"));
                break 'outer;
            }
            continue 'outer;
        }
    }
    let Some(base) = base.filter(|b| !b.is_empty()) else {
        return (Vec::new(), Vec::new());
    };
    // /checkGroup\(\s*managed\.\w+\s*,\s*("..."|'...')\s*\)/g
    let mut roots: Vec<String> = Vec::new();
    for i in 0..chars.len() {
        if !cs_starts_with(&chars, i, "checkGroup(") {
            continue;
        }
        let mut j = ws_run(&chars, i + "checkGroup(".len());
        if !cs_starts_with(&chars, j, "managed.") {
            continue;
        }
        j += "managed.".len();
        let word_start = j;
        while j < chars.len() && is_ascii_word(chars[j]) {
            j += 1;
        }
        if j == word_start {
            continue;
        }
        j = ws_run(&chars, j);
        if !(j < chars.len() && chars[j] == ',') {
            continue;
        }
        j = ws_run(&chars, j + 1);
        if !(j < chars.len() && (chars[j] == '"' || chars[j] == '\'')) {
            continue;
        }
        let quote = chars[j];
        let start = j + 1;
        let mut m = start;
        while m < chars.len() && chars[m] != quote && chars[m] != '\\' {
            m += 1;
        }
        if m >= chars.len() || chars[m] != quote {
            continue;
        }
        let rel: String = chars[start..m].iter().collect();
        let k = ws_run(&chars, m + 1);
        if !(k < chars.len() && chars[k] == ')') {
            continue;
        }
        let joined = if rel.is_empty() { base.clone() } else { format!("{base}/{rel}") };
        if !roots.contains(&joined) {
            roots.push(joined);
        }
    }
    js_default_str_sort(&mut roots);
    (roots, Vec::new())
}

struct ActiveGuard {
    def: &'static RegenGuardDef,
    roots: Vec<String>,
    required_files: Vec<String>,
}

/// deriveRegenGuards: absent script -> inactive; present-but-blind -> throw.
fn derive_regen_guards(root: &Path) -> MR<Vec<ActiveGuard>> {
    let mut active = Vec::new();
    for guard in REGEN_GUARDS.iter() {
        let mut file = root.to_path_buf();
        for segment in guard.script.split('/') {
            file = file.join(segment);
        }
        let source = match std::fs::read(&file) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(_) => continue, // guard not installed — nothing to owe
        };
        let (roots, required_files) = (guard.derive)(&source);
        if roots.is_empty() {
            return Err(Fail::Thrown(format!(
                "regen obligation: could not derive any covered root from \"{}\" — the guard would be blind, so the write is refused rather than passed silently. FIX: the script's shape changed; update deriveRegenGuards in lib/cells.mjs to read the new shape (never paste a literal root list in — see D2).",
                guard.script
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
fn regen_obligation_refusal(root: &Path, cell: &Map<String, Value>, verb: &str) -> MR<Option<String>> {
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
    for guard in derive_regen_guards(root)? {
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
            guard.def.script,
            missing.join("; "),
            fixes.join(", "),
            guard.def.regen,
        )));
    }
    Ok(None)
}

fn assert_regen_obligation(root: &Path, cell: &Map<String, Value>, verb: &str) -> MR<()> {
    match regen_obligation_refusal(root, cell, verb)? {
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
    assert_regen_obligation(root, map, "addCell")
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
/// governs) | Some(approved_gates). readLaneStrict's corrupt refusal is a
/// deterministic thrown message; its unreadable-file branch (embeds errno)
/// delegates.
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
        Err(_) => return Err(Fail::Delegate), // message embeds errno — Node's
    };
    let corrupt = || {
        Fail::Thrown(format!(
            "readLaneStrict: lane record \"{}\" exists but is corrupt (not a JSON object naming feature \"{id}\"). The bee CLI refuses to rebuild a lane from defaults over a present-but-corrupt file — that would silently clobber real lane state (gates, phase) while reporting success. FIX: inspect/restore the file (e.g. \"git checkout -- {}\"), then retry.",
            file.display(),
            lane_rel_path(&id)
        ))
    };
    let parsed = match parse_json_js(&text, false) {
        JsParse::Value(v) => v,
        JsParse::NotJson => return Err(corrupt()),
        JsParse::Delegate => return Err(Fail::Delegate),
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
/// consumed here (claimedFeatureHasRoute). Corrupt -> Delegate (Node warns).
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
        // Mismatched/corrupt-shaped record: readLane WARNS (deterministic
        // line, but stacked after readJson's own possible warn) — delegate
        // rather than model the warn cascade.
        Some(_) => Err(Fail::Delegate),
        None => Ok(None),
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

/// worktree-holds.mjs readStore (fail-open shape, Delegate on corrupt/null
/// entries).
fn read_holds_store(root: &Path) -> MR<Value> {
    let store = match read_json(&holds_ledger_path(root)) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => return Err(Fail::Delegate),
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

// ─── impact registry (scripts/impact_registry.mjs queryRegistry, E1) ───────
// Fail-open BY CONTRACT: the whole E1 block sits in a try/catch that
// swallows everything, so every unmodelable shape returns None (no warning)
// rather than delegating.

fn np_split_abs(p: &str) -> (String, Vec<String>) {
    let normalized = p.replace('\\', "/");
    let (prefix, rest) = if cfg!(windows) {
        let bytes = normalized.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
            (normalized[..2].to_string(), normalized[2..].to_string())
        } else {
            (String::new(), normalized.clone())
        }
    } else {
        (String::new(), normalized.clone())
    };
    let mut comps: Vec<String> = Vec::new();
    for part in rest.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                comps.pop();
            }
            other => comps.push(other.to_string()),
        }
    }
    (prefix, comps)
}

fn np_is_absolute(p: &str) -> bool {
    if cfg!(windows) {
        let b = p.as_bytes();
        p.starts_with('/') || p.starts_with('\\') || (b.len() >= 3 && b[1] == b':' && (b[2] == b'/' || b[2] == b'\\'))
    } else {
        p.starts_with('/')
    }
}

/// path.resolve(cwd, p) then path.relative(root, abs), '/'-joined — the
/// normalizeQueryPath shape (module REPO_ROOT == the CLI's own root when the
/// module lives at <root>/scripts/).
fn repo_relative_query(root: &Path, cwd: &Path, p: &str) -> String {
    let abs = if np_is_absolute(p) {
        np_split_abs(p)
    } else {
        let joined = format!("{}/{}", cwd.to_string_lossy(), p);
        np_split_abs(&joined)
    };
    let base = np_split_abs(&root.to_string_lossy());
    let eq = |a: &str, b: &str| {
        if cfg!(windows) {
            a.eq_ignore_ascii_case(b)
        } else {
            a == b
        }
    };
    if !eq(&abs.0, &base.0) {
        // Different roots: path.relative returns `to` as-is; '/'-joined.
        let mut out = abs.0.clone();
        out.push('/');
        out.push_str(&abs.1.join("/"));
        return out;
    }
    let mut common = 0usize;
    while common < abs.1.len() && common < base.1.len() && eq(&abs.1[common], &base.1[common]) {
        common += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in common..base.1.len() {
        parts.push("..".to_string());
    }
    for comp in &abs.1[common..] {
        parts.push(comp.clone());
    }
    parts.join("/")
}

/// capCell's E1 cross-check: Some(warning line) | None, never an error.
fn impact_registry_warning(
    root: &Path,
    cwd: &Path,
    cell_files: &[Value],
    verify: &str,
    id: &str,
) -> Option<String> {
    if cell_files.is_empty() {
        return None;
    }
    // The lazily-imported module must exist, or Node's catch skips silently.
    if !root.join("scripts").join("impact_registry.mjs").is_file() {
        return None;
    }
    let registry_text = std::fs::read_to_string(root.join("scripts").join("impact-registry.json")).ok()?;
    let registry: Value = serde_json::from_str(&registry_text).ok()?;
    let files = match registry.get("files") {
        Some(Value::Object(m)) => m,
        _ => return None, // property access on undefined throws -> catch
    };
    let mut mapped: Vec<String> = Vec::new();
    for f in cell_files {
        let Value::String(f) = f else { return None }; // path.isAbsolute(non-string) throws
        let rel = repo_relative_query(root, cwd, f);
        let Some(entry) = files.get(&rel) else { continue }; // unmapped
        let all = match entry.get("all") {
            Some(Value::Array(a)) => a,
            _ => continue, // undefined/absent -> unmapped branch
        };
        if all.is_empty() {
            continue;
        }
        let direct = match entry.get("direct") {
            Some(Value::Array(d)) => d,
            _ => return None, // for..of undefined throws -> catch
        };
        for s in direct {
            let Value::String(s) = s else { return None }; // non-string suites: unmodeled
            if !mapped.contains(s) {
                mapped.push(s.clone());
            }
        }
    }
    js_default_str_sort(&mut mapped);
    let missing: Vec<String> = mapped.into_iter().filter(|s| !verify.contains(s.as_str())).collect();
    if missing.is_empty() {
        return None;
    }
    Some(format!(
        "capCell: cell \"{id}\" verify does not mention impact-registry direct-edge suite(s) {} for file(s) {} — derived-check-hardening E1 non-blocking warning.",
        missing.join(", "),
        js_join(cell_files, ", ")
    ))
}

// ─── delegation pre-scans ──────────────────────────────────────────────────
// The mutators must never return None after an output or a write. Every
// Delegate-class trigger (corrupt JSON behind a readJson warn, JS-exotic
// number shapes) is probed up front; Thrown-class outcomes are ignored here
// (the real flow reproduces them at Node's own point in the order).

fn delegate_only<T>(result: MR<T>) -> MR<()> {
    match result {
        Err(Fail::Delegate) => Err(Fail::Delegate),
        _ => Ok(()),
    }
}

/// Probe every active + archived cell file for corrupt JSON / exotic numbers.
fn prescan_cells_store(root: &Path) -> MR<()> {
    let dir = cells_dir(root);
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !name.ends_with(".json") {
                continue;
            }
            if let Some(v) = read_cell_json(&entry.path()).map_err(|_| Fail::Delegate)? {
                rsv::js_numberify(&v).map_err(|_| Fail::Delegate)?;
            }
        }
    }
    let archive_root = dir.join(ARCHIVE_DIR_NAME);
    if let Ok(entries) = std::fs::read_dir(&archive_root) {
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            if let Ok(files) = std::fs::read_dir(entry.path()) {
                for file in files.flatten() {
                    let name = file.file_name();
                    let Some(name) = name.to_str() else { continue };
                    if !name.ends_with(".json") {
                        continue;
                    }
                    if let Some(v) = read_cell_json(&file.path()).map_err(|_| Fail::Delegate)? {
                        rsv::js_numberify(&v).map_err(|_| Fail::Delegate)?;
                    }
                }
            }
        }
    }
    Ok(())
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
            "WARNING: cell \"{id}\" verify mentions release_manifest but files is missing \"{RELEASE_MANIFEST_LINT_PATH}\" — a cold worker will hit red verify with no sanctioned fix. FIX: add the manifest path to files; regenerate it only via \"node scripts/release_manifest.mjs --write\"."
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
        // Pre-scan every Delegate trigger BEFORE stdin is consumed (once
        // read, the Node child would see EOF — no more None).
        prescan_cells_store(&root)?;
        delegate_only(read_commands_slice(&root))?;
        let text = if stdin {
            read_stdin_text()?
        } else {
            let file = require_flag_native(&flags, "file")?;
            read_file_text(&file, "cell")?
        };
        let payload = match parse_json_js(&text, false) {
            JsParse::Value(v) => v,
            JsParse::NotJson => return Err(Fail::Thrown("add: input is not valid JSON.".into())),
            JsParse::Delegate => {
                if stdin {
                    // Residual (header note): stdin is consumed — refuse the
                    // way a strict parse would rather than delegating.
                    return Err(Fail::Thrown("add: input is not valid JSON.".into()));
                }
                return Err(Fail::Delegate);
            }
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
        "tier" => Some("use the tier verb (bee.mjs cells tier --id ID --tier T)"),
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
        prescan_cells_store(&root)?;
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
            JsParse::Delegate => {
                if stdin {
                    return Err(Fail::Thrown("update: patch input is not valid JSON.".into()));
                }
                return Err(Fail::Delegate);
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
                Err(_) => return Err(Fail::Delegate), // message embeds errno — Node's
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
                JsParse::NotJson => {
                    return Err(Fail::Thrown(format!(
                        "updateCell: \"{}\" exists but is not valid JSON — refusing to merge a patch over a corrupt cell. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\"), then retry.",
                        file.display()
                    )))
                }
                JsParse::Delegate => return Err(Fail::Delegate),
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
            assert_regen_obligation(&root, &merged, "updateCell")?;
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
        if let Some(t) = ttl {
            if !t.is_finite() || t <= 0.0 {
                return Err(Fail::Thrown("--ttl must be a positive integer (seconds).".into()));
            }
        }
        // Pre-scan: everything after claimCellFile's O_EXCL write must never
        // delegate (the file would already exist for the Node re-run).
        prescan_cells_store(&root)?;
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
            if let Some(Value::String(feature)) = cell.get("feature") {
                delegate_only(read_lane_route(&root, feature))?;
            }
            match cell.get("deps") {
                None => {}
                Some(deps) if !js_truthy(deps) => {}
                Some(Value::Array(_)) => {}
                Some(_) => return Err(Fail::Delegate), // truthy non-array deps
            }
        }

        let session_id = resolve_session_flag_env(session_flag.as_deref());

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
                        "bee write-policy (shared-disjoint): no exact-path lease held for: {}. A broad/glob reservation never satisfies shared-disjoint — an exact-path lease is mandatory before write. FIX: bee.mjs reservations reserve --agent <worker> --cell <id> --path <path>{session_suffix} for each path, then retry.",
                        missing.join(", ")
                    )));
                }
            }
        }

        // claimCellCrossSession.
        if js_trim(&worker).is_empty() {
            return Err(Fail::Thrown("claimCellCrossSession: worker is required.".into()));
        }
        if js_trim(&id).is_empty() {
            return Err(Fail::Thrown("claimCellCrossSession: cellId is required.".into()));
        }
        let session = session_id.clone();
        let cell_id = js_trim(&id).to_string();
        let file_claim = claim_cell_file(&control, session.as_deref(), &cell_id, ttl)?;
        if let ClaimFileOutcome::Refused { code, reason } = file_claim {
            return Err(Fail::Thrown(format!("claim: {code} — {reason}")));
        }
        // Budget check inside the O_EXCL window.
        if let Some(Value::Object(cell_map)) = &cell_for_policy {
            match check_cell_budgets(cell_map) {
                Ok(BudgetCheck::Ok) => {}
                Ok(BudgetCheck::Refused { code, reason }) => {
                    release_claim(&control, session.as_deref(), &cell_id)?;
                    return Err(Fail::Thrown(format!("claim: {code} — {reason}")));
                }
                Err(fail) => {
                    // Pre-scanned; a mid-command race lands here — unwind the
                    // claim file before surfacing anything.
                    release_claim(&control, session.as_deref(), &cell_id)?;
                    return Err(fail);
                }
            }
        }
        // claimCell under the per-cell store lock; every throw unwinds the
        // claim file and surfaces as CLAIM_CELL_FAILED.
        let claim_result = (|| -> MR<Value> {
            let mut guard = acquire_named_lock(&root, &format!("cells:{cell_id}"))?;
            let outcome = (|| -> MR<Value> {
                assert_not_archived(&root, "claimCell", &cell_id)?;
                let cell = read_cell_norm(&root, &cell_id)?;
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
                        "claimCell: cell \"{cell_id}\" is \"{}\", not \"open\" — only open cells can be claimed. Run bee.mjs cells ready to list claimable cells.",
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
                trace.insert("worker".into(), Value::String(js_trim(&worker).to_string()));
                trace.insert(
                    "claim_session".into(),
                    session.clone().map(Value::String).unwrap_or(Value::Null),
                );
                trace.insert("claimed_at".into(), Value::String(utc_now()));
                cell_map.insert("trace".into(), Value::Object(trace));
                let cell_value = Value::Object(cell_map);
                write_cell(&root, &cell_value)?;
                Ok(cell_value)
            })();
            guard.release();
            outcome
        })();
        let claimed = match claim_result {
            Ok(cell) => cell,
            Err(Fail::Thrown(message)) => {
                release_claim(&control, session.as_deref(), &cell_id)?;
                return Err(Fail::Thrown(format!("claim: CLAIM_CELL_FAILED — {message}")));
            }
            Err(Fail::Delegate) => {
                // Pre-scanned; only a mid-command race lands here. Unwind so
                // the Node re-run isn't refused by our own claim file.
                release_claim(&control, session.as_deref(), &cell_id)?;
                return Err(Fail::Delegate);
            }
        };
        // explicit-triage D3 soft route warning (stderr, never a refusal).
        if !claimed_feature_has_route(&root, claimed.get("feature"))? {
            eprint!(
                "WARNING: cell \"{}\" claimed for feature \"{}\" with no route record — run \"bee state route --set --class <c> --lane <l> --flags <f> --files <n>\" to record the triage (D3, soft enforcement).\n",
                js_string_or_undefined(claimed.get("id")),
                js_string_or_undefined(claimed.get("feature"))
            );
        }
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
        JsParse::Delegate => Err(Fail::Delegate),
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
fn cap_cell_from_flags(root: &Path, cwd: &Path, f: &CapFlags, finish: bool) -> MR<Value> {
    let id = &f.id;
    // Pre-scan (see the pre-scan section header).
    prescan_cells_store(root)?;
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
                "capCell: cell \"{id}\" has a NEEDS_REVISION semantic-judge verdict — rework the cell and record a PASS verdict (bee.mjs cells judge-record), or cap with an audited override (bee.mjs cells cap --id {id} --override-judge \"<reason>\")."
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
        // derived-check-hardening E1 (loud, never a refusal, fail-open).
        let cell_files: Vec<Value> = match cell_map.get("files") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        let verify_text = match cell_map.get("verify") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let impact = impact_registry_warning(root, cwd, &cell_files, &verify_text, id);
        if let Some(warning) = &impact {
            eprintln!("{warning}");
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
        trace.insert(
            "warnings".into(),
            Value::Array(impact.into_iter().map(Value::String).collect()),
        );
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
        let cwd = std::env::current_dir().map_err(|_| Fail::Delegate)?;
        if let Some(file) = &deviations_file {
            if !file.is_empty() {
                // `flags['deviations-file'] ? parse : []` — truthy only.
                cap_flags.deviations = parse_deviations_file(file)?;
            }
        }
        let cell = cap_cell_from_flags(&root, &cwd, &cap_flags, finish)?;
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
    prescan_cells_store(root)?;
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
        let root2 = root.clone();
        let id2 = id.clone();
        // unclaimCell has NO assertNotArchived (an archived cell reads as
        // capped/dropped and takes the not-claimed refusal instead).
        let cell = mutate_cell(&root, &id, "unclaimCell", None, true, move |cell_map| {
            let claimed = matches!(cell_map.get("status"), Some(Value::String(s)) if s == "claimed");
            if !claimed {
                return Err(Fail::Thrown(format!(
                    "unclaimCell: cell \"{id2}\" is \"{}\", not \"claimed\" — only a claimed cell can be unclaimed (returned to open). For a capped/blocked/dropped cell use bee.mjs cells reopen.",
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
        let text = format!("Unclaimed {} — back to open.", js_string_or_undefined(cell.get("id")));
        Ok(Out::Emit(cell, text, 0))
    })
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
                        "reopenCell: cell \"{id2}\" is \"claimed\" — use bee.mjs cells unclaim to release the claim back to open."
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
        prescan_cells_store(&root)?;
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
            JsParse::NotJson => Value::String(raw.clone()), // free prose — validator rejects
            JsParse::Delegate => return Err(Fail::Delegate),
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
        prescan_cells_store(&root)?;
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
        ReadJson::Missing => return Ok(()),
        ReadJson::Corrupt => return Err(Fail::Delegate), // readJson warns (V8)
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

/// archivedSummary — {} on absent/shape-less, Delegate on corrupt.
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
        prescan_cells_store(&root)?;
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
    fn list_cells_delegates_on_corrupt_or_array_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        std::fs::write(cells_dir(root).join("bad.json"), "{nope").unwrap();
        assert!(list_cells(root, None, None).is_err(), "corrupt JSON must delegate");

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
    fn show_delegates_on_corrupt_or_non_object_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        std::fs::write(cells_dir(root).join("bad-1.json"), "{nope").unwrap();
        assert!(handle_show(root, "bad-1").is_err(), "corrupt cell delegates");
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
    #[test]
    fn regen_guard_derives_roots_and_refuses_missing_check() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("scripts")).unwrap();
        std::fs::write(
            root.join("scripts").join("release_manifest.mjs"),
            r#"
const MANIFEST_PATH = path.join(REPO_ROOT, "docs", "history", "m.json");
const A = path.join(REPO_ROOT, "templates");
const B = path.join(REPO_ROOT, "packages", "bee");
const C = path.join(REPO_ROOT, dynamic);
"#,
        )
        .unwrap();
        let guards = derive_regen_guards(root).unwrap();
        assert_eq!(guards.len(), 1);
        assert_eq!(guards[0].roots, vec!["packages/bee".to_string(), "templates".to_string()]);
        assert_eq!(guards[0].required_files, vec!["docs/history/m.json".to_string()]);
        // A cell touching a covered root without the check refuses…
        let cell = json!({
            "id": "r-1", "files": ["templates/x.md"], "verify": "echo ok"
        });
        let refusal = regen_obligation_refusal(root, cell.as_object().unwrap(), "addCell")
            .unwrap()
            .expect("must refuse");
        assert!(refusal.starts_with("addCell: REGEN_OBLIGATION — cell \"r-1\" touches \"templates/x.md\""));
        assert!(refusal.contains("verify does not contain \"release_manifest.mjs --check\""));
        assert!(refusal.contains("files does not list \"docs/history/m.json\""));
        // …the ack skips it, a compliant cell passes.
        let acked = json!({"id": "r-1", "files": ["templates/x.md"], "verify": "x", "regen_obligation_ack": "wave-barrier"});
        assert!(regen_obligation_refusal(root, acked.as_object().unwrap(), "addCell").unwrap().is_none());
        let ok = json!({
            "id": "r-1",
            "files": ["templates/x.md", "docs/history/m.json"],
            "verify": "node scripts/release_manifest.mjs --check"
        });
        assert!(regen_obligation_refusal(root, ok.as_object().unwrap(), "addCell").unwrap().is_none());
        // A present-but-blind guard throws the named refusal.
        std::fs::write(root.join("scripts").join("release_manifest.mjs"), "nothing here").unwrap();
        assert!(thrown(derive_regen_guards(root).map(|_| ()))
            .starts_with("regen obligation: could not derive any covered root"));
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
            "writeCell: cell \"z-1\" is archived — unarchive its feature first (bee.mjs cells unarchive --feature <feature>)."
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
        // Corrupt journal delegates (Node's readJson warning).
        std::fs::write(archive_journal_path(root, "f"), "{nope").unwrap();
        assert!(matches!(recover_archive_journal(root, "f"), Err(Fail::Delegate)));
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
            Some("use the tier verb (bee.mjs cells tier --id ID --tier T)")
        );
        assert_eq!(update_frozen_hint("status"), Some("status moves only through claim/verify/cap/block/drop"));
        assert_eq!(update_frozen_hint("nonsense"), None);
    }

    // ── impact registry query ─────────────────────────────────────────────
    #[test]
    fn impact_registry_warning_names_missing_direct_suites() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("scripts")).unwrap();
        std::fs::write(root.join("scripts").join("impact_registry.mjs"), "// module present").unwrap();
        std::fs::write(
            root.join("scripts").join("impact-registry.json"),
            r#"{"files": {"src/a.js": {"direct": ["suiteA", "suiteB"], "all": ["suiteA", "suiteB", "suiteC"]}}}"#,
        )
        .unwrap();
        let warning = impact_registry_warning(
            root,
            root,
            &[json!("src/a.js")],
            "runs suiteB only",
            "i-1",
        )
        .expect("must warn");
        assert_eq!(
            warning,
            "capCell: cell \"i-1\" verify does not mention impact-registry direct-edge suite(s) suiteA for file(s) src/a.js — derived-check-hardening E1 non-blocking warning."
        );
        // Verify covering every direct suite: silent.
        assert!(impact_registry_warning(root, root, &[json!("src/a.js")], "suiteA suiteB", "i-1").is_none());
        // No module / no registry / unmapped file: silent.
        assert!(impact_registry_warning(root, root, &[json!("src/other.js")], "", "i-1").is_none());
        std::fs::remove_file(root.join("scripts").join("impact_registry.mjs")).unwrap();
        assert!(impact_registry_warning(root, root, &[json!("src/a.js")], "", "i-1").is_none());
    }
}
