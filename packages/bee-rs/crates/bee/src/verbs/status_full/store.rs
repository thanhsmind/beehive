// fs primitives and the state/config layers read out of the store
//
// Split out of the single 7k-line verbs/status_full.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, LinkedRoots, RootsWt};
use crate::state::{bypass_level, read_config_raw};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use crate::version::BEE_VERSION;

// ─── fs primitives (fsutil.mjs / recovery.mjs) ─────────────────────────────

/// readJson(file, fallback): Missing -> Ok(None); Parsed -> Ok(Some(v)).
///
/// CUTOVER (2026-08-01). Corrupt used to be `Ex::Bail` — the whole snapshot
/// went back to Node — because Node's warning interpolated V8's own
/// `JSON.parse` message. It now WARNS (buffered, so a later bail still emits
/// nothing) and returns readJson's `null` fallback, which is exactly what
/// every caller's `!x` / `?? null` guard already handled. Same status, same
/// exit code, one extra line of explanation on stderr.
pub(crate) fn rj(ctx: &Ctx, file: &Path) -> R<Option<Value>> {
    match crate::fsutil::read_json(file) {
        crate::fsutil::ReadJson::Missing => Ok(None),
        crate::fsutil::ReadJson::Corrupt => {
            ctx.warn(corrupt_json_warn_line(file));
            Ok(None)
        }
        crate::fsutil::ReadJson::Parsed(v) => Ok(Some(v)),
    }
}

/// The line `crate::fsutil::warn_corrupt_json` PRINTS, returned as a string
/// instead. Copied from fsutil (`warn_corrupt_json` + its private
/// `corrupt_json_reason`) because this file buffers stderr rather than
/// writing it, and fsutil offers no string-returning form.
pub(crate) fn corrupt_json_warn_line(file: &Path) -> String {
    let reason = match std::fs::read(file) {
        Err(_) => "the file could not be read".to_string(),
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
            match serde_json::from_str::<Value>(text) {
                // Raced: it parses now. Say only what is still true.
                Ok(_) => "invalid JSON".to_string(),
                Err(e) if e.line() > 0 => {
                    format!("invalid JSON at line {} column {}", e.line(), e.column())
                }
                Err(_) => "invalid JSON".to_string(),
            }
        }
    };
    format!(
        "bee: could not parse JSON at {} — {reason}. Using fallback; fix the file.",
        file.display()
    )
}

pub(crate) fn read_text_opt(file: &Path) -> Option<String> {
    std::fs::read(file).ok().map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// readJsonl: split /\r?\n/, trim, JSON.parse per line, silent skip.
pub(crate) fn read_jsonl(file: &Path) -> Vec<Value> {
    match read_text_opt(file) {
        Some(text) => parse_jsonl_text(&text),
        None => Vec::new(),
    }
}

pub(crate) fn parse_jsonl_text(text: &str) -> Vec<Value> {
    let mut events = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = js_trim(line);
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            events.push(v);
        }
    }
    events
}

/// fsutil.mjs hashFile: sha256 of the file's UTF-8 STRING content — read as
/// lossy utf8 (Node fs.readFileSync 'utf8'), hash those string bytes. No BOM
/// strip, matching Node.
pub(crate) fn hash_file(file: &Path) -> Option<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(file).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    Some(format!("{:x}", hasher.finalize()))
}

/// recovery.mjs readTranscriptTail — bounded tail window, drop the first
/// (truncated) line when the window starts mid-file, silent per-line parse.
pub(crate) fn read_transcript_tail(file: &Path, max_bytes: u64) -> R<Vec<Value>> {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(meta) = std::fs::metadata(file) else {
        return Ok(Vec::new()); // statSync throw -> [] (catch at top of fn)
    };
    let size = meta.len();
    if size == 0 {
        return Ok(Vec::new());
    }
    let start = size.saturating_sub(max_bytes);
    // Node openSync/readSync failures THROW (caught by the caller's own
    // fail-open wrapper where one exists).
    let mut f = std::fs::File::open(file).map_err(|_| Ex::Thrown)?;
    f.seek(SeekFrom::Start(start)).map_err(|_| Ex::Thrown)?;
    let mut buf = Vec::with_capacity((size - start) as usize);
    f.take(size - start).read_to_end(&mut buf).map_err(|_| Ex::Thrown)?;
    let mut text = String::from_utf8_lossy(&buf).into_owned();
    if start > 0 {
        match text.find('\n') {
            Some(nl) => text = text[nl + 1..].to_string(),
            None => text = String::new(),
        }
    }
    Ok(parse_jsonl_text(&text))
}

/// Node path.join/resolve-style lexical normalization of an absolute path
/// (separator unification + '.'/'..' collapse). Only used where Node's
/// path output shape is observable (worktree resolution, projects root).
pub(crate) fn normalize_abs_lexical(p: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    let unified: String = p.replace(['/', '\\'], &sep.to_string());
    let mut prefix = String::new();
    let mut rest = unified.as_str();
    if cfg!(windows) {
        let bytes = rest.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' {
            prefix = rest[..2].to_string();
            rest = &rest[2..];
        }
    }
    let absolute = rest.starts_with(sep);
    let mut parts: Vec<&str> = Vec::new();
    for seg in rest.split(sep) {
        match seg {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() {
                    parts.pop();
                }
            }
            s => parts.push(s),
        }
    }
    let mut out = prefix;
    if absolute {
        out.push(sep);
    }
    out.push_str(&parts.join(&sep.to_string()));
    out
}

