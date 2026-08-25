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
use crate::verbs::drivers::{normalize_models, resolve_tier, Resolved};
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
        evaluate_codex_spawn(&tool_input, &models)
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

// ─── anchored role marker (ANCHORED_TIER_MARKER_RE / _CODEX_) ─────────────
// /^\s*\[bee-tier:\s*(<role>)\]/i
//
// model-role-split D2 (store 06e49368): the capture group used to be a
// hand-maintained alternation of legal names, kept TWICE — CLAUDE_TIERS with
// four entries, CODEX_TIERS with five — and the two had already drifted with
// nothing intending it (`advisor` was a name a Codex spawn could open with
// and an Agent dispatch could not). Both lists are retired. The parser reads
// whatever NAME the marker carries, and legality is one question asked in one
// place: is this role configured for this runtime.

/// The role name a `[bee-tier: <name>]` marker opens with, exactly as
/// written. Parsing decides SHAPE only — anchored at the start, one
/// whitespace-free token, closed by `]` — never legality: under an open role
/// set there is no closed list to decide legality against.
fn marker_role_name(value: &Value) -> Option<String> {
    let text = value.as_str()?;
    let rest = text.trim_start_matches(is_js_ws);
    let rest = strip_prefix_ascii_ci(rest, "[bee-tier:")?;
    let rest = rest.trim_start_matches(is_js_ws);
    let (name, _) = rest.split_once(']')?;
    // The same strictness the old alternation had: the name ran up to `]`
    // with nothing between. A candidate carrying whitespace or a second `[`
    // is prose that happens to open with the marker text, not a role name.
    if name.is_empty() || name.contains(is_js_ws) || name.contains('[') {
        return None;
    }
    Some(name.to_string())
}

fn strip_prefix_ascii_ci<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if text.len() < prefix.len() {
        return None;
    }
    let (head, tail) = text.split_at_checked(prefix.len())?;
    head.eq_ignore_ascii_case(prefix).then_some(tail)
}

/// A well-formed marker, classified against what this runtime can resolve.
enum Marker {
    /// Names a role bee can resolve, in the spelling the config carries.
    Role(String),
    /// Well-formed, but names a role nothing configures — the loud case.
    Unconfigured(String),
}

/// Every role name a dispatch on this runtime may legally declare.
///
/// The derivation itself lives in `verbs::drivers` (T012a, store 8ff6e79e):
/// `bee dispatch prepare --role` asks the SAME question at the door that this
/// hook asks at the guard, and two copies of "is this role legal" is the
/// defect this whole feature exists to remove. One home, two callers.
fn known_roles(models: &Map<String, Value>, runtime: &str) -> BTreeSet<String> {
    crate::verbs::drivers::known_roles(models, runtime)
}

/// The configured roles as one FIX-line fragment.
fn role_list(models: &Map<String, Value>, runtime: &str) -> String {
    crate::verbs::drivers::role_list(models, runtime)
}

/// Case-insensitive, exactly as the old alternation matched — `[BEE-TIER:
/// Generation]` declares the `generation` role — and the answer is the
/// CONFIG's own spelling, so every downstream read (the audit line,
/// `resolve_role`, the FIX text) gets a key it can look up.
fn classify_marker(name: &str, models: &Map<String, Value>, runtime: &str) -> Marker {
    match known_roles(models, runtime).iter().find(|k| k.eq_ignore_ascii_case(name)) {
        Some(canonical) => Marker::Role(canonical.clone()),
        None => Marker::Unconfigured(name.to_string()),
    }
}

fn marker_of(value: &Value, models: &Map<String, Value>, runtime: &str) -> Option<Marker> {
    marker_role_name(value).map(|name| classify_marker(&name, models, runtime))
}

/// The claude dispatch's marker: description FIRST, then prompt. A marker in
/// `description` wins even when it names nothing configured — it is a role
/// declaration either way, and reading past it to `prompt` would let a typo
/// in the field the host displays pass unremarked.
fn marker_tier(tool_input: &Map<String, Value>, models: &Map<String, Value>) -> Option<Marker> {
    tool_input
        .get("description")
        .and_then(|d| marker_of(d, models, "claude"))
        .or_else(|| tool_input.get("prompt").and_then(|p| marker_of(p, models, "claude")))
}

/// The refusal a marker naming a role nothing configures earns. Never a
/// silent fall-through: a name bee cannot resolve would run the subagent on
/// the session model while the audit line recorded a role that selects no
/// model at all (model-role-split D2, store 06e49368 — "a name nothing
/// configures is warned, never silently accepted").
fn unconfigured_role_reason(name: &str, models: &Map<String, Value>, runtime: &str) -> String {
    let roles = role_list(models, runtime);
    format!(
        "bee-model-guard: [bee-tier: {name}] names a role nothing configures — \
models.{runtime} in .bee/config.json carries no \"{name}\" entry, so the dispatch would \
silently inherit the session model while dispatch.jsonl recorded a role that selects no \
model.\n\
FIX: open with a configured role ({roles}), or configure this one — add \
\"{name}\": \"<model>\" to models.{runtime} in .bee/config.json. Any role name you \
configure is legal; bee holds no fixed list."
    )
}

// ─── model config (the ONE parser lives in verbs::drivers) ────────────────
//
// model-role-split D1 (store cd72ec97): this hook used to carry a SECOND
// implementation of the `models.<runtime>` shape — its own Slot/Slots/Models
// normalize plus a private resolve_tier/resolve_advisor. The two copies had
// already drifted (four tier names against five) with nothing intending it,
// so `verbs::drivers` is now the single parser and this hook calls it. No
// behavior moved with the deletion; see GUARD_PURPOSE for the one place the
// two parsers were not interchangeable.

/// The dispatch purpose every resolution in this hook asks with.
///
/// The surviving parser is purpose-parameterized (`purpose_is_gather`, which
/// is `kind != "cell"`); the deleted one was purpose-BLIND and refused every
/// `{kind:"cli"}` slot unconditionally. This hook's whole surface is
/// Agent/Task/spawn_agent — an in-family subagent, which can no more BE an
/// external cli process than it can be a herding pane — so "cell", the one
/// non-gather purpose, is the deliberate choice at every call site here: it
/// is exactly the purpose under which a cli slot resolves to
/// `Resolved::Refused`, reproducing the deleted parser's blanket refusal.
/// Widening any call site to a gather purpose would hand an Agent/Task
/// dispatch a `Resolved::Cli` it has no transport for.
const GUARD_PURPOSE: &str = "cell";

/// The model name a resolved slot names, if it names one — the accessor the
/// deleted private enum carried as `Resolved::model_name`. Native counts
/// here (it carries an equally readable model string) exactly as it did
/// before; `configured_model_set` deliberately does NOT use this, because
/// membership admits `Resolved::Model` only.
fn resolved_model_name(resolved: &Resolved) -> Option<&str> {
    match resolved {
        Resolved::Model { model, .. } | Resolved::Native { model, .. } => Some(model),
        _ => None,
    }
}

/// How many roles the dispatch door prints before it starts counting.
///
/// The door block is injected into EVERY session and re-injected after every
/// compaction, so its length is a real, repeated cost — publishing a long
/// list of names would be worse than the fixed tier list it replaced. Six is
/// bee's own slots plus room for a couple of the operator's; past that the
/// line says how many more there are and `bee dispatch prepare --role <name>`
/// answers for any of them by name.
const DOOR_ROLES_SHOWN: usize = 6;

