// bee hook model-guard — Rust port of hooks/bee-model-guard.mjs (PreToolUse,
// Agent|Task|spawn_agent). HOT path: the deny/allow decision, the stderr deny
// reason, the dispatch.jsonl audit line and the hooks.jsonl deny line are all
// fully native and byte-identical to the Node wrapper.
//
// Ported lib functions (provenance, all from the vendored <root>/.bee/bin/lib
// copies, byte-identical to packages/bee/lib at port time):
//   - dispatch-guard.mjs: evaluateDispatch (evaluateClaudeDispatch +
//     evaluateCodexSpawn), ANCHORED_TIER_MARKER_RE / ANCHORED_CODEX_TIER_-
//     MARKER_RE, PINNED_AGENT_TYPE, configuredModelSet, deriveEconomics.
//   - state.mjs: normalizeTierValue / normalizeModels (DEFAULT_MODELS seed),
//     resolveTier (3-arg form; purpose never passed => cli slots refuse),
//     resolveAdvisor, modelForTier (inlined into configured_model_set),
//     hookEnabled (over the merged tracked+overlay config).
//
// CUTOVER (2026-08-01). The one strangler bail this hook had — a
// present-but-corrupt .bee/config.json or .bee/config.local.json, whose Node
// readJson warning embedded V8's own parse-error message — is NATIVE now, in
// crate::state::read_config_raw: it warns in bee's own words and takes
// readConfig's fallback, so the unreadable file reads as absent and the merge
// proceeds from whatever survives (the sibling overlay still applies). The
// guard therefore evaluates against the same config Node would have used, and
// still denies/allows identically.
//
// Strangler bails (Outcome::Delegate, all before any output/log write): none
// remain.
// Known accepted divergence (not detectable natively): a vendored state.mjs /
// dispatch-guard.mjs that PARSES as present but throws on import (Node:
// crash-log + exit 0). A wholly MISSING dispatch-guard.mjs is handled: crash
// line + exit 0, like Node's failed dynamic import (log text differs, shape
// matches).

use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::read_config_raw;
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::ExitCode;

const HOOK_NAME: &str = "model-guard";
const CODEX_SPAWN_TOOL: &str = "spawn_agent";

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    match run_inner(argv, stdin) {
        Ok((code, stderr)) => {
            if !stderr.is_empty() {
                use std::io::Write;
                let _ = std::io::stderr().write_all(stderr.as_bytes());
            }
            Outcome::Done(ExitCode::from(code))
        }
        Err(()) => Outcome::Delegate,
    }
}

/// Returns (exit code, stderr bytes) or Err(()) => delegate to Node.
fn run_inner(argv: &[String], stdin: &str) -> Result<(u8, String), ()> {
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = ctx.root.clone() else {
        return Ok((0, String::new()));
    };

    // Vendored-lib presence gate (fs.existsSync — any file type counts).
    if !crate::hooks::adapter::bee_installed(&root) {
        return Ok((0, String::new()));
    }

    // const toolName = payload.tool_name || payload.toolName || "";
    let tool_name_val = pick_truthy(&ctx.payload, &["tool_name", "toolName"]);
    let is_codex_spawn =
        ctx.payload.get("tool_name").and_then(Value::as_str) == Some(CODEX_SPAWN_TOOL);
    let tool_name: Option<String> = match &tool_name_val {
        Value::String(s) => Some(s.clone()),
        _ => None,
    };
    let is_dispatch_tool =
        matches!(tool_name.as_deref(), Some("Agent") | Some("Task"));
    if !is_codex_spawn && !is_dispatch_tool {
        return Ok((0, String::new()));
    }
    // Reachable only with a string toolName ("Agent"/"Task"/"spawn_agent").
    let tool_name = tool_name.expect("dispatch tools are string-named");

    // hookEnabled over the merged config. read_config_raw warns and treats an
    // unparseable file as absent (readConfig's readJson fallback), so this can
    // no longer bail — the `Err` arm is kept only because the signature is
    // still fallible.
    let config = read_config_raw(&root).unwrap_or_default();
    if matches!(config.get("hooks").and_then(|h| h.get(HOOK_NAME)), Some(Value::Bool(false))) {
        return Ok((0, String::new()));
    }

    // CUTOVER: a dispatch-guard.mjs presence gate stood here — a missing
    // vendored module threw in Node and landed in the fail-open catch. The
    // guard is compiled in now.

    let models = normalize_models(config.get("models"));
    let tool_input = ctx.payload.get("tool_input").cloned().unwrap_or(Value::Null);

    let verdict = if is_codex_spawn {
        evaluate_codex_spawn(&tool_input)
    } else {
        evaluate_claude_dispatch(&tool_input, &models)
    };

    let Some(transport) = verdict.transport else {
        // No opinion — never log, exit 0 silently.
        return Ok((0, String::new()));
    };

    let economics = derive_dispatch_economics(&models, is_codex_spawn, &verdict);

    // toolInput is a plain object on every verdict-carrying path.
    let input_map = match &tool_input {
        Value::Object(m) => m.clone(),
        _ => Map::new(),
    };

    if verdict.deny {
        log_dispatch(&root, &tool_name, &input_map, transport, &verdict, economics.as_ref());
        log_deny(&root, &tool_name, &input_map);
        return Ok((2, verdict.reason.unwrap_or_default()));
    }
    log_dispatch(&root, &tool_name, &input_map, transport, &verdict, economics.as_ref());
    Ok((0, String::new()))
}