/// Node path.resolve(base, p) for gitdir pointers: absolute p normalizes
/// alone; relative joins onto base.
pub(crate) fn path_resolve(base: &Path, p: &str) -> String {
    let is_abs = {
        let b = p.as_bytes();
        p.starts_with('/') || p.starts_with('\\') || (b.len() >= 2 && b[1] == b':')
    };
    if is_abs {
        normalize_abs_lexical(p)
    } else {
        normalize_abs_lexical(&format!("{}{}{}", base.display(), std::path::MAIN_SEPARATOR, p))
    }
}

pub(crate) fn path_dirname(p: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    match p.rfind(sep) {
        Some(idx) if idx > 0 => p[..idx].to_string(),
        Some(_) => p[..1].to_string(),
        None => p.to_string(),
    }
}

pub(crate) fn path_basename(p: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    match p.rfind(sep) {
        Some(idx) => p[idx + 1..].to_string(),
        None => p.to_string(),
    }
}

pub(crate) fn home_dir() -> String {
    if cfg!(windows) {
        std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_default()
    } else {
        std::env::var("HOME").unwrap_or_default()
    }
}

/// perf.mjs claudeProjectsRoot — CLAUDE_CONFIG_DIR override (JS || falsy),
/// else <home>/.claude; 'projects' joined with Node path.join shape.
pub(crate) fn claude_projects_root() -> String {
    let base = match std::env::var("CLAUDE_CONFIG_DIR") {
        Ok(v) if !v.is_empty() => v,
        _ => format!("{}{}.claude", home_dir(), std::path::MAIN_SEPARATOR),
    };
    normalize_abs_lexical(&format!(
        "{}{}projects",
        base.trim_end_matches(['/', '\\']),
        std::path::MAIN_SEPARATOR
    ))
}

/// encodeProjectDir: replace [\\/.:] with '-'.
///
/// Divergence from perf.mjs, taken deliberately AT CUTOVER (plans/rust-port.md,
/// "the two filed win32 defects"): Node's regex was `/[\\/.]/g`, which leaves a
/// Windows drive colon in the directory NAME (`D:-projects-…`). That component
/// cannot exist on NTFS — `mkdir` fails EINVAL and the colon is taken as an
/// alternate data stream on the parent — so every transcript-dependent path
/// (recovery scan, perf rollup) was unreachable on win32 for BOTH runtimes.
/// Mapping ':' too yields `D--projects-…`, which is what Claude Code itself
/// writes, so the layout this names is the layout that actually exists.
pub(crate) fn encode_project_dir(project_path: &str) -> String {
    project_path
        .chars()
        .map(|c| if matches!(c, '\\' | '/' | '.' | ':') { '-' } else { c })
        .collect()
}

// ─── state layer (state.mjs) ───────────────────────────────────────────────

pub(crate) fn default_gates() -> JMap {
    let mut m = JMap::new();
    for g in GATE_NAMES {
        m.insert(g.into(), Value::Bool(false));
    }
    m
}

/// state.mjs defaultState().
pub(crate) fn default_state() -> JMap {
    let mut m = JMap::new();
    m.insert("schema_version".into(), json!("1.0"));
    m.insert("phase".into(), json!("idle"));
    m.insert("feature".into(), Value::Null);
    m.insert("mode".into(), Value::Null);
    m.insert("approved_gates".into(), Value::Object(default_gates()));
    m.insert("workers".into(), json!([]));
    m.insert("summary".into(), json!(""));
    m.insert("next_action".into(), json!("No active bee work — awaiting a user request."));
    m
}

/// state.mjs readState — fail-open merge over defaultState(), with the D13
/// legacy 'validating' -> 'planning' coercion. Truthy non-object
/// approved_gates spreads JS-exotically -> bail.
pub(crate) fn read_state_full(ctx: &Ctx) -> R<JMap> {
    let parsed = rj(ctx, &ctx.root.join(".bee").join("state.json"))?;
    let file_state = match parsed {
        Some(Value::Object(m)) => Some(m),
        _ => None,
    };
    let mut merged = default_state();
    let Some(state) = file_state else {
        return Ok(merged);
    };
    for (k, v) in &state {
        merged.insert(k.clone(), v.clone()); // existing keys keep position (JS spread)
    }
    let gates = match state.get("approved_gates") {
        None | Some(Value::Null) | Some(Value::Bool(false)) => default_gates(),
        Some(Value::String(s)) if s.is_empty() => default_gates(),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => default_gates(),
        Some(Value::Object(overlay)) => {
            let mut g = default_gates();
            for (k, v) in overlay {
                g.insert(k.clone(), v.clone());
            }
            g
        }
        Some(_) => return Err(Ex::Bail), // truthy non-object spread — JS-exotic
    };
    merged.insert("approved_gates".into(), Value::Object(gates));
    if str_eq(merged.get("phase"), "validating") {
        merged.insert("phase".into(), json!("planning"));
    }
    Ok(merged)
}

/// state.mjs readOnboarding.
pub(crate) fn read_onboarding(ctx: &Ctx) -> R<Option<Value>> {
    rj(ctx, &ctx.root.join(".bee").join("onboarding.json"))
}

