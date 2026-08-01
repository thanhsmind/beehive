// bee knowledge — native port of the knowledge verb group (bee.mjs
// handleKnowledgeCheck/Index/List/Context + lib/knowledge.mjs).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   knowledge check   [--strict] [--json]
//   knowledge index   [--check] [--json]
//   knowledge list    [--type T] [--lifecycle L] [--area A] [--json]
//   knowledge context --work W (--budget N | --lane tiny|small|standard|high-risk) [--json]
//
// DELEGATED to Node permanently: `knowledge promote` (cell-trace mining +
// canonical delivery/pattern draft rendering — not in this wave's scope).
//
// Additional delegation triggers (None before any output/write):
//   - --help anywhere, unknown flags, non-flag tokens, validate()-failing
//     shapes (missing --work/--budget, non-numeric --budget, --strict=x, ...)
//   - a configured non-empty `product_root` (repo-divorce topology: Node's
//     resolveProductRoot warn/path semantics are not replicated here)
//   - corrupt .bee/config.json or state files (Node warns with V8 text)
//   - bundle file/dir names carrying chars >= U+E000 (JS sorts by UTF-16
//     code units; Rust by UTF-8 bytes — they disagree only across that range)
//   - frontmatter quoted scalars that only V8 could parse (lone surrogates)
//   - unreadable bundle files mid-walk (Node embeds the V8 error message)
//   - --budget values outside the plain decimal/scientific grammar that JS
//     Number() also accepts (hex, Infinity, ...)
//   - any emitted value failing the JS number round-trip guard
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
// knowledgeIndexDrift/renderKnowledgeIndexes/normalizeSubject(ASCII subset)/
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
    /// A shape only V8 could decide (lone-surrogate escapes in a quoted
    /// scalar) — the whole command must delegate.
    NeedsNode,
}

fn fm_fail(code: &'static str, message: String, line: usize) -> Result<Value, Fm> {
    Err(Fm::Failed { code, message, line })
}

/// Lone-surrogate escape sniff (\uD800–\uDFFF): JSON.parse accepts them,
/// serde rejects — same heuristic feedback.rs uses for jsonl rows.
fn has_surrogate_escape(s: &str) -> bool {
    let b = s.as_bytes();
    for i in 0..b.len().saturating_sub(3) {
        if b[i] == b'\\'
            && (b[i + 1] == b'u' || b[i + 1] == b'U')
            && (b[i + 2] == b'd' || b[i + 2] == b'D')
            && matches!(b[i + 3], b'8' | b'9' | b'a'..=b'f' | b'A'..=b'F')
        {
            return true;
        }
    }
    false
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
            Err(_) => {
                if has_surrogate_escape(raw) {
                    return Err(Fm::NeedsNode);
                }
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
        Fm::NeedsNode => None,
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

/// None => delegate (walk/name issues, V8-only frontmatter).
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
                Fm::NeedsNode => return None,
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

/// normalizeSubject, restricted to ASCII claims (NFKC/diacritic/confusable
/// folds are identity there). None => non-ASCII claim, delegate.
fn normalize_subject_ascii(claim: &str) -> Option<String> {
    if !claim.is_ascii() {
        return None;
    }
    let lower = claim.to_ascii_lowercase();
    let mut out = String::new();
    let mut in_run = false;
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            in_run = false;
        } else if !in_run {
            out.push(' ');
            in_run = true;
        }
    }
    Some(out.trim_matches(' ').to_string())
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
            Err(_) => return None, // Node embeds the V8 error message — delegate
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
            Fm::NeedsNode => return None,
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
                let key = normalize_subject_ascii(claim)?; // non-ASCII → delegate
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
        Fm::NeedsNode => None,
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
        _ => None, // promote + unknown verbs (group-usage fallback) → Node
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
        // A lone-surrogate escape only V8 can parse → NeedsNode, not a finding.
        assert!(matches!(parse_frontmatter("---\ntitle: \"\\ud800\"\n---\n"), Fm::NeedsNode));
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

    #[test]
    fn normalize_subject_folds_ascii_encoding_only() {
        assert_eq!(normalize_subject_ascii("Billing: Refunds!").unwrap(), "billing refunds");
        assert_eq!(normalize_subject_ascii("...").unwrap(), "");
        assert!(normalize_subject_ascii("caf\u{e9}").is_none()); // non-ASCII → delegate
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
}