// ─── JS semantics helpers ───────────────────────────────────────────────────

fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// `payload.a || payload.b || ""` — first truthy value, else "".
fn pick_truthy(payload: &Map<String, Value>, keys: &[&str]) -> Value {
    for key in keys {
        if let Some(v) = payload.get(*key) {
            if js_truthy(v) {
                return v.clone();
            }
        }
    }
    Value::String(String::new())
}

/// JS regex \s (no /u flag): ASCII whitespace + NBSP + Unicode spaces + BOM.
fn is_js_ws(c: char) -> bool {
    matches!(c,
        '\t' | '\n' | '\u{000b}' | '\u{000c}' | '\r' | ' ' | '\u{00a0}' | '\u{1680}'
        | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}'
        | '\u{3000}' | '\u{feff}')
}

// ─── anchored tier marker (ANCHORED_TIER_MARKER_RE / _CODEX_) ──────────────
// /^\s*\[bee-tier:\s*(ceiling|generation|extraction|review[|advisor])\]/i

const CLAUDE_TIERS: [&str; 4] = ["ceiling", "generation", "extraction", "review"];
const CODEX_TIERS: [&str; 5] = ["ceiling", "generation", "extraction", "review", "advisor"];

fn starts_with_tier_marker(value: &Value, tiers: &[&str]) -> Option<String> {
    let text = value.as_str()?;
    let rest = text.trim_start_matches(is_js_ws);
    let rest = strip_prefix_ascii_ci(rest, "[bee-tier:")?;
    let rest = rest.trim_start_matches(is_js_ws);
    for tier in tiers {
        if let Some(after) = strip_prefix_ascii_ci(rest, tier) {
            if after.starts_with(']') {
                return Some(tier.to_string());
            }
        }
    }
    None
}

fn strip_prefix_ascii_ci<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if text.len() < prefix.len() {
        return None;
    }
    let (head, tail) = text.split_at_checked(prefix.len())?;
    head.eq_ignore_ascii_case(prefix).then_some(tail)
}

fn marker_tier(tool_input: &Map<String, Value>) -> Option<String> {
    // description FIRST, then prompt (dispatch-guard.mjs markerTier order).
    tool_input
        .get("description")
        .and_then(|d| starts_with_tier_marker(d, &CLAUDE_TIERS))
        .or_else(|| tool_input.get("prompt").and_then(|p| starts_with_tier_marker(p, &CLAUDE_TIERS)))
}

// ─── state.mjs model config (normalizeTierValue / normalizeModels) ─────────

#[derive(Clone, Debug, PartialEq)]
enum Slot {
    /// Key absent after normalize (only the advisor slot defaults here).
    Unset,
    Null,
    /// A plain model name, or a {model[, effort]} object (same resolution).
    Name(String),
    Cli(String),
    /// {kind:'native',...} or an explicit-fallback composite's native primary.
    Native(String),
}

#[derive(Clone, Debug)]
struct Slots {
    extraction: Slot,
    generation: Slot,
    review: Slot,
    advisor: Slot,
}

struct Models {
    claude: Slots,
    codex: Slots,
}

/// normalizeTierValue — None means "undefined" (invalid shape, keep default).
fn normalize_tier_value(value: &Value) -> Option<Slot> {
    match value {
        Value::String(s) if !s.trim().is_empty() => Some(Slot::Name(s.trim().to_string())),
        Value::Null => Some(Slot::Null),
        Value::Object(obj) => {
            let str_field = |key: &str| -> Option<String> {
                obj.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(String::from)
            };
            let kind = obj.get("kind").and_then(Value::as_str);
            if kind == Some("cli") {
                if let Some(command) = str_field("command") {
                    return Some(Slot::Cli(command));
                }
            }
            if kind == Some("native") {
                if let Some(model) = str_field("model") {
                    return Some(Slot::Native(model));
                }
            }
            // Explicit-fallback composite: {primary:{kind:'native', model}, ...}.
            if let Some(Value::Object(primary)) = obj.get("primary") {
                if primary.get("kind").and_then(Value::as_str) == Some("native") {
                    if let Some(model) = primary
                        .get("model")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                    {
                        return Some(Slot::Native(model.to_string()));
                    }
                }
            }
            if obj.get("kind").is_none() {
                if let Some(model) = str_field("model") {
                    return Some(Slot::Name(model));
                }
            }
            None
        }
        _ => None,
    }
}