/// state.mjs readHandoff — fail-open; non-object parses return verbatim; an
/// object gets `kind` normalized (missing/unknown -> 'pause') at its original
/// key position (JS `{...handoff, kind}` semantics).
pub(crate) fn read_handoff(ctx: &Ctx) -> R<Option<Value>> {
    let parsed = rj(ctx, &ctx.root.join(".bee").join("HANDOFF.json"))?;
    let Some(v) = parsed else { return Ok(Some(Value::Null)) }; // readJson fallback null
    match v {
        Value::Object(m) => {
            let kind = if str_eq(m.get("kind"), "planned-next") { "planned-next" } else { "pause" };
            let mut out = m;
            out.insert("kind".into(), json!(kind));
            Ok(Some(Value::Object(out)))
        }
        other => Ok(Some(other)),
    }
}

// ─── config layer (state.mjs readConfig + normalizers) ─────────────────────

pub(crate) struct Config {
    /// Merged tracked+overlay raw object, advisor key stripped (the value
    /// readConfig spreads as `...rest`; gate_bypass/ship_visibility/
    /// product_root/recovery read straight off this).
    pub(crate) raw: JMap,
    pub(crate) commands: JMap,
    pub(crate) models: JMap,
}

/// state.mjs normalizeCommands.
pub(crate) fn normalize_commands(raw: Option<&Value>) -> JMap {
    let mut commands = JMap::new();
    let Some(Value::Object(obj)) = raw else { return commands };
    for key in COMMAND_KEYS.iter().chain(WORKTREE_COMPANION_COMMAND_KEYS.iter()) {
        match obj.get(*key) {
            Some(Value::String(s)) if !js_trim(s).is_empty() => {
                commands.insert((*key).into(), json!(js_trim(s)));
            }
            Some(Value::Array(items)) if *key == "test" => {
                let list: Vec<Value> = items
                    .iter()
                    .filter_map(|c| c.as_str())
                    .filter(|c| !js_trim(c).is_empty())
                    .map(|c| json!(js_trim(c)))
                    .collect();
                if !list.is_empty() {
                    commands.insert((*key).into(), Value::Array(list));
                }
            }
            _ => {}
        }
    }
    commands
}

