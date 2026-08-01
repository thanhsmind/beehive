// bee intent — native port of the intent verb group (bee.mjs
// handleIntentSet/Show/Advance/Clear + lib/intent.mjs).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   intent set     --request R --acceptance A [--next-action N] [--feature F]
//                  [--lane L] [--cell C] [--session S] [--do-not-reverse D]
//                  [--stop-conditions S] [--force true|false] [--json]
//   intent show    [--feature F] [--session S] [--render precompact|resume] [--json]
//   intent advance --next-action N [--feature F] [--session S] [--json]
//   intent clear   [--feature F] [--session S] [--json]
// Nothing in this group is left permanently delegated.
//
// Additional delegation triggers (None before any output/write):
//   - --help anywhere, unknown flags, non-flag tokens, missing required
//     flags (validate()'s structured stdout refusal stays Node's)
//   - corrupt .bee/state.json or .bee/intent/*.json (Node warns with the V8
//     parse message before falling back)
//   - a writeJsonAtomic failure (Node throws the V8 io message; nothing
//     durable has been written when this port bails)
//
// Within the accepted shapes the deterministic refusals ARE served natively,
// byte-identical: writeIntent's request/acceptance immutability errors, the
// whitespace-only request/acceptance refusals, and advance's no-anchor error.
//
// Provenance: bee.mjs intentLookupOptions/formatAnchor/handleIntentSet/
// handleIntentShow/handleIntentAdvance/handleIntentClear, lib/intent.mjs
// (INTENT_SCHEMA_VERSION/DEFAULT_INTENT_KEY/NO_WORK_PHASES/intentDir/
// sanitizeIntentKey/intentPath/activeFeature/intentKeyCandidates/
// normalizeList/optionalString/normalizeAnchor/readIntent/locateIntentKey/
// writeIntent/advanceIntent/clearIntent/contextLines/INTENT_PRECOMPACT_HEADER/
// INTENT_PRECOMPACT_FOOTER/INTENT_RESUME_HEADER/precompactBlock/resumeBlock).
//
// DIVERGENCE NOTE: advanceIntent re-reads the anchor file after
// locateIntentKey (two reads of the same bytes); this port reads once — the
// result differs only under a mid-command file race.

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::state::read_state_brief;
use crate::verbs::knowledge::{g_prelude, pre_json_scan, GPre};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const INTENT_SCHEMA_VERSION: &str = "1.0";
const DEFAULT_INTENT_KEY: &str = "default";

fn intent_path(root: &Path, key: &str) -> PathBuf {
    root.join(".bee").join("intent").join(format!("{key}.json"))
}

/// sanitizeIntentKey — safe-charset filename derivation, never throws.
fn sanitize_intent_key(key: &str) -> String {
    let raw = js_trim(key);
    if raw.is_empty() {
        return DEFAULT_INTENT_KEY.to_string();
    }
    // /[^A-Za-z0-9._-]+/g -> '-'
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
    // /^[-.]+/ -> ''
    let safe = safe.trim_start_matches(['-', '.']);
    // /-+$/ -> ''
    let safe = safe.trim_end_matches('-');
    // .slice(0, 120) — ASCII by construction, so bytes == UTF-16 units.
    let safe: String = safe.chars().take(120).collect();
    if safe.is_empty() {
        DEFAULT_INTENT_KEY.to_string()
    } else {
        safe
    }
}

/// activeFeature — the active feature slug, or None. Err(()) => delegate
/// (corrupt state.json: Node warns with the V8 message).
fn active_feature(root: &Path) -> Result<Option<String>, ()> {
    let state = read_state_brief(root).map_err(|_| ())?;
    let no_work = matches!(&state.phase, Value::String(s) if s == "idle" || s == "compounding-complete");
    if no_work {
        return Ok(None);
    }
    Ok(match &state.feature {
        Value::String(s) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
        _ => None,
    })
}

