// bee hook model-guard (PreToolUse, Agent|Task|spawn_agent). HOT path: the
// deny/allow decision, the stderr deny reason, the dispatch.jsonl audit line
// and the hooks.jsonl deny line are all fully native.
//
// A present-but-corrupt .bee/config.json or .bee/config.local.json is native:
// crate::state::read_config_raw warns in bee's own words and takes its
// fallback, so the unreadable file reads as absent and the merge proceeds
// from whatever survives (the sibling overlay still applies).
//
// No branch in this hook still returns Outcome::Delegate; every decision is
// native.

use crate::fsutil::{read_json, ReadJson};
use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context};
use crate::hooks::Outcome;
use crate::textutil::truncate_chars_head;
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
        Ok((code, stdout, stderr)) => {
            use std::io::Write;
            if !stdout.is_empty() {
                let _ = std::io::stdout().write_all(stdout.as_bytes());
            }
            if !stderr.is_empty() {
                let _ = std::io::stderr().write_all(stderr.as_bytes());
            }
            Outcome::Done(ExitCode::from(code))
        }
        Err(()) => Outcome::Delegate,
    }
}

/// Returns (exit code, stdout bytes, stderr bytes) or Err(()) => delegate.
fn run_inner(argv: &[String], stdin: &str) -> Result<(u8, String, String), ()> {
    let ctx = read_hook_context(HOOK_NAME, argv, stdin);
    let Some(root) = ctx.root.clone() else {
        return Ok((0, String::new(), String::new()));
    };

    // Vendored-lib presence gate (fs.existsSync — any file type counts).
    if !crate::hooks::adapter::bee_installed(&root) {
        return Ok((0, String::new(), String::new()));
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
        return Ok((0, String::new(), String::new()));
    }
    // Reachable only with a string toolName ("Agent"/"Task"/"spawn_agent").
    let tool_name = tool_name.expect("dispatch tools are string-named");

    // hookEnabled over the merged config. read_config_raw warns and treats an
    // unparseable file as absent (readConfig's readJson fallback), so a
    // corrupt config just reads as {} — no fallible arm to handle here.
    let config = read_config_raw(&root);
    if matches!(config.get("hooks").and_then(|h| h.get(HOOK_NAME)), Some(Value::Bool(false))) {
        return Ok((0, String::new(), String::new()));
    }

    let models = normalize_models(config.get("models"));
    let tool_input = ctx.payload.get("tool_input").cloned().unwrap_or(Value::Null);

    let mut verdict = if is_codex_spawn {
        evaluate_codex_spawn(&tool_input)
    } else {
        evaluate_claude_dispatch(&tool_input, &models)
    };

    let Some(transport) = verdict.transport else {
        // No opinion — never log, exit 0 silently.
        return Ok((0, String::new(), String::new()));
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
        return Ok((2, String::new(), verdict.reason.unwrap_or_default()));
    }
    // dispatch-label-chokepoint (dlc-2): every dispatch reaches this point,
    // including one written by hand that never called `prepare` — the one
    // case no amount of fixing `prepare` can reach. Runs on top of whatever
    // the verdict above already changed, so a param-tier repair and a label
    // repair on the same dispatch both land in one updatedInput.
    let label_base = verdict
        .updated_input
        .clone()
        .unwrap_or_else(|| Value::Object(input_map.clone()));
    if let Some((fixed, note)) = repair_dispatch_label(&root, is_codex_spawn, &label_base) {
        verdict.updated_input = Some(fixed);
        verdict.notes.push(note);
    }

    // A repaired dispatch is audited as the request that will actually run —
    // logging the field the guard just replaced would put a value in the
    // audit trail that never reached the runtime.
    let audited_input = verdict
        .updated_input
        .as_ref()
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_else(|| input_map.clone());
    log_dispatch(&root, &tool_name, &audited_input, transport, &verdict, economics.as_ref());
    let stdout = match &verdict.updated_input {
        Some(fixed) => repair_stdout(fixed, &verdict.notes),
        None => String::new(),
    };
    Ok((0, stdout, String::new()))
}

/// The repair emission. No `permissionDecision` rides along: a guard that
/// corrects a field is not thereby approving the call — the dispatch takes the
/// host's ordinary permission flow, exactly as an untouched one would
/// (hook-runtime R23). The rewrite is announced twice: to the agent as
/// additionalContext, to the human as a systemMessage.
fn repair_stdout(fixed: &Value, notes: &[String]) -> String {
    let joined = notes.join("; ");
    let mut hso = Map::new();
    hso.insert("hookEventName".into(), Value::String("PreToolUse".into()));
    hso.insert("updatedInput".into(), fixed.clone());
    hso.insert(
        "additionalContext".into(),
        Value::String(format!("bee-model-guard auto-fixed this dispatch: {joined}")),
    );
    let mut out = Map::new();
    out.insert("hookSpecificOutput".into(), Value::Object(hso));
    out.insert("systemMessage".into(), Value::String(format!("bee-model-guard: {joined}")));
    jsjson::stringify(&Value::Object(out))
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
    // description FIRST, then prompt.
    tool_input
        .get("description")
        .and_then(|d| starts_with_tier_marker(d, &CLAUDE_TIERS))
        .or_else(|| tool_input.get("prompt").and_then(|p| starts_with_tier_marker(p, &CLAUDE_TIERS)))
}

// ─── model config (normalize_tier_value / normalize_models) ────────────────

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
    /// herding-tier D1: `{kind:"herding"}` — the dispatch seam turns this
    /// into a `bee herding run` Bash payload; an Agent/Task subagent can no
    /// more be a herding pane than it can be a cli executor (D5).
    Herding,
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
    // opencode-support E4/S4: parsed and resolvable exactly like claude/codex
    // (docs/config-reference.md's models.<rt> schema). NOTE: the dispatch
    // verdicts below (evaluate_claude_dispatch's model-param branches) still
    // resolve against a literal "claude" — OpenCode's Task payload carries no
    // runtime marker to key off of (confirmed live: its `task` tool has no
    // `model` argument at all, so those branches never fire for an OpenCode
    // dispatch either way). The structural half of OpenCode's model-guard
    // mapping is the per-agent `model:` pin in each `.opencode/agent/bee-*.md`
    // file, not a runtime switch here — see plan.md's model-guard fallback row.
    opencode: Slots,
}