/// state.mjs normalizeTierValue — returns None for "undefined" (invalid
/// shape: the seeded default stays).
pub(crate) fn normalize_tier_value(value: Option<&Value>) -> Option<Value> {
    match value {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Some(json!(js_trim(s))),
        Some(Value::Null) => Some(Value::Null),
        Some(Value::Object(o)) => {
            if str_eq(o.get("kind"), "cli") {
                if let Some(Value::String(cmd)) = o.get("command") {
                    if !js_trim(cmd).is_empty() {
                        let mut out = JMap::new();
                        out.insert("kind".into(), json!("cli"));
                        out.insert("command".into(), json!(js_trim(cmd)));
                        return Some(Value::Object(out));
                    }
                }
            }
            if str_eq(o.get("kind"), "native") {
                if let Some(Value::String(model)) = o.get("model") {
                    if !js_trim(model).is_empty() {
                        let mut out = JMap::new();
                        out.insert("kind".into(), json!("native"));
                        out.insert("model".into(), json!(js_trim(model)));
                        if let Some(Value::String(e)) = o.get("effort") {
                            if EFFORT_LEVELS.contains(&js_trim(e)) {
                                out.insert("effort".into(), json!(js_trim(e)));
                            }
                        }
                        if let Some(Value::String(ft)) = o.get("fork_turns") {
                            if js_trim(ft) == "none" {
                                out.insert("fork_turns".into(), json!("none"));
                            }
                        }
                        if let Some(Value::String(at)) = o.get("agent_type") {
                            if !js_trim(at).is_empty() {
                                out.insert("agent_type".into(), json!(js_trim(at)));
                            }
                        }
                        return Some(Value::Object(out));
                    }
                }
            }
            // Explicit-fallback composite: primary must be a valid native leaf.
            if let Some(primary @ Value::Object(p)) = o.get("primary") {
                let primary_ok = str_eq(p.get("kind"), "native")
                    && matches!(p.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty());
                if primary_ok {
                    let mut out = JMap::new();
                    out.insert("primary".into(), normalize_tier_value(Some(primary)).unwrap_or(Value::Null));
                    if str_eq(o.get("fallback_policy"), "explicit-only") {
                        out.insert("fallback_policy".into(), json!("explicit-only"));
                        if let Some(Value::Object(fb)) = o.get("fallback") {
                            if str_eq(fb.get("kind"), "cli") {
                                if let Some(Value::String(cmd)) = fb.get("command") {
                                    if !js_trim(cmd).is_empty() {
                                        let mut f = JMap::new();
                                        f.insert("kind".into(), json!("cli"));
                                        f.insert("command".into(), json!(js_trim(cmd)));
                                        out.insert("fallback".into(), Value::Object(f));
                                    }
                                }
                            }
                        }
                    }
                    return Some(Value::Object(out));
                }
            }
            if o.get("kind").is_none() {
                if let Some(Value::String(model)) = o.get("model") {
                    if !js_trim(model).is_empty() {
                        let mut out = JMap::new();
                        out.insert("model".into(), json!(js_trim(model)));
                        if let Some(Value::String(e)) = o.get("effort") {
                            if EFFORT_LEVELS.contains(&js_trim(e)) {
                                out.insert("effort".into(), json!(js_trim(e)));
                            }
                        }
                        return Some(Value::Object(out));
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// state.mjs DEFAULT_MODELS + normalizeModels. opencode-support oc-13: a
/// third `opencode` entry joins claude/codex, defaulted null exactly like
/// codex (no built-in model name — resolved per-agent by the `.opencode/agent/bee-*.md`
/// `model:` pin instead, per drivers/models.rs's own `default_models`).
pub(crate) fn normalize_models(raw: Option<&Value>) -> JMap {
    let mut claude = JMap::new();
    claude.insert("extraction".into(), json!("haiku"));
    claude.insert("generation".into(), json!("sonnet"));
    claude.insert("review".into(), json!("opus"));
    let mut codex = JMap::new();
    codex.insert("extraction".into(), Value::Null);
    codex.insert("generation".into(), Value::Null);
    codex.insert("review".into(), Value::Null);
    let mut opencode = JMap::new();
    opencode.insert("extraction".into(), Value::Null);
    opencode.insert("generation".into(), Value::Null);
    opencode.insert("review".into(), Value::Null);
    let mut out = JMap::new();
    out.insert("claude".into(), Value::Object(claude));
    out.insert("codex".into(), Value::Object(codex));
    out.insert("opencode".into(), Value::Object(opencode));
    if let Some(Value::Object(m)) = raw {
        for rt in RUNTIMES {
            let Some(Value::Object(src)) = m.get(rt) else { continue };
            for slot in MODEL_NORMALIZE_SLOTS {
                if let Some(v) = normalize_tier_value(src.get(slot)) {
                    if let Some(Value::Object(target)) = out.get_mut(rt) {
                        target.insert(slot.into(), v);
                    }
                }
            }
        }
    }
    out
}

/// state.mjs normalizeDogfoodRepos — the WARNING side only (the normalized
/// list itself is never read by status). Every readConfig call re-emits.
pub(crate) fn dogfood_warnings(ctx: &mut Ctx, raw: &JMap) {
    let Some(Value::Array(items)) = raw.get("dogfood_repos") else { return };
    if std::env::var("BEE_HOOK_CONTEXT").map(|v| !v.is_empty()).unwrap_or(false) {
        // Warning suppressed under a hook context; entries still skipped.
        return;
    }
    for item in items {
        let raw_path: Option<&str> = match item {
            Value::String(s) => Some(s.as_str()),
            Value::Object(o) => match o.get("path") {
                Some(Value::String(p)) => Some(p.as_str()),
                _ => None,
            },
            _ => None,
        };
        let Some(raw_path) = raw_path else { continue };
        if js_trim(raw_path).is_empty() {
            continue;
        }
        let trimmed = js_trim(raw_path);
        let resolved = if Path::new(trimmed).is_absolute() {
            PathBuf::from(trimmed)
        } else {
            ctx.cwd.join(trimmed)
        };
        if let Err(err) = dunce::canonicalize(&resolved) {
            let code = match err.kind() {
                std::io::ErrorKind::NotFound => "ENOENT".to_string(),
                std::io::ErrorKind::PermissionDenied => "EACCES".to_string(),
                _ => format!("{err}"),
            };
            ctx.warn(format!(
                "dogfood_repos: skipping \"{raw_path}\" — {code} (dead or unreadable repo; the bee session continues)"
            ));
        }
    }
}

/// state.mjs readConfig — merged tracked+overlay (via crate::state::
/// read_config_raw, which warns and falls back on corrupt), advisor stripped,
/// plus the normalized commands/models this port consumes. Emits dogfood
/// warnings on EVERY call, mirroring Node's per-call normalization.
pub(crate) fn read_config(ctx: &mut Ctx) -> R<Config> {
    let raw = read_config_raw(&ctx.root);
    let commands = normalize_commands(raw.get("commands"));
    let models = normalize_models(raw.get("models"));
    dogfood_warnings(ctx, &raw);
    Ok(Config { raw, commands, models })
}

/// state.mjs shipVisibility — warn+normalize on unrecognized values.
pub(crate) fn ship_visibility(ctx: &mut Ctx) -> R<String> {
    let config = read_config(ctx)?;
    match config.raw.get("ship_visibility") {
        None | Some(Value::Null) => Ok("off".into()),
        Some(Value::String(s)) if s == "off" || s == "draft-pr" => Ok(s.clone()),
        Some(other) => {
            ctx.warn(format!(
                "config: unrecognized ship_visibility \"{}\" in .bee/config.json — normalized to \"off\". Allowed: off, draft-pr.",
                jsjson::js_to_string(other)
            ));
            Ok("off".into())
        }
    }
}

/// state.mjs bypassLevel(root).
pub(crate) fn bypass_level_root(ctx: &mut Ctx) -> R<&'static str> {
    let config = read_config(ctx)?;
    Ok(bypass_level(&config.raw))
}

/// state.mjs bypassBanner.
pub(crate) fn bypass_banner(level: &str) -> &'static str {
    match level {
        "total" => "⚡⚡⚡ GATE BYPASS: TOTAL AUTOPILOT — ZERO STOPS. Every gate (any lane, high-risk/hard-gate included), secret-file reads, and review P1 findings auto-proceed; NO human checkpoint remains. Turn off: bee-hive bypass off",
        "full" => "⚡⚡ GATE BYPASS: FULL AUTOPILOT — ALL Gates 1-2 auto-approved including high-risk/hard-gate work; only secret-file reads and a review P1 finding still stop for the human. Turn off: bee-hive bypass off",
        "normal" => "⚡ GATE BYPASS: NORMAL — Gates 1-2 auto-approved for tiny/small/standard work only; high-risk/hard-gate, secret reads, and Gate 3 UAT still stop. Turn off: bee-hive bypass off",
        _ => "",
    }
}

/// state.mjs hasStaleAdvisorKey — reads the TRACKED config.json raw.
pub(crate) fn has_stale_advisor_key(ctx: &Ctx) -> R<bool> {
    let raw = rj(ctx, &ctx.root.join(".bee").join("config.json"))?;
    Ok(matches!(raw, Some(Value::Object(m)) if m.contains_key("advisor")))
}

/// bee.mjs readRawConfigForValidation — None = no config file at all
/// (undefined); Some(v) = whatever was parsed. A corrupt file present on disk
/// is `Some(Value::Null)`: readJson warned and returned its `null` fallback,
/// which is precisely what validation then sees.
pub(crate) fn read_raw_config_for_validation(ctx: &Ctx) -> R<Option<Value>> {
    let file = ctx.root.join(".bee").join("config.json");
    if !file.exists() {
        return Ok(None);
    }
    Ok(Some(rj(ctx, &file)?.unwrap_or(Value::Null)))
}

pub(crate) struct Problem {
    pub(crate) code: &'static str,
    pub(crate) runtime: Option<&'static str>,
    pub(crate) slot: Option<&'static str>,
    pub(crate) message: String,
    /// Only for validateAgentFilesDrift rows.
    pub(crate) agent: Option<&'static str>,
}

/// state.mjs validateModelsConfig — never throws; returns problem rows.
pub(crate) fn validate_models_config(config: Option<&Value>) -> Vec<Problem> {
    let mut problems = Vec::new();
    let Some(config) = config else { return problems };
    let obj = match config {
        Value::Object(m) => m,
        _ => {
            problems.push(Problem {
                code: "config-malformed",
                runtime: None,
                slot: None,
                message: ".bee/config.json content is null or not an object — models config cannot be validated; defaults apply.".into(),
                agent: None,
            });
            return problems;
        }
    };
    let Some(models) = obj.get("models") else { return problems };
    let models = match models {
        Value::Object(m) => m,
        _ => {
            problems.push(Problem {
                code: "config-malformed",
                runtime: None,
                slot: None,
                message: "`models` in .bee/config.json is present but not an object — ignored; defaults apply.".into(),
                agent: None,
            });
            return problems;
        }
    };
    for rt in RUNTIMES {
        let Some(src) = models.get(rt) else { continue };
        let src = match src {
            Value::Object(m) => m,
            _ => {
                problems.push(Problem {
                    code: "runtime-malformed",
                    runtime: Some(rt),
                    slot: None,
                    message: format!("models.{rt} is present but not an object — ignored; defaults apply."),
                    agent: None,
                });
                continue;
            }
        };
        for slot in MODEL_VALIDATE_SLOTS {
            let value = match src.get(slot) {
                None | Some(Value::Null) => continue,
                Some(Value::String(_)) => continue,
                Some(v) => v,
            };
            let vobj = match value {
                Value::Object(m) => m,
                _ => {
                    problems.push(Problem {
                        code: "slot-value-malformed",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is not a string, object, or null — ignored; defaults apply."),
                        agent: None,
                    });
                    continue;
                }
            };
            let is_composite = vobj.contains_key("primary")
                || vobj.contains_key("fallback")
                || vobj.contains_key("fallback_policy");
            if is_composite {
                let primary = vobj.get("primary");
                let primary_ok = matches!(primary, Some(Value::Object(p))
                    if str_eq(p.get("kind"), "native")
                        && matches!(p.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty()));
                if !primary_ok {
                    problems.push(Problem {
                        code: "composite-primary-malformed",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is a composite (primary/fallback) but its primary is not a valid native override {{kind:\"native\", model}} — ignored; today this silently reverts to the seeded default (D2)."),
                        agent: None,
                    });
                    continue;
                }
                if let Some(Value::Object(p)) = primary {
                    if let Some(ft) = p.get("fork_turns") {
                        if !str_eq(Some(ft), "none") {
                            problems.push(Problem {
                                code: "native-fork-turns-unknown",
                                runtime: Some(rt),
                                slot: Some(slot),
                                message: format!(
                                    "models.{rt}.{slot} composite primary has fork_turns:{} — only \"none\" is valid; a full-history fork rejects model overrides (E2/D2).",
                                    jsjson::stringify(ft)
                                ),
                                agent: None,
                            });
                        }
                    }
                }
                if !str_eq(vobj.get("fallback_policy"), "explicit-only") {
                    problems.push(Problem {
                        code: "composite-fallback-policy-missing",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is a composite but has no fallback_policy:\"explicit-only\" — its cli fallback is silently dropped and no fallback is ever taken; silent native->cli fallback is forbidden (D1). Set fallback_policy:\"explicit-only\" to opt in."),
                        agent: None,
                    });
                    continue;
                }
                let fb_ok = matches!(vobj.get("fallback"), Some(Value::Object(f))
                    if str_eq(f.get("kind"), "cli")
                        && matches!(f.get("command"), Some(Value::String(c)) if !js_trim(c).is_empty()));
                if !fb_ok {
                    problems.push(Problem {
                        code: "composite-fallback-malformed",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} composite declares fallback_policy:\"explicit-only\" but its fallback is not a valid cli executor {{kind:\"cli\", command}} — the fallback is silently dropped; fix or remove it (D2)."),
                        agent: None,
                    });
                }
                continue;
            }
            if str_eq(vobj.get("kind"), "native") {
                let model_ok = matches!(vobj.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty());
                if !model_ok {
                    problems.push(Problem {
                        code: "native-model-missing",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is a native override (kind:\"native\") but has no non-empty model — the exact catalog model id is required; today this silently reverts to the seeded default (D2)."),
                        agent: None,
                    });
                    continue;
                }
                if let Some(ft) = vobj.get("fork_turns") {
                    if !str_eq(Some(ft), "none") {
                        problems.push(Problem {
                            code: "native-fork-turns-unknown",
                            runtime: Some(rt),
                            slot: Some(slot),
                            message: format!(
                                "models.{rt}.{slot} native override has fork_turns:{} — only \"none\" is valid; a full-history fork rejects model overrides (E2/D2).",
                                jsjson::stringify(ft)
                            ),
                            agent: None,
                        });
                    }
                }
                continue;
            }
            let looks_like_cli = vobj.contains_key("kind") || vobj.contains_key("command");
            if looks_like_cli {
                let kind_ok = str_eq(vobj.get("kind"), "cli");
                let command = vobj.get("command").and_then(|v| v.as_str());
                let command_ok = command.map(|c| !js_trim(c).is_empty()).unwrap_or(false);
                if !kind_ok || !command_ok {
                    problems.push(Problem {
                        code: "cli-malformed",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} looks like a cli executor but is missing kind:\"cli\" or a non-empty command — today this silently reverts to the seeded default; fix or remove it (W-e)."),
                        agent: None,
                    });
                    continue;
                }
                let transport_ok = matches!(vobj.get("promptVia"), Some(Value::String(p)) if !js_trim(p).is_empty());
                if !transport_ok {
                    problems.push(Problem {
                        code: "cli-prompt-transport-missing",
                        runtime: Some(rt),
                        slot: Some(slot),
                        message: format!("models.{rt}.{slot} is a cli executor with no declared prompt transport — set promptVia (e.g. \"stdin\") so the prompt reliably reaches it; never inferred from the command string (B2)."),
                        agent: None,
                    });
                }
                let command = command.unwrap_or("");
                for flag in UNSAFE_CLI_FLAGS {
                    if command.contains(flag) {
                        problems.push(Problem {
                            code: "cli-unsafe-flag",
                            runtime: Some(rt),
                            slot: Some(slot),
                            message: format!("models.{rt}.{slot} command contains \"{flag}\" — a known auto-approve/sandbox-bypass flag; remove it (B6/B7). This is a blocklist of KNOWN-BAD flags, not a positive read-only guarantee."),
                            agent: None,
                        });
                    }
                }
                if ADVICE_CLASS_SLOTS.contains(&slot) {
                    for token in ADVICE_CLASS_WRITABLE_TOKENS {
                        if command.contains(token) {
                            problems.push(Problem {
                                code: "cli-advice-slot-writable",
                                runtime: Some(rt),
                                slot: Some(slot),
                                message: format!("models.{rt}.{slot} is an advice-class cli slot (advisor/review must run read-only, AO8) and its command contains \"{token}\" — a known write-granting sandbox token; remove it. This is a blocklist of KNOWN write-granting tokens, not a positive read-only guarantee."),
                                agent: None,
                            });
                        }
                    }
                }
                continue;
            }
            let model_ok = matches!(vobj.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty());
            if !model_ok {
                problems.push(Problem {
                    code: "model-shape-malformed",
                    runtime: Some(rt),
                    slot: Some(slot),
                    message: format!("models.{rt}.{slot} is an object but neither a valid cli executor nor a valid {{model}} shape — ignored; today this silently reverts to the seeded default."),
                    agent: None,
                });
            }
        }
    }
    problems
}