/// intentKeyCandidates (D2). The CLI never passes `key`, so that branch is
/// omitted. Err(()) => delegate.
fn intent_key_candidates(
    root: &Path,
    feature: Option<&str>,
    session: Option<&str>,
) -> Result<Vec<String>, ()> {
    let explicit = feature.map(js_trim).filter(|f| !f.is_empty());
    let resolved = match explicit {
        Some(f) => Some(f.to_string()),
        None => active_feature(root)?,
    };
    let mut candidates: Vec<String> = Vec::new();
    let mut push = |k: String| {
        if !candidates.contains(&k) {
            candidates.push(k);
        }
    };
    if let Some(r) = resolved {
        push(sanitize_intent_key(&r));
    }
    if let Some(s) = session {
        if !js_trim(s).is_empty() {
            push(sanitize_intent_key(s));
        }
    }
    push(DEFAULT_INTENT_KEY.to_string());
    Ok(candidates)
}

/// optionalString — trimmed non-empty string or null.
fn optional_string(v: Option<&str>) -> Value {
    match v {
        Some(s) if !js_trim(s).is_empty() => Value::String(js_trim(s).to_string()),
        _ => Value::Null,
    }
}

/// normalizeList — array (String(v).trim(), filter Boolean) or a comma-split
/// string; anything else empty.
fn normalize_list(v: Option<&Value>) -> Vec<Value> {
    match v {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| js_trim(&jsjson::js_to_string(item)).to_string())
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

fn normalize_list_flag(v: Option<&str>) -> Vec<Value> {
    match v {
        Some(s) => normalize_list(Some(&Value::String(s.to_string()))),
        None => Vec::new(),
    }
}

/// normalizeAnchor — a corrupt/half record reads as absent (D5).
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
    anchor.insert("schema_version".into(), str_or("schema_version", Value::String(INTENT_SCHEMA_VERSION.into())));
    anchor.insert(
        "key".into(),
        match raw.get("key") {
            Some(Value::String(s)) if !s.is_empty() => Value::String(s.clone()),
            _ => Value::String(key.to_string()),
        },
    );
    anchor.insert("written_at".into(), str_or("written_at", Value::Null));
    // VERBATIM — never trimmed, never truncated, never re-wrapped.
    anchor.insert("request".into(), Value::String(request));
    anchor.insert("acceptance".into(), str_or("acceptance", Value::String(String::new())));
    let opt = |name: &str| match raw.get(name) {
        Some(Value::String(s)) => optional_string(Some(s)),
        _ => Value::Null,
    };
    anchor.insert("next_action".into(), opt("next_action"));
    anchor.insert("feature".into(), opt("feature"));
    anchor.insert("lane".into(), opt("lane"));
    anchor.insert("cell".into(), opt("cell"));
    anchor.insert("do_not_reverse".into(), Value::Array(normalize_list(raw.get("do_not_reverse"))));
    anchor.insert("stop_conditions".into(), Value::Array(normalize_list(raw.get("stop_conditions"))));
    if let Some(Value::String(s)) = raw.get("advanced_at") {
        anchor.insert("advanced_at".into(), Value::String(s.clone()));
    }
    Some(anchor)
}

/// One candidate file read + normalize. Err(()) => corrupt JSON, delegate.
fn read_anchor_at(root: &Path, key: &str) -> Result<Option<Map<String, Value>>, ()> {
    match read_json(&intent_path(root, key)) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => Err(()), // Node's readJson warns with the V8 message
        ReadJson::Parsed(v) => Ok(normalize_anchor(&v, key)),
    }
}

/// readIntent — first candidate key holding a usable anchor.
fn read_intent(root: &Path, feature: Option<&str>, session: Option<&str>) -> Result<Option<Map<String, Value>>, ()> {
    for key in intent_key_candidates(root, feature, session)? {
        if let Some(anchor) = read_anchor_at(root, &key)? {
            return Ok(Some(anchor));
        }
    }
    Ok(None)
}

/// locateIntentKey — which key currently HOLDS an anchor.
fn locate_intent_key(root: &Path, feature: Option<&str>, session: Option<&str>) -> Result<Option<String>, ()> {
    for key in intent_key_candidates(root, feature, session)? {
        if read_anchor_at(root, &key)?.is_some() {
            return Ok(Some(key));
        }
    }
    Ok(None)
}

