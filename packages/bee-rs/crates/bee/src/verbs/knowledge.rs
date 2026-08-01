// bee knowledge — native port of the knowledge verb group (bee.mjs
// handleKnowledgeCheck/Index/List/Context + lib/knowledge.mjs).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   knowledge check   [--strict] [--json]
//   knowledge index   [--check] [--json]
//   knowledge list    [--type T] [--lifecycle L] [--area A] [--json]
//   knowledge context --work W (--budget N | --lane tiny|small|standard|high-risk) [--json]
//
//   knowledge promote --work W [--json]
//
// Nothing in this group is left permanently delegated. `promote` mines the
// capped cell traces of a work item and renders the delivery/area/pattern
// proposals; it NEVER writes (writes: [] is the contract) and its two typed
// refusals (missing_work / unknown_work) are deterministic, so both are
// native. Its extra delegation triggers: a .bee/cells/*.json file serde
// refuses but V8 might parse, and a trace `verification_evidence` string in
// the same class — BOTH RETIRED at the cutover below.
//
// Additional delegation triggers (None before any output/write):
//   - --help anywhere, unknown flags, non-flag tokens, validate()-failing
//     shapes (missing --work/--budget, non-numeric --budget, --strict=x, ...)
//   - a configured non-empty `product_root` (repo-divorce topology: Node's
//     resolveProductRoot warn/path semantics are not replicated here)
//   - corrupt .bee/config.json or state files (Node warns with V8 text)
//   - bundle file/dir names carrying chars >= U+E000 (JS sorts by UTF-16
//     code units; Rust by UTF-8 bytes — they disagree only across that range)
//   - --budget values outside the plain decimal/scientific grammar that JS
//     Number() also accepts (hex, Infinity, ...)
//   - any emitted value failing the JS number round-trip guard
//
// CUTOVER (2026-08-01) — the arms that existed only because Node's text
// would have carried V8/libuv bytes are native now:
//   - a frontmatter quoted scalar with a lone-surrogate escape (U+D800..
//     U+DFFF) is no longer "a shape only V8 could decide": it is an
//     undecodable quoted scalar, and takes the same bad_quoted_string
//     finding every other one takes.
//   - an unreadable bundle file mid-walk pushes checkBundle's own
//     `unreadable` finding and keeps walking (the Rust io message stands
//     where Node put the libuv one).
//   - JSON-looking text that serde refuses — a cell file in promote's walk,
//     a trace `verification_evidence` — takes Node's OWN catch branch
//     (silently skipped / kept as raw text) instead of delegating: with one
//     parser left, which branch ran is no longer in doubt.
//
// DIVERGENCE NOTES (documented, unreachable-different for real bee data):
//   - relevance scores use Rust's libm ln() vs V8's fdlibm port — equal for
//     all practical inputs, possibly one ulp apart in razor-edge ties, and
//     toFixed(6) here rounds half-to-even where JS toFixed rounds ties up
//     (binary doubles essentially never land on exact decimal midpoints).
//   - toLowerCase in relevance tokens uses Rust's Unicode lowercasing, which
//     can differ from JS on a handful of special-cased code points (same
//     accepted approximation decisions.rs documents).
//   - `knowledge index` write failures surface a Rust io message where Node
//     would print the V8 message (partial writes make delegation unsafe).
//
// Provenance: bee.mjs handleKnowledgeCheck/handleKnowledgeIndex/
// handleKnowledgeList/handleKnowledgeContext + resolveKnowledgeContextLaneBudget,
// lib/knowledge.mjs (CONCEPT_TYPES/PROFILE_REQUIRED/KEY_RE/RESERVED_BASENAMES/
// bundleDir/emitFrontmatter/parseFrontmatter/listBundleMarkdown/
// isIsoDateHeading/checkIndexFile/checkLogFile/readPath/resolveInsideBundle/
// checkBundle/collectConcepts/listConcepts/computeIndexFiles/
// knowledgeIndexDrift/renderKnowledgeIndexes/CONFUSABLE_FOLD/foldEncoding/
// normalizeSubject/
// CONTEXT_ESTIMATOR/estimateTokens/KNOWLEDGE_CONTEXT_LANE_BUDGETS/beeOf/dirOf/
// normalizeBundleTarget/CRITICAL_RELEVANCE/RELEVANCE_STOPWORDS/relevanceTokens/
// conceptBody/metaTextOf/scoreCriticalRelevance/buildContextManifest),
// lib/state.mjs resolveProductRoot (delegating branch only).
//
// This file also hosts the pub(crate) dispatch frame (GCtx / g_prelude)
// shared by the R3-wave-2 group files intent_group.rs / reviews.rs /
// tmp_group.rs — the same root → drift → emit/fail/timing shape
// reservations.rs's Ctx implements, with the no-root path keyed on the
// PRE-parse --json scan exactly like bee.mjs main().

use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::emit_no_root_error;
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── shared dispatch frame (pub(crate): intent_group / reviews / tmp_group) ─

pub(crate) struct GCtx {
    pub(crate) root: PathBuf,
    cmd: &'static str,
    pub(crate) json: bool,
    t0: Instant,
    drift_changed: bool,
    drift_hint: &'static str,
}

pub(crate) enum GPre {
    Go(GCtx),
    Emitted(ExitCode),
}

/// bee.mjs main()'s root + manifest-drift preamble. `pre_json` is the
/// pre-parse rest scan (the no-root error fires before parseFlags in Node);
/// `json` is the authoritative post-parse flag. Ok wrapped in Option:
/// None => delegate (linked worktree, corrupt drift cache).
pub(crate) fn g_prelude(cmd: &'static str, json: bool, pre_json: bool, t0: Instant) -> Option<GPre> {
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::NeedsNode => return None,
        Roots::None => return Some(GPre::Emitted(emit_no_root_error(&cwd, cmd, pre_json, t0))),
    };
    let Ok(drift) = check_manifest_drift(&root) else { return None };
    Some(GPre::Go(GCtx {
        root,
        cmd,
        json,
        t0,
        drift_changed: drift.manifest_changed,
        drift_hint: drift.hint,
    }))
}

impl GCtx {
    /// bee.mjs emit(): drift line (stderr) + result/text (stdout) + timing.
    pub(crate) fn emit(&self, result: &Value, text: &str, exit_code: u8) -> ExitCode {
        if self.drift_changed {
            eprintln!("manifest_changed: true — {}", self.drift_hint);
        }
        if self.json {
            println!("{}", jsjson::stringify_pretty(result));
        } else {
            println!("{text}");
        }
        crate::verbs::record_timing(&self.root, self.cmd, self.t0, exit_code == 0);
        ExitCode::from(exit_code)
    }

    /// bee.mjs emitError(): no drift line, {"error"} on stdout or msg on
    /// stderr, exit 1.
    pub(crate) fn fail(&self, message: &str) -> ExitCode {
        if self.json {
            println!("{}", jsjson::stringify(&json!({ "error": message })));
        } else {
            eprintln!("{message}");
        }
        crate::verbs::record_timing(&self.root, self.cmd, self.t0, false);
        ExitCode::FAILURE
    }
}

/// jsonRequested — bee.mjs main()'s pre-parse rest scan.
pub(crate) fn pre_json_scan(toks: &[&str]) -> bool {
    toks.iter().any(|t| *t == "--json" || t.starts_with("--json="))
}

/// A registry type:"boolean" flag through validate() + the handler's
/// `flags.x === true` test: bare flag => true; "true"/"false" string values
/// pass validate but are NOT `=== true`; anything else fails validate
/// (delegate => None).
pub(crate) fn js_bool_flag(flags: &Flags, name: &str) -> Option<bool> {
    match flags.get(name) {
        None => Some(false),
        Some(FlagV::Present) => Some(true),
        Some(FlagV::S(s)) if s == "true" || s == "false" => Some(false),
        Some(FlagV::S(_)) => None,
    }
}

/// Template-literal coercion for a possibly-absent field (undefined =>
/// "undefined"), shared by the group files.
pub(crate) fn js_str_or_undefined(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

// ─── bundle root (bundleDir + the delegating slice of resolveProductRoot) ──

/// docs/knowledge under the product root. None => delegate: a configured
/// non-empty product_root (divorce topology, GitHub #14 — Node's warn/path
/// semantics live there) or a corrupt config file (V8 warning).
fn bundle_dir(root: &Path) -> Option<PathBuf> {
    let config = read_config_raw(root).ok()?;
    match config.get("product_root") {
        None | Some(Value::Null) => {}
        Some(Value::String(s)) if s.is_empty() => {}
        Some(_) => return None,
    }
    Some(root.join("docs").join("knowledge"))
}

// ─── constants (lib/knowledge.mjs) ─────────────────────────────────────────

const OKF_VERSION: &str = "0.1";

const CONCEPT_TYPES: [&str; 9] = [
    "bee.area",
    "bee.feature",
    "bee.work-item",
    "bee.plan",
    "bee.delivery",
    "bee.decision",
    "bee.pattern",
    "bee.runbook",
    "bee.evidence",
];

const ROOT_KEY_ORDER: [&str; 6] = ["type", "title", "description", "tags", "timestamp", "resource"];
const BEE_KEY_ORDER: [&str; 13] = [
    "id",
    "lifecycle",
    "areas",
    "required_context",
    "decisions",
    "sources",
    "lane",
    "polarity",
    "critical",
    "authoritative_for",
    "review_status",
    "supersedes",
    "superseded_by",
];

const PROFILE_REQUIRED: [&[&str]; 4] = [&["title"], &["description"], &["bee", "id"], &["bee", "lifecycle"]];

fn key_re_ok(key: &str) -> bool {
    // /^[A-Za-z_][A-Za-z0-9_.-]*$/
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

fn is_reserved_basename(base: &str) -> bool {
    base == "index.md" || base == "log.md"
}

/// JS `\s` (same set String.prototype.trim strips) — via reservations.
fn js_is_space(c: char) -> bool {
    crate::verbs::reservations::js_is_ws(c)
}

fn js_quote_str(s: &str) -> String {
    jsjson::stringify(&Value::String(s.to_string()))
}

// ─── emitter (emitFrontmatter — the D12 subset's source of truth) ──────────

fn is_plain_safe(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    if value != js_trim(value) {
        return false;
    }
    // /[:#"'\\\[\]{},\t\r\n]/
    if value
        .chars()
        .any(|c| matches!(c, ':' | '#' | '"' | '\'' | '\\' | '[' | ']' | '{' | '}' | ',' | '\t' | '\r' | '\n'))
    {
        return false;
    }
    // /^[-?&*!|>%@`]/
    if matches!(
        value.chars().next(),
        Some('-' | '?' | '&' | '*' | '!' | '|' | '>' | '%' | '@' | '`')
    ) {
        return false;
    }
    !(value == "true" || value == "false" || value == "null")
}

/// emitScalar — Err(()) mirrors the JS throw (caught by the round-trip guard).
fn emit_scalar(value: &Value) -> Result<String, ()> {
    match value {
        Value::Bool(true) => Ok("true".to_string()),
        Value::Bool(false) => Ok("false".to_string()),
        Value::String(s) => Ok(if is_plain_safe(s) { s.clone() } else { js_quote_str(s) }),
        _ => Err(()),
    }
}

fn emit_value(value: &Value) -> Result<String, ()> {
    match value {
        Value::Array(items) => {
            let parts = items.iter().map(emit_scalar).collect::<Result<Vec<_>, ()>>()?;
            Ok(format!("[{}]", parts.join(", ")))
        }
        other => emit_scalar(other),
    }
}

fn emit_entries(lines: &mut Vec<String>, map: &Map<String, Value>, order: &[&str], indent: &str) -> Result<(), ()> {
    let known: Vec<&String> = order
        .iter()
        .filter_map(|k| map.keys().find(|key| key.as_str() == *k))
        .collect();
    let mut unknown: Vec<&String> = map
        .keys()
        .filter(|k| !order.contains(&k.as_str()) && k.as_str() != "bee")
        .collect();
    unknown.sort(); // JS default sort — keys are KEY_RE ASCII, byte order matches
    for key in known.into_iter().chain(unknown) {
        if !key_re_ok(key) {
            return Err(());
        }
        let value = &map[key.as_str()];
        if matches!(value, Value::Object(_)) {
            return Err(()); // nested map — only root-level "bee:" is legal
        }
        lines.push(format!("{indent}{key}: {}", emit_value(value)?));
    }
    Ok(())
}

/// emitFrontmatter(data) — canonical block incl. both --- lines, LF, trailing \n.
fn emit_frontmatter(data: &Map<String, Value>) -> Result<String, ()> {
    let mut lines = vec!["---".to_string()];
    emit_entries(&mut lines, data, &ROOT_KEY_ORDER, "")?;
    if let Some(bee) = data.get("bee") {
        let Value::Object(bee) = bee else { return Err(()) };
        lines.push("bee:".to_string());
        emit_entries(&mut lines, bee, &BEE_KEY_ORDER, "  ")?;
    }
    lines.push("---".to_string());
    Ok(format!("{}\n", lines.join("\n")))
}

// ─── parser (accepts exactly the emitted subset; loud typed failure) ───────

enum Fm {
    Absent,
    Parsed {
        data: Map<String, Value>,
        block: String,
        body: String,
    },
    Failed {
        code: &'static str,
        message: String,
        line: usize,
    },
}

fn fm_fail(code: &'static str, message: String, line: usize) -> Result<Value, Fm> {
    Err(Fm::Failed { code, message, line })
}

fn parse_scalar_token(raw: &str, line_no: usize) -> Result<Value, Fm> {
    if raw == "true" {
        return Ok(Value::Bool(true));
    }
    if raw == "false" {
        return Ok(Value::Bool(false));
    }
    if raw.starts_with('"') {
        match serde_json::from_str::<Value>(raw) {
            Ok(Value::String(s)) => return Ok(Value::String(s)),
            Ok(_) => {
                return fm_fail("bad_quoted_string", "quoted value did not decode to a string".to_string(), line_no)
            }
            // CUTOVER: a lone-surrogate escape (U+D800..U+DFFF) used to
            // return Fm::NeedsNode here — V8's JSON.parse accepted it where
            // serde never can, so the whole command delegated. There is no
            // second parser left, so it takes the SAME bad_quoted_string
            // finding every other undecodable quoted scalar takes.
            Err(_) => {
                return fm_fail(
                    "bad_quoted_string",
                    format!("quoted value {} is not one complete JSON string", js_quote_str(raw)),
                    line_no,
                );
            }
        }
    }
    if raw.starts_with('\'') {
        return fm_fail(
            "single_quoted_string",
            "single-quoted scalars are outside the emitted subset — use double quotes".to_string(),
            line_no,
        );
    }
    // /^[&*!|>%@`{}]/
    if matches!(raw.chars().next(), Some('&' | '*' | '!' | '|' | '>' | '%' | '@' | '`' | '{' | '}')) {
        return fm_fail(
            "unsupported_scalar",
            format!(
                "value starting with \"{}\" (anchor/alias/block/flow-map indicator) is outside the emitted subset",
                raw.chars().next().unwrap()
            ),
            line_no,
        );
    }
    Ok(Value::String(raw.to_string()))
}

fn parse_flow_list(raw: &str, line_no: usize) -> Result<Value, Fm> {
    if !raw.ends_with(']') {
        return fm_fail(
            "bad_flow_list",
            format!("flow list {} does not close with \"]\"", js_quote_str(raw)),
            line_no,
        );
    }
    let inner = js_trim(&raw[1..raw.len() - 1]);
    if inner.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;
    for ch in inner.chars() {
        if in_quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quote = false;
            }
        } else if ch == '"' {
            current.push(ch);
            in_quote = true;
        } else if ch == ',' {
            segments.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    if in_quote {
        return fm_fail("bad_flow_list", "unterminated quoted item inside flow list".to_string(), line_no);
    }
    segments.push(current);
    let mut value = Vec::new();
    for segment in &segments {
        let token = js_trim(segment);
        if token.is_empty() {
            return fm_fail("bad_flow_list", "empty item inside flow list".to_string(), line_no);
        }
        value.push(parse_scalar_token(token, line_no)?);
    }
    Ok(Value::Array(value))
}

fn parse_key_value_line(line: &str, target: &mut Map<String, Value>, line_no: usize, prefix: &str) -> Result<(), Fm> {
    let Some(sep) = line.find(": ") else {
        return fm_fail(
            "unrecognized_line",
            format!(
                "line {} is not \"key: value\", a \"bee:\" map header, or a closing \"---\"",
                js_quote_str(line)
            ),
            line_no,
        )
        .map(|_| ());
    };
    let key = &line[..sep];
    if !key_re_ok(key) {
        return fm_fail(
            "bad_key",
            format!("{} is not a legal frontmatter key", js_quote_str(key)),
            line_no,
        )
        .map(|_| ());
    }
    if target.contains_key(key) {
        return fm_fail("duplicate_key", format!("duplicate key \"{prefix}{key}\""), line_no).map(|_| ());
    }
    let raw = &line[sep + 2..];
    if raw.is_empty() {
        return fm_fail("empty_value", format!("key \"{prefix}{key}\" has no value after \": \""), line_no)
            .map(|_| ());
    }
    let parsed = if raw.starts_with('[') {
        parse_flow_list(raw, line_no)?
    } else {
        parse_scalar_token(raw, line_no)?
    };
    target.insert(key.to_string(), parsed);
    Ok(())
}

/// parseFrontmatter(text) — see lib/knowledge.mjs for the full contract.
fn parse_frontmatter(text: &str) -> Fm {
    let open_len = if text.starts_with("---\r\n") {
        5
    } else if text.starts_with("---\n") {
        4
    } else {
        return Fm::Absent;
    };

    let mut cursor = open_len;
    let mut block_end: Option<usize> = None;
    let mut inner_end = 0usize;
    while cursor <= text.len() {
        let nl = text[cursor..].find('\n').map(|p| p + cursor);
        let line_end = nl.unwrap_or(text.len());
        let mut line = &text[cursor..line_end];
        if let Some(stripped) = line.strip_suffix('\r') {
            line = stripped;
        }
        if line == "---" {
            inner_end = cursor;
            block_end = Some(nl.map(|p| p + 1).unwrap_or(text.len()));
            break;
        }
        let Some(nl) = nl else { break };
        cursor = nl + 1;
    }
    let Some(block_end) = block_end else {
        return Fm::Failed {
            code: "unclosed_frontmatter",
            message: "frontmatter opened with \"---\" but never closed".to_string(),
            line: 1,
        };
    };

    let block = text[..block_end].to_string();
    let body = text[block_end..].to_string();
    let inner_raw = &text[open_len..inner_end];
    let inner_lines: Vec<&str> = if inner_raw.is_empty() {
        Vec::new()
    } else {
        let mut v: Vec<&str> = inner_raw.split('\n').collect();
        v.pop();
        v
    };

    let mut data: Map<String, Value> = Map::new();
    let mut in_bee_map = false;
    let mut line_no = 1usize;
    for raw_line in inner_lines {
        line_no += 1;
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() {
            return Fm::Failed {
                code: "blank_line",
                message: "blank line inside frontmatter is outside the emitted subset".to_string(),
                line: line_no,
            };
        }
        if line.contains('\t') {
            return Fm::Failed {
                code: "tab_in_frontmatter",
                message: "tab character inside frontmatter is outside the emitted subset".to_string(),
                line: line_no,
            };
        }
        if let Some(inner) = line.strip_prefix("  ") {
            if !in_bee_map {
                return Fm::Failed {
                    code: "unexpected_indent",
                    message: "indented line outside the \"bee:\" map".to_string(),
                    line: line_no,
                };
            }
            if inner.starts_with(' ') {
                return Fm::Failed {
                    code: "bad_indent",
                    message: "bee: map entries are indented exactly two spaces".to_string(),
                    line: line_no,
                };
            }
            let bee = data
                .get_mut("bee")
                .and_then(Value::as_object_mut)
                .expect("bee map exists while in_bee_map");
            match parse_key_value_line(inner, bee, line_no, "bee.") {
                Ok(()) => continue,
                Err(f) => return f,
            }
        }
        if line.starts_with(' ') {
            return Fm::Failed {
                code: "bad_indent",
                message: "root-level lines must not be indented".to_string(),
                line: line_no,
            };
        }
        in_bee_map = false;
        // /^([^:\s]+):$/ — a map header line.
        let header_key = line.strip_suffix(':').filter(|key| {
            !key.is_empty() && key.chars().all(|c| c != ':' && !js_is_space(c))
        });
        if let Some(key) = header_key {
            if !key_re_ok(key) {
                return Fm::Failed {
                    code: "bad_key",
                    message: format!("{} is not a legal frontmatter key", js_quote_str(key)),
                    line: line_no,
                };
            }
            if key != "bee" {
                return Fm::Failed {
                    code: "unsupported_map",
                    message: format!(
                        "nested map \"{key}:\" is outside the emitted subset (the only nested map is \"bee:\")"
                    ),
                    line: line_no,
                };
            }
            if data.contains_key("bee") {
                return Fm::Failed {
                    code: "duplicate_key",
                    message: "duplicate key \"bee\"".to_string(),
                    line: line_no,
                };
            }
            data.insert("bee".to_string(), Value::Object(Map::new()));
            in_bee_map = true;
            continue;
        }
        if let Err(f) = parse_key_value_line(line, &mut data, line_no, "") {
            return f;
        }
    }

    Fm::Parsed { data, block, body }
}

// ─── bundle walk (listBundleMarkdown — never leaves docs/knowledge/, D23) ──

/// lstat-level symlink test matching Node's dirent.isSymbolicLink(): on
/// Windows any reparse point (symlink OR junction) counts, like libuv.
fn is_symlinkish(path: &Path) -> bool {
    let Ok(md) = std::fs::symlink_metadata(path) else { return false };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        (md.file_attributes() & 0x400) != 0 // FILE_ATTRIBUTE_REPARSE_POINT
    }
    #[cfg(not(windows))]
    {
        md.file_type().is_symlink()
    }
}

/// None => delegate (non-UTF-16-sortable names or unrepresentable OsStrings).
fn list_bundle_markdown(dir: &Path) -> Option<Vec<String>> {
    fn walk(abs: &Path, rel: &str, out: &mut Vec<String>) -> Option<()> {
        let entries = match std::fs::read_dir(abs) {
            Ok(rd) => rd,
            Err(_) => return Some(()),
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_str()?.to_string();
            let child_abs = entry.path();
            if is_symlinkish(&child_abs) {
                continue; // a symlink could escape the bundle — never follow (D23)
            }
            let Ok(ft) = entry.file_type() else { continue };
            let child_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if ft.is_dir() {
                walk(&child_abs, &child_rel, out)?;
            } else if ft.is_file() && name.ends_with(".md") {
                out.push(child_rel);
            }
        }
        Some(())
    }
    let mut out = Vec::new();
    if dir.exists() {
        walk(dir, "", &mut out)?;
    }
    // JS Array#sort compares UTF-16 code units; UTF-8 byte order agrees below
    // U+E000 (supplementary chars sort before U+E000..U+FFFF under UTF-16).
    if out.iter().any(|rel| rel.chars().any(|c| c >= '\u{e000}')) {
        return None;
    }
    out.sort();
    Some(out)
}

fn read_file_lossy(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

// ─── log.md / index.md checks ──────────────────────────────────────────────

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// isIsoDateHeading — the ISO_HEADING_RE match + the Date.UTC round-trip
/// check (which also rejects years < 100: Date.UTC maps 0–99 to 1900+y).
fn is_iso_date_heading(text: &str) -> bool {
    let b = text.as_bytes();
    let digit = |i: usize| i < b.len() && b[i].is_ascii_digit();
    if !(digit(0) && digit(1) && digit(2) && digit(3) && b.get(4) == Some(&b'-') && digit(5) && digit(6)
        && b.get(7) == Some(&b'-') && digit(8) && digit(9))
    {
        return false;
    }
    let mut i = 10usize;
    if i < b.len() {
        // optional time part: [T ]HH:MM(:SS(.frac)?)?(Z|[+-]HH:?MM)?
        if !(b[i] == b'T' || b[i] == b' ') {
            return false;
        }
        i += 1;
        if !(digit(i) && digit(i + 1) && b.get(i + 2) == Some(&b':') && digit(i + 3) && digit(i + 4)) {
            return false;
        }
        i += 5;
        if b.get(i) == Some(&b':') {
            if !(digit(i + 1) && digit(i + 2)) {
                return false;
            }
            i += 3;
            if b.get(i) == Some(&b'.') {
                i += 1;
                let start = i;
                while digit(i) {
                    i += 1;
                }
                if i == start {
                    return false;
                }
            }
        }
        match b.get(i) {
            None => {}
            Some(b'Z') => {
                i += 1;
            }
            Some(b'+') | Some(b'-') => {
                i += 1;
                if !(digit(i) && digit(i + 1)) {
                    return false;
                }
                i += 2;
                if b.get(i) == Some(&b':') {
                    i += 1;
                }
                if !(digit(i) && digit(i + 1)) {
                    return false;
                }
                i += 2;
            }
            Some(_) => return false,
        }
        if i != b.len() {
            return false;
        }
    }
    let y: i64 = text[0..4].parse().unwrap_or(-1);
    let m: i64 = text[5..7].parse().unwrap_or(-1);
    let d: i64 = text[8..10].parse().unwrap_or(-1);
    y >= 100 && (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m)
}

fn finding(file: &str, code: &str, message: String) -> Value {
    let mut m = Map::new();
    m.insert("file".into(), Value::String(file.to_string()));
    m.insert("code".into(), Value::String(code.to_string()));
    m.insert("message".into(), Value::String(message));
    Value::Object(m)
}

fn check_index_file(rel: &str, text: &str, errors: &mut Vec<Value>) -> Option<()> {
    let parsed = parse_frontmatter(text);
    let is_root = rel == "index.md";
    if !is_root {
        if !matches!(parsed, Fm::Absent) {
            // presence alone decides — parseability does not rescue it
            errors.push(finding(
                rel,
                "index_frontmatter",
                "a non-root index.md must not carry frontmatter (OKF §6; D4)".to_string(),
            ));
        }
        return Some(());
    }
    match parsed {
        Fm::Absent => Some(()),
        Fm::Failed { code, message, line } => {
            errors.push(finding(
                rel,
                "unparseable_frontmatter",
                format!("root index.md frontmatter is unparseable — {code}: {message} (line {line})"),
            ));
            Some(())
        }
        Fm::Parsed { data, .. } => {
            let extra: Vec<&String> = data.keys().filter(|k| k.as_str() != "okf_version").collect();
            if !extra.is_empty() {
                errors.push(finding(
                    rel,
                    "root_index_extra_keys",
                    format!(
                        "root index.md may carry only okf_version (OKF §9); found extra key(s): {}",
                        extra.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                    ),
                ));
            }
            Some(())
        }
    }
}

fn check_log_file(rel: &str, text: &str, errors: &mut Vec<Value>) {
    for (i, line) in text.split('\n').enumerate() {
        // /^##\s+(.*?)\s*$/ — '##', >=1 JS-\s, content trimmed of trailing \s.
        let Some(rest) = line.strip_prefix("##") else { continue };
        let after_ws = rest.trim_start_matches(js_is_space);
        if after_ws.len() == rest.len() {
            continue; // no whitespace after '##' — the regex requires \s+
        }
        let content = after_ws.trim_end_matches(js_is_space);
        if !is_iso_date_heading(content) {
            errors.push(finding(
                rel,
                "log_heading_not_iso",
                format!(
                    "log.md date heading {} (line {}) is not ISO 8601 (OKF §7 MUST)",
                    js_quote_str(content),
                    i + 1
                ),
            ));
        }
    }
}

// ─── path resolution inside the bundle (resolveInsideBundle subset) ────────

/// resolveInsideBundle + normalizeBundleTarget: lexically resolve `target`
/// against the ABSOLUTE bundle `dir` exactly like path.resolve (pops through
/// '..' and re-entry, clamps at the filesystem root, case-sensitive prefix
/// compare), and return the bundle-relative path with '/' separators when the
/// result is a strict descendant of `dir`; None when it escapes (never
/// followed, D23). Err(()) => delegate (drive-letter / rooted shapes whose
/// win32 path.resolve semantics are not fully modeled here).
fn normalize_bundle_target(dir: &Path, target: &str) -> Result<Option<String>, ()> {
    if target.is_empty() {
        return Ok(None);
    }
    if target.contains(':') || target.starts_with('/') || target.starts_with('\\') {
        return Err(()); // drive-relative / rooted forms — Node decides
    }
    // The bundle dir's own normal components are the containment prefix
    // (path.resolve(dir) — dir is already absolute and '..'-free here).
    let base: Vec<String> = dir
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(os) => os.to_str().map(str::to_string),
            _ => None,
        })
        .collect();
    let mut stack: Vec<String> = base.clone();
    for seg in target.split(['/', '\\']) {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            stack.pop(); // at the root, path.resolve clamps — pop of empty is a no-op
        } else {
            stack.push(seg.to_string());
        }
    }
    if stack.len() <= base.len() || stack[..base.len()] != base[..] {
        return Ok(None); // not a strict descendant of the bundle dir
    }
    Ok(Some(stack[base.len()..].join("/")))
}