/// None means "undefined" (invalid shape, keep default).
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
            // herding-tier D1: no other field is required — `kind` alone
            // routes the slot.
            if kind == Some("herding") {
                return Some(Slot::Herding);
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
    // Default model set.
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
        opencode: Slots {
            extraction: Slot::Null,
            generation: Slot::Null,
            review: Slot::Null,
            advisor: Slot::Unset,
        },
    };
    let Some(Value::Object(raw)) = raw else { return out };
    for rt in ["claude", "codex", "opencode"] {
        let Some(Value::Object(src)) = raw.get(rt) else { continue };
        let dst = match rt {
            "claude" => &mut out.claude,
            "codex" => &mut out.codex,
            _ => &mut out.opencode,
        };
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
    /// herding-tier D5: the slot is `{kind:"herding"}` — an Agent/Task
    /// subagent cannot BE the herding pane; only the herding-exec Bash
    /// path (dispatch prepare's Resolved::Herding arm) may run it.
    Herding,
}

impl Resolved {
    fn model_name(&self) -> Option<&str> {
        match self {
            Resolved::Model(m) | Resolved::Native(m) => Some(m),
            _ => None,
        }
    }
}

/// No purpose => cli slots refuse.
fn resolve_tier(models: &Models, slot: &str, runtime: &str) -> Resolved {
    if slot == "ceiling" {
        return Resolved::Inherit;
    }
    let slots = match runtime {
        "codex" => &models.codex,
        "opencode" => &models.opencode,
        _ => &models.claude,
    };
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
        Slot::Herding => Resolved::Herding,
    }
}

/// None = "no advisor".
fn resolve_advisor(models: &Models, runtime: &str) -> Option<Resolved> {
    let slots = match runtime {
        "codex" => &models.codex,
        "opencode" => &models.opencode,
        _ => &models.claude,
    };
    match &slots.advisor {
        Slot::Unset | Slot::Null => None,
        Slot::Name(m) => Some(Resolved::Model(m.clone())),
        Slot::Cli(_) => Some(Resolved::Refused), // {type:'cli'} — never a model member
        Slot::Native(m) => Some(Resolved::Native(m.clone())),
        Slot::Herding => Some(Resolved::Herding), // never a model member either
    }
}

/// The configurable-slot models plus the advisor slot's own resolved model.
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

// ─── the verdict ────────────────────────────────────────────────────────────

struct Verdict {
    deny: bool,
    /// None = no opinion — the caller never logs.
    transport: Option<&'static str>,
    reason: Option<String>,
    tier: Option<String>,
    model: Option<String>,
    /// The audit line reads subagent_type straight from tool_input.
    #[allow(dead_code)]
    subagent_type: Option<String>,
    /// Set when the guard repaired the request instead of refusing it. Carries
    /// the WHOLE tool_input with the offending field rewritten — never a
    /// partial object, because the host's merge-vs-replace semantics for
    /// updatedInput are not something a guard should bet a dropped prompt on.
    updated_input: Option<Value>,
    notes: Vec<String>,
}

fn no_opinion() -> Verdict {
    Verdict { deny: false, transport: None, reason: None, tier: None, model: None, subagent_type: None, updated_input: None, notes: Vec::new() }
}

fn allow(transport: &'static str, tier: Option<String>, model: Option<String>, subagent_type: Option<String>) -> Verdict {
    Verdict { deny: false, transport: Some(transport), reason: None, tier, model, subagent_type, updated_input: None, notes: Vec::new() }
}

fn deny(reason: String, transport: &'static str, tier: Option<String>, model: Option<String>, subagent_type: Option<String>) -> Verdict {
    Verdict { deny: true, transport: Some(transport), reason: Some(reason), tier, model, subagent_type, updated_input: None, notes: Vec::new() }
}

/// Allow, with one field of the request rewritten on a replacement copy.
fn repair(
    transport: &'static str,
    tier: Option<String>,
    model: Option<String>,
    subagent_type: Option<String>,
    updated_input: Value,
    note: String,
) -> Verdict {
    Verdict {
        deny: false,
        transport: Some(transport),
        reason: None,
        tier,
        model,
        subagent_type,
        updated_input: Some(updated_input),
        notes: vec![note],
    }
}

/// The whole request with one key replaced — the original map is never mutated.
fn with_field(tool_input: &Map<String, Value>, key: &str, value: Value) -> Value {
    let mut copy = tool_input.clone();
    copy.insert(key.to_string(), value);
    Value::Object(copy)
}

/// The tier a rendered agent stands for. `generation` appears twice on
/// purpose: bee-gather reads and bee-build writes, and both run at that
/// tier's model — which is why `pinned_type_for` refuses to answer for it.
const PINNED_AGENT_TYPE: [(&str, &str); 4] = [
    ("generation", "bee-gather"),
    ("generation", "bee-build"),
    ("extraction", "bee-extract"),
    ("review", "bee-review"),
];

fn pinned_type_for(tier: &str) -> &'static str {
    PINNED_AGENT_TYPE
        .iter()
        .find(|(t, _)| *t == tier)
        .map(|(_, p)| *p)
        .unwrap_or("undefined") // unreachable: ceiling is exempted before lookup
}

/// The tier a rendered bee agent type already stands for. These files are
/// generated FROM the tier's configured model at onboarding, so naming one is
/// a tier declaration in every sense that matters — the guard reading it as
/// one is what keeps the tier decision off the caller's memory.
fn tier_for_pinned_type(subagent_type: &str) -> Option<&'static str> {
    PINNED_AGENT_TYPE
        .iter()
        .find(|(_, pinned)| *pinned == subagent_type)
        .map(|(tier, _)| *tier)
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

