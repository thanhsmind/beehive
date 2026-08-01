// bee backlog — native port of the backlog verb group (bee.mjs
// handleBacklogCounts/Findings/Add/Propose/Pbi* + lib/backlog.mjs).
//
// Verbs served natively (exact argv shapes only — see each probe):
//   backlog counts     [--json]
//   backlog findings   --feature <v> [--text <v>] [--json]
//   backlog add        --type <v> --title <v> --severity <v> --layer <v>
//                      [--detail <v>] [--feature <v>] [--json]
//   backlog propose    --story <v> --cos <v> [--feature <v>] [--json]
//   backlog pbi add    --title <v> [--cos <v>] [--status <v>] [--feature <v>]
//                      [--id <v>] [--json]
//   backlog pbi status --id <v> --to <v> [--feature <v>] [--json]
//   backlog pbi amend  --id <v> [--title <v>] [--cos <v>] [--json]
//   backlog pbi list   [--status <v>] [--json]
//   backlog rank       [--write] [--json]
//   backlog badges     [--write] [--json]
//   backlog render     [--write] [--check] [--json]
//
// STILL DELEGATED TO NODE (strangler note):
//   - `backlog add --queue-submit ...` — the scoped git auto-commit path
//     (commitBacklogRow's spawnSync git calls). Without the flag Node never
//     touches git, which is the exact subset ported here.
//   - within accepted shapes, every refusal path (missing/empty/over-length
//     flags, bad enums, duplicate/unknown pbi ids, whitespace-only values)
//     returns None so Node's byte-exact error text is preserved.
//
// Additional delegation triggers (None before any output/write):
//   - linked-worktree roots, corrupt manifest-hash cache/config
//   - backlog.jsonl lines only V8 could parse (lone surrogate escapes, ...)
//   - `pbi list` when any listed id falls outside ^p-[0-9a-f]+$ (Node sorts
//     by localeCompare; lowercase-hex `p-` ids are the provably
//     byte-order-identical subset — legacy `P<n>` ids delegate). `render`
//     uses the full calibrated ICU model instead (see `locale_cmp` below),
//     so it serves mixed `p-<hex>` / legacy `P<n>` stores natively and only
//     delegates on an id outside the calibrated alphabet (`collation_safe`).
//   - non-ASCII --feature/--text values (JS /i canonicalization + casefold)
//   - `findings` rows whose numbers fail the JS round-trip guard
//   - config product_root shapes Node warns about (non-string, missing dir,
//     drive-relative absolutes)
// The one replicated ERROR path is the backlog-pbi store-lock-busy refusal
// (BacklogPbiLockBusyError): delegating after the lock attempts would leave
// extra contention-telemetry writes behind, so the message is reproduced
// byte-for-byte instead.

use super::feedback::{
    backlog_allowed_type, emit_error, emit_success, find_ci, js_is_space, js_trim, js_truthy,
    now_iso, parse_shape, random_bytes, read_jsonl, require_flag, utf16_len, value_js_safe,
    ParsedArgs,
};
use crate::fsutil::append_jsonl;
use crate::jsjson;
use crate::lock::{acquire_store_lock_once, AcquireOnce};
use crate::registry::{check_manifest_drift, Drift};
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::emit_no_root_error;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const PBI_STATUSES: [&str; 5] = ["proposed", "in-flight", "parked", "done", "declined"];
const BACKLOG_STATUSES: [&str; 3] = ["proposed", "in-flight", "done"];
const BACKLOG_SEVERITIES: [&str; 3] = ["P1", "P2", "P3"];
const BACKLOG_MAX_TITLE: usize = 200;
const BACKLOG_MAX_LAYER: usize = 40;
const BACKLOG_MAX_STORY: usize = 200;
const BACKLOG_MAX_COS: usize = 2000;

fn backlog_jsonl_path(root: &Path) -> PathBuf {
    root.join(".bee").join("backlog.jsonl")
}

// ─── event-sourced PBI fold (lib/backlog.mjs foldPbis) ─────────────────────

#[derive(Clone)]
struct Pbi {
    id: String,
    title: String,
    cos: String,
    status: String,
    feature: Option<String>,
}

struct Fold {
    /// Map insertion order (first-add order), like the JS Map.
    order: Vec<String>,
    items: HashMap<String, Pbi>,
    has_events: bool,
}

/// foldPbis: last-event-wins fold over kind:'pbi' rows. None => delegate
/// (a line only V8 could parse).
fn fold_pbis(root: &Path) -> Option<Fold> {
    let read = read_jsonl(&backlog_jsonl_path(root));
    if read.needs_node {
        return None;
    }
    let mut fold = Fold { order: Vec::new(), items: HashMap::new(), has_events: false };
    for row in &read.rows {
        // JS lets arrays through `typeof row === 'object'`, but an array's
        // .kind is undefined — only real objects can be pbi rows.
        let Value::Object(m) = row else { continue };
        if m.get("kind").and_then(Value::as_str) != Some("pbi") {
            continue;
        }
        fold.has_events = true;
        let Some(id) = m.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()) else {
            continue;
        };
        let str_field = |name: &str| m.get(name).and_then(Value::as_str);
        match m.get("event").and_then(Value::as_str) {
            Some("add") => {
                if fold.items.contains_key(id) {
                    continue; // duplicate add refused — first add wins (D2)
                }
                let status = str_field("status")
                    .filter(|s| PBI_STATUSES.contains(s))
                    .unwrap_or("proposed");
                fold.items.insert(
                    id.to_string(),
                    Pbi {
                        id: id.to_string(),
                        title: str_field("title").unwrap_or("").to_string(),
                        cos: str_field("cos").unwrap_or("").to_string(),
                        status: status.to_string(),
                        feature: str_field("feature")
                            .filter(|s| !s.is_empty())
                            .map(str::to_string),
                    },
                );
                fold.order.push(id.to_string());
            }
            Some("status") => {
                let status = str_field("status")
                    .filter(|s| PBI_STATUSES.contains(s))
                    .map(str::to_string);
                let feature = str_field("feature").filter(|s| !s.is_empty()).map(str::to_string);
                if let Some(item) = fold.items.get_mut(id) {
                    if let Some(s) = status {
                        item.status = s;
                    }
                    if let Some(f) = feature {
                        item.feature = Some(f);
                    }
                }
            }
            Some("amend") => {
                let title = str_field("title").filter(|s| !s.is_empty()).map(str::to_string);
                let cos = str_field("cos").filter(|s| !s.is_empty()).map(str::to_string);
                if let Some(item) = fold.items.get_mut(id) {
                    if let Some(t) = title {
                        item.title = t;
                    }
                    if let Some(c) = cos {
                        item.cos = c;
                    }
                }
            }
            _ => {}
        }
    }
    Some(fold)
}

/// The fold item as bee.mjs emits it: {id, title, cos, status, feature}.
fn pbi_value(p: &Pbi) -> Value {
    let mut m = Map::new();
    m.insert("id".into(), Value::String(p.id.clone()));
    m.insert("title".into(), Value::String(p.title.clone()));
    m.insert("cos".into(), Value::String(p.cos.clone()));
    m.insert("status".into(), Value::String(p.status.clone()));
    m.insert("feature".into(), p.feature.clone().map(Value::String).unwrap_or(Value::Null));
    Value::Object(m)
}

/// R6a: the PBI fold as `backlog pbi list --json` emits each record, for
/// IN-PROCESS callers (`bee herding classify-lane`, which used to spawn that
/// very command from Node). Fold order, not the list verb's sorted order —
/// the only caller looks an id up. None => a JSONL line only V8 could parse,
/// which the caller must treat as an unreadable fold.
pub(crate) fn fold_pbi_records(root: &Path) -> Option<Vec<Value>> {
    let fold = fold_pbis(root)?;
    Some(fold.order.iter().filter_map(|id| fold.items.get(id)).map(pbi_value).collect())
}

/// localeCompare-safe id guard: within ^p-[0-9a-f]+$ (lowercase hex after a
/// fixed "p-" prefix) ICU collation and byte order provably agree.
fn id_sort_safe(id: &str) -> bool {
    let rest = match id.strip_prefix("p-") {
        Some(r) => r,
        None => return false,
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// ─── counts (readBacklogCounts: fold-first, legacy table fallback) ─────────

/// tokenKey: 'in-flight' -> 'inFlight'.
fn token_key(token: &str) -> String {
    let mut out = String::new();
    let mut chars = token.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '-' {
            if let Some(&n) = chars.peek() {
                if n.is_ascii_lowercase() {
                    out.push(n.to_ascii_uppercase());
                    chars.next();
                    continue;
                }
            }
            out.push('-');
        } else {
            out.push(c);
        }
    }
    out
}

/// splitRow: '|'-split, trimmed cells, bordering empties dropped.
fn split_row(line: &str) -> Vec<String> {
    let mut cells: Vec<String> = line.split('|').map(|c| js_trim(c).to_string()).collect();
    if cells.first().map(|c| c.is_empty()).unwrap_or(false) {
        cells.remove(0);
    }
    if cells.last().map(|c| c.is_empty()).unwrap_or(false) {
        cells.pop();
    }
    cells
}

/// normalizeStatus: strip bold/italic/code markup, trim, lowercase.
fn normalize_status(cell: &str) -> String {
    let stripped: String = cell.chars().filter(|c| !matches!(c, '*' | '`' | '_')).collect();
    js_trim(&stripped).to_lowercase()
}

/// resolveProductRoot (lib/state.mjs), happy paths only: absent/empty ->
/// root; an existing-directory string resolves. Every warn path (non-string,
/// missing dir, drive-relative absolutes where Node's win32 isAbsolute and
/// Rust's disagree) delegates.
fn resolve_product_root(root: &Path) -> Option<PathBuf> {
    let config = read_config_raw(root).ok()?;
    match config.get("product_root") {
        None | Some(Value::Null) => Some(root.to_path_buf()),
        Some(Value::String(s)) if s.is_empty() => Some(root.to_path_buf()),
        Some(Value::String(s)) => {
            let p = Path::new(s);
            if (s.starts_with('/') || s.starts_with('\\')) && !p.is_absolute() {
                return None;
            }
            let resolved = if p.is_absolute() { PathBuf::from(s) } else { lexical_resolve(root, s) };
            let is_dir = std::fs::metadata(&resolved).map(|m| m.is_dir()).unwrap_or(false);
            if is_dir {
                Some(resolved)
            } else {
                None // Node console.warns and still proceeds — delegate
            }
        }
        Some(_) => None, // non-string: Node console.warns
    }
}

/// path.resolve(root, rel) — lexical '..'/'.' normalization, both separators.
fn lexical_resolve(root: &Path, rel: &str) -> PathBuf {
    let mut out = root.to_path_buf();
    for part in rel.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            p => out.push(p),
        }
    }
    out
}