/// resolveInsideBundle for existence checks: absolute path when contained.
fn resolve_inside_bundle(dir: &Path, target: &str) -> Result<Option<PathBuf>, ()> {
    Ok(normalize_bundle_target(dir, target)?.map(|rel| join_rel(dir, &rel)))
}

// ─── concept inventory (collectConcepts) ───────────────────────────────────

struct Concept {
    path: String,
    data: Map<String, Value>,
}

/// None => delegate (walk/name issues).
fn collect_concepts(dir: &Path) -> Option<Vec<Concept>> {
    let mut concepts = Vec::new();
    for rel in list_bundle_markdown(dir)? {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if is_reserved_basename(base) {
            continue;
        }
        let data = match read_file_lossy(&join_rel(dir, &rel)) {
            Err(_) => Map::new(), // unreadable: keep the row with empty data
            Ok(text) => match parse_frontmatter(&text) {
                Fm::Parsed { data, .. } => data,
                _ => Map::new(),
            },
        };
        concepts.push(Concept { path: rel, data });
    }
    Some(concepts)
}

fn join_rel(dir: &Path, rel: &str) -> PathBuf {
    let mut p = dir.to_path_buf();
    for seg in rel.split('/') {
        p.push(seg);
    }
    p
}

/// beeOf(data) — the bee map when it is a plain object, else empty.
fn bee_of(data: &Map<String, Value>) -> Map<String, Value> {
    match data.get("bee") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    }
}

fn dir_of(rel: &str) -> &str {
    match rel.rfind('/') {
        Some(p) => &rel[..p],
        None => "",
    }
}

fn str_field<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    match map.get(key) {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        _ => None,
    }
}

// ─── checkBundle (D4/D13 + G14 layer 3) ────────────────────────────────────

struct CheckReport {
    okf_errors: Vec<Value>,
    profile_errors: Vec<Value>,
    warnings: Vec<Value>,
    files: usize,
    concepts: usize,
    ok: bool,
}

// ─── G14 layer 1: the subject SKELETON (knowledge.mjs foldEncoding /
//     normalizeSubject) ────────────────────────────────────────────────────
//
// Encoding must never be able to buy a fork: a trailing period and a Cyrillic
// 'е' homoglyph both used to buy a NEW concept sitting beside the owner, so
// `bee.authoritative_for` identity is a skeleton, not a string:
//
//   NFKC            -> fullwidth, ligature and math-alphanumeric forms
//   lowercase       -> case is not identity
//   NFD + strip \p{M} -> diacritics are not identity
//   confusable fold -> cross-script look-alikes (NFKC does NOT do this: a
//                      Cyrillic 'е' U+0435 and a Latin 'e' stay distinct
//                      codepoints forever, which is exactly the defeat)
//   non-letter/digit runs -> a single space, ends trimmed
//
// This replaces the ASCII-only subset the port shipped with (a non-ASCII
// claim used to DELEGATE the whole `knowledge check` rather than guess the
// fold), so the anti-fork gate now answers natively for every claim.

/// knowledge.mjs CONFUSABLE_FOLD — the UTS #39 skeleton fold, bounded to the
/// look-alikes that collide with ASCII. Transcribed key-for-key from the .mjs
/// map (Cyrillic then Greek); order is irrelevant, membership is not.
const CONFUSABLE_FOLD: [(char, char); 42] = [
    // Cyrillic -> Latin
    ('а', 'a'), ('в', 'b'), ('е', 'e'), ('ё', 'e'), ('з', '3'), ('к', 'k'),
    ('м', 'm'), ('н', 'h'), ('о', 'o'), ('р', 'p'), ('с', 'c'), ('т', 't'),
    ('у', 'y'), ('х', 'x'), ('ѕ', 's'), ('і', 'i'), ('ї', 'i'), ('ј', 'j'),
    ('ԁ', 'd'), ('ԛ', 'q'), ('ԝ', 'w'), ('ѵ', 'v'), ('ӏ', 'l'), ('ѡ', 'w'),
    ('ғ', 'f'),
    // Greek -> Latin
    ('α', 'a'), ('β', 'b'), ('γ', 'y'), ('ε', 'e'), ('ζ', 'z'), ('η', 'n'),
    ('ι', 'i'), ('κ', 'k'), ('ν', 'v'), ('ο', 'o'), ('ρ', 'p'), ('τ', 't'),
    ('υ', 'u'), ('χ', 'x'), ('ϲ', 'c'), ('ϳ', 'j'), ('ϱ', 'p'),
];

fn confusable_fold(c: char) -> char {
    match CONFUSABLE_FOLD.iter().find(|(from, _)| *from == c) {
        Some((_, to)) => *to,
        None => c,
    }
}

/// knowledge.mjs foldEncoding — `NFKC -> toLowerCase -> NFD -> strip \p{M} ->
/// confusable map`. Keeps punctuation (normalizeSubject strips it).
pub(crate) fn fold_encoding(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let bare: String = text
        .nfkc()
        .collect::<String>()
        // JS String.prototype.toLowerCase is the full Unicode Default Case
        // Conversion (locale-independent), which is exactly str::to_lowercase.
        .to_lowercase()
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect();
    bare.chars().map(confusable_fold).collect()
}

/// knowledge.mjs normalizeSubject — the skeleton used for `authoritative_for`
/// ownership. `''` for a subject carrying no letters or digits at all (null,
/// '', '   ', '...'), which is the signal layer 2 refuses on.
///
/// `\p{L}|\p{N}` is spelled here as `is_alphabetic() || is_numeric()`. The two
/// sets differ only by Other_Alphabetic, which is made up of marks and of
/// enclosed/squared letter forms — the marks are already gone (NFD + \p{M}
/// strip above) and the enclosed forms are already folded to plain letters by
/// NFKC, so on the input this function actually sees the two spellings agree.
fn normalize_subject(subject: &str) -> String {
    let folded = fold_encoding(subject);
    let mut out = String::new();
    let mut in_run = false;
    for c in folded.chars() {
        if c.is_alphabetic() || c.is_numeric() {
            out.push(c);
            in_run = false;
        } else if !in_run {
            out.push(' ');
            in_run = true;
        }
    }
    // `.trim()` after the replacement: only ASCII spaces can remain at the ends.
    out.trim_matches(' ').to_string()
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

fn check_bundle(dir: &Path, strict: bool) -> Option<CheckReport> {
    let mut errors: Vec<Value> = Vec::new();
    let mut warnings: Vec<Value> = Vec::new();
    let mut profile_errors: Vec<Value> = Vec::new();
    let files = list_bundle_markdown(dir)?;
    let mut parsed_concepts: Vec<Concept> = Vec::new();
    let mut concept_count = 0usize;

    for rel in &files {
        let base = rel.rsplit('/').next().unwrap_or(rel);
        let text = match read_file_lossy(&join_rel(dir, rel)) {
            Ok(t) => t,
            // checkBundle's own catch: push an `unreadable` finding and keep
            // walking. Node interpolated the libuv message; this carries the
            // Rust io message in the same sentence. CUTOVER: this arm used to
            // delegate purely because of those bytes.
            Err(e) => {
                errors.push(finding(rel, "unreadable", format!("could not read file: {e}")));
                continue;
            }
        };
        if is_reserved_basename(base) {
            if base == "index.md" {
                check_index_file(rel, &text, &mut errors)?;
            } else {
                check_log_file(rel, &text, &mut errors);
            }
            continue;
        }

        concept_count += 1;
        let parsed = parse_frontmatter(&text);
        let (data, block) = match parsed {
            Fm::Absent => {
                errors.push(finding(
                    rel,
                    "missing_frontmatter",
                    "a non-reserved .md inside the bundle is a concept and must carry frontmatter (D23; OKF §4)"
                        .to_string(),
                ));
                continue;
            }
            Fm::Failed { code, message, line } => {
                errors.push(finding(
                    rel,
                    "unparseable_frontmatter",
                    format!("frontmatter is unparseable — {code}: {message} (line {line})"),
                ));
                continue;
            }
            Fm::Parsed { data, block, .. } => (data, block),
        };

        match data.get("type") {
            Some(Value::String(s)) if !js_trim(s).is_empty() => {
                if !CONCEPT_TYPES.contains(&s.as_str()) {
                    warnings.push(finding(
                        rel,
                        "unknown_type",
                        format!(
                            "type \"{s}\" is outside the profile's nine types (D18); OKF consumers tolerate it, bee flags it"
                        ),
                    ));
                }
            }
            _ => {
                errors.push(finding(
                    rel,
                    "empty_type",
                    "type is required and must be a non-empty string (OKF §4.1 MUST)".to_string(),
                ));
            }
        }

        for key_path in PROFILE_REQUIRED {
            // readPath: walk objects; anything non-object mid-walk => undefined.
            let mut value: Option<&Value> = None;
            let mut cursor: Option<&Map<String, Value>> = Some(&data);
            for (i, key) in key_path.iter().enumerate() {
                let Some(map) = cursor else {
                    value = None;
                    break;
                };
                let v = map.get(*key);
                if i + 1 == key_path.len() {
                    value = v;
                } else {
                    cursor = match v {
                        Some(Value::Object(m)) => Some(m),
                        _ => None, // arrays have no such string key; primitives stop the walk
                    };
                }
            }
            let present = matches!(value, Some(Value::String(s)) if !js_trim(s).is_empty());
            if !present {
                warnings.push(finding(
                    rel,
                    "missing_profile_field",
                    format!(
                        "profile-required field \"{}\" is missing or empty (D10: never invented — author it)",
                        key_path.join(".")
                    ),
                ));
            }
        }

        let re_emitted = emit_frontmatter(&data).ok();
        if re_emitted.as_deref() != Some(block.as_str()) {
            warnings.push(finding(
                rel,
                "not_canonical",
                "frontmatter parse→re-emit differs byte-wise from the file (hand-edited colon/#/CRLF/key-order outside the canonical emitted form) — normalize by re-emitting"
                    .to_string(),
            ));
        }

        parsed_concepts.push(Concept { path: rel.clone(), data });
    }

    // ── bundle-level profile checks (D31 uniqueness, D4 dangling targets) ──
    let mut by_id: Vec<(String, Vec<String>)> = Vec::new();
    let mut by_authority: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for concept in &parsed_concepts {
        let bee = bee_of(&concept.data);
        if let Some(id) = str_field(&bee, "id") {
            match by_id.iter_mut().find(|(k, _)| k == id) {
                Some((_, holders)) => holders.push(concept.path.clone()),
                None => by_id.push((id.to_string(), vec![concept.path.clone()])),
            }
        }
        if let Some(claim_v) = bee.get("authoritative_for") {
            let claim_ok = matches!(claim_v, Value::String(s) if !js_trim(s).is_empty());
            if !claim_ok {
                let got = match claim_v {
                    Value::Null => "null",
                    Value::Array(_) => "array",
                    other => typeof_word(other),
                };
                profile_errors.push(finding(
                    &concept.path,
                    "malformed_authoritative_for",
                    format!(
                        "bee.authoritative_for must be one non-empty string (got {got}) — a claim bee cannot read is an owner the anti-fork gate cannot see (D31)"
                    ),
                ));
            } else if let Value::String(claim) = claim_v {
                // The HARDENED skeleton, not the raw string: two claims that
                // differ only by punctuation, case or ENCODING are one
                // subject with two authorities.
                let key = normalize_subject(claim);
                match by_authority.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, holders)) => holders.push((concept.path.clone(), claim.clone())),
                    None => by_authority.push((key, vec![(concept.path.clone(), claim.clone())])),
                }
            }
        }
    }
    for (id, holders) in &by_id {
        if holders.len() > 1 {
            warnings.push(finding(
                &holders[0],
                "duplicate_id",
                format!(
                    "bee.id \"{id}\" is claimed by {} concepts ({}) — ids are globally unique (D31)",
                    holders.len(),
                    holders.join(", ")
                ),
            ));
        }
    }
    for (_, holders) in &by_authority {
        if holders.len() > 1 {
            let mut subjects: Vec<String> = Vec::new();
            for (_, claim) in holders {
                if !subjects.contains(claim) {
                    subjects.push(claim.clone());
                }
            }
            let quoted: Vec<String> = subjects.iter().map(|s| format!("\"{s}\"")).collect();
            profile_errors.push(finding(
                &holders[0].0,
                "duplicate_authoritative_for",
                format!(
                    "bee.authoritative_for {} {} claimed by {} concepts ({}) — one subject, one authority (D31). Two authorities on one subject both parse and both index, and no reader can tell which is true.",
                    quoted.join(" / "),
                    if quoted.len() > 1 { "name one subject and are" } else { "is" },
                    holders.len(),
                    holders.iter().map(|(f, _)| f.as_str()).collect::<Vec<_>>().join(", ")
                ),
            ));
        }
    }
    for concept in &parsed_concepts {
        let bee = bee_of(&concept.data);
        if let Some(Value::Array(targets)) = bee.get("required_context") {
            for target in targets {
                let resolved = match target {
                    Value::String(s) => match resolve_inside_bundle(dir, s) {
                        Ok(r) => r,
                        Err(()) => return None,
                    },
                    _ => None,
                };
                let exists = resolved.map(|p| p.exists()).unwrap_or(false);
                if !exists {
                    warnings.push(finding(
                        &concept.path,
                        "dangling_required_context",
                        format!(
                            "required_context target \"{}\" does not resolve inside the bundle (D19: bundle-relative paths)",
                            jsjson::js_to_string(target)
                        ),
                    ));
                }
            }
        }
        if let Some(sup) = str_field(&bee, "supersedes") {
            if !by_id.iter().any(|(k, _)| k == sup) {
                warnings.push(finding(
                    &concept.path,
                    "dangling_supersedes",
                    format!("supersedes target id \"{sup}\" matches no concept's bee.id in the bundle"),
                ));
            }
        }
    }

    let ok = errors.is_empty() && profile_errors.is_empty() && (!strict || warnings.is_empty());
    Some(CheckReport {
        okf_errors: errors,
        profile_errors,
        warnings,
        files: files.len(),
        concepts: concept_count,
        ok,
    })
}

