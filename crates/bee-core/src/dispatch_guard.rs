//! dispatch_guard — Rust port of `.bee/bin/lib/dispatch-guard.mjs`'s
//! `evaluateDispatch` (rust-port-10, CONTEXT.md D2/D7): the PURE decision
//! core for bee's dispatch-transport enforcement (decision 0023). Zero I/O
//! — no fs writes, no stderr, no logging; every decision is a pure function
//! of `(tool_name, tool_input, root)`, matching the mjs source's own
//! contract exactly. Callers (the `queen-bee` `model-guard` hook) own every
//! side effect.
//!
//! `.bee/bin/lib/dispatch-guard.mjs` is FROZEN for the duration of the
//! rust-port feature (D1) — this module is conformance-checked against a
//! sha256-verified copy of it plus `bee-model-guard.mjs`
//! (`crates/queen-bee/tests/modelguard_conformance.rs`), never edited to
//! "improve" on it. Every deny reason below is copied VERBATIM from the mjs
//! source so a deny's stderr is byte-identical across runtimes — do not
//! reword them even for clarity.
//!
//! Seven deny classes (the cell's must-have coverage), plus the allow
//! paths:
//!   - `codex-spawn-unmarked`   — evaluate_codex_spawn, unmarked message
//!   - `generic-type-denied`    — evaluate_claude_dispatch (0), pinned-type rule
//!   - `param-tier-mismatch`    — evaluate_claude_dispatch (1)
//!   - `param-on-nameless-tier` — evaluate_claude_dispatch (1)
//!   - `param-not-configured`   — evaluate_claude_dispatch (2)
//!   - `cli-tier-denied`        — evaluate_claude_dispatch (3)
//!   - `bare-denied`            — evaluate_claude_dispatch (4)
//! Allow transports: `codex-spawn-marker`, `model-param`, `marker`.

use std::path::Path;

use serde_json::Value;

use crate::config::{self, ResolvePurpose, ResolvedTier};

/// mjs `CODEX_SPAWN_TOOL` — the Codex-native collaboration spawn tool name
/// observed through PreToolUse (codex-native-runtime-v2 D4).
pub const CODEX_SPAWN_TOOL: &str = "spawn_agent";
/// mjs `DISPATCH_TOOLS` — the Claude in-family dispatch tool set.
pub const DISPATCH_TOOLS: [&str; 2] = ["Agent", "Task"];

/// The tiers an anchored `[bee-tier: <tier>]` marker may name on the Claude
/// branch (mjs `ANCHORED_TIER_MARKER_RE`) — deliberately excludes `advisor`
/// (a native-transport-only slot label, R1 "claude branch regex untouched").
const CLAUDE_MARKER_TIERS: [&str; 4] = ["ceiling", "generation", "extraction", "review"];
/// The Codex-branch marker vocabulary (mjs `ANCHORED_CODEX_TIER_MARKER_RE`)
/// — additionally recognizes `advisor`.
const CODEX_MARKER_TIERS: [&str; 5] = ["ceiling", "generation", "extraction", "review", "advisor"];

/// mjs `PINNED_AGENT_TYPE` (W3, AO5/AO10/AO11) — each model-backed tier's
/// rendered bee agent type; `ceiling` deliberately has none (it IS the
/// session model).
fn pinned_agent_type(tier: &str) -> Option<&'static str> {
    match tier {
        "generation" => Some("bee-gather"),
        "extraction" => Some("bee-extract"),
        "review" => Some("bee-review"),
        _ => None,
    }
}

/// Allow or deny, mirroring mjs `evaluateDispatch`'s returned `decision`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

/// The full verdict `evaluateDispatch` returns. `transport === None` is the
/// mjs source's "no opinion" sentinel — the caller must never log a
/// dispatch line for it (wrong tool, or a malformed/absent `tool_input`
/// that never reached a real branch). Every other `transport` value — allow
/// or deny — is a real evaluated dispatch the caller logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchResult {
    pub decision: Decision,
    pub transport: Option<String>,
    pub reason: Option<String>,
    pub tier: Option<String>,
    pub model: Option<String>,
    pub subagent_type: Option<String>,
}