/// state.mjs readAgentFileModel — regex-free port of the frontmatter probe.
/// Returns (found, model): model None = frontmatter unparseable (or no
/// model line), Some = the trimmed value.
pub(crate) fn read_agent_file_model(file: &Path) -> (bool, Option<String>) {
    let Some(raw) = read_text_opt(file) else {
        return (false, None);
    };
    // /^---\r?\n([\s\S]*?)\r?\n---/ anchored at content start, lazy body.
    let body_start = if let Some(rest) = raw.strip_prefix("---\r\n") {
        Some((5usize, rest))
    } else {
        raw.strip_prefix("---\n").map(|rest| (4usize, rest))
    };
    let Some((offset, rest)) = body_start else {
        return (true, None);
    };
    // First "\n---" after the opening (an optional \r before it is consumed).
    let mut close: Option<usize> = None;
    let bytes = rest.as_bytes();
    let mut k = 0;
    while k + 4 <= bytes.len() {
        if &bytes[k..k + 4] == b"\n---" {
            close = Some(k);
            break;
        }
        k += 1;
    }
    let Some(close) = close else {
        return (true, None);
    };
    let _ = offset;
    let mut frontmatter = &rest[..close];
    if frontmatter.ends_with('\r') {
        frontmatter = &frontmatter[..frontmatter.len() - 1];
    }
    // /^model:\s*(.+)$/m over the frontmatter.
    for line in frontmatter.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some(after) = line.strip_prefix("model:") {
            if !after.is_empty() {
                return (true, Some(js_trim(after).to_string()));
            }
        }
    }
    (true, None)
}