// ─── index (computeIndexFiles / knowledgeIndexDrift / renderKnowledgeIndexes)

const KNOWLEDGE_INDEX_HEADER: &str = "<!--\nGENERATED FILE — do not hand-edit.\nRendered by `bee knowledge index` from concept frontmatter inside docs/knowledge/ (okf-foundation D21).\nRegenerate: `bee knowledge index`. Check freshness: `bee knowledge index --check`.\nDeterministic: byte-identical for the same bundle contents — path-sorted entries, LF endings,\nnever a generation timestamp or any other wall-clock value.\n-->";

fn concept_entry_line(concept: &Concept, from_dir: &str) -> String {
    let target = if from_dir.is_empty() {
        concept.path.clone()
    } else {
        concept.path[from_dir.len() + 1..].to_string()
    };
    let base = concept.path.rsplit('/').next().unwrap_or(&concept.path);
    let title = str_field(&concept.data, "title").unwrap_or(base);
    match str_field(&concept.data, "description") {
        Some(desc) => format!("- [{title}]({target}) — {desc}"),
        None => format!("- [{title}]({target})"),
    }
}

/// computeIndexFiles(root) -> [(rel, content)] path-sorted. None => delegate.
fn compute_index_files(dir: &Path) -> Option<Vec<(String, String)>> {
    let concepts = collect_concepts(dir)?;

    let mut index_dirs: Vec<String> = vec![String::new()];
    for concept in &concepts {
        let segments: Vec<&str> = concept.path.split('/').collect();
        for i in 1..segments.len() {
            let d = segments[..i].join("/");
            if !index_dirs.contains(&d) {
                index_dirs.push(d);
            }
        }
    }
    let mut sorted_dirs = index_dirs.clone();
    sorted_dirs.sort();

    let mut files = Vec::new();
    for dir_rel in &sorted_dirs {
        let direct: Vec<&Concept> = concepts
            .iter()
            .filter(|c| dir_of(&c.path) == dir_rel.as_str())
            .collect();
        let child_dirs: Vec<&String> = {
            let mut v: Vec<&String> = index_dirs
                .iter()
                .filter(|d| {
                    !d.is_empty()
                        && if dir_rel.is_empty() {
                            !d.contains('/')
                        } else {
                            d.starts_with(&format!("{dir_rel}/")) && !d[dir_rel.len() + 1..].contains('/')
                        }
                })
                .collect();
            v.sort();
            v
        };

        let mut sections: Vec<String> = Vec::new();
        if !direct.is_empty() {
            let mut lines = vec!["## Concepts".to_string(), String::new()];
            lines.extend(direct.iter().map(|c| concept_entry_line(c, dir_rel)));
            sections.push(lines.join("\n"));
        }
        if !child_dirs.is_empty() {
            let mut lines = vec!["## Sections".to_string(), String::new()];
            for child in &child_dirs {
                let name = if dir_rel.is_empty() { child.as_str() } else { &child[dir_rel.len() + 1..] };
                let count = concepts.iter().filter(|c| c.path.starts_with(&format!("{child}/"))).count();
                lines.push(format!("- [{name}/]({name}/index.md) — {count} concept(s)"));
            }
            sections.push(lines.join("\n"));
        }
        if dir_rel.is_empty() {
            let critical: Vec<&Concept> = concepts
                .iter()
                .filter(|c| matches!(bee_of(&c.data).get("critical"), Some(Value::Bool(true))))
                .collect();
            let mut lines = vec!["## Critical patterns".to_string(), String::new()];
            if critical.is_empty() {
                lines.push("None.".to_string());
            } else {
                lines.extend(critical.iter().map(|c| concept_entry_line(c, "")));
            }
            sections.push(lines.join("\n"));
        }

        let heading = if dir_rel.is_empty() { "# Knowledge Bundle".to_string() } else { format!("# {dir_rel}/") };
        let mut body_parts = vec![heading];
        body_parts.extend(sections);
        let body = body_parts.join("\n\n");
        let frontmatter = if dir_rel.is_empty() {
            let mut fm = Map::new();
            fm.insert("okf_version".to_string(), Value::String(OKF_VERSION.to_string()));
            emit_frontmatter(&fm).ok()?
        } else {
            String::new()
        };
        let rel = if dir_rel.is_empty() { "index.md".to_string() } else { format!("{dir_rel}/index.md") };
        files.push((rel, format!("{frontmatter}{KNOWLEDGE_INDEX_HEADER}\n\n{body}\n")));
    }
    Some(files)
}

// ─── context (buildContextManifest + relevance ranking) ────────────────────

const CONTEXT_ESTIMATOR: &str = "bytes/4";
const KEEP: usize = 20;
const FLOOR: usize = 3;
const META_WEIGHT: f64 = 0.25;
const BODY_WEIGHT: f64 = 1.0;
const TAG_WEIGHT: f64 = 0.05;
const AREA_WEIGHT: f64 = 0.05;
const ZERO_SIGNAL_MIN_POPULATION: usize = 10;
const ZERO_SIGNAL_MAX_RATIO: f64 = 0.5;

const RELEVANCE_STOPWORDS: &str = "a an the and or but if then else for of to in on at by is are was were be been being it its this that these those with without from as not no never always every each any all some one two three you your we our they their he she i me my do does did done can could should would may might must will shall have has had so than which who whom what when where why how more most less least very just only also into out up down over under again further once here there both few other own same too s t don now";

fn stopwords() -> HashSet<&'static str> {
    RELEVANCE_STOPWORDS.split(' ').collect()
}

/// relevanceTokens(text) — lowercase, [a-z0-9]+ runs, >2 chars, stopped,
/// crude singularization.
fn relevance_tokens(text: &str, stops: &HashSet<&'static str>) -> Vec<String> {
    let lower: String = text.to_lowercase();
    let mut out = Vec::new();
    for raw in lower.split(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit()) {
        if raw.len() <= 2 || stops.contains(raw) {
            continue;
        }
        let token = if raw.len() > 4 && raw.ends_with('s') && !raw.ends_with("ss") {
            &raw[..raw.len() - 1]
        } else {
            raw
        };
        out.push(token.to_string());
    }
    out
}

/// Insertion-ordered unique token list (JS Set semantics for f64-sum order).
fn uniq(tokens: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for t in tokens {
        if seen.insert(t.clone()) {
            out.push(t);
        }
    }
    out
}

fn concept_body(dir: &Path, rel: &str) -> Option<String> {
    let raw = match read_file_lossy(&join_rel(dir, rel)) {
        Ok(t) => t,
        Err(_) => return Some(String::new()),
    };
    match parse_frontmatter(&raw) {
        Fm::Parsed { body, .. } => Some(body),
        _ => Some(raw),
    }
}

fn meta_text_of(concept: &Concept) -> String {
    let t = match concept.data.get("title") {
        Some(v) if crate::verbs::reservations::truthy(v) => jsjson::js_to_string(v),
        _ => String::new(),
    };
    let d = match concept.data.get("description") {
        Some(v) if crate::verbs::reservations::truthy(v) => jsjson::js_to_string(v),
        _ => String::new(),
    };
    let tags = match concept.data.get("tags") {
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| match v {
                Value::Null => String::new(),
                other => jsjson::js_to_string(other),
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    };
    format!("{t} {d} {tags}")
}

/// Number(score.toFixed(6)) — display-precision rounding (divergence note in
/// the header covers the tie-rounding difference).
fn to_fixed6(x: f64) -> f64 {
    format!("{x:.6}").parse().unwrap_or(x)
}

fn score_critical_relevance(
    dir: &Path,
    criticals: &[&Concept],
    work: &Concept,
) -> Option<Vec<(String, f64)>> {
    let stops = stopwords();
    let mut fields: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    let mut df: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for concept in criticals {
        let meta = uniq(relevance_tokens(&meta_text_of(concept), &stops));
        let body = uniq(relevance_tokens(&concept_body(dir, &concept.path)?, &stops));
        let mut union_seen: HashSet<&String> = HashSet::new();
        for token in meta.iter().chain(body.iter()) {
            if union_seen.insert(token) {
                *df.entry(token.clone()).or_insert(0) += 1;
            }
        }
        fields.push((meta, body));
    }
    let population = criticals.len();
    let idf = |token: &str| ((population as f64 + 1.0) / (*df.get(token).unwrap_or(&0) as f64 + 1.0)).ln() + 1.0;

    let query: HashSet<String> = relevance_tokens(
        &format!("{} {}", meta_text_of(work), concept_body(dir, &work.path)?),
        &stops,
    )
    .into_iter()
    .collect();
    let work_bee = bee_of(&work.data);
    let work_tags: HashSet<String> = match work.data.get("tags") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_lowercase())
            .collect(),
        _ => HashSet::new(),
    };
    let work_areas: HashSet<&str> = match work_bee.get("areas") {
        Some(Value::Array(items)) => items.iter().filter_map(|v| v.as_str()).collect(),
        _ => HashSet::new(),
    };

    let coverage = |set: &[String]| -> f64 {
        let mut hit = 0.0f64;
        let mut total = 0.0f64;
        for token in set {
            let weight = idf(token);
            total += weight;
            if query.contains(token) {
                hit += weight;
            }
        }
        if total == 0.0 {
            0.0
        } else {
            hit / total
        }
    };

    let mut scores = Vec::new();
    for (i, concept) in criticals.iter().enumerate() {
        let bee = bee_of(&concept.data);
        let tags = match concept.data.get("tags") {
            Some(Value::Array(items)) => items
                .iter()
                .filter(|v| matches!(v, Value::String(s) if work_tags.contains(&s.to_lowercase())))
                .count(),
            _ => 0,
        };
        let areas = match bee.get("areas") {
            Some(Value::Array(items)) => items
                .iter()
                .filter(|v| matches!(v, Value::String(s) if work_areas.contains(s.as_str())))
                .count(),
            _ => 0,
        };
        let (meta, body) = &fields[i];
        let score = TAG_WEIGHT * tags as f64
            + AREA_WEIGHT * areas as f64
            + META_WEIGHT * coverage(meta)
            + BODY_WEIGHT * coverage(body);
        scores.push((concept.path.clone(), to_fixed6(score)));
    }
    Some(scores)
}

fn num(v: f64) -> Value {
    Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null)
}

enum ManifestOut {
    Built(Value),
    Thrown(String),
    NeedsNode,
}