/// legacyBacklogCounts: count docs/backlog.md table rows by Status column.
/// None only when the file is absent/unreadable.
fn legacy_counts(product_root: &Path) -> Option<Map<String, Value>> {
    let bytes = std::fs::read(product_root.join("docs").join("backlog.md")).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let mut counts: Vec<(&str, usize)> = BACKLOG_STATUSES.iter().map(|s| (*s, 0)).collect();
    let mut status_index: Option<usize> = None;
    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if !line.contains('|') {
            continue;
        }
        let cells = split_row(line);
        match status_index {
            None => {
                // The header row is the first table row carrying 'Status'.
                if let Some(idx) = cells.iter().position(|c| normalize_status(c) == "status") {
                    status_index = Some(idx);
                }
            }
            Some(idx) => {
                if cells.len() <= idx {
                    continue; // malformed row: skipped
                }
                let token = normalize_status(&cells[idx]);
                if let Some(entry) = counts.iter_mut().find(|(s, _)| *s == token) {
                    entry.1 += 1;
                }
            }
        }
    }
    let total: usize = counts.iter().map(|(_, n)| n).sum();
    let mut m = Map::new();
    for (s, n) in counts {
        m.insert(token_key(s), Value::from(n));
    }
    m.insert("total".into(), Value::from(total));
    Some(m)
}

/// foldedBacklogCounts: one entry per PBI_STATUSES value + total.
fn folded_counts(fold: &Fold) -> Map<String, Value> {
    let mut counts: Vec<(&str, usize)> = PBI_STATUSES.iter().map(|s| (*s, 0)).collect();
    for id in &fold.order {
        let item = &fold.items[id];
        if let Some(entry) = counts.iter_mut().find(|(s, _)| *s == item.status) {
            entry.1 += 1;
        }
    }
    let total: usize = counts.iter().map(|(_, n)| n).sum();
    let mut m = Map::new();
    for (s, n) in counts {
        m.insert(token_key(s), Value::from(n));
    }
    m.insert("total".into(), Value::from(total));
    m
}

fn counts_text(counts: &Map<String, Value>) -> String {
    let n = |key: &str| counts.get(key).map(jsjson::js_to_string).unwrap_or_default();
    format!(
        "PBI: {} done / {} in-flight / {} proposed ({} total)",
        n("done"),
        n("inFlight"),
        n("proposed"),
        n("total")
    )
}

// ─── findings (sqs-b2) ─────────────────────────────────────────────────────

/// matchesWholeToken: case-insensitive token occurrence with neither
/// neighbor in [\w-] (the lookaround form). Token is ASCII-guarded upstream.
fn matches_whole_token(haystacks: &[String], token: &str) -> bool {
    let word_or_hyphen = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
    let tb = token.as_bytes();
    haystacks.iter().any(|h| {
        let hb = h.as_bytes();
        let mut i = 0;
        while let Some(p) = find_ci(hb, tb, i) {
            let pre_ok = p == 0 || !word_or_hyphen(hb[p - 1]);
            let end = p + tb.len();
            let post_ok = end == hb.len() || !word_or_hyphen(hb[end]);
            if pre_ok && post_ok {
                return true;
            }
            i = p + 1;
        }
        false
    })
}

const FINDING_KINDS: [&str; 2] = ["friction", "finding"];

/// isBacklogFindingRow: kind:'pbi' never; kind OR type in {friction,finding}.
fn is_finding_row(row: &Value) -> bool {
    let Value::Object(m) = row else { return false };
    if m.get("kind").and_then(Value::as_str) == Some("pbi") {
        return false;
    }
    let hit = |name: &str| {
        m.get(name)
            .and_then(Value::as_str)
            .map(|s| FINDING_KINDS.contains(&s))
            .unwrap_or(false)
    };
    hit("kind") || hit("type")
}

/// matchesBacklogFeature: String(row.feature) whole-token match.
fn matches_feature(row: &Value, feature: &str) -> bool {
    let value = match row.get("feature") {
        None | Some(Value::Null) => return false, // row.feature != null
        Some(v) => jsjson::js_to_string(v),
    };
    if value.is_empty() {
        return false;
    }
    matches_whole_token(&[value], feature)
}

/// matchesBacklogText: any whitespace-split term substring-hits title/detail.
fn matches_text(row: &Value, text: &str) -> bool {
    let lowered = text.to_lowercase();
    let terms: Vec<&str> = lowered.split(js_is_space).filter(|t| !t.is_empty()).collect();
    if terms.is_empty() {
        return true;
    }
    let mut haystacks: Vec<String> = Vec::new();
    for key in ["title", "detail"] {
        match row.get(key) {
            None | Some(Value::Null) => {}
            Some(Value::String(s)) if s.is_empty() => {}
            Some(v) => haystacks.push(jsjson::js_to_string(v).to_lowercase()),
        }
    }
    terms.iter().any(|term| haystacks.iter().any(|h| h.contains(term)))
}

/// JS template coercion for a possibly-absent row field.
fn coerce_row_field(row: &Value, name: &str) -> String {
    match row.get(name) {
        Some(v) => jsjson::js_to_string(v),
        None => "undefined".to_string(),
    }
}

// ─── the backlog-pbi store lock (pic-1 / issue #55) ────────────────────────

const BACKLOG_PBI_LOCK_NAME: &str = "backlog-pbi";
const LOCK_RETRY_ATTEMPTS: u32 = 15;
const LOCK_RETRY_DELAY_MS: u64 = 20;

enum PbiAdd {
    Ok(Pbi),
    /// Any Node-side throw (validation, duplicate id, fold delegate, append
    /// failure) — return None so Node re-runs and owns the refusal.
    Delegate,
    /// Lock busy past the retry budget — replicated natively (see header).
    Busy(Option<Value>),
}

/// addPbi (lib/backlog.mjs): fold-read + id generate/check + append as one
/// critical section under the backlog-pbi store lock, 1+15 bounded attempts
/// at 20ms — the same cadence as withBacklogPbiLockSync.
fn add_pbi(
    root: &Path,
    requested_id: Option<&str>,
    title: &str,
    cos: &str,
    status: &str,
    feature: Option<&str>,
) -> PbiAdd {
    let title_trim = js_trim(title);
    if title_trim.is_empty() {
        return PbiAdd::Delegate; // "pbi add: --title is required..."
    }
    if !PBI_STATUSES.contains(&status) {
        return PbiAdd::Delegate; // invalid --status enum
    }
    let requested = requested_id.map(js_trim).filter(|s| !s.is_empty());
    let cos_trim = js_trim(cos);
    let feature_trim = feature.map(js_trim).filter(|s| !s.is_empty());

    // Pre-lock delegation probe: a deterministic duplicate id (or a fold that
    // only V8 can read) must delegate WITHOUT acquiring the lock — acquiring
    // writes an "acquired" contention-telemetry row, and a write before
    // returning None breaks the strangler contract. The same checks re-run
    // under the lock below for the (vanishingly rare) racing-writer case.
    {
        let Some(fold) = fold_pbis(root) else { return PbiAdd::Delegate };
        if let Some(id) = requested {
            if fold.items.contains_key(id) {
                return PbiAdd::Delegate; // duplicate add refused — Node's text
            }
        }
    }

    let mut attempt = 0u32;
    let mut guard = loop {
        match acquire_store_lock_once(root, BACKLOG_PBI_LOCK_NAME) {
            AcquireOnce::Acquired(g) => break g,
            AcquireOnce::Busy { holder } => {
                if attempt >= LOCK_RETRY_ATTEMPTS {
                    return PbiAdd::Busy(holder);
                }
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(LOCK_RETRY_DELAY_MS));
            }
        }
    };

    let outcome = (|| {
        let Some(fold) = fold_pbis(root) else { return PbiAdd::Delegate };
        let final_id = match requested {
            Some(id) => {
                if fold.items.contains_key(id) {
                    return PbiAdd::Delegate; // duplicate add refused
                }
                id.to_string()
            }
            None => {
                let mut generated = None;
                for _ in 0..16 {
                    let candidate = format!("p-{}", super::feedback::hex_lower(&random_bytes(4)));
                    if !fold.items.contains_key(candidate.as_str()) {
                        generated = Some(candidate);
                        break;
                    }
                }
                match generated {
                    Some(id) => id,
                    None => return PbiAdd::Delegate, // exhausted (unreachable)
                }
            }
        };
        // Event key order: ts, kind, event, id, title, status[, cos][, feature].
        let mut event = Map::new();
        event.insert("ts".into(), Value::String(now_iso()));
        event.insert("kind".into(), Value::String("pbi".into()));
        event.insert("event".into(), Value::String("add".into()));
        event.insert("id".into(), Value::String(final_id.clone()));
        event.insert("title".into(), Value::String(title_trim.to_string()));
        event.insert("status".into(), Value::String(status.to_string()));
        if !cos_trim.is_empty() {
            event.insert("cos".into(), Value::String(cos_trim.to_string()));
        }
        if let Some(f) = feature_trim {
            event.insert("feature".into(), Value::String(f.to_string()));
        }
        if append_jsonl(&backlog_jsonl_path(root), &Value::Object(event)).is_err() {
            return PbiAdd::Delegate;
        }
        PbiAdd::Ok(Pbi {
            id: final_id,
            title: title_trim.to_string(),
            cos: cos_trim.to_string(),
            status: status.to_string(),
            feature: feature_trim.map(str::to_string),
        })
    })();
    guard.release();
    outcome
}

