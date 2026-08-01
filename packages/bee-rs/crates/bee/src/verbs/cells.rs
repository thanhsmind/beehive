// bee cells — READ-ONLY slice served natively: `cells list`, `cells ready`,
// `cells show` (flags: --json; --feature/--status on list; --feature on
// ready; --id on show). Every MUTATING cells verb (add, update, claim, cap,
// finish, block, drop, unclaim, reopen, tier, judge, claim-next,
// reset-budget, judge-record, schedule, archive, unarchive) stays delegated
// to the Node runtime, as does every other argv shape: bare `cells`,
// unknown/misvalued flags, positionals, --help, a missing/empty --id (Node's
// validate() path), and any corrupt or JS-exotic on-disk shape whose Node
// rendering embeds V8 text.
//
// Mirrors bee.mjs end to end for the accepted shapes:
//   root resolution -> manifest-drift check (cache write) -> handler
//   (handleCellsList / handleCellsReady / handleCellsShow, backed by
//   lib/cells.mjs listCells / readyCells / depsAllCapped / readCell)
//   -> emit (drift stderr line, stdout payload) or emitError (handler throw)
//   -> timing (timings.jsonl + "[bee] cells <verb> Nms" stderr line).
//
// Conservative routing: `try_native` accepts ONLY argv where every token
// after the verb is one of the verb's own flags in a provably-equivalent
// form (`--flag value` with a non-`--` value token, or `--flag=value`).
// Anything else returns None before ANY output and the whole command re-runs
// under Node. A drift-cache write before a later None is acceptable (the
// Node re-run redoes the same idempotent write).

use crate::fsutil::{read_json, ReadJson};
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::verbs::{emit_no_root_error, record_timing};
use serde_json::{Map, Value};
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
        _ => return None, // mutating/unknown cells verbs, bare `cells --json`, ... — Node's
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_cell(root: &Path, id: &str, body: &Value) {
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
        write_cell(root, "w-10", &cell("w-10", "open", "f", json!([])));
        write_cell(root, "w-2", &cell("w-2", "open", "f", json!([])));
        write_cell(root, "a-1", &cell("a-1", "capped", "g", json!([])));
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
        write_cell(root, "f-1", &cell("f-1", "open", "feat", json!([])));
        write_cell(root, "f-2", &cell("f-2", "capped", "feat", json!([])));
        write_cell(root, "g-1", &cell("g-1", "open", "other", json!([])));
        // A cell with NO feature field never matches a truthy filter.
        write_cell(root, "h-1", &json!({"id": "h-1", "status": "open"}));
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
        write_cell(root, "base-1", &cell("base-1", "capped", "f", json!([])));
        write_cell(root, "base-2", &cell("base-2", "open", "f", json!([])));
        write_cell(root, "ok-1", &cell("ok-1", "open", "f", json!(["base-1"])));
        write_cell(root, "wait-1", &cell("wait-1", "open", "f", json!(["base-1", "base-2"])));
        write_cell(root, "ghost-1", &cell("ghost-1", "open", "f", json!(["missing-9"])));
        write_cell(root, "free-1", &cell("free-1", "open", "f", json!([])));
        // deps: falsy value behaves as [] (readiness unconditional).
        write_cell(root, "nul-1", &json!({"id": "nul-1", "status": "open", "deps": null}));
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
        write_cell(root, "next-1", &cell("next-1", "open", "f", json!(["old-1"])));
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
        write_cell(root, "s-1", &json!({"id": "s-1", "status": "open", "deps": "x-1"}));
        assert!(handle_ready(root, None).is_err(), "string deps (char iteration) delegates");

        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        write_cell(root2, "z-1", &json!({"id": "z-1", "status": "open", "deps": [""]}));
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
        write_cell(root, "bare-1", &json!({"id": "bare-1"}));
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
        write_cell(root, "t-1", &body);
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
        write_cell(root, "nv-1", &json!({"id": "nv-1", "status": "open"}));
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
}