fn build_context_manifest(dir: &Path, work: &str, budget: f64, budget_raw: &Value) -> ManifestOut {
    let work_id = js_trim(work);
    if work_id.is_empty() {
        return ManifestOut::Thrown("knowledge context: missing_work — --work <id> is required (D27).".to_string());
    }
    if !budget.is_finite() || budget < 0.0 {
        // Node's message JSON.stringify's the RAW flags.budget — the CLI's
        // string (quoted) or the lane-preset number, never the conversion.
        return ManifestOut::Thrown(format!(
            "knowledge context: bad_budget — --budget must be a non-negative token count, got {} (D27).",
            jsjson::stringify(budget_raw)
        ));
    }

    let Some(concepts) = collect_concepts(dir) else { return ManifestOut::NeedsNode };

    let work_concept = concepts.iter().find(|c| {
        matches!(c.data.get("type"), Some(Value::String(t)) if t == "bee.work-item")
            && matches!(bee_of(&c.data).get("id"), Some(Value::String(id)) if id == work_id)
    });
    let Some(work_concept) = work_concept else {
        return ManifestOut::Thrown(format!(
            "knowledge context: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id \"{work_id}\" (D27)."
        ));
    };

    let mut ranked: Vec<(String, String)> = Vec::new(); // (rel, reason)
    let mut selected: HashSet<String> = HashSet::new();
    let by_path: std::collections::HashMap<&str, &Concept> =
        concepts.iter().map(|c| (c.path.as_str(), c)).collect();
    let select = |rel: &str, reason: String, ranked: &mut Vec<(String, String)>, selected: &mut HashSet<String>| {
        if selected.contains(rel) || !by_path.contains_key(rel) {
            return false;
        }
        selected.insert(rel.to_string());
        ranked.push((rel.to_string(), reason));
        true
    };

    // (1) the work item
    select(&work_concept.path, "work item".to_string(), &mut ranked, &mut selected);

    // (2) the plan sibling in the same work/<id>/ directory
    let work_dir = dir_of(&work_concept.path).to_string();
    let plan = concepts.iter().find(|c| {
        matches!(c.data.get("type"), Some(Value::String(t)) if t == "bee.plan") && dir_of(&c.path) == work_dir
    });
    if let Some(plan) = plan {
        select(&plan.path, format!("plan sibling in {work_dir}/"), &mut ranked, &mut selected);
    }

    // (3) required_context, transitive, BFS depth order, cycles deduped silently
    let mut queue: std::collections::VecDeque<(String, usize)> =
        ranked.iter().map(|(rel, _)| (rel.clone(), 0usize)).collect();
    while let Some((node_rel, depth)) = queue.pop_front() {
        let targets = match by_path.get(node_rel.as_str()).map(|c| bee_of(&c.data)) {
            Some(bee) => match bee.get("required_context") {
                Some(Value::Array(items)) => items.clone(),
                _ => continue,
            },
            None => continue,
        };
        for target in &targets {
            let Value::String(target) = target else { continue };
            let rel = match normalize_bundle_target(dir, target) {
                Ok(Some(rel)) => rel,
                Ok(None) => continue,
                Err(()) => return ManifestOut::NeedsNode,
            };
            if !by_path.contains_key(rel.as_str()) || selected.contains(&rel) {
                continue;
            }
            select(&rel, format!("required_context depth {} via {node_rel}", depth + 1), &mut ranked, &mut selected);
            queue.push_back((rel, depth + 1));
        }
    }

    // (4) the critical concepts, ranked by relevance and cut (G5/G11)
    let criticals: Vec<&Concept> = concepts
        .iter()
        .filter(|c| matches!(bee_of(&c.data).get("critical"), Some(Value::Bool(true))))
        .collect();
    let Some(relevance) = score_critical_relevance(dir, &criticals, work_concept) else {
        return ManifestOut::NeedsNode;
    };
    let score_of = |path: &str| -> f64 {
        relevance.iter().find(|(p, _)| p == path).map(|(_, s)| *s).unwrap_or(0.0)
    };
    let zero_signal_count = criticals.iter().filter(|c| score_of(&c.path) == 0.0).count();
    if criticals.len() >= ZERO_SIGNAL_MIN_POPULATION
        && (zero_signal_count as f64) > criticals.len() as f64 * ZERO_SIGNAL_MAX_RATIO
    {
        return ManifestOut::Thrown(format!(
            "knowledge context: zero_signal — {zero_signal_count} of {} bee.critical concepts score 0 against work item \"{work_id}\", above the pinned {} ratio. A ranking where most items tie at zero is a path sort wearing a relevance label — widen the work item's description/body, or fix the ranking, but do not ship this order (G11).",
            criticals.len(),
            jsjson::js_f64_to_string(ZERO_SIGNAL_MAX_RATIO)
        ));
    }
    let mut ranked_criticals: Vec<&&Concept> = criticals.iter().collect();
    ranked_criticals.sort_by(|a, b| {
        score_of(&b.path)
            .partial_cmp(&score_of(&a.path))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    let mut excluded: Vec<Value> = Vec::new();
    let mut floor_paths: Vec<String> = Vec::new();
    let mut kept = 0usize;
    for (index, concept) in ranked_criticals.iter().enumerate() {
        let rank = index + 1;
        let score = score_of(&concept.path);
        if selected.contains(&concept.path) {
            continue; // already in via required_context — never re-cut
        }
        if kept >= KEEP {
            let mut m = Map::new();
            m.insert("path".into(), Value::String(format!("docs/knowledge/{}", concept.path)));
            m.insert("score".into(), num(score));
            m.insert(
                "reason".into(),
                Value::String(format!(
                    "below the relevance cut — rank {rank} of {}, keep {KEEP} (G5)",
                    ranked_criticals.len()
                )),
            );
            excluded.push(Value::Object(m));
            continue;
        }
        let is_floor = kept < FLOOR;
        if is_floor {
            floor_paths.push(format!("docs/knowledge/{}", concept.path));
        }
        select(
            &concept.path,
            format!(
                "critical pattern (relevance {}, rank {rank} of {}{})",
                jsjson::js_f64_to_string(score),
                ranked_criticals.len(),
                if is_floor { ", floor" } else { "" }
            ),
            &mut ranked,
            &mut selected,
        );
        kept += 1;
    }

    // (5) decisions whose areas overlap the work item's areas
    let work_bee_map = bee_of(&work_concept.data);
    let work_areas: Vec<&str> = match work_bee_map.get("areas") {
        Some(Value::Array(items)) => items.iter().filter_map(|v| v.as_str()).collect(),
        _ => Vec::new(),
    };
    for concept in &concepts {
        if !matches!(concept.data.get("type"), Some(Value::String(t)) if t == "bee.decision") {
            continue;
        }
        let areas = match bee_of(&concept.data).get("areas") {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        let overlap: Vec<String> = areas
            .iter()
            .filter(|a| matches!(a, Value::String(s) if work_areas.contains(&s.as_str())))
            .map(jsjson::js_to_string)
            .collect();
        if overlap.is_empty() {
            continue;
        }
        select(&concept.path, format!("decision for area {}", overlap.join(", ")), &mut ranked, &mut selected);
    }

    struct Sized {
        repo_rel: String,
        reason: String,
        bytes: u64,
        est: f64,
        floor: bool,
    }
    let sized: Vec<Sized> = ranked
        .iter()
        .map(|(rel, reason)| {
            let repo_rel = format!("docs/knowledge/{rel}");
            let bytes = std::fs::metadata(join_rel(dir, rel)).map(|m| m.len()).unwrap_or(0);
            let est = (bytes as f64 / 4.0).ceil();
            let floor = floor_paths.contains(&repo_rel);
            Sized { repo_rel, reason: reason.clone(), bytes, est, floor }
        })
        .collect();

    let floor_cost: f64 = sized.iter().filter(|s| s.floor).map(|s| s.est).sum();
    let rank_one_cost = sized.first().map(|s| s.est).unwrap_or(0.0);
    let mut reserve = (budget - rank_one_cost).min(floor_cost).max(0.0);
    let mut available = budget - reserve;

    let mut entries: Vec<Value> = Vec::new();
    let mut truncated: Vec<String> = Vec::new();
    let mut total_est = 0.0f64;
    let mut cutting = false;
    for item in &sized {
        if item.floor {
            if item.est > reserve {
                truncated.push(item.repo_rel.clone());
                continue;
            }
            reserve -= item.est;
            total_est += item.est;
        } else {
            if cutting || item.est > available {
                cutting = true;
                truncated.push(item.repo_rel.clone());
                continue;
            }
            available -= item.est;
            total_est += item.est;
        }
        let mut m = Map::new();
        m.insert("path".into(), Value::String(item.repo_rel.clone()));
        m.insert("bytes".into(), Value::Number(Number::from(item.bytes)));
        m.insert("est_tokens".into(), num(item.est));
        m.insert("reason".into(), Value::String(item.reason.clone()));
        entries.push(Value::Object(m));
    }

    let decisions: Vec<Value> = match bee_of(&work_concept.data).get("decisions") {
        Some(Value::Array(items)) => items.iter().filter(|v| v.is_string()).cloned().collect(),
        _ => Vec::new(),
    };

    // CONSERVATION (G11)
    let accounted: HashSet<String> = entries
        .iter()
        .filter_map(|e| e.get("path").and_then(Value::as_str).map(str::to_string))
        .chain(truncated.iter().cloned())
        .chain(excluded.iter().filter_map(|e| e.get("path").and_then(Value::as_str).map(str::to_string)))
        .collect();
    let lost: Vec<String> = criticals
        .iter()
        .map(|c| format!("docs/knowledge/{}", c.path))
        .filter(|repo_rel| !accounted.contains(repo_rel))
        .collect();
    if !lost.is_empty() {
        return ManifestOut::Thrown(format!(
            "knowledge context: conservation — {} bee.critical concept(s) were neither included, truncated nor excluded: {} (G11). This is a bug in the ranking, not a condition of the bundle.",
            lost.len(),
            lost.join(", ")
        ));
    }

    let mut manifest = Map::new();
    manifest.insert("work".into(), Value::String(work_id.to_string()));
    manifest.insert("decisions".into(), Value::Array(decisions));
    manifest.insert("budget".into(), num(budget));
    manifest.insert("estimator".into(), Value::String(CONTEXT_ESTIMATOR.to_string()));
    manifest.insert("total_est".into(), num(total_est));
    manifest.insert("entries".into(), Value::Array(entries));
    manifest.insert("truncated".into(), Value::Array(truncated.into_iter().map(Value::String).collect()));
    manifest.insert("excluded".into(), Value::Array(excluded));
    manifest.insert("floor".into(), Value::Array(floor_paths.into_iter().map(Value::String).collect()));
    manifest.insert("critical_total".into(), Value::Number(Number::from(criticals.len())));
    manifest.insert("zero_signal_count".into(), Value::Number(Number::from(zero_signal_count)));
    ManifestOut::Built(Value::Object(manifest))
}

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "knowledge" {
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
        "check" => run_check(flags, json, pre_json, t0),
        "index" => run_index(flags, json, pre_json, t0),
        "list" => run_list(flags, json, pre_json, t0),
        "context" => run_context(flags, json, pre_json, t0),
        "promote" => run_promote(flags, json, pre_json, t0),
        _ => None, // unknown verbs (group-usage fallback) → Node
    }
}

fn run_check(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["strict"]) {
        return None;
    }
    let strict = js_bool_flag(&flags, "strict")?;
    let ctx = match g_prelude("knowledge check", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let report = check_bundle(&dir, strict)?;
    let failing = !report.ok;

    let mut lines: Vec<String> = Vec::new();
    let line_of = |f: &Value, tag: &str| {
        format!(
            "{tag} [{}] {}: {}",
            f.get("code").and_then(Value::as_str).unwrap_or(""),
            f.get("file").and_then(Value::as_str).unwrap_or(""),
            f.get("message").and_then(Value::as_str).unwrap_or("")
        )
    };
    for f in &report.okf_errors {
        lines.push(line_of(f, "ERROR"));
    }
    for f in &report.profile_errors {
        lines.push(line_of(f, "ERROR"));
    }
    for f in &report.warnings {
        lines.push(line_of(f, if strict { "ERROR(strict)" } else { "WARN" }));
    }
    lines.push(format!(
        "knowledge check: {} concept(s) in {} file(s), {} OKF error(s), {} profile error(s), {} profile warning(s){} — {}",
        report.concepts,
        report.files,
        report.okf_errors.len(),
        report.profile_errors.len(),
        report.warnings.len(),
        if strict { " [--strict]" } else { "" },
        if failing { "FAIL" } else { "OK" }
    ));

    let mut counts = Map::new();
    counts.insert("files".into(), Value::from(report.files));
    counts.insert("concepts".into(), Value::from(report.concepts));
    counts.insert("errors".into(), Value::from(report.okf_errors.len()));
    counts.insert("profile_errors".into(), Value::from(report.profile_errors.len()));
    counts.insert("warnings".into(), Value::from(report.warnings.len()));
    let mut okf = Map::new();
    okf.insert("errors".into(), Value::Array(report.okf_errors));
    let mut profile = Map::new();
    profile.insert("errors".into(), Value::Array(report.profile_errors));
    profile.insert("warnings".into(), Value::Array(report.warnings));
    let mut result = Map::new();
    result.insert("okf".into(), Value::Object(okf));
    result.insert("profile".into(), Value::Object(profile));
    result.insert("counts".into(), Value::Object(counts));

    Some(ctx.emit(&Value::Object(result), &lines.join("\n"), u8::from(failing)))
}

fn run_index(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["check"]) {
        return None;
    }
    let check = js_bool_flag(&flags, "check")?;
    let ctx = match g_prelude("knowledge index", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let expected = compute_index_files(&dir)?;

    if check {
        let mut stale: Vec<String> = Vec::new();
        for (rel, content) in &expected {
            let on_disk = read_file_lossy(&join_rel(&dir, rel)).ok();
            if on_disk.as_deref() != Some(content.as_str()) {
                stale.push(format!("docs/knowledge/{rel}"));
            }
        }
        let drift = !stale.is_empty();
        let mut lines: Vec<String> = stale.iter().map(|f| format!("STALE {f}")).collect();
        lines.push(format!(
            "knowledge index --check: {} expected index file(s), {} stale — {}",
            expected.len(),
            stale.len(),
            if drift { "FAIL (regenerate: bee knowledge index)" } else { "OK" }
        ));
        let mut result = Map::new();
        result.insert("checked".into(), Value::from(expected.len()));
        result.insert("stale".into(), Value::Array(stale.into_iter().map(Value::String).collect()));
        result.insert("drift".into(), Value::Bool(drift));
        return Some(ctx.emit(&Value::Object(result), &lines.join("\n"), u8::from(drift)));
    }

    let mut written: Vec<String> = Vec::new();
    for (rel, content) in &expected {
        let abs = join_rel(&dir, rel);
        let write = abs
            .parent()
            .map(std::fs::create_dir_all)
            .unwrap_or(Ok(()))
            .and_then(|()| std::fs::write(&abs, content));
        if let Err(e) = write {
            // DIVERGENCE (header note): partial writes forbid delegation, so
            // the Rust io message stands in for Node's V8-worded one.
            return Some(ctx.fail(&e.to_string()));
        }
        written.push(format!("docs/knowledge/{rel}"));
    }
    let count = written.len();
    let text = format!("Rendered {count} generated index file(s) under docs/knowledge/.");
    let mut result = Map::new();
    result.insert("written".into(), Value::Array(written.into_iter().map(Value::String).collect()));
    result.insert("count".into(), Value::from(count));
    Some(ctx.emit(&Value::Object(result), &text, 0))
}

fn run_list(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["type", "lifecycle", "area"]) {
        return None;
    }
    // handler: `typeof flags.x === 'string' ? flags.x : null` — bare booleans
    // cannot occur (none of these are FLAG_ALONE_BOOLEANS), so every present
    // flag is a string filter, empty strings included.
    let filter = |name: &str| -> Option<String> {
        match flags.get(name) {
            Some(FlagV::S(s)) => Some(s.clone()),
            _ => None,
        }
    };
    let f_type = filter("type");
    let f_lifecycle = filter("lifecycle");
    let f_area = filter("area");

    let ctx = match g_prelude("knowledge list", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let concepts = collect_concepts(&dir)?;

    let mut rows: Vec<Value> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    for concept in &concepts {
        let bee = bee_of(&concept.data);
        let id = str_field(&bee, "id");
        let c_type = str_field(&concept.data, "type");
        let lifecycle = str_field(&bee, "lifecycle");
        let title = str_field(&concept.data, "title");
        if let Some(t) = &f_type {
            if c_type != Some(t.as_str()) {
                continue;
            }
        }
        if let Some(l) = &f_lifecycle {
            if lifecycle != Some(l.as_str()) {
                continue;
            }
        }
        if let Some(a) = &f_area {
            let areas = bee.get("areas");
            let member = matches!(areas, Some(Value::Array(items)) if items.iter().any(|v| matches!(v, Value::String(s) if s == a)));
            if !member {
                continue;
            }
        }
        lines.push(format!(
            "{} · {} · {} · {} · {}",
            concept.path,
            id.unwrap_or("-"),
            c_type.unwrap_or("-"),
            lifecycle.unwrap_or("-"),
            title.unwrap_or("-")
        ));
        let mut row = Map::new();
        row.insert("path".into(), Value::String(concept.path.clone()));
        let opt = |v: Option<&str>| v.map(|s| Value::String(s.to_string())).unwrap_or(Value::Null);
        row.insert("id".into(), opt(id));
        row.insert("type".into(), opt(c_type));
        row.insert("lifecycle".into(), opt(lifecycle));
        row.insert("title".into(), opt(title));
        rows.push(Value::Object(row));
    }
    lines.push(format!("{} concept(s).", rows.len()));

    let mut result = Map::new();
    let count = rows.len();
    result.insert("concepts".into(), Value::Array(rows));
    result.insert("count".into(), Value::from(count));
    Some(ctx.emit(&Value::Object(result), &lines.join("\n"), 0))
}

/// i54-closeout D3 lane presets (KNOWLEDGE_CONTEXT_LANE_BUDGETS).
fn lane_budget(lane: &str) -> Option<f64> {
    match lane {
        "tiny" => Some(8000.0),
        "small" => Some(12000.0),
        "standard" => Some(20000.0),
        "high-risk" => Some(30000.0),
        _ => None,
    }
}

/// JS Number(<string>) over the plain decimal/scientific grammar; None =>
/// delegate (hex/binary/Infinity/other legacy shapes Node must answer).
fn js_number_conv(raw: &str) -> Option<f64> {
    let t = js_trim(raw);
    if t.is_empty() {
        return Some(0.0);
    }
    let bytes = t.as_bytes();
    let mut i = 0usize;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let int_len = i - int_start;
    let mut frac_len = 0usize;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let fs = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        frac_len = i - fs;
    }
    if int_len == 0 && frac_len == 0 {
        return None;
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        let es = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return None;
        }
    }
    if i != bytes.len() {
        return None;
    }
    t.parse::<f64>().ok().filter(|f| f.is_finite())
}

fn run_context(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["work", "budget", "lane"]) {
        return None;
    }
    // validate(): work required (present, non-'').
    let work = match flags.get("work") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    // resolveKnowledgeContextLaneBudget: an explicit non-empty --budget wins;
    // otherwise a recognized --lane fills it; otherwise validate refuses
    // (required, missing) — Node's own message, delegate.
    let (budget, budget_raw): (f64, Value) = match flags.get("budget") {
        Some(FlagV::S(s)) if !s.is_empty() => {
            if js_trim(s).is_empty() {
                return None; // validate: invalid type (whitespace-only)
            }
            (js_number_conv(s)?, Value::String(s.clone()))
        }
        _ => match flags.get("lane") {
            Some(FlagV::S(l)) if !l.is_empty() => {
                let preset = lane_budget(l)?;
                (preset, num(preset))
            }
            _ => return None,
        },
    };

    let ctx = match g_prelude("knowledge context", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let manifest = match build_context_manifest(&dir, &work, budget, &budget_raw) {
        ManifestOut::Built(m) => m,
        ManifestOut::Thrown(msg) => return Some(ctx.fail(&msg)),
        ManifestOut::NeedsNode => return None,
    };
    if !crate::verbs::feedback::value_js_safe(&manifest) {
        return None;
    }

    let g = |k: &str| manifest.get(k).cloned().unwrap_or(Value::Null);
    let arr = |k: &str| match manifest.get(k) {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let mut lines = vec![format!(
        "work: {} · budget: {} token(s) · estimator: {}",
        jsjson::js_to_string(&g("work")),
        jsjson::js_to_string(&g("budget")),
        jsjson::js_to_string(&g("estimator"))
    )];
    let decisions = arr("decisions");
    if !decisions.is_empty() {
        lines.push(format!(
            "decisions: {}",
            decisions.iter().map(jsjson::js_to_string).collect::<Vec<_>>().join(" · ")
        ));
    }
    lines.push("PATH · BYTES · EST TOKENS · REASON".to_string());
    for entry in arr("entries") {
        lines.push(format!(
            "{} · {} · {} · {}",
            js_str_or_undefined(entry.get("path")),
            js_str_or_undefined(entry.get("bytes")),
            js_str_or_undefined(entry.get("est_tokens")),
            js_str_or_undefined(entry.get("reason"))
        ));
    }
    for cut in arr("truncated") {
        lines.push(format!("TRUNCATED {}", jsjson::js_to_string(&cut)));
    }
    for dropped in arr("excluded") {
        lines.push(format!(
            "EXCLUDED {} · {} · {}",
            js_str_or_undefined(dropped.get("path")),
            js_str_or_undefined(dropped.get("score")),
            js_str_or_undefined(dropped.get("reason"))
        ));
    }
    lines.push(format!(
        "knowledge context: {} entry(ies), {} est token(s) of {} budget (estimator {}), {} truncated, {} excluded of {} critical pattern(s); zero_signal_count {}; floor {}.",
        arr("entries").len(),
        jsjson::js_to_string(&g("total_est")),
        jsjson::js_to_string(&g("budget")),
        jsjson::js_to_string(&g("estimator")),
        arr("truncated").len(),
        arr("excluded").len(),
        jsjson::js_to_string(&g("critical_total")),
        jsjson::js_to_string(&g("zero_signal_count")),
        arr("floor").len()
    ));

    Some(ctx.emit(&manifest, &lines.join("\n"), 0))
}


// ═══════════════════════════════════════════════════════════════════════════
// knowledge promote  (bee.mjs handleKnowledgePromote + lib/knowledge.mjs
// buildPromotion / readCappedCellTraces / compareCellIds / oneLine /
// deviationText / verifySummary / isoDate / touchesSubject)
// ═══════════════════════════════════════════════════════════════════════════
//
// No collation anywhere on this path: compareCellIds is a hand-written
// natural-order comparator over `id.split(/(\d+)/)` using `<`/`>` (UTF-16
// code units), and the two `.sort()` calls (capped dates, area subjects)
// are JS default sorts, i.e. UTF-16 code units — never localeCompare. That
// is why this verb ports without the confidence-guard machinery
// verbs/feedback.rs needs.
//
// promote NEVER writes (D2): `writes` is always [], and nothing here opens a
// file for writing. Both typed refusals (missing_work / unknown_work) are
// deterministic text with no V8 message and no lock attempt, so they are
// reproduced natively.

/// `text.split(/\s+/).join(' ').trim()`, optionally capped at `limit` UTF-16
/// units with a trailing ellipsis.
fn one_line(text: &str, limit: usize) -> String {
    let mut flat = String::new();
    let mut in_ws = false;
    for c in text.chars() {
        if js_is_space(c) {
            in_ws = true;
        } else {
            if in_ws {
                flat.push(' ');
            }
            in_ws = false;
            flat.push(c);
        }
    }
    if in_ws {
        flat.push(' ');
    }
    let flat = flat.trim_matches(js_is_space).to_string();
    if limit == 0 {
        return flat;
    }
    let units: Vec<u16> = flat.encode_utf16().collect();
    if units.len() <= limit {
        return flat;
    }
    format!("{}\u{2026}", String::from_utf16_lossy(&units[..limit - 1]))
}

/// deviationText: a plain string, `type: description`, or JSON.stringify.
fn deviation_text(entry: &Value) -> String {
    match entry {
        Value::String(s) => s.clone(),
        Value::Object(m) => {
            let desc = m.get("description").and_then(Value::as_str).filter(|s| !s.is_empty());
            match desc {
                Some(d) => match m.get("type").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    Some(t) => format!("{t}: {d}"),
                    None => d.to_string(),
                },
                None => jsjson::stringify(entry),
            }
        }
        Value::Array(_) => jsjson::stringify(entry), // typeof [] === 'object'
        other => jsjson::js_to_string(other),
    }
}

/// verifySummary(trace): the first of verify_tail/verify_output/evidence/
/// summary in the parsed evidence JSON, else the raw text.
fn verify_summary(trace: &Value) -> Option<String> {
    let raw = match trace.get("verification_evidence") {
        Some(Value::String(s)) => s.as_str(),
        _ => "",
    };
    if raw.trim_matches(js_is_space).is_empty() {
        return Some(String::new());
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(parsed @ (Value::Object(_) | Value::Array(_))) => {
            for key in ["verify_tail", "verify_output", "evidence", "summary"] {
                if let Some(Value::String(s)) = parsed.get(key) {
                    if !s.trim_matches(js_is_space).is_empty() {
                        return Some(one_line(s, 200));
                    }
                }
            }
            Some(one_line(raw, 200))
        }
        Ok(_) => Some(one_line(raw, 200)), // parsed, but not an object
        // CUTOVER: JSON-looking text serde refuses used to delegate, because
        // only V8 could say whether its own parse threw. Nothing else parses
        // it here now, so the catch branch IS the answer: keep the raw text.
        Err(_) => Some(one_line(raw, 200)),
    }
}

/// compareCellIds — natural order over `id.split(/(\d+)/)`. Pure `<`/`>`
/// string compare (UTF-16 code units) plus numeric compare on digit runs.
fn compare_cell_ids(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    fn split(id: &str) -> Vec<String> {
        let mut parts: Vec<String> = Vec::new();
        let chars: Vec<char> = id.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let digit = chars[i].is_ascii_digit();
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() == digit {
                i += 1;
            }
            parts.push(chars[start..i].iter().collect());
        }
        parts
    }
    let left = split(a);
    let right = split(b);
    for i in 0..left.len().max(right.len()) {
        let (l, r) = (left.get(i), right.get(i));
        match (l, r) {
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(l), Some(r)) => {
                let both_numeric = !l.is_empty()
                    && !r.is_empty()
                    && l.chars().all(|c| c.is_ascii_digit())
                    && r.chars().all(|c| c.is_ascii_digit());
                if both_numeric {
                    // Number(l) — a run long enough to lose precision compares
                    // as the f64 both runtimes produce.
                    let (nl, nr) = (js_digits_to_f64(l), js_digits_to_f64(r));
                    if nl != nr {
                        return if nl < nr { Ordering::Less } else { Ordering::Greater };
                    }
                } else if l != r {
                    let (lu, ru): (Vec<u16>, Vec<u16>) =
                        (l.encode_utf16().collect(), r.encode_utf16().collect());
                    return lu.cmp(&ru);
                }
            }
        }
    }
    Ordering::Equal
}

fn js_digits_to_f64(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(f64::NAN)
}

/// isoDate: the `YYYY-MM-DD` prefix of an ISO-ish string, else None.
fn iso_date(v: Option<&Value>) -> Option<String> {
    let s = match v {
        Some(Value::String(s)) => s.as_str(),
        _ => return None,
    };
    let b = s.as_bytes();
    let d = |i: usize| i < b.len() && b[i].is_ascii_digit();
    if b.len() >= 10
        && d(0) && d(1) && d(2) && d(3)
        && b[4] == b'-'
        && d(5) && d(6)
        && b[7] == b'-'
        && d(8) && d(9)
    {
        Some(s[..10].to_string())
    } else {
        None
    }
}

/// touchesSubject: exact match, or either path containing the other as a dir.
fn touches_subject(file: &str, subject: &str) -> bool {
    file == subject
        || file.starts_with(&format!("{subject}/"))
        || subject.starts_with(&format!("{file}/"))
}

struct CappedCell {
    id: String,
    title: String,
    lane: Option<String>,
    behavior_change: bool,
    outcome: String,
    files_changed: Vec<String>,
    deviations: Vec<String>,
    failure_signatures: Vec<String>,
    verify: String,
    verify_summary: String,
    capped_at: Option<String>,
    trace_path: String,
}

fn cell_value(c: &CappedCell) -> Value {
    let arr = |v: &Vec<String>| Value::Array(v.iter().cloned().map(Value::String).collect());
    let mut m = Map::new();
    m.insert("id".into(), Value::String(c.id.clone()));
    m.insert("title".into(), Value::String(c.title.clone()));
    m.insert("lane".into(), c.lane.clone().map(Value::String).unwrap_or(Value::Null));
    m.insert("behavior_change".into(), Value::Bool(c.behavior_change));
    m.insert("outcome".into(), Value::String(c.outcome.clone()));
    m.insert("files_changed".into(), arr(&c.files_changed));
    m.insert("deviations".into(), arr(&c.deviations));
    m.insert("failure_signatures".into(), arr(&c.failure_signatures));
    m.insert("verify".into(), Value::String(c.verify.clone()));
    m.insert("verify_summary".into(), Value::String(c.verify_summary.clone()));
    m.insert("capped_at".into(), c.capped_at.clone().map(Value::String).unwrap_or(Value::Null));
    m.insert("trace_path".into(), Value::String(c.trace_path.clone()));
    Value::Object(m)
}