/// BacklogPbiLockBusyError's message, byte-for-byte.
fn lock_busy_message(holder: &Option<Value>) -> String {
    let who = match holder {
        Some(Value::Object(h)) => {
            let get = |key: &str| match h.get(key) {
                None | Some(Value::Null) => "unknown".to_string(), // ?? 'unknown'
                Some(v) => jsjson::js_to_string(v),
            };
            format!("pid={} session={} since {}", get("pid"), get("session"), get("ts"))
        }
        _ => "unknown holder".to_string(),
    };
    format!("backlog-pbi store lock busy: held by {who}")
}

// ─── dispatch ──────────────────────────────────────────────────────────────

struct Ctx {
    root: PathBuf,
    drift: Drift,
}

/// Root + manifest-drift preamble. Err(code) is the no-root exit; Ok(None)
/// delegates.
fn preamble(cmd: &str, pre_json: bool, t0: Instant) -> Result<Option<Ctx>, ExitCode> {
    let Ok(cwd) = std::env::current_dir() else { return Ok(None) };
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::NeedsNode => return Ok(None),
        Roots::None => return Err(emit_no_root_error(&cwd, cmd, pre_json, t0)),
    };
    let Ok(drift) = check_manifest_drift(&root) else { return Ok(None) };
    Ok(Some(Ctx { root, drift }))
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "backlog" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    match verb {
        "counts" => run_counts(parse_shape(&args[2..], &[])?, t0),
        "findings" => run_findings(parse_shape(&args[2..], &["feature", "text"])?, t0),
        "add" => run_add(
            parse_shape(&args[2..], &["type", "title", "severity", "layer", "detail", "feature"])?,
            t0,
        ),
        "propose" => run_propose(parse_shape(&args[2..], &["story", "cos", "feature"])?, t0),
        "pbi" => {
            let sub = args.get(2)?.to_str()?;
            let rest = &args[3..];
            match sub {
                "add" => run_pbi_add(
                    parse_shape(rest, &["title", "cos", "status", "feature", "id"])?,
                    t0,
                ),
                "status" => run_pbi_status(parse_shape(rest, &["id", "to", "feature"])?, t0),
                "amend" => run_pbi_amend(parse_shape(rest, &["id", "title", "cos"])?, t0),
                "list" => run_pbi_list(parse_shape(rest, &["status"])?, t0),
                _ => None,
            }
        }
        "rank" => run_rank(&args[2..], t0),
        "badges" => run_badges(&args[2..], t0),
        "render" => run_render(&args[2..], t0),
        _ => None, // unknown verbs stay delegated
    }
}