/// D1(d): the `--kind` a FIX message hands to `dispatch prepare` so it reads
/// the same slot the guard just resolved by tier name. `slot_for_kind`
/// (prepare.rs:33-39) only goes kind -> tier and has no extraction arm, so
/// there is no `--kind` value that resolves the extraction slot today —
/// `None` here means exactly that: a FIX for this tier must name the
/// refused slot's own transport instead of a `--kind` that would silently
/// resolve a different slot (e.g. `advisor`) than the one just refused.
fn dispatch_kind_for_tier(tier: &str) -> Option<&'static str> {
    match tier {
        "review" => Some("reviewer"),
        "generation" => Some("gather"),
        _ => None,
    }
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

    // (0) Pinned-type rule (W3, AO5/AO10/AO11) — REPAIRED, not refused. The
    // tier is already stated; which agent file carries it is a lookup the
    // guard owns outright, so making the caller re-issue the dispatch to
    // supply it buys nothing but a round trip and a chance to guess again.
    if let Some(t) = &tier {
        if t == "generation" && subagent_type.as_deref() == Some("general-purpose") {
            // Two agents, one tier: the guard cannot tell a gather from a
            // cell execution by the tier alone, and guessing picked the
            // read-only one for years — an execution dispatch that could
            // never write. The caller says which; that is one word.
            let reason = "bee-model-guard: [bee-tier: generation] dispatched with subagent_type \
\"general-purpose\", and the generation tier carries TWO rendered agents — the guard will \
not guess which.\n\
FIX: name the one you mean. subagent_type \"bee-build\" executes a cell (reserves, writes, \
commits, caps); subagent_type \"bee-gather\" reads and reports (never writes)."
                .to_string();
            return deny(
                reason,
                "generic-type-denied",
                tier.clone(),
                model_param,
                subagent_type,
            );
        }
        if t != "ceiling" && subagent_type.as_deref() == Some("general-purpose") {
            let pinned = pinned_type_for(t);
            let note = format!(
                "[bee-tier: {t}] dispatched with subagent_type \"general-purpose\" → \"{pinned}\" \
(general-purpose carries no tier identity and would run under the runtime default, \
not the rendered bee agent for this tier)"
            );
            return repair(
                "generic-type-repaired",
                tier.clone(),
                model_param,
                Some(pinned.to_string()),
                with_field(obj, "subagent_type", Value::String(pinned.to_string())),
                note,
            );
        }
    }

    // (1) Marker + model param — AO5 strict equality.
    if let (Some(t), Some(param)) = (&tier, &model_param) {
        let resolved = resolve_tier(models, t, "claude");
        if let Resolved::Model(resolved_model) = &resolved {
            if param == resolved_model {
                return allow("model-param", tier, model_param, subagent_type);
            }
            // The tier and the param disagree — and AO5 already settled who
            // wins: config is the authority, the model does not get a vote.
            // With the winner named in advance there is nothing left to ask,
            // so the param is rewritten to the tier's model rather than
            // bounced back for the caller to guess a second time.
            let note = format!(
                "[bee-tier: {t}] resolves to \"{resolved_model}\", dispatch carried \
model: \"{param}\" → \"{resolved_model}\" (AO5: config is the authority)"
            );
            return repair(
                "param-tier-repaired",
                tier.clone(),
                Some(resolved_model.clone()),
                subagent_type,
                with_field(obj, "model", Value::String(resolved_model.clone())),
                note,
            );
        }
        let cli_note = if resolved == Resolved::Refused { " (the slot is a cli executor)" } else { "" };
        // param-on-nameless-tier nit (round 2): the first two options both
        // demand a second dispatch attempt before the caller reaches a
        // working door. Naming the resolving verb directly closes that gap
        // in the same message, using the same Option-aware helper the
        // tier-transport denials below use.
        // The clause is appended ONLY when there is a remedy to name — a
        // resolving --kind, or a slot-own transport (herding-exec, or the
        // configured cli command on stdin). A tier that resolves to the
        // session model (ceiling, and anything else that resolves to
        // Resolved::Inherit) has no remedy to name — appending "or" with
        // nothing after it reads as a dangling clause, so the message ends
        // at "you intended." instead, exactly as it did before dod-1.
        let door: Option<String> = match dispatch_kind_for_tier(t) {
            Some(kind) => Some(format!(
                "run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" \
for the {t} tier's own transport"
            )),
            None => match resolved {
                Resolved::Herding => Some(format!(
                    "dispatch prepare has no --kind for the {t} tier yet — run \".bee/bin/bee \
herding run --task-file - --json\" directly with the prompt on stdin"
                )),
                Resolved::Refused => Some(format!(
                    "dispatch prepare has no --kind for the {t} tier yet — run the configured \
command verbatim with the prompt on stdin"
                )),
                _ => None,
            },
        };
        let reason = match &door {
            Some(door) => format!(
                "bee-model-guard: [bee-tier: {t}] resolves to no model name{cli_note}\
, but the dispatch carries model: \"{param}\". The marker would record one \
thing in dispatch.jsonl while the subagent actually runs on the param.\n\
FIX: drop the model param (the marker alone selects the tier), or drop the marker \
and declare the tier whose configured model equals the param you intended; or {door}."
            ),
            None => format!(
                "bee-model-guard: [bee-tier: {t}] resolves to no model name{cli_note}\
, but the dispatch carries model: \"{param}\". The marker would record one \
thing in dispatch.jsonl while the subagent actually runs on the param.\n\
FIX: drop the model param (the marker alone selects the tier), or drop the marker \
and declare the tier whose configured model equals the param you intended."
            ),
        };
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
            let fix = match dispatch_kind_for_tier(t) {
                Some(kind) => format!(
                    "FIX: run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" — it \
reads .bee/config.json for this slot and returns the tool and exact payload to run \
(here, a Bash call running the configured command verbatim with the prompt on \
stdin). Do not attach a model param; the cli command names its own model."
                ),
                None => format!(
                    "FIX: dispatch prepare has no --kind for the {t} tier yet — run the cli \
slot's own transport directly instead: a Bash call running the configured command \
verbatim with the prompt on stdin. Do not attach a model param; the cli command \
names its own model."
                ),
            };
            let reason = format!(
                "bee-model-guard: [bee-tier: {t}] resolves to a cli executor, which an \
in-family Agent/Task subagent cannot be — a cli tier runs as an external process, \
not a spawned subagent.\n\
{fix}"
            );
            return deny(reason, "cli-tier-denied", tier, None, subagent_type);
        }
        if resolved == Resolved::Herding {
            // herding-tier D5: mirror of the cli-tier-denied wording just
            // above — an Agent/Task subagent cannot BE the pane a herding
            // slot spawns.
            let fix = match dispatch_kind_for_tier(t) {
                Some(kind) => format!(
                    "FIX: run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" — it \
reads .bee/config.json for this slot and returns the tool and exact payload to run \
(here, a Bash call running \".bee/bin/bee herding run --task-file - --json\", plus \
--cwd for a granted worktree, with the prompt on stdin). Do not attach a model \
param; the herding worker names its own model."
                ),
                None => format!(
                    "FIX: dispatch prepare has no --kind for the {t} tier yet — run the \
herding slot's own transport directly instead: a Bash call running \".bee/bin/bee \
herding run --task-file - --json\" (plus --cwd for a granted worktree) with the \
prompt on stdin. Do not attach a model param; the herding worker names its own \
model."
                ),
            };
            let reason = format!(
                "bee-model-guard: [bee-tier: {t}] resolves to a herding-executor pane, which \
an in-family Agent/Task subagent cannot be — a herding tier runs one bee-ignorant \
external agent in its own pane, not a spawned subagent.\n\
{fix}"
            );
            return deny(reason, "herding-tier-denied", tier, None, subagent_type);
        }
        return allow("marker", tier, None, subagent_type);
    }

    // (3b) No marker, no param — but the dispatch NAMES a rendered bee agent,
    // and that agent was generated from one tier's configured model. The tier
    // is therefore already declared, in the field that decides which agent
    // actually runs. Reading it here is what turns the single most common
    // refusal ("you named bee-gather, now also say which tier bee-gather is")
    // into no event at all. Purely additive: it can only fire where the guard
    // used to have nothing to go on, so no dispatch that passes today changes
    // its verdict.
    if let Some(t) = subagent_type.as_deref().and_then(tier_for_pinned_type) {
        let resolved = resolve_tier(models, t, "claude");
        if resolved == Resolved::Refused {
            let fix = match dispatch_kind_for_tier(t) {
                Some(kind) => format!(
                    "FIX: run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" (it \
reads .bee/config.json and returns the tool and exact payload — here, a Bash call \
running the configured command verbatim with the prompt on stdin), or name a tier \
whose slot is a model."
                ),
                None => format!(
                    "FIX: dispatch prepare has no --kind for the {t} tier yet — run the cli \
slot's own transport directly instead (a Bash call running the configured command \
verbatim with the prompt on stdin), or name a tier whose slot is a model."
                ),
            };
            let reason = format!(
                "bee-model-guard: subagent_type \"{}\" stands for the {t} tier, which resolves \
to a cli executor — an in-family Agent/Task subagent cannot be an external process.\n\
{fix}",
                subagent_type.as_deref().unwrap_or_default()
            );
            return deny(reason, "cli-tier-denied", Some(t.to_string()), None, subagent_type);
        }
        if resolved == Resolved::Herding {
            // herding-tier D5: same mirror as the marker-only branch above,
            // reached here when the pinned subagent_type itself implies the
            // tier instead of an explicit marker.
            let fix = match dispatch_kind_for_tier(t) {
                Some(kind) => format!(
                    "FIX: run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" (it \
reads .bee/config.json and returns the tool and exact payload — here, a Bash call \
running \".bee/bin/bee herding run --task-file - --json\" with the prompt on stdin), \
or name a tier whose slot is a model."
                ),
                None => format!(
                    "FIX: dispatch prepare has no --kind for the {t} tier yet — run the \
herding slot's own transport directly instead (a Bash call running \".bee/bin/bee \
herding run --task-file - --json\" with the prompt on stdin), or name a tier whose \
slot is a model."
                ),
            };
            let reason = format!(
                "bee-model-guard: subagent_type \"{}\" stands for the {t} tier, which resolves \
to a herding-executor pane — an in-family Agent/Task subagent cannot be an external \
pane worker.\n\
{fix}",
                subagent_type.as_deref().unwrap_or_default()
            );
            return deny(reason, "herding-tier-denied", Some(t.to_string()), None, subagent_type);
        }
        return allow("pinned-type", Some(t.to_string()), None, subagent_type);
    }

    // (4) Bare — deny; resolve the generation slot for the FIX.
    let gen_resolved = resolve_tier(models, "generation", "claude");
    let bare_fix = if let Resolved::Model(gen_model) = &gen_resolved {
        format!(
            "FIX: name one of bee's rendered agents in subagent_type (bee-gather = generation, \
bee-extract = extraction, bee-review = review) — that alone declares the tier. \
Otherwise pass model: \"{gen_model}\" for the generation tier, or open the \
prompt/description with [bee-tier: ceiling] (or generation/extraction/review)."
        )
    } else {
        let slot_kind = match gen_resolved {
            Resolved::Herding => "a herding executor",
            Resolved::Refused => "a cli executor",
            _ => "unconfigured",
        };
        format!(
            "FIX: name one of bee's rendered agents in subagent_type (bee-gather = generation, \
bee-extract = extraction, bee-review = review) — that alone declares the tier. \
Otherwise open the prompt/description with [bee-tier: ceiling] (or another tier: \
generation/extraction/review). The generation tier is {slot_kind}: run \
\".bee/bin/bee dispatch prepare --runtime claude --kind gather --json\" — it reads \
.bee/config.json and returns the tool and exact payload to run (a Bash call, either \
the configured cli command or a herding-pane invocation) rather than a model param."
        )
    };
    let reason = format!(
        "bee-model-guard: every Agent/Task dispatch needs an explicit tier — a rendered \
bee agent type, a `model` param, or a `[bee-tier: <tier>]` marker opening the \
prompt/description (decision 0023). A bare dispatch would silently inherit the most \
expensive session model.\n{bare_fix}"
    );
    deny(reason, "bare-denied", None, None, subagent_type)
}