/// readCappedCellTraces(root, feature). None => delegate (an unreadable
/// entry or a non-UTF-8 name; an unparseable cell is skipped, like Node).
fn read_capped_cell_traces(root: &Path, feature: &str) -> Option<Vec<CappedCell>> {
    let dir = root.join(".bee").join("cells");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Some(Vec::new()); // Node's catch: an absent store yields []
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.ok()?;
        let ft = entry.file_type().ok()?;
        if !ft.is_file() {
            continue; // dirs (incl. archive/) and symlinks are skipped
        }
        let name = entry.file_name().to_str()?.to_string();
        if name.ends_with(".json") {
            names.push(name);
        }
    }
    // readdirSync order only decides which cells are seen, never their order
    // (the result is sorted by compareCellIds below) — but keep it stable.
    names.sort();

    let mut cells = Vec::new();
    for name in names {
        let bytes = std::fs::read(dir.join(&name)).ok()?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let cell: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            // Node silently skips an unparseable cell. CUTOVER: the
            // "JSON-looking text serde refuses" sub-case used to delegate
            // rather than guess which V8 branch ran; there is no other branch
            // now, so every unparseable cell is skipped, as Node skipped it.
            Err(_) => continue,
        };
        let Value::Object(cell_map) = &cell else { continue };
        if cell_map.get("feature").and_then(Value::as_str) != Some(feature)
            || cell_map.get("status").and_then(Value::as_str) != Some("capped")
        {
            continue;
        }
        let empty = Value::Object(Map::new());
        let trace = match cell_map.get("trace") {
            Some(t @ Value::Object(_)) => t,
            Some(t @ Value::Array(_)) => t, // typeof [] === 'object'
            _ => &empty,
        };
        let deviations: Vec<String> = match trace.get("deviations") {
            Some(Value::Array(a)) => a
                .iter()
                .map(deviation_text)
                .filter(|t| !t.trim_matches(js_is_space).is_empty())
                .collect(),
            _ => Vec::new(),
        };
        let mut failure_signatures: Vec<String> = Vec::new();
        for key in ["attempts", "semantic_judge"] {
            if let Some(Value::Array(a)) = trace.get(key) {
                for item in a {
                    if let Some(Value::String(s)) = item.get("failure_signature") {
                        if !s.trim_matches(js_is_space).is_empty() {
                            failure_signatures.push(s.clone());
                        }
                    }
                }
            }
        }
        let id = match cell_map.get("id") {
            Some(Value::String(s)) => s.clone(),
            _ => name.trim_end_matches(".json").to_string(),
        };
        let cell_title = match cell_map.get("title") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let behavior_change = matches!(trace.get("behavior_change"), Some(Value::Bool(true)))
            || (trace.get("behavior_change").is_none()
                && matches!(cell_map.get("behavior_change"), Some(Value::Bool(true))));
        let outcome = match trace.get("outcome") {
            Some(Value::String(s)) if !s.trim_matches(js_is_space).is_empty() => s.clone(),
            _ => cell_title.clone(),
        };
        cells.push(CappedCell {
            id: id.clone(),
            title: cell_title,
            lane: match cell_map.get("lane") {
                Some(Value::String(s)) => Some(s.clone()),
                _ => None,
            },
            behavior_change,
            outcome,
            files_changed: match trace.get("files_changed") {
                Some(Value::Array(a)) => a
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect(),
                _ => Vec::new(),
            },
            deviations,
            failure_signatures,
            verify: match cell_map.get("verify") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            },
            verify_summary: verify_summary(trace)?,
            capped_at: match trace.get("capped_at") {
                Some(Value::String(s)) => Some(s.clone()),
                _ => None,
            },
            trace_path: format!(".bee/cells/{id}.json"),
        });
    }
    // Stable sort == JS Array.prototype.sort (spec-guaranteed since ES2019).
    cells.sort_by(|a, b| compare_cell_ids(&a.id, &b.id));
    Some(cells)
}

fn sort_utf16(list: &mut [String]) {
    list.sort_by(|a, b| {
        a.encode_utf16()
            .collect::<Vec<_>>()
            .cmp(&b.encode_utf16().collect::<Vec<_>>())
    });
}