fn no_opinion() -> DispatchResult {
    DispatchResult { decision: Decision::Allow, transport: None, reason: None, tier: None, model: None, subagent_type: None }
}

fn allow_result(transport: &str, tier: Option<String>, model: Option<String>, subagent_type: Option<String>) -> DispatchResult {
    DispatchResult { decision: Decision::Allow, transport: Some(transport.to_string()), reason: None, tier, model, subagent_type }
}

fn deny_result(
    reason: String,
    transport: &str,
    tier: Option<String>,
    model: Option<String>,
    subagent_type: Option<String>,
) -> DispatchResult {
    DispatchResult { decision: Decision::Deny, transport: Some(transport.to_string()), reason: Some(reason), tier, model, subagent_type }
}

/// Anchored marker match: the FIRST non-whitespace content of `text` must
/// be `[bee-tier: <tier>]` (case-insensitive on the literal characters and
/// the tier word), immediately closed by `]` — a marker occurring anywhere
/// else in `text` never counts (P1-1). Returns the lowercased tier name
/// when `text` opens with a marker naming one of `tiers`.
fn starts_with_tier_marker(text: &str, tiers: &[&str]) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    const PREFIX: &str = "[bee-tier:";
    let prefix_chars: Vec<char> = PREFIX.chars().collect();
    if chars.len() < i + prefix_chars.len() {
        return None;
    }
    for (k, pc) in prefix_chars.iter().enumerate() {
        if !chars[i + k].eq_ignore_ascii_case(pc) {
            return None;
        }
    }
    i += prefix_chars.len();
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    let start = i;
    while i < chars.len() && chars[i].is_ascii_alphabetic() {
        i += 1;
    }
    if start == i || i >= chars.len() || chars[i] != ']' {
        return None;
    }
    let word: String = chars[start..i].iter().collect::<String>().to_lowercase();
    if tiers.contains(&word.as_str()) {
        Some(word)
    } else {
        None
    }
}

/// mjs `markerTier(toolInput)`: description checked before prompt.
fn marker_tier(tool_input: &Value) -> Option<String> {
    tool_input
        .get("description")
        .and_then(Value::as_str)
        .and_then(|d| starts_with_tier_marker(d, &CLAUDE_MARKER_TIERS))
        .or_else(|| tool_input.get("prompt").and_then(Value::as_str).and_then(|p| starts_with_tier_marker(p, &CLAUDE_MARKER_TIERS)))
}

/// Port of `evaluateCodexSpawn(toolInput)` — the Codex-native collaboration
/// spawn branch. Deny class: `codex-spawn-unmarked`. Allow transport:
/// `codex-spawn-marker`.
fn evaluate_codex_spawn(tool_input: Option<&Value>) -> DispatchResult {
    let Some(input) = tool_input.filter(|v| v.is_object()) else {
        return no_opinion();
    };
    let message = input.get("message").and_then(Value::as_str);
    let Some(message) = message.filter(|m| !m.is_empty()) else {
        return no_opinion();
    };
    if let Some(tier) = starts_with_tier_marker(message, &CODEX_MARKER_TIERS) {
        return allow_result("codex-spawn-marker", Some(tier), None, None);
    }
    let reason = "bee-model-guard: every Codex spawn_agent needs an explicit tier — its \
message must OPEN with a [bee-tier: <tier>] marker (decision 0023 \
parity, codex-native-runtime-v2 D4, i54-closeout D1). A marker anywhere but the \
start of the message does not count, and a marker in any other field is ignored; \
without one the spawned worker silently inherits the session model.\n\
FIX: begin the spawn message with the marker, e.g. \
\"[bee-tier: generation] <task>\" (tiers: ceiling/generation/extraction/review/advisor)."
        .to_string();
    deny_result(reason, "codex-spawn-unmarked", None, None, None)
}