/// One resolved role, as the door publishes it — or `None` when the role
/// selects no model at all (`Resolved::Budget`: a name the table does not
/// carry, or a slot the config explicitly turned off), which the door drops
/// rather than printing a name with nothing behind it.
///
/// **`effort` is NOT rendered, and that is the point.** model-role-split
/// records `effort` as a known NON-delivery (plan S10), so printing it here
/// was the door stating a fact no dispatch carries — the same silent-lie
/// shape this feature exists to remove. Three separate facts, because the two
/// runtimes fail for DIFFERENT reasons:
///
/// * `models.<runtime>.<role>` accepts `{model, effort}` and
///   `normalize_tier_value` keeps the value, so it does reach
///   `Resolved::Model`. Config and parsing are not the gap.
/// * On CLAUDE it dies at the door: every `Resolved::Model` site in
///   `verbs/drivers/prepare.rs` destructures `{ model, .. }`, and the Agent
///   tool takes no effort parameter to carry it even if they did not. That
///   half is a harness limit, not a bee gap.
/// * On CODEX it dies for a different reason, and this one IS bee's own: only
///   the `native` transport arm emits `reasoning_effort`. A `Resolved::Model`
///   on codex falls into the `spawn_agent` arm, which emits neither `model`
///   nor `reasoning_effort` — on the one runtime that demonstrably accepts
///   it. The claude harness explanation does NOT cover this half, and anyone
///   who reads "harness limit" and takes the whole thing as closed is reading
///   past a live gap.
///
/// `Resolved::Native` is the one place effort IS delivered; it is not
/// rendered here either, because the native arm's own `reasoning_effort` is
/// what speaks for it and this line publishes the model, not the transport.
fn render_role(resolved: &Resolved) -> Option<String> {
    Some(match resolved {
        Resolved::Model { model, .. } => model.clone(),
        Resolved::Herding { agent, fallback } => {
            let mut s = match agent {
                Some(a) => format!("herding ({a})"),
                None => "herding".to_string(),
            };
            if let Some(fb) = fallback {
                s.push_str(&format!(" fallback={fb}"));
            }
            s
        }
        Resolved::Cli { .. } | Resolved::Refused { .. } => "cli".to_string(),
        Resolved::Native { model, .. } => format!("native:{model}"),
        Resolved::Inherit => "session default".to_string(),
        Resolved::Budget => return None,
    })
}

/// The roles this host actually configures, each resolved to what it selects.
///
/// model-role-split D2 (store 06e49368): this was `tier_slot_display` and it
/// returned a FIXED four-entry vector — `generation`, `extraction`, `review`,
/// `advisor` — because under a closed set those were the only names that
/// could exist. The set is open now, so there is no fixed list to print and
/// the names are DERIVED from `models.<runtime>` after `normalize_models`
/// (the operator's own keys plus the defaults bee seeds there), exactly as
/// every other role surface derives them.
///
/// Read from the TABLE rather than from `known_roles`, deliberately: the
/// resolver warns on a name absent from both the table and the built-in
/// defaults, and `known_roles` adds `ceiling`, which no config carries —
/// walking it here would print one stderr warning per session render on every
/// host. Every key of the table is by definition present in the table, so this
/// walk cannot warn. (`known_roles` used to add `advisor` on every host too,
/// configured or not; that was the P2 this comment already smelled, and it is
/// fixed at the source — see `verbs::drivers::known_roles`.)
///
/// `ceiling` is excluded: D5 (store 97ce5225) makes it an escalation flag,
/// never a role, and it selects no model.
///
/// Order is derived too, never hand-listed: the slots bee's own dispatch
/// kinds resolve come first, in `DISPATCH_KINDS` order, so the name most
/// dispatches land on still reads first; whatever else the host configures
/// follows in config order.
pub(crate) fn role_slot_display(
    models_raw: Option<&Value>,
    runtime: &str,
) -> Vec<(String, String)> {
    use crate::verbs::drivers::{
        normalize_models, resolve_role, slot_for_kind, DISPATCH_KINDS, ESCALATION_WORD,
    };
    let map = normalize_models(models_raw);
    let table = map.get(runtime).and_then(Value::as_object);
    let mut order: Vec<String> = Vec::new();
    for kind in DISPATCH_KINDS {
        if let Some(slot) = slot_for_kind(kind) {
            if table.is_some_and(|t| t.contains_key(slot)) && !order.iter().any(|n| n == slot) {
                order.push(slot.to_string());
            }
        }
    }
    if let Some(t) = table {
        for name in t.keys() {
            if name == ESCALATION_WORD || order.iter().any(|n| n == name) {
                continue;
            }
            order.push(name.clone());
        }
    }
    order
        .into_iter()
        .filter_map(|name| {
            let resolved = resolve_role(&map, &[name.as_str()], runtime, "gather");
            render_role(&resolved).map(|text| (name, text))
        })
        .collect()
}

/// The dispatch door block — both lines, in ONE place.
///
/// The session preamble renders it at session start
/// (`hooks/session_preamble/budget.rs`) and `hooks/compaction.rs` re-injects
/// it after a compaction. They used to carry two copies of the same literal,
/// which is exactly how a post-compaction agent ends up being told something
/// the preamble no longer says.
///
/// What changed with the open role set (D2, and `--role` from store
/// `8ff6e79e`): the command line names `--role <name>`, and the second line
/// publishes the host's roles instead of a fixed tier list. The list is safe
/// to truncate because a wrong guess is not silent — a name nothing
/// configures refuses BY NAME with a FIX at both doors
/// (`unconfigured_role_reason` here, `--role`'s refusal in `prepare.rs`).
pub(crate) fn dispatch_door_lines(models_raw: Option<&Value>, runtime: &str) -> Vec<String> {
    let roles = role_slot_display(models_raw, runtime);
    let shown = std::cmp::min(DOOR_ROLES_SHOWN, roles.len());
    let mut listed =
        roles[..shown].iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" | ");
    if roles.len() > shown {
        listed.push_str(&format!(" +{} more", roles.len() - shown));
    }
    if listed.is_empty() {
        listed.push_str("none configured");
    }
    vec![
        format!(
            "- Every subagent/worker dispatch starts with `.bee/bin/bee dispatch prepare --runtime {runtime} --kind cell|gather|reviewer|advisor [--role <name>] --json` — run the exact tool+payload it returns; never hand-pick subagent_type, model, or a [bee-tier] marker."
        ),
        format!(
            "- Roles ({runtime}): {listed} — open set: any name models.{runtime} configures is legal; one nothing configures refuses by name."
        ),
    ]
}