// ─── dispatch economics (g22-2) ─────────────────────────────────────────────

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

// ─── dispatch-label chokepoint (dispatch-label-chokepoint dlc-2) ───────────
//
// Every Agent/Task/spawn_agent dispatch passes this hook, including one
// typed by hand that never called `prepare` — the exact case a fix inside
// `prepare` can never reach. When the dispatch names a cell (the worker
// prompt/message carries a line "Assigned cell id: <id>") and the label
// field does not already carry that cell's title, the label is rewritten to
// `<id>: <title>` — the same form `prepare` emits for kind=="cell" — and the
// rewrite rides the same repair_stdout announcement every other repair here
// uses.
//
// REPAIR, NEVER REFUSE (ask-guard-autofix D1/D2): a label is not worth
// losing a dispatch over. Every resolution failure — no cell id in the
// prompt/message, no cell record at the active or archived path, an
// unreadable or unparseable record, a missing or blank title — returns
// `None` and the payload passes through untouched and silent; no branch
// here can produce a deny or an error.

/// `<label field>` for Agent/Task is `description`; for codex spawn_agent it
/// is `task_name`. The cell id is read from `prompt` (Agent/Task) or
/// `message` (spawn_agent) — the field that carries the worker's full
/// dispatch text.
fn repair_dispatch_label(root: &Path, is_codex_spawn: bool, tool_input: &Value) -> Option<(Value, String)> {
    let Value::Object(obj) = tool_input else { return None };
    let search_field = if is_codex_spawn { "message" } else { "prompt" };
    let text = obj.get(search_field).and_then(Value::as_str)?;
    let cell_id = extract_assigned_cell_id(text)?;
    let title = read_cell_title(root, &cell_id)?;
    if title.is_empty() {
        return None;
    }
    let prepared = format!("{cell_id}: {title}");
    let label_field = if is_codex_spawn { "task_name" } else { "description" };
    let current = obj.get(label_field).and_then(Value::as_str).unwrap_or("");
    if current.contains(title.as_str()) {
        return None; // already carries the title — byte-identical, nothing to do
    }
    let note = format!("dispatch label rewritten to \"{prepared}\" — it did not name what cell {cell_id} does");
    Some((with_field(obj, label_field, Value::String(prepared)), note))
}