fn normalize_models(raw: Option<&Value>) -> Models {
    // DEFAULT_MODELS (state.mjs).
    let mut out = Models {
        claude: Slots {
            extraction: Slot::Name("haiku".into()),
            generation: Slot::Name("sonnet".into()),
            review: Slot::Name("opus".into()),
            advisor: Slot::Unset,
        },
        codex: Slots {
            extraction: Slot::Null,
            generation: Slot::Null,
            review: Slot::Null,
            advisor: Slot::Unset,
        },
    };
    let Some(Value::Object(raw)) = raw else { return out };
    for rt in ["claude", "codex"] {
        let Some(Value::Object(src)) = raw.get(rt) else { continue };
        let dst = if rt == "claude" { &mut out.claude } else { &mut out.codex };
        for slot in ["extraction", "generation", "review", "advisor"] {
            let Some(value) = src.get(slot) else { continue };
            if let Some(v) = normalize_tier_value(value) {
                match slot {
                    "extraction" => dst.extraction = v,
                    "generation" => dst.generation = v,
                    "review" => dst.review = v,
                    _ => dst.advisor = v,
                }
            }
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq)]
enum Resolved {
    Inherit,
    Model(String),
    Budget,
    Refused,
    Native(String),
}

impl Resolved {
    fn model_name(&self) -> Option<&str> {
        match self {
            Resolved::Model(m) | Resolved::Native(m) => Some(m),
            _ => None,
        }
    }
}

/// state.mjs resolveTier (3-arg form; no purpose => cli slots refuse).
fn resolve_tier(models: &Models, slot: &str, runtime: &str) -> Resolved {
    if slot == "ceiling" {
        return Resolved::Inherit;
    }
    let slots = if runtime == "codex" { &models.codex } else { &models.claude };
    let s = if matches!(slot, "extraction" | "generation" | "review") { slot } else { "generation" };
    let mut value = match s {
        "extraction" => &slots.extraction,
        "review" => &slots.review,
        _ => &slots.generation,
    };
    if matches!(value, Slot::Null | Slot::Unset) && s == "review" {
        value = &slots.generation; // review falls back to generation
    }
    match value {
        Slot::Null | Slot::Unset => Resolved::Budget,
        Slot::Name(m) => Resolved::Model(m.clone()),
        Slot::Cli(_) => Resolved::Refused,
        Slot::Native(m) => Resolved::Native(m.clone()),
    }
}

/// state.mjs resolveAdvisor — None = "no advisor".
fn resolve_advisor(models: &Models, runtime: &str) -> Option<Resolved> {
    let slots = if runtime == "codex" { &models.codex } else { &models.claude };
    match &slots.advisor {
        Slot::Unset | Slot::Null => None,
        Slot::Name(m) => Some(Resolved::Model(m.clone())),
        Slot::Cli(_) => Some(Resolved::Refused), // {type:'cli'} — never a model member
        Slot::Native(m) => Some(Resolved::Native(m.clone())),
    }
}

/// dispatch-guard.mjs configuredModelSet: CONFIGURABLE_SLOTS models + the
/// advisor slot's own resolved model (cnt-7 union).
fn configured_model_set(models: &Models) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for slot in ["extraction", "generation", "review"] {
        if let Resolved::Model(m) = resolve_tier(models, slot, "claude") {
            if !m.trim().is_empty() {
                set.insert(m.trim().to_string());
            }
        }
    }
    if let Some(Resolved::Model(m)) = resolve_advisor(models, "claude") {
        if !m.trim().is_empty() {
            set.insert(m.trim().to_string());
        }
    }
    set
}

// ─── the verdict (dispatch-guard.mjs evaluateDispatch) ─────────────────────

struct Verdict {
    deny: bool,
    /// None = noOpinion — the caller never logs.
    transport: Option<&'static str>,
    reason: Option<String>,
    tier: Option<String>,
    model: Option<String>,
    /// Mirrors dispatch-guard's result shape; the audit line reads
    /// subagent_type straight from tool_input like the .mjs does.
    #[allow(dead_code)]
    subagent_type: Option<String>,
}

fn no_opinion() -> Verdict {
    Verdict { deny: false, transport: None, reason: None, tier: None, model: None, subagent_type: None }
}

fn allow(transport: &'static str, tier: Option<String>, model: Option<String>, subagent_type: Option<String>) -> Verdict {
    Verdict { deny: false, transport: Some(transport), reason: None, tier, model, subagent_type }
}

fn deny(reason: String, transport: &'static str, tier: Option<String>, model: Option<String>, subagent_type: Option<String>) -> Verdict {
    Verdict { deny: true, transport: Some(transport), reason: Some(reason), tier, model, subagent_type }
}

const PINNED_AGENT_TYPE: [(&str, &str); 3] =
    [("generation", "bee-gather"), ("extraction", "bee-extract"), ("review", "bee-review")];

fn pinned_type_for(tier: &str) -> &'static str {
    PINNED_AGENT_TYPE
        .iter()
        .find(|(t, _)| *t == tier)
        .map(|(_, p)| *p)
        .unwrap_or("undefined") // unreachable: ceiling is exempted before lookup
}

fn evaluate_codex_spawn(tool_input: &Value) -> Verdict {
    let Value::Object(obj) = tool_input else { return no_opinion() };
    let Some(message) = obj.get("message").and_then(Value::as_str) else { return no_opinion() };
    if message.is_empty() {
        return no_opinion();
    }
    if let Some(tier) =
        starts_with_tier_marker(&Value::String(message.to_string()), &CODEX_TIERS)
    {
        return allow("codex-spawn-marker", Some(tier), None, None);
    }
    let reason = "bee-model-guard: every Codex spawn_agent needs an explicit tier — its \
message must OPEN with a [bee-tier: <tier>] marker (decision 0023 \
parity, codex-native-runtime-v2 D4, i54-closeout D1). A marker anywhere but the \
start of the message does not count, and a marker in any other field is ignored; \
without one the spawned worker silently inherits the session model.\n\
FIX: begin the spawn message with the marker, e.g. \
\"[bee-tier: generation] <task>\" (tiers: ceiling/generation/extraction/review/advisor)."
        .to_string();
    deny(reason, "codex-spawn-unmarked", None, None, None)
}