/// Every model a bare `model:` param may name — derived from what the
/// resolver can publish for this runtime, never from a list kept beside it.
///
/// model-role-split D2 (store 06e49368): this walked the literal
/// `["extraction", "generation", "review"]` plus the advisor slot, so a model
/// configured under any OTHER role (`models.claude.test`) failed membership
/// and the dispatch DENIED — the hard blocker on an open role set. It now
/// walks whatever `models.claude` carries, which is that map's keys after
/// `normalize_models`: the operator's roles plus bee's own seeded defaults.
/// `advisor` needs no special case any more — it is one of those keys when it
/// is configured, and `resolve_tier` reads its slot directly (mrs-2), so the
/// separate `resolve_advisor` call was a second read of the same value.
///
/// hgf-1: a herding slot carrying `fallback:"default"` also contributes its
/// runtime default model. `dispatch prepare`'s Resolved::Herding arm publishes
/// exactly that model as `payload.fallback.model` — bee telling the
/// orchestrator to re-dispatch through the Agent path on a failed herding run
/// — so refusing it here would deny a dispatch bee itself prepared. The name
/// comes from drivers::models::default_models, the same table prepare reads;
/// a second copy in this hook would drift the moment either door moved.
fn configured_model_set(models: &Map<String, Value>) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    let roles: Vec<String> = models
        .get("claude")
        .and_then(Value::as_object)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default();
    for role in &roles {
        // Resolved::Model ONLY. A native slot carries a model string too, and
        // matching it here would silently widen what a bare `model:` param may
        // name — a native slot is a transport this hook's Agent/Task surface
        // does not dispatch, so it contributes no member. Deliberate, and the
        // reason the port is a match on one variant rather than a model_name()
        // call.
        if let Resolved::Model { model, .. } = resolve_tier(models, role, "claude", GUARD_PURPOSE)
        {
            if !model.trim().is_empty() {
                set.insert(model.trim().to_string());
            }
        }
    }
    // hgf-1: the herding-fallback contribution covers exactly the slots a
    // PREPARED dispatch can publish a fallback for — `slot_for_kind` over
    // `DISPATCH_KINDS`, read from prepare.rs itself instead of restated here
    // as ["generation", "review"]. `extraction` is not among them (no --kind
    // resolves it), so no prepared dispatch ever publishes an extraction
    // fallback and admitting one would widen the guard past the door it
    // mirrors; `advisor` IS among them, and contributes nothing because
    // `default_models` carries no advisor entry to publish.
    //
    // model-role-split, decision 8dad7c2e's second consequence: that decision
    // said this member set "must widen to the extraction slot, since prepare
    // can now publish an extraction fallback". It does NOT widen here, and
    // nothing is left to widen by hand. a2f85972's rule is "exactly the set
    // prepare can publish — no wider, no narrower", and the loop below is a
    // DERIVED view of that set rather than a fence around it: the day
    // `slot_for_kind` can reach the extraction role, this loop admits it in
    // the same commit, with no edit here and no chance of the two doors
    // disagreeing again. Until then `prepare` still publishes only
    // generation/review/advisor, so admitting haiku today would make the
    // guard wider than the door — the exact defect a2f85972 was logged for.
    for kind in crate::verbs::drivers::DISPATCH_KINDS {
        let Some(slot) = crate::verbs::drivers::slot_for_kind(kind) else { continue };
        // The shared parser carries the flag on the resolved value itself
        // (`Resolved::Herding { fallback }` mirrors the normalized
        // `"fallback": "default"` verbatim), so the raw-slot peek the deleted
        // private enum forced — its Herding was a unit variant — is gone. The
        // review-falls-back-to-generation rule for an explicitly null or
        // absent review slot rides along inside resolve_tier, the same rule
        // applied at the same moment as before.
        if matches!(
            resolve_tier(models, slot, "claude", GUARD_PURPOSE),
            Resolved::Herding { fallback: Some(ref f), .. } if f == "default"
        ) {
            if let Some(Value::String(m)) =
                crate::verbs::drivers::default_models("claude").get(slot)
            {
                if !m.trim().is_empty() {
                    set.insert(m.trim().to_string());
                }
            }
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

/// The rendered bee agent a ROLE is served by — `None` when the role has none
/// of its own.
///
/// model-role-split D2/D3: this hook used to carry its own copy of the
/// role→agent table, a hand-maintained twin of `verbs::drivers::ROLE_AGENTS`.
/// One table now, in the drivers module, exactly as D1 did for the config
/// parser: the guard ASKS it and never restates it, so the two cannot drift
/// the way the two tier lists already had.
///
/// `None` is live and legal. The old `unwrap_or("undefined")` was unreachable
/// while only four names could reach the lookup; under an open role set most
/// configured roles have no rendered agent at all, and rewriting a dispatch's
/// `subagent_type` to a generic — or to the literal "undefined" — would be a
/// repair that breaks the call it repairs. The caller skips the repair.
///
/// `generation` is answered by bee-gather, the read-only one of the two
/// agents that serve it: the guard cannot tell a gather from a cell execution
/// by the role alone, so it refuses to guess and asks the caller instead
/// (the `generic-type-denied` branch below).
fn pinned_type_for(role: &str) -> Option<&'static str> {
    crate::verbs::drivers::agent_for_role(role)
}

/// The role a rendered bee agent type already stands for. These files are
/// generated FROM the role's configured model at onboarding, so naming one is
/// a role declaration in every sense that matters — the guard reading it as
/// one is what keeps the role decision off the caller's memory.
fn role_for_pinned_type(subagent_type: &str) -> Option<&'static str> {
    crate::verbs::drivers::role_for_agent(subagent_type)
}

fn evaluate_codex_spawn(tool_input: &Value, models: &Map<String, Value>) -> Verdict {
    let Value::Object(obj) = tool_input else { return no_opinion() };
    let Some(message) = obj.get("message").and_then(Value::as_str) else { return no_opinion() };
    if message.is_empty() {
        return no_opinion();
    }
    match marker_of(&Value::String(message.to_string()), models, "codex") {
        Some(Marker::Role(role)) => allow("codex-spawn-marker", Some(role), None, None),
        // model-role-split D2: a marker IS present and names a role bee
        // cannot resolve. Under the deleted CODEX_TIERS list this read as no
        // marker at all and earned the unmarked refusal below, which never
        // said the one thing worth saying — that the name itself is the
        // problem. The spawn is still refused; now it is refused by name.
        Some(Marker::Unconfigured(name)) => {
            let reason = unconfigured_role_reason(&name, models, "codex");
            deny(reason, "codex-spawn-role-unconfigured", Some(name), None, None)
        }
        None => {
            let roles = role_list(models, "codex");
            let reason = format!(
                "bee-model-guard: every Codex spawn_agent needs an explicit role — its \
message must OPEN with a [bee-tier: <role>] marker (decision 0023 \
parity, codex-native-runtime-v2 D4, i54-closeout D1). A marker anywhere but the \
start of the message does not count, and a marker in any other field is ignored; \
without one the spawned worker silently inherits the session model.\n\
FIX: begin the spawn message with the marker, e.g. \
\"[bee-tier: generation] <task>\" (configured roles: {roles})."
            );
            deny(reason, "codex-spawn-unmarked", None, None, None)
        }
    }
}

/// D1(d): the `--kind` a FIX message hands to `dispatch prepare` so it reads
/// the same slot the guard just resolved by role name.
///
/// DERIVED from `slot_for_kind` over `DISPATCH_KINDS`, never listed here
/// (model-role-split D2/D3, the same rule `known_roles` and the
/// herding-fallback loop already follow). The hand-written table this
/// replaces answered `reviewer` and `gather` and `None` for everything else,
/// which was already WRONG for `advisor`: `--kind advisor` resolves the
/// advisor slot and prepare handles every transport it can carry, yet the
/// FIX told the reader no `--kind` existed. A derived answer cannot go stale
/// when a kind's slot changes, and it cannot name a `--kind` that would
/// resolve a DIFFERENT slot than the one just refused.
///
/// `cell` is skipped: it is not a door a reader can walk through from a
/// refusal (it requires an already-claimed cell id and a worker name), and it
/// shares `gather`'s slot, so skipping it costs no coverage.
///
/// `None` means this role has no `--kind` at all — and under D3 it never
/// will, because a job role is not published as a dispatch kind. A FIX for
/// such a role names the refused slot's OWN transport, and says nothing about
/// a `--kind` that is not coming.
fn dispatch_kind_for_role(role: &str) -> Option<&'static str> {
    crate::verbs::drivers::DISPATCH_KINDS
        .into_iter()
        .filter(|kind| *kind != "cell")
        .find(|kind| crate::verbs::drivers::slot_for_kind(kind) == Some(role))
}