/// The one line every worker prompt/message carries: "Assigned cell id:
/// <id>". First matching line, exact prefix, trimmed. No such line, or a
/// blank id, is a dispatch naming no cell — None, and the caller leaves the
/// label alone.
fn extract_assigned_cell_id(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("Assigned cell id:") {
            let id = rest.trim();
            if !id.is_empty() && cell_id_looks_safe(id) {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// The shape lib/cells.mjs ID_PATTERN enforces — guards the path joins in
/// [`read_cell_title`] against a `..` or path-separator segment riding in on
/// hand-typed text. Anything outside the shape fails resolution silently
/// rather than erroring.
fn cell_id_looks_safe(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// `.bee/cells/<id>.json`, falling back to every `.bee/cells/archive/*/`
/// directory the cells store uses (lib/cells.mjs readCell) — the same shape
/// as the cells store's own read, kept local so this hot-path hook stays
/// native and self-contained. Missing, corrupt, or unparseable all read as
/// absent — no warning: this repair only ever adds a label, never a message
/// about why it could not.
fn read_cell_title(root: &Path, id: &str) -> Option<String> {
    let cells_dir = root.join(".bee").join("cells");
    let read = |file: &Path| -> Option<Value> {
        match read_json(file) {
            ReadJson::Parsed(v) => Some(v),
            ReadJson::Missing | ReadJson::Corrupt => None,
        }
    };
    let title_of = |v: &Value| v.get("title").and_then(Value::as_str).map(str::to_string);
    if let Some(v) = read(&cells_dir.join(format!("{id}.json"))) {
        return title_of(&v);
    }
    let entries = std::fs::read_dir(cells_dir.join("archive")).ok()?;
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        if let Some(v) = read(&entry.path().join(format!("{id}.json"))) {
            return title_of(&v);
        }
    }
    None
}

// ─── audit logs ─────────────────────────────────────────────────────────────

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
        .map(|d| truncate_chars_head(d, 120))
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

    /// (exit code, stderr) — the two the older rows assert on.
    fn run_payload(root: &Path, payload: Value) -> (u8, String) {
        let (code, _stdout, stderr) = run_full(root, payload);
        (code, stderr)
    }

    /// (exit code, stdout, stderr) — for the repair rows, which read the
    /// updatedInput emission.
    fn run_full(root: &Path, payload: Value) -> (u8, String, String) {
        let mut body = payload;
        body["cwd"] = json!(root.to_string_lossy());
        run_inner(&[], &serde_json::to_string(&body).unwrap()).expect("native run")
    }

    /// The hookSpecificOutput of a repair emission.
    fn repair_output(stdout: &str) -> Value {
        serde_json::from_str::<Value>(stdout).expect("repair stdout must be JSON")
    }

    fn last_jsonl(file: PathBuf) -> Option<Value> {
        let text = std::fs::read_to_string(file).ok()?;
        let line = text.lines().filter(|l| !l.trim().is_empty()).last()?;
        serde_json::from_str(line).ok()
    }

    fn dispatch_log(root: &Path) -> PathBuf {
        root.join(".bee").join("logs").join("dispatch.jsonl")
    }

    // opencode-support E4/S4: models.opencode used to be silently dropped by
    // this hook's own normalize_models (only ["claude", "codex"] were parsed)
    // — docs/config-reference.md called that "dead config that never
    // resolves". It is now a real third key, resolved exactly like
    // claude/codex.
    #[test]
    fn opencode_models_block_is_parsed_and_resolved() {
        let raw = json!({
            "opencode": {
                "extraction": "opencode/ling-3.0-tiny-free",
                "generation": "opencode/big-pickle",
                "review": "opencode/nemotron-3-ultra-free"
            }
        });
        let models = normalize_models(Some(&raw));
        assert_eq!(
            resolve_tier(&models, "generation", "opencode"),
            Resolved::Model("opencode/big-pickle".into())
        );
        assert_eq!(
            resolve_tier(&models, "extraction", "opencode"),
            Resolved::Model("opencode/ling-3.0-tiny-free".into())
        );
        assert_eq!(
            resolve_tier(&models, "review", "opencode"),
            Resolved::Model("opencode/nemotron-3-ultra-free".into())
        );
        // Unconfigured (no models.opencode key at all) resolves to Budget on
        // every slot — same no-baked-in-default treatment codex gets.
        let models = normalize_models(None);
        assert_eq!(resolve_tier(&models, "generation", "opencode"), Resolved::Budget);
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
    fn bare_dispatch_under_a_herding_generation_slot_names_a_herding_executor() {
        // D1(d): the bare-denied FIX reads gen_resolved instead of asserting
        // a fixed "cli executor or unconfigured" — a herding-shaped
        // generation slot is named a herding executor, never a cli one.
        let herding = fixture(&json!({"models": {"claude": {
            "extraction": "haiku",
            "generation": {"kind": "herding"},
            "review": "opus"
        }}}));
        let (code, stderr) = run_payload(
            herding.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "implement the widget", "description": "some description"}}),
        );
        assert_eq!(code, 2);
        assert!(stderr.contains("a herding executor"), "{stderr}");
        assert!(!stderr.contains("cli executor"), "{stderr}");
        assert!(stderr.contains("dispatch prepare"), "{stderr}");
        let d = last_jsonl(dispatch_log(herding.path())).unwrap();
        assert_eq!(d["transport"], "bare-denied");
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
        // disagree -> the param is rewritten to the tier's configured model
        let (code, stdout, _) = run_full(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] go", "model": "opus"}}));
        assert_eq!(code, 0);
        let out = repair_output(&stdout);
        assert_eq!(out["hookSpecificOutput"]["updatedInput"]["model"], json!("sonnet"));
        assert_eq!(out["hookSpecificOutput"]["updatedInput"]["prompt"], json!("[bee-tier: generation] go"));
        assert!(out["systemMessage"].as_str().unwrap().contains("sonnet"));
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "param-tier-repaired");
        assert_eq!(d["model"], "sonnet", "the audit records what will actually run");
        // ceiling + param -> deny "drop the model param", and no dangling
        // "or" clause (dod-6: ceiling resolves to Resolved::Inherit, which
        // has no --kind and no slot-own transport to name)
        let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: ceiling] go", "model": "sonnet"}}));
        assert_eq!(code, 2);
        assert!(stderr.contains("drop the model param"));
        assert!(!stderr.contains("has no --kind for the ceiling tier"), "{stderr}");
        assert!(stderr.trim_end().ends_with("you intended."), "{stderr}");
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
        assert!(stderr.contains("dispatch prepare --runtime claude --kind gather") && stderr.contains("stdin"));
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

    /// The generation tier carries two agents — bee-gather reads, bee-build
    /// writes — so "which agent does this tier mean" has no single answer and
    /// the guard stops guessing. It used to answer bee-gather, which made
    /// every execution dispatch land in an agent whose own contract forbids
    /// writing: a cell could never be executed by a dispatched worker.
    #[test]
    fn a_generation_dispatch_names_its_agent_and_bee_build_is_allowed() {
        let fx = fixture(&repo_config());

        // Naming the execution agent is enough on its own — no marker needed.
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "go", "subagent_type": "bee-build"}}));
        assert_eq!(code, 0);

        // general-purpose at the generation tier is refused, and the refusal
        // names both agents and what each one is for.
        let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] go", "subagent_type": "general-purpose"}}));
        assert_ne!(code, 0, "generation must not be repaired to one of two agents");
        assert!(stderr.contains("bee-build"), "{stderr}");
        assert!(stderr.contains("bee-gather"), "{stderr}");

        // The other tiers have exactly one agent each and still repair.
        for (tier, pinned) in [("extraction", "bee-extract"), ("review", "bee-review")] {
            let (code, stdout, _) = run_full(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": format!("[bee-tier: {tier}] go"), "subagent_type": "general-purpose"}}));
            assert_eq!(code, 0, "tier {tier} still repairs");
            assert_eq!(
                repair_output(&stdout)["hookSpecificOutput"]["updatedInput"]["subagent_type"],
                json!(pinned)
            );
        }
    }

    #[test]
    fn pinned_type_rule() {
        let fx = fixture(&repo_config());
        // generation is excluded: it carries two agents and refuses instead
        // of repairing (a_generation_dispatch_names_its_agent_and_bee_build_is_allowed).
        for &(tier, pinned) in PINNED_AGENT_TYPE.iter().filter(|(t, _)| *t != "generation") {
            let (code, stdout, stderr) = run_full(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": format!("[bee-tier: {tier}] go"), "subagent_type": "general-purpose"}}));
            // Repaired, not refused: the tier was stated, so the agent type
            // it implies is the guard's own lookup to perform.
            assert_eq!(code, 0, "tier {tier}: {stderr}");
            let out = repair_output(&stdout);
            assert_eq!(out["hookSpecificOutput"]["updatedInput"]["subagent_type"], json!(pinned));
            // the rest of the request survives the rewrite untouched
            assert_eq!(
                out["hookSpecificOutput"]["updatedInput"]["prompt"],
                json!(format!("[bee-tier: {tier}] go"))
            );
            // no permission verdict rides along (hook-runtime R23)
            assert!(out["hookSpecificOutput"]["permissionDecision"].is_null());
            assert!(out["systemMessage"].as_str().unwrap().contains(pinned));
            let d = last_jsonl(dispatch_log(fx.path())).unwrap();
            assert_eq!(d["transport"], "generic-type-repaired");
            assert_eq!(d["tier"], tier);
            assert_eq!(d["subagent_type"], pinned, "the audit records what will run");
        }
        // a matching param does not rescue general-purpose at generation —
        // the type is still unnamed, and the refusal says so
        let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] go", "model": "sonnet", "subagent_type": "general-purpose"}}));
        assert_ne!(code, 0);
        assert!(stderr.contains("bee-build"), "{stderr}");
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
    fn a_rendered_bee_agent_type_declares_its_own_tier() {
        let fx = fixture(&repo_config());
        // The refusal in the field report: a dispatch that names bee-gather
        // and nothing else. It now carries its tier in the agent type.
        for (tier, pinned) in PINNED_AGENT_TYPE {
            let (code, stdout, stderr) = run_full(fx.path(), json!({"tool_name": "Agent", "tool_input": {"subagent_type": pinned, "description": "map campaign source_type usage", "prompt": "find every caller"}}));
            assert_eq!(code, 0, "{pinned}: {stderr}");
            assert_eq!(stdout, "", "an inferred tier needs no repair");
            let d = last_jsonl(dispatch_log(fx.path())).unwrap();
            assert_eq!(d["transport"], "pinned-type");
            assert_eq!(d["tier"], tier);
            assert_eq!(d["subagent_type"], pinned);
            // the tier's configured model is what the audit says was requested
            assert_eq!(d["requested_model"], json!(match tier {
                "generation" => "sonnet",
                "extraction" => "haiku",
                _ => "opus",
            }));
        }
        // Additive only: an unknown agent type is still bare, still refused.
        let (code, stderr) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"subagent_type": "some-other-agent", "prompt": "go"}}));
        assert_eq!(code, 2);
        assert!(stderr.contains("bee-gather = generation"), "the FIX teaches the new route");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "bare-denied");
        // An explicit param still wins the read — inference never overrides a
        // stated model, so nothing that passes today changes verdict.
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"subagent_type": "bee-gather", "model": "opus"}}));
        assert_eq!(code, 0);
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "model-param");
        assert_eq!(d["model"], "opus");
        assert_eq!(d["tier"], Value::Null);
    }

    #[test]
    fn a_herding_shaped_generation_slot_denies_the_marker_only_path() {
        // herding-tier D5: mirror of cli_slot_and_empty_and_malformed_model_sets'
        // marker-only denial, for `{kind:"herding"}` instead of `{kind:"cli"}`.
        let herding = fixture(&json!({"models": {"claude": {
            "extraction": "haiku",
            "generation": {"kind": "herding"},
            "review": "opus"
        }}}));
        let (code, stderr) = run_payload(
            herding.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] go"}}),
        );
        assert_eq!(code, 2, "a herding slot cannot be an in-family Agent/Task subagent");
        assert!(stderr.contains("herding-executor pane") && stderr.contains("herding run --task-file - --json") && stderr.contains("dispatch prepare"));
        let d = last_jsonl(dispatch_log(herding.path())).unwrap();
        assert_eq!(d["transport"], "herding-tier-denied");
        assert_eq!(d["tier"], "generation");
    }

    #[test]
    fn an_inferred_herding_tier_is_still_denied() {
        // herding-tier D5: mirror of an_inferred_cli_tier_is_still_refused —
        // subagent_type alone implies the tier.
        let herding = fixture(&json!({"models": {"claude": {
            "extraction": "haiku",
            "generation": {"kind": "herding"},
            "review": "opus"
        }}}));
        let (code, stderr) = run_payload(
            herding.path(),
            json!({"tool_name": "Agent", "tool_input": {"subagent_type": "bee-gather", "prompt": "gather"}}),
        );
        assert_eq!(code, 2, "a herding slot cannot be an in-family subagent");
        assert!(stderr.contains("bee-gather") && stderr.contains("herding-executor pane"));
        let d = last_jsonl(dispatch_log(herding.path())).unwrap();
        assert_eq!(d["transport"], "herding-tier-denied");
        assert_eq!(d["tier"], "generation");
    }

    #[test]
    fn a_herding_shaped_extraction_slot_denies_bee_extract_without_a_wrong_kind() {
        // D1(d) round 2: dispatch_kind_for_tier has no --kind that resolves
        // the extraction slot (slot_for_kind in prepare.rs has no extraction
        // arm), so this FIX must not print --kind advisor — that would
        // resolve the advisor slot, never the refused extraction one.
        let herding = fixture(&json!({"models": {"claude": {
            "extraction": {"kind": "herding"},
            "generation": "sonnet",
            "review": "opus"
        }}}));
        let (code, stderr) = run_payload(
            herding.path(),
            json!({"tool_name": "Agent", "tool_input": {"subagent_type": "bee-extract", "prompt": "extract"}}),
        );
        assert_eq!(code, 2, "a herding-shaped extraction slot cannot be an in-family subagent");
        assert!(!stderr.contains("--kind advisor"), "{stderr}");
        assert!(stderr.contains("herding-executor pane") && stderr.contains("herding run --task-file - --json"));
        assert!(stderr.contains("dispatch prepare has no --kind for the extraction tier yet"), "{stderr}");
        let d = last_jsonl(dispatch_log(herding.path())).unwrap();
        assert_eq!(d["transport"], "herding-tier-denied");
        assert_eq!(d["tier"], "extraction");
    }

    // herding-review-slots D1: this hook has no purpose concept of its own
    // — it denies an Agent/Task dispatch by the RESOLVED tier alone, so a
    // herding-shaped review slot was already denied for a reviewer-purpose
    // dispatch before this feature (the same arm the generation/cell tests
    // above pin). This test names that coverage explicitly rather than
    // leaving D1's "reviewer" half unproven by omission.
    #[test]
    fn a_herding_shaped_review_slot_denies_the_reviewer_marker_too() {
        let herding = fixture(&json!({"models": {"claude": {
            "extraction": "haiku",
            "generation": "sonnet",
            "review": {"kind": "herding", "agent": "agy-flash"}
        }}}));
        let (code, stderr) = run_payload(
            herding.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: review] check", "subagent_type": "bee-review"}}),
        );
        assert_eq!(code, 2, "a herding-shaped review slot cannot be an in-family Agent/Task subagent");
        assert!(stderr.contains("herding-executor pane") && stderr.contains("herding run --task-file - --json") && stderr.contains("dispatch prepare"));
        let d = last_jsonl(dispatch_log(herding.path())).unwrap();
        assert_eq!(d["transport"], "herding-tier-denied");
        assert_eq!(d["tier"], "review");
    }

    #[test]
    fn an_inferred_cli_tier_is_still_refused() {
        let cli = fixture(&json!({"models": {"claude": {
            "extraction": "haiku",
            "generation": {"kind": "cli", "command": "codex exec -m gpt-5.5 -s read-only -"},
            "review": "opus"
        }}}));
        let (code, stderr) = run_payload(cli.path(), json!({"tool_name": "Agent", "tool_input": {"subagent_type": "bee-gather", "prompt": "gather"}}));
        assert_eq!(code, 2, "a cli slot cannot be an in-family subagent");
        assert!(stderr.contains("bee-gather") && stderr.contains("cli executor") && stderr.contains("dispatch prepare"));
        let d = last_jsonl(dispatch_log(cli.path())).unwrap();
        assert_eq!(d["transport"], "cli-tier-denied");
        assert_eq!(d["tier"], "generation");
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

    // Rows 11/12/15/16 — the fail-open arms the existing malformed-input test
    // never reaches, because `run_payload` always hands the hook a
    // well-formed JSON object with a cwd.
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
            let (code, _stdout, stderr) = run_inner(&[], &stdin).expect("must decide natively");
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
        let (code, _stdout, stderr) =
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
        // Presence gate (the onboarding marker): a host with no installed
        // harness gets no verdict rather than a wrong one.
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
    fn description_is_truncated_to_120_chars() {
        let fx = fixture(&repo_config());
        let (code, _) = run_payload(fx.path(), json!({"tool_name": "Agent", "tool_input": {"model": "haiku", "description": "z".repeat(300)}}));
        assert_eq!(code, 0);
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["description"].as_str().unwrap().len(), 120);
    }

    // ─── dispatch-label chokepoint (dlc-2) ─────────────────────────────────

    fn write_cell(root: &Path, id: &str, title: &str) {
        std::fs::create_dir_all(root.join(".bee").join("cells")).unwrap();
        std::fs::write(
            root.join(".bee").join("cells").join(format!("{id}.json")),
            json!({"id": id, "title": title}).to_string(),
        )
        .unwrap();
    }

    const TITLE: &str = "Repair a dispatch label at the hook every dispatch passes";

    #[test]
    fn bare_id_label_on_a_cell_dispatch_is_rewritten_to_carry_the_title() {
        let fx = fixture(&repo_config());
        write_cell(fx.path(), "dlc-2", TITLE);
        let prompt = "[bee-tier: generation] Execute cell dlc-2\nAssigned cell id: dlc-2\nFeature: dispatch-label-chokepoint\n";
        let (code, stdout, _) = run_full(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": prompt, "description": "dlc-2"}}),
        );
        assert_eq!(code, 0);
        let out = repair_output(&stdout);
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["description"],
            json!(format!("dlc-2: {TITLE}"))
        );
        // the rest of the request survives the rewrite untouched
        assert_eq!(out["hookSpecificOutput"]["updatedInput"]["prompt"], json!(prompt));
        assert!(out["systemMessage"].as_str().unwrap().contains("dlc-2"));
        assert!(out["hookSpecificOutput"]["permissionDecision"].is_null());
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["description"], json!(format!("dlc-2: {TITLE}")), "the audit records what will run");
    }

    #[test]
    fn already_correct_label_is_left_byte_identical() {
        let fx = fixture(&repo_config());
        write_cell(fx.path(), "dlc-2", TITLE);
        let prompt = "[bee-tier: generation] Execute cell dlc-2\nAssigned cell id: dlc-2\n";
        let (code, stdout, _) = run_full(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": prompt, "description": format!("dlc-2: {TITLE}")}}),
        );
        assert_eq!(code, 0);
        assert_eq!(stdout, "", "already-correct label triggers no repair emission at all");
    }

    #[test]
    fn a_dispatch_naming_no_cell_is_untouched() {
        let fx = fixture(&repo_config());
        write_cell(fx.path(), "dlc-2", TITLE);
        // No "Assigned cell id:" line anywhere in the prompt — the bare
        // "dlc-2" description looks repairable but nothing names a cell.
        let (code, stdout, _) = run_full(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: generation] just go, no cell named here", "description": "dlc-2"}}),
        );
        assert_eq!(code, 0);
        assert_eq!(stdout, "", "no cell named -> nothing to repair");
    }

    #[test]
    fn unreadable_or_missing_cell_record_is_untouched_and_does_not_error() {
        let fx = fixture(&repo_config());
        // Present but corrupt.
        std::fs::create_dir_all(fx.path().join(".bee").join("cells")).unwrap();
        std::fs::write(fx.path().join(".bee").join("cells").join("dlc-corrupt.json"), "{not json").unwrap();
        for (cell_id, description) in [("dlc-corrupt", "dlc-corrupt"), ("dlc-missing", "dlc-missing")] {
            let prompt = format!("[bee-tier: generation] go\nAssigned cell id: {cell_id}\n");
            let (code, stdout, stderr) = run_full(
                fx.path(),
                json!({"tool_name": "Agent", "tool_input": {"prompt": prompt, "description": description}}),
            );
            assert_eq!(code, 0, "{cell_id} must not error");
            assert_eq!(stderr, "");
            assert_eq!(stdout, "", "{cell_id}: unresolved record leaves the payload untouched");
        }
    }

    #[test]
    fn label_repair_never_produces_a_deny_and_composes_with_other_repairs() {
        let fx = fixture(&repo_config());
        write_cell(fx.path(), "dlc-2", TITLE);
        // A bare, unmarked dispatch that also names a resolvable cell still
        // denies for the untouched reason — naming a cell never rescues a
        // dispatch this hook would otherwise refuse.
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "no tier here\nAssigned cell id: dlc-2\n"}}),
        );
        assert_eq!(code, 2);
        assert!(stderr.contains("bee-tier"));

        // Composes with an existing repair: subagent_type "general-purpose"
        // at [bee-tier: extraction] is rewritten to "bee-extract" AND the
        // bare-id description is rewritten to carry the cell's title, both
        // in the one updatedInput.
        let prompt = "[bee-tier: extraction] Execute cell dlc-2\nAssigned cell id: dlc-2\n";
        let (code, stdout, _) = run_full(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": prompt, "description": "dlc-2", "subagent_type": "general-purpose"}}),
        );
        assert_eq!(code, 0);
        let out = repair_output(&stdout);
        assert_eq!(out["hookSpecificOutput"]["updatedInput"]["subagent_type"], json!("bee-extract"));
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["description"],
            json!(format!("dlc-2: {TITLE}"))
        );
    }

    #[test]
    fn codex_spawn_label_repair_targets_task_name_from_message() {
        let fx = fixture(&repo_config());
        write_cell(fx.path(), "dlc-2", TITLE);
        let message = "[bee-tier: generation] gather\nAssigned cell id: dlc-2\n";
        let (code, stdout, _) = run_full(
            fx.path(),
            json!({"tool_name": "spawn_agent", "tool_input": {"agent_type": "worker", "message": message, "task_name": "dlc-2"}}),
        );
        assert_eq!(code, 0);
        let out = repair_output(&stdout);
        assert_eq!(
            out["hookSpecificOutput"]["updatedInput"]["task_name"],
            json!(format!("dlc-2: {TITLE}"))
        );
        assert_eq!(out["hookSpecificOutput"]["updatedInput"]["message"], json!(message));
        // A codex dispatch with no cell id in `message` is untouched, even
        // when `prompt` (not the read field) carries one.
        let (code, stdout, _) = run_full(
            fx.path(),
            json!({"tool_name": "spawn_agent", "tool_input": {"agent_type": "worker", "message": "[bee-tier: generation] gather", "prompt": "Assigned cell id: dlc-2\n"}}),
        );
        assert_eq!(code, 0);
        assert_eq!(stdout, "");
    }
}