fn str_array(map: &Map<String, Value>, key: &str) -> Vec<String> {
    match map.get(key) {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

enum Promo {
    Ok(Value),
    /// A deterministic typed refusal (bee.mjs emitError bytes).
    Thrown(String),
}

/// buildPromotion(root, {work}). None => delegate.
fn build_promotion(root: &Path, dir: &Path, work: &str) -> Option<Promo> {
    let work_id = work.trim_matches(js_is_space);
    if work_id.is_empty() {
        return Some(Promo::Thrown(
            "knowledge promote: missing_work — --work <id> is required (D38).".into(),
        ));
    }
    let concepts = collect_concepts(dir)?;
    let Some(work_concept) = concepts.iter().find(|c| {
        c.data.get("type").and_then(Value::as_str) == Some("bee.work-item")
            && bee_of(&c.data).get("id").and_then(Value::as_str) == Some(work_id)
    }) else {
        return Some(Promo::Thrown(format!(
            "knowledge promote: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id \"{work_id}\" (D38)."
        )));
    };

    let work_bee = bee_of(&work_concept.data);
    let work_areas: Vec<String> = str_array(&work_bee, "areas")
        .into_iter()
        .filter(|a| !a.is_empty())
        .collect();
    let work_decisions = str_array(&work_bee, "decisions");
    let work_tags = str_array(&work_concept.data, "tags");
    let cells = read_capped_cell_traces(root, work_id)?;

    // ── (a) delivery draft ────────────────────────────────────────────────
    let work_dir = dir_of(&work_concept.path);
    let delivery_path = if work_dir.is_empty() {
        "delivery.md".to_string()
    } else {
        format!("{work_dir}/delivery.md")
    };
    let mut capped_dates: Vec<String> = cells
        .iter()
        .filter_map(|c| iso_date(c.capped_at.as_ref().map(|s| Value::String(s.clone())).as_ref()))
        .collect();
    sort_utf16(&mut capped_dates);
    let timestamp = match capped_dates.last() {
        Some(d) => Some(d.clone()),
        None => iso_date(work_concept.data.get("timestamp")),
    };
    let deviation_count: usize = cells.iter().map(|c| c.deviations.len()).sum();
    let work_title = match work_concept.data.get("title") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => work_id.to_string(),
    };

    let mut delivery_data = Map::new();
    delivery_data.insert("type".into(), Value::String("bee.delivery".into()));
    delivery_data.insert("title".into(), Value::String(format!("{work_title} — delivery")));
    delivery_data.insert(
        "description".into(),
        Value::String(format!(
            "Delivery record proposed by bee knowledge promote for work item {work_id}: {} capped cell(s), {deviation_count} recorded deviation(s).",
            cells.len()
        )),
    );
    if !work_tags.is_empty() {
        delivery_data.insert(
            "tags".into(),
            Value::Array(work_tags.iter().cloned().map(Value::String).collect()),
        );
    }
    if let Some(ts) = &timestamp {
        delivery_data.insert("timestamp".into(), Value::String(ts.clone()));
    }
    let mut delivery_bee = Map::new();
    delivery_bee.insert("id".into(), Value::String(format!("{work_id}-delivery")));
    delivery_bee.insert("lifecycle".into(), Value::String("active".into()));
    if !work_areas.is_empty() {
        delivery_bee.insert(
            "areas".into(),
            Value::Array(work_areas.iter().cloned().map(Value::String).collect()),
        );
    }
    delivery_bee.insert(
        "required_context".into(),
        Value::Array(vec![Value::String(work_concept.path.clone())]),
    );
    if !work_decisions.is_empty() {
        delivery_bee.insert(
            "decisions".into(),
            Value::Array(work_decisions.iter().cloned().map(Value::String).collect()),
        );
    }
    let mut sources = vec![Value::String(format!("docs/knowledge/{}", work_concept.path))];
    sources.extend(cells.iter().map(|c| Value::String(c.trace_path.clone())));
    delivery_bee.insert("sources".into(), Value::Array(sources));
    if let Some(lane) = work_bee.get("lane").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        delivery_bee.insert("lane".into(), Value::String(lane.to_string()));
    }
    delivery_data.insert("bee".into(), Value::Object(delivery_bee));

    let shipped: Vec<String> = if !cells.is_empty() {
        cells
            .iter()
            .map(|c| {
                format!(
                    "- **{}** — {} ({} file(s) changed)",
                    c.id,
                    one_line(&c.outcome, 0),
                    c.files_changed.len()
                )
            })
            .collect()
    } else {
        vec![format!(
            "No capped cell trace for work item {work_id} exists in .bee/cells/ at proposal time."
        )]
    };
    let verified: Vec<String> = if !cells.is_empty() {
        cells
            .iter()
            .map(|c| {
                let suffix = if c.verify_summary.is_empty() {
                    String::new()
                } else {
                    format!(" — {}", c.verify_summary)
                };
                format!("- **{}** — `{}`{suffix}", c.id, c.verify)
            })
            .collect()
    } else {
        vec!["Nothing to verify: no capped cell trace was found.".to_string()]
    };
    let mut deviation_lines: Vec<String> = Vec::new();
    for c in &cells {
        for d in &c.deviations {
            deviation_lines.push(format!("- **{}** — {}", c.id, one_line(d, 0)));
        }
    }
    if deviation_lines.is_empty() {
        deviation_lines.push("None recorded in the capped cell traces.".to_string());
    }

    let mut body: Vec<String> = vec![
        format!("# {work_title} — Delivery"),
        String::new(),
        "## What shipped".into(),
        String::new(),
    ];
    body.extend(shipped);
    body.extend([String::new(), "## Verify".into(), String::new(),
        "Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.".into(),
        String::new()]);
    body.extend(verified);
    body.extend([String::new(), "## Deviations".into(), String::new()]);
    body.extend(deviation_lines);
    body.extend([String::new(), "## Provenance".into(), String::new()]);
    body.push(format!(
        "Proposed by `bee knowledge promote --work {work_id}` from {} capped cell trace(s) in `.bee/cells/` and the work item `docs/knowledge/{}`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.",
        cells.len(),
        work_concept.path
    ));
    body.push(String::new());
    let delivery_content = format!("{}\n{}", emit_frontmatter(&delivery_data).ok()?, body.join("\n"));

    let mut delivery = Map::new();
    delivery.insert("path".into(), Value::String(delivery_path.clone()));
    delivery.insert(
        "repo_path".into(),
        Value::String(format!("docs/knowledge/{delivery_path}")),
    );
    delivery.insert("content".into(), Value::String(delivery_content));

    // ── (b) area updates ──────────────────────────────────────────────────
    let mut area_updates: Vec<Value> = Vec::new();
    for area in &work_areas {
        let mut subjects: Vec<String> = Vec::new(); // insertion-ordered Set
        for concept in &concepts {
            let bee = bee_of(&concept.data);
            if !str_array(&bee, "areas").iter().any(|a| a == area) {
                continue;
            }
            let own = format!("docs/knowledge/{}", concept.path);
            if !subjects.contains(&own) {
                subjects.push(own);
            }
            for source in str_array(&bee, "sources") {
                if !source.is_empty() && !subjects.contains(&source) {
                    subjects.push(source);
                }
            }
        }
        let mut bullets: Vec<Value> = Vec::new();
        for c in &cells {
            if !c.behavior_change {
                continue;
            }
            let touched: Vec<String> = c
                .files_changed
                .iter()
                .filter(|file| subjects.iter().any(|s| touches_subject(file, s)))
                .cloned()
                .collect();
            if touched.is_empty() {
                continue;
            }
            let mut b = Map::new();
            b.insert("cell".into(), Value::String(c.id.clone()));
            b.insert("text".into(), Value::String(one_line(&c.outcome, 0)));
            b.insert(
                "files".into(),
                Value::Array(touched.into_iter().map(Value::String).collect()),
            );
            b.insert("trace".into(), Value::String(c.trace_path.clone()));
            bullets.push(Value::Object(b));
        }
        let mut sorted = subjects.clone();
        sort_utf16(&mut sorted);
        let mut u = Map::new();
        u.insert("area".into(), Value::String(area.clone()));
        u.insert(
            "subjects".into(),
            Value::Array(sorted.into_iter().map(Value::String).collect()),
        );
        u.insert("bullets".into(), Value::Array(bullets));
        area_updates.push(Value::Object(u));
    }

    // ── (c) pattern candidates ────────────────────────────────────────────
    let mut pattern_candidates: Vec<Value> = Vec::new();
    for c in &cells {
        if c.deviations.is_empty() && c.failure_signatures.is_empty() {
            continue;
        }
        let mut evidence: Vec<(&'static str, String)> = Vec::new();
        for d in &c.deviations {
            evidence.push(("deviation", d.clone()));
        }
        for f in &c.failure_signatures {
            evidence.push(("failure_signature", f.clone()));
        }
        let mut data = Map::new();
        data.insert("type".into(), Value::String("bee.pattern".into()));
        data.insert(
            "title".into(),
            Value::String(format!("{work_id} cell {} — pitfall candidate", c.id)),
        );
        data.insert(
            "description".into(),
            Value::String(format!(
                "Pitfall candidate mined from cell {}'s capped trace: {}",
                c.id,
                one_line(&evidence[0].1, 160)
            )),
        );
        if let Some(ts) = iso_date(c.capped_at.as_ref().map(|s| Value::String(s.clone())).as_ref())
        {
            data.insert("timestamp".into(), Value::String(ts));
        }
        let mut b = Map::new();
        b.insert("id".into(), Value::String(format!("{work_id}-{}-pitfall", c.id)));
        b.insert("lifecycle".into(), Value::String("draft".into()));
        if !work_areas.is_empty() {
            b.insert(
                "areas".into(),
                Value::Array(work_areas.iter().cloned().map(Value::String).collect()),
            );
        }
        b.insert(
            "sources".into(),
            Value::Array(vec![Value::String(c.trace_path.clone())]),
        );
        b.insert("polarity".into(), Value::String("pitfall".into()));
        data.insert("bee".into(), Value::Object(b));

        let mut lines: Vec<String> = vec![
            format!("# {work_id} cell {} — pitfall candidate", c.id),
            String::new(),
            "## What the cell did".into(),
            String::new(),
            one_line(&c.outcome, 0),
            String::new(),
            format!("## Recorded evidence (verbatim from {})", c.trace_path),
            String::new(),
        ];
        for (kind, text) in &evidence {
            lines.push(format!("- **{kind}** — {}", one_line(text, 0)));
        }
        lines.extend([
            String::new(),
            "## Status".into(),
            String::new(),
            "Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.".into(),
            String::new(),
        ]);

        let rel = format!("patterns/{work_id}-{}-pitfall.md", c.id);
        let mut cand = Map::new();
        cand.insert("cell".into(), Value::String(c.id.clone()));
        cand.insert("path".into(), Value::String(rel.clone()));
        cand.insert("repo_path".into(), Value::String(format!("docs/knowledge/{rel}")));
        cand.insert(
            "evidence".into(),
            Value::Array(
                evidence
                    .iter()
                    .map(|(kind, text)| {
                        let mut e = Map::new();
                        e.insert("kind".into(), Value::String((*kind).into()));
                        e.insert("text".into(), Value::String(text.clone()));
                        Value::Object(e)
                    })
                    .collect(),
            ),
        );
        cand.insert(
            "content".into(),
            Value::String(format!("{}\n{}", emit_frontmatter(&data).ok()?, lines.join("\n"))),
        );
        pattern_candidates.push(Value::Object(cand));
    }

    let mut out = Map::new();
    out.insert("work".into(), Value::String(work_id.to_string()));
    out.insert("work_item".into(), Value::String(work_concept.path.clone()));
    out.insert(
        "cells".into(),
        Value::Array(cells.iter().map(cell_value).collect()),
    );
    out.insert("delivery".into(), Value::Object(delivery));
    out.insert("area_updates".into(), Value::Array(area_updates));
    out.insert("pattern_candidates".into(), Value::Array(pattern_candidates));
    out.insert("writes".into(), Value::Array(Vec::new()));
    Some(Promo::Ok(Value::Object(out)))
}

/// handleKnowledgePromote's human rendering.
fn promote_text(p: &Value) -> String {
    let cells = p["cells"].as_array().cloned().unwrap_or_default();
    let ids: Vec<String> = cells
        .iter()
        .map(|c| c["id"].as_str().unwrap_or("").to_string())
        .collect();
    let head = format!(
        "promote proposal for work item \"{}\" ({}) — {} capped cell(s){}",
        p["work"].as_str().unwrap_or(""),
        p["work_item"].as_str().unwrap_or(""),
        cells.len(),
        if cells.is_empty() { String::new() } else { format!(": {}", ids.join(", ")) }
    );
    let mut lines = vec![
        head,
        "PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.".to_string(),
        String::new(),
        format!("(a) DELIVERY DRAFT — save as {}", p["delivery"]["repo_path"].as_str().unwrap_or("")),
        String::new(),
        strip_one_trailing_newline(p["delivery"]["content"].as_str().unwrap_or("")),
        String::new(),
        "(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell".to_string(),
        String::new(),
    ];
    let area_updates = p["area_updates"].as_array().cloned().unwrap_or_default();
    if area_updates.is_empty() {
        lines.push(
            "None: the work item declares no bee.areas, so there is no area to sync (D19)."
                .to_string(),
        );
        lines.push(String::new());
    }
    for update in &area_updates {
        lines.push(format!("area {}:", update["area"].as_str().unwrap_or("")));
        let bullets = update["bullets"].as_array().cloned().unwrap_or_default();
        if bullets.is_empty() {
            lines.push("  (no capped behavior_change cell touched this area's subjects)".into());
        }
        for b in &bullets {
            let files: Vec<String> = b["files"]
                .as_array()
                .map(|a| a.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect())
                .unwrap_or_default();
            lines.push(format!(
                "  - [{}] {} — touched {} (trace {})",
                b["cell"].as_str().unwrap_or(""),
                b["text"].as_str().unwrap_or(""),
                files.join(", "),
                b["trace"].as_str().unwrap_or("")
            ));
        }
        lines.push(String::new());
    }
    lines.push("(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall".into());
    lines.push(String::new());
    let candidates = p["pattern_candidates"].as_array().cloned().unwrap_or_default();
    if candidates.is_empty() {
        lines.push("None: no capped cell trace carries a deviation or a failure signature.".into());
        lines.push(String::new());
    }
    for c in &candidates {
        lines.push(format!(
            "from cell {} — save as {}",
            c["cell"].as_str().unwrap_or(""),
            c["repo_path"].as_str().unwrap_or("")
        ));
        lines.push(String::new());
        lines.push(strip_one_trailing_newline(c["content"].as_str().unwrap_or("")));
        lines.push(String::new());
    }
    let bullet_total: usize = area_updates
        .iter()
        .map(|u| u["bullets"].as_array().map(Vec::len).unwrap_or(0))
        .sum();
    lines.push(format!(
        "knowledge promote: {} capped cell(s) mined, 1 delivery draft, {bullet_total} area bullet(s), {} pattern candidate(s), 0 file(s) written.",
        cells.len(),
        candidates.len()
    ));
    lines.join("\n")
}

/// `.replace(/\n$/, '')`
fn strip_one_trailing_newline(s: &str) -> String {
    s.strip_suffix('\n').unwrap_or(s).to_string()
}

fn run_promote(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["work"]) {
        return None;
    }
    // validate() owns the missing/empty required flag; a bare `--work` is
    // impossible (not a FLAG_ALONE_BOOLEAN).
    let work = match flags.get("work") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    let ctx = match g_prelude("knowledge promote", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    match build_promotion(&ctx.root, &dir, &work)? {
        Promo::Thrown(msg) => Some(ctx.fail(&msg)),
        Promo::Ok(proposal) => {
            let text = promote_text(&proposal);
            Some(ctx.emit(&proposal, &text, 0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(text: &str) -> Map<String, Value> {
        match parse_frontmatter(text) {
            Fm::Parsed { data, .. } => data,
            _ => panic!("expected parse"),
        }
    }

    #[test]
    fn frontmatter_round_trips_canonical_form() {
        let text = "---\ntype: bee.pattern\ntitle: \"A: colon title\"\ntags: [one, two]\nbee:\n  id: p-1\n  lifecycle: active\n  critical: true\n---\nbody\n";
        let (data, block, body) = match parse_frontmatter(text) {
            Fm::Parsed { data, block, body } => (data, block, body),
            _ => panic!("parse failed"),
        };
        assert_eq!(body, "body\n");
        assert_eq!(emit_frontmatter(&data).unwrap(), block);
        assert_eq!(data["title"], Value::String("A: colon title".into()));
        assert_eq!(data["bee"]["critical"], Value::Bool(true));
    }

    #[test]
    fn frontmatter_failures_match_node_codes() {
        match parse_frontmatter("---\ntitle 'x'\n---\n") {
            Fm::Failed { code, .. } => assert_eq!(code, "unrecognized_line"),
            _ => panic!("expected failure"),
        }
        match parse_frontmatter("---\ntitle: 'x'\n---\n") {
            Fm::Failed { code, .. } => assert_eq!(code, "single_quoted_string"),
            _ => panic!("expected failure"),
        }
        match parse_frontmatter("---\n\ntitle: x\n---\n") {
            Fm::Failed { code, line, .. } => {
                assert_eq!(code, "blank_line");
                assert_eq!(line, 2);
            }
            _ => panic!("expected failure"),
        }
        match parse_frontmatter("---\ntitle: x") {
            Fm::Failed { code, .. } => assert_eq!(code, "unclosed_frontmatter"),
            _ => panic!("expected failure"),
        }
        // CUTOVER: a lone-surrogate escape used to be NeedsNode (delegate).
        // It is now the ordinary undecodable-quoted-scalar finding.
        match parse_frontmatter("---\ntitle: \"\\ud800\"\n---\n") {
            Fm::Failed { code, .. } => assert_eq!(code, "bad_quoted_string"),
            _ => panic!("a lone surrogate must be a finding, not a delegation"),
        }
    }

    #[test]
    fn crlf_parses_but_block_keeps_bytes() {
        let text = "---\r\ntype: bee.pattern\r\n---\r\nbody";
        match parse_frontmatter(text) {
            Fm::Parsed { data, block, .. } => {
                assert_eq!(data["type"], Value::String("bee.pattern".into()));
                assert!(block.contains('\r'));
                assert_ne!(emit_frontmatter(&data).unwrap(), block); // not_canonical trigger
            }
            _ => panic!("expected parse"),
        }
    }

    #[test]
    fn iso_heading_calendar_check_matches_date_utc() {
        assert!(is_iso_date_heading("2026-02-28"));
        assert!(is_iso_date_heading("2024-02-29"));
        assert!(!is_iso_date_heading("2026-02-29"));
        assert!(!is_iso_date_heading("2026-13-01"));
        assert!(!is_iso_date_heading("0099-01-01")); // Date.UTC maps 0-99 to 1900+y
        assert!(is_iso_date_heading("2026-07-01T10:30"));
        assert!(is_iso_date_heading("2026-07-01 10:30:05.123Z"));
        assert!(is_iso_date_heading("2026-07-01T10:30:05+07:00"));
        assert!(!is_iso_date_heading("yesterday"));
    }

    #[test]
    fn bundle_check_and_index_render() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("knowledge");
        std::fs::create_dir_all(dir.join("areas/billing")).unwrap();
        std::fs::write(
            dir.join("areas/billing/refunds.md"),
            "---\ntype: bee.pattern\ntitle: Refund flow\ndescription: How refunds settle\nbee:\n  id: pat-1\n  lifecycle: active\n  critical: true\n---\nBody here.\n",
        )
        .unwrap();
        std::fs::write(dir.join("notes.md"), "no frontmatter\n").unwrap();

        let report = check_bundle(&dir, false).unwrap();
        assert_eq!(report.files, 2);
        assert_eq!(report.concepts, 2);
        assert_eq!(report.okf_errors.len(), 1); // notes.md missing_frontmatter
        assert_eq!(report.okf_errors[0]["code"], "missing_frontmatter");
        assert!(report.warnings.is_empty());
        assert!(!report.ok);

        let files = compute_index_files(&dir).unwrap();
        // Dir-sort order: '' sorts first, so the root index leads the set.
        let rels: Vec<&str> = files.iter().map(|(r, _)| r.as_str()).collect();
        assert_eq!(rels, vec!["index.md", "areas/index.md", "areas/billing/index.md"]);
        let root = &files[0].1;
        assert!(root.starts_with("---\nokf_version: 0.1\n---\n<!--\n"));
        assert!(root.contains("## Critical patterns"));
        assert!(root.contains("- [Refund flow](areas/billing/refunds.md) — How refunds settle"));
        assert!(root.contains("- [areas/](areas/index.md) — 1 concept(s)"));
        // Non-root indexes carry no frontmatter.
        assert!(files[1].1.starts_with("<!--\n"));
    }

    /// knowledge.mjs foldEncoding + normalizeSubject, unit-level. Every row is
    /// an ENCODING difference that must NOT be able to buy a second authority
    /// for one subject, paired with the genuine-difference control.
    #[test]
    fn normalize_subject_is_a_skeleton_not_a_string() {
        // Case, punctuation and whitespace are not identity.
        assert_eq!(normalize_subject("Billing: Refunds!"), "billing refunds");
        assert_eq!(normalize_subject("  BILLING---refunds.  "), "billing refunds");
        // No letters or digits at all -> '' (the signal layer 2 refuses on).
        for empty in ["", "   ", "...", "-- //"] {
            assert_eq!(normalize_subject(empty), "", "{empty:?}");
        }

        // NFKC: fullwidth, ligature and math-alphanumeric forms all fold.
        assert_eq!(normalize_subject("\u{ff47}\u{ff41}\u{ff54}\u{ff45}\u{ff53}"), "gates");
        assert_eq!(normalize_subject("\u{fb01}le"), "file"); // ﬁ ligature
        assert_eq!(normalize_subject("\u{1d420}ates"), "gates"); // 𝐠 math bold
        assert_eq!(normalize_subject("\u{2460}"), "1"); // ① circled digit
        // NFD + \p{M} strip: diacritics are not identity, precomposed or not.
        assert_eq!(normalize_subject("caf\u{e9}"), "cafe");
        assert_eq!(normalize_subject("cafe\u{301}"), "cafe");
        assert_eq!(normalize_subject("N\u{c3}\u{a9}"), normalize_subject("N\u{c3}\u{a9}"));
        // Confusable fold: NFKC alone leaves these distinct forever.
        assert_eq!(normalize_subject("g\u{430}tes"), "gates"); // Cyrillic 'а'
        assert_eq!(normalize_subject("gat\u{435}s"), "gates"); // Cyrillic 'е'
        assert_eq!(normalize_subject("g\u{3b1}tes"), "gates"); // Greek 'α'
        assert_eq!(normalize_subject("\u{41a}ey"), "key"); // uppercase Cyrillic 'К'
        assert_eq!(normalize_subject("\u{451}poch"), "epoch"); // Cyrillic 'ё'
        // The fold is bounded: a letter with no ASCII look-alike survives, so
        // two genuinely different scripts are still two subjects.
        assert_ne!(normalize_subject("gates"), normalize_subject("шлюзы"));
        // …and a word-order paraphrase is a DIFFERENT subject, never folded
        // (the residual layer-1 cannot close, by design).
        assert_ne!(
            normalize_subject("refunds and reversals"),
            normalize_subject("reversals and refunds")
        );
    }

    /// The confusable table is transcribed by hand from the .mjs map, so it is
    /// pinned as a set: exactly the 25 Cyrillic + 17 Greek entries, no more.
    #[test]
    fn the_confusable_table_is_exactly_the_mjs_map() {
        assert_eq!(CONFUSABLE_FOLD.len(), 42);
        let mut seen: Vec<char> = CONFUSABLE_FOLD.iter().map(|(f, _)| *f).collect();
        let before = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), before, "a duplicated key would silently shadow");
        for (from, to) in CONFUSABLE_FOLD {
            assert!(to.is_ascii(), "{from:?} folds to a non-ASCII target {to:?}");
            assert_eq!(from.to_lowercase().next(), Some(from), "{from:?} must be a lowercase key");
        }
    }

    #[test]
    fn bundle_target_normalization_matches_path_resolve_containment() {
        let dir = Path::new("D:\\repo\\docs\\knowledge");
        let norm = |t: &str| normalize_bundle_target(dir, t);
        assert_eq!(norm("areas/x.md").unwrap().unwrap(), "areas/x.md");
        assert_eq!(norm("a/./b/../c.md").unwrap().unwrap(), "a/c.md");
        assert_eq!(norm("../escape.md").unwrap(), None);
        // Climb out and re-enter — path.resolve calls this contained.
        assert_eq!(norm("../knowledge/y.md").unwrap().unwrap(), "y.md");
        assert_eq!(norm("../KNOWLEDGE/y.md").unwrap(), None); // case-sensitive prefix
        assert!(norm("/abs.md").is_err()); // rooted → delegate
        assert!(norm("C:/x.md").is_err()); // drive shape → delegate
    }

    #[test]
    fn relevance_tokens_stop_and_singularize() {
        let stops = stopwords();
        // "rows".length is 4, NOT > 4 — Node keeps it plural; "refunds" drops the s.
        assert_eq!(
            relevance_tokens("The refunds and Reversals of class rows!", &stops),
            vec!["refund", "reversal", "class", "rows"]
        );
        // <=2 chars and stopwords drop; 'ss' endings keep.
        assert_eq!(relevance_tokens("is at process", &stops), vec!["process"]);
    }

    #[test]
    fn js_number_conv_subset() {
        assert_eq!(js_number_conv("20000"), Some(20000.0));
        assert_eq!(js_number_conv("  1.5e3 "), Some(1500.0));
        assert_eq!(js_number_conv(".5"), Some(0.5));
        assert_eq!(js_number_conv(""), Some(0.0));
        assert_eq!(js_number_conv("0x10"), None); // JS-valid but delegated
        assert_eq!(js_number_conv("Infinity"), None);
        assert_eq!(js_number_conv("12px"), None);
    }

    #[test]
    fn context_manifest_orders_and_cuts() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("knowledge");
        std::fs::create_dir_all(dir.join("work/w1")).unwrap();
        std::fs::write(
            dir.join("work/w1/item.md"),
            // required_context targets resolve against the BUNDLE root (D19).
            "---\ntype: bee.work-item\ntitle: Widget work\ndescription: widgets and gears\nbee:\n  id: w1\n  lifecycle: active\n  areas: [billing]\n  decisions: [\"0001\"]\n  required_context: [ctx.md]\n---\nwidgets gears assembly\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("work/w1/plan.md"),
            "---\ntype: bee.plan\ntitle: Plan\nbee:\n  id: w1-plan\n  lifecycle: active\n---\nplan body\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("ctx.md"),
            "---\ntype: bee.pattern\ntitle: Context doc\nbee:\n  id: ctx\n  lifecycle: active\n---\nctx body\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("crit.md"),
            "---\ntype: bee.pattern\ntitle: Widget gear lesson\ndescription: widgets gears\nbee:\n  id: crit\n  lifecycle: active\n  critical: true\n---\nwidgets gears everywhere\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("dec.md"),
            "---\ntype: bee.decision\ntitle: Billing decision\nbee:\n  id: dec\n  lifecycle: active\n  areas: [billing]\n---\ndecision body\n",
        )
        .unwrap();

        let manifest = match build_context_manifest(&dir, "w1", 20000.0, &json_raw("20000")) {
            ManifestOut::Built(m) => m,
            _ => panic!("expected manifest"),
        };
        let entries: Vec<String> = manifest["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["path"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            entries,
            vec![
                "docs/knowledge/work/w1/item.md",
                "docs/knowledge/work/w1/plan.md",
                "docs/knowledge/ctx.md",
                "docs/knowledge/crit.md",
                "docs/knowledge/dec.md",
            ]
        );
        assert_eq!(manifest["decisions"], serde_json::json!(["0001"]));
        assert_eq!(manifest["critical_total"], serde_json::json!(1));
        assert_eq!(manifest["floor"], serde_json::json!(["docs/knowledge/crit.md"]));
        let reasons: Vec<&str> = manifest["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["reason"].as_str().unwrap())
            .collect();
        assert_eq!(reasons[0], "work item");
        assert_eq!(reasons[1], "plan sibling in work/w1/");
        assert_eq!(reasons[2], "required_context depth 1 via work/w1/item.md");
        assert!(reasons[3].starts_with("critical pattern (relevance "));
        assert!(reasons[3].ends_with(", rank 1 of 1, floor)"));
        assert_eq!(reasons[4], "decision for area billing");

        // Zero budget still includes nothing — hard ceiling.
        let manifest0 = match build_context_manifest(&dir, "w1", 0.0, &json_raw("0")) {
            ManifestOut::Built(m) => m,
            _ => panic!("expected manifest"),
        };
        assert!(manifest0["entries"].as_array().unwrap().is_empty());
        assert_eq!(manifest0["truncated"].as_array().unwrap().len(), 5);

        // Unknown work id throws the typed error.
        match build_context_manifest(&dir, "nope", 100.0, &json_raw("100")) {
            ManifestOut::Thrown(msg) => assert!(msg.contains("unknown_work")),
            _ => panic!("expected thrown"),
        }

        // bad_budget quotes the RAW CLI string, JSON.stringify-style.
        match build_context_manifest(&dir, "w1", -5.0, &json_raw("-5")) {
            ManifestOut::Thrown(msg) => assert!(msg.contains("got \"-5\" (D27)")),
            _ => panic!("expected thrown"),
        }
    }

    fn json_raw(s: &str) -> Value {
        Value::String(s.to_string())
    }

    #[test]
    fn to_fixed6_matches_number_tofixed_shape() {
        assert_eq!(to_fixed6(0.05), 0.05);
        assert_eq!(to_fixed6(0.1234567), 0.123457);
        assert_eq!(to_fixed6(0.0), 0.0);
    }

    // ── knowledge promote ──────────────────────────────────────────────────

    #[test]
    fn compare_cell_ids_is_natural_order() {
        use std::cmp::Ordering::*;
        let mut ids = vec!["okf-10", "okf-9", "okf-1", "okf-2b", "okf-2a", "zz", "okf"];
        ids.sort_by(|a, b| compare_cell_ids(a, b));
        assert_eq!(ids, vec!["okf", "okf-1", "okf-2a", "okf-2b", "okf-9", "okf-10", "zz"]);
        assert_eq!(compare_cell_ids("a1", "a1"), Equal);
        assert_eq!(compare_cell_ids("a01", "a1"), Equal); // Number('01') === 1
        assert_eq!(compare_cell_ids("a", "a1"), Less);    // shorter split runs out
        assert_eq!(compare_cell_ids("a1", "a"), Greater);
    }

    #[test]
    fn one_line_collapses_whitespace_and_caps() {
        assert_eq!(one_line("  a\n\tb   c  ", 0), "a b c");
        assert_eq!(one_line("", 0), "");
        assert_eq!(one_line("abcdef", 4), "abc\u{2026}");
        assert_eq!(one_line("abcd", 4), "abcd"); // exactly at the limit
        assert_eq!(strip_one_trailing_newline("x\n"), "x");
        assert_eq!(strip_one_trailing_newline("x\n\n"), "x\n");
    }

    #[test]
    fn deviation_text_handles_both_recorded_shapes() {
        assert_eq!(deviation_text(&json!("plain")), "plain");
        assert_eq!(
            deviation_text(&json!({"type": "scope", "description": "why"})),
            "scope: why"
        );
        assert_eq!(deviation_text(&json!({"description": "why"})), "why");
        assert_eq!(deviation_text(&json!({"note": "x"})), r#"{"note":"x"}"#);
        assert_eq!(deviation_text(&json!(["a"])), r#"["a"]"#);
        assert_eq!(deviation_text(&json!(7)), "7");
    }

    #[test]
    fn verify_summary_prefers_the_recorded_keys() {
        let ev = |raw: &str| verify_summary(&json!({"verification_evidence": raw}));
        assert_eq!(ev("").unwrap(), "");
        assert_eq!(ev("   ").unwrap(), "");
        assert_eq!(ev(r#"{"summary":"s","verify_tail":"t"}"#).unwrap(), "t"); // key order fixed
        assert_eq!(ev(r#"{"evidence":"e"}"#).unwrap(), "e");
        assert_eq!(ev(r#"{"other":"x"}"#).unwrap(), r#"{"other":"x"}"#);
        assert_eq!(ev("just text  here").unwrap(), "just text here");
        assert_eq!(verify_summary(&json!({})).unwrap(), "");
        // CUTOVER: JSON-looking text this CLI cannot parse used to delegate
        // ("only V8 knows which branch ran"). With one parser left, the catch
        // branch IS the answer — the raw text, one-lined.
        assert_eq!(ev(r#"{"a":"\ud800"}"#).unwrap(), r#"{"a":"\ud800"}"#);
        assert_eq!(ev("{not json").unwrap(), "{not json");
    }

    #[test]
    fn iso_date_and_touches_subject() {
        assert_eq!(iso_date(Some(&json!("2024-06-02T10:00:00Z"))).as_deref(), Some("2024-06-02"));
        assert_eq!(iso_date(Some(&json!("2024-06-02"))).as_deref(), Some("2024-06-02"));
        assert_eq!(iso_date(Some(&json!("2024-6-2"))), None);
        assert_eq!(iso_date(Some(&json!(20240602))), None);
        assert_eq!(iso_date(None), None);
        assert!(touches_subject("src/cli/main.rs", "src/cli"));
        assert!(touches_subject("src/cli", "src/cli/main.rs"));
        assert!(touches_subject("a", "a"));
        assert!(!touches_subject("src/clix/main.rs", "src/cli"));
    }

    #[test]
    fn promotion_mines_capped_traces_and_proposes_without_writing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let kn = root.join("docs").join("knowledge");
        std::fs::create_dir_all(kn.join("work")).unwrap();
        std::fs::create_dir_all(kn.join("areas")).unwrap();
        std::fs::write(
            kn.join("work").join("w1.md"),
            "---\ntype: bee.work-item\ntitle: Widget work\ndescription: does widgets\ntags: [alpha]\nbee:\n  id: w1\n  lifecycle: active\n  areas: [cli]\n  lane: small\n---\n\n# Widget work\n",
        )
        .unwrap();
        std::fs::write(
            kn.join("areas").join("cli.md"),
            "---\ntype: bee.area\ntitle: CLI\ndescription: the cli\nbee:\n  id: a-cli\n  lifecycle: active\n  areas: [cli]\n  sources: [\"src/cli\"]\n---\n\n# CLI\n",
        )
        .unwrap();
        let cells = root.join(".bee").join("cells");
        std::fs::create_dir_all(cells.join("archive")).unwrap();
        std::fs::write(
            cells.join("w1-10.json"),
            r#"{"id":"w1-10","feature":"w1","status":"capped","title":"tenth","verify":"cargo test","trace":{"behavior_change":true,"outcome":"did   the  thing","files_changed":["src/cli/main.rs","other.rs"],"deviations":["dev one",{"type":"scope","description":"dev two"},"  "],"attempts":[{"failure_signature":"boom"}],"capped_at":"2024-06-02T10:00:00.000Z","verification_evidence":"{\"verify_tail\":\"green\"}"}}"#,
        )
        .unwrap();
        std::fs::write(
            cells.join("w1-9.json"),
            r#"{"id":"w1-9","feature":"w1","status":"capped","title":"ninth","verify":"npm test","behavior_change":true,"trace":{"files_changed":["src/store/x.rs"],"capped_at":"2024-06-01T00:00:00.000Z"}}"#,
        )
        .unwrap();
        std::fs::write(
            cells.join("w1-3.json"),
            r#"{"id":"w1-3","feature":"w1","status":"open","title":"open cell"}"#,
        )
        .unwrap();
        std::fs::write(
            cells.join("archive").join("w1-99.json"),
            r#"{"id":"w1-99","feature":"w1","status":"capped","title":"archived"}"#,
        )
        .unwrap();

        let Some(Promo::Ok(p)) = build_promotion(root, &kn, "w1") else {
            panic!("expected a proposal")
        };
        // Natural id order; the archive subdir and the open cell never appear.
        let ids: Vec<&str> = p["cells"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["w1-9", "w1-10"]);
        assert_eq!(p["writes"], json!([]));
        assert_eq!(p["work_item"], "work/w1.md");
        assert_eq!(p["delivery"]["path"], "work/delivery.md");
        assert_eq!(p["delivery"]["repo_path"], "docs/knowledge/work/delivery.md");
        // trace.outcome wins over the title; the fallback is the title.
        assert_eq!(p["cells"][1]["outcome"], "did   the  thing");
        assert_eq!(p["cells"][0]["outcome"], "ninth");
        assert_eq!(p["cells"][1]["verify_summary"], "green");
        // behavior_change: trace.true, and the cell-level fallback.
        assert_eq!(p["cells"][0]["behavior_change"], true);
        // Timestamp = the LATEST capped date.
        let content = p["delivery"]["content"].as_str().unwrap();
        assert!(content.contains("timestamp: 2024-06-02"));
        assert!(content.starts_with("---\ntype: bee.delivery\n"));
        assert!(content.contains("bee:\n  id: w1-delivery\n  lifecycle: active\n  areas: [cli]\n"));
        assert!(content.contains("lane: small"));
        assert!(content.contains("- **w1-10** — did the thing (2 file(s) changed)"));
        assert!(content.contains("- **w1-10** — `cargo test` — green"));
        assert!(content.contains("- **w1-10** — scope: dev two"));

        // Area bullets: only the behavior_change cell whose files touch the
        // area subjects (src/cli via the area concept's sources).
        let areas = p["area_updates"].as_array().unwrap();
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0]["area"], "cli");
        assert_eq!(
            areas[0]["subjects"],
            json!(["docs/knowledge/areas/cli.md", "docs/knowledge/work/w1.md", "src/cli"])
        );
        assert_eq!(areas[0]["bullets"].as_array().unwrap().len(), 1);
        assert_eq!(areas[0]["bullets"][0]["files"], json!(["src/cli/main.rs"]));

        // Pattern candidates: only cells carrying a deviation or a signature.
        let cands = p["pattern_candidates"].as_array().unwrap();
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0]["cell"], "w1-10");
        assert_eq!(cands[0]["repo_path"], "docs/knowledge/patterns/w1-w1-10-pitfall.md");
        assert_eq!(
            cands[0]["evidence"],
            json!([
                {"kind": "deviation", "text": "dev one"},
                {"kind": "deviation", "text": "scope: dev two"},
                {"kind": "failure_signature", "text": "boom"},
            ])
        );
        assert!(cands[0]["content"].as_str().unwrap().contains("polarity: pitfall"));

        // Nothing was written anywhere under docs/knowledge/.
        assert!(!kn.join("work").join("delivery.md").exists());
        assert!(!kn.join("patterns").exists());

        // Typed refusals.
        assert!(matches!(
            build_promotion(root, &kn, "   "),
            Some(Promo::Thrown(m)) if m == "knowledge promote: missing_work — --work <id> is required (D38)."
        ));
        assert!(matches!(
            build_promotion(root, &kn, "nope"),
            Some(Promo::Thrown(m)) if m.starts_with("knowledge promote: unknown_work — no bee.work-item concept")
        ));
    }

    // ═══ R5: fixture builders ══════════════════════════════════════════════
    //
    // Node oracle: tests/test_knowledge.mjs makeRepo / writeBundleFile /
    // conceptText (l.60–108). Fixtures are authored THROUGH `emit_frontmatter`
    // — D12 makes the emitter the subset's source of truth — so every fixture
    // is canonical by construction and `not_canonical` can only fire where a
    // test bends the bytes on purpose.

    fn bundle() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        (tmp, dir)
    }

    fn write_bundle_file(dir: &Path, rel: &str, text: &str) {
        let abs = join_rel(dir, rel);
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(abs, text).unwrap();
    }

    struct Cx {
        ty: &'static str,
        title: String,
        description: Option<String>,
        id: String,
        lifecycle: &'static str,
        tags: Vec<String>,
        areas: Vec<String>,
        bee_extra: Vec<(&'static str, Value)>,
        body: String,
    }

    impl Cx {
        fn new(id: &str) -> Self {
            Cx {
                ty: "bee.pattern",
                title: "A demo pattern".into(),
                description: Some("A canonical fixture concept".into()),
                id: id.into(),
                lifecycle: "active",
                tags: vec!["demo".into()],
                areas: vec!["demo-area".into()],
                bee_extra: Vec::new(),
                body: "Body.".into(),
            }
        }
        fn ty(mut self, t: &'static str) -> Self {
            self.ty = t;
            self
        }
        fn title(mut self, t: &str) -> Self {
            self.title = t.into();
            self
        }
        fn description(mut self, d: &str) -> Self {
            self.description = Some(d.into());
            self
        }
        fn no_description(mut self) -> Self {
            self.description = None;
            self
        }
        fn lifecycle(mut self, l: &'static str) -> Self {
            self.lifecycle = l;
            self
        }
        fn tags(mut self, t: &[&str]) -> Self {
            self.tags = t.iter().map(|s| (*s).to_string()).collect();
            self
        }
        fn areas(mut self, a: &[&str]) -> Self {
            self.areas = a.iter().map(|s| (*s).to_string()).collect();
            self
        }
        fn body(mut self, b: &str) -> Self {
            self.body = b.into();
            self
        }
        fn bee(mut self, key: &'static str, value: Value) -> Self {
            self.bee_extra.push((key, value));
            self
        }
        fn critical(self) -> Self {
            self.bee("critical", json!(true))
        }

        fn text(&self) -> String {
            let mut data = Map::new();
            data.insert("type".into(), json!(self.ty));
            data.insert("title".into(), json!(self.title));
            if let Some(d) = &self.description {
                data.insert("description".into(), json!(d));
            }
            data.insert("tags".into(), json!(self.tags));
            data.insert("timestamp".into(), json!("2026-07-22"));
            let mut bee = Map::new();
            bee.insert("id".into(), json!(self.id));
            bee.insert("lifecycle".into(), json!(self.lifecycle));
            bee.insert("areas".into(), json!(self.areas));
            bee.insert("required_context".into(), json!([]));
            bee.insert("decisions".into(), json!([]));
            bee.insert("sources".into(), json!([]));
            for (k, v) in &self.bee_extra {
                bee.insert((*k).to_string(), v.clone());
            }
            data.insert("bee".into(), Value::Object(bee));
            format!("{}\n# {}\n\n{}\n", emit_frontmatter(&data).unwrap(), self.title, self.body)
        }
    }

    fn put(dir: &Path, rel: &str, c: Cx) {
        write_bundle_file(dir, rel, &c.text());
    }

    fn codes(list: &[Value]) -> Vec<&str> {
        list.iter().map(|f| f["code"].as_str().unwrap()).collect()
    }

    fn of_code<'a>(list: &'a [Value], code: &str) -> Vec<&'a Value> {
        list.iter().filter(|f| f["code"] == code).collect()
    }

    fn msg(f: &Value) -> &str {
        f["message"].as_str().unwrap()
    }

    // ═══ profile WARNINGS (D4) ═════════════════════════════════════════════

    /// Node: 'profile warning: type outside the D18 nine warns, does not
    /// error, exits ok un-strict' (test_knowledge.mjs l.271).
    #[test]
    fn unknown_type_warns_without_erroring_and_a_profile_type_stays_silent() {
        let (_tmp, dir) = bundle();
        put(&dir, "patterns/guide.md", Cx::new("guide-1").ty("bee.guide"));
        put(&dir, "patterns/known.md", Cx::new("known-1")); // control: bee.pattern is in the nine
        let report = check_bundle(&dir, false).unwrap();
        assert!(
            report.okf_errors.is_empty(),
            "an unknown type is a SHOULD, never an OKF error: {:?}",
            codes(&report.okf_errors)
        );
        let warns = of_code(&report.warnings, "unknown_type");
        assert_eq!(warns.len(), 1, "only the off-vocabulary file may warn: {:?}", report.warnings);
        assert_eq!(warns[0]["file"], "patterns/guide.md");
        assert!(msg(warns[0]).contains("bee.guide"), "the offending type must flow into the message: {}", msg(warns[0]));
        assert!(report.ok, "warnings alone must not fail un-strict");
    }

    /// Node: 'profile warning: missing profile-required field (D10: never
    /// invented, warned by name)' (l.281). The nested `bee.id`/`bee.lifecycle`
    /// paths exercise readPath's object walk, which the flat cases cannot.
    #[test]
    fn missing_profile_field_warns_by_name_and_a_complete_concept_stays_silent() {
        let (_tmp, dir) = bundle();
        put(&dir, "patterns/complete.md", Cx::new("complete")); // control: all four present
        put(&dir, "patterns/undescribed.md", Cx::new("undescribed").no_description());
        // No `bee:` map at all — readPath stops mid-walk on both nested keys.
        write_bundle_file(
            &dir,
            "patterns/nobee.md",
            "---\ntype: bee.pattern\ntitle: No bee map\ndescription: Has no bee map\n---\nBody.\n",
        );
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.okf_errors.is_empty(), "a missing profile field is never an OKF error: {:?}", codes(&report.okf_errors));
        let warns = of_code(&report.warnings, "missing_profile_field");
        // Walk order: files path-sorted, keys in PROFILE_REQUIRED order.
        let got: Vec<(&str, &str)> = warns.iter().map(|w| (w["file"].as_str().unwrap(), msg(w))).collect();
        assert_eq!(got.len(), 3, "exactly the three absent fields may warn: {:?}", report.warnings);
        assert_eq!(got[0].0, "patterns/nobee.md");
        assert!(got[0].1.contains("\"bee.id\""), "{}", got[0].1);
        assert_eq!(got[1].0, "patterns/nobee.md");
        assert!(got[1].1.contains("\"bee.lifecycle\""), "{}", got[1].1);
        assert_eq!(got[2].0, "patterns/undescribed.md");
        assert!(got[2].1.contains("\"description\""), "{}", got[2].1);
        assert!(report.ok, "a missing profile field is a warning, green un-strict");
    }

    /// Node: 'profile warning: dangling required_context path; a resolving
    /// path stays silent' (l.290).
    #[test]
    fn dangling_required_context_warns_only_for_the_unresolvable_target() {
        let (_tmp, dir) = bundle();
        put(&dir, "areas/demo/overview.md", Cx::new("demo-overview").ty("bee.area"));
        put(
            &dir,
            "patterns/linked.md",
            Cx::new("linked").bee(
                "required_context",
                // one resolving target (the control) + one ghost
                json!(["areas/demo/overview.md", "areas/ghost/nothing.md"]),
            ),
        );
        let report = check_bundle(&dir, false).unwrap();
        let dangling = of_code(&report.warnings, "dangling_required_context");
        assert_eq!(dangling.len(), 1, "only the ghost path may warn: {:?}", report.warnings);
        assert_eq!(dangling[0]["file"], "patterns/linked.md");
        assert!(
            msg(dangling[0]).contains("areas/ghost/nothing.md"),
            "the unresolved target must be named: {}",
            msg(dangling[0])
        );
        assert!(report.ok);
    }

    /// Node: 'profile warning: dangling supersedes id; a resolving id stays
    /// silent' (l.300).
    #[test]
    fn dangling_supersedes_warns_only_for_the_id_no_concept_claims() {
        let (_tmp, dir) = bundle();
        put(&dir, "patterns/old.md", Cx::new("old-pattern").lifecycle("superseded"));
        put(&dir, "patterns/new.md", Cx::new("new-pattern").bee("supersedes", json!("old-pattern")));
        put(&dir, "patterns/orphan.md", Cx::new("orphan").bee("supersedes", json!("never-existed")));
        let report = check_bundle(&dir, false).unwrap();
        let dangling = of_code(&report.warnings, "dangling_supersedes");
        assert_eq!(dangling.len(), 1, "the resolving supersedes must stay silent: {:?}", report.warnings);
        assert_eq!(dangling[0]["file"], "patterns/orphan.md");
        assert!(msg(dangling[0]).contains("never-existed"), "{}", msg(dangling[0]));
        assert!(report.ok);
    }

    /// Node: 'profile warning: duplicate bee.id (D31: id is globally unique)'
    /// (l.310). A duplicate id is a WARNING — the pair to the authority ERROR
    /// below, which fails the chain on its own.
    #[test]
    fn duplicate_id_warns_and_names_every_claimant() {
        let (_tmp, dir) = bundle();
        put(&dir, "patterns/a.md", Cx::new("same-id"));
        put(&dir, "patterns/b.md", Cx::new("same-id"));
        put(&dir, "patterns/c.md", Cx::new("unique-id")); // control: never named
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.profile_errors.is_empty(), "a duplicate id is not a profile error: {:?}", codes(&report.profile_errors));
        let dup = of_code(&report.warnings, "duplicate_id");
        assert_eq!(dup.len(), 1, "one finding per duplicated id: {:?}", report.warnings);
        assert_eq!(dup[0]["file"], "patterns/a.md", "the finding is filed against the first claimant");
        let m = msg(dup[0]);
        assert!(m.contains("same-id"), "{m}");
        assert!(m.contains("patterns/a.md") && m.contains("patterns/b.md"), "both claimants must be traceable: {m}");
        assert!(!m.contains("patterns/c.md"), "the unique id must not be dragged in: {m}");
        assert!(report.ok, "a duplicate id alone stays green un-strict");
    }

    // ═══ profile ERRORS (G14 layer 3 / cell f3-3) ══════════════════════════

    /// Node: 'profile ERROR: duplicate bee.authoritative_for FAILS the chain'
    /// (l.337) + 'grouped by the HARDENED subject' (l.352). The chain runs
    /// `knowledge check` WITHOUT --strict, so this must be an error, not a
    /// warning promoted only under strict.
    #[test]
    fn duplicate_authoritative_for_is_a_chain_failing_error_over_the_hardened_subject() {
        // Control: two DIFFERENT subjects stay green — the grouping is not a
        // blanket "two claims anywhere" rule.
        {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!("gates")));
            put(&dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!("locks")));
            let report = check_bundle(&dir, false).unwrap();
            assert!(report.profile_errors.is_empty(), "distinct subjects: {:?}", report.profile_errors);
            assert!(report.ok);
        }
        // Every ASCII spelling that normalizeSubject folds onto "gates".
        for second in ["gates", "gates.", "  GATES  ", "Gates!", "GATES---"] {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!("gates")));
            put(&dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!(second)));
            let report = check_bundle(&dir, false).unwrap();
            let dup = of_code(&report.profile_errors, "duplicate_authoritative_for");
            assert_eq!(dup.len(), 1, "{second:?}: exact-string grouping misses this, hardened grouping must not: {:?}", report.profile_errors);
            assert_eq!(dup[0]["file"], "areas/x/one.md");
            let m = msg(dup[0]);
            assert!(m.contains("areas/x/one.md") && m.contains("areas/x/two.md"), "{second:?}: both claimants must be named: {m}");
            assert!(m.contains("\"gates\""), "{second:?}: the RAW claim is quoted, not the normalized key: {m}");
            assert!(
                !report.ok,
                "{second:?}: a forked subject must fail the chain with no --strict"
            );
            assert!(
                of_code(&report.warnings, "duplicate_authoritative_for").is_empty(),
                "{second:?}: promoted to profile.errors, not duplicated across buckets"
            );
        }
    }

    /// Node's hardened grouping folds NFKC + confusables (l.352: Cyrillic 'а',
    /// fullwidth). This port used to model only the ASCII-identity slice and
    /// DELEGATE a non-ASCII claim; it now answers natively, so a homoglyph can
    /// no longer buy a second authority for an already-owned subject.
    #[test]
    fn a_homoglyph_authority_claim_is_caught_natively_as_a_duplicate() {
        // Every non-ASCII spelling that normalizeSubject folds onto "gates".
        for second in [
            "g\u{430}tes",                                   // Cyrillic 'а'
            "\u{ff47}\u{ff41}\u{ff54}\u{ff45}\u{ff53}",      // fullwidth
            "\u{1d420}ates",                                 // math bold 𝐠
            "G\u{430}TES.",                                  // homoglyph + case + punctuation
            "g\u{3b1}t\u{435}s",                             // Greek 'α' + Cyrillic 'е'
        ] {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!("gates")));
            put(&dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!(second)));
            let report = check_bundle(&dir, false)
                .unwrap_or_else(|| panic!("{second:?}: a non-ASCII claim must be ANSWERED, not delegated"));
            let dup = of_code(&report.profile_errors, "duplicate_authoritative_for");
            assert_eq!(dup.len(), 1, "{second:?}: {:?}", report.profile_errors);
            assert_eq!(dup[0]["file"], "areas/x/one.md");
            let m = msg(dup[0]);
            assert!(m.contains("areas/x/one.md") && m.contains("areas/x/two.md"), "{second:?}: {m}");
            assert!(m.contains(&format!("\"{second}\"")), "{second:?}: the RAW claim is quoted: {m}");
            assert!(!report.ok, "{second:?}: a forked subject must fail the chain");
        }

        // A diacritic is likewise not identity — and the control beside it: a
        // genuinely different subject in another script is NOT a duplicate.
        for (a, b, is_dup) in [
            ("caf\u{e9}", "cafe", true),
            ("caf\u{e9}", "cafe\u{301}", true),
            ("gates", "\u{448}\u{43b}\u{44e}\u{437}\u{44b}", false),
        ] {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!(a)));
            put(&dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!(b)));
            let report = check_bundle(&dir, false).expect("answered natively");
            assert_eq!(
                of_code(&report.profile_errors, "duplicate_authoritative_for").len(),
                usize::from(is_dup),
                "{a:?} vs {b:?}"
            );
        }
    }

    /// Node: 'profile ERROR: a MALFORMED bee.authoritative_for is a
    /// chain-failing error naming the file, never a silent skip' (l.369). The
    /// reachable set is measured against the D12 parser: `42`/`null` parse as
    /// STRINGS, and a mapping is already an unparseable_frontmatter OKF error.
    #[test]
    fn malformed_authoritative_for_is_a_chain_failing_error_naming_the_got_type() {
        for (literal, got) in [
            (json!(["gates", "locks"]), "array"),
            (json!(true), "boolean"),
            (json!(""), "string"),
            (json!("   "), "string"),
        ] {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/bad.md", Cx::new("x-bad").ty("bee.area").bee("authoritative_for", literal.clone()));
            let report = check_bundle(&dir, false).unwrap();
            assert!(
                report.okf_errors.is_empty(),
                "{literal}: the frontmatter itself parses — this is a profile fault, not an OKF one: {:?}",
                report.okf_errors
            );
            let bad = of_code(&report.profile_errors, "malformed_authoritative_for");
            assert_eq!(bad.len(), 1, "{literal}: {:?}", report.profile_errors);
            assert_eq!(bad[0]["file"], "areas/x/bad.md");
            assert!(msg(bad[0]).contains(&format!("(got {got})")), "{literal}: {}", msg(bad[0]));
            assert!(!report.ok, "{literal}: a claim bee cannot read must fail the chain");
        }
        // Control: a well-formed claim produces neither finding.
        let (_tmp, dir) = bundle();
        put(&dir, "areas/x/good.md", Cx::new("x-good").ty("bee.area").bee("authoritative_for", json!("gates")));
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.profile_errors.is_empty(), "{:?}", report.profile_errors);
        assert!(report.ok);
    }

    // ═══ --strict (D4/D13) ═════════════════════════════════════════════════

    /// Node: 'strict flip: a warnings-only bundle is ok un-strict and not ok
    /// under strict' (l.473) + 'CLI (f3-3): a duplicated authority exits
    /// NON-ZERO with no --strict' (l.511). `run_check` turns `!report.ok`
    /// straight into the exit code (l.1815/1862), so `ok` IS the exit-code
    /// contract at this level.
    #[test]
    fn strict_flips_a_warnings_only_bundle_but_an_authority_error_fails_without_it() {
        let (_tmp, warn_dir) = bundle();
        put(&warn_dir, "patterns/guide.md", Cx::new("guide-1").ty("bee.guide"));
        let loose = check_bundle(&warn_dir, false).unwrap();
        assert!(loose.okf_errors.is_empty() && loose.profile_errors.is_empty());
        assert!(!loose.warnings.is_empty(), "the fixture must actually warn or the flip proves nothing");
        assert!(loose.ok, "un-strict passes on warnings only");
        assert!(!check_bundle(&warn_dir, true).unwrap().ok, "--strict fails on any finding");

        // Control: strict must not invent a failure on a clean bundle.
        let (_tmp2, clean_dir) = bundle();
        put(&clean_dir, "patterns/clean.md", Cx::new("clean"));
        assert!(check_bundle(&clean_dir, false).unwrap().ok);
        assert!(check_bundle(&clean_dir, true).unwrap().ok, "--strict is a warning promoter, not a new check");

        // A forked authority is non-zero WITHOUT --strict.
        let (_tmp3, dup_dir) = bundle();
        put(&dup_dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!("gates")));
        put(&dup_dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!("gates")));
        let dup = check_bundle(&dup_dir, false).unwrap();
        assert!(dup.warnings.is_empty(), "nothing in this bundle is a mere warning: {:?}", dup.warnings);
        assert_eq!(codes(&dup.profile_errors), vec!["duplicate_authoritative_for"]);
        assert!(!dup.ok, "the fork fails the chain with the flag absent");
    }

    // ═══ round-trip guard: not_canonical (D12) ═════════════════════════════

    /// Node: round-trip guard for an unquoted colon (l.416), a mid-value '#'
    /// (l.429) and CRLF (l.441), plus 'a fully canonical bundle yields zero
    /// not_canonical warnings' (l.452). Each bend must keep the DATA intact
    /// and warn — a silent misparse is the failure this guard exists to stop.
    #[test]
    fn not_canonical_warns_on_bent_bytes_and_a_canonical_bundle_warns_zero_times() {
        let canonical = Cx::new("bent").text();
        for (rel, bent, expected_title) in [
            (
                "patterns/colon.md",
                canonical.replace("title: A demo pattern", "title: Routing: the golden rule"),
                "Routing: the golden rule",
            ),
            (
                "patterns/hash.md",
                canonical.replace("title: A demo pattern", "title: value # not a comment"),
                "value # not a comment",
            ),
            ("patterns/crlf.md", canonical.replace('\n', "\r\n"), "A demo pattern"),
        ] {
            let (_tmp, dir) = bundle();
            write_bundle_file(&dir, rel, &bent);
            let report = check_bundle(&dir, false).unwrap();
            assert!(report.okf_errors.is_empty(), "{rel}: bent bytes are a profile warning, never an OKF error: {:?}", report.okf_errors);
            let warns = of_code(&report.warnings, "not_canonical");
            assert_eq!(warns.len(), 1, "{rel}: {:?}", report.warnings);
            assert_eq!(warns[0]["file"], rel);
            // The value survived the bend intact — never comment-stripped,
            // never truncated at the colon.
            let data = parse_ok(&bent);
            assert_eq!(data["title"], json!(expected_title), "{rel}");
            assert_eq!(data["bee"]["id"], json!("bent"), "{rel}");
        }
        // Control: the same concept, unbent, warns zero times.
        let (_tmp, dir) = bundle();
        write_bundle_file(&dir, "patterns/clean.md", &canonical);
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.warnings.is_empty(), "a canonical file must not warn: {:?}", report.warnings);
        assert!(report.ok);
    }

    // ═══ knowledge index (D21) ═════════════════════════════════════════════

    /// makeIndexFixture (test_knowledge.mjs l.573): nested dirs, one critical,
    /// one plain, and a log.md with an ISO heading.
    fn index_fixture(dir: &Path) {
        put(
            dir,
            "areas/demo/overview.md",
            Cx::new("demo-overview")
                .ty("bee.area")
                .title("Demo overview")
                .description("Overview of the demo area")
                .areas(&["routing"])
                .bee("authoritative_for", json!("demo-overview")),
        );
        put(
            dir,
            "areas/demo/rules.md",
            Cx::new("demo-rules")
                .ty("bee.area")
                .title("Demo rules")
                .description("Rules of the demo area")
                .lifecycle("draft")
                .areas(&["routing"])
                .bee("authoritative_for", json!("demo-rules")),
        );
        put(
            dir,
            "patterns/critical-one.md",
            Cx::new("critical-one").title("A critical pattern").description("Always in context").critical(),
        );
        put(dir, "patterns/plain.md", Cx::new("plain-one").title("A plain pattern").description("Not critical"));
        write_bundle_file(dir, "log.md", "# Log\n\n## 2026-07-22\n\n- Fixture bundle created.\n");
    }

    /// Node: 'index generates an index at every level ... two consecutive runs
    /// are byte-identical, LF-only' (l.594) + 'index --check exits non-zero
    /// naming a doctored stale index; regeneration heals it' (l.614).
    ///
    /// `run_index --check` (l.1877-1884) is `compute_index_files` plus a
    /// read-and-compare per file; it cannot be entered without a process cwd,
    /// so both production halves are driven directly and only the three-line
    /// join is written here.
    #[test]
    fn index_check_flags_exactly_the_doctored_index_and_regeneration_heals_it() {
        let (_tmp, dir) = bundle();
        index_fixture(&dir);
        let expected = compute_index_files(&dir).unwrap();
        let rels: Vec<&str> = expected.iter().map(|(r, _)| r.as_str()).collect();
        assert_eq!(rels, vec!["index.md", "areas/index.md", "areas/demo/index.md", "patterns/index.md"]);
        for (rel, content) in &expected {
            assert!(!content.contains('\r'), "{rel}: a generated index is LF-only");
            let has_clock = content.as_bytes().windows(8).any(|w| {
                w[0].is_ascii_digit()
                    && w[1].is_ascii_digit()
                    && w[2] == b':'
                    && w[3].is_ascii_digit()
                    && w[4].is_ascii_digit()
                    && w[5] == b':'
                    && w[6].is_ascii_digit()
                    && w[7].is_ascii_digit()
            });
            assert!(!has_clock, "{rel}: a generated index carries no wall-clock value");
        }
        assert_eq!(compute_index_files(&dir).unwrap(), expected, "recomputation must be byte-identical");

        // Render, exactly as run_index's write loop does.
        for (rel, content) in &expected {
            write_bundle_file(&dir, rel, content);
        }
        let stale = |exp: &Vec<(String, String)>| -> Vec<String> {
            exp.iter()
                .filter(|(rel, content)| read_file_lossy(&join_rel(&dir, rel)).ok().as_deref() != Some(content.as_str()))
                .map(|(rel, _)| format!("docs/knowledge/{rel}"))
                .collect()
        };
        assert!(stale(&expected).is_empty(), "a fresh render is not stale: {:?}", stale(&expected));

        // Doctor exactly one index. index.md is a reserved basename, so the
        // expected SET must not move — only that one file goes stale.
        let doctored = join_rel(&dir, "areas/demo/index.md");
        let bent = format!("{}\nHand-edited drift.\n", read_file_lossy(&doctored).unwrap());
        std::fs::write(&doctored, &bent).unwrap();
        let after = compute_index_files(&dir).unwrap();
        assert_eq!(after, expected, "a hand-edited index is not a concept and cannot change the expected set");
        assert_eq!(stale(&after), vec!["docs/knowledge/areas/demo/index.md"]);

        // Regeneration heals it.
        for (rel, content) in &after {
            write_bundle_file(&dir, rel, content);
        }
        assert!(stale(&after).is_empty(), "regeneration must clear the drift: {:?}", stale(&after));
    }

    /// Node: 'generated non-root indexes carry NO frontmatter — only the HTML
    /// provenance comment' (l.631) + 'generated root index keeps
    /// okf_version-only frontmatter ... and the generated bundle passes
    /// knowledge check' (l.644).
    #[test]
    fn generated_indexes_obey_the_okf_frontmatter_rules_and_pass_check() {
        let (_tmp, dir) = bundle();
        index_fixture(&dir);
        let expected = compute_index_files(&dir).unwrap();
        for (rel, content) in &expected {
            if rel == "index.md" {
                let Fm::Parsed { data, .. } = parse_frontmatter(content) else {
                    panic!("the root index must carry frontmatter");
                };
                assert_eq!(
                    data.keys().map(String::as_str).collect::<Vec<_>>(),
                    vec!["okf_version"],
                    "root index frontmatter carries ONLY okf_version"
                );
                assert_eq!(data["okf_version"], json!("0.1"));
            } else {
                assert!(matches!(parse_frontmatter(content), Fm::Absent), "{rel}: a non-root index must carry no frontmatter");
                assert!(content.starts_with("<!--"), "{rel}: must open with the provenance comment");
            }
            // PINNED PROSE: D21 makes the provenance header part of the
            // artifact's contract (the Node oracle asserts it at l.639-640),
            // so these two strings are asserted deliberately.
            assert!(content.contains("GENERATED FILE — do not hand-edit"), "{rel}");
            assert!(content.contains("bee knowledge index"), "{rel}: the regenerate command must be named");
        }

        // Rendered into the bundle, the generated indexes keep check green —
        // the control proving check_index_file agrees with what index emits.
        for (rel, content) in &expected {
            write_bundle_file(&dir, rel, content);
        }
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.okf_errors.is_empty(), "generated indexes must produce zero OKF errors: {:?}", report.okf_errors);
        assert!(report.ok);

        // And the control proving check_index_file BITES: hand-added
        // frontmatter on a NON-root index is an OKF error.
        let patterns = expected.iter().find(|(r, _)| r == "patterns/index.md").unwrap();
        write_bundle_file(&dir, "patterns/index.md", &format!("---\nokf_version: 0.1\n---\n{}", patterns.1));
        let bent = check_bundle(&dir, false).unwrap();
        assert_eq!(codes(&bent.okf_errors), vec!["index_frontmatter"]);
        assert!(!bent.ok);
    }

    // ═══ knowledge context: the relevance-cut invariants (G5/G11) ══════════

    fn built(dir: &Path, work: &str, budget: f64) -> Value {
        match build_context_manifest(dir, work, budget, &json_raw(&format!("{budget}"))) {
            ManifestOut::Built(m) => m,
            ManifestOut::Thrown(m) => panic!("unexpected throw: {m}"),
            ManifestOut::NeedsNode => panic!("unexpected delegation"),
        }
    }

    fn entry_paths(manifest: &Value) -> Vec<String> {
        manifest["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["path"].as_str().unwrap().to_string())
            .collect()
    }

    fn str_list(v: &Value) -> Vec<String> {
        v.as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_string()).collect()
    }

    /// Node: 'knowledge context CONSERVES the critical set: entries +
    /// truncated + excluded == every bee.critical concept, no duplicates'
    /// (l.1360). buildContextManifest carries its own conservation guard; this
    /// asserts the PAYLOAD accounts for the set independently of it.
    #[test]
    fn context_conserves_the_critical_set_at_every_budget() {
        let (_tmp, dir) = bundle();
        put(
            &dir,
            "work/billing/work-item.md",
            Cx::new("billing-migration")
                .ty("bee.work-item")
                .title("Migrate the billing ledger onto the invoice schema")
                .description("Move every ledger row into the new billing schema behind a coverage gate.")
                .tags(&["billing", "ledger", "migration"])
                .areas(&["billing"])
                .body("Every ledger row is migrated into the invoice schema, one migration cell per ledger table."),
        );
        let critical_names = ["ledger-rows", "coverage-gate", "schema-rollback", "kiln-firing", "estuary-silt"];
        for name in critical_names {
            put(
                &dir,
                &format!("patterns/{name}.md"),
                Cx::new(name)
                    .title(&format!("{name} guidance"))
                    .description(&format!("{name} guidance for the ledger schema migration"))
                    .tags(&["pattern"])
                    .areas(&["billing"])
                    .critical()
                    .body(&format!("{name} guidance notes, technique and maintenance for a ledger row.")),
            );
        }
        let all: Vec<String> = critical_names.iter().map(|n| format!("docs/knowledge/patterns/{n}.md")).collect();

        for budget in [100000.0, 900.0, 0.0] {
            let manifest = built(&dir, "billing-migration", budget);
            let mut accounted: Vec<String> = entry_paths(&manifest);
            accounted.extend(str_list(&manifest["truncated"]));
            accounted.extend(
                manifest["excluded"].as_array().unwrap().iter().map(|e| e["path"].as_str().unwrap().to_string()),
            );
            accounted.retain(|p| all.contains(p));
            let unique: HashSet<&String> = accounted.iter().collect();
            assert_eq!(unique.len(), accounted.len(), "budget {budget}: a critical is accounted for exactly ONCE: {accounted:?}");
            assert_eq!(
                unique.len(),
                all.len(),
                "budget {budget}: CONSERVATION FAILED — {} criticals exist, {} accounted for",
                all.len(),
                unique.len()
            );
            assert_eq!(manifest["critical_total"], json!(all.len()), "budget {budget}: critical_total states the full population");
        }
    }

    /// Node: 'knowledge context FLOOR: the highest-scoring critical survives a
    /// budget that the plain prefix cut would have evicted it under' (l.1376).
    #[test]
    fn context_floor_keeps_the_top_criticals_a_plain_prefix_cut_would_evict() {
        let (_tmp, dir) = bundle();
        put(
            &dir,
            "work/billing/work-item.md",
            Cx::new("billing-migration")
                .ty("bee.work-item")
                .title("Migrate the billing ledger onto the invoice schema")
                .description("Move every ledger row into the new billing schema behind a coverage gate.")
                .tags(&["billing", "ledger", "migration"])
                .areas(&["billing"])
                .bee(
                    "required_context",
                    json!([
                        "areas/billing/ledger-schema.md",
                        "areas/billing/invoice-rows.md",
                        "areas/billing/rollback-runbook.md"
                    ]),
                )
                .body("Every ledger row is migrated into the invoice schema, one migration cell per ledger table."),
        );
        // The required_context chain is deliberately far larger than the floor:
        // under a plain prefix cut it eats the whole budget and every critical
        // is evicted, which is the failure the floor exists to stop.
        for name in ["ledger-schema", "invoice-rows", "rollback-runbook"] {
            put(
                &dir,
                &format!("areas/billing/{name}.md"),
                Cx::new(name)
                    .ty("bee.area")
                    .title(&format!("The {name}"))
                    .description(&format!("{name} reference"))
                    .tags(&["billing"])
                    .areas(&["billing"])
                    .body(&format!("{name} reference material. ").repeat(60)),
            );
        }
        for name in ["rel-ledger-rows", "rel-coverage-gate", "rel-schema-rollback", "irr-kiln-firing"] {
            put(
                &dir,
                &format!("patterns/{name}.md"),
                Cx::new(name)
                    .title(&format!("{name} guidance"))
                    .description(&format!("{name} guidance for the ledger schema migration"))
                    .tags(&["pattern"])
                    .areas(&["billing"])
                    .critical()
                    .body(&format!("{name} guidance notes and technique for a migrated ledger row.")),
            );
        }

        let full = built(&dir, "billing-migration", 100_000.0);
        let work_path = "docs/knowledge/work/billing/work-item.md";
        assert_eq!(full["entries"][0]["path"], work_path, "rank 1 is the work item");
        let floor = str_list(&full["floor"]);
        assert_eq!(floor.len(), FLOOR, "the floor is the pinned FLOOR: {floor:?}");
        let entries = full["entries"].as_array().unwrap();
        let top_critical = entries
            .iter()
            .find(|e| e["reason"].as_str().unwrap().starts_with("critical pattern"))
            .expect("a critical must be in entries at a large budget");
        assert!(floor.contains(&top_critical["path"].as_str().unwrap().to_string()), "the highest-scoring critical is in the floor");

        let est = |path: &str| -> f64 {
            entries.iter().find(|e| e["path"] == path).unwrap()["est_tokens"].as_f64().unwrap()
        };
        let work_cost = est(work_path);
        let floor_cost: f64 = floor.iter().map(|p| est(p)).sum();
        let req_cost: f64 = entries
            .iter()
            .filter(|e| e["reason"].as_str().unwrap().contains("required_context"))
            .map(|e| e["est_tokens"].as_f64().unwrap())
            .sum();
        assert!(
            req_cost > floor_cost,
            "the fixture must make the required_context chain the thing that would evict the floor ({req_cost} vs {floor_cost})"
        );

        // Exactly the work item plus the floor.
        let tight = work_cost + floor_cost;
        let cut = built(&dir, "billing-migration", tight);
        assert!(cut["total_est"].as_f64().unwrap() <= tight, "the budget stays a hard ceiling even with a floor");
        let cut_paths = entry_paths(&cut);
        for p in &floor {
            assert!(cut_paths.contains(p), "every floor critical must survive a tight budget; {p} was evicted from {cut_paths:?}");
        }
        assert_eq!(cut_paths[0], work_path, "the work item is never displaced by its own floor");
        assert_eq!(cut_paths.len(), 1 + floor.len(), "under this budget exactly the work item and the floor survive: {cut_paths:?}");
        let truncated = str_list(&cut["truncated"]);
        assert!(
            truncated.iter().any(|p| p.contains("areas/billing/")),
            "the floor must beat the higher-ranked required_context chain: {truncated:?}"
        );
    }

    /// Node: 'knowledge context FAILS when zero_signal_count exceeds the
    /// pinned threshold' (l.1423) + 'the zero-signal guard is inert below the
    /// pinned population floor' (l.1450).
    #[test]
    fn context_zero_signal_fails_above_the_population_floor_and_is_inert_below() {
        let work = || {
            Cx::new("signalless")
                .ty("bee.work-item")
                .title("Reconcile quarterly payroll withholding")
                .description("Withholding reconciliation across payroll periods.")
                .tags(&["payroll"])
                .areas(&["payroll"])
                .body("Payroll withholding reconciliation across quarterly periods, employer contributions included.")
        };
        let void = |topic: &str, i: usize| {
            Cx::new(&format!("void-{i}"))
                .title(&format!("{topic} guidance"))
                .description(&format!("{topic} guidance notes"))
                .tags(&["unrelated"])
                .areas(&["unrelated"])
                .critical()
                .body(&format!("{topic} guidance notes, {topic} technique, {topic} maintenance."))
        };
        let topics = [
            "kubernetes ingress",
            "sourdough hydration",
            "telescope collimation",
            "bicycle derailleur",
            "harpsichord tuning",
            "glacier moraine",
            "origami tessellation",
            "submarine ballast",
            "volcanic tephra",
            "lighthouse fresnel",
            "saffron cultivation",
            "permafrost drilling",
        ];

        let (_tmp, dir) = bundle();
        put(&dir, "work/lonely/work-item.md", work());
        for (i, topic) in topics.iter().enumerate() {
            put(&dir, &format!("patterns/void-{i:02}.md"), void(topic, i));
        }
        match build_context_manifest(&dir, "signalless", 100_000.0, &json_raw("100000")) {
            ManifestOut::Thrown(m) => {
                assert!(m.contains("zero_signal"), "the typed code must lead: {m}");
                assert!(m.contains("12 of 12"), "the measured counts must flow into the failure: {m}");
                assert!(m.contains("0.5"), "the pinned ratio must be named: {m}");
            }
            ManifestOut::Built(_) => panic!("an all-zero ranking must FAIL the run"),
            ManifestOut::NeedsNode => panic!("unexpected delegation"),
        }

        // Control: the SAME zero-signal vocabulary, one critical — below
        // ZERO_SIGNAL_MIN_POPULATION the guard is inert, and the count is
        // still reported.
        let (_tmp2, small) = bundle();
        put(&small, "work/lonely/work-item.md", work());
        put(&small, "patterns/void-00.md", void(topics[0], 0));
        let manifest = built(&small, "signalless", 100_000.0);
        assert_eq!(manifest["zero_signal_count"], json!(1), "the count is still REPORTED below the floor");
        assert_eq!(manifest["critical_total"], json!(1));
    }

    /// Node: 'knowledge context: relevance ties break DETERMINISTICALLY by
    /// path, and repeat runs are byte-identical' (l.1401).
    #[test]
    fn context_relevance_ties_break_deterministically_by_path() {
        let (_tmp, dir) = bundle();
        put(
            &dir,
            "work/twins/work-item.md",
            Cx::new("twins")
                .ty("bee.work-item")
                .title("Twin ranking")
                .description("Two criticals with identical vocabulary must not flap")
                .tags(&["twin"])
                .areas(&["twin"])
                .body("Identical vocabulary twin ranking flap determinism."),
        );
        for name in ["zulu-twin", "alpha-twin"] {
            put(
                &dir,
                &format!("patterns/{name}.md"),
                Cx::new(name)
                    .title("Twin pattern")
                    .description("Identical vocabulary twin")
                    .tags(&["twin"])
                    .areas(&["twin"])
                    .critical()
                    .body("Identical vocabulary twin ranking flap determinism, word for word."),
            );
        }
        let first = built(&dir, "twins", 100_000.0);
        let second = built(&dir, "twins", 100_000.0);
        assert_eq!(
            jsjson::stringify(&first),
            jsjson::stringify(&second),
            "two runs over the same bundle must serialize identically"
        );
        let criticals: Vec<String> = first["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["reason"].as_str().unwrap().starts_with("critical pattern"))
            .map(|e| e["path"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            criticals,
            vec!["docs/knowledge/patterns/alpha-twin.md", "docs/knowledge/patterns/zulu-twin.md"],
            "tied scores must order by path"
        );
        // Control: the tie is real — both criticals carry the same score.
        let scores: Vec<&str> = first["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|e| e["reason"].as_str())
            .filter(|r| r.starts_with("critical pattern"))
            .map(|r| r.split("relevance ").nth(1).unwrap().split(',').next().unwrap())
            .collect();
        assert_eq!(scores[0], scores[1], "the fixture must actually tie or the path tie-break proves nothing");
    }
}