/// state.mjs validateAgentFilesDrift. opencode-support oc-13: a second
/// runtime root joins `.claude/agents/` — `.opencode/agent/` (singular
/// "agent", per oc-11's hand-authored `.opencode/agent/bee-*.md` files and
/// discovery.md's verified on-disk layout), checked against `models.opencode`
/// instead of `models.claude`. Same three read-only agents on both roots;
/// `bee-build` carries no tier check on either (AGENT_FILE_TIER's existing
/// scope, unchanged).
pub(crate) fn validate_agent_files_drift(ctx: &Ctx, raw_config: Option<&Value>) -> Vec<Problem> {
    const AGENT_FILE_TIER: [(&str, &str); 3] = [
        ("bee-gather", "generation"),
        ("bee-extract", "extraction"),
        ("bee-review", "review"),
    ];
    const AGENT_FILE_ROOTS: [(&str, &str, &str); 2] =
        [("claude", ".claude", "agents"), ("opencode", ".opencode", "agent")];
    let mut problems = Vec::new();
    let raw_models = raw_config.and_then(|c| match c {
        Value::Object(m) => m.get("models"),
        _ => None,
    });
    let models = normalize_models(raw_models);
    for (runtime, dir, subdir) in AGENT_FILE_ROOTS {
        let rel_prefix = format!("{dir}/{subdir}");
        for (agent_name, slot) in AGENT_FILE_TIER {
            let file = ctx.root.join(dir).join(subdir).join(format!("{agent_name}.md"));
            let (found, file_model) = read_agent_file_model(&file);
            if !found {
                continue;
            }
            let Some(file_model) = file_model else {
                problems.push(Problem {
                    code: "agent-file-malformed",
                    runtime: None,
                    slot: Some(slot),
                    message: format!("{rel_prefix}/{agent_name}.md has no readable \"model:\" frontmatter line — cannot check drift; re-run onboarding to re-render it."),
                    agent: Some(agent_name),
                });
                continue;
            };
            let rt_models = models.get(runtime).and_then(|v| v.as_object());
            let mut value = rt_models.and_then(|c| c.get(slot));
            if nullish(value) && slot == "review" {
                value = rt_models.and_then(|c| c.get("generation"));
            }
            let expected: Option<String> = match value {
                Some(Value::String(s)) => Some(s.clone()),
                Some(Value::Object(o)) => o.get("model").and_then(|m| m.as_str()).map(str::to_string),
                _ => None,
            };
            match expected {
                // Only claude carries a non-null built-in default (haiku/
                // sonnet/opus — normalize_models seeds it unconditionally),
                // so an unconfigured claude slot resolving to None only
                // happens when config explicitly opted the slot OUT (a
                // cli-shaped or literal-null override) — a real "this file
                // should not exist" signal. opencode has no such default:
                // its agent files are hand-authored, pinned to a free-tier
                // model regardless of config (oc-11), so an unconfigured
                // models.opencode slot is the ORDINARY state, not drift.
                None if runtime == "claude" => problems.push(Problem {
                    code: "agent-file-drift",
                    runtime: None,
                    slot: Some(slot),
                    message: format!("{rel_prefix}/{agent_name}.md declares model: \"{file_model}\" but the {slot} slot is now cli-shaped or unconfigured (no model name) — re-run onboarding to remove the stale file."),
                    agent: Some(agent_name),
                }),
                None => {}
                Some(expected) if expected != file_model && runtime == "claude" => problems.push(Problem {
                    code: "agent-file-drift",
                    runtime: None,
                    slot: Some(slot),
                    message: format!("{rel_prefix}/{agent_name}.md declares model: \"{file_model}\" but the configured {slot} model is \"{expected}\" — re-run onboarding to re-render it."),
                    agent: Some(agent_name),
                }),
                // opencode's agent files are hand-authored (oc-11), not
                // onboarding-rendered, so "re-run onboarding" would be a
                // promise bee cannot keep yet — name the fix that actually
                // applies today instead.
                Some(expected) if expected != file_model => problems.push(Problem {
                    code: "agent-file-drift",
                    runtime: None,
                    slot: Some(slot),
                    message: format!("{rel_prefix}/{agent_name}.md declares model: \"{file_model}\" but the configured {slot} model is \"{expected}\" — {rel_prefix}/{agent_name}.md is hand-authored, not onboarding-rendered; update its \"model:\" line by hand or change models.{runtime}.{slot} to match."),
                    agent: Some(agent_name),
                }),
                _ => {}
            }
        }
    }
    problems
}