fn evaluate_claude_dispatch(tool_input: &Value, models: &Models) -> Verdict {
    let Value::Object(obj) = tool_input else { return no_opinion() };

    let model_param: Option<String> = obj
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);
    let tier = marker_tier(obj);
    let subagent_type: Option<String> =
        obj.get("subagent_type").and_then(Value::as_str).map(String::from);

    // (0) Pinned-type rule (W3, AO5/AO10/AO11).
    if let Some(t) = &tier {
        if t != "ceiling" && subagent_type.as_deref() == Some("general-purpose") {
            let pinned = pinned_type_for(t);
            let reason = format!(
                "bee-model-guard: [bee-tier: {t}] must spawn its pinned agent type, not \
subagent_type: \"general-purpose\" — general-purpose carries no tier identity and \
would run under whatever runtime default is in effect, not the rendered bee agent \
for this tier (AO5/AO10).\n\
FIX: set subagent_type: \"{pinned}\" (bee's rendered agent for the {t} tier), \
or use \"Explore\" for a read-only gather that does not need the rendered agent."
            );
            return deny(reason, "generic-type-denied", tier, model_param, subagent_type);
        }
    }

    // (1) Marker + model param — AO5 strict equality.
    if let (Some(t), Some(param)) = (&tier, &model_param) {
        let resolved = resolve_tier(models, t, "claude");
        if let Resolved::Model(resolved_model) = &resolved {
            if param == resolved_model {
                return allow("model-param", tier, model_param, subagent_type);
            }
            let reason = format!(
                "bee-model-guard: [bee-tier: {t}] resolves to model \"{resolved_model}\", but \
the dispatch carries model: \"{param}\" — the tier label and the param \
disagree, so the dispatch would run on the param while the audit records the \
tier (AO5: config is the authority, the model does not get a vote).\n\
FIX: set model: \"{resolved_model}\" to match the {t} tier, or drop the \
marker and declare the tier whose configured model is the one you want."
            );
            return deny(reason, "param-tier-mismatch", tier, model_param, subagent_type);
        }
        let cli_note = if resolved == Resolved::Refused { " (the slot is a cli executor)" } else { "" };
        let reason = format!(
            "bee-model-guard: [bee-tier: {t}] resolves to no model name{cli_note}\
, but the dispatch carries model: \"{param}\". The marker would record one \
thing in dispatch.jsonl while the subagent actually runs on the param.\n\
FIX: drop the model param (the marker alone selects the tier), or drop the marker \
and declare the tier whose configured model equals the param you intended."
        );
        return deny(reason, "param-on-nameless-tier", tier, model_param, subagent_type);
    }

    // (2) Model param, no marker — B5 membership against configured tier slots.
    if let Some(param) = &model_param {
        let member_set = configured_model_set(models);
        if member_set.is_empty() || member_set.contains(param) {
            return allow("model-param", None, model_param, subagent_type);
        }
        let configured = member_set.iter().cloned().collect::<Vec<_>>().join(", ");
        let reason = format!(
            "bee-model-guard: model: \"{param}\" is not a model configured for any claude \
tier — a param outside config selects an unaudited model and, for an up-dispatch, \
hides ceiling scarcity (AO5/B5: config is the sole authority; there is no hardcoded \
allowlist).\n\
FIX: use one of the configured models ({configured}); or, for a session-model \
dispatch, add [bee-tier: ceiling] (ceiling = the session model) to the \
prompt/description; or add this model to a configured tier slot in .bee/config.json."
        );
        return deny(reason, "param-not-configured", None, model_param, subagent_type);
    }

    // (3) Marker, no param — B4(1)/W10.
    if let Some(t) = &tier {
        let resolved = resolve_tier(models, t, "claude");
        if resolved == Resolved::Refused {
            let reason = format!(
                "bee-model-guard: [bee-tier: {t}] resolves to a cli executor, which an \
in-family Agent/Task subagent cannot be — a cli tier runs as an external process, \
not a spawned subagent.\n\
FIX: dispatch it through the external-executor gather path — a Bash call running \
the configured command verbatim with the prompt on stdin (resolveTier(root, slot, \
runtime, {{for:'gather'}}) returns {{type:'cli', command}}). Do not attach a model \
param; the cli command names its own model."
            );
            return deny(reason, "cli-tier-denied", tier, None, subagent_type);
        }
        return allow("marker", tier, None, subagent_type);
    }

    // (4) Bare — deny; resolve the generation slot for the FIX.
    let gen_resolved = resolve_tier(models, "generation", "claude");
    let bare_fix = if let Resolved::Model(gen_model) = &gen_resolved {
        format!(
            "FIX: pass model: \"{gen_model}\" for the generation tier, or add \
[bee-tier: ceiling] (or another tier: generation/extraction/review) to the prompt/description."
        )
    } else {
        "FIX: add [bee-tier: ceiling] (or another tier: generation/extraction/review) to the \
prompt/description; the generation tier is a cli executor or unconfigured, so run it \
through the external-executor gather path (a Bash call with the command verbatim and \
the prompt on stdin) rather than a model param."
            .to_string()
    };
    let reason = format!(
        "bee-model-guard: every Agent/Task dispatch needs an explicit tier — a `model` \
param or a `[bee-tier: <tier>]` marker in the prompt/description (decision 0023). \
A bare dispatch would silently inherit the most expensive session model.\n{bare_fix}"
    );
    deny(reason, "bare-denied", None, None, subagent_type)
}

// ─── dispatch economics (dispatch-guard.mjs deriveEconomics, g22-2) ────────

