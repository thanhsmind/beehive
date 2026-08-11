// lib/cells.mjs read path + the JS value/collation semantics it needs
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt};
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

// ─── lib/cells.mjs read path ───────────────────────────────────────────────

pub(crate) fn cells_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("cells")
}

/// lib/cells.mjs ARCHIVE_DIR_NAME — a reserved child of cellsDir.
pub(crate) const ARCHIVE_DIR_NAME: &str = "archive";

/// lib/cells.mjs ID_PATTERN: /^[A-Za-z0-9][A-Za-z0-9._-]*$/.
pub(crate) fn id_pattern_ok(id: &str) -> bool {
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
pub(crate) fn read_cell_json(file: &Path) -> Result<Option<Value>, Delegate> {
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
pub(crate) fn warn_corrupt_json_once(file: &Path) {
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

/// PBI p-9c48a67c / ips-1 read-side residue (irf-1): the single feature a
/// GRANTED worktree island's `.bee/cells` reads are scoped to, or `None` for
/// every other shape (an ordinary checkout, the main store, or an UNGRANTED
/// worktree quietly sharing main's store — reads on those stay
/// byte-identical). `git worktree add` checks out `.bee/cells` in FULL (it
/// is git-tracked), and `ips-1`'s prune-on-register pass only ever removes
/// UNTRACKED foreign-feature files (registry.rs's `sync_worktree_cells`), so
/// a TRACKED foreign-feature cell legitimately rides along on disk forever —
/// it must never surface in a listing, a ready scan, or a status count.
///
/// Resolved ONCE per caller, off the SAME grant-identity walk every other
/// worktree-native verb already uses (`resolve_store_root_worktree` /
/// `LinkedRoots::granted()`, roots.rs) and the SAME creation-identity read
/// `bee status` / `bee orient` already use
/// (`status_full::read_worktree_feature`) — no second implementation of
/// either.
pub(crate) fn island_feature_scope(root: &Path) -> Option<String> {
    let RootsWt::Go(store_roots) = resolve_store_root_worktree(root) else {
        return None;
    };
    let linked = store_roots.linked.as_ref()?;
    if !linked.granted() {
        return None;
    }
    crate::verbs::status_full::read_worktree_feature(&root.to_string_lossy())
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
pub(crate) fn list_cells(root: &Path, feature: Option<&str>, status: Option<&str>) -> Result<Vec<Value>, Delegate> {
    let island_feature = island_feature_scope(root);
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
        if let Some(scope) = island_feature.as_deref() {
            if !matches!(map.get("feature"), Some(Value::String(s)) if s == scope) {
                continue; // foreign-feature residue in a granted island — never surfaced
            }
        }
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
pub(crate) fn read_cell(root: &Path, id: &str) -> Result<Option<Value>, Delegate> {
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
pub(crate) fn deps_all_capped_is_empty(root: &Path, cell: &Value) -> Result<bool, Delegate> {
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
pub(crate) fn js_truthy(v: &Value) -> bool {
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
pub(crate) fn js_string_or_undefined(v: Option<&Value>) -> String {
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
pub(crate) fn natural_cmp(a: &str, b: &str) -> Ordering {
    primary_cmp(a, b).then_with(|| tertiary_case_cmp(a, b))
}

/// ICU primary-strength class rank (probe-calibrated).
pub(crate) fn char_rank(c: char) -> u8 {
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
pub(crate) fn punct_key(c: char) -> (u8, u32) {
    match c {
        '_' => (0, 0),
        '-' => (1, 0),
        '.' => (2, 0),
        other => (3, other as u32),
    }
}

pub(crate) fn primary_cmp(a: &str, b: &str) -> Ordering {
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
pub(crate) fn tertiary_case_cmp(a: &str, b: &str) -> Ordering {
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
