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
//   - a writeJsonAtomic failure (Node throws the V8 io message; nothing
//     durable has been written when this port bails)
//
// CUTOVER (2026-08-01): a corrupt .bee/intent/*.json no longer delegates. It
// warns once (crate::fsutil::warn_corrupt_json) and reads as "no anchor at
// this key" — `normalizeAnchor(readJson(file, null), key)` is null for that
// fallback, so the candidate walk and every downstream refusal are unchanged.
// A corrupt .bee/state.json fails open inside crate::state::read_state_brief,
// converted in its own file.
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

/// One candidate file read + normalize.
///
/// CUTOVER: a corrupt anchor used to delegate because Node's readJson warning
/// carried a V8 parse message. It now warns once and reads as "no anchor
/// here" — `normalizeAnchor(readJson(file, null), key)` is exactly `null` for
/// the fallback, so the candidate walk moves on to the next key unchanged.
fn read_anchor_at(root: &Path, key: &str) -> Result<Option<Map<String, Value>>, ()> {
    let file = intent_path(root, key);
    match read_json(&file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => {
            crate::fsutil::warn_corrupt_json(&file);
            Ok(None)
        }
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

    // ─── whole-command harness ─────────────────────────────────────────────
    //
    // `try_native` runs the real dispatch frame, and g_prelude resolves the
    // repo root from the PROCESS cwd. Mutating cwd in-process would race every
    // other test sharing this binary, so each command runs in a child copy of
    // THIS test binary (always the freshly built code under test — never a
    // possibly stale target/<profile>/bee executable) with cwd set to the
    // fixture. The child brackets bee's own streams with markers so the parent
    // can lift them out of libtest's chatter.

    const ARGV_ENV: &str = "BEE_RS_INTENT_GROUP_TEST_ARGV";
    const CHILD_TEST: &str = "verbs::intent_group::tests::intent_child_process";
    const OUT_OPEN: &str = "<<<bee-stdout";
    const OUT_CLOSE: &str = "bee-stdout>>>";
    const ERR_OPEN: &str = "<<<bee-stderr";
    const ERR_CLOSE: &str = "bee-stderr>>>";
    /// Tripwire: printed only when the router declined and Node would have
    /// served the command. `run_intent` refuses to let that pass for a green.
    const DELEGATED: &str = "__DELEGATED_TO_NODE__";

    #[test]
    #[ignore = "child process of run_intent(): needs a fixture cwd, never runs in-process"]
    fn intent_child_process() -> ExitCode {
        let raw = std::env::var(ARGV_ENV).expect("child spawned without an argv env var");
        let argv: Vec<OsString> = raw.split('\u{1f}').map(OsString::from).collect();
        println!("{OUT_OPEN}");
        eprintln!("{ERR_OPEN}");
        let code = match try_native(&argv, Instant::now()) {
            Some(code) => code,
            None => {
                println!("{DELEGATED}");
                ExitCode::SUCCESS
            }
        };
        println!("{OUT_CLOSE}");
        eprintln!("{ERR_CLOSE}");
        code
    }

    struct Run {
        /// The command exited non-zero (ctx.fail), as Node's `status !== 0`.
        refused: bool,
        stdout: String,
        #[allow(dead_code)]
        stderr: String,
    }

    impl Run {
        fn json(&self) -> Value {
            serde_json::from_str(&self.stdout)
                .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {:?}", self.stdout))
        }
    }

    fn between(hay: &str, open: &str, close: &str, whole: &str) -> String {
        let start = match hay.find(open) {
            Some(i) => i + open.len(),
            None => panic!("child never printed {open} — full child output:\n{whole}"),
        };
        let end = match hay[start..].find(close) {
            Some(i) => start + i,
            None => panic!("child never printed {close} — full child output:\n{whole}"),
        };
        hay[start..end].trim().to_string()
    }

    fn run_intent(root: &Path, args: &[&str]) -> Run {
        let exe = std::env::current_exe().expect("test binary path");
        let out = std::process::Command::new(exe)
            .args([CHILD_TEST, "--exact", "--ignored", "--nocapture"])
            .env(ARGV_ENV, args.join("\u{1f}"))
            .current_dir(root)
            .output()
            .expect("spawn the child test binary");
        let raw_out = String::from_utf8_lossy(&out.stdout).into_owned();
        let raw_err = String::from_utf8_lossy(&out.stderr).into_owned();
        let whole = format!("--- stdout ---\n{raw_out}\n--- stderr ---\n{raw_err}");
        let stdout = between(&raw_out, OUT_OPEN, OUT_CLOSE, &whole);
        let stderr = between(&raw_err, ERR_OPEN, ERR_CLOSE, &whole);
        assert!(
            !stdout.contains(DELEGATED),
            "`bee {}` fell through to the Node delegate — the native path under test never ran",
            args.join(" ")
        );
        Run { refused: !out.status.success(), stdout, stderr }
    }

    /// A fixture the WHOLE command can run against: `resolve_store_root` needs
    /// an onboarding marker before g_prelude will hand a verb a root.
    fn setup_cli_repo() -> tempfile::TempDir {
        let repo = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(repo.path().join(".bee")).unwrap();
        std::fs::write(
            repo.path().join(".bee").join("onboarding.json"),
            r#"{"schema_version":"1.0","bee_version":"0.1.0"}"#,
        )
        .unwrap();
        repo
    }

    /// The user's own words: long, punctuated, and with an embedded newline,
    /// so "verbatim" means more than "a substring survived" (test_intent.mjs).
    const REQUEST: &str = "Make the /orders endpoint idempotent under retries — the same Idempotency-Key must never\ncreate a second order, and please do NOT change the existing response shape.";
    const ACCEPTANCE: &str = "Replaying an identical POST /orders with the same Idempotency-Key returns the first order and creates no second row.";

    fn anchor_file(root: &Path) -> PathBuf {
        intent_path(root, DEFAULT_INTENT_KEY)
    }

    fn on_disk(root: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(anchor_file(root)).unwrap()).unwrap()
    }

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
        // CUTOVER: a corrupt candidate file used to delegate (Err). It now
        // warns once and reads as "no anchor at this key" —
        // normalizeAnchor(readJson(file, null), key) is null for that
        // fallback — so the candidate walk simply moves on.
        std::fs::write(dir.join("bad.json"), "{nope").unwrap();
        assert!(read_anchor_at(tmp.path(), "bad").unwrap().is_none());
        // The candidate walk skips it and lands on the DEFAULT key, exactly
        // as it does for a candidate whose file is absent.
        let anchor = read_intent(tmp.path(), Some("bad"), None).unwrap().unwrap();
        assert_eq!(anchor.get("key"), Some(&json!("default")));
        assert_eq!(
            locate_intent_key(tmp.path(), Some("bad"), None).unwrap(),
            Some("default".to_string())
        );
        // With no usable candidate at all, the whole walk answers "no anchor".
        std::fs::remove_file(dir.join("default.json")).unwrap();
        assert!(read_intent(tmp.path(), Some("bad"), None).unwrap().is_none());
        assert!(locate_intent_key(tmp.path(), Some("bad"), None).unwrap().is_none());
    }

    // ─── whole-command contracts (oracle: packages/bee/tests/test_intent.mjs) ─

    /// Oracle: "D1 — re-setting the SAME request is idempotent; a DIFFERENT
    /// one refuses without --force".
    #[test]
    fn set_is_idempotent_for_the_same_objective_and_refuses_a_changed_one() {
        let repo = setup_cli_repo();
        let root = repo.path();
        let set = |req: &str, acc: &str| -> Run {
            run_intent(root, &["intent", "set", "--request", req, "--acceptance", acc, "--json"])
        };

        let first = set(REQUEST, ACCEPTANCE);
        assert!(!first.refused, "the first set must be served: {:?}", first.stdout);
        let anchor = first.json();
        assert_eq!(anchor["key"], json!(DEFAULT_INTENT_KEY));
        assert_eq!(anchor["request"], json!(REQUEST), "the request round-trips verbatim");
        assert_eq!(on_disk(root)["request"], json!(REQUEST), "…newline and all, on disk");

        // Idempotent: the same objective again is allowed, not a refusal.
        let again = set(REQUEST, ACCEPTANCE);
        assert!(!again.refused, "re-setting the same objective must be allowed: {:?}", again.stdout);
        assert_eq!(again.json()["request"], json!(REQUEST));

        // A DIFFERENT request refuses with the typed D1 message…
        let changed = set("something else entirely", ACCEPTANCE);
        assert!(changed.refused, "a different request must refuse: {:?}", changed.stdout);
        let msg = changed.json()["error"].as_str().unwrap_or_default().to_string();
        assert!(msg.contains("request is immutable once set (D1)"), "{msg}");
        assert!(msg.contains("an anchor already exists at \"default\""), "{msg}");
        assert_eq!(on_disk(root)["request"], json!(REQUEST), "a refused write changes nothing");

        // …and so does a different acceptance, with its own typed message.
        let reworded = set(REQUEST, "a different definition of done");
        assert!(reworded.refused, "different acceptance must refuse: {:?}", reworded.stdout);
        let msg = reworded.json()["error"].as_str().unwrap_or_default().to_string();
        assert!(msg.contains("acceptance is immutable once set (D1)"), "{msg}");
        assert_eq!(on_disk(root)["acceptance"], json!(ACCEPTANCE), "…and changed nothing");

        // CONTROL — `--force true` replaces the objective deliberately, so the
        // refusals above are the immutability gate and not a stuck writer.
        let forced = run_intent(
            root,
            &[
                "intent", "set",
                "--request", "a genuinely new objective",
                "--acceptance", "the new objective is met",
                "--force", "true",
                "--json",
            ],
        );
        assert!(!forced.refused, "--force must replace the objective: {:?}", forced.stdout);
        assert_eq!(on_disk(root)["request"], json!("a genuinely new objective"));
        assert_eq!(on_disk(root)["acceptance"], json!("the new objective is met"));
    }

    /// Oracle: "D1 — advance() moves next_action ONLY; request and acceptance
    /// cannot be mutated".
    #[test]
    fn advance_moves_next_action_only_and_leaves_every_other_field_alone() {
        let repo = setup_cli_repo();
        let root = repo.path();
        let set = run_intent(
            root,
            &[
                "intent", "set",
                "--request", REQUEST,
                "--acceptance", ACCEPTANCE,
                "--next-action", "step one",
                "--lane", "standard",
                "--cell", "oi-2",
                "--do-not-reverse", "the response shape stays byte-identical, Idempotency-Key stays the dedupe key",
                "--json",
            ],
        );
        assert!(!set.refused, "{:?}", set.stdout);
        let before = on_disk(root);

        let adv = run_intent(root, &["intent", "advance", "--next-action", "step two", "--json"]);
        assert!(!adv.refused, "advance must be served: {:?}", adv.stdout);
        let payload = adv.json();
        assert_eq!(payload["next_action"], json!("step two"), "next_action advanced");
        assert!(
            payload["advanced_at"].as_str().is_some_and(|s| !s.is_empty()),
            "advance stamps advanced_at: {payload}"
        );

        // The durable record, not just the payload: everything except
        // next_action (moved) and advanced_at (stamped) is byte-for-byte the
        // record that was written — request and acceptance included.
        let after = on_disk(root);
        assert_eq!(after["request"], json!(REQUEST), "request is immutable through advance");
        assert_eq!(after["acceptance"], json!(ACCEPTANCE), "acceptance too");
        let mut b = before.as_object().unwrap().clone();
        let mut a = after.as_object().unwrap().clone();
        assert_eq!(b.remove("next_action"), Some(json!("step one")));
        assert_eq!(a.remove("next_action"), Some(json!("step two")));
        assert!(a.remove("advanced_at").is_some());
        assert_eq!(
            Value::Object(a),
            Value::Object(b),
            "advance touched a field other than next_action/advanced_at"
        );
    }

    /// Oracle: "D1 — advance() on a repo with no anchor returns null rather
    /// than inventing one" (the CLI arm of the same contract is the typed
    /// refusal).
    #[test]
    fn advance_without_an_anchor_refuses_and_invents_nothing() {
        let repo = setup_cli_repo();
        let root = repo.path();

        let miss = run_intent(root, &["intent", "advance", "--next-action", "anything", "--json"]);
        assert!(miss.refused, "advance with no anchor must refuse: {:?}", miss.stdout);
        assert_eq!(
            miss.json()["error"],
            json!("intent advance: no intent anchor exists to advance — run `bee intent set` first.")
        );
        assert!(!anchor_file(root).exists(), "a refused advance must never invent an anchor");

        // CONTROL — once an anchor exists the identical command is served, so
        // the refusal is the missing anchor and not a broken argv shape.
        let set = run_intent(
            root,
            &["intent", "set", "--request", REQUEST, "--acceptance", ACCEPTANCE, "--json"],
        );
        assert!(!set.refused, "{:?}", set.stdout);
        let ok = run_intent(root, &["intent", "advance", "--next-action", "anything", "--json"]);
        assert!(!ok.refused, "the same advance must be served once an anchor exists: {:?}", ok.stdout);
        assert_eq!(ok.json()["next_action"], json!("anything"));
    }

    /// Oracle: "clear() removes the anchor and is idempotent".
    #[test]
    fn clear_removes_the_anchor_and_is_idempotent() {
        let repo = setup_cli_repo();
        let root = repo.path();
        let set = run_intent(
            root,
            &["intent", "set", "--request", REQUEST, "--acceptance", ACCEPTANCE, "--json"],
        );
        assert!(!set.refused, "{:?}", set.stdout);
        assert_eq!(locate_intent_key(root, None, None).unwrap().as_deref(), Some(DEFAULT_INTENT_KEY));

        let first = run_intent(root, &["intent", "clear", "--json"]);
        assert!(!first.refused, "{:?}", first.stdout);
        let first = first.json();
        assert_eq!(first["cleared"], json!(true), "the first clear removes it");
        assert_eq!(first["key"], json!(DEFAULT_INTENT_KEY));
        assert!(!anchor_file(root).exists(), "the anchor file is gone");

        let second = run_intent(root, &["intent", "clear", "--json"]);
        assert!(!second.refused, "a second clear is a no-op, never an error: {:?}", second.stdout);
        assert_eq!(second.json()["cleared"], json!(false));

        // Nothing reads back afterwards, through the real show path.
        let show = run_intent(root, &["intent", "show", "--json"]);
        assert!(!show.refused, "{:?}", show.stdout);
        assert_eq!(show.json(), Value::Null);
    }

    /// Oracle: "D5 — a missing anchor reads as null and both renderers return
    /// '' (never throw)" + "D5 — a CORRUPT anchor reads exactly like a missing
    /// one". CUTOVER: a record that does not PARSE at all is covered here
    /// too now (case 4) — it used to delegate to Node.
    #[test]
    fn missing_and_half_written_anchors_read_as_absent_and_render_empty() {
        let repo = setup_cli_repo();
        let root = repo.path();

        let assert_absent = |label: &str| {
            for render in ["precompact", "resume"] {
                let run = run_intent(root, &["intent", "show", "--render", render, "--json"]);
                assert!(!run.refused, "{label}/{render}: {:?}", run.stdout);
                let payload = run.json();
                assert_eq!(payload["anchor"], Value::Null, "{label}/{render}: {payload}");
                assert_eq!(payload["block"], json!(""), "{label}/{render}: {payload}");
            }
            let text = run_intent(root, &["intent", "show", "--render", "precompact"]);
            assert_eq!(text.stdout, "(no intent anchor)", "{label}: text render");
        };

        // 1. No anchor file at all.
        assert_absent("missing");
        assert!(read_intent(root, None, None).unwrap().is_none());

        // 2. Parseable, but not an anchor: a record with no request is not
        //    half an objective, it is no objective.
        std::fs::create_dir_all(anchor_file(root).parent().unwrap()).unwrap();
        std::fs::write(anchor_file(root), r#"{"acceptance":"only this"}"#).unwrap();
        assert_absent("request-less record");
        assert!(read_intent(root, None, None).unwrap().is_none());

        // 3. A non-object record.
        std::fs::write(anchor_file(root), r#"["an","array"]"#).unwrap();
        assert_absent("non-object record");
        assert!(read_intent(root, None, None).unwrap().is_none());

        // 4. A record that does not parse at all — same answer, and the run
        //    still succeeds instead of routing the command to Node.
        std::fs::write(anchor_file(root), "{nope").unwrap();
        assert_absent("unparseable record");
        assert!(read_intent(root, None, None).unwrap().is_none());

        // CONTROL — a real anchor renders a labelled, non-empty block through
        // the same two renderers, so the empties above are the absence and not
        // a renderer that never emits anything.
        std::fs::remove_file(anchor_file(root)).unwrap();
        let set = run_intent(
            root,
            &["intent", "set", "--request", REQUEST, "--acceptance", ACCEPTANCE, "--json"],
        );
        assert!(!set.refused, "{:?}", set.stdout);
        let pre = run_intent(root, &["intent", "show", "--render", "precompact", "--json"]).json();
        let block = pre["block"].as_str().unwrap();
        assert!(block.starts_with(PRECOMPACT_HEADER), "{block}");
        assert!(block.ends_with(PRECOMPACT_FOOTER), "{block}");
        assert!(block.contains(REQUEST), "the verbatim request survives, newline and all: {block}");
        let res = run_intent(root, &["intent", "show", "--render", "resume", "--json"]).json();
        let block = res["block"].as_str().unwrap();
        assert!(block.starts_with(RESUME_HEADER), "{block}");
        assert!(block.contains(REQUEST), "{block}");
    }
}