fn derive_dispatch_economics(
    models: &Models,
    is_codex_spawn: bool,
    verdict: &Verdict,
) -> Option<Map<String, Value>> {
    let channel = if is_codex_spawn { "codex-native" } else { "claude-agent" };
    let runtime = if is_codex_spawn { "codex" } else { "claude" };
    let resolved = verdict.tier.as_ref().map(|t| resolve_tier(models, t, runtime));
    let resolved_model = resolved.as_ref().and_then(|r| r.model_name());
    let param_model = verdict.model.as_deref();

    // nativeConfirmed is never passed by the hook => always false.
    let enforcement = if is_codex_spawn {
        "prompt-budget"
    } else if param_model.is_some() {
        "model-param"
    } else {
        "prompt-budget"
    };
    let (effective_model, effective_status) = if is_codex_spawn {
        (None, "inherited-or-unknown")
    } else if let Some(p) = param_model {
        (Some(p), "pinned")
    } else {
        (None, "unverified")
    };
    let requested_model = param_model.or(resolved_model);

    let str_or_null = |v: Option<&str>| v.map_or(Value::Null, |s| Value::String(s.to_string()));
    let mut out = Map::new();
    out.insert("logical_tier".into(), str_or_null(verdict.tier.as_deref()));
    out.insert("requested_model".into(), str_or_null(requested_model));
    out.insert("effective_model".into(), str_or_null(effective_model));
    out.insert("effective_model_status".into(), Value::String(effective_status.to_string()));
    out.insert("channel".into(), Value::String(channel.to_string()));
    out.insert("enforcement".into(), Value::String(enforcement.to_string()));
    Some(out)
}

// ─── audit logs ─────────────────────────────────────────────────────────────

/// JS String.prototype.slice(0, n) — UTF-16 code units. When the cut lands
/// inside a surrogate pair we keep 119 units instead of emitting the lone
/// surrogate Node would (log-only field; shape unchanged).
fn utf16_slice(text: &str, max_units: usize) -> String {
    let mut units = 0usize;
    let mut out = String::new();
    for c in text.chars() {
        let w = c.len_utf16();
        if units + w > max_units {
            break;
        }
        units += w;
        out.push(c);
    }
    out
}

fn log_dispatch(
    root: &Path,
    tool_name: &str,
    tool_input: &Map<String, Value>,
    transport: &str,
    verdict: &Verdict,
    economics: Option<&Map<String, Value>>,
) {
    // Fail-open: any fs failure is swallowed — auditing never blocks a dispatch.
    let logs_dir = root.join(".bee").join("logs");
    if std::fs::create_dir_all(&logs_dir).is_err() {
        return;
    }
    let description = tool_input
        .get("description")
        .and_then(Value::as_str)
        .map(|d| utf16_slice(d, 120))
        .unwrap_or_default();
    let str_or_null = |v: Option<&str>| v.map_or(Value::Null, |s| Value::String(s.to_string()));
    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(now_iso()));
    entry.insert("tool".into(), Value::String(tool_name.to_string()));
    entry.insert("transport".into(), Value::String(transport.to_string()));
    entry.insert("model".into(), str_or_null(verdict.model.as_deref()));
    entry.insert("tier".into(), str_or_null(verdict.tier.as_deref()));
    entry.insert(
        "subagent_type".into(),
        str_or_null(tool_input.get("subagent_type").and_then(Value::as_str)),
    );
    entry.insert("description".into(), Value::String(description));
    if let Some(econ) = economics {
        for (k, v) in econ {
            entry.insert(k.clone(), v.clone());
        }
    }
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs_dir.join("dispatch.jsonl"))
    {
        let _ = f.write_all(format!("{}\n", jsjson::stringify(&Value::Object(entry))).as_bytes());
    }
}