/// Port of `evaluateClaudeDispatch(rawToolInput, root)` — every branch of
/// the original `bee-model-guard.mjs` `main()`, unchanged. Deny classes:
/// `generic-type-denied`, `param-tier-mismatch`, `param-on-nameless-tier`,
/// `param-not-configured`, `cli-tier-denied`, `bare-denied`. Allow
/// transports: `model-param`, `marker`.
fn evaluate_claude_dispatch(tool_input: Option<&Value>, root: &Path) -> DispatchResult {
    let Some(tool_input) = tool_input.filter(|v| v.is_object()) else {
        return no_opinion();
    };

    let model_param = tool_input.get("model").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let tier = marker_tier(tool_input);
    let subagent_type = tool_input.get("subagent_type").and_then(Value::as_str).map(str::to_string);

    // (0) Pinned-type rule (W3, AO5/AO10/AO11) — fires BEFORE every allow
    // branch below.
    if let Some(t) = &tier {
        if t != "ceiling" && subagent_type.as_deref() == Some("general-purpose") {
            let pinned = pinned_agent_type(t).unwrap_or("");
            let reason = format!(
                "bee-model-guard: [bee-tier: {t}] must spawn its pinned agent type, not subagent_type: \"general-purpose\" — general-purpose carries no tier identity and would run under whatever runtime default is in effect, not the rendered bee agent for this tier (AO5/AO10).\nFIX: set subagent_type: \"{pinned}\" (bee's rendered agent for the {t} tier), or use \"Explore\" for a read-only gather that does not need the rendered agent."
            );
            return deny_result(reason, "generic-type-denied", tier.clone(), model_param.clone(), subagent_type.clone());
        }
    }

    // (1) Marker + model param — AO5 strict equality.
    if let (Some(t), Some(m)) = (&tier, &model_param) {
        let resolved = config::resolve_tier(root, t, "claude", ResolvePurpose::Cell);
        if let ResolvedTier::Model { model: resolved_model, .. } = &resolved {
            if m == resolved_model {
                return allow_result("model-param", Some(t.clone()), Some(m.clone()), subagent_type.clone());
            }
            let reason = format!(
                "bee-model-guard: [bee-tier: {t}] resolves to model \"{resolved_model}\", but the dispatch carries model: \"{m}\" — the tier label and the param disagree, so the dispatch would run on the param while the audit records the tier (AO5: config is the authority, the model does not get a vote).\nFIX: set model: \"{resolved_model}\" to match the {t} tier, or drop the marker and declare the tier whose configured model is the one you want."
            );
            return deny_result(reason, "param-tier-mismatch", Some(t.clone()), Some(m.clone()), subagent_type.clone());
        }
        let refused_suffix = if matches!(resolved, ResolvedTier::Refused { .. }) { " (the slot is a cli executor)" } else { "" };
        let reason = format!(
            "bee-model-guard: [bee-tier: {t}] resolves to no model name{refused_suffix}, but the dispatch carries model: \"{m}\". The marker would record one thing in dispatch.jsonl while the subagent actually runs on the param.\nFIX: drop the model param (the marker alone selects the tier), or drop the marker and declare the tier whose configured model equals the param you intended."
        );
        return deny_result(reason, "param-on-nameless-tier", Some(t.clone()), Some(m.clone()), subagent_type.clone());
    }

    // (2) Model param, no marker — B5 membership against configured tier slots.
    if let Some(m) = &model_param {
        let member_set = configured_model_set(root);
        if member_set.is_empty() || member_set.iter().any(|c| c == m) {
            // Empty set = unconfigured repo -> fail-open allow (today's behavior).
            return allow_result("model-param", None, Some(m.clone()), subagent_type.clone());
        }
        let configured = member_set.join(", ");
        let reason = format!(
            "bee-model-guard: model: \"{m}\" is not a model configured for any claude tier — a param outside config selects an unaudited model and, for an up-dispatch, hides ceiling scarcity (AO5/B5: config is the sole authority; there is no hardcoded allowlist).\nFIX: use one of the configured models ({configured}); or, for a session-model dispatch, add [bee-tier: ceiling] (ceiling = the session model) to the prompt/description; or add this model to a configured tier slot in .bee/config.json."
        );
        return deny_result(reason, "param-not-configured", None, Some(m.clone()), subagent_type.clone());
    }

    // (3) Marker, no param — B4(1)/W10.
    if let Some(t) = &tier {
        let resolved = config::resolve_tier(root, t, "claude", ResolvePurpose::Cell);
        if matches!(resolved, ResolvedTier::Refused { .. }) {
            // A cli-shaped slot: an in-family Agent/Task subagent cannot BE
            // the external CLI (it runs as its own process, not a spawned
            // subagent).
            let reason = format!(
                "bee-model-guard: [bee-tier: {t}] resolves to a cli executor, which an in-family Agent/Task subagent cannot be — a cli tier runs as an external process, not a spawned subagent.\nFIX: dispatch it through the external-executor gather path — a Bash call running the configured command verbatim with the prompt on stdin (resolveTier(root, slot, runtime, {{for:'gather'}}) returns {{type:'cli', command}}). Do not attach a model param; the cli command names its own model."
            );
            return deny_result(reason, "cli-tier-denied", Some(t.clone()), None, subagent_type.clone());
        }
        // model / budget / inherit / native -> allow (today's behavior, resolution-backed).
        return allow_result("marker", Some(t.clone()), None, subagent_type.clone());
    }

    // (4) Bare — deny (today's behavior), but resolve the generation slot for
    // the FIX so we never tell the agent to pass a model that does not exist.
    let gen_resolved = config::resolve_tier(root, "generation", "claude", ResolvePurpose::Cell);
    let bare_fix = if let ResolvedTier::Model { model, .. } = &gen_resolved {
        format!(
            "FIX: pass model: \"{model}\" for the generation tier, or add [bee-tier: ceiling] (or another tier: generation/extraction/review) to the prompt/description."
        )
    } else {
        "FIX: add [bee-tier: ceiling] (or another tier: generation/extraction/review) to the prompt/description; the generation tier is a cli executor or unconfigured, so run it through the external-executor gather path (a Bash call with the command verbatim and the prompt on stdin) rather than a model param."
            .to_string()
    };
    let reason = format!(
        "bee-model-guard: every Agent/Task dispatch needs an explicit tier — a `model` param or a `[bee-tier: <tier>]` marker in the prompt/description (decision 0023). A bare dispatch would silently inherit the most expensive session model.\n{bare_fix}"
    );
    deny_result(reason, "bare-denied", None, None, subagent_type.clone())
}