fn run_counts(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let ctx = match preamble("backlog counts", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let fold = fold_pbis(&ctx.root)?;
    let (result, text) = if fold.has_events {
        let counts = folded_counts(&fold);
        let text = counts_text(&counts);
        (Value::Object(counts), text)
    } else {
        let product_root = resolve_product_root(&ctx.root)?;
        match legacy_counts(&product_root) {
            None => (Value::Null, "No docs/backlog.md found.".to_string()),
            Some(counts) => {
                let text = counts_text(&counts);
                (Value::Object(counts), text)
            }
        }
    };
    Some(emit_success(&ctx.root, "backlog counts", parsed.json, &ctx.drift, &result, &text, t0))
}

fn run_findings(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let feature = require_flag(&parsed, "feature")?.to_string();
    if !feature.is_ascii() {
        return None; // JS /i canonicalization beyond ASCII — delegate
    }
    // `flags.text !== undefined && !== true ? String(flags.text) : null` —
    // '' stays '' (falsy: filter disabled), matching Node.
    let text_filter = parsed.flags.get("text").cloned();
    if let Some(t) = &text_filter {
        if !t.is_ascii() {
            return None;
        }
    }
    let ctx = match preamble("backlog findings", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let read = read_jsonl(&backlog_jsonl_path(&ctx.root));
    if read.needs_node {
        return None;
    }
    let findings: Vec<Value> = read
        .rows
        .into_iter()
        .filter(|row| {
            if !is_finding_row(row) || !matches_feature(row, &feature) {
                return false;
            }
            match &text_filter {
                Some(t) if !t.is_empty() => matches_text(row, t),
                _ => true,
            }
        })
        .collect();
    if !findings.iter().all(value_js_safe) {
        return None; // rows echo verbatim — numbers must round-trip like JS
    }
    let text = if findings.is_empty() {
        format!("No friction/finding rows for feature \"{feature}\".")
    } else {
        findings
            .iter()
            .map(|f| {
                let severity = match f.get("severity") {
                    Some(v) if js_truthy(v) => jsjson::js_to_string(v),
                    _ => "—".to_string(),
                };
                format!("[{severity}] {}", coerce_row_field(f, "title"))
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut result = Map::new();
    result.insert("findings".into(), Value::Array(findings));
    Some(emit_success(
        &ctx.root,
        "backlog findings",
        parsed.json,
        &ctx.drift,
        &Value::Object(result),
        &text,
        t0,
    ))
}

fn run_add(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    // requireFlags batch (ce-1): every missing/enum/length problem is a
    // Node-owned refusal — delegate on the first one seen.
    let ty = require_flag(&parsed, "type")?;
    if !backlog_allowed_type(ty) {
        return None;
    }
    let title = require_flag(&parsed, "title")?;
    let severity = require_flag(&parsed, "severity")?;
    if !BACKLOG_SEVERITIES.contains(&severity) {
        return None;
    }
    let layer = require_flag(&parsed, "layer")?;
    if utf16_len(title) > BACKLOG_MAX_TITLE || utf16_len(layer) > BACKLOG_MAX_LAYER {
        return None;
    }
    // `flags.detail !== undefined && !== true ? String(flags.detail) : ''`.
    let detail = parsed.flags.get("detail").cloned().unwrap_or_default();
    let feature = parsed.flags.get("feature").cloned().unwrap_or_default();

    let ctx = match preamble("backlog add", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    // Row key order: ts, type, title, detail, severity, layer, feature.
    let mut line = Map::new();
    line.insert("ts".into(), Value::String(now_iso()));
    line.insert("type".into(), Value::String(ty.to_string()));
    line.insert("title".into(), Value::String(title.to_string()));
    line.insert("detail".into(), Value::String(detail));
    line.insert("severity".into(), Value::String(severity.to_string()));
    line.insert("layer".into(), Value::String(layer.to_string()));
    line.insert("feature".into(), Value::String(feature));
    if append_jsonl(&backlog_jsonl_path(&ctx.root), &Value::Object(line.clone())).is_err() {
        return None;
    }
    // No --queue-submit in the accepted shapes, so commitBacklogRow returns
    // {committed:false, sha:null} without ever invoking git.
    let mut result = line;
    result.insert("committed".into(), Value::Bool(false));
    let text = format!("Appended {severity} {ty} row to .bee/backlog.jsonl: \"{title}\"");
    Some(emit_success(
        &ctx.root,
        "backlog add",
        parsed.json,
        &ctx.drift,
        &Value::Object(result),
        &text,
        t0,
    ))
}

fn run_propose(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let story = js_trim(require_flag(&parsed, "story")?).to_string();
    if story.is_empty() || utf16_len(&story) > BACKLOG_MAX_STORY {
        return None;
    }
    let cos = js_trim(require_flag(&parsed, "cos")?).to_string();
    if cos.is_empty() || utf16_len(&cos) > BACKLOG_MAX_COS {
        return None;
    }
    let feature = parsed.flags.get("feature").cloned().unwrap_or_default();

    let ctx = match preamble("backlog propose", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let item = match add_pbi(&ctx.root, None, &story, &cos, "proposed", Some(&feature)) {
        PbiAdd::Ok(item) => item,
        PbiAdd::Delegate => return None,
        PbiAdd::Busy(holder) => {
            return Some(emit_error(
                &ctx.root,
                "backlog propose",
                parsed.json,
                &lock_busy_message(&holder),
                t0,
            ))
        }
    };
    // Row key order: id, story, cos, feature. `item.cos || cos` — item.cos is
    // the trimmed, validated-non-empty cos, so it always wins.
    let feature_out = item.feature.clone().unwrap_or_else(|| "—".to_string());
    let mut row = Map::new();
    row.insert("id".into(), Value::String(item.id.clone()));
    row.insert("story".into(), Value::String(item.title.clone()));
    row.insert("cos".into(), Value::String(item.cos.clone()));
    row.insert("feature".into(), Value::String(feature_out.clone()));
    let text = format!("Proposed {}: \"{}\" (feature: {feature_out})", item.id, item.title);
    Some(emit_success(
        &ctx.root,
        "backlog propose",
        parsed.json,
        &ctx.drift,
        &Value::Object(row),
        &text,
        t0,
    ))
}

fn run_pbi_add(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let title = require_flag(&parsed, "title")?.to_string();
    let cos = parsed.flags.get("cos").cloned().unwrap_or_default();
    let status = parsed.flags.get("status").cloned().unwrap_or_else(|| "proposed".to_string());
    let feature = parsed.flags.get("feature").cloned();
    let id = parsed.flags.get("id").cloned();

    let ctx = match preamble("backlog pbi add", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let item = match add_pbi(&ctx.root, id.as_deref(), &title, &cos, &status, feature.as_deref()) {
        PbiAdd::Ok(item) => item,
        PbiAdd::Delegate => return None,
        PbiAdd::Busy(holder) => {
            return Some(emit_error(
                &ctx.root,
                "backlog pbi add",
                parsed.json,
                &lock_busy_message(&holder),
                t0,
            ))
        }
    };
    let text = format!("Added PBI {}: \"{}\"", item.id, item.title);
    Some(emit_success(
        &ctx.root,
        "backlog pbi add",
        parsed.json,
        &ctx.drift,
        &pbi_value(&item),
        &text,
        t0,
    ))
}

fn run_pbi_status(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let id_raw = require_flag(&parsed, "id")?.to_string();
    let to = require_flag(&parsed, "to")?.to_string();
    if !PBI_STATUSES.contains(&to.as_str()) {
        return None; // out-of-enum --to: Node's refusal
    }
    let feature_raw = parsed.flags.get("feature").cloned();
    let id = js_trim(&id_raw).to_string();
    if id.is_empty() {
        return None; // whitespace-only --id
    }

    let ctx = match preamble("backlog pbi status", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let fold = fold_pbis(&ctx.root)?;
    let current = fold.items.get(id.as_str())?.clone(); // unknown id -> Node
    let feature_trim = feature_raw
        .as_deref()
        .map(js_trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    // Event key order: ts, kind, event, id, status[, feature].
    let mut event = Map::new();
    event.insert("ts".into(), Value::String(now_iso()));
    event.insert("kind".into(), Value::String("pbi".into()));
    event.insert("event".into(), Value::String("status".into()));
    event.insert("id".into(), Value::String(id.clone()));
    event.insert("status".into(), Value::String(to.clone()));
    if let Some(f) = &feature_trim {
        event.insert("feature".into(), Value::String(f.clone()));
    }
    if append_jsonl(&backlog_jsonl_path(&ctx.root), &Value::Object(event)).is_err() {
        return None;
    }
    let merged = Pbi {
        status: to.clone(),
        feature: feature_trim.or(current.feature.clone()),
        ..current
    };
    // Text uses the RAW flag values, like the handler's template.
    let feature_suffix = match &feature_raw {
        Some(f) if !f.is_empty() => format!(" (feature: {f})"),
        _ => String::new(),
    };
    let text = format!("PBI {id_raw} -> {to}{feature_suffix}");
    Some(emit_success(
        &ctx.root,
        "backlog pbi status",
        parsed.json,
        &ctx.drift,
        &pbi_value(&merged),
        &text,
        t0,
    ))
}

fn run_pbi_amend(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let id_raw = require_flag(&parsed, "id")?.to_string();
    let title = parsed.flags.get("title").cloned();
    let cos = parsed.flags.get("cos").cloned();
    let title_trim = title.as_deref().map(js_trim).filter(|s| !s.is_empty()).map(str::to_string);
    let cos_trim = cos.as_deref().map(js_trim).filter(|s| !s.is_empty()).map(str::to_string);
    if title_trim.is_none() && cos_trim.is_none() {
        return None; // "at least one of --title or --cos is required."
    }
    let id = js_trim(&id_raw).to_string();
    if id.is_empty() {
        return None;
    }

    let ctx = match preamble("backlog pbi amend", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let fold = fold_pbis(&ctx.root)?;
    let current = fold.items.get(id.as_str())?.clone(); // unknown id -> Node
    // Event key order: ts, kind, event, id[, title][, cos].
    let mut event = Map::new();
    event.insert("ts".into(), Value::String(now_iso()));
    event.insert("kind".into(), Value::String("pbi".into()));
    event.insert("event".into(), Value::String("amend".into()));
    event.insert("id".into(), Value::String(id.clone()));
    if let Some(t) = &title_trim {
        event.insert("title".into(), Value::String(t.clone()));
    }
    if let Some(c) = &cos_trim {
        event.insert("cos".into(), Value::String(c.clone()));
    }
    if append_jsonl(&backlog_jsonl_path(&ctx.root), &Value::Object(event)).is_err() {
        return None;
    }
    let merged = Pbi {
        title: title_trim.unwrap_or_else(|| current.title.clone()),
        cos: cos_trim.unwrap_or_else(|| current.cos.clone()),
        ..current
    };
    let text = format!("Amended PBI {id_raw}");
    Some(emit_success(
        &ctx.root,
        "backlog pbi amend",
        parsed.json,
        &ctx.drift,
        &pbi_value(&merged),
        &text,
        t0,
    ))
}

fn run_pbi_list(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let status = parsed.flags.get("status").cloned();
    if let Some(s) = &status {
        // '' is falsy in listPbis (filter disabled); a non-empty out-of-enum
        // value is Node's refusal.
        if !s.is_empty() && !PBI_STATUSES.contains(&s.as_str()) {
            return None;
        }
    }
    let ctx = match preamble("backlog pbi list", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let fold = fold_pbis(&ctx.root)?;
    let mut list: Vec<&Pbi> = fold.order.iter().map(|id| &fold.items[id]).collect();
    if let Some(s) = &status {
        if !s.is_empty() {
            list.retain(|item| &item.status == s);
        }
    }
    // localeCompare guard: every listed id must be in the byte-order-safe
    // shape (legacy P<n> ids delegate).
    if !list.iter().all(|item| id_sort_safe(&item.id)) {
        return None;
    }
    list.sort_by(|a, b| a.id.cmp(&b.id));
    let text = if list.is_empty() {
        "No PBIs.".to_string()
    } else {
        list.iter()
            .map(|item| format!("{} [{}] {}", item.id, item.status, item.title))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let result = Value::Array(list.iter().map(|p| pbi_value(p)).collect());
    Some(emit_success(
        &ctx.root,
        "backlog pbi list",
        parsed.json,
        &ctx.drift,
        &result,
        &text,
        t0,
    ))
}

// ═══════════════════════════════════════════════════════════════════════════
// backlog rank / badges / render
// ═══════════════════════════════════════════════════════════════════════════

// ─── String.prototype.localeCompare(b) — non-numeric arm ───────────────────
//
// provenance: re-derived from verbs/cells.rs `natural_cmp`/`primary_cmp`/
// `tertiary_case_cmp` and verbs/status_full.rs `locale_cmp(a, b, false)`
// (both private to their modules; verbs/decisions.rs carries the same
// re-derivation for the decisions index). computeBacklogRenderContent's
// tiebreak is `a.id.localeCompare(b.id)` — default locale, no options.
//
// The model: whitespace < punctuation < digits < letters, ICU's
// '_' < '-' < '.' inside punctuation, letters case-insensitive at primary
// strength, first case difference (lowercase first) as the tertiary
// tiebreak, shorter-prefix first. This is what makes `p-4ae119b0` sort
// BEFORE `P40` (primary 'p' == 'P', then punctuation '-' < digit '4') where
// plain byte order would put `P40` first — which is why `pbi list`'s
// narrower `id_sort_safe` byte-order guard cannot serve the render sort.
// `locale_cmp_agrees_with_the_calibrated_probes` asserts agreement with the
// same measured V8/ICU probe vectors.
fn locale_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let n = av.len().min(bv.len());
    for k in 0..n {
        let ord = lc_primary_key(av[k]).cmp(&lc_primary_key(bv[k]));
        if ord != Ordering::Equal {
            return ord;
        }
    }
    let ord = av.len().cmp(&bv.len());
    if ord != Ordering::Equal {
        return ord;
    }
    for k in 0..n {
        let (x, y) = (av[k], bv[k]);
        if x != y && x.is_alphabetic() && y.is_alphabetic() {
            let (lx, ly) = (x.is_lowercase(), y.is_lowercase());
            if lx != ly {
                return if lx { Ordering::Less } else { Ordering::Greater };
            }
        }
    }
    Ordering::Equal
}

fn lc_primary_key(c: char) -> (u8, u32) {
    if c.is_whitespace() {
        return (0, c as u32);
    }
    match c {
        '_' => (1, 0),
        '-' => (1, 1),
        ',' => (1, 2),
        ';' => (1, 3),
        ':' => (1, 4),
        '!' => (1, 5),
        '?' => (1, 6),
        '.' => (1, 7),
        _ if c.is_ascii_digit() => (2, c as u32 - '0' as u32),
        _ if c.is_alphabetic() => (3, c.to_lowercase().next().unwrap_or(c) as u32),
        _ => (1, 100 + c as u32),
    }
}

/// The alphabet the collation model above is CALIBRATED on. A PBI id with
/// anything else leaves the proven region — the verb delegates.
fn collation_safe(key: &str) -> bool {
    key.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.'))
}

/// PBI_RANK_WEIGHT / PBI_RANK_UNKNOWN_WEIGHT (lib/backlog.mjs).
fn pbi_rank_weight(status: &str) -> i32 {
    match status {
        "in-flight" => 0,
        "proposed" => 1,
        "parked" => 2,
        "done" => 3,
        "declined" => 4,
        _ => 5,
    }
}

fn backlog_md_path(product_root: &Path) -> PathBuf {
    product_root.join("docs").join("backlog.md")
}

// ─── computeBacklogRenderContent / renderBacklogPbiView ───────────────────

const BACKLOG_RENDER_HEADER: &str = concat!(
    "<!--\n",
    "GENERATED FILE — do not hand-edit.\n",
    "Rendered by `bee backlog render` from event-sourced PBI records in .bee/backlog.jsonl (backlog-unification D1/D3).\n",
    "Regenerate: `bee backlog render --write`. Check freshness: `bee backlog render --check`.\n",
    "Deterministic: byte-identical for the same backlog.jsonl contents — status-grouped, id-sorted entries, LF endings,\n",
    "never a generation timestamp or any other wall-clock value.\n",
    "-->",
);

/// escapeCell: newlines flattened to a space, pipes escaped, then trimmed.
/// `String(value || '')` — an empty string stays empty.
fn escape_cell(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' if chars.peek() == Some(&'\n') => {
                chars.next();
                out.push(' ');
            }
            '\n' => out.push(' '),
            '|' => out.push_str("\\|"),
            other => out.push(other),
        }
    }
    js_trim(&out).to_string()
}

/// computeBacklogRenderContent. None => delegate (unparseable rows, or an id
/// outside the calibrated collation alphabet).
fn compute_backlog_render_content(root: &Path) -> Option<String> {
    let fold = fold_pbis(root)?;
    let mut all: Vec<&Pbi> = fold.order.iter().map(|id| &fold.items[id]).collect();
    if !all.iter().all(|p| collation_safe(&p.id)) {
        return None;
    }
    // JS sort is stable (ES2019+), so a stable sort_by reproduces it exactly.
    all.sort_by(|a, b| {
        pbi_rank_weight(&a.status)
            .cmp(&pbi_rank_weight(&b.status))
            .then_with(|| locale_cmp(&a.id, &b.id))
    });
    let collapsed_status = |s: &str| s == "done" || s == "declined";

    let mut lines: Vec<String> = vec![
        "# Product Backlog".into(),
        String::new(),
        BACKLOG_RENDER_HEADER.into(),
        String::new(),
        "| ID | Story | CoS | Status | Feature |".into(),
        "|----|-------|-----|--------|---------|".into(),
    ];
    for item in all.iter().filter(|p| !collapsed_status(&p.status)) {
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            escape_cell(&item.id),
            escape_cell(&item.title),
            escape_cell(&item.cos),
            item.status,
            // `item.feature || '—'` — an empty/absent feature falls back.
            escape_cell(item.feature.as_deref().filter(|f| !f.is_empty()).unwrap_or("—")),
        ));
    }
    let collapsed: Vec<&&Pbi> = all.iter().filter(|p| collapsed_status(&p.status)).collect();
    if !collapsed.is_empty() {
        lines.push(String::new());
        lines.push("## Done / Declined".into());
        lines.push(String::new());
        for item in collapsed {
            lines.push(format!(
                "- [{}] {} — {}",
                escape_cell(&item.id),
                escape_cell(&item.title),
                item.status
            ));
        }
    }
    Some(format!("{}\n", lines.join("\n")))
}

/// renderBacklogPbiView(root, {write}) -> (changed, content).
fn render_backlog_pbi_view(product_root: &Path, root: &Path, write: bool) -> Option<(bool, String)> {
    let content = compute_backlog_render_content(root)?;
    let file = backlog_md_path(product_root);
    let existing = std::fs::read(&file)
        .ok()
        .map(|b| String::from_utf8_lossy(&b).into_owned());
    let changed = existing.as_deref() != Some(content.as_str());
    if write {
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir).ok()?;
        }
        std::fs::write(&file, &content).ok()?;
    }
    Some((changed, content))
}

// ─── walkBacklogIdRows / rankBacklog ──────────────────────────────────────

const RANK_UNKNOWN_WEIGHT: i32 = 2;

fn rank_weight(token: &str) -> i32 {
    match token {
        "in-flight" => 0,
        "proposed" => 1,
        "done" => 3,
        _ => RANK_UNKNOWN_WEIGHT,
    }
}

struct RankRow {
    id: String,
    weight: i32,
    position: usize,
}

/// walkBacklogIdRows — the single docs/backlog.md data-row parse. None when
/// the file is absent or has no parseable table.
fn walk_backlog_id_rows(product_root: &Path) -> Option<Vec<RankRow>> {
    let bytes = std::fs::read(backlog_md_path(product_root)).ok()?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let lines: Vec<&str> = text
        .split('\n')
        .map(|seg| seg.strip_suffix('\r').unwrap_or(seg))
        .collect();
    let mut status_index: isize = -1;
    let mut separator_line: isize = -1;
    let mut rows: Vec<RankRow> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if !line.contains('|') {
            // A non-table line after the table body ends the block.
            if separator_line != -1 && !rows.is_empty() {
                break;
            }
            continue;
        }
        let cells = split_row(line);
        if status_index == -1 {
            if let Some(idx) = cells.iter().position(|c| normalize_status(c) == "status") {
                status_index = idx as isize;
            }
            continue;
        }
        if separator_line == -1 {
            separator_line = i as isize; // the |---| row right after the header
            continue;
        }
        let token = if cells.len() as isize > status_index {
            normalize_status(&cells[status_index as usize])
        } else {
            String::new()
        };
        let id = match cells.first() {
            Some(c) if !c.is_empty() => {
                let stripped: String =
                    c.chars().filter(|ch| !matches!(ch, '*' | '`' | '_')).collect();
                js_trim(&stripped).to_string()
            }
            _ => String::new(),
        };
        let position = rows.len();
        rows.push(RankRow { id, weight: rank_weight(&token), position });
    }
    if status_index == -1 || separator_line == -1 || rows.is_empty() {
        return None;
    }
    Some(rows)
}

// ─── featureBacklogRank (fsh-11 D2 cross-lane ordering) ───────────────────
// The OPPOSITE lookup from rankBacklog above: "where does feature X rank",
// keyed by the Feature column (or, fold-first, by the PBI's own `feature`)
// rather than by row id. `cells claim-next`'s cross-lane pool is the only
// caller — hence pub(crate).

/// lib/backlog.mjs featureBacklogRank. `None` = delegate (a backlog.jsonl
/// line only V8 parses, an unresolvable product_root, or a PBI id outside
/// `id_sort_safe`'s proven localeCompare region).
pub(crate) fn feature_backlog_rank(root: &Path) -> Option<HashMap<String, usize>> {
    let fold = fold_pbis(root)?;
    if fold.has_events {
        // `[...folded.items.values()]` — JS Map value order is insertion order.
        let mut rows: Vec<(Option<&str>, i32, &str)> = Vec::with_capacity(fold.order.len());
        for id in &fold.order {
            let item = fold.items.get(id)?;
            if !id_sort_safe(&item.id) {
                return None; // a.id.localeCompare(b.id) outside the calibrated alphabet
            }
            rows.push((
                item.feature.as_deref(),
                pbi_rank_weight(&item.status),
                item.id.as_str(),
            ));
        }
        // `a.weight - b.weight || a.id.localeCompare(b.id)` — stable, like V8's.
        rows.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| locale_cmp(a.2, b.2)));
        let mut map: HashMap<String, usize> = HashMap::new();
        for (rank, row) in rows.iter().enumerate() {
            if let Some(feature) = row.0 {
                map.entry(feature.to_string()).or_insert(rank);
            }
        }
        return Some(map);
    }

    // Legacy pre-migration branch: the docs/backlog.md table's Feature column.
    let product_root = resolve_product_root(root)?;
    let bytes = match std::fs::read(backlog_md_path(&product_root)) {
        Ok(b) => b,
        Err(_) => return Some(HashMap::new()), // readFileSync threw → new Map()
    };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let lines: Vec<&str> = text
        .split('\n')
        .map(|seg| seg.strip_suffix('\r').unwrap_or(seg))
        .collect();
    let mut status_index: isize = -1;
    let mut feature_index: isize = -1;
    let mut separator_line: isize = -1;
    // (feature, weight, position)
    let mut rows: Vec<(Option<String>, i32, usize)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if !line.contains('|') {
            if separator_line != -1 && !rows.is_empty() {
                break;
            }
            continue;
        }
        let cells = split_row(line);
        if status_index == -1 {
            if let Some(idx) = cells.iter().position(|c| normalize_status(c) == "status") {
                status_index = idx as isize;
                feature_index = cells
                    .iter()
                    .position(|c| normalize_status(c) == "feature")
                    .map(|p| p as isize)
                    .unwrap_or(-1);
            }
            continue;
        }
        if separator_line == -1 {
            separator_line = i as isize; // the |---| row right after the header
            continue;
        }
        let token = if cells.len() as isize > status_index {
            normalize_status(&cells[status_index as usize])
        } else {
            String::new()
        };
        let raw_feature = if feature_index != -1 && cells.len() as isize > feature_index {
            cells[feature_index as usize].as_str()
        } else {
            ""
        };
        // `.replace(/[*`_]/g, '').trim()` — markup stripped, then trimmed.
        let stripped: String = raw_feature
            .chars()
            .filter(|c| !matches!(c, '*' | '`' | '_'))
            .collect();
        let feature = js_trim(&stripped).to_string();
        let position = rows.len();
        let feature = if !feature.is_empty() && feature != "\u{2014}" && feature != "-" {
            Some(feature)
        } else {
            None
        };
        rows.push((feature, rank_weight(&token), position));
    }
    if status_index == -1 || feature_index == -1 || separator_line == -1 || rows.is_empty() {
        return Some(HashMap::new());
    }
    let mut ranked: Vec<&(Option<String>, i32, usize)> = rows.iter().collect();
    ranked.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.2.cmp(&b.2)));
    let mut map: HashMap<String, usize> = HashMap::new();
    for (rank, row) in ranked.iter().enumerate() {
        if let Some(feature) = &row.0 {
            map.entry(feature.clone()).or_insert(rank);
        }
    }
    Some(map)
}