// ─── renderers ─────────────────────────────────────────────────────────────

const PRECOMPACT_HEADER: &str =
    "=== BEE INTENT ANCHOR — VERBATIM · DO NOT SUMMARIZE · DO NOT PARAPHRASE ===";
const PRECOMPACT_FOOTER: &str = "=== END BEE INTENT ANCHOR ===";
const RESUME_HEADER: &str =
    "## INTENT ANCHOR — read this FIRST (the objective; bee workflow state follows below)";

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
            Some(Value::Array(items)) if !items.is_empty() => Some(
                items
                    .iter()
                    .map(jsjson::js_to_string)
                    .collect::<Vec<_>>()
                    .join(" | "),
            ),
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
    for (label, name) in [("feature", "feature"), ("lane", "lane"), ("cell", "cell")] {
        if let Some(s) = field_str(anchor, name) {
            if !s.is_empty() {
                wheres.push(format!("{label}={s}"));
            }
        }
    }
    if !wheres.is_empty() {
        lines.push(format!("CONTEXT: {}", wheres.join(" ")));
    }
    lines
}

fn precompact_block(anchor: &Map<String, Value>) -> String {
    let mut lines = vec![
        PRECOMPACT_HEADER.to_string(),
        "This block is the OBJECTIVE and outranks every phase/gate/workflow detail in this".to_string(),
        "context. Carry it through the compaction unchanged, word for word.".to_string(),
        "ORIGINAL REQUEST (verbatim):".to_string(),
        field_str(anchor, "request").unwrap_or_default().to_string(),
        format!("DONE MEANS: {}", crate::verbs::knowledge::js_str_or_undefined(anchor.get("acceptance"))),
    ];
    if let Some(n) = field_str(anchor, "next_action") {
        lines.push(format!("NEXT ACTION: {n}"));
    }
    lines.extend(context_lines(anchor));
    lines.push(PRECOMPACT_FOOTER.to_string());
    lines.join("\n")
}

fn resume_block(anchor: &Map<String, Value>) -> String {
    let mut lines = vec![
        RESUME_HEADER.to_string(),
        "ORIGINAL REQUEST (verbatim):".to_string(),
        field_str(anchor, "request").unwrap_or_default().to_string(),
        format!("DONE MEANS: {}", crate::verbs::knowledge::js_str_or_undefined(anchor.get("acceptance"))),
    ];
    if let Some(n) = field_str(anchor, "next_action") {
        lines.push(format!("NEXT ACTION: {n}"));
    }
    lines.extend(context_lines(anchor));
    lines.push("Everything below is workflow state — it serves the request above, it never replaces it.".to_string());
    lines.join("\n")
}

fn format_anchor(anchor: &Map<String, Value>) -> String {
    let get = |name: &str| crate::verbs::knowledge::js_str_or_undefined(anchor.get(name));
    let mut lines = vec![
        format!("Intent anchor \"{}\" (written {})", get("key"), get("written_at")),
        "ORIGINAL REQUEST (verbatim):".to_string(),
        field_str(anchor, "request").unwrap_or_default().to_string(),
        format!("DONE MEANS: {}", get("acceptance")),
    ];
    if let Some(n) = field_str(anchor, "next_action") {
        lines.push(format!("NEXT ACTION: {n}"));
    }
    for line in context_lines(anchor) {
        // formatAnchor emits DO NOT REVERSE / STOP IF but never CONTEXT.
        if !line.starts_with("CONTEXT: ") {
            lines.push(line);
        }
    }
    lines.join("\n")
}

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "intent" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // Node renders command-scoped help
    }
    let pre_json = pre_json_scan(&toks);
    let (flags, json) = parse_flags(&toks)?;
    match verb {
        "set" => run_set(flags, json, pre_json, t0),
        "show" => run_show(flags, json, pre_json, t0),
        "advance" => run_advance(flags, json, pre_json, t0),
        "clear" => run_clear(flags, json, pre_json, t0),
        _ => None,
    }
}