fn evaluate_claude_dispatch(tool_input: &Value, models: &Map<String, Value>) -> Verdict {
    let Value::Object(obj) = tool_input else { return no_opinion() };

    let model_param: Option<String> = obj
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);
    let subagent_type: Option<String> =
        obj.get("subagent_type").and_then(Value::as_str).map(String::from);

    // model-role-split D2: the marker is parsed for SHAPE, then classified
    // against what this runtime can resolve. A well-formed marker naming a
    // role nothing configures refuses HERE, before any other branch — the
    // declared role is wrong whatever else the dispatch carries, and letting
    // it read as "no marker" (what the deleted CLAUDE_TIERS list did) would
    // hand a typo the bare-dispatch refusal, or worse, let a pinned
    // subagent_type rescue it silently.
    let tier: Option<String> = match marker_tier(obj, models) {
        Some(Marker::Unconfigured(name)) => {
            let reason = unconfigured_role_reason(&name, models, "claude");
            return deny(
                reason,
                "role-not-configured",
                Some(name),
                model_param,
                subagent_type,
            );
        }
        Some(Marker::Role(role)) => Some(role),
        None => None,
    };

    // (0) Pinned-type rule (W3, AO5/AO10/AO11) — REPAIRED, not refused. The
    // tier is already stated; which agent file carries it is a lookup the
    // guard owns outright, so making the caller re-issue the dispatch to
    // supply it buys nothing but a round trip and a chance to guess again.
    if let Some(t) = &tier {
        // Two agents, one role: the guard cannot tell a gather from a cell
        // execution by the role alone, and guessing picked the read-only one
        // for years — an execution dispatch that could never write. The
        // caller says which; that is one word.
        //
        // DERIVED, never named. This condition read `t == "generation"`, and
        // `generation` is the HISTORICAL spelling: every freshly onboarded
        // host configures `code`, which aliases onto the same job, so
        // `[bee-tier: code]` walked past the check and the branch below
        // repaired it onto `bee-gather` — the read-only agent — and the
        // execution dispatch died later at the write guard with the audit
        // line naming an agent that never ran. The ambiguity is a property of
        // the TABLE ("more than one rendered agent serves this job"), so the
        // guard asks the table. A new alias, or a second agent rendered for a
        // role that has one today, is then covered by construction.
        let agents = crate::verbs::drivers::agents_for_role(t);
        if agents.len() > 1 && subagent_type.as_deref() == Some("general-purpose") {
            let reason = format!(
                "bee-model-guard: [bee-tier: {t}] dispatched with subagent_type \
\"general-purpose\", and the {t} role carries {} rendered agents ({}) — the guard will \
not guess which.\n\
FIX: name the one you mean. subagent_type \"bee-build\" executes a cell (reserves, writes, \
commits, caps); subagent_type \"bee-gather\" reads and reports (never writes).",
                agents.len(),
                agents.join(", ")
            );
            return deny(
                reason,
                "generic-type-denied",
                tier.clone(),
                model_param,
                subagent_type,
            );
        }
        // D2: only a role that HAS a rendered agent is repaired onto it.
        // Every other configured role (`advisor`, and any job name the
        // operator invents) has no agent file to name, so the dispatch falls
        // through to the branches below, where the marker either agrees with
        // the model param or stands alone as the budget declaration.
        if let (false, Some("general-purpose"), Some(pinned)) =
            (t == "ceiling", subagent_type.as_deref(), pinned_type_for(t))
        {
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
        // GUARD_PURPOSE: an Agent/Task dispatch is a cell-execution-shaped
        // subagent, never a cli process — the cli slot must refuse here.
        let resolved = resolve_tier(models, t, "claude", GUARD_PURPOSE);
        if let Resolved::Model { model: resolved_model, .. } = &resolved {
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
        let cli_note =
            if matches!(resolved, Resolved::Refused { .. }) { " (the slot is a cli executor)" } else { "" };
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
        let door: Option<String> = match dispatch_kind_for_role(t) {
            Some(kind) => Some(format!(
                "run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" \
for the {t} role's own transport"
            )),
            None => match resolved {
                Resolved::Herding { .. } => Some(format!(
                    "run \".bee/bin/bee herding run --task-file - --json\" directly with the \
prompt on stdin — the {t} role's own transport"
                )),
                Resolved::Refused { .. } => Some(format!(
                    "run the configured command verbatim with the prompt on stdin — the {t} \
role's own transport"
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
        // GUARD_PURPOSE: same cell-execution purpose as branch (1).
        let resolved = resolve_tier(models, t, "claude", GUARD_PURPOSE);
        if matches!(resolved, Resolved::Refused { .. }) {
            let fix = match dispatch_kind_for_role(t) {
                Some(kind) => format!(
                    "FIX: run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" — it \
reads .bee/config.json for this slot and returns the tool and exact payload to run \
(here, a Bash call running the configured command verbatim with the prompt on \
stdin). Do not attach a model param; the cli command names its own model."
                ),
                None => format!(
                    "FIX: run the {t} role's cli slot directly — a Bash call running the \
configured command verbatim with the prompt on stdin. Do not attach a model param; \
the cli command names its own model."
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
        if matches!(resolved, Resolved::Herding { .. }) {
            // herding-tier D5: mirror of the cli-tier-denied wording just
            // above — an Agent/Task subagent cannot BE the pane a herding
            // slot spawns.
            let fix = match dispatch_kind_for_role(t) {
                Some(kind) => format!(
                    "FIX: run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" — it \
reads .bee/config.json for this slot and returns the tool and exact payload to run \
(here, a Bash call running \".bee/bin/bee herding run --task-file - --json\", plus \
--cwd for a granted worktree, with the prompt on stdin). Do not attach a model \
param; the herding worker names its own model."
                ),
                None => format!(
                    "FIX: run the {t} role's herding slot directly — a Bash call running \
\".bee/bin/bee herding run --task-file - --json\" (plus --cwd for a granted \
worktree) with the prompt on stdin. Do not attach a model param; the herding worker \
names its own model."
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
    if let Some(t) = subagent_type.as_deref().and_then(role_for_pinned_type) {
        // GUARD_PURPOSE: same cell-execution purpose as branch (1).
        let resolved = resolve_tier(models, t, "claude", GUARD_PURPOSE);
        if matches!(resolved, Resolved::Refused { .. }) {
            let fix = match dispatch_kind_for_role(t) {
                Some(kind) => format!(
                    "FIX: run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" (it \
reads .bee/config.json and returns the tool and exact payload — here, a Bash call \
running the configured command verbatim with the prompt on stdin), or name a role \
whose slot is a model."
                ),
                None => format!(
                    "FIX: run the {t} role's cli slot directly (a Bash call running the \
configured command verbatim with the prompt on stdin), or name a role whose slot is \
a model."
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
        if matches!(resolved, Resolved::Herding { .. }) {
            // herding-tier D5: same mirror as the marker-only branch above,
            // reached here when the pinned subagent_type itself implies the
            // tier instead of an explicit marker.
            let fix = match dispatch_kind_for_role(t) {
                Some(kind) => format!(
                    "FIX: run \".bee/bin/bee dispatch prepare --runtime claude --kind {kind} --json\" (it \
reads .bee/config.json and returns the tool and exact payload — here, a Bash call \
running \".bee/bin/bee herding run --task-file - --json\" with the prompt on stdin), \
or name a role whose slot is a model."
                ),
                None => format!(
                    "FIX: run the {t} role's herding slot directly (a Bash call running \
\".bee/bin/bee herding run --task-file - --json\" with the prompt on stdin), or name \
a role whose slot is a model."
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
    // GUARD_PURPOSE: the FIX text names the generation slot's own transport,
    // so it must read the slot exactly as the branches above refuse it.
    let gen_resolved = resolve_tier(models, "generation", "claude", GUARD_PURPOSE);
    // D2: the role names this FIX offers are the CONFIGURED ones, read at the
    // moment of the refusal — naming a fixed three here would send a caller
    // whose config carries other roles to a shorter list than bee accepts.
    let roles = role_list(models, "claude");
    let bare_fix = if let Resolved::Model { model: gen_model, .. } = &gen_resolved {
        format!(
            "FIX: name one of bee's rendered agents in subagent_type (bee-gather = generation, \
bee-extract = extraction, bee-review = review) — that alone declares the role. \
Otherwise pass model: \"{gen_model}\" for the generation role, or open the \
prompt/description with [bee-tier: ceiling] (or any configured role: {roles})."
        )
    } else {
        let slot_kind = match gen_resolved {
            Resolved::Herding { .. } => "a herding executor",
            Resolved::Refused { .. } => "a cli executor",
            _ => "unconfigured",
        };
        format!(
            "FIX: name one of bee's rendered agents in subagent_type (bee-gather = generation, \
bee-extract = extraction, bee-review = review) — that alone declares the role. \
Otherwise open the prompt/description with [bee-tier: ceiling] (or any configured \
role: {roles}). The generation role is {slot_kind}: run \
\".bee/bin/bee dispatch prepare --runtime claude --kind gather --json\" — it reads \
.bee/config.json and returns the tool and exact payload to run (a Bash call, either \
the configured cli command or a herding-pane invocation) rather than a model param."
        )
    };
    let reason = format!(
        "bee-model-guard: every Agent/Task dispatch needs an explicit role — a rendered \
bee agent type, a `model` param, or a `[bee-tier: <role>]` marker opening the \
prompt/description (decision 0023). A bare dispatch would silently inherit the most \
expensive session model.\n{bare_fix}"
    );
    deny(reason, "bare-denied", None, None, subagent_type)
}

// ─── dispatch economics (g22-2) ─────────────────────────────────────────────

fn derive_dispatch_economics(
    models: &Map<String, Value>,
    is_codex_spawn: bool,
    verdict: &Verdict,
) -> Option<Map<String, Value>> {
    let channel = if is_codex_spawn { "codex-native" } else { "claude-agent" };
    let runtime = if is_codex_spawn { "codex" } else { "claude" };
    // GUARD_PURPOSE — the one genuinely ambiguous site: this line is driven
    // by a [bee-tier] marker on a claude Task, which may be a cell execution
    // or a gather, and the marker does not say which. Resolved either way
    // yields the SAME audit value (a cli slot resolves to Resolved::Refused
    // under "cell" and to Resolved::Cli under a gather purpose, and neither
    // names a model), so the choice is free of behavior. It is made "cell"
    // for one reason: the audit line must record the model the verdict above
    // was reached on, and every verdict branch resolved with GUARD_PURPOSE.
    let resolved = verdict.tier.as_ref().map(|t| resolve_tier(models, t, runtime, GUARD_PURPOSE));
    let resolved_model = resolved.as_ref().and_then(resolved_model_name);
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
    // this hook's own second normalize_models (only ["claude", "codex"] were
    // parsed) — docs/config-reference.md called that "dead config that never
    // resolves". model-role-split D1 deleted that copy, so these rows now
    // read the ONE parser through the exact call this hook makes
    // (GUARD_PURPOSE); every assertion is the one the private-resolver test
    // made.
    fn model(name: &str) -> Resolved {
        Resolved::Model { model: name.to_string(), effort: None }
    }

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
            resolve_tier(&models, "generation", "opencode", GUARD_PURPOSE),
            model("opencode/big-pickle")
        );
        assert_eq!(
            resolve_tier(&models, "extraction", "opencode", GUARD_PURPOSE),
            model("opencode/ling-3.0-tiny-free")
        );
        assert_eq!(
            resolve_tier(&models, "review", "opencode", GUARD_PURPOSE),
            model("opencode/nemotron-3-ultra-free")
        );
        // Unconfigured (no models.opencode key at all) resolves to Budget on
        // every slot — same no-baked-in-default treatment codex gets.
        let models = normalize_models(None);
        assert_eq!(
            resolve_tier(&models, "generation", "opencode", GUARD_PURPOSE),
            Resolved::Budget
        );
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

        // EVERY spelling of the job, not just the one this check used to name.
        // The condition read the literal `"generation"`; a freshly onboarded
        // host configures `code`, which aliases onto the same two agents, so
        // `[bee-tier: code]` walked past the refusal and was repaired onto
        // bee-gather — the READ-ONLY agent — and the execution dispatch died
        // later at the write guard with the audit line naming an agent that
        // never ran. Sweeping both tables is what makes an alias spelling
        // reachable by this test at all.
        let migrated = fixture(&json!({"models": {"claude": {
            "code": "sonnet", "read": "haiku",
            "extraction": "haiku", "generation": "sonnet", "review": "opus"
        }}}));
        let spellings = crate::verbs::drivers::ROLE_ALIASES
            .iter()
            .map(|(job, _)| *job)
            .chain(crate::verbs::drivers::ROLE_AGENTS.iter().map(|(role, _)| *role));
        for role in spellings {
            let agents = crate::verbs::drivers::agents_for_role(role);
            let (code, stdout, stderr) = run_full(migrated.path(), json!({"tool_name": "Agent", "tool_input": {"prompt": format!("[bee-tier: {role}] go"), "subagent_type": "general-purpose"}}));
            if agents.len() > 1 {
                assert_ne!(code, 0, "{role} carries {agents:?} and must not be guessed: {stderr}");
                for agent in &agents {
                    assert!(stderr.contains(agent), "the refusal names {agent}: {stderr}");
                }
            } else {
                assert_eq!(code, 0, "{role}: {stderr}");
                assert_eq!(
                    repair_output(&stdout)["hookSpecificOutput"]["updatedInput"]["subagent_type"],
                    json!(agents[0]),
                    "{role}"
                );
            }
        }
    }

    #[test]
    fn pinned_type_rule() {
        let fx = fixture(&repo_config());
        // generation is excluded: it carries two agents and refuses instead
        // of repairing (a_generation_dispatch_names_its_agent_and_bee_build_is_allowed).
        for &(tier, pinned) in
            crate::verbs::drivers::ROLE_AGENTS.iter().filter(|(t, _)| *t != "generation")
        {
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
        // and nothing else. It now carries its role in the agent type.
        for (tier, pinned) in crate::verbs::drivers::ROLE_AGENTS {
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
        // D1(d) round 2, retargeted by model-role-split: the behavior guarded
        // here is that a FIX for a role with no `--kind` names that role's OWN
        // transport and never a `--kind` resolving a DIFFERENT slot (`--kind
        // advisor` would resolve the advisor slot, never the refused
        // extraction one). `dispatch_kind_for_role` now derives its answer
        // from `slot_for_kind`, so extraction still yields no kind.
        //
        // What changed is the WORDING, deliberately: the message used to say
        // "dispatch prepare has no --kind for the extraction tier yet", which
        // named a remedy that is not coming — under D3 a job role is never
        // published as a dispatch kind, so "yet" was a promise bee has
        // decided not to keep. The assertion below pins the behavior instead
        // of the sentence: NO --kind at all, and the slot's own transport
        // named in full.
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
        assert!(!stderr.contains("--kind"), "no --kind may be named for a role that has none: {stderr}");
        assert!(stderr.contains("herding-executor pane") && stderr.contains("herding run --task-file - --json"));
        assert!(
            !stderr.contains("has no --kind for the") && !stderr.contains("yet"),
            "the retired sentence named a remedy that does not exist: {stderr}"
        );
        assert!(
            stderr.contains("the extraction role's herding slot"),
            "the FIX names the refused role's own transport: {stderr}"
        );
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

    // hgf-1: the two doors used to disagree. `dispatch prepare`'s
    // Resolved::Herding arm reads `fallback:"default"` and publishes
    // `payload.fallback = {model: <default_models(runtime)[slot]>}` — the
    // model the orchestrator re-dispatches on when the herding run fails —
    // while this hook's member set collected only Resolved::Model slots and
    // denied that very model. The flag now widens the membership check, and
    // nothing else.
    fn herding_generation(fallback: Option<Value>) -> Value {
        let mut slot = json!({"kind": "herding", "agent": "agy-flash"});
        if let Some(fb) = fallback {
            slot["fallback"] = fb;
        }
        json!({"models": {"claude": {
            "extraction": "haiku",
            "generation": slot,
            "review": "opus"
        }}})
    }

    #[test]
    fn a_herding_slots_default_fallback_model_is_a_member() {
        let fx = fixture(&herding_generation(Some(json!("default"))));
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"model": "sonnet", "prompt": "re-dispatch after a failed herding run"}}),
        );
        assert_eq!(code, 0, "prepare publishes sonnet as this slot's fallback: {stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "model-param");
        assert_eq!(d["model"], "sonnet");
        // The flag rides on the resolved slot, so the set is directly checkable.
        let models = normalize_models(herding_generation(Some(json!("default"))).get("models"));
        assert!(configured_model_set(&models).contains("sonnet"));
    }

    #[test]
    fn a_herding_slot_without_fallback_admits_no_model() {
        let fx = fixture(&herding_generation(None));
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"model": "sonnet"}}),
        );
        assert_eq!(code, 2, "no fallback field, no membership");
        assert!(stderr.contains("model: \"sonnet\" is not a model configured for any claude tier"), "{stderr}");
        assert!(stderr.contains("(haiku, opus)"), "{stderr}");
        assert!(stderr.contains("[bee-tier: ceiling]"), "{stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "param-not-configured");
    }

    #[test]
    fn only_the_literal_default_fallback_admits_a_model() {
        // Same exact-match posture drivers/models.rs uses when it round-trips
        // the field: anything but the literal string "default" is dropped.
        for fallback in [json!("Default"), json!("sonnet"), json!(""), json!("  default  "), json!(true), json!({"model": "sonnet"})] {
            let fx = fixture(&herding_generation(Some(fallback.clone())));
            let (code, stderr) = run_payload(
                fx.path(),
                json!({"tool_name": "Agent", "tool_input": {"model": "sonnet"}}),
            );
            assert_eq!(code, 2, "fallback {fallback} must not admit a model: {stderr}");
            assert!(stderr.contains("is not a model configured for any claude tier"), "{stderr}");
            assert!(stderr.contains("(haiku, opus)"), "{stderr}");
        }
    }

    #[test]
    fn a_herding_review_slots_fallback_default_is_opus() {
        // A herding review slot resolves on its own (no review→generation
        // fallback fires), so its default-model table entry is "opus".
        let raw = json!({"claude": {
            "extraction": "haiku",
            "generation": "sonnet",
            "review": {"kind": "herding", "agent": "agy-flash", "fallback": "default"}
        }});
        let models = normalize_models(Some(&raw));
        let set = configured_model_set(&models);
        assert!(set.contains("opus"), "{set:?}");
        assert_eq!(set.iter().cloned().collect::<Vec<_>>(), vec!["haiku", "opus", "sonnet"]);
    }

    #[test]
    fn a_herding_review_slots_fallback_default_is_allowed_as_a_param() {
        // The observable half of the row above: the verdict, not the set.
        let fx = fixture(&json!({"models": {"claude": {
            "extraction": "haiku",
            "generation": "sonnet",
            "review": {"kind": "herding", "agent": "agy-flash", "fallback": "default"}
        }}}));
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"model": "opus", "prompt": "re-review after a failed herding run"}}),
        );
        assert_eq!(code, 0, "{stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "model-param");
        assert_eq!(d["model"], "opus");
    }

    #[test]
    fn an_explicitly_null_review_slot_inherits_the_generation_herding_fallback() {
        // resolve_tier makes a null review slot fall back to generation, so a
        // reviewer dispatch resolves to the herding slot and prepare publishes
        // default_models("claude")["review"] = "opus". The member set must
        // follow that same fallback, or the original bug survives in this one
        // config shape.
        let raw = json!({"claude": {
            "extraction": "haiku",
            "generation": {"kind": "herding", "agent": "agy-flash", "fallback": "default"},
            "review": null
        }});
        let models = normalize_models(Some(&raw));
        assert!(configured_model_set(&models).contains("opus"), "{:?}", configured_model_set(&models));
        let fx = fixture(&json!({"models": raw}));
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"model": "opus", "prompt": "re-review after a failed herding run"}}),
        );
        assert_eq!(code, 0, "{stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "model-param");
    }

    #[test]
    fn a_herding_extraction_slots_fallback_admits_nothing() {
        // The live repo config shape. prepare.rs's slot_for_kind never yields
        // "extraction", so no prepared dispatch publishes an extraction
        // fallback and the guard must not admit one — this is the fence that
        // keeps the membership check from widening past the door it mirrors.
        let fx = fixture(&json!({"models": {"claude": {
            "extraction": {"kind": "herding", "agent": "agy-flash", "fallback": "default"},
            "generation": {"kind": "herding", "agent": "agy-flash", "fallback": "default"},
            "review": "opus"
        }}}));
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"model": "haiku"}}),
        );
        assert_eq!(code, 2, "the extraction default is not a prepared fallback");
        assert!(stderr.contains("model: \"haiku\" is not a model configured for any claude tier"), "{stderr}");
        assert!(stderr.contains("(opus, sonnet)"), "{stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "param-not-configured");
    }

    #[test]
    fn the_fallback_flag_does_not_widen_the_tier_or_transport_check() {
        // hgf-1 widens the model-membership check ONLY. An Agent/Task
        // subagent still cannot BE a herding pane (herding-tier D5), whether
        // the tier arrives as a marker or as a pinned subagent_type.
        let fx = fixture(&herding_generation(Some(json!("default"))));
        for tool_input in [
            json!({"subagent_type": "bee-gather", "prompt": "gather"}),
            json!({"prompt": "[bee-tier: generation] go"}),
        ] {
            let (code, stderr) = run_payload(
                fx.path(),
                json!({"tool_name": "Agent", "tool_input": tool_input.clone()}),
            );
            assert_eq!(code, 2, "still denied: {tool_input}");
            assert!(stderr.contains("herding-executor pane"), "{stderr}");
            assert!(stderr.contains("herding run --task-file - --json"), "{stderr}");
            let d = last_jsonl(dispatch_log(fx.path())).unwrap();
            assert_eq!(d["transport"], "herding-tier-denied");
            assert_eq!(d["tier"], "generation");
        }
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
        // doc-canonical marked shape + extras tolerated
        for ti in [
            json!({"task_name": "wt-a1", "message": "[bee-tier: generation] gather", "fork_turns": "none"}),
            json!({"agent_type": "worker", "message": "[bee-tier: review] check", "extra": 1, "task_name": "x"}),
        ] {
            let (code, _) = run_payload(fx.path(), json!({"tool_name": "spawn_agent", "tool_input": ti}));
            assert_eq!(code, 0);
        }
        // The advisor tier, on a host that CONFIGURES one. `repo_config`'s
        // codex table does not, and the row used to run there and pass —
        // `known_roles` handed every dispatch-door slot out as legal whether
        // the host configured it or not, so this spawn inherited the session
        // model in silence. It is a refusal there now, so the acceptance row
        // states its own precondition.
        let advisor = fixture(&json!({"models": {
            "codex": {"extraction": "gpt-5.5", "generation": "gpt-5.5", "advisor": "gpt-5.5"}
        }}));
        let (code, stderr) = run_payload(
            advisor.path(),
            json!({"tool_name": "spawn_agent", "tool_input": {"agent_type": "worker", "message": "[bee-tier: advisor] consult", "model": "totally-different", "reasoning_effort": "extreme", "fork_turns": "full"}}),
        );
        assert_eq!(code, 0, "{stderr}");
        let d = last_jsonl(dispatch_log(advisor.path())).unwrap();
        assert_eq!(d["transport"], "codex-spawn-marker");
        assert_eq!(d["tier"], "advisor");
        // Same spawn, same marker, on the host with no codex advisor.
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "spawn_agent", "tool_input": {"agent_type": "worker", "message": "[bee-tier: advisor] consult", "fork_turns": "full"}}),
        );
        assert_eq!(code, 2, "an unconfigured advisor is refused by name: {stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "codex-spawn-role-unconfigured");
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

    // ─── model-role-split D2 (store 06e49368): the open role set ───────────

    /// A config carrying roles bee ships no default for — `test` and an
    /// `advisor` on claude, `design` on codex — beside the seeded ones. The
    /// two runtimes deliberately DISAGREE about `advisor`: claude configures
    /// one, codex does not, which is the pair the role-legality question has
    /// to answer differently.
    fn open_role_config() -> Value {
        json!({"models": {
            "claude": { "extraction": "haiku", "generation": "sonnet", "review": "opus", "test": "gpt-test", "advisor": "fable" },
            "codex": { "generation": "gpt-5.5", "design": "gpt-design" }
        }})
    }

    #[test]
    fn the_known_role_set_is_derived_from_what_the_host_configures() {
        let models = normalize_models(open_role_config().get("models"));
        let claude = known_roles(&models, "claude");
        // The operator's own roles, the defaults normalize seeds, and the
        // escalation word — every entry published by something, none of them
        // typed into this file.
        for name in ["test", "advisor", "extraction", "generation", "review", "ceiling"] {
            assert!(claude.contains(name), "{name} missing from {claude:?}");
        }
        assert!(!claude.contains("tset"), "{claude:?}");
        // One derivation for both runtimes, so the 4-against-5 drift the two
        // deleted lists carried cannot come back: the runtimes differ by
        // exactly what each config carries and nothing else.
        let codex = known_roles(&models, "codex");
        assert!(codex.contains("design"), "{codex:?}");
        // A dispatch-door SLOT is legal only where it is configured. The set
        // used to union every `slot_for_kind` answer in unconditionally, so
        // `advisor` was legal on a host with no advisor and a marker naming it
        // inherited the session model in silence.
        assert!(!codex.contains("advisor"), "an unconfigured advisor is not a legal role: {codex:?}");
        assert_eq!(
            claude.difference(&codex).cloned().collect::<Vec<_>>(),
            vec!["advisor".to_string(), "test".to_string()],
            "claude {claude:?} / codex {codex:?}"
        );
        // And the FIX line offers exactly the legal names, so the refusal
        // cannot advertise a role the same host would then refuse.
        assert!(!role_list(&models, "codex").contains("advisor"), "{}", role_list(&models, "codex"));
    }

    #[test]
    fn a_role_outside_the_old_three_slots_is_a_configured_model() {
        // The hard dependency D2 names: the member set walked a literal
        // ["extraction", "generation", "review"], so this model DENIED.
        let models = normalize_models(open_role_config().get("models"));
        let set = configured_model_set(&models);
        assert!(set.contains("gpt-test"), "{set:?}");
        let fx = fixture(&open_role_config());
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"model": "gpt-test", "prompt": "run the tests"}}),
        );
        assert_eq!(code, 0, "{stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "model-param");
        assert_eq!(d["model"], "gpt-test");
    }

    #[test]
    fn a_marker_naming_any_configured_role_is_accepted() {
        let fx = fixture(&open_role_config());
        for (payload, role) in [
            (json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: test] run them"}}), "test"),
            // `advisor` on a claude Agent: the exact name the deleted
            // CLAUDE_TIERS list dropped and CODEX_TIERS kept. It is accepted
            // here because THIS host configures one — see
            // `an_unconfigured_advisor_marker_is_refused_like_any_other_role`
            // for the same marker on a host that does not.
            (json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: advisor] consult"}}), "advisor"),
            // Case-insensitive as the old alternation was, answering in the
            // config's own spelling.
            (json!({"tool_name": "Agent", "tool_input": {"description": "[BEE-TIER: Test] mixed case"}}), "test"),
        ] {
            let (code, stderr) = run_payload(fx.path(), payload.clone());
            assert_eq!(code, 0, "{payload}: {stderr}");
            let d = last_jsonl(dispatch_log(fx.path())).unwrap();
            assert_eq!(d["tier"], role, "{payload}");
        }
        // The param path agrees with the marker path: a configured role's own
        // model is what its marker resolves to.
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: test] go", "model": "gpt-test"}}),
        );
        assert_eq!(code, 0, "{stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "model-param");
    }

    /// The dispatch-door slots are roles like any other: legal where the host
    /// configures them, refused where it does not.
    ///
    /// `advisor` was the one name `known_roles` added unconditionally, so this
    /// marker classified as `Marker::Role`, skipped the unconfigured-role
    /// refusal, resolved to `Resolved::Budget` and was ALLOWED with no model
    /// param — the subagent inheriting the session model, which is verbatim
    /// the outcome `unconfigured_role_reason` exists to prevent. The same host
    /// refused `bee dispatch prepare --role advisor`: one question, two doors,
    /// two answers.
    #[test]
    fn an_unconfigured_advisor_marker_is_refused_like_any_other_role() {
        // Three seeded slots and nothing else — the shape of a host that
        // never configured an advisor.
        let fx = fixture(&json!({"models": {
            "claude": {"extraction": "haiku", "generation": "sonnet", "review": "opus"},
            "codex": {"generation": "gpt-5.5"}
        }}));
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: advisor] consult"}}),
        );
        assert_eq!(code, 2, "an unconfigured advisor must not inherit the session model: {stderr}");
        assert!(stderr.contains("[bee-tier: advisor] names a role nothing configures"), "{stderr}");
        // The FIX teaches the remedy and does not offer the very name it just
        // refused as one of the roles to use instead.
        assert!(stderr.contains("models.claude in .bee/config.json"), "{stderr}");
        assert!(!stderr.contains("(advisor/"), "the FIX must not advertise advisor: {stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "role-not-configured");
        assert_eq!(d["tier"], "advisor");
        assert_eq!(d["model"], Value::Null, "nothing was allowed onto the session model");

        // The same marker on a host that DOES configure one is untouched.
        let with_advisor = fixture(&json!({"models": {"claude": {
            "extraction": "haiku", "generation": "sonnet", "review": "opus", "advisor": "fable"
        }}}));
        let (code, stderr) = run_payload(
            with_advisor.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: advisor] consult"}}),
        );
        assert_eq!(code, 0, "{stderr}");
        let d = last_jsonl(dispatch_log(with_advisor.path())).unwrap();
        assert_eq!(d["tier"], "advisor");
    }

    #[test]
    fn a_marker_naming_an_unconfigured_role_refuses_with_a_fix() {
        let fx = fixture(&open_role_config());
        // A pinned subagent_type would have rescued this dispatch if the
        // typo'd marker read as "no marker" the way the closed list made it.
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: tset] typo", "subagent_type": "bee-gather"}}),
        );
        assert_eq!(code, 2, "{stderr}");
        assert!(stderr.contains("[bee-tier: tset] names a role nothing configures"), "{stderr}");
        assert!(stderr.contains("FIX:"), "{stderr}");
        // The remedy is named, and the roles it offers are the configured
        // ones — including the operator's own.
        assert!(stderr.contains("models.claude in .bee/config.json"), "{stderr}");
        assert!(stderr.contains("test"), "{stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "role-not-configured");
        assert_eq!(d["tier"], "tset");
    }

    #[test]
    fn an_unconfigured_role_on_a_codex_spawn_refuses_by_name() {
        let fx = fixture(&open_role_config());
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "spawn_agent", "tool_input": {"agent_type": "worker", "message": "[bee-tier: tset] go"}}),
        );
        assert_eq!(code, 2, "{stderr}");
        assert!(stderr.contains("names a role nothing configures"), "{stderr}");
        assert!(stderr.contains("models.codex in .bee/config.json"), "{stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "codex-spawn-role-unconfigured");
        assert_eq!(d["tier"], "tset");
        // A codex role bee ships no default for is accepted on its own name.
        let (code, stderr) = run_payload(
            fx.path(),
            json!({"tool_name": "spawn_agent", "tool_input": {"agent_type": "worker", "message": "[bee-tier: design] draw it"}}),
        );
        assert_eq!(code, 0, "{stderr}");
        let d = last_jsonl(dispatch_log(fx.path())).unwrap();
        assert_eq!(d["transport"], "codex-spawn-marker");
        assert_eq!(d["tier"], "design");
    }

    #[test]
    fn a_role_with_no_rendered_agent_is_never_repaired_to_undefined() {
        // `pinned_type_for` ended in `unwrap_or("undefined")`, unreachable
        // while only four names could reach it. Every role an operator
        // invents reaches it now, and a repair to a subagent_type that does
        // not exist would break the dispatch it claims to fix.
        let fx = fixture(&open_role_config());
        for role in ["test", "advisor"] {
            let (code, stdout, stderr) = run_full(
                fx.path(),
                json!({"tool_name": "Agent", "tool_input": {"prompt": format!("[bee-tier: {role}] go"), "subagent_type": "general-purpose"}}),
            );
            assert_eq!(code, 0, "{role}: {stderr}");
            assert_eq!(stdout, "", "{role} has no rendered agent to repair onto");
            let d = last_jsonl(dispatch_log(fx.path())).unwrap();
            assert_eq!(d["subagent_type"], "general-purpose", "{role}");
            assert_eq!(d["tier"], role);
        }
        // The roles that DO have one are repaired exactly as before.
        let (code, stdout, _) = run_full(
            fx.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: review] check", "subagent_type": "general-purpose"}}),
        );
        assert_eq!(code, 0);
        let out = repair_output(&stdout);
        assert_eq!(out["hookSpecificOutput"]["updatedInput"]["subagent_type"], json!("bee-review"));
    }

    #[test]
    fn the_dispatch_kind_in_a_fix_is_derived_from_the_door_not_listed() {
        // The hand-written table this replaces answered Some only for
        // `review` and `generation`, so an `advisor` refusal printed "dispatch
        // prepare has no --kind for the advisor tier yet" while `--kind
        // advisor` existed and resolved exactly that slot. Derived from
        // `slot_for_kind`, the FIX cannot be wrong about the door.
        assert_eq!(dispatch_kind_for_role("generation"), Some("gather"));
        assert_eq!(dispatch_kind_for_role("review"), Some("reviewer"));
        assert_eq!(dispatch_kind_for_role("advisor"), Some("advisor"));
        // A role no kind resolves — the shipped extraction slot, and any job
        // role an operator invents. Under D3 no `--kind` is coming for these,
        // so the FIX names the slot's own transport and promises nothing.
        assert_eq!(dispatch_kind_for_role("extraction"), None);
        assert_eq!(dispatch_kind_for_role("test"), None);

        // End to end: a herding-shaped advisor slot is still denied (an
        // Agent/Task subagent cannot BE a pane), but the FIX now names the
        // door that resolves it instead of denying that one exists.
        let herding = fixture(&json!({"models": {"claude": {
            "extraction": "haiku",
            "generation": "sonnet",
            "review": "opus",
            "advisor": {"kind": "herding", "agent": "agy-flash"}
        }}}));
        let (code, stderr) = run_payload(
            herding.path(),
            json!({"tool_name": "Agent", "tool_input": {"prompt": "[bee-tier: advisor] consult"}}),
        );
        assert_eq!(code, 2, "a herding-shaped advisor slot cannot be an in-family subagent");
        assert!(
            stderr.contains("--kind advisor"),
            "the FIX names the door that resolves the refused slot: {stderr}"
        );
        assert!(!stderr.contains("has no --kind for the"), "{stderr}");
        let d = last_jsonl(dispatch_log(herding.path())).unwrap();
        assert_eq!(d["transport"], "herding-tier-denied");
        assert_eq!(d["tier"], "advisor");
    }

    #[test]
    fn the_role_to_agent_table_has_exactly_one_home() {
        // model-role-split D2/D3: the hook's own copy is gone. Both lookups
        // read `verbs::drivers::ROLE_AGENTS`, so the pair cannot drift the
        // way the two tier lists already had.
        for (role, agent) in crate::verbs::drivers::ROLE_AGENTS {
            assert_eq!(role_for_pinned_type(agent), Some(role), "{agent}");
        }
        assert_eq!(pinned_type_for("generation"), Some("bee-gather"));
        assert_eq!(pinned_type_for("extraction"), Some("bee-extract"));
        assert_eq!(pinned_type_for("review"), Some("bee-review"));
        // A role with no rendered agent answers None — a legal answer, never
        // a generic or invented type.
        assert_eq!(pinned_type_for("advisor"), None);
        assert_eq!(pinned_type_for("test"), None);
        assert_eq!(pinned_type_for("ceiling"), None);
        assert_eq!(role_for_pinned_type("general-purpose"), None);
        assert_eq!(role_for_pinned_type("some-other-agent"), None);

        // HOW MANY agents serve a job is the question the pinned-type rule
        // asks, and it must answer the same for both spellings of one job —
        // the alias is why keying that rule on a literal was wrong.
        use crate::verbs::drivers::{agents_for_role, ROLE_ALIASES};
        for (job, keyed) in ROLE_ALIASES {
            assert_eq!(agents_for_role(job), agents_for_role(keyed), "{job}/{keyed}");
        }
        assert_eq!(agents_for_role("code"), vec!["bee-gather", "bee-build"]);
        assert_eq!(agents_for_role("read"), vec!["bee-extract"]);
        assert_eq!(agents_for_role("review"), vec!["bee-review"]);
        // A role with no rendered agent has none, in every spelling.
        assert!(agents_for_role("advisor").is_empty());
        assert!(agents_for_role("test").is_empty());
    }
}