/// state.mjs resolveProductRoot — root unless config `product_root` points
/// elsewhere; warnings replicated (each call re-reads config).
pub(crate) fn resolve_product_root(ctx: &mut Ctx) -> R<PathBuf> {
    let config = read_config(ctx)?;
    let configured = config.raw.get("product_root").cloned();
    match configured {
        None | Some(Value::Null) => Ok(ctx.root.clone()),
        Some(Value::String(s)) if s.is_empty() => Ok(ctx.root.clone()),
        Some(Value::String(s)) => {
            let resolved = if Path::new(&s).is_absolute() {
                PathBuf::from(&s)
            } else {
                PathBuf::from(normalize_abs_lexical(&format!(
                    "{}{}{}",
                    ctx.root.display(),
                    std::path::MAIN_SEPARATOR,
                    s
                )))
            };
            let is_dir = std::fs::metadata(&resolved).map(|m| m.is_dir()).unwrap_or(false);
            if !is_dir {
                ctx.warn(format!(
                    "bee: config product_root \"{s}\" -> \"{}\" is not an existing directory; product-doc reads (docs/backlog.md, docs/specs/) will find nothing until you fix .bee/config.json product_root. (GitHub #14)",
                    resolved.display()
                ));
            }
            Ok(resolved)
        }
        Some(other) => {
            let ty = match other {
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::Array(_) | Value::Object(_) => "object",
                _ => "object",
            };
            ctx.warn(format!(
                "bee: .bee/config.json product_root must be a string path (got {ty}); ignoring it and using the bee root."
            ));
            Ok(ctx.root.clone())
        }
    }
}