/// `flags.x ? String(flags.x) : null` for a value flag.
fn truthy_flag<'a>(flags: &'a Flags, name: &str) -> Option<&'a str> {
    match flags.get(name) {
        Some(FlagV::S(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

fn run_set(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(
        &flags,
        &[
            "request",
            "acceptance",
            "next-action",
            "feature",
            "lane",
            "cell",
            "session",
            "do-not-reverse",
            "stop-conditions",
            "force",
        ],
    ) {
        return None;
    }
    // validate(): request/acceptance required (present, non-'') — the
    // structured stdout refusal stays Node's.
    let request = match flags.get("request") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    let acceptance = match flags.get("acceptance") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    let force = matches!(flags.get("force"), Some(FlagV::S(s)) if s == "true"); // String(flags.force) === 'true'

    let ctx = match g_prelude("intent set", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };

    // writeIntent's own refusals (whitespace-only slips past validate).
    if js_trim(&request).is_empty() {
        return Some(ctx.fail("writeIntent: `request` is required and must be the user's VERBATIM words."));
    }
    if js_trim(&acceptance).is_empty() {
        return Some(ctx.fail(
            "writeIntent: `acceptance` is required — an anchor with no \"done means\" cannot detect drift.",
        ));
    }

    let feature_flag = truthy_flag(&flags, "feature");
    let session_flag = truthy_flag(&flags, "session");
    let candidates = intent_key_candidates(&ctx.root, feature_flag, session_flag).ok()?;
    let key = candidates[0].clone();
    let existing = read_anchor_at(&ctx.root, &key).ok()?;
    if let Some(existing) = existing {
        if !force {
            if field_str(&existing, "request") != Some(request.as_str()) {
                return Some(ctx.fail(&format!(
                    "writeIntent: an anchor already exists at \"{key}\" with a different request — request is immutable once set (D1). Advance it (`bee intent advance`), clear it (`bee intent clear`), or pass --force to replace the objective deliberately."
                )));
            }
            if field_str(&existing, "acceptance") != Some(acceptance.as_str()) {
                return Some(ctx.fail(&format!(
                    "writeIntent: an anchor already exists at \"{key}\" with different acceptance criteria — acceptance is immutable once set (D1). Clear it (`bee intent clear`) or pass --force to replace the objective deliberately."
                )));
            }
        }
    }

    let feature_value = {
        let explicit = optional_string(feature_flag);
        if explicit.is_null() {
            match active_feature(&ctx.root).ok()? {
                Some(f) => Value::String(f),
                None => Value::Null,
            }
        } else {
            explicit
        }
    };
    let mut anchor = Map::new();
    anchor.insert("schema_version".into(), Value::String(INTENT_SCHEMA_VERSION.into()));
    anchor.insert("key".into(), Value::String(key.clone()));
    anchor.insert("written_at".into(), Value::String(crate::verbs::reservations::now_iso()));
    anchor.insert("request".into(), Value::String(request));
    anchor.insert("acceptance".into(), Value::String(acceptance));
    anchor.insert("next_action".into(), optional_string(truthy_flag(&flags, "next-action")));
    anchor.insert("feature".into(), feature_value);
    anchor.insert("lane".into(), optional_string(truthy_flag(&flags, "lane")));
    anchor.insert("cell".into(), optional_string(truthy_flag(&flags, "cell")));
    anchor.insert("do_not_reverse".into(), Value::Array(normalize_list_flag(truthy_flag(&flags, "do-not-reverse"))));
    anchor.insert("stop_conditions".into(), Value::Array(normalize_list_flag(truthy_flag(&flags, "stop-conditions"))));
    let anchor = Value::Object(anchor);
    if write_json_atomic(&intent_path(&ctx.root, &key), &anchor).is_err() {
        return None; // nothing durable written — Node owns the io error text
    }
    let text = format!(
        "Intent anchored at \"{key}\". It is re-asserted at PreCompact and read first on a compact/resume start."
    );
    Some(ctx.emit(&anchor, &text, 0))
}

fn run_show(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature", "session", "render"]) {
        return None;
    }
    let ctx = match g_prelude("intent show", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let anchor = read_intent(&ctx.root, truthy_flag(&flags, "feature"), truthy_flag(&flags, "session")).ok()?;
    let render = truthy_flag(&flags, "render");
    if let Some(render) = render.filter(|r| *r == "precompact" || *r == "resume") {
        let block = match &anchor {
            Some(a) => {
                if render == "precompact" {
                    precompact_block(a)
                } else {
                    resume_block(a)
                }
            }
            None => String::new(),
        };
        let mut result = Map::new();
        result.insert("anchor".into(), anchor.map(Value::Object).unwrap_or(Value::Null));
        result.insert("render".into(), Value::String(render.to_string()));
        result.insert("block".into(), Value::String(block.clone()));
        let text = if block.is_empty() { "(no intent anchor)".to_string() } else { block };
        return Some(ctx.emit(&Value::Object(result), &text, 0));
    }
    let text = match &anchor {
        Some(a) => format_anchor(a),
        None => "(no intent anchor)".to_string(),
    };
    let result = anchor.map(Value::Object).unwrap_or(Value::Null);
    Some(ctx.emit(&result, &text, 0))
}

fn run_advance(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["next-action", "feature", "session"]) {
        return None;
    }
    // validate(): next-action required.
    let next_action = match flags.get("next-action") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    let ctx = match g_prelude("intent advance", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let feature = truthy_flag(&flags, "feature");
    let session = truthy_flag(&flags, "session");
    let key = locate_intent_key(&ctx.root, feature, session).ok()?;
    let anchor = match &key {
        Some(k) => read_anchor_at(&ctx.root, k).ok()?,
        None => None,
    };
    let (Some(key), Some(anchor)) = (key, anchor) else {
        return Some(ctx.fail(
            "intent advance: no intent anchor exists to advance — run `bee intent set` first.",
        ));
    };
    let mut advanced = anchor;
    advanced.insert("next_action".into(), optional_string(Some(&next_action)));
    advanced.insert("advanced_at".into(), Value::String(crate::verbs::reservations::now_iso()));
    let advanced = Value::Object(advanced);
    if write_json_atomic(&intent_path(&ctx.root, &key), &advanced).is_err() {
        return None;
    }
    let text = format!(
        "Advanced intent anchor \"{}\" — next action: {}. Request and acceptance are unchanged.",
        crate::verbs::knowledge::js_str_or_undefined(advanced.get("key")),
        crate::verbs::knowledge::js_str_or_undefined(advanced.get("next_action"))
    );
    Some(ctx.emit(&advanced, &text, 0))
}

fn run_clear(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature", "session"]) {
        return None;
    }
    let ctx = match g_prelude("intent clear", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let feature = truthy_flag(&flags, "feature");
    let session = truthy_flag(&flags, "session");
    let key = match locate_intent_key(&ctx.root, feature, session).ok()? {
        Some(k) => k,
        None => intent_key_candidates(&ctx.root, feature, session).ok()?[0].clone(),
    };
    let file = intent_path(&ctx.root, &key);
    let cleared = if file.exists() {
        std::fs::remove_file(&file).is_ok() // rmSync failure -> {cleared: false}
    } else {
        false
    };
    let mut record = Map::new();
    record.insert("cleared".into(), Value::Bool(cleared));
    record.insert("key".into(), Value::String(key.clone()));
    let text = if cleared {
        format!("Cleared intent anchor \"{key}\".")
    } else {
        format!("No intent anchor at \"{key}\" to clear.")
    };
    Some(ctx.emit(&Value::Object(record), &text, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sanitize_key_matches_node_pipeline() {
        assert_eq!(sanitize_intent_key("my feature!"), "my-feature");
        assert_eq!(sanitize_intent_key("  .-lead"), "lead");
        assert_eq!(sanitize_intent_key("tail--"), "tail");
        assert_eq!(sanitize_intent_key("  "), "default");
        assert_eq!(sanitize_intent_key("***"), "default");
        assert_eq!(sanitize_intent_key(&"x".repeat(200)).len(), 120);
        assert_eq!(sanitize_intent_key("a_b.c-d"), "a_b.c-d");
    }

    #[test]
    fn normalize_anchor_rejects_half_records_and_orders_keys() {
        assert!(normalize_anchor(&json!(null), "k").is_none());
        assert!(normalize_anchor(&json!([]), "k").is_none());
        assert!(normalize_anchor(&json!({"request": "   "}), "k").is_none());
        let a = normalize_anchor(
            &json!({"request": "do the thing", "do_not_reverse": ["a", " b ", ""], "stop_conditions": "x, ,y"}),
            "feat",
        )
        .unwrap();
        assert_eq!(
            jsjson::stringify(&Value::Object(a)),
            r#"{"schema_version":"1.0","key":"feat","written_at":null,"request":"do the thing","acceptance":"","next_action":null,"feature":null,"lane":null,"cell":null,"do_not_reverse":["a","b"],"stop_conditions":["x","y"]}"#
        );
    }

    #[test]
    fn blocks_render_byte_shape() {
        let anchor = normalize_anchor(
            &json!({
                "request": "verbatim words",
                "acceptance": "it works",
                "next_action": "step 2",
                "feature": "f1",
                "do_not_reverse": ["d1", "d2"]
            }),
            "f1",
        )
        .unwrap();
        let pre = precompact_block(&anchor);
        assert!(pre.starts_with(PRECOMPACT_HEADER));
        assert!(pre.ends_with(PRECOMPACT_FOOTER));
        assert!(pre.contains("verbatim words\nDONE MEANS: it works\nNEXT ACTION: step 2\nDO NOT REVERSE: d1 | d2\nCONTEXT: feature=f1"));
        let res = resume_block(&anchor);
        assert!(res.starts_with(RESUME_HEADER));
        assert!(res.ends_with("it never replaces it."));
        // formatAnchor carries DO NOT REVERSE but no CONTEXT line.
        let fmt = format_anchor(&anchor);
        assert!(fmt.contains("DO NOT REVERSE: d1 | d2"));
        assert!(!fmt.contains("CONTEXT:"));
        assert!(fmt.starts_with("Intent anchor \"f1\" (written null)"));
    }

    #[test]
    fn candidates_prefer_feature_then_session_then_default() {
        let tmp = tempfile::tempdir().unwrap();
        // No state.json: activeFeature is None.
        let c = intent_key_candidates(tmp.path(), Some("Feat!"), Some("sess-1")).unwrap();
        assert_eq!(c, vec!["Feat".to_string(), "sess-1".to_string(), "default".to_string()]);
        let c = intent_key_candidates(tmp.path(), None, None).unwrap();
        assert_eq!(c, vec!["default".to_string()]);
        // Active feature at a working phase joins the front.
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(
            tmp.path().join(".bee").join("state.json"),
            r#"{"phase":"executing","feature":"live-f"}"#,
        )
        .unwrap();
        let c = intent_key_candidates(tmp.path(), None, Some("sess")).unwrap();
        assert_eq!(c, vec!["live-f".to_string(), "sess".to_string(), "default".to_string()]);
        // Terminal phase: the stale feature string does NOT key the anchor.
        std::fs::write(
            tmp.path().join(".bee").join("state.json"),
            r#"{"phase":"idle","feature":"stale"}"#,
        )
        .unwrap();
        let c = intent_key_candidates(tmp.path(), None, None).unwrap();
        assert_eq!(c, vec!["default".to_string()]);
    }

    #[test]
    fn read_intent_walks_candidates_and_skips_unusable() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".bee").join("intent");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("default.json"), r#"{"request":"fallback req","acceptance":"a"}"#).unwrap();
        let anchor = read_intent(tmp.path(), None, Some("missing-sess")).unwrap().unwrap();
        assert_eq!(anchor.get("request"), Some(&json!("fallback req")));
        assert_eq!(anchor.get("key"), Some(&json!("default")));
        // Corrupt candidate file → delegate (Err), never a silent skip.
        std::fs::write(dir.join("bad.json"), "{nope").unwrap();
        assert!(read_intent(tmp.path(), Some("bad"), None).is_err());
    }
}