fn log_deny(root: &Path, tool_name: &str, tool_input: &Map<String, Value>) {
    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(now_iso()));
    entry.insert("hook".into(), Value::String(HOOK_NAME.to_string()));
    entry.insert("event".into(), Value::String("deny".to_string()));
    entry.insert("tool_name".into(), Value::String(tool_name.to_string()));
    entry.insert(
        "tool_input_keys".into(),
        Value::Array(tool_input.keys().map(|k| Value::String(k.clone())).collect()),
    );
    append_hook_log(root, &Value::Object(entry));
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    /// Fixture with .bee/onboarding.json + vendored-lib markers + a config.
    fn fixture(config: &Value) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".bee").join("bin").join("lib")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        std::fs::write(root.join(".bee").join("bin").join("lib").join("state.mjs"), "// stub\n").unwrap();
        std::fs::write(
            root.join(".bee").join("bin").join("lib").join("dispatch-guard.mjs"),
            "// stub\n",
        )
        .unwrap();
        std::fs::write(
            root.join(".bee").join("config.json"),
            format!("{}\n", serde_json::to_string_pretty(config).unwrap()),
        )
        .unwrap();
        dir
    }

    fn repo_config() -> Value {
        json!({
            "models": {
                "claude": { "extraction": "haiku", "generation": "sonnet", "review": "opus", "advisor": "fable" },
                "codex": { "extraction": "gpt-5.5", "generation": "gpt-5.5" }
            }
        })
    }

    fn run_payload(root: &Path, payload: Value) -> (u8, String) {
        let mut body = payload;
        body["cwd"] = json!(root.to_string_lossy());
        run_inner(&[], &serde_json::to_string(&body).unwrap()).expect("native run")
    }

    fn last_jsonl(file: PathBuf) -> Option<Value> {
        let text = std::fs::read_to_string(file).ok()?;
        let line = text.lines().filter(|l| !l.trim().is_empty()).last()?;
        serde_json::from_str(line).ok()
    }

    fn dispatch_log(root: &Path) -> PathBuf {
        root.join(".bee").join("logs").join("dispatch.jsonl")
    }

    #[test]
    fn bare_dispatch_denied_with_fix_naming_generation_model() {
        let fx = fixture(&repo_config());
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "implement the widget", "description": "some description"}}),
        );
        assert_eq!(code, 2);
        assert!(stderr.contains("bee-tier") && stderr.contains("FIX"));
        assert!(stderr.contains("sonnet"));
        // deny line in hooks.jsonl with matching tool_input_keys
        let deny = last_jsonl(fx.path().join(".bee").join("logs").join("hooks.jsonl")).unwrap();
        assert_eq!(deny["hook"], "model-guard");
        assert_eq!(deny["event"], "deny");
        assert_eq!(deny["tool_name"], "Agent");
        assert_eq!(deny["tool_input_keys"], json!(["prompt", "description"]));
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "bare-denied");
        assert_eq!(d["channel"], "claude-agent");
        assert_eq!(d["effective_model_status"], "unverified");
        assert_eq!(d["requested_model"], Value::Null);
    }

    #[test]
    fn marker_allows_anchored_only() {
        let fx = fixture(&repo_config());
        for (payload, expect) in [
            (json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: ceiling] do the thing"}}), 0),
            (json!({"tool_name": "Agent", "tool_input": {"description": "[bee-tier: generation] short", "prompt": "no marker"}}), 0),
            (json!({"tool_name": "Agent", "tool_input": {"description": "[BEE-TIER: Generation] mixed case"}}), 0),
            (json!({"tool_name": "Agent", "tool_input": {"prompt": "   [bee-tier: ceiling] with leading ws"}}), 0),
            // embedded after text — denied (P1-1 anchor rule)
            (json!({"tool_name": "Agent", "tool_input": {"prompt": format!("{} [bee-tier: ceiling] rest", "x".repeat(100))}}), 2),
            (json!({"tool_name": "Agent", "tool_input": {"description": "text before [bee-tier: ceiling] marker"}}), 2),
            // long tail stays allowed (no window cutoff)
            (json!({"tool_name": "Agent", "tool_input": {"prompt": format!("[bee-tier: ceiling] {}", "y".repeat(2000))}}), 0),
        ] {
            let (code, _) = run_payload(fx.path(), payload.clone());
            assert_eq!(code, expect, "payload: {payload}");
        }
    }

    #[test]
    fn non_dispatch_and_malformed_inputs_fail_open() {
        let fx = fixture(&repo_config());
        for payload in [
            json!({"tool_name": "Agent"}),
            json!({"tool_name": "Agent", "tool_input": "oops"}),
            json!({"tool_name": "Edit", "tool_input": {}}),
            json!({"toolName": "spawn_agent", "tool_input": {"agent_type": "worker", "message": "no marker"}}),
        ] {
            let (code, stderr) = run_payload(fx.path(), payload);
            assert_eq!(code, 0);
            assert_eq!(stderr, "");
        }
    }

    #[test]
    fn disabled_hook_allows_everything() {
        let fx = fixture(&json!({"hooks": {"model-guard": false}}));
        let (code, stderr) =
            run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "bare"}}));
        assert_eq!(code, 0);
        assert_eq!(stderr, "");
        assert!(!dispatch_log(fx.path()).exists());
    }

    #[test]
    fn model_param_membership_table() {
        let fx = fixture(&repo_config());
        // member allow + economics pinned
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"model": "haiku", "description": "pattern extractor", "subagent_type": "general-purpose"}}));
        assert_eq!(code, 0);
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "model-param");
        assert_eq!(d["model"], "haiku");
        assert_eq!(d["subagent_type"], "general-purpose");
        assert_eq!(d["description"], "pattern extractor");
        assert_eq!(d["enforcement"], "model-param");
        assert_eq!(d["effective_model_status"], "pinned");
        assert_eq!(d["effective_model"], "haiku");
        assert_eq!(d["requested_model"], "haiku");
        // advisor model is a member (cnt-7 union)
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"model": "fable"}}));
        assert_eq!(code, 0);
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "model-param");
        // non-member denied, FIX lists every member sorted, ceiling route taught
        let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"model": "banana"}}));
        assert_eq!(code, 2);
        for m in ["haiku", "sonnet", "opus", "fable"] {
            assert!(stderr.contains(m), "missing {m} in {stderr}");
        }
        assert!(stderr.contains("(fable, haiku, opus, sonnet)"));
        assert!(stderr.contains("[bee-tier: ceiling]"));
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "param-not-configured");
        assert_eq!(d["model"], "banana");
    }

    #[test]
    fn marker_plus_param_agreement_rules() {
        let fx = fixture(&repo_config());
        // agree -> allow
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] go", "model": "sonnet"}}));
        assert_eq!(code, 0);
        // disagree -> deny naming the tier's model
        let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] go", "model": "opus"}}));
        assert_eq!(code, 2);
        assert!(stderr.contains("\"sonnet\""));
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "param-tier-mismatch");
        // ceiling + param -> deny "drop the model param"
        let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: ceiling] go", "model": "sonnet"}}));
        assert_eq!(code, 2);
        assert!(stderr.contains("drop the model param"));
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "param-on-nameless-tier");
        // review + its own model -> allow
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: review] check", "model": "opus"}}));
        assert_eq!(code, 0);
        // marker economics: prompt-budget/unverified with requested from config
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Task", "tool_input": {"prompt": "[bee-tier: review] check the diff"}}));
        assert_eq!(code, 0);
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "marker");
        assert_eq!(d["tier"], "review");
        assert_eq!(d["tool"], "Task");
        assert_eq!(d["enforcement"], "prompt-budget");
        assert_eq!(d["effective_model_status"], "unverified");
        assert_eq!(d["effective_model"], Value::Null);
        assert_eq!(d["requested_model"], "opus");
    }

    #[test]
    fn cli_slot_and_empty_and_malformed_model_sets() {
        // cli-shaped generation slot
        let cli = fixture(&json!({"models": {"claude": {
            "extraction": "haiku",
            "generation": {"kind": "cli", "command": "codex exec -m gpt-5.5 -s read-only -"},
            "review": "opus"
        }}}));
        let (code, stderr) = run_payload(cli.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] gather"}}));
        assert_eq!(code, 2);
        assert!(stderr.contains("{for:'gather'}") && stderr.contains("stdin"));
        assert!(!stderr.contains("model: \"") && !stderr.contains("gpt-5.5"));
        let d = last_jsonl(dispatch_log(cli.path())).unwrap();
        assert_eq!(d["transport"], "cli-tier-denied");
        // bare deny under cli generation names no model
        let (code, stderr) = run_payload(cli.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "no tier given"}}));
        assert_eq!(code, 2);
        assert!(stderr.contains("bee-tier") && stderr.contains("FIX"));
        assert!(!stderr.contains("model: \""));
        // empty member set fail-opens a bare param
        let empty = fixture(&json!({"models": {"claude": {"extraction": null, "generation": null, "review": null}}}));
        let (code, stderr) = run_payload(empty.path(), json!({"tool_name": "Agent", "tool_input": {"model": "anything-at-all"}}));
        assert_eq!(code, 0);
        assert_eq!(stderr, "");
        // malformed slot shapes fall back to seeded defaults (no crash)
        let malformed = fixture(&json!({"models": {"claude": {"extraction": {"nonsense": true}, "generation": {"foo": "bar"}, "review": 42}}}));
        let (code, _) = run_payload(malformed.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] go"}}));
        assert_eq!(code, 0);
    }

    #[test]
    fn pinned_type_rule() {
        let fx = fixture(&repo_config());
        for (tier, pinned) in PINNED_AGENT_TYPE {
            let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": format!("[bee-tier: {tier}] go"), "subagent_type": "general-purpose"}}));
            assert_eq!(code, 2, "tier {tier}");
            assert!(stderr.contains(pinned));
            let d = last_jsonl(dispatch_log(fx.path())).unwrap();
            assert_eq!(d["transport"], "generic-type-denied");
            assert_eq!(d["tier"], tier);
        }
        // matching param does not rescue general-purpose
        let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] go", "model": "sonnet", "subagent_type": "general-purpose"}}));
        assert_eq!(code, 2);
        assert!(stderr.contains("bee-gather"));
        // ceiling has no pinned agent
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: ceiling] go", "subagent_type": "general-purpose"}}));
        assert_eq!(code, 0);
        // Explore / own pinned type / absent subagent_type allowed
        for st in [json!("Explore"), json!("bee-gather")] {
            let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] go", "subagent_type": st}}));
            assert_eq!(code, 0);
        }
        // bare param + general-purpose (no marker) stays allowed
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"model": "haiku", "subagent_type": "general-purpose"}}));
        assert_eq!(code, 0);
    }

    #[test]
    fn codex_spawn_rules() {
        let fx = fixture(&repo_config());
        // anchored marker -> allow, codex economics inherited-or-unknown
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "spawn_agent", "tool_input": {"agent_type": "worker", "message": "[bee-tier: generation] gather the callers"}}));
        assert_eq!(code, 0);
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "codex-spawn-marker");
        assert_eq!(d["tier"], "generation");
        assert_eq!(d["tool"], "spawn_agent");
        assert_eq!(d["channel"], "codex-native");
        assert_eq!(d["enforcement"], "prompt-budget");
        assert_eq!(d["effective_model_status"], "inherited-or-unknown");
        assert_eq!(d["effective_model"], Value::Null);
        assert_eq!(d["requested_model"], "gpt-5.5");
        // mid-message marker -> deny, Codex-shaped
        let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "spawn_agent", "tool_input": {"agent_type": "worker", "message": "please [bee-tier: generation] do it"}}));
        assert_eq!(code, 2);
        assert!(stderr.contains("spawn_agent") && stderr.contains("message must OPEN"));
        assert!(!stderr.contains("Agent/Task"));
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "codex-spawn-unmarked");
        assert_eq!(d["effective_model_status"], "inherited-or-unknown");
        // empty / missing / non-string message fail open
        for ti in [
            json!({"agent_type": "worker", "message": ""}),
            json!({"agent_type": "worker"}),
            json!({"agent_type": "worker", "message": {"not": "a string"}}),
            json!("oops"),
        ] {
            let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "spawn_agent", "tool_input": ti}));
            assert_eq!(code, 0);
            assert_eq!(stderr, "");
        }
        // agent_type never keys the verdict; prompt marker never rescues
        for ti in [
            json!({"agent_type": "default", "message": "no marker here at all"}),
            json!({"agent_type": "explorer", "message": "no marker here at all"}),
            json!({"message": "no marker here at all"}),
            json!({"agent_type": "worker", "message": "reply with OK", "prompt": "[bee-tier: generation] task"}),
            json!({"task_name": "wt-a1", "message": "no marker here at all", "fork_turns": "none"}),
        ] {
            let (code, _) = run_payload(fx.path(), json!({"tool_name": "spawn_agent", "tool_input": ti}));
            assert_eq!(code, 2);
        }
        // doc-canonical marked shape + extras tolerated + advisor tier
        for ti in [
            json!({"task_name": "wt-a1", "message": "[bee-tier: generation] gather", "fork_turns": "none"}),
            json!({"agent_type": "worker", "message": "[bee-tier: review] check", "extra": 1, "task_name": "x"}),
            json!({"agent_type": "worker", "message": "[bee-tier: advisor] consult", "model": "totally-different", "reasoning_effort": "extreme", "fork_turns": "full"}),
        ] {
            let (code, _) = run_payload(fx.path(), json!({"tool_name": "spawn_agent", "tool_input": ti}));
            assert_eq!(code, 0);
        }
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "codex-spawn-marker");
        assert_eq!(d["tier"], "advisor");
    }

    // R5 port of test_model_guard.mjs rows 11/12/15/16 — the fail-open arms
    // the existing malformed-input test never reaches, because `run_payload`
    // always hands the hook a well-formed JSON object with a cwd.
    #[test]
    fn rows11_16_unparseable_and_non_object_stdin_fail_open() {
        let fx = fixture(&repo_config());
        let cwd = fx.path().to_string_lossy().into_owned();
        for (row, stdin) in [
            ("row11: junk stdin", "not json at all {{{".to_string()),
            ("row11b: empty stdin", String::new()),
            ("row15: top-level null", "null".to_string()),
            ("row16: top-level array", "[]".to_string()),
            // A well-formed object whose cwd is not a string: the root
            // resolver must degrade, not panic.
            ("object cwd", json!({"cwd": {"not": "a string"}}).to_string()),
            // The dispatch shape is present but the envelope is not an
            // object — the tool_name read must not reach into a non-map.
            (
                "row16b: array carrying a dispatch-shaped element",
                json!([{"tool_name": "Agent", "tool_input": {"prompt": "bare"}, "cwd": cwd}])
                    .to_string(),
            ),
        ] {
            let (code, stderr) = run_inner(&[], &stdin).expect("must decide natively");
            assert_eq!(code, 0, "{row} must fail open");
            assert_eq!(stderr, "", "{row} must stay silent");
        }
    }

    #[test]
    fn row12_no_repo_root_is_silent_success() {
        // A bare temp dir with no `.bee/` at all: there is no repo to read a
        // model set from, so the guard has no verdict to give.
        let dir = tempfile::tempdir().unwrap();
        let body = json!({
            "tool_name": "Agent",
            "tool_input": { "prompt": "bare dispatch, no marker, no model" },
            "cwd": dir.path().to_string_lossy(),
        });
        let (code, stderr) =
            run_inner(&[], &serde_json::to_string(&body).unwrap()).expect("native");
        assert_eq!(code, 0);
        assert_eq!(stderr, "");
        // Control: the SAME payload inside a real fixture repo denies — so
        // the exit 0 above is the missing root, not a toothless guard.
        let fx = fixture(&repo_config());
        let (denied, _) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "bare dispatch, no marker, no model"}}),
        );
        assert_eq!(denied, 2);
    }

    #[test]
    fn missing_vendored_lib_is_silent_success() {
        // Presence gate (bee-model-guard.mjs's existsSync on the vendored
        // state.mjs): a host mid-vendoring gets no verdict rather than a
        // wrong one.
        let fx = fixture(&repo_config());
        std::fs::remove_file(fx.path().join(".bee").join("onboarding.json")).unwrap();
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "bare"}}),
        );
        assert_eq!(code, 0);
        assert_eq!(stderr, "");
        assert!(
            !dispatch_log(fx.path()).exists(),
            "a gated-off run must not leave dispatch telemetry"
        );
    }

    #[test]
    fn corrupt_config_warns_and_reads_as_absent() {
        // The tracked config will not parse: readConfig's readJson fallback
        // makes it read as absent, so the guard evaluates against the DEFAULT
        // model set — it still decides natively and still denies a bare
        // dispatch, with the default generation model named in the FIX.
        let fx = fixture(&repo_config());
        std::fs::write(fx.path().join(".bee").join("config.json"), "{broken").unwrap();
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "bare"}}),
        );
        assert_eq!(code, 2, "a corrupt config must not soften the guard");
        assert!(stderr.contains("bee-tier") && stderr.contains("FIX"));
        // The repo's own "sonnet" mapping is gone with the unreadable file.
        assert!(!stderr.contains("haiku"));
    }

    #[test]
    fn corrupt_tracked_config_still_honours_a_readable_overlay() {
        // readJson fails open PER FILE: the overlay survives its sibling's
        // corruption, so a hook disabled there is still disabled.
        let fx = fixture(&repo_config());
        std::fs::write(fx.path().join(".bee").join("config.json"), "{broken").unwrap();
        std::fs::write(
            fx.path().join(".bee").join("config.local.json"),
            r#"{"hooks":{"model-guard":false}}"#,
        )
        .unwrap();
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "bare"}}),
        );
        assert_eq!(code, 0);
        assert_eq!(stderr, "");
    }

    #[test]
    fn description_is_truncated_to_120_utf16_units() {
        let fx = fixture(&repo_config());
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"model": "haiku", "description": "z".repeat(300)}}));
        assert_eq!(code, 0);
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["description"].as_str().unwrap().len(), 120);
    }
}