/// rankBacklog(root, {write:false}) — read-only: handleBacklogRank refuses
/// --write outright, so the write arm is never reached from the CLI.
fn rank_backlog(product_root: &Path) -> Option<(bool, Vec<String>)> {
    let rows = walk_backlog_id_rows(product_root)?;
    let mut ranked: Vec<&RankRow> = rows.iter().collect();
    ranked.sort_by(|a, b| a.weight.cmp(&b.weight).then_with(|| a.position.cmp(&b.position)));
    // `ranked.some((row, i) => row !== rows[i])` — reference inequality, i.e.
    // any row that moved out of its original slot.
    let changed = ranked.iter().enumerate().any(|(i, r)| r.position != i);
    Some((changed, ranked.iter().map(|r| r.id.clone()).collect()))
}

// ─── renderBacklogBadges / updateReadmeBadges ─────────────────────────────

const BADGE_MARKER_START: &str = "<!-- BEE:BACKLOG-BADGES:START -->";
const BADGE_MARKER_END: &str = "<!-- BEE:BACKLOG-BADGES:END -->";

fn badge_color(status: &str) -> &'static str {
    match status {
        "done" => "brightgreen",
        "in-flight" => "blue",
        "proposed" => "lightgrey",
        "parked" => "yellow",
        "declined" => "red",
        _ => "",
    }
}

/// shieldsEscape: '-' doubles, '_' doubles, space becomes '%20'.
fn shields_escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '-' => out.push_str("--"),
            '_' => out.push_str("__"),
            ' ' => out.push_str("%20"),
            other => out.push(other),
        }
    }
    out
}

/// renderBacklogBadges — fold-first status set, counts from
/// readBacklogCounts. Ok(None) = Node's null (no counts at all).
fn render_backlog_badges(root: &Path, product_root: &Path) -> Option<Option<String>> {
    let fold = fold_pbis(root)?;
    let counts = if fold.has_events {
        folded_counts(&fold)
    } else {
        match legacy_counts(product_root) {
            None => return Some(None),
            Some(c) => c,
        }
    };
    let statuses: &[&str] = if fold.has_events { &PBI_STATUSES } else { &BACKLOG_STATUSES };
    let badges: Vec<String> = statuses
        .iter()
        .rev() // done first — the headline number
        .map(|status| {
            let label = shields_escape(&format!("backlog {status}"));
            // `counts[key] || 0` — a 0 count still renders as 0.
            let value = counts
                .get(&token_key(status))
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            format!(
                "![backlog {status}](https://img.shields.io/badge/{label}-{}-{})",
                jsjson::js_to_string(&Value::from(value)),
                badge_color(status)
            )
        })
        .collect();
    Some(Some(badges.join(" ")))
}

