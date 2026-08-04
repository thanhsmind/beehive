// bee capture — native port of the capture verb group (bee.mjs
// handleCaptureAdd/List/Flush/Count + lib/capture.mjs).
//
// Verbs served natively (exact argv shapes only — see each probe):
//   capture count [--json]
//   capture list  [--json]
//   capture add   --outcome <v> [--did <v>] [--area <v>] [--files <v>]
//                 [--lane <v>] [--source <v>] [--json]
//   capture flush --id <v> [--into <v>] [--json]
// Nothing in this group is left permanently delegated; within the accepted
// shapes, ALL refusal/error paths still delegate to Node (missing/empty
// required flags, whitespace-only outcome, lane=high-risk, secret/injection
// pattern hits, unknown flush id) so the byte-exact error text stays Node's.
//
// Additional delegation triggers (None before any output/write):
//   - linked-worktree roots, corrupt manifest-hash cache
//   - stub/flush ids that are objects/arrays (JS Set identity semantics)
//   - `capture list` when any pending stub can't be re-emitted byte-exactly:
//     numbers outside the JS round-trip guard, a truthy non-array dids/files
//     (Node would throw .join on them), or an `at` outside the sortable-safe
//     ISO shape (Node sorts with localeCompare; same-shape ISO strings are
//     the provably-identical subset).
//
// DIVERGENCE NOTES (documented, unreachable-different for real bee data):
//   - stub ids come from a SHA-256/OS-entropy generator, not crypto.randomUUID
//     (format-identical v4; the draw is random on both runtimes).
//   - pendingCaptureStubs sorts with localeCompare; natively stubs sort by
//     byte order, guarded by the ISO-shape check above.

use super::feedback::{
    emit_success, has_injection, has_secret, iso_sortable, js_trim, js_truthy, now_iso,
    parse_shape, random_uuid_v4, read_jsonl, require_flag, value_js_safe, ParsedArgs,
};
use crate::fsutil::append_jsonl;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

fn queue_path(root: &Path) -> PathBuf {
    root.join(".bee").join("capture-queue.jsonl")
}

/// pendingCaptureStubs (lib/capture.mjs): stub events minus flushed ids.
/// None => delegate (an id whose JS Set semantics — object identity — a
/// serialized key can't reproduce).
///
/// CUTOVER (2026-08-01): a queue row only V8's JSON.parse could read (a lone
/// surrogate escape, ...) used to delegate the whole command. read_jsonl now
/// skips it, which is precisely what lib/fsutil.mjs readJsonl did with every
/// other corrupt line — the queue is read fail-open either way.
fn pending_stubs(root: &Path) -> Option<Vec<Value>> {
    let read = read_jsonl(&queue_path(root));
    // JS Set.has uses SameValueZero: strings/numbers/bools compare by value
    // (the compact-serialized key reproduces that exactly); objects/arrays
    // compare by identity, which serialization can NOT reproduce — delegate.
    let mut flushed: Vec<String> = Vec::new();
    let mut stubs: Vec<Value> = Vec::new();
    for event in &read.rows {
        let Value::Object(m) = event else { continue };
        let id = m.get("id");
        let id_truthy = id.map(js_truthy).unwrap_or(false);
        let kind = m.get("kind").and_then(Value::as_str);
        if kind == Some("flush") && id_truthy {
            let id = id.unwrap();
            if matches!(id, Value::Object(_) | Value::Array(_)) {
                return None;
            }
            flushed.push(jsjson::stringify(id));
        } else if kind == Some("stub") && id_truthy {
            if matches!(id.unwrap(), Value::Object(_) | Value::Array(_)) {
                return None;
            }
            stubs.push(event.clone());
        }
    }
    Some(
        stubs
            .into_iter()
            .filter(|s| {
                let key = jsjson::stringify(s.get("id").unwrap_or(&Value::Null));
                !flushed.contains(&key)
            })
            .collect(),
    )
}