/// mjs `configuredModelSet(root)`: the set of model NAMES resolvable from
/// the claude runtime's configured tier slots, folded together with the
/// advisor slot's model (cnt-7, advisor-digest R2) — config is the sole
/// membership authority for a bare `model` param (B5), never a hardcoded
/// allowlist. Returned SORTED (mirrors `[...memberSet].sort()`); an empty
/// result means the repo configures no model tier, and the caller
/// fail-opens.
fn configured_model_set(root: &Path) -> Vec<String> {
    let mut models: Vec<String> = Vec::new();
    for slot in config::CONFIGURABLE_SLOTS {
        if let Some(m) = config::model_for_tier(root, slot, "claude") {
            if !models.contains(&m) {
                models.push(m);
            }
        }
    }
    if let Some(ResolvedTier::Model { model, .. }) = config::resolve_advisor(root, "claude") {
        if !models.contains(&model) {
            models.push(model);
        }
    }
    models.sort();
    models
}

/// `evaluateDispatch(toolName, toolInput, root)` — the single decision
/// function the guard hook calls. `tool_input` is exactly what the hook
/// would see as `payload.tool_input` (for Codex: the object carrying
/// `message` directly, not a further-wrapped envelope); `None` when the
/// field is absent — matching mjs `undefined`.
pub fn evaluate_dispatch(tool_name: &str, tool_input: Option<&Value>, root: &Path) -> DispatchResult {
    if tool_name == CODEX_SPAWN_TOOL {
        return evaluate_codex_spawn(tool_input);
    }
    if DISPATCH_TOOLS.contains(&tool_name) {
        return evaluate_claude_dispatch(tool_input, root);
    }
    no_opinion()
}