/// updateReadmeBadges(root, {write}) -> Node's {changed, badges} | null.
fn update_readme_badges(
    root: &Path,
    product_root: &Path,
    write: bool,
) -> Option<Option<(bool, String)>> {
    let Some(badges) = render_backlog_badges(root, product_root)? else {
        return Some(None);
    };
    let file = product_root.join("README.md");
    let Ok(bytes) = std::fs::read(&file) else {
        return Some(None);
    };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let block = format!("{BADGE_MARKER_START}\n{badges}\n{BADGE_MARKER_END}");
    let next = if text.contains(BADGE_MARKER_START) && text.contains(BADGE_MARKER_END) {
        // `text.replace(/START[\s\S]*?END/)` — FIRST start marker, then the
        // nearest following end marker (non-greedy), replaced once.
        let s = text.find(BADGE_MARKER_START)?;
        let after = s + BADGE_MARKER_START.len();
        let Some(rel_e) = text[after..].find(BADGE_MARKER_END) else {
            // START present but no END after it: the JS regex finds no match,
            // so `replace` is a no-op.
            return Some(Some((false, badges)));
        };
        let e = after + rel_e + BADGE_MARKER_END.len();
        format!("{}{}{}", &text[..s], block, &text[e..])
    } else {
        // No markers yet: place the block right under the first heading line.
        let mut lines: Vec<String> = text
            .split('\n')
            .map(|seg| seg.strip_suffix('\r').unwrap_or(seg).to_string())
            .collect();
        let at = lines
            .iter()
            .position(|l| l.starts_with('#'))
            .map(|i| i + 1)
            .unwrap_or(0);
        lines.splice(at..at, [String::new(), block]);
        lines.join("\n")
    };
    let changed = next != text;
    if write && changed {
        std::fs::write(&file, &next).ok()?;
    }
    Some(Some((changed, badges)))
}

// ─── routing for the three boolean-flag verbs ─────────────────────────────

/// parseFlags for a verb whose only flags are FLAG_ALONE_BOOLEANS members
/// (`--write`, `--check`) plus `--json`. Returns each flag's `=== true`
/// reading. `--x=true`/`--x=false` pass validate() but are NOT `true`;
/// any other `=value` is validate()'s refusal (delegate), and so is any
/// unknown flag or non-flag token.
fn parse_bool_shape(tokens: &[OsString], bool_flags: &[&str]) -> Option<(HashMap<String, bool>, bool, bool)> {
    let toks: Vec<&str> = tokens.iter().map(|t| t.to_str()).collect::<Option<_>>()?;
    let pre_json = toks.iter().any(|t| *t == "--json" || t.starts_with("--json="));
    let mut out: HashMap<String, bool> = HashMap::new();
    let mut json = false;
    for tok in &toks {
        if !tok.starts_with("--") {
            return None;
        }
        let body = &tok[2..];
        let (name, eq_val) = match body.find('=') {
            Some(p) => (&body[..p], Some(&body[p + 1..])),
            None => (body, None),
        };
        if name == "json" {
            if eq_val.is_some() {
                return None;
            }
            json = true;
            continue;
        }
        if !bool_flags.contains(&name) {
            return None;
        }
        match eq_val {
            None => {
                out.insert(name.to_string(), true);
            }
            Some("true") | Some("false") => {
                out.insert(name.to_string(), false); // present, but !== true
            }
            Some(_) => return None, // validate() refuses
        }
    }
    Some((out, json, pre_json))
}