/// The pending list in emission order (sorted by `at`). None => delegate
/// (non-sortable-safe `at` values — Node uses localeCompare).
fn pending_sorted(root: &Path) -> Option<Vec<Value>> {
    let mut stubs = pending_stubs(root)?;
    for stub in &stubs {
        let at = stub.get("at").map(jsjson::js_to_string).unwrap_or_else(|| "undefined".into());
        if !iso_sortable(&at) {
            return None;
        }
    }
    // Stable sort == JS Array.prototype.sort stability (spec since ES2019).
    stubs.sort_by(|a, b| {
        let ka = a.get("at").map(jsjson::js_to_string).unwrap_or_default();
        let kb = b.get("at").map(jsjson::js_to_string).unwrap_or_default();
        ka.cmp(&kb)
    });
    Some(stubs)
}

/// JS template-literal coercion for a possibly-absent field.
fn coerce_field(stub: &Value, name: &str) -> String {
    match stub.get(name) {
        Some(v) => jsjson::js_to_string(v),
        None => "undefined".to_string(),
    }
}

/// Array.prototype.join(', ') — null elements render empty (undefined can't
/// appear in parsed JSON).
fn join_list(items: &[Value]) -> String {
    items
        .iter()
        .map(|v| match v {
            Value::Null => String::new(),
            other => jsjson::js_to_string(other),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// A truthy dids/files that is not an array would make Node throw on .join —
/// delegate those. Returns the array when the field should render.
fn renderable_list<'a>(stub: &'a Value, name: &str) -> Result<Option<&'a Vec<Value>>, ()> {
    match stub.get(name) {
        None => Ok(None),
        Some(v) if !js_truthy(v) => Ok(None),
        Some(Value::Array(a)) => Ok(if a.is_empty() { None } else { Some(a) }),
        Some(_) => Err(()),
    }
}

/// formatCaptureStub (bee.mjs). Err(()) => delegate.
fn format_stub(stub: &Value) -> Result<String, ()> {
    let marker = if stub.get("source").and_then(Value::as_str) == Some("mined") {
        " [mined]"
    } else {
        ""
    };
    let mut parts = vec![format!(
        "[{}] {}{marker} (id {})",
        coerce_field(stub, "at"),
        coerce_field(stub, "outcome"),
        coerce_field(stub, "id"),
    )];
    if let Some(dids) = renderable_list(stub, "dids")? {
        parts.push(format!("  decisions: {}", join_list(dids)));
    }
    if stub.get("area").map(js_truthy).unwrap_or(false) {
        parts.push(format!("  area: {}", coerce_field(stub, "area")));
    }
    if let Some(files) = renderable_list(stub, "files")? {
        parts.push(format!("  files: {}", join_list(files)));
    }
    if stub.get("source").map(js_truthy).unwrap_or(false) {
        parts.push(format!("  source: {}", coerce_field(stub, "source")));
    }
    Ok(parts.join("\n"))
}

/// normalizeList (lib/capture.mjs) for the string branch.
fn normalize_list(value: Option<&str>) -> Vec<String> {
    match value {
        Some(s) if !js_trim(s).is_empty() => s
            .split(',')
            .map(js_trim)
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

struct Ctx {
    root: PathBuf,
    drift: crate::registry::Drift,
}

/// Root + manifest-drift preamble shared by all four verbs. Err(code) is the
/// no-root exit; Ok(None) delegates.
fn preamble(cmd: &str, pre_json: bool, t0: Instant) -> Result<Option<Ctx>, ExitCode> {
    let Ok(cwd) = std::env::current_dir() else { return Ok(None) };
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Err(emit_unsupported_root(&cwd, cmd, pre_json, t0, &why))
        }
        Roots::None => return Err(emit_no_root_error(&cwd, cmd, pre_json, t0)),
    };
    let drift = check_manifest_drift(&root);
    Ok(Some(Ctx { root, drift }))
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "capture" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let rest = &args[2..];
    match verb {
        "count" => run_count(parse_shape(rest, &[])?, t0),
        "list" => run_list(parse_shape(rest, &[])?, t0),
        "add" => run_add(
            parse_shape(rest, &["outcome", "did", "area", "files", "lane", "source"])?,
            t0,
        ),
        "flush" => run_flush(parse_shape(rest, &["id", "into"])?, t0),
        _ => None,
    }
}

fn run_count(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let ctx = match preamble("capture count", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let count = pending_stubs(&ctx.root)?.len();
    let mut result = Map::new();
    result.insert("count".into(), Value::from(count));
    let text = format!("{count} pending capture stub(s).");
    Some(emit_success(&ctx.root, "capture count", parsed.json, &ctx.drift, &Value::Object(result), &text, t0))
}

fn run_list(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let ctx = match preamble("capture list", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let stubs = pending_sorted(&ctx.root)?;
    // Byte-exact re-emission guard for rows read from disk.
    if !stubs.iter().all(value_js_safe) {
        return None;
    }
    let mut lines = Vec::new();
    for stub in &stubs {
        match format_stub(stub) {
            Ok(line) => lines.push(line),
            Err(()) => return None,
        }
    }
    let text = if lines.is_empty() {
        "Capture queue is empty.".to_string()
    } else {
        lines.join("\n")
    };
    let mut result = Map::new();
    result.insert("count".into(), Value::from(stubs.len()));
    result.insert("stubs".into(), Value::Array(stubs));
    Some(emit_success(&ctx.root, "capture list", parsed.json, &ctx.drift, &Value::Object(result), &text, t0))
}

fn run_add(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    // ── pure-argv validation first (bee.mjs handleCaptureAdd +
    // lib/capture.mjs addCaptureStub) — every refusal delegates. ────────────
    let outcome_raw = require_flag(&parsed, "outcome")?; // missing/'' -> Node error
    let outcome = js_trim(outcome_raw);
    if outcome.is_empty() {
        return None; // addCaptureStub: outcome text is required
    }
    // `flags.x ? String(flags.x) : null` — '' is falsy.
    let opt = |name: &str| parsed.flags.get(name).filter(|v| !v.is_empty()).map(|v| v.as_str());
    let lane_raw = opt("lane");
    if lane_raw == Some("high-risk") {
        return None; // high-risk settlements never queue — Node's refusal
    }
    if has_secret(outcome) || has_injection(outcome) {
        return None; // assertSafeContent('outcome') — Node's refusal text
    }
    let area = opt("area").map(js_trim).filter(|a| !a.is_empty());
    if let Some(area) = area {
        if has_secret(area) || has_injection(area) {
            return None; // assertSafeContent('area')
        }
    }
    let dids = normalize_list(opt("did"));
    let files = normalize_list(opt("files"));
    let lane = lane_raw.map(js_trim).filter(|l| !l.is_empty());
    let source = opt("source").map(js_trim).filter(|s| !s.is_empty());

    let ctx = match preamble("capture add", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };

    // Stub key order: kind, id, at, outcome, dids, area, files, lane[, source].
    let mut stub = Map::new();
    stub.insert("kind".into(), Value::String("stub".into()));
    stub.insert("id".into(), Value::String(random_uuid_v4()));
    stub.insert("at".into(), Value::String(now_iso()));
    stub.insert("outcome".into(), Value::String(outcome.to_string()));
    stub.insert("dids".into(), Value::Array(dids.into_iter().map(Value::String).collect()));
    stub.insert("area".into(), area.map(|a| Value::String(a.to_string())).unwrap_or(Value::Null));
    stub.insert("files".into(), Value::Array(files.into_iter().map(Value::String).collect()));
    stub.insert("lane".into(), lane.map(|l| Value::String(l.to_string())).unwrap_or(Value::Null));
    if let Some(source) = source {
        stub.insert("source".into(), Value::String(source.to_string()));
    }
    let stub = Value::Object(stub);
    // Append failure: no line was written — delegate so Node owns the error.
    if append_jsonl(&queue_path(&ctx.root), &stub).is_err() {
        return None;
    }
    let id = stub.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
    let text = format!(
        "Queued capture stub {id}. Flush via bee-capturing at wrap-up, before compact/clear, or next session (decision 0017)."
    );
    Some(emit_success(&ctx.root, "capture add", parsed.json, &ctx.drift, &stub, &text, t0))
}

fn run_flush(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let id_raw = require_flag(&parsed, "id")?; // missing/'' -> Node error
    let id = js_trim(id_raw);
    if id.is_empty() {
        return None; // flushCaptureStub: stub id is required
    }
    let into = parsed
        .flags
        .get("into")
        .filter(|v| !v.is_empty()) // `flags.into ? String(flags.into) : null`
        .map(|v| js_trim(v))
        .filter(|v| !v.is_empty());

    let ctx = match preamble("capture flush", parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let pending = pending_stubs(&ctx.root)?;
    let found = pending
        .iter()
        .any(|s| s.get("id").and_then(Value::as_str) == Some(id));
    if !found {
        // CUTOVER FIX: this used to `return None` so Node could own the
        // refusal bytes. With Node gone that returned the caller to the
        // dispatcher's end-of-line, which answers "this command does not
        // exist" — for the everyday case of a stub id that has already been
        // flushed. A missing target is the VERB's error, and it says so.
        let msg = format!(
            "bee capture flush: no pending capture stub with id {id}. \
             FIX: `bee capture list --json` lists the ids still pending."
        );
        return Some(crate::verbs::feedback::emit_error(
            &ctx.root,
            "capture flush",
            parsed.json,
            &msg,
            t0,
        ));
    }

    // Record key order: kind, id, at, into.
    let mut record = Map::new();
    record.insert("kind".into(), Value::String("flush".into()));
    record.insert("id".into(), Value::String(id.to_string()));
    record.insert("at".into(), Value::String(now_iso()));
    record.insert(
        "into".into(),
        into.map(|v| Value::String(v.to_string())).unwrap_or(Value::Null),
    );
    let record = Value::Object(record);
    if append_jsonl(&queue_path(&ctx.root), &record).is_err() {
        return None;
    }
    let into_suffix = match record.get("into") {
        Some(Value::String(s)) => format!(" into {s}"),
        _ => String::new(),
    };
    let text = format!("Flushed stub {id}{into_suffix}.");
    Some(emit_success(&ctx.root, "capture flush", parsed.json, &ctx.drift, &record, &text, t0))
}

// The lock module is not used here on purpose: neither lib/capture.mjs writer
// runs under withStoreLock — both are bare appendJsonl calls in Node too.

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_queue(root: &Path, lines: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(queue_path(root), lines).unwrap();
    }

    #[test]
    fn pending_folds_stub_and_flush_events() {
        let tmp = tempfile::tempdir().unwrap();
        write_queue(
            tmp.path(),
            concat!(
                "{\"kind\":\"stub\",\"id\":\"a\",\"at\":\"2026-01-02T00:00:00.000Z\",\"outcome\":\"second\",\"dids\":[],\"area\":null,\"files\":[],\"lane\":null}\n",
                "{\"kind\":\"stub\",\"id\":\"b\",\"at\":\"2026-01-01T00:00:00.000Z\",\"outcome\":\"first\",\"dids\":[\"0001\"],\"area\":\"x\",\"files\":[\"f1\",\"f2\"],\"lane\":\"tiny\",\"source\":\"mined\"}\n",
                "{\"kind\":\"stub\",\"id\":\"c\",\"at\":\"2026-01-03T00:00:00.000Z\",\"outcome\":\"gone\",\"dids\":[],\"area\":null,\"files\":[],\"lane\":null}\n",
                "{\"kind\":\"flush\",\"id\":\"c\",\"at\":\"2026-01-04T00:00:00.000Z\",\"into\":null}\n",
                "{\"noise\":true}\n",
            ),
        );
        let pending = pending_stubs(tmp.path()).unwrap();
        assert_eq!(pending.len(), 2);
        let sorted = pending_sorted(tmp.path()).unwrap();
        let ids: Vec<_> = sorted.iter().map(|s| s.get("id").unwrap().as_str().unwrap()).collect();
        assert_eq!(ids, vec!["b", "a"]); // oldest first
    }

    #[test]
    fn format_stub_matches_node_layout() {
        let stub = json!({
            "kind": "stub",
            "id": "b",
            "at": "2026-01-01T00:00:00.000Z",
            "outcome": "first",
            "dids": ["0001", "0002"],
            "area": "specs",
            "files": ["f1", "f2"],
            "lane": "tiny",
            "source": "mined"
        });
        assert_eq!(
            format_stub(&stub).unwrap(),
            "[2026-01-01T00:00:00.000Z] first [mined] (id b)\n  decisions: 0001, 0002\n  area: specs\n  files: f1, f2\n  source: mined"
        );
        // Minimal stub: only the head line.
        let bare = json!({"kind":"stub","id":"a","at":"t","outcome":"o","dids":[],"area":null,"files":[],"lane":null});
        assert_eq!(format_stub(&bare).unwrap(), "[t] o (id a)");
        // A truthy non-array dids would crash Node's .join — delegate.
        let bad = json!({"kind":"stub","id":"a","at":"t","outcome":"o","dids":"oops"});
        assert!(format_stub(&bad).is_err());
    }

    #[test]
    fn pending_sorted_delegates_on_non_iso_at() {
        let tmp = tempfile::tempdir().unwrap();
        write_queue(
            tmp.path(),
            "{\"kind\":\"stub\",\"id\":\"a\",\"at\":\"yesterday\",\"outcome\":\"o\"}\n",
        );
        assert_eq!(pending_stubs(tmp.path()).unwrap().len(), 1); // count still native
        assert!(pending_sorted(tmp.path()).is_none()); // list delegates
    }

    #[test]
    fn object_ids_delegate_for_set_identity() {
        let tmp = tempfile::tempdir().unwrap();
        write_queue(tmp.path(), "{\"kind\":\"flush\",\"id\":{\"x\":1}}\n");
        assert!(pending_stubs(tmp.path()).is_none());
    }

    #[test]
    fn numeric_and_string_ids_stay_distinct() {
        let tmp = tempfile::tempdir().unwrap();
        write_queue(
            tmp.path(),
            concat!(
                "{\"kind\":\"stub\",\"id\":5,\"at\":\"2026-01-01T00:00:00.000Z\",\"outcome\":\"n\"}\n",
                "{\"kind\":\"flush\",\"id\":\"5\"}\n", // string \"5\" flushes nothing
            ),
        );
        assert_eq!(pending_stubs(tmp.path()).unwrap().len(), 1);
    }

    #[test]
    fn normalize_list_splits_trims_filters() {
        assert_eq!(normalize_list(Some("a, b ,,c ")), vec!["a", "b", "c"]);
        assert_eq!(normalize_list(Some("   ")), Vec::<String>::new());
        assert_eq!(normalize_list(None), Vec::<String>::new());
    }

    #[test]
    fn add_stub_shape_matches_node_key_order() {
        // Mirrors run_add's construction without the CLI plumbing.
        let mut stub = Map::new();
        stub.insert("kind".into(), Value::String("stub".into()));
        stub.insert("id".into(), Value::String("fixed".into()));
        stub.insert("at".into(), Value::String("t".into()));
        stub.insert("outcome".into(), Value::String("o".into()));
        stub.insert("dids".into(), json!(["1"]));
        stub.insert("area".into(), Value::Null);
        stub.insert("files".into(), json!([]));
        stub.insert("lane".into(), Value::Null);
        assert_eq!(
            jsjson::stringify(&Value::Object(stub)),
            r#"{"kind":"stub","id":"fixed","at":"t","outcome":"o","dids":["1"],"area":null,"files":[],"lane":null}"#
        );
    }
}