// ─── Dispatch economics (g22-2, GH #22 P1-6, D3 + advisor R3) ──────────────

/// The dispatch-economics record `deriveEconomics` computes — the honest
/// channel/logical/requested/effective split appended to every
/// `.bee/logs/dispatch.jsonl` line (allowed or denied).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Economics {
    pub logical_tier: Option<String>,
    pub requested_model: Option<String>,
    pub effective_model: Option<String>,
    pub effective_model_status: String,
    pub channel: String,
    pub enforcement: String,
}

/// Port of `deriveEconomics({channel, tier, paramModel, resolved,
/// nativeConfirmed})` — a PURE function, zero I/O, mapping already-known
/// facts to the audit-line shape. `resolved` is the tier's already-resolved
/// [`ResolvedTier`] (or `None` when no tier was declared); `native_confirmed`
/// is always `false` from the model-guard hook today (no native-transport
/// confirmation probe wired yet — matches mjs's implicit default).
pub fn derive_economics(channel: &str, tier: Option<&str>, param_model: Option<&str>, resolved: Option<&ResolvedTier>, native_confirmed: bool) -> Economics {
    let is_native_confirmed = channel == "codex-native" && matches!(resolved, Some(ResolvedTier::Native { .. })) && native_confirmed;
    let resolved_model: Option<String> = match resolved {
        Some(ResolvedTier::Model { model, .. }) | Some(ResolvedTier::Native { model, .. }) => Some(model.clone()),
        _ => None,
    };

    let enforcement = if channel == "cli-exec" {
        "cli-command".to_string()
    } else if is_native_confirmed {
        "native-model-param".to_string()
    } else if channel == "codex-native" {
        "prompt-budget".to_string()
    } else if param_model.is_some() {
        "model-param".to_string()
    } else {
        "prompt-budget".to_string()
    };

    let mut effective_model: Option<String> = None;
    let effective_model_status = if is_native_confirmed {
        "native-requested".to_string()
    } else if channel == "codex-native" {
        "inherited-or-unknown".to_string()
    } else if channel == "cli-exec" {
        "unverified".to_string()
    } else if let Some(pm) = param_model {
        effective_model = Some(pm.to_string());
        "pinned".to_string()
    } else {
        "unverified".to_string()
    };

    let requested_model = if channel == "cli-exec" { None } else { param_model.map(str::to_string).or(resolved_model) };

    Economics {
        logical_tier: tier.map(str::to_string),
        requested_model,
        effective_model,
        effective_model_status,
        channel: channel.to_string(),
        enforcement,
    }
}