/// worktree-store.mjs readGrants — silent {} on any failure.
pub(crate) fn read_grants(main_store_root: &Path) -> JMap {
    let file = main_store_root.join("runtime").join("worktree-grants.json");
    match read_text_opt(&file).and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
        Some(Value::Object(m)) => m,
        _ => JMap::new(),
    }
}

/// state.mjs controlRootFor(root) -> resolveContext(root).controlRoot ?? root.
///
/// NOTE the argument: resolveContext is handed `root` (main()'s already-
/// resolved storeRoot), NOT cwd. That distinction is what makes this a
/// two-line function rather than a second walk:
///   * ordinary checkout        -> resolveRootsCore(root) is ordinary,
///                                 controlRoot === workspaceRoot === root.
///   * UNGRANTED linked worktree-> `root` already fell back to mainRoot, and
///                                 resolveRootsCore(mainRoot) is ordinary,
///                                 so controlRoot === root again.
///   * GRANTED linked worktree  -> `root` IS the worktree checkout, so
///                                 resolveRootsCore(root) is linked-valid and
///                                 controlRoot === its mainRoot.
/// So the only case that moves is the granted one, and `ctx.linked` (resolved
/// from the same cwd `root` was) already carries its mainRoot.
///
/// The resolveContext side effects Node performs anyway are replicated for
/// warning parity: readGrants over the MAIN store's `.bee` (silent), and
/// resolveProductRoot(workspaceRoot) — workspaceRoot is `root` in all three
/// cases above, so `resolve_product_root(ctx)` is exact — with its warnings.
pub(crate) fn control_root_for(ctx: &mut Ctx) -> R<PathBuf> {
    let control = match ctx.granted_worktree() {
        Some(l) => l.main_root.clone(),
        None => ctx.root.clone(),
    };
    let _ = read_grants(&control.join(".bee"));
    let _ = resolve_product_root(ctx)?;
    Ok(control)
}

/// reservations.mjs's cycle-safe controlRootFor replica: a pure git walk-up
/// (findMainRoot), NO config read — that module cannot import state.mjs's
/// controlRootFor without a cycle, so it carries its own. Ordinary git root
/// -> that root; a `.git` FILE -> the bidirectionally-validated mainRoot;
/// anything malformed or no git at all -> `root` (findMainRoot's null).
///
/// This is NOT the same walk as `control_root_for` above even though the two
/// agree on every shape bee actually produces: this one starts at `root` and
/// consults no grant registry, so it answers mainRoot for a granted worktree
/// via the git link alone.
pub(crate) fn reservations_control_root(ctx: &Ctx) -> PathBuf {
    let walk = || -> Option<PathBuf> {
        // locateGitRoot(root)
        let mut dir: Option<&Path> = Some(&ctx.root);
        let (work_root, marker) = loop {
            let d = dir?;
            let marker = d.join(".git");
            if marker.exists() {
                break (d.to_path_buf(), marker);
            }
            dir = d.parent();
        };
        if !std::fs::metadata(&marker).ok()?.is_file() {
            return Some(work_root); // ordinary checkout: mainRoot === workRoot
        }
        let read_ptr = |file: &Path, base: &Path| -> Option<String> {
            let raw = read_text_opt(file)?;
            let raw = js_trim(&raw);
            if raw.is_empty() {
                return None;
            }
            let raw = match raw.strip_prefix("gitdir:") {
                Some(rest) => js_trim(rest),
                None => raw,
            };
            let fixed = raw.replace('\\', &std::path::MAIN_SEPARATOR.to_string());
            Some(path_resolve(base, &fixed))
        };
        let gitdir = read_ptr(&marker, &work_root)?;
        let worktrees_root = path_resolve(Path::new(&gitdir), "..");
        let common_git_dir = path_resolve(Path::new(&worktrees_root), "..");
        if path_basename(&common_git_dir) != ".git" || path_basename(&worktrees_root) != "worktrees"
        {
            return None;
        }
        let id = path_basename(&gitdir);
        if id.is_empty() || id == "." || id == ".." {
            return None;
        }
        let reverse = read_ptr(&Path::new(&gitdir).join("gitdir"), Path::new(&gitdir))?;
        // Identity, not spelling — the third copy of this walk, and the same
        // reason as the other two (roots.rs `same_path`): an 8.3 component, a
        // drive-letter case, or a junction makes two names for one file, and a
        // byte compare here fails OPEN, silently answering with the worktree
        // where main is the right answer.
        if !crate::roots::same_path(&reverse, &marker.to_string_lossy()) {
            return None;
        }
        Some(PathBuf::from(path_dirname(&common_git_dir)))
    };
    walk().unwrap_or_else(|| ctx.root.clone())
}