fn run_rank(tokens: &[OsString], t0: Instant) -> Option<ExitCode> {
    let (flags, json, pre_json) = parse_bool_shape(tokens, &["write"])?;
    let ctx = match preamble("backlog rank", pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    if *flags.get("write").unwrap_or(&false) {
        // Deterministic retirement refusal — no store read, no lock.
        return Some(emit_error(
            &ctx.root,
            "backlog rank",
            json,
            "backlog rank --write is retired — \"bee backlog render --write\" now owns the generated docs/backlog.md view.",
            t0,
        ));
    }
    let product_root = resolve_product_root(&ctx.root)?;
    let (result, text) = match rank_backlog(&product_root) {
        None => (Value::Null, "No parseable backlog table in docs/backlog.md.".to_string()),
        Some((changed, order)) => {
            let verb = if changed { "Would reorder to" } else { "Already ordered" };
            let suffix = if changed {
                " (\"rank --write\" is retired — run \"bee backlog render --write\" instead)"
            } else {
                ""
            };
            let text = format!("{verb}: {}{suffix}", order.join(", "));
            let mut m = Map::new();
            m.insert("changed".into(), Value::Bool(changed));
            m.insert(
                "order".into(),
                Value::Array(order.into_iter().map(Value::String).collect()),
            );
            (Value::Object(m), text)
        }
    };
    Some(emit_success(&ctx.root, "backlog rank", json, &ctx.drift, &result, &text, t0))
}

fn run_badges(tokens: &[OsString], t0: Instant) -> Option<ExitCode> {
    let (flags, json, pre_json) = parse_bool_shape(tokens, &["write"])?;
    let write = *flags.get("write").unwrap_or(&false);
    let ctx = match preamble("backlog badges", pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let product_root = resolve_product_root(&ctx.root)?;
    let (result, text) = match update_readme_badges(&ctx.root, &product_root, write)? {
        None => (
            Value::Null,
            "README.md or docs/backlog.md missing — nothing to badge.".to_string(),
        ),
        Some((changed, badges)) => {
            let verb = if write {
                if changed { "README badges refreshed" } else { "README badges already current" }
            } else if changed {
                "README badges stale (re-run with --write to apply)"
            } else {
                "README badges already current"
            };
            let text = format!("{verb}: {badges}");
            let mut m = Map::new();
            m.insert("changed".into(), Value::Bool(changed));
            m.insert("badges".into(), Value::String(badges));
            (Value::Object(m), text)
        }
    };
    Some(emit_success(&ctx.root, "backlog badges", json, &ctx.drift, &result, &text, t0))
}

fn run_render(tokens: &[OsString], t0: Instant) -> Option<ExitCode> {
    let (flags, json, pre_json) = parse_bool_shape(tokens, &["write", "check"])?;
    let write = *flags.get("write").unwrap_or(&false);
    let check = *flags.get("check").unwrap_or(&false);
    let ctx = match preamble("backlog render", pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let product_root = resolve_product_root(&ctx.root)?;
    let (changed, content) = render_backlog_pbi_view(&product_root, &ctx.root, write)?;
    if check && changed {
        // Deterministic drift refusal (fixed wording, no V8 text).
        return Some(emit_error(
            &ctx.root,
            "backlog render",
            json,
            "backlog render --check: docs/backlog.md is stale. FIX: run \"bee backlog render --write\" to refresh it.",
            t0,
        ));
    }
    let verb = if write {
        if changed { "Rendered" } else { "Already current" }
    } else if check {
        if changed { "STALE" } else { "Current" }
    } else if changed {
        "Would render (re-run with --write to apply)"
    } else {
        "Already current"
    };
    let mut m = Map::new();
    m.insert("changed".into(), Value::Bool(changed));
    m.insert("content".into(), Value::String(content));
    Some(emit_success(
        &ctx.root,
        "backlog render",
        json,
        &ctx.drift,
        &Value::Object(m),
        &format!("{verb}: docs/backlog.md"),
        t0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_backlog(root: &Path, lines: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(backlog_jsonl_path(root), lines).unwrap();
    }

    // ── rank / badges / render ─────────────────────────────────────────────

    /// Same calibrated V8/ICU probe vectors verbs/cells.rs and
    /// verbs/status_full.rs were pinned against — this file's re-derived
    /// `locale_cmp` must answer them identically.
    #[test]
    fn locale_cmp_agrees_with_the_calibrated_probes() {
        use std::cmp::Ordering::*;
        let probes: &[(&str, &str, std::cmp::Ordering)] = &[
            ("a b", "a_b", Less),
            ("a_b", "a-b", Less), // ICU '_' < '-'
            ("a-b", "a.b", Less), // ICU '-' < '.'
            ("a.b", "a0b", Less),
            ("a0b", "aab", Less),
            ("x10", "x9", Less),  // non-numeric: '1' < '9'
            ("x09", "x10", Less),
            ("P1", "P10", Less),  // prefix first
            ("Ab", "aC", Less),   // case is a deferred tertiary
            ("zed", "Zed", Less), // lowercase first on a primary tie
            ("ab", "ab", Equal),
            // The case byte order gets wrong: punctuation beats a digit even
            // when the byte value says otherwise.
            ("p-4ae119b0", "P40", Less),
            ("p-727e9529", "P43", Less),
        ];
        for (a, b, want) in probes {
            assert_eq!(locale_cmp(a, b), *want, "locale_cmp({a:?}, {b:?})");
            assert_eq!(locale_cmp(b, a), want.reverse(), "reverse({a:?}, {b:?})");
        }
        // The real repo's in-flight group, byte-diffed against Node.
        let mut ids = vec!["P40", "P76", "p-727e9529", "P43", "p-4ae119b0", "P69"];
        ids.sort_by(|a, b| locale_cmp(a, b));
        assert_eq!(
            ids,
            vec!["p-4ae119b0", "p-727e9529", "P40", "P43", "P69", "P76"]
        );
        assert!(collation_safe("p-4ae119b0"));
        assert!(collation_safe("P77"));
        assert!(!collation_safe("p→x"));
    }

    #[test]
    fn escape_cell_flattens_newlines_and_escapes_pipes() {
        assert_eq!(escape_cell("  a | b  "), "a \\| b");
        assert_eq!(escape_cell("one\r\ntwo\nthree"), "one two three");
        assert_eq!(escape_cell(""), "");
        assert_eq!(escape_cell("\n  x  \n"), "x");
        assert_eq!(shields_escape("backlog in-flight"), "backlog%20in--flight");
        assert_eq!(shields_escape("a_b"), "a__b");
    }

    #[test]
    fn render_content_groups_by_weight_then_collated_id() {
        let tmp = tempfile::tempdir().unwrap();
        write_backlog(
            tmp.path(),
            concat!(
                r#"{"kind":"pbi","event":"add","id":"P40","title":"forty","status":"in-flight"}"#, "\n",
                r#"{"kind":"pbi","event":"add","id":"p-727e9529","title":"port","status":"in-flight"}"#, "\n",
                r#"{"kind":"pbi","event":"add","id":"P9","title":"nine","status":"proposed","cos":"a|b"}"#, "\n",
                r#"{"kind":"pbi","event":"add","id":"P10","title":"ten\nlines","status":"done"}"#, "\n",
                r#"{"kind":"pbi","event":"add","id":"p-00ff11aa","title":"dec","status":"declined"}"#, "\n",
                r#"{"kind":"pbi","event":"add","id":"P2","title":"two","status":"parked","feature":"f x"}"#, "\n",
            ),
        );
        let content = compute_backlog_render_content(tmp.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // in-flight (ICU: p-… before P40), proposed, parked; done/declined
        // collapse below.
        assert_eq!(lines[10], "| ID | Story | CoS | Status | Feature |");
        assert_eq!(lines[12], "| p-727e9529 | port |  | in-flight | — |");
        assert_eq!(lines[13], "| P40 | forty |  | in-flight | — |");
        assert_eq!(lines[14], "| P9 | nine | a\\|b | proposed | — |");
        assert_eq!(lines[15], "| P2 | two |  | parked | f x |");
        assert_eq!(lines[17], "## Done / Declined");
        assert_eq!(lines[19], "- [P10] ten lines — done");
        assert_eq!(lines[20], "- [p-00ff11aa] dec — declined");
        assert!(content.ends_with('\n'));

        // An empty store still renders the stable shell.
        let empty = tempfile::tempdir().unwrap();
        write_backlog(empty.path(), "");
        let shell = compute_backlog_render_content(empty.path()).unwrap();
        assert!(shell.ends_with("|----|-------|-----|--------|---------|\n"));

        // An id outside the calibrated alphabet delegates.
        let exotic = tempfile::tempdir().unwrap();
        write_backlog(
            exotic.path(),
            "{\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p→x\",\"title\":\"t\"}\n",
        );
        assert!(compute_backlog_render_content(exotic.path()).is_none());
    }

    #[test]
    fn rank_walks_the_legacy_table_and_reports_order() {
        let tmp = tempfile::tempdir().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(
            docs.join("backlog.md"),
            [
                "# Backlog",
                "",
                "| ID | Story | Status |",
                "|----|-------|--------|",
                "| P3 | third | done |",
                "| `P1` | first | **proposed** |",
                "| P10 | tenth | in-flight |",
                "| P2 | second | weird |",
                "| | blank | proposed |",
                "",
                "prose ends the block",
                "| X | outside | done |",
            ]
            .join("\n"),
        )
        .unwrap();
        let (changed, order) = rank_backlog(tmp.path()).unwrap();
        assert!(changed);
        // in-flight(0) < proposed(1) < unknown(2) < done(3); stable within.
        assert_eq!(order, vec!["P10", "P1", "", "P2", "P3"]);
        // Markup stripped from the ID cell; the post-prose row never joins.
        assert!(!order.contains(&"X".to_string()));

        // Already-ranked table reports no change.
        std::fs::write(
            docs.join("backlog.md"),
            "| ID | Status |\n|----|--------|\n| A | in-flight |\n| B | done |\n",
        )
        .unwrap();
        assert_eq!(rank_backlog(tmp.path()).unwrap().0, false);

        // No table / no file -> null.
        std::fs::write(docs.join("backlog.md"), "# Just prose\n").unwrap();
        assert!(rank_backlog(tmp.path()).is_none());
        assert!(rank_backlog(&tmp.path().join("nowhere")).is_none());
    }

    #[test]
    fn badges_use_the_fold_first_status_set_and_rewrite_the_marker_block() {
        let tmp = tempfile::tempdir().unwrap();
        write_backlog(
            tmp.path(),
            "{\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p-aa11bb22\",\"title\":\"t\",\"status\":\"done\"}\n",
        );
        let badges = render_backlog_badges(tmp.path(), tmp.path()).unwrap().unwrap();
        // Five PBI statuses, done first (reversed enum order).
        assert!(badges.starts_with(
            "![backlog declined](https://img.shields.io/badge/backlog%20declined-0-red)"
        ));
        assert!(badges.ends_with(
            "![backlog proposed](https://img.shields.io/badge/backlog%20proposed-0-lightgrey)"
        ));
        assert!(badges.contains("backlog%20in--flight-0-blue"));
        assert!(badges.contains("backlog%20done-1-brightgreen"));

        // No README -> Node's null.
        assert!(update_readme_badges(tmp.path(), tmp.path(), false).unwrap().is_none());

        // Markers present: the block is replaced in place.
        let readme = tmp.path().join("README.md");
        std::fs::write(
            &readme,
            format!("# T\n\n{BADGE_MARKER_START}\nOLD\n{BADGE_MARKER_END}\n\ntail\n"),
        )
        .unwrap();
        let (changed, _) = update_readme_badges(tmp.path(), tmp.path(), true).unwrap().unwrap();
        assert!(changed);
        let text = std::fs::read_to_string(&readme).unwrap();
        assert!(text.starts_with("# T\n\n<!-- BEE:BACKLOG-BADGES:START -->\n![backlog declined]"));
        assert!(text.ends_with("<!-- BEE:BACKLOG-BADGES:END -->\n\ntail\n"));
        // Idempotent.
        assert_eq!(update_readme_badges(tmp.path(), tmp.path(), true).unwrap().unwrap().0, false);

        // No markers: the block lands right under the first heading line.
        std::fs::write(&readme, "intro\n# Title\nbody\n").unwrap();
        update_readme_badges(tmp.path(), tmp.path(), true).unwrap().unwrap();
        let text = std::fs::read_to_string(&readme).unwrap();
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(lines[0], "intro");
        assert_eq!(lines[1], "# Title");
        assert_eq!(lines[2], "");
        assert_eq!(lines[3], BADGE_MARKER_START);
        // No heading at all: the block goes to the top.
        std::fs::write(&readme, "no heading\n").unwrap();
        update_readme_badges(tmp.path(), tmp.path(), true).unwrap().unwrap();
        assert!(std::fs::read_to_string(&readme).unwrap().starts_with("\n<!-- BEE:BACKLOG-BADGES:START -->"));
    }

    #[test]
    fn bool_shape_parser_matches_flag_alone_boolean_semantics() {
        let os = |v: &[&str]| -> Vec<OsString> { v.iter().map(OsString::from).collect() };
        let (f, json, pre) = parse_bool_shape(&os(&["--write", "--json"]), &["write"]).unwrap();
        assert_eq!(f.get("write"), Some(&true));
        assert!(json && pre);
        // `--write=true` passes validate() but is not `=== true`.
        let (f, _, _) = parse_bool_shape(&os(&["--write=true"]), &["write"]).unwrap();
        assert_eq!(f.get("write"), Some(&false));
        let (f, _, _) = parse_bool_shape(&os(&["--write=false"]), &["write"]).unwrap();
        assert_eq!(f.get("write"), Some(&false));
        // Everything Node answers itself delegates.
        assert!(parse_bool_shape(&os(&["--write=maybe"]), &["write"]).is_none());
        assert!(parse_bool_shape(&os(&["--check"]), &["write"]).is_none());
        assert!(parse_bool_shape(&os(&["stray"]), &["write"]).is_none());
        assert!(parse_bool_shape(&os(&["--json=1"]), &["write"]).is_none());
        assert!(parse_bool_shape(&os(&[]), &["write"]).unwrap().0.is_empty());
    }

    #[test]
    fn fold_applies_add_status_amend_last_event_wins() {
        let tmp = tempfile::tempdir().unwrap();
        write_backlog(
            tmp.path(),
            concat!(
                "{\"ts\":\"t\",\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p-aa11bb22\",\"title\":\"first\",\"status\":\"proposed\",\"cos\":\"c1\"}\n",
                "{\"ts\":\"t\",\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p-aa11bb22\",\"title\":\"dupe ignored\",\"status\":\"done\"}\n",
                "{\"ts\":\"t\",\"kind\":\"pbi\",\"event\":\"status\",\"id\":\"p-aa11bb22\",\"status\":\"in-flight\",\"feature\":\"f1\"}\n",
                "{\"ts\":\"t\",\"kind\":\"pbi\",\"event\":\"amend\",\"id\":\"p-aa11bb22\",\"cos\":\"c2\"}\n",
                "{\"ts\":\"t\",\"kind\":\"pbi\",\"event\":\"status\",\"id\":\"p-unknown\",\"status\":\"done\"}\n",
                "{\"ts\":\"t\",\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"\",\"title\":\"no id\"}\n",
                "{\"type\":\"friction\",\"title\":\"not a pbi\"}\n",
            ),
        );
        let fold = fold_pbis(tmp.path()).unwrap();
        assert!(fold.has_events);
        assert_eq!(fold.order, vec!["p-aa11bb22"]);
        let item = &fold.items["p-aa11bb22"];
        assert_eq!(item.title, "first");
        assert_eq!(item.status, "in-flight");
        assert_eq!(item.cos, "c2");
        assert_eq!(item.feature.as_deref(), Some("f1"));
        assert_eq!(
            jsjson::stringify(&pbi_value(item)),
            r#"{"id":"p-aa11bb22","title":"first","cos":"c2","status":"in-flight","feature":"f1"}"#
        );
    }

    #[test]
    fn folded_counts_shape_and_text() {
        let tmp = tempfile::tempdir().unwrap();
        write_backlog(
            tmp.path(),
            concat!(
                "{\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p-01\",\"title\":\"a\",\"status\":\"done\"}\n",
                "{\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p-02\",\"title\":\"b\",\"status\":\"in-flight\"}\n",
                "{\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p-03\",\"title\":\"c\"}\n",
            ),
        );
        let fold = fold_pbis(tmp.path()).unwrap();
        let counts = folded_counts(&fold);
        assert_eq!(
            jsjson::stringify(&Value::Object(counts.clone())),
            r#"{"proposed":1,"inFlight":1,"parked":0,"done":1,"declined":0,"total":3}"#
        );
        assert_eq!(counts_text(&counts), "PBI: 1 done / 1 in-flight / 1 proposed (3 total)");
    }

    #[test]
    fn legacy_counts_parse_table_by_status_column() {
        let tmp = tempfile::tempdir().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(
            docs.join("backlog.md"),
            concat!(
                "# Backlog\n\n",
                "| ID | Story | Status | Feature |\n",
                "|----|-------|--------|---------|\n",
                "| P1 | a | **done** | — |\n",
                "| P2 | b | in-flight | f |\n",
                "| P3 | c | proposed | f |\n",
                "| P4 | d | parked | f |\n", // not in the legacy 3-enum: uncounted
                "| bad row |\n",
            ),
        )
        .unwrap();
        let counts = legacy_counts(tmp.path()).unwrap();
        assert_eq!(
            jsjson::stringify(&Value::Object(counts.clone())),
            r#"{"proposed":1,"inFlight":1,"done":1,"total":3}"#
        );
        assert_eq!(counts_text(&counts), "PBI: 1 done / 1 in-flight / 1 proposed (3 total)");
        assert!(legacy_counts(&tmp.path().join("nope")).is_none());
    }

    #[test]
    fn whole_token_matching_is_boundary_aware() {
        let rows = |f: &str| vec![f.to_string()];
        assert!(matches_whole_token(&rows("si-1"), "si-1"));
        assert!(matches_whole_token(&rows("use SI-1 now"), "si-1")); // ci
        assert!(!matches_whole_token(&rows("si-10"), "si-1"));
        assert!(!matches_whole_token(&rows("si-1-extra"), "si-1"));
        assert!(!matches_whole_token(&rows("billing-export-v2"), "billing-export"));
        assert!(matches_whole_token(&rows("x billing-export."), "billing-export"));
    }

    #[test]
    fn finding_rows_filter_like_node() {
        use serde_json::json;
        assert!(is_finding_row(&json!({"kind":"friction","title":"t"})));
        assert!(is_finding_row(&json!({"type":"finding","title":"t"})));
        assert!(!is_finding_row(&json!({"kind":"pbi","type":"friction"})));
        assert!(!is_finding_row(&json!({"type":"proposal"})));
        assert!(!is_finding_row(&json!("nope")));
        assert!(!is_finding_row(&json!([1, 2])));
        // feature match is exact-modulo-case on the row's feature field
        let row = json!({"type":"friction","feature":"auth","title":"T","detail":"d"});
        assert!(matches_feature(&row, "auth"));
        assert!(!matches_feature(&row, "authz"));
        assert!(!matches_feature(&json!({"type":"friction"}), "auth"));
        // numeric features coerce through String(...)
        assert!(matches_feature(&json!({"type":"friction","feature":42}), "42"));
        // text terms are any-term substring over title+detail
        assert!(matches_text(&row, "big T"));
        assert!(!matches_text(&row, "zzz"));
        assert!(matches_text(&row, "")); // empty filter passes everything
    }

    #[test]
    fn token_key_camelcases_hyphens() {
        assert_eq!(token_key("in-flight"), "inFlight");
        assert_eq!(token_key("proposed"), "proposed");
        assert_eq!(token_key("done"), "done");
    }

    #[test]
    fn id_sort_guard_accepts_only_lowercase_hex_p_ids() {
        assert!(id_sort_safe("p-a1b2c3d4"));
        assert!(id_sort_safe("p-0f"));
        assert!(!id_sort_safe("P50")); // legacy id -> delegate
        assert!(!id_sort_safe("p-"));
        assert!(!id_sort_safe("p-A1B2C3D4"));
        assert!(!id_sort_safe("q-abcd"));
    }

    #[test]
    fn add_pbi_appends_event_and_respects_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        let item = match add_pbi(tmp.path(), Some("p-fixed001"), " Title ", " cos ", "proposed", Some(" feat "))
        {
            PbiAdd::Ok(i) => i,
            _ => panic!("expected Ok"),
        };
        assert_eq!(item.id, "p-fixed001");
        assert_eq!(item.title, "Title");
        assert_eq!(item.cos, "cos");
        assert_eq!(item.feature.as_deref(), Some("feat"));
        // duplicate requested id -> Delegate (Node's refusal)
        assert!(matches!(
            add_pbi(tmp.path(), Some("p-fixed001"), "x", "", "proposed", None),
            PbiAdd::Delegate
        ));
        // generated id: p-<8hex>, lands in the fold
        let generated = match add_pbi(tmp.path(), None, "gen", "", "done", None) {
            PbiAdd::Ok(i) => i,
            _ => panic!("expected Ok"),
        };
        assert!(id_sort_safe(&generated.id) && generated.id.len() == 10);
        let fold = fold_pbis(tmp.path()).unwrap();
        assert_eq!(fold.order.len(), 2);
        // the written event carries the frozen key order
        let raw = std::fs::read_to_string(backlog_jsonl_path(tmp.path())).unwrap();
        let first = raw.lines().next().unwrap();
        let parsed: Value = serde_json::from_str(first).unwrap();
        let keys: Vec<String> = parsed.as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, vec!["ts", "kind", "event", "id", "title", "status", "cos", "feature"]);
        // validation refusals delegate
        assert!(matches!(add_pbi(tmp.path(), None, "   ", "", "proposed", None), PbiAdd::Delegate));
        assert!(matches!(add_pbi(tmp.path(), None, "t", "", "bogus", None), PbiAdd::Delegate));
    }

    #[test]
    fn lock_busy_message_matches_node() {
        use serde_json::json;
        assert_eq!(
            lock_busy_message(&Some(json!({"pid": 123, "session": "s1", "ts": "T"}))),
            "backlog-pbi store lock busy: held by pid=123 session=s1 since T"
        );
        assert_eq!(
            lock_busy_message(&Some(json!({"pid": null}))),
            "backlog-pbi store lock busy: held by pid=unknown session=unknown since unknown"
        );
        assert_eq!(lock_busy_message(&None), "backlog-pbi store lock busy: held by unknown holder");
    }

    #[test]
    fn resolve_product_root_happy_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        // no config at all -> root
        assert_eq!(resolve_product_root(root).unwrap(), root);
        // empty string -> root
        std::fs::write(root.join(".bee").join("config.json"), r#"{"product_root":""}"#).unwrap();
        assert_eq!(resolve_product_root(root).unwrap(), root);
        // existing relative dir -> resolved
        std::fs::create_dir_all(root.join("product")).unwrap();
        std::fs::write(root.join(".bee").join("config.json"), r#"{"product_root":"product"}"#)
            .unwrap();
        assert_eq!(resolve_product_root(root).unwrap(), root.join("product"));
        // missing dir -> Node warns -> delegate
        std::fs::write(root.join(".bee").join("config.json"), r#"{"product_root":"missing"}"#)
            .unwrap();
        assert!(resolve_product_root(root).is_none());
        // non-string -> Node warns -> delegate
        std::fs::write(root.join(".bee").join("config.json"), r#"{"product_root":42}"#).unwrap();
        assert!(resolve_product_root(root).is_none());
    }

    // ── featureBacklogRank (R6, cells claim-next's cross-lane ordering) ────

    #[test]
    fn feature_backlog_rank_reads_the_feature_column_then_the_pbi_fold() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        // No backlog at all → an empty map (never a delegate).
        assert!(feature_backlog_rank(root).unwrap().is_empty());

        // Legacy table: status-grouped weight, stable within a group, and the
        // BEST-ranked occurrence of a feature wins.
        std::fs::create_dir_all(root.join("docs")).unwrap();
        std::fs::write(
            root.join("docs").join("backlog.md"),
            "# B\n\n| ID | Story | Status | Feature |\n|----|-------|--------|---------|\n\
             | P1 | a | done | gamma |\n\
             | P2 | b | proposed | beta |\n\
             | P3 | c | in-flight | alpha |\n\
             | P4 | d | in-flight | \u{2014} |\n\
             | P5 | e | done | alpha |\n",
        )
        .unwrap();
        let rank = feature_backlog_rank(root).unwrap();
        assert_eq!(rank.get("alpha"), Some(&0));
        assert_eq!(rank.get("beta"), Some(&2));
        assert_eq!(rank.get("gamma"), Some(&3));
        assert_eq!(rank.get("\u{2014}"), None, "the placeholder never claims a slug");

        // A table with no Feature column contributes nothing.
        std::fs::write(
            root.join("docs").join("backlog.md"),
            "| ID | Status |\n|----|--------|\n| P1 | done |\n",
        )
        .unwrap();
        assert!(feature_backlog_rank(root).unwrap().is_empty());

        // Fold-first: one kind:'pbi' event and the table is ignored entirely.
        write_backlog(
            root,
            "{\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p-0002\",\"status\":\"done\",\"feature\":\"zeta\"}\n\
             {\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p-0001\",\"status\":\"in-flight\",\"feature\":\"omega\"}\n\
             {\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"p-0003\",\"status\":\"in-flight\",\"feature\":\"zeta\"}\n",
        );
        let folded = feature_backlog_rank(root).unwrap();
        // weight asc, then id.localeCompare: p-0001(0) p-0003(1) p-0002(2).
        assert_eq!(folded.get("omega"), Some(&0));
        assert_eq!(folded.get("zeta"), Some(&1), "best-ranked occurrence wins");

        // An id outside the calibrated localeCompare alphabet delegates.
        write_backlog(
            root,
            "{\"kind\":\"pbi\",\"event\":\"add\",\"id\":\"P50\",\"status\":\"done\",\"feature\":\"z\"}\n",
        );
        assert!(feature_backlog_rank(root).is_none());
    }
}