/// mjs `PINNED_MODEL_STATUS` — exported so any future judge-style consumer
/// reuses the same vocabulary value rather than a second hand-rolled
/// literal that could drift from [`derive_economics`]'s actual output.
pub const PINNED_MODEL_STATUS: &str = "pinned";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_config(root: &Path, models: Value) {
        let path = root.join(".bee").join("config.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_string(&json!({ "models": models })).unwrap()).unwrap();
    }

    #[test]
    fn wrong_tool_is_no_opinion() {
        let dir = tempfile::tempdir().unwrap();
        let result = evaluate_dispatch("Read", Some(&json!({})), dir.path());
        assert_eq!(result.transport, None);
        assert_eq!(result.decision, Decision::Allow);
    }

    #[test]
    fn bare_claude_dispatch_denies() {
        let dir = tempfile::tempdir().unwrap();
        let result = evaluate_dispatch("Agent", Some(&json!({})), dir.path());
        assert_eq!(result.decision, Decision::Deny);
        assert_eq!(result.transport.as_deref(), Some("bare-denied"));
        assert!(result.reason.unwrap().contains("needs an explicit tier"));
    }

    #[test]
    fn marker_allows_without_param() {
        let dir = tempfile::tempdir().unwrap();
        let result = evaluate_dispatch("Agent", Some(&json!({"description": "[bee-tier: generation] do a thing", "subagent_type": "bee-gather"})), dir.path());
        assert_eq!(result.decision, Decision::Allow);
        assert_eq!(result.transport.as_deref(), Some("marker"));
        assert_eq!(result.tier.as_deref(), Some("generation"));
    }

    #[test]
    fn generic_type_denied_fires_for_pinned_tier_on_general_purpose() {
        let dir = tempfile::tempdir().unwrap();
        let result = evaluate_dispatch(
            "Agent",
            Some(&json!({"description": "[bee-tier: generation] do a thing", "subagent_type": "general-purpose"})),
            dir.path(),
        );
        assert_eq!(result.decision, Decision::Deny);
        assert_eq!(result.transport.as_deref(), Some("generic-type-denied"));
    }

    #[test]
    fn param_tier_mismatch_and_match() {
        let dir = tempfile::tempdir().unwrap();
        write_config(dir.path(), json!({ "claude": { "generation": "sonnet" } }));
        let mismatch = evaluate_dispatch(
            "Agent",
            Some(&json!({"description": "[bee-tier: generation] x", "model": "haiku", "subagent_type": "bee-gather"})),
            dir.path(),
        );
        assert_eq!(mismatch.transport.as_deref(), Some("param-tier-mismatch"));
        let matched = evaluate_dispatch(
            "Agent",
            Some(&json!({"description": "[bee-tier: generation] x", "model": "sonnet", "subagent_type": "bee-gather"})),
            dir.path(),
        );
        assert_eq!(matched.decision, Decision::Allow);
        assert_eq!(matched.transport.as_deref(), Some("model-param"));
    }

    #[test]
    fn codex_spawn_unmarked_denies_and_marked_allows() {
        let dir = tempfile::tempdir().unwrap();
        let unmarked = evaluate_dispatch("spawn_agent", Some(&json!({"message": "do a thing"})), dir.path());
        assert_eq!(unmarked.transport.as_deref(), Some("codex-spawn-unmarked"));
        let marked = evaluate_dispatch("spawn_agent", Some(&json!({"message": "[bee-tier: generation] do a thing"})), dir.path());
        assert_eq!(marked.decision, Decision::Allow);
        assert_eq!(marked.transport.as_deref(), Some("codex-spawn-marker"));
    }

    #[test]
    fn cli_tier_denied_for_marker_only_cli_slot() {
        let dir = tempfile::tempdir().unwrap();
        write_config(dir.path(), json!({ "claude": { "review": { "kind": "cli", "command": "codex exec -" } } }));
        let result = evaluate_dispatch("Agent", Some(&json!({"description": "[bee-tier: review] x", "subagent_type": "bee-review"})), dir.path());
        assert_eq!(result.transport.as_deref(), Some("cli-tier-denied"));
    }

    #[test]
    fn param_not_configured_lists_sorted_configured_models() {
        let dir = tempfile::tempdir().unwrap();
        write_config(dir.path(), json!({ "claude": { "extraction": "haiku", "generation": "sonnet", "review": "opus" } }));
        let result = evaluate_dispatch("Agent", Some(&json!({"model": "gpt-5"})), dir.path());
        assert_eq!(result.transport.as_deref(), Some("param-not-configured"));
        let reason = result.reason.unwrap();
        assert!(reason.contains("haiku, opus, sonnet"), "expected sorted configured list, got: {reason}");
    }
}
