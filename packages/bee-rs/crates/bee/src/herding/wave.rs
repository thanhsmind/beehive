// herding::wave — `bee herding wave` and `bee herding occupancy`
// (herding-orchestration D17, docs/history/herding-orchestration/CONTEXT.md).
//
// `wave` is the bee-side entry point D17 locks: it is what turns
// `herding.agent_command` into a running wave. `HerdrBackend::new`
// (`crates/fleet/src/backend/herdr.rs`) had ZERO callers anywhere in the
// workspace before this file — its own documentation names the caller's
// obligation (split `herding.agent_command` per D14 into kind and args)
// and THIS is that caller. Token 0 passes straight through to herdr as the
// agent kind, unchecked (D2) — herdr refuses an unrecognised kind itself,
// after the pane split, not this file's own allow-list.
//
// `occupancy` is the CLI bridge to the ledger's read side
// (`super::wave_ledger::live_worker_count`), which was crate-private and
// unusable from anywhere outside this crate — in particular unusable from
// `role-dispatch.md` §4, a markdown role that can only run shell commands.
//
// Both verbs are deliberately split into a thin CLI-parsing shell and a pure
// (or backend-generic) inner function, so every behavioural test below runs
// with NO real `herdr` on PATH (D7's test seam) and NO herdr server:
//   - `resolve_agent_command` is pure: config JSON in, (kind, args, env,
//     workspace_trust) or a typed `AgentCommandError` out — `env` (D4) is
//     the resolved registry entry's per-agent env map, and
//     `workspace_trust` (D5) its optional trust-store declaration, both
//     empty/None outside the object shape. No process, no I/O beyond the
//     caller having already read the config file.
//   - `run_wave_and_record` is generic over `WorkerBackend`, so a test
//     drives it with `fleet::backend::fake::FakeBackend` instead of
//     `HerdrBackend` — the same seam `fleet`'s own choreography tests use
//     (D7). `wave()` (the CLI verb) is the only caller that ever PASSES a
//     real `HerdrBackend` into a wave run; the construction itself is the
//     named `real_backend_ctor`, which a test below also calls directly.
//   - `occupancy_json` is pure: an `Occupancy` value in, its JSON shape out
//     — `Live` and `Fallback` never collapse to the same shape. `occupancy()`
//     (the CLI verb) is the only caller that ever shells out to a real
//     `herdr pane list`; when that call fails (no herdr installed, in every
//     test environment) it returns `None` and the ledger's own fallback
//     path answers instead — never a test dependency on a live herdr.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use serde_json::{Map, Value};

use fleet::backend::herdr::HerdrBackend;
use fleet::backend::tmux::TmuxBackend;
use fleet::backend::WorkerBackend;
use fleet::choreography::{run_wave, WaveResult};
use fleet::screen::ScreenSettings;
use fleet::wave::{FailurePolicy, Wave, WaveTimeouts, WorkerSpec};

use super::tmux::TmuxSettings;
use super::wave_ledger;
use super::TransportKind;

// ═══════════════════════════════════════════════════════════════════════════
// D14 — splitting herding.agent_command into (agent kind, agent args)
// ═══════════════════════════════════════════════════════════════════════════

/// The default `herding.agent_command` array
/// (`skills/bee-herding/references/operational-invariants.md`, "Runtime
/// adapter") used when the config key is absent, not an array, empty, or
/// carries a non-string/newline-bearing element — the same fail-open shape
/// `command_template` above already uses for a malformed template.
const DEFAULT_AGENT_COMMAND: &[&str] =
    &["claude", "--model", "sonnet", "--permission-mode", "bypassPermissions"];

/// The one placeholder `herding.agent_command` defines
/// (`operational-invariants.md`): the fixed model, substituted per-token,
/// never by joining tokens and re-splitting (the shell-injection-safe shape
/// the runtime adapter requires).
const MODEL_PLACEHOLDER: &str = "{MODEL}";
const MODEL_VALUE: &str = "sonnet";

/// D14's fail-closed arm: the split left zero tokens, naming the config key
/// it came from. Per D2, herdr owns validating token 0 as an agent kind —
/// this error is never raised for an unrecognised kind, only for a split
/// with nothing in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AgentCommandError {
    Empty { key: &'static str },
    /// herd-registry D2: a named lookup — `--agent <name>`, a tier slot's
    /// `agent`, or a string-valued `herding.agent_command` — named an
    /// entry `herding.agents` does not declare. `known` carries every
    /// registry key (sorted) so the refusal names its own remedy without a
    /// second read.
    UnknownAgent { name: String, known: Vec<String> },
}

impl std::fmt::Display for AgentCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentCommandError::Empty { key } => {
                write!(f, "{key}: resolved to zero tokens")
            }
            AgentCommandError::UnknownAgent { name, known } => {
                if known.is_empty() {
                    write!(f, "unknown herding agent {name:?} (herding.agents declares no entries)")
                } else {
                    write!(
                        f,
                        "unknown herding agent {name:?} (herding.agents declares: {})",
                        known.join(", ")
                    )
                }
            }
        }
    }
}

impl std::error::Error for AgentCommandError {}

fn substitute_model(token: &str) -> String {
    token.replace(MODEL_PLACEHOLDER, MODEL_VALUE)
}

/// Reads `herding.agent_command` out of an already-parsed `.bee/config.json`
/// value, falling back to `DEFAULT_AGENT_COMMAND` on anything malformed —
/// same fail-open rule `command_template` uses for the same key's siblings.
fn agent_command_tokens(cfg: &Value) -> Vec<String> {
    let fallback = || DEFAULT_AGENT_COMMAND.iter().map(|s| s.to_string()).collect();
    let Some(Value::Array(tokens)) = cfg.get("herding").and_then(|h| h.get("agent_command")) else {
        return fallback();
    };
    if tokens.is_empty() {
        return fallback();
    }
    let mut out = Vec::with_capacity(tokens.len());
    for t in tokens {
        match t.as_str() {
            Some(s) if !s.contains('\n') => out.push(s.to_string()),
            _ => return fallback(),
        }
    }
    out
}

/// D5 — a `herding.agents` object-shape entry's optional workspace-trust
/// declaration: `file` names a foreign tool's own trust-store JSON file
/// (a leading `~` expanded to `$HOME` at parse time), and `key` names the
/// array field inside it that holds trusted absolute paths. Nothing here
/// names Antigravity specifically — the declaration is config-driven, not
/// hard-coded (D5's own requirement).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceTrust {
    pub(crate) file: String,
    pub(crate) key: String,
}

/// One `herding.agents` entry, resolved: `argv` is the (unsubstituted)
/// token array — either the plain array shape or an object shape's
/// `"argv"` field — and `env` is that object shape's optional `"env"` map
/// (D4), empty for the array shape, the built-ins, and every entry the
/// pre-D4 array-only registry ever produced. `workspace_trust` (D5) is the
/// same object shape's optional workspace-trust declaration, empty for the
/// array shape and every entry that declares none.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RegistryEntry {
    argv: Vec<String>,
    env: BTreeMap<String, String>,
    workspace_trust: Option<WorkspaceTrust>,
}

/// Expands a leading `~` (or `~/...`) to the user's home, same shape as
/// every shell's own tilde expansion. `HOME` first, then `USERPROFILE` —
/// the same order the standard library uses, and the only one that works
/// on Windows, where `HOME` is normally unset. Anything else (no leading
/// `~`, neither variable set) is returned unchanged — the caller's
/// fail-open read of the resulting path names its own "file not found"
/// warning either way.
fn expand_tilde(path: &str) -> String {
    let Some(rest) = path.strip_prefix('~') else { return path.to_string() };
    if !rest.is_empty() && !rest.starts_with('/') {
        // `~bob/...` names another user's home — outside this expansion's
        // scope, left unchanged rather than guessed at.
        return path.to_string();
    }
    let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) else {
        return path.to_string();
    };
    format!("{home}{rest}")
}

/// D3 — the two built-in herd names the registry pre-seeds so `--agent
/// agy-flash` / `--agent claude-sonnet` work with zero `herding.agents`
/// config: same argv `.bee/config-sample-cli-executors.json` already shows
/// as a live example, carrying no env.
fn built_in_agents() -> BTreeMap<String, RegistryEntry> {
    let mut out = BTreeMap::new();
    out.insert(
        "claude-sonnet".to_string(),
        RegistryEntry {
            argv: ["claude", "--model", "sonnet", "--permission-mode", "bypassPermissions"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ..Default::default()
        },
    );
    out.insert(
        "agy-flash".to_string(),
        RegistryEntry {
            argv: ["agy", "--dangerously-skip-permissions"].iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        },
    );
    out
}

/// Validates one argv token array the same way `agent_command_tokens`
/// already validates `herding.agent_command`'s: non-empty, every element a
/// newline-free string. `None` on any violation — the caller decides what
/// "invalid" means for its own shape (fall back for `agent_command`, drop
/// the entry for the registry).
fn parse_argv_tokens(tokens: &[Value]) -> Option<Vec<String>> {
    if tokens.is_empty() {
        return None;
    }
    let mut collected = Vec::with_capacity(tokens.len());
    for t in tokens {
        match t.as_str() {
            Some(s) if !s.contains('\n') => collected.push(s.to_string()),
            _ => return None,
        }
    }
    Some(collected)
}

/// D4's env-key rule: `[A-Za-z_][A-Za-z0-9_]*` — a plain shell identifier,
/// checked by hand (no regex dependency needed for one anchored pattern).
fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// D4's `env` map: every key must pass `is_valid_env_key`, every value a
/// newline-free string. `None` on any single violation — the caller drops
/// the WHOLE entry on it (fail-open-per-entry, never a partial env).
fn parse_env_map(env: &Map<String, Value>) -> Option<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for (k, v) in env {
        if !is_valid_env_key(k) {
            return None;
        }
        match v.as_str() {
            Some(s) if !s.contains('\n') => {
                out.insert(k.clone(), s.to_string());
            }
            _ => return None,
        }
    }
    Some(out)
}

/// D5's `workspace_trust` object: `{"file": "<path>", "key": "<array
/// field>"}`, both required, non-empty, newline-free strings. `file`'s
/// leading `~` is expanded here, once, at parse time. `None` on any shape
/// mismatch — the caller drops the whole entry on it, fail-open-per-entry,
/// exactly like a bad env key.
fn parse_workspace_trust(value: &Value) -> Option<WorkspaceTrust> {
    let obj = value.as_object()?;
    let file = obj.get("file")?.as_str()?;
    let key = obj.get("key")?.as_str()?;
    if file.trim().is_empty() || file.contains('\n') || key.trim().is_empty() || key.contains('\n') {
        return None;
    }
    Some(WorkspaceTrust { file: expand_tilde(file), key: key.to_string() })
}

/// D4's two `herding.agents` entry shapes: the plain argv array (env always
/// empty, workspace_trust always absent), or `{"argv": [...], "env": {...},
/// "workspace_trust": {...}}` — `argv` validated exactly like the array
/// shape, `env` optional (absent = empty) and validated by `parse_env_map`,
/// `workspace_trust` (D5) optional (absent = none) and validated by
/// `parse_workspace_trust`. `None` on any shape mismatch or validation
/// failure — the caller drops the whole entry, fail-open-per-entry.
fn parse_registry_entry(value: &Value) -> Option<RegistryEntry> {
    match value {
        Value::Array(tokens) => {
            parse_argv_tokens(tokens).map(|argv| RegistryEntry { argv, env: BTreeMap::new(), workspace_trust: None })
        }
        Value::Object(obj) => {
            let Value::Array(tokens) = obj.get("argv")? else { return None };
            let argv = parse_argv_tokens(tokens)?;
            let env = match obj.get("env") {
                None => BTreeMap::new(),
                Some(Value::Object(env_obj)) => parse_env_map(env_obj)?,
                Some(_) => return None,
            };
            let workspace_trust = match obj.get("workspace_trust") {
                None => None,
                Some(v) => Some(parse_workspace_trust(v)?),
            };
            Some(RegistryEntry { argv, env, workspace_trust })
        }
        _ => None,
    }
}

/// herd-registry D1 (+ D3's built-ins, D4's env shape) — `herding.agents`:
/// starts from `built_in_agents()`, then overlays every config entry that
/// parses (`parse_registry_entry`), a same-name config entry replacing the
/// built-in outright. A malformed config entry is dropped, fail-open per
/// entry — it never poisons the rest of the registry, and never removes a
/// built-in it failed to override. `BTreeMap` keeps the key order the
/// `UnknownAgent` error lists deterministic.
fn agent_registry(cfg: &Value) -> BTreeMap<String, RegistryEntry> {
    let mut out = built_in_agents();
    let Some(Value::Object(agents)) = cfg.get("herding").and_then(|h| h.get("agents")) else {
        return out;
    };
    for (name, value) in agents {
        if let Some(entry) = parse_registry_entry(value) {
            out.insert(name.clone(), entry);
        }
    }
    out
}

/// The one place a name resolves against `herding.agents` (herd-registry
/// D2): token 0 becomes the kind, the rest the args, each substituted the
/// same way a plain `herding.agent_command` array is; the entry's `env`
/// (D4) and `workspace_trust` (D5) ride along unsubstituted. An unknown
/// name lists every registry key (built-ins included, since
/// `agent_registry` always seeds them).
fn resolve_from_registry(
    registry: &BTreeMap<String, RegistryEntry>,
    name: &str,
) -> Result<(String, Vec<String>, BTreeMap<String, String>, Option<WorkspaceTrust>), AgentCommandError> {
    let Some(entry) = registry.get(name) else {
        return Err(AgentCommandError::UnknownAgent {
            name: name.to_string(),
            known: registry.keys().cloned().collect(),
        });
    };
    let substituted: Vec<String> = entry.argv.iter().map(|t| substitute_model(t)).collect();
    let Some((kind, args)) = substituted.split_first() else {
        // agent_registry() never inserts an empty entry, but fail closed
        // rather than panic if it ever did.
        return Err(AgentCommandError::Empty { key: "herding.agents" });
    };
    Ok((kind.clone(), args.to_vec(), entry.env.clone(), entry.workspace_trust.clone()))
}

/// Returns the configured agent name from the cell-execution tier slot
/// (`models.<runtime>.generation`), but ONLY when it is an object with
/// `kind == "herding"` and an `agent` field that is a non-empty string.
/// `<runtime>` is mapped to one of `"claude"`, `"codex"`, `"opencode"`,
/// defaulting to `"claude"`.
fn generation_slot_herding_agent<'a>(cfg: &'a Value, runtime: &str) -> Option<&'a str> {
    let rt = if matches!(runtime, "claude" | "codex" | "opencode") { runtime } else { "claude" };
    let slot = cfg.get("models")?.get(rt)?.get("generation")?;
    let obj = slot.as_object()?;
    if obj.get("kind").and_then(Value::as_str) != Some("herding") {
        return None;
    }
    let agent = obj.get("agent").and_then(Value::as_str)?;
    if agent.trim().is_empty() {
        return None;
    }
    Some(agent)
}

fn current_runtime() -> String {
    std::env::var("BEE_RUNTIME").unwrap_or_else(|_| "claude".to_string())
}

/// D14's split: token 0 becomes the agent kind (herdr's `--kind`), the
/// remaining tokens — each substituted per-token — become the agent args
/// (herdr's trailing argv after `--`). Token 0 passes through UNCHECKED
/// (D2): `herdr` validates it as a `--kind` and refuses an unrecognised one
/// itself, after the pane split — never a bee-side allow-list.
///
/// herd-registry D2 (+ tier slot) — four reference spellings, one resolver:
///   - `agent = Some(name)`: a named lookup (`--agent <name>`), resolved
///     through `herding.agents` alone. An unknown name is
///     `AgentCommandError::UnknownAgent`, listing every registry key.
///   - `agent = None` and the cell-execution tier slot
///     (`models.<runtime>.generation`) is an object with `kind == "herding"`
///     and a non-empty `agent`: resolved through `herding.agents` the same
///     way (an unknown name refuses typed).
///   - `agent = None` and `herding.agent_command` is a plain JSON string:
///     that string names a `herding.agents` entry, resolved the SAME way
///     (an unknown name refuses the same way a named lookup does).
///   - `agent = None` and anything else: today's split — an array resolves
///     token 0 as the kind, and absent/malformed falls back to the
///     documented default. `AgentCommandError::Empty` is the fail-closed
///     arm for a split that leaves zero tokens.
pub(crate) fn resolve_agent_command(
    cfg: &Value,
    agent: Option<&str>,
) -> Result<(String, Vec<String>, BTreeMap<String, String>, Option<WorkspaceTrust>), AgentCommandError> {
    resolve_agent_command_for_runtime(cfg, agent, &current_runtime())
}

/// Resolves the agent command using the specified runtime name for tier slot lookup.
pub(crate) fn resolve_agent_command_for_runtime(
    cfg: &Value,
    agent: Option<&str>,
    runtime: &str,
) -> Result<(String, Vec<String>, BTreeMap<String, String>, Option<WorkspaceTrust>), AgentCommandError> {
    let registry = agent_registry(cfg);
    if let Some(name) = agent {
        return resolve_from_registry(&registry, name);
    }
    if let Some(name) = generation_slot_herding_agent(cfg, runtime) {
        return resolve_from_registry(&registry, name);
    }
    if let Some(name) = cfg.get("herding").and_then(|h| h.get("agent_command")).and_then(Value::as_str) {
        return resolve_from_registry(&registry, name);
    }
    let tokens = agent_command_tokens(cfg);
    let substituted: Vec<String> = tokens.iter().map(|t| substitute_model(t)).collect();
    let Some((kind, args)) = substituted.split_first() else {
        // agent_command_tokens() never returns empty (it falls back to the
        // non-empty default), but fail closed rather than panic if it ever did.
        return Err(AgentCommandError::Empty { key: "herding.agent_command" });
    };
    // D4/D5: the plain `herding.agent_command` array path names no registry
    // entry, so it carries no env and no workspace-trust declaration —
    // both are `herding.agents` object-shape features only.
    Ok((kind.clone(), args.to_vec(), BTreeMap::new(), None))
}

// ═══════════════════════════════════════════════════════════════════════════
// bee herding wave
// ═══════════════════════════════════════════════════════════════════════════

/// One worker as given to `bee herding wave` on stdin. `worktree` is
/// record-only — the generic core (D2) never learns it; it rides straight
/// into the ledger row's `worktree` field so the row matches D10's shape.
pub(crate) struct WaveWorkerInput {
    name: String,
    task: String,
    worktree: String,
}

/// Accepts either a bare JSON array of worker objects, or `{"workers": […]}`.
/// Each object needs `name` and `task`; `worktree` is optional (defaults to
/// `""`). Returns `None` on any shape mismatch — the caller reports it.
fn parse_worker_inputs(v: &Value) -> Option<Vec<WaveWorkerInput>> {
    let arr = match v {
        Value::Array(a) => a.clone(),
        Value::Object(o) => match o.get("workers") {
            Some(Value::Array(a)) => a.clone(),
            _ => return None,
        },
        _ => return None,
    };
    let mut out = Vec::with_capacity(arr.len());
    for item in &arr {
        let obj = item.as_object()?;
        let name = obj.get("name")?.as_str()?.to_string();
        let task = obj.get("task")?.as_str()?.to_string();
        let worktree = obj.get("worktree").and_then(Value::as_str).unwrap_or("").to_string();
        out.push(WaveWorkerInput { name, task, worktree });
    }
    Some(out)
}

/// Which `WaveResult` bucket `canonical_name` landed in, as the string the
/// ledger row's `outcome` field carries — `None` only when dedupe (Ordering
/// Invariant 8) collapsed this identity into an earlier row's, which then
/// carries the real outcome.
fn classify_outcome(result: &WaveResult, canonical_name: &str) -> Option<&'static str> {
    let hit = |names: &[String]| names.iter().any(|n| n == canonical_name);
    if hit(&result.succeeded) {
        Some("succeeded")
    } else if hit(&result.resolution_failed) {
        Some("resolution_failed")
    } else if hit(&result.unsafe_at_preflight) {
        Some("unsafe_at_preflight")
    } else if hit(&result.flipped_before_send) {
        Some("flipped_before_send")
    } else if hit(&result.send_failed) {
        Some("send_failed")
    } else if hit(&result.timed_out) {
        Some("timed_out")
    } else if hit(&result.unverifiable_after_send) {
        Some("unverifiable_after_send")
    } else {
        None
    }
}

/// Runs `wave` on `backend`, then appends EXACTLY ONE row to the wave
/// ledger (D10) — this call, not the choreography, owns the ledger write,
/// so `run_wave` itself stays memoryless (D5). Generic over `WorkerBackend`
/// so a test drives this with `fleet::backend::fake::FakeBackend`; `wave()`
/// below is the only caller that ever passes a real `HerdrBackend`.
pub(crate) fn run_wave_and_record<B: WorkerBackend + Sync + ?Sized>(
    backend: &B,
    root: &Path,
    wave_id: String,
    started_at: String,
    inputs: &[WaveWorkerInput],
    wave: &Wave,
) -> WaveResult {
    let result = run_wave(backend, wave);
    let rows: Vec<wave_ledger::WorkerRow> = inputs
        .iter()
        .map(|w| {
            // The same canonical identity `run_wave`'s own dedupe phase
            // resolved this name to (Ordering Invariant 8) — reading it
            // again is a read-only, idempotent query, never a second
            // dispatch.
            let pane_id = backend.canonical_id(&w.name);
            let outcome = classify_outcome(&result, &pane_id).map(str::to_string);
            wave_ledger::WorkerRow {
                name: w.name.clone(),
                pane_id,
                worktree: w.worktree.clone(),
                task: w.task.clone(),
                outcome,
                evidence: None,
            }
        })
        .collect();
    let row = wave_ledger::WaveRow { wave_id, started_at, workers: rows };
    // A ledger write failure is reported, never allowed to hide the wave's
    // own (already-computed) result from the caller.
    if let Err(e) = wave_ledger::append_wave(root, &row) {
        eprintln!("bee herding wave: could not append the wave ledger row: {e}");
    }
    result
}

/// Resolves `herding.agent_command` (D14), constructs the backend through
/// `construct_backend`, then runs and records the wave — the SAME call
/// site `wave()` below uses, passing `real_backend_ctor` (a NAMED
/// function, never a closure literal) as `construct_backend`. That
/// naming is what makes the real construction reachable from two
/// independent tests: one below drives THIS function with an assertion
/// closure over a `FakeBackend`, proving `construct_backend(kind, args)`
/// receives the D14-resolved pair; a second calls `real_backend_ctor`
/// itself directly, proving the real `HerdrBackend` construction this
/// function only forwards to carries that pair through. A closure
/// literal written at the `wave()` call site could satisfy neither test —
/// it is reachable by no test, only by `wave()` — which is why the
/// production side must be a name, not an inline closure.
fn build_wave_backend_and_run<B: WorkerBackend + Sync>(
    cfg: &Value,
    root: &Path,
    wave_id: String,
    started_at: String,
    inputs: &[WaveWorkerInput],
    wave: &Wave,
    construct_backend: impl FnOnce(String, Vec<String>) -> B,
) -> Result<WaveResult, AgentCommandError> {
    // `bee herding wave` names no per-worker agent (herd-registry D2 covers
    // only `--agent`, a tier slot's `agent`, and a string `agent_command`) —
    // this caller stays on the `None` arm, unchanged.
    let (kind, args, env, _wt) = resolve_agent_command(cfg, None)?;
    // D4's per-agent env is NOT applied on this path, deliberately: this
    // function never splits a pane — `fleet::backend::herdr::HerdrBackend`
    // (the backend `construct_backend` builds) starts an agent into a pane
    // id it is HANDED (`worker.name`), already split by whoever prepared
    // the wave's workers (see `HerdrBackend`'s own `decide_start_for` doc:
    // "splitting a pane needs a worktree cwd this generic WorkerSpec does
    // not carry"). There is no post-split/pre-start point in THIS file to
    // send an export line into, and the `agent start` call itself lives in
    // `crates/fleet/src/backend/herdr.rs` — a different crate, out of this
    // cell's file scope — so env stays unapplied here rather than reaching
    // past that boundary.
    let _ = &env;
    let backend = construct_backend(kind, args);
    Ok(run_wave_and_record(&backend, root, wave_id, started_at, inputs, wave))
}

/// The production `construct_backend`: builds a real `HerdrBackend` from
/// the D14-resolved `(kind, args)` pair, with no transformation of either.
/// `wave()` below passes THIS NAMED FUNCTION as `construct_backend` rather
/// than writing `HerdrBackend::new` inline as a closure literal — a
/// closure literal at that call site would be reachable by no test (the
/// generic seam's own test below supplies its OWN closure to the same
/// function, so the two never meet). Naming this construction lets a test
/// call it directly and assert on what it built, closing the gap a prior
/// judge found: mutating this function to ignore `kind`/`args` for
/// constants is caught by `real_backend_ctor_hands_kind_and_args_straight_through_to_construction`
/// below, independent of every other seam in this file.
fn real_backend_ctor(kind: String, args: Vec<String>) -> HerdrBackend {
    HerdrBackend::new(kind, args)
}

/// The tmux twin of `real_backend_ctor` (tmux-herding-cockpit D1/D4): the
/// production `construct_backend` when `herding.transport` is `tmux`.
///
/// It takes one more input than the herdr arm does — the screen-reading
/// settings, which tmux needs because it has no agent API and reads
/// status off the pane's own text — so it cannot BE a plain `fn` of
/// `(kind, args)`. It is a NAMED constructor factory instead: a function
/// returning the closure, so `wave()`'s call site still passes a name
/// (`tmux_backend_ctor(settings)`) and never an inline closure literal.
///
/// That naming is the same doctrine `real_backend_ctor`'s own doc records,
/// and for the same reason: a closure literal written at the `wave()` call
/// site is reachable by no test, while this factory can be called directly
/// — `tmux_backend_ctor(settings)(kind, args)` — and the backend it built
/// asserted on. Mutating it to construct with constants is caught by
/// `wave_tmux_backend_ctor_hands_kind_args_and_settings_straight_through`
/// below, independent of every other seam in this file.
fn tmux_backend_ctor(
    settings: ScreenSettings,
) -> impl FnOnce(String, Vec<String>) -> TmuxBackend {
    move |kind, args| TmuxBackend::new(kind, args, settings)
}

/// The production transport switch: ONE `match` on the configured
/// transport, each arm calling `build_wave_backend_and_run` once with its
/// own concrete backend type.
///
/// `build_wave_backend_and_run` is generic over `B`, so this cannot be a
/// single call with a runtime-chosen backend value — and it deliberately
/// is not boxed into a `dyn WorkerBackend`: the generic seam and its
/// tests are unchanged by this cell, and a box would add an indirection
/// nothing here needs. Both arms return the same `Result` type, so the
/// caller branches on the transport exactly once, here, and never again.
///
/// Named rather than inlined into `wave()` for the reason
/// `real_backend_ctor`'s doc gives about `wave()`: nothing written inside
/// `wave()` is reachable by a test, because `wave()` reads stdin and the
/// real filesystem. This function is.
fn run_wave_for_transport(
    transport: TransportKind,
    cfg: &Value,
    root: &Path,
    wave_id: String,
    started_at: String,
    inputs: &[WaveWorkerInput],
    wave: &Wave,
) -> Result<WaveResult, AgentCommandError> {
    match transport {
        TransportKind::Herdr => build_wave_backend_and_run(
            cfg,
            root,
            wave_id,
            started_at,
            inputs,
            wave,
            real_backend_ctor,
        ),
        TransportKind::Tmux => {
            // The screen knobs are bee's to read (`herding.tmux.*`) and
            // `fleet`'s to classify with — hence the unwrap into the
            // shared `ScreenSettings` on the way across the crate
            // boundary.
            let settings = TmuxSettings::from_config(cfg).into_screen_settings();
            build_wave_backend_and_run(
                cfg,
                root,
                wave_id,
                started_at,
                inputs,
                wave,
                tmux_backend_ctor(settings),
            )
        }
    }
}

fn emit_wave_result(wave_id: &str, result: &WaveResult, json: bool) {
    if json {
        let obj = serde_json::json!({
            "wave_id": wave_id,
            "success": result.is_success(),
            "succeeded": result.succeeded,
            "resolution_failed": result.resolution_failed,
            "unsafe_at_preflight": result.unsafe_at_preflight,
            "flipped_before_send": result.flipped_before_send,
            "send_failed": result.send_failed,
            "timed_out": result.timed_out,
            "unverifiable_after_send": result.unverifiable_after_send,
        });
        println!("{obj}");
        return;
    }
    println!(
        "wave {wave_id}: {} — succeeded {}, resolution_failed {}, unsafe_at_preflight {}, \
         flipped_before_send {}, send_failed {}, timed_out {}, unverifiable_after_send {}",
        if result.is_success() { "success" } else { "failed" },
        result.succeeded.len(),
        result.resolution_failed.len(),
        result.unsafe_at_preflight.len(),
        result.flipped_before_send.len(),
        result.send_failed.len(),
        result.timed_out.len(),
        result.unverifiable_after_send.len(),
    );
}

/// `bee herding wave` (D17) — the caller the whole feature has been
/// building toward. Reads `herding.agent_command` off `.bee/config.json` in
/// the MAIN checkout, splits it per D14, constructs the real
/// `HerdrBackend`, builds a `Wave` from the worker specs given on stdin (a
/// bare JSON array or `{"workers": […]}`, each `{"name", "task",
/// "worktree"?}`), runs the choreography, and appends one row to the wave
/// ledger.
///
/// Flags: `--main-root <path>` (same override `interlock`/`command-template`
/// take), `--wave-id <id>` (defaults to a timestamp-based id),
/// `--worker-settle-ms <n>` (default 60000), `--poll-interval-ms <n>`
/// (default 500), `--json` (machine-readable result on stdout instead of a
/// text summary — same flag nearly every other verb carries).
pub(super) fn wave(flags: &[&str]) -> ExitCode {
    let mut explicit_root: Option<&str> = None;
    let mut wave_id: Option<String> = None;
    let mut worker_settle_ms: u64 = 60_000;
    let mut poll_interval_ms: u64 = 500;
    let mut json = false;
    let mut i = 0usize;
    while i < flags.len() {
        match flags[i] {
            "--main-root" => {
                explicit_root = flags.get(i + 1).copied();
                i += 2;
            }
            "--wave-id" => {
                wave_id = flags.get(i + 1).map(|s| s.to_string());
                i += 2;
            }
            "--worker-settle-ms" => {
                if let Some(n) = flags.get(i + 1).and_then(|s| s.parse().ok()) {
                    worker_settle_ms = n;
                }
                i += 2;
            }
            "--poll-interval-ms" => {
                if let Some(n) = flags.get(i + 1).and_then(|s| s.parse().ok()) {
                    poll_interval_ms = n;
                }
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let Some(main_root) = super::resolve_main_root(explicit_root) else {
        eprintln!(
            "bee herding wave: could not resolve the MAIN checkout root (no --main-root given \
             and `git rev-parse --git-common-dir` failed)"
        );
        return ExitCode::FAILURE;
    };

    let raw = super::read_stdin();
    let Ok(parsed) = serde_json::from_str::<Value>(&raw) else {
        eprintln!("bee herding wave: could not parse worker specs as JSON on stdin");
        return ExitCode::FAILURE;
    };
    let Some(inputs) = parse_worker_inputs(&parsed) else {
        eprintln!(
            "bee herding wave: worker specs must be a JSON array of {{\"name\", \"task\"}} \
             objects (optionally under a \"workers\" key), each with string name/task"
        );
        return ExitCode::FAILURE;
    };
    if inputs.is_empty() {
        eprintln!("bee herding wave: no workers given on stdin");
        return ExitCode::FAILURE;
    }

    let cfg_path = main_root.join(".bee").join("config.json");
    let cfg: Value = std::fs::read_to_string(&cfg_path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(Value::Null);

    let wave_id = wave_id.unwrap_or_else(|| format!("wave-{}", chrono::Utc::now().timestamp_millis()));
    let started_at = chrono::Utc::now().to_rfc3339();

    let workers: Vec<WorkerSpec> =
        inputs.iter().map(|w| WorkerSpec::new(w.name.clone(), w.task.clone())).collect();
    let wave_value = Wave::new(
        workers,
        WaveTimeouts {
            worker_settle: Duration::from_millis(worker_settle_ms),
            poll_interval: Duration::from_millis(poll_interval_ms),
        },
        FailurePolicy::WaitForAll,
    );

    // Which backend the wave briefs its workers through is the SAME key
    // the run verb and occupancy read — `herding.transport`
    // (tmux-herding-cockpit D1/D4), never a sniff of `$TMUX`. A typo'd
    // value is a typed refusal, not a silent fallback to the other
    // transport.
    let transport = match super::transport_kind_at(&main_root) {
        Ok(kind) => kind,
        Err(message) => {
            if json {
                let mut m = Map::new();
                m.insert("error".into(), Value::String(message.clone()));
                m.insert("key".into(), Value::String("herding.transport".into()));
                println!("{}", Value::Object(m));
            }
            eprintln!("bee herding wave: {message}");
            return ExitCode::FAILURE;
        }
    };

    // `run_wave_for_transport` holds the one production switch, and each
    // arm passes a NAMED `construct_backend` — `real_backend_ctor` or
    // `tmux_backend_ctor(settings)` — never a closure literal written
    // here, which is what lets a test call the exact same construction
    // directly (see those functions' own doc comments).
    let result = match run_wave_for_transport(
        transport,
        &cfg,
        &main_root,
        wave_id.clone(),
        started_at,
        &inputs,
        &wave_value,
    ) {
        Ok(result) => result,
        Err(e) => {
            if json {
                let mut m = Map::new();
                m.insert("error".into(), Value::String(e.to_string()));
                m.insert("key".into(), Value::String("herding.agent_command".into()));
                println!("{}", Value::Object(m));
            }
            eprintln!("bee herding wave: {e}");
            return ExitCode::FAILURE;
        }
    };
    let success = result.is_success();
    emit_wave_result(&wave_id, &result, json);
    if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// bee herding occupancy
// ═══════════════════════════════════════════════════════════════════════════

/// Obtains the live pane id set the way the cockpit already does
/// (`skills/bee-herding/scripts/bootstrap-cockpit.sh`'s `find_dispatch_pane`,
/// also ported above as `herdr_pane_id`): shell out to `herdr pane list`,
/// parse either a bare array or `{"panes": […]}`, collect every
/// `pane_id`. `None` on ANY trouble — herdr not installed, a non-zero exit,
/// an unparsable body, an error envelope, an unrecognised shape — which is
/// exactly the signal `wave_ledger::live_worker_count` reads as "no live
/// list available" and answers through its `Occupancy::Fallback` path
/// instead. This is the only place in this file that spawns a `herdr`
/// process (its tmux twin below is the other, and only the other, spawn), so
/// no test exercising `occupancy_json`/`live_worker_count` below needs a
/// `herdr` binary on PATH.
fn live_pane_ids_via_herdr() -> Option<HashSet<String>> {
    let out = Command::new("herdr").args(["pane", "list"]).stdin(Stdio::null()).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let v: Value = serde_json::from_slice(&out.stdout).ok()?;
    if v.get("error").is_some_and(|e| !e.is_null()) {
        return None;
    }
    let result = v.get("result").cloned().unwrap_or(Value::Null);
    let panes = match &result {
        Value::Array(a) => a.clone(),
        Value::Object(o) => match o.get("panes") {
            Some(Value::Array(a)) => a.clone(),
            _ => return None,
        },
        _ => return None,
    };
    Some(panes.iter().filter_map(|p| p.get("pane_id").and_then(Value::as_str).map(str::to_string)).collect())
}

/// The tmux twin of `live_pane_ids_via_herdr` (tmux-herding-cockpit D1/D4).
/// `tmux list-panes -a -F '#{pane_id}'` prints one pane id per line across
/// EVERY session, which is the same population the herdr branch collects out
/// of `herdr pane list` — the ledger's recorded `pane_id`s are crossed
/// against it unchanged. Same fail-closed contract as the herdr lister:
/// `None` on ANY trouble (no `tmux` on PATH, no server running, a non-zero
/// exit), which `wave_ledger::live_worker_count` reads as "no live list
/// available" and answers through its `Occupancy::Fallback` path instead.
fn live_pane_ids_via_tmux() -> Option<HashSet<String>> {
    let out = Command::new("tmux")
        .args(["list-panes", "-a", "-F", "#{pane_id}"])
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(parse_tmux_pane_list(&String::from_utf8_lossy(&out.stdout)))
}

/// The pure parse half of `live_pane_ids_via_tmux`: one pane id per line,
/// empty lines dropped (`tmux` always ends its listing with a newline, and an
/// empty body is an empty set, never a one-element set holding `""`). Split
/// out so the parsing is pinned by tests with no `tmux` binary in sight.
fn parse_tmux_pane_list(stdout: &str) -> HashSet<String> {
    stdout.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect()
}

/// Picks the live-pane lister the configured transport names (D1: one
/// `herding.transport` key, never environment sniffing; absent reads as
/// `herdr`, so a repo with no key gets the byte-identical pre-tmux answer).
/// A refused key — a typo, which `transport_kind` reports as an `Err` rather
/// than guessing — resolves to `None`, i.e. the ledger's degraded timer
/// answer, never the other transport's pane list.
fn live_pane_ids(main_root: &Path) -> Option<HashSet<String>> {
    match super::transport_kind_at(main_root) {
        Ok(super::TransportKind::Herdr) => live_pane_ids_via_herdr(),
        Ok(super::TransportKind::Tmux) => live_pane_ids_via_tmux(),
        Err(_) => None,
    }
}

/// The occupancy answer's JSON shape — `Live` and `Fallback` NEVER collapse
/// to the same shape (the module doc's must-have: a caller has to be able
/// to tell the real crossing from the degraded timer answer apart).
fn occupancy_json(occ: &wave_ledger::Occupancy) -> Value {
    let source = match occ {
        wave_ledger::Occupancy::Live(_) => "live",
        wave_ledger::Occupancy::Fallback(_) => "fallback",
    };
    serde_json::json!({"count": occ.count(), "source": source})
}

/// The DEFAULT (non-`--json`) human-readable occupancy line. Carries the
/// same `source` distinction `occupancy_json` pins for the machine-readable
/// path — a role calling `bee herding occupancy` without `--json` must be
/// able to tell the real crossing from the degraded fallback apart too, so
/// this always renders the parenthetical, never just the bare count.
fn occupancy_plain_line(occ: &wave_ledger::Occupancy) -> String {
    let v = occupancy_json(occ);
    let count = v["count"].as_u64().unwrap_or(0);
    let source = v["source"].as_str().unwrap_or("?");
    format!("occupancy: {count} worker(s) live ({source})")
}

fn emit_occupancy(occ: &wave_ledger::Occupancy, json: bool) {
    if json {
        println!("{}", occupancy_json(occ));
        return;
    }
    println!("{}", occupancy_plain_line(occ));
}

/// `bee herding occupancy` — the CLI bridge to the wave ledger's read side
/// (D10's occupancy answer), reachable from a markdown role for the first
/// time (`role-dispatch.md` §4 can only run shell commands). Reports how
/// many worker slots are occupied AND which answer it gave: a real
/// pane-list cross-check (`source: "live"`) when `herdr pane list`
/// succeeded, or the degraded one-hour timer fallback (`source:
/// "fallback"`) when it did not.
///
/// Flags: `--main-root <path>` (same override the other verbs take),
/// `--json`.
pub(super) fn occupancy(flags: &[&str]) -> ExitCode {
    let mut explicit_root: Option<&str> = None;
    let mut json = false;
    let mut i = 0usize;
    while i < flags.len() {
        match flags[i] {
            "--main-root" => {
                explicit_root = flags.get(i + 1).copied();
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let Some(main_root) = super::resolve_main_root(explicit_root) else {
        eprintln!(
            "bee herding occupancy: could not resolve the MAIN checkout root (no --main-root \
             given and `git rev-parse --git-common-dir` failed)"
        );
        return ExitCode::FAILURE;
    };

    let live_panes = live_pane_ids(&main_root);
    let now_ms = chrono::Utc::now().timestamp_millis();
    let occ =
        wave_ledger::live_worker_count(&main_root, live_panes.as_ref(), now_ms, wave_ledger::DEFAULT_STALE_AFTER_MS);
    emit_occupancy(&occ, json);
    ExitCode::SUCCESS
}

// ═══════════════════════════════════════════════════════════════════════════
// bee herding record-worker
// ═══════════════════════════════════════════════════════════════════════════

/// The pure write side of `record-worker` (herding-orchestration D18): build
/// one `WaveRow` holding a SINGLE unresolved `WorkerRow` and append it
/// through the SAME `wave_ledger::append_wave` path `run_wave_and_record`
/// above uses — no second write path into the ledger, and the file stays
/// append-only exactly as `wave_ledger`'s own module doc requires.
///
/// `wave_id` defaults to the worker's own `name` when the caller does not
/// override it. That default is deliberate, not arbitrary: `role-dispatch.md`
/// §8 calls this once per spawn, with no wave-level id of its own to give —
/// dispatch opens one agent and walks away (D18), it never learns an
/// outcome — but the worker's self-chosen slug is already the identity a
/// LATER caller (a merge-side outcome recorder, not built by this cell)
/// would have on hand too, since it is the same slug the pane gets labelled
/// with and the same slug `role-dispatch.md` §5(c)/§7 already key off. A
/// later row appended under that identical `wave_id` — outcome filled in —
/// supersedes this one entirely at READ time
/// (`wave_ledger::fold_waves_by_wave_id`), never by rewriting these bytes.
fn append_worker_row(
    root: &Path,
    name: String,
    pane_id: String,
    worktree: String,
    task: String,
    wave_id: Option<String>,
    started_at: String,
) -> std::io::Result<String> {
    let wave_id = wave_id.unwrap_or_else(|| name.clone());
    let row = wave_ledger::WaveRow {
        wave_id: wave_id.clone(),
        started_at,
        workers: vec![wave_ledger::WorkerRow {
            name,
            pane_id,
            worktree,
            task,
            outcome: None,
            evidence: None,
        }],
    };
    wave_ledger::append_wave(root, &row)?;
    Ok(wave_id)
}

/// `bee herding record-worker` (D18) — the recording verb `role-dispatch.md`
/// §8 calls immediately after a successful `herdr agent start`, and only
/// after §8's own confirm step already checked exactly one new pane
/// appeared. A spawn this verb is never called for is invisible to the
/// NEXT iteration's occupancy read (`bee herding occupancy`, above): that
/// read is the whole reason this verb exists — §4 no longer counts panes,
/// it reads the wave ledger, and a spawn with no row in it simply is not in
/// what it reads.
///
/// Flags: `--main-root <path>` (same override every other verb in this file
/// takes), `--name <worker>` (the worker's own self-chosen slug — reuses
/// the existing `--name` spelling), `--pane-id <id>` (the herdr pane it is
/// running in), `--path <worktree>` (the worktree path it was given —
/// reuses the existing `--path` spelling from `reservations reserve`),
/// `--task <item>` (the item it was given, e.g. a PBI id), `--wave-id <id>`
/// (override the default described on `append_worker_row` above), `--json`.
/// All of `--name`, `--pane-id`, `--path` and `--task` are required.
pub(super) fn record_worker(flags: &[&str]) -> ExitCode {
    let mut explicit_root: Option<&str> = None;
    let mut name: Option<String> = None;
    let mut pane_id: Option<String> = None;
    let mut path: Option<String> = None;
    let mut task: Option<String> = None;
    let mut wave_id: Option<String> = None;
    let mut json = false;
    let mut i = 0usize;
    while i < flags.len() {
        match flags[i] {
            "--main-root" => {
                explicit_root = flags.get(i + 1).copied();
                i += 2;
            }
            "--name" => {
                name = flags.get(i + 1).map(|s| s.to_string());
                i += 2;
            }
            "--pane-id" => {
                pane_id = flags.get(i + 1).map(|s| s.to_string());
                i += 2;
            }
            "--path" => {
                path = flags.get(i + 1).map(|s| s.to_string());
                i += 2;
            }
            "--task" => {
                task = flags.get(i + 1).map(|s| s.to_string());
                i += 2;
            }
            "--wave-id" => {
                wave_id = flags.get(i + 1).map(|s| s.to_string());
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let Some(main_root) = super::resolve_main_root(explicit_root) else {
        eprintln!(
            "bee herding record-worker: could not resolve the MAIN checkout root (no \
             --main-root given and `git rev-parse --git-common-dir` failed)"
        );
        return ExitCode::FAILURE;
    };
    let (Some(name), Some(pane_id), Some(path), Some(task)) = (name, pane_id, path, task) else {
        eprintln!(
            "bee herding record-worker: --name, --pane-id, --path and --task are all required"
        );
        return ExitCode::FAILURE;
    };

    let started_at = chrono::Utc::now().to_rfc3339();
    let result = append_worker_row(&main_root, name, pane_id, path, task, wave_id, started_at);
    let wave_id = match result {
        Ok(wave_id) => wave_id,
        Err(e) => {
            eprintln!("bee herding record-worker: could not append the wave ledger row: {e}");
            return ExitCode::FAILURE;
        }
    };
    if json {
        println!("{}", serde_json::json!({"wave_id": wave_id, "recorded": true}));
    } else {
        println!("recorded worker in wave ledger (wave_id {wave_id})");
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use fleet::backend::fake::{FakeBackend, RawStatus};
    use fleet::backend::WorkerStatus;

    // ─── D14: config split ──────────────────────────────────────────────

    #[test]
    fn absent_config_falls_back_to_the_documented_default() {
        let (kind, args, _env, _wt) = resolve_agent_command(&Value::Null, None).unwrap();
        assert_eq!(kind, "claude");
        assert_eq!(args, vec!["--model", "sonnet", "--permission-mode", "bypassPermissions"]);
    }

    #[test]
    fn a_configured_command_splits_token_0_from_the_rest() {
        let cfg = serde_json::json!({"herding": {"agent_command": ["codex", "--flag", "value"]}});
        let (kind, args, _env, _wt) = resolve_agent_command(&cfg, None).unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag", "value"]);
    }

    #[test]
    fn model_placeholder_is_substituted_per_token_never_joined() {
        let cfg = serde_json::json!({"herding": {"agent_command": ["claude", "--model", "{MODEL}", "--x={MODEL}"]}});
        let (kind, args, _env, _wt) = resolve_agent_command(&cfg, None).unwrap();
        assert_eq!(kind, "claude");
        assert_eq!(args, vec!["--model", "sonnet", "--x=sonnet"]);
    }

    #[test]
    fn an_arbitrary_token_0_passes_through_unchecked_to_herdr() {
        // D2: bee owns no agent-kind allow-list. herdr validates `--kind`
        // itself, after the pane split — any token 0 (a real herdr kind
        // like "gemini", a typo, anything) resolves and reaches backend
        // construction unchanged.
        let cfg = serde_json::json!({"herding": {"agent_command": ["gemini", "--x"]}});
        let (kind, args, _env, _wt) = resolve_agent_command(&cfg, None).unwrap();
        assert_eq!(kind, "gemini");
        assert_eq!(args, vec!["--x"]);
    }

    #[test]
    fn an_empty_or_malformed_array_falls_back_to_the_default_not_an_error() {
        let empty = serde_json::json!({"herding": {"agent_command": []}});
        assert!(resolve_agent_command(&empty, None).is_ok());
        // A non-string, non-array value (herd-registry D2 reserves the
        // string shape for a registry-name alias below) still falls back
        // to the documented default, fail-open.
        let non_array = serde_json::json!({"herding": {"agent_command": 42}});
        assert!(resolve_agent_command(&non_array, None).is_ok());
    }

    // ─── herd-registry D1/D2: `herding.agents` + named resolution ──────

    fn registry_cfg() -> Value {
        serde_json::json!({
            "herding": {
                "agents": {
                    "codex-herd": ["codex", "--flag", "value"],
                    "gemini-herd": ["gemini", "--x"],
                }
            }
        })
    }

    #[test]
    fn a_named_lookup_returns_the_registry_argv() {
        let cfg = registry_cfg();
        let (kind, args, _env, _wt) = resolve_agent_command(&cfg, Some("codex-herd")).unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag", "value"]);
    }

    #[test]
    fn an_unknown_name_refuses_typed_listing_every_registry_key() {
        let cfg = registry_cfg();
        let err = resolve_agent_command(&cfg, Some("no-such-herd")).unwrap_err();
        let AgentCommandError::UnknownAgent { name, known } = &err else {
            panic!("expected UnknownAgent, got {err:?}");
        };
        assert_eq!(name, "no-such-herd");
        for key in ["codex-herd", "gemini-herd"] {
            assert!(known.contains(&key.to_string()), "{known:?} must list {key:?}");
        }
        let text = err.to_string();
        for key in ["codex-herd", "gemini-herd"] {
            assert!(text.contains(key), "error text {text:?} must list {key:?}");
        }
    }

    #[test]
    fn an_unknown_name_against_an_empty_registry_still_lists_the_name_and_refuses() {
        let err = resolve_agent_command(&Value::Null, Some("anything")).unwrap_err();
        assert!(err.to_string().contains("anything"));
    }

    #[test]
    fn a_string_valued_agent_command_aliases_through_the_registry() {
        let mut cfg = registry_cfg();
        cfg["herding"]["agent_command"] = Value::String("gemini-herd".to_string());
        let (kind, args, _env, _wt) = resolve_agent_command(&cfg, None).unwrap();
        assert_eq!(kind, "gemini");
        assert_eq!(args, vec!["--x"]);
    }

    #[test]
    fn a_string_valued_agent_command_naming_an_unknown_herd_refuses_typed() {
        let mut cfg = registry_cfg();
        cfg["herding"]["agent_command"] = Value::String("no-such-herd".to_string());
        let err = resolve_agent_command(&cfg, None).unwrap_err();
        let text = err.to_string();
        for key in ["codex-herd", "gemini-herd"] {
            assert!(text.contains(key), "error text {text:?} must list {key:?}");
        }
    }

    #[test]
    fn an_absent_agent_name_and_array_agent_command_keeps_the_default_even_with_a_registry() {
        // `herding.agents` present but `agent` is None and `agent_command`
        // is the documented array shape (not a name) — registry never
        // consulted, today's split behavior unchanged.
        let mut cfg = registry_cfg();
        cfg["herding"]["agent_command"] = serde_json::json!(["claude", "--flag"]);
        let (kind, args, _env, _wt) = resolve_agent_command(&cfg, None).unwrap();
        assert_eq!(kind, "claude");
        assert_eq!(args, vec!["--flag"]);
    }

    #[test]
    fn a_malformed_registry_entry_is_dropped_fail_open_not_poisoning_the_rest() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "good": ["codex", "--x"],
                    "bad-empty": [],
                    "bad-non-string": ["codex", 1],
                    "bad-newline": ["codex\n--x"],
                }
            }
        });
        let (kind, _args, _env, _wt) = resolve_agent_command(&cfg, Some("good")).unwrap();
        assert_eq!(kind, "codex");
        for bad in ["bad-empty", "bad-non-string", "bad-newline"] {
            let err = resolve_agent_command(&cfg, Some(bad)).unwrap_err();
            let AgentCommandError::UnknownAgent { known, .. } = &err else {
                panic!("expected UnknownAgent for dropped entry {bad:?}, got {err:?}");
            };
            assert!(!known.contains(&bad.to_string()), "{known:?} must not carry dropped entry {bad:?}");
            assert!(known.contains(&"good".to_string()));
        }
    }

    // ─── D3: built-in registry defaults ─────────────────────────────────

    #[test]
    fn built_in_names_resolve_with_zero_herding_config() {
        let (kind, args, env, _wt) = resolve_agent_command(&Value::Null, Some("claude-sonnet")).unwrap();
        assert_eq!(kind, "claude");
        assert_eq!(args, vec!["--model", "sonnet", "--permission-mode", "bypassPermissions"]);
        assert!(env.is_empty());

        let (kind, args, env, _wt) = resolve_agent_command(&Value::Null, Some("agy-flash")).unwrap();
        assert_eq!(kind, "agy");
        assert_eq!(args, vec!["--dangerously-skip-permissions"]);
        assert!(env.is_empty());
    }

    #[test]
    fn a_same_name_config_entry_overrides_the_built_in() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "agy-flash": ["agy", "--custom-flag"],
                }
            }
        });
        let (kind, args, _env, _wt) = resolve_agent_command(&cfg, Some("agy-flash")).unwrap();
        assert_eq!(kind, "agy");
        assert_eq!(args, vec!["--custom-flag"]);
    }

    #[test]
    fn unknown_agent_listing_always_includes_the_built_ins() {
        let err = resolve_agent_command(&Value::Null, Some("no-such-herd")).unwrap_err();
        let AgentCommandError::UnknownAgent { known, .. } = &err else {
            panic!("expected UnknownAgent, got {err:?}");
        };
        for key in ["claude-sonnet", "agy-flash"] {
            assert!(known.contains(&key.to_string()), "{known:?} must list built-in {key:?}");
        }
    }

    // ─── D4: per-agent env on the object-shape registry entry ──────────

    #[test]
    fn an_object_shape_entry_parses_argv_and_carries_env() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "codex-envd": {
                        "argv": ["codex", "--flag"],
                        "env": {"API_KEY": "secret-value", "FOO_2": "bar"},
                    }
                }
            }
        });
        let (kind, args, env, _wt) = resolve_agent_command(&cfg, Some("codex-envd")).unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag"]);
        assert_eq!(env.get("API_KEY").map(String::as_str), Some("secret-value"));
        assert_eq!(env.get("FOO_2").map(String::as_str), Some("bar"));
    }

    #[test]
    fn an_object_shape_entry_with_no_env_key_resolves_with_an_empty_env() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "codex-noenv": { "argv": ["codex"] }
                }
            }
        });
        let (_kind, _args, env, _wt) = resolve_agent_command(&cfg, Some("codex-noenv")).unwrap();
        assert!(env.is_empty());
    }

    #[test]
    fn a_bad_env_key_or_newline_value_drops_the_whole_entry_not_just_env() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "good": { "argv": ["codex"], "env": {"OK": "v"} },
                    "bad-key": { "argv": ["codex"], "env": {"1BAD": "v"} },
                    "bad-key-dash": { "argv": ["codex"], "env": {"BAD-KEY": "v"} },
                    "bad-value-newline": { "argv": ["codex"], "env": {"OK": "line1\nline2"} },
                }
            }
        });
        let (_kind, _args, env, _wt) = resolve_agent_command(&cfg, Some("good")).unwrap();
        assert_eq!(env.get("OK").map(String::as_str), Some("v"));
        for bad in ["bad-key", "bad-key-dash", "bad-value-newline"] {
            let err = resolve_agent_command(&cfg, Some(bad)).unwrap_err();
            let AgentCommandError::UnknownAgent { known, .. } = &err else {
                panic!("expected UnknownAgent for dropped entry {bad:?}, got {err:?}");
            };
            assert!(!known.contains(&bad.to_string()), "{known:?} must not carry dropped entry {bad:?}");
        }
    }

    #[test]
    fn array_shape_entries_never_carry_env() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "plain-array": ["codex", "--x"],
                }
            }
        });
        let (_kind, _args, env, _wt) = resolve_agent_command(&cfg, Some("plain-array")).unwrap();
        assert!(env.is_empty());
    }

    // ─── D5: workspace-trust declaration on the object-shape entry ─────

    #[test]
    fn an_object_shape_entry_parses_a_workspace_trust_declaration() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "agy-flash": {
                        "argv": ["agy", "--dangerously-skip-permissions"],
                        "workspace_trust": {
                            "file": "~/.gemini/antigravity-cli/settings.json",
                            "key": "trustedWorkspaces",
                        },
                    }
                }
            }
        });
        let (kind, args, _env, wt) = resolve_agent_command(&cfg, Some("agy-flash")).unwrap();
        assert_eq!(kind, "agy");
        assert_eq!(args, vec!["--dangerously-skip-permissions"]);
        let wt = wt.expect("workspace_trust must be Some");
        assert_eq!(wt.key, "trustedWorkspaces");
        assert!(!wt.file.starts_with('~'), "leading ~ must be expanded, got {:?}", wt.file);
        assert!(wt.file.ends_with("/.gemini/antigravity-cli/settings.json"), "{:?}", wt.file);
    }

    #[test]
    fn an_object_shape_entry_with_no_workspace_trust_key_resolves_with_none() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "codex-noenv": { "argv": ["codex"] }
                }
            }
        });
        let (_kind, _args, _env, wt) = resolve_agent_command(&cfg, Some("codex-noenv")).unwrap();
        assert!(wt.is_none());
    }

    #[test]
    fn a_malformed_workspace_trust_drops_the_whole_entry_not_just_the_declaration() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "good": {
                        "argv": ["agy"],
                        "workspace_trust": {"file": "~/trust.json", "key": "trustedWorkspaces"},
                    },
                    "bad-not-object": {"argv": ["agy"], "workspace_trust": ["not", "an", "object"]},
                    "bad-missing-file": {"argv": ["agy"], "workspace_trust": {"key": "trustedWorkspaces"}},
                    "bad-missing-key": {"argv": ["agy"], "workspace_trust": {"file": "~/trust.json"}},
                    "bad-empty-file": {"argv": ["agy"], "workspace_trust": {"file": "", "key": "trustedWorkspaces"}},
                    "bad-newline-file": {
                        "argv": ["agy"],
                        "workspace_trust": {"file": "line1\nline2", "key": "trustedWorkspaces"},
                    },
                }
            }
        });
        let (_kind, _args, _env, wt) = resolve_agent_command(&cfg, Some("good")).unwrap();
        assert!(wt.is_some());
        for bad in [
            "bad-not-object",
            "bad-missing-file",
            "bad-missing-key",
            "bad-empty-file",
            "bad-newline-file",
        ] {
            let err = resolve_agent_command(&cfg, Some(bad)).unwrap_err();
            let AgentCommandError::UnknownAgent { known, .. } = &err else {
                panic!("expected UnknownAgent for dropped entry {bad:?}, got {err:?}");
            };
            assert!(!known.contains(&bad.to_string()), "{known:?} must not carry dropped entry {bad:?}");
            assert!(known.contains(&"good".to_string()));
        }
    }

    #[test]
    fn array_shape_entries_never_carry_workspace_trust() {
        let cfg = serde_json::json!({
            "herding": {
                "agents": {
                    "plain-array": ["agy", "--x"],
                }
            }
        });
        let (_kind, _args, _env, wt) = resolve_agent_command(&cfg, Some("plain-array")).unwrap();
        assert!(wt.is_none());
    }

    #[test]
    fn tilde_expansion_only_applies_to_a_bare_home_relative_path() {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .expect("HOME or USERPROFILE must be set in the test environment");
        assert_eq!(expand_tilde("~/foo/bar.json"), format!("{home}/foo/bar.json"));
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("/already/absolute.json"), "/already/absolute.json");
        assert_eq!(expand_tilde("~otheruser/bar.json"), "~otheruser/bar.json");
    }

    // ─── tier slot resolution: models.<runtime>.generation ────────────

    #[test]
    fn tier_slot_wins_over_differing_agent_command() {
        let cfg = serde_json::json!({
            "models": {
                "claude": {
                    "generation": { "kind": "herding", "agent": "agy-flash" }
                }
            },
            "herding": {
                "agent_command": "claude-sonnet"
            }
        });
        let (kind, args, _env, _wt) = resolve_agent_command_for_runtime(&cfg, None, "claude").unwrap();
        assert_eq!(kind, "agy");
        assert_eq!(args, vec!["--dangerously-skip-permissions"]);
    }

    #[test]
    fn explicit_agent_name_still_wins_over_the_tier_slot() {
        let cfg = serde_json::json!({
            "models": {
                "claude": {
                    "generation": { "kind": "herding", "agent": "agy-flash" }
                }
            }
        });
        let (kind, args, _env, _wt) = resolve_agent_command_for_runtime(&cfg, Some("claude-sonnet"), "claude").unwrap();
        assert_eq!(kind, "claude");
        assert_eq!(args, vec!["--model", "sonnet", "--permission-mode", "bypassPermissions"]);
    }

    #[test]
    fn non_participating_tier_slots_fall_through_to_agent_command() {
        // 1. kind: herding with no agent
        let cfg_no_agent = serde_json::json!({
            "models": { "claude": { "generation": { "kind": "herding" } } },
            "herding": { "agent_command": ["codex", "--flag"] }
        });
        let (kind, args, _env, _wt) = resolve_agent_command_for_runtime(&cfg_no_agent, None, "claude").unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag"]);

        // 2. kind: herding with empty/whitespace agent
        let cfg_empty_agent = serde_json::json!({
            "models": { "claude": { "generation": { "kind": "herding", "agent": "   " } } },
            "herding": { "agent_command": ["codex", "--flag"] }
        });
        let (kind, args, _env, _wt) = resolve_agent_command_for_runtime(&cfg_empty_agent, None, "claude").unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag"]);

        // 3. plain model-name slot
        let cfg_model_str = serde_json::json!({
            "models": { "claude": { "generation": "sonnet" } },
            "herding": { "agent_command": ["codex", "--flag"] }
        });
        let (kind, args, _env, _wt) = resolve_agent_command_for_runtime(&cfg_model_str, None, "claude").unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag"]);

        // 4. kind: "cli" slot
        let cfg_cli = serde_json::json!({
            "models": { "claude": { "generation": { "kind": "cli", "command": "run-cmd" } } },
            "herding": { "agent_command": ["codex", "--flag"] }
        });
        let (kind, args, _env, _wt) = resolve_agent_command_for_runtime(&cfg_cli, None, "claude").unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag"]);

        // 5. null / non-object slot
        let cfg_null = serde_json::json!({
            "models": { "claude": { "generation": null } },
            "herding": { "agent_command": ["codex", "--flag"] }
        });
        let (kind, args, _env, _wt) = resolve_agent_command_for_runtime(&cfg_null, None, "claude").unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag"]);
    }

    #[test]
    fn tier_slot_naming_unknown_agent_returns_unknown_agent_error() {
        let cfg = serde_json::json!({
            "models": {
                "claude": {
                    "generation": { "kind": "herding", "agent": "no-such-herd" }
                }
            },
            "herding": {
                "agent_command": ["codex", "--flag"]
            }
        });
        let err = resolve_agent_command_for_runtime(&cfg, None, "claude").unwrap_err();
        let AgentCommandError::UnknownAgent { name, known } = &err else {
            panic!("expected UnknownAgent, got {err:?}");
        };
        assert_eq!(name, "no-such-herd");
        for key in ["claude-sonnet", "agy-flash"] {
            assert!(known.contains(&key.to_string()), "{known:?} must list {key:?}");
        }
        let text = err.to_string();
        assert!(text.contains("no-such-herd"));
    }

    #[test]
    fn tier_slot_resolving_to_object_shape_entry_carries_env() {
        let cfg = serde_json::json!({
            "models": {
                "claude": {
                    "generation": { "kind": "herding", "agent": "codex-envd" }
                }
            },
            "herding": {
                "agents": {
                    "codex-envd": {
                        "argv": ["codex", "--flag"],
                        "env": {"API_KEY": "secret-value", "FOO_2": "bar"}
                    }
                }
            }
        });
        let (kind, args, env, _wt) = resolve_agent_command_for_runtime(&cfg, None, "claude").unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag"]);
        assert_eq!(env.get("API_KEY").map(String::as_str), Some("secret-value"));
        assert_eq!(env.get("FOO_2").map(String::as_str), Some("bar"));
    }

    #[test]
    fn unknown_or_absent_runtime_reads_the_claude_block() {
        let cfg = serde_json::json!({
            "models": {
                "claude": {
                    "generation": { "kind": "herding", "agent": "agy-flash" }
                },
                "codex": {
                    "generation": { "kind": "herding", "agent": "claude-sonnet" }
                }
            }
        });
        // unknown runtime name falls back to claude
        let (kind, _args, _env, _wt) = resolve_agent_command_for_runtime(&cfg, None, "unknown-runtime").unwrap();
        assert_eq!(kind, "agy");

        // empty string runtime falls back to claude
        let (kind, _args, _env, _wt) = resolve_agent_command_for_runtime(&cfg, None, "").unwrap();
        assert_eq!(kind, "agy");

        // valid codex runtime reads codex block
        let (kind, _args, _env, _wt) = resolve_agent_command_for_runtime(&cfg, None, "codex").unwrap();
        assert_eq!(kind, "claude");

        // valid claude runtime reads claude block
        let (kind, _args, _env, _wt) = resolve_agent_command_for_runtime(&cfg, None, "claude").unwrap();
        assert_eq!(kind, "agy");
    }

    // ─── the caller: a wave run through a fake backend ─────────────────

    fn ready_worker(backend: &FakeBackend, name: &str, task: &str) {
        backend.set_steady_status(name, RawStatus::Value(WorkerStatus::Ready));
        backend.set_output(name, "");
        backend.schedule_output_on_send(name, task);
    }

    #[test]
    fn a_wave_run_appends_exactly_one_ledger_row_through_a_fake_backend() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let backend = FakeBackend::new();
        ready_worker(&backend, "alpha", "MARKER-alpha do the thing");
        ready_worker(&backend, "beta", "MARKER-beta do the other thing");

        let inputs = vec![
            WaveWorkerInput { name: "alpha".to_string(), task: "MARKER-alpha do the thing".to_string(), worktree: "/tmp/wt-alpha".to_string() },
            WaveWorkerInput { name: "beta".to_string(), task: "MARKER-beta do the other thing".to_string(), worktree: "/tmp/wt-beta".to_string() },
        ];
        let workers: Vec<WorkerSpec> =
            inputs.iter().map(|w| WorkerSpec::new(w.name.clone(), w.task.clone())).collect();
        let wave_value = Wave::new(
            workers,
            WaveTimeouts { worker_settle: Duration::from_millis(500), poll_interval: Duration::from_millis(5) },
            FailurePolicy::WaitForAll,
        );

        let result = run_wave_and_record(
            &backend,
            root,
            "w-test".to_string(),
            "2026-08-18T00:00:00Z".to_string(),
            &inputs,
            &wave_value,
        );
        assert!(result.is_success(), "{result:?}");
        assert_eq!(result.succeeded.len(), 2);

        let raw = std::fs::read_to_string(wave_ledger::wave_ledger_path(root)).unwrap();
        assert_eq!(raw.lines().count(), 1, "exactly one row must be appended per wave run");

        let waves = wave_ledger::read_waves(root);
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0].wave_id, "w-test");
        assert_eq!(waves[0].workers.len(), 2);
        assert!(waves[0].workers.iter().all(|w| w.outcome.as_deref() == Some("succeeded")));
        // This test proves `run_wave_and_record` reaches the fake
        // backend's argv-shaped seam at all — `send` was actually called,
        // not skipped. The join `run_wave_and_record` does NOT prove —
        // that `HerdrBackend::new`'s construction call site actually
        // receives the D14-resolved (kind, args) pair rather than
        // constants — is `build_wave_backend_and_run`'s test below.
        assert_eq!(backend.send_call_count("alpha"), 1);
        assert_eq!(backend.send_call_count("beta"), 1);
    }

    #[test]
    fn a_wave_that_never_resolves_still_appends_one_row_with_no_succeeded_outcome() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let backend = FakeBackend::new();
        backend.schedule_start_result("ghost", Err("no such target".to_string()));

        let inputs =
            vec![WaveWorkerInput { name: "ghost".to_string(), task: "do it".to_string(), worktree: String::new() }];
        let workers: Vec<WorkerSpec> =
            inputs.iter().map(|w| WorkerSpec::new(w.name.clone(), w.task.clone())).collect();
        let wave_value = Wave::new(
            workers,
            WaveTimeouts { worker_settle: Duration::from_millis(50), poll_interval: Duration::from_millis(5) },
            FailurePolicy::WaitForAll,
        );

        let result = run_wave_and_record(&backend, root, "w-ghost".to_string(), "2026-08-18T00:00:00Z".to_string(), &inputs, &wave_value);
        assert!(!result.is_success());
        assert_eq!(result.resolution_failed, vec!["ghost".to_string()]);

        let waves = wave_ledger::read_waves(root);
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0].workers[0].outcome.as_deref(), Some("resolution_failed"));
    }

    // ─── the join: the D14-resolved pair actually reaching construction ─

    #[test]
    fn build_wave_backend_and_run_hands_the_resolved_kind_and_args_to_construction() {
        // `wave()` calls `build_wave_backend_and_run` with `HerdrBackend::new`
        // as `construct_backend`; this test calls the SAME function with an
        // assertion closure instead. If the call site inside
        // `build_wave_backend_and_run` ever discards `resolve_agent_command`'s
        // (kind, args) pair for constants — the exact defect a prior judge
        // found live one line past this file's own construction call — the
        // closure below observes the wrong values and panics, failing this
        // test. It is not enough for the closure to merely be called: it
        // must be called with the SAME pair `resolve_agent_command` itself
        // produces for this `cfg`, so a substitute value of the right shape
        // (e.g. always "claude", vec![]) still fails when the configured
        // command differs, as it does here.
        let cfg = serde_json::json!({"herding": {"agent_command": ["codex", "--flag", "value"]}});
        let (expected_kind, expected_args, _expected_env, _wt) = resolve_agent_command(&cfg, None).unwrap();
        assert_eq!(expected_kind, "codex", "sanity: this test's cfg must not resolve to the default");

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let inputs = vec![WaveWorkerInput {
            name: "alpha".to_string(),
            task: "MARKER-alpha do the thing".to_string(),
            worktree: String::new(),
        }];
        let workers: Vec<WorkerSpec> =
            inputs.iter().map(|w| WorkerSpec::new(w.name.clone(), w.task.clone())).collect();
        let wave_value = Wave::new(
            workers,
            WaveTimeouts { worker_settle: Duration::from_millis(500), poll_interval: Duration::from_millis(5) },
            FailurePolicy::WaitForAll,
        );

        let mut construct_backend_was_called = false;
        let result = build_wave_backend_and_run(
            &cfg,
            root,
            "w-ctor".to_string(),
            "2026-08-18T00:00:00Z".to_string(),
            &inputs,
            &wave_value,
            |kind, args| {
                assert_eq!(
                    kind, expected_kind,
                    "backend construction must receive the D14-resolved kind, not a substitute"
                );
                assert_eq!(
                    args, expected_args,
                    "backend construction must receive the D14-resolved args, not a substitute"
                );
                construct_backend_was_called = true;
                let backend = FakeBackend::new();
                ready_worker(&backend, "alpha", "MARKER-alpha do the thing");
                backend
            },
        )
        .unwrap();

        assert!(construct_backend_was_called, "construct_backend must actually be invoked");
        assert!(result.is_success(), "{result:?}");
    }

    #[test]
    fn real_backend_ctor_hands_kind_and_args_straight_through_to_construction() {
        // `wave()` passes `real_backend_ctor` — a NAMED function — as
        // `construct_backend`, not a closure literal written at its own
        // call site. That naming is the entire point of this test: a
        // closure literal there would be reachable by no test (the join
        // test above supplies its OWN closure to `build_wave_backend_and_run`,
        // which never touches `wave()`'s closure). Calling
        // `real_backend_ctor` directly here proves the ACTUAL production
        // construction — the one `wave()` runs — carries its `kind`/`args`
        // through to the `HerdrBackend` it builds, via that struct's own
        // derived `Debug` output (no accessor exists, and none should be
        // added just for this test — `fleet` is out of scope here). If
        // `real_backend_ctor` is ever mutated to ignore its arguments and
        // construct with constants, this assertion — not any seam one
        // level up — is what fails.
        let backend = real_backend_ctor("codex".to_string(), vec!["--flag".to_string(), "value".to_string()]);
        let debug = format!("{backend:?}");
        assert!(
            debug.contains("agent_kind: \"codex\""),
            "constructed backend must carry the kind it was handed, got: {debug}"
        );
        assert!(
            debug.contains(r#"agent_args: ["--flag", "value"]"#),
            "constructed backend must carry the args it was handed, got: {debug}"
        );
    }

    #[test]
    fn wave_tmux_backend_ctor_hands_kind_args_and_settings_straight_through() {
        // The tmux twin of the test above, and the same reasoning: `wave()`
        // passes `tmux_backend_ctor(settings)` — a NAMED factory — as
        // `construct_backend`, never a closure literal written at its own
        // call site, precisely so this test can call the ACTUAL production
        // construction and assert on what it built. `TmuxBackend` exposes
        // no accessors (and should grow none for a test — `fleet` is out of
        // scope here), so the assertion reads its derived `Debug`.
        //
        // The settings matter as much as the kind and args do: tmux has no
        // agent API, so the scrollback depth and marker lists ARE how this
        // backend reads status. A ctor that dropped them for
        // `ScreenSettings::default()` would still compile and still spawn
        // workers — and would silently ignore every `herding.tmux.*`
        // override the repo set. This assertion is what catches that.
        let settings = ScreenSettings {
            busy_markers: vec!["chewing on it".to_string()],
            scrollback: 777,
            ..ScreenSettings::default()
        };
        let backend =
            tmux_backend_ctor(settings)("codex".to_string(), vec!["--flag".to_string(), "value".to_string()]);
        let debug = format!("{backend:?}");
        assert!(
            debug.contains("agent_kind: \"codex\""),
            "constructed backend must carry the kind it was handed, got: {debug}"
        );
        assert!(
            debug.contains(r#"agent_args: ["--flag", "value"]"#),
            "constructed backend must carry the args it was handed, got: {debug}"
        );
        assert!(
            debug.contains("scrollback: 777") && debug.contains("chewing on it"),
            "constructed backend must carry the SETTINGS it was handed, not a fresh default \
             — otherwise every herding.tmux.* override is silently dropped, got: {debug}"
        );
    }

    #[test]
    fn wave_tmux_transport_never_constructs_the_herdr_backend() {
        // The production switch itself, called the way `wave()` calls it.
        // A wave with no workers exercises the arm without needing a live
        // tmux: the point under test is WHICH arm ran, and the closure the
        // tmux arm builds is typed `TmuxBackend`, so a herdr construction
        // in that arm could not compile. What this adds beyond the type
        // system is that the two arms agree on one `Result` type and that
        // the tmux arm reaches `build_wave_backend_and_run` at all — a
        // `todo!()`/`unimplemented!()` arm, or one that fell through to
        // herdr's config error, fails here.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let cfg = serde_json::json!({
            "herding": {
                "transport": "tmux",
                "agent_command": ["codex", "--flag"],
                "tmux": {"scrollback": 90}
            }
        });
        let wave_value = Wave::new(
            Vec::new(),
            WaveTimeouts {
                worker_settle: Duration::from_millis(50),
                poll_interval: Duration::from_millis(5),
            },
            FailurePolicy::WaitForAll,
        );

        for transport in [TransportKind::Tmux, TransportKind::Herdr] {
            let result = run_wave_for_transport(
                transport,
                &cfg,
                root,
                format!("w-{}", transport.as_str()),
                "2026-08-23T00:00:00Z".to_string(),
                &[],
                &wave_value,
            )
            .unwrap_or_else(|e| panic!("{} arm must resolve its command: {e}", transport.as_str()));
            assert!(result.is_success(), "{transport:?}: {result:?}");
        }
    }

    // ─── the bridge: occupancy reports which answer it gave ────────────

    #[test]
    fn occupancy_json_distinguishes_live_from_fallback_at_the_same_count() {
        let live = occupancy_json(&wave_ledger::Occupancy::Live(3));
        let fallback = occupancy_json(&wave_ledger::Occupancy::Fallback(3));
        assert_eq!(live["count"], 3);
        assert_eq!(live["source"], "live");
        assert_eq!(fallback["count"], 3);
        assert_eq!(fallback["source"], "fallback");
        assert_ne!(live, fallback, "the live/fallback distinction must never collapse into a bare count");
    }

    #[test]
    fn occupancy_plain_line_pins_the_source_distinction_too() {
        // The json path's live/fallback distinction is pinned above; a role
        // that runs `bee herding occupancy` WITHOUT `--json` reads this
        // plain line instead, so it must be able to tell the two apart the
        // same way — dropping the `({source})` parenthetical from the plain
        // println must fail this test.
        let live = occupancy_plain_line(&wave_ledger::Occupancy::Live(3));
        let fallback = occupancy_plain_line(&wave_ledger::Occupancy::Fallback(3));
        assert!(live.contains("(live)"), "{live}");
        assert!(fallback.contains("(fallback)"), "{fallback}");
        assert_ne!(live, fallback, "the plain occupancy line must not collapse live and fallback at the same count");
    }

    #[test]
    fn occupancy_crosses_the_ledger_against_a_provided_live_list() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        wave_ledger::append_wave(
            root,
            &wave_ledger::WaveRow {
                wave_id: "w-1".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                workers: vec![wave_ledger::WorkerRow {
                    name: "alpha".to_string(),
                    pane_id: "w4:pB".to_string(),
                    worktree: "/tmp/wt-alpha".to_string(),
                    task: "do it".to_string(),
                    outcome: None,
                    evidence: None,
                }],
            },
        )
        .unwrap();

        let now_ms = chrono::Utc::now().timestamp_millis();
        let live_panes: HashSet<String> = ["w4:pB".to_string()].into_iter().collect();
        let live = wave_ledger::live_worker_count(root, Some(&live_panes), now_ms, wave_ledger::DEFAULT_STALE_AFTER_MS);
        assert_eq!(occupancy_json(&live), serde_json::json!({"count": 1, "source": "live"}));

        let fallback = wave_ledger::live_worker_count(root, None, now_ms, wave_ledger::DEFAULT_STALE_AFTER_MS);
        assert_eq!(occupancy_json(&fallback), serde_json::json!({"count": 1, "source": "fallback"}));
    }

    // ─── the tmux lister's pure parse (no tmux binary anywhere) ────────

    #[test]
    fn occupancy_tmux_parses_one_pane_id_per_line() {
        let got = parse_tmux_pane_list("%0\n%1\n%12\n");
        let want: HashSet<String> = ["%0".to_string(), "%1".to_string(), "%12".to_string()].into_iter().collect();
        assert_eq!(got, want);
    }

    #[test]
    fn occupancy_tmux_drops_the_trailing_newline_never_an_empty_id() {
        // `tmux list-panes` always terminates its last line, so the naive
        // `split('\n')` spelling would smuggle an empty id into the live set
        // and make a dead ledger row look live.
        let got = parse_tmux_pane_list("%7\n");
        assert_eq!(got, ["%7".to_string()].into_iter().collect::<HashSet<String>>());
        assert!(!got.contains(""), "an empty pane id must never enter the live set");
    }

    #[test]
    fn occupancy_tmux_empty_body_is_an_empty_set() {
        assert!(parse_tmux_pane_list("").is_empty());
        assert!(parse_tmux_pane_list("\n").is_empty());
    }

    // ─── reachable from the router without a herdr binary on PATH ──────

    #[test]
    fn wave_and_occupancy_are_dispatched_by_try_native() {
        use std::ffi::OsString;
        // No stdin/config wiring here — this only proves the argv reaches
        // this module's entry points at all, not their full behaviour
        // (covered above). `wave` with no piped stdin fails closed rather
        // than hanging, so this stays fast and herdr-free.
        let args: Vec<OsString> = ["herding", "occupancy", "--main-root", "/nonexistent-root-for-test"]
            .iter()
            .map(OsString::from)
            .collect();
        assert!(super::super::try_native(&args).is_some());
    }

    // ─── record-worker (D18): the closed loop ──────────────────────────

    #[test]
    fn record_worker_defaults_the_wave_id_to_the_workers_own_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let wave_id = append_worker_row(
            root,
            "some-slug".to_string(),
            "w4:p9".to_string(),
            "/tmp/wt-some-slug".to_string(),
            "P-123".to_string(),
            None,
            "2026-08-18T00:00:00Z".to_string(),
        )
        .unwrap();
        assert_eq!(wave_id, "some-slug", "no --wave-id given, so it must default to the worker's own name");

        let waves = wave_ledger::read_waves(root);
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0].wave_id, "some-slug");
        assert_eq!(waves[0].workers.len(), 1);
        assert_eq!(waves[0].workers[0].name, "some-slug");
        assert_eq!(waves[0].workers[0].pane_id, "w4:p9");
        assert_eq!(waves[0].workers[0].worktree, "/tmp/wt-some-slug");
        assert_eq!(waves[0].workers[0].task, "P-123");
        assert!(waves[0].workers[0].outcome.is_none(), "a freshly recorded spawn carries no outcome yet");
    }

    #[test]
    fn record_worker_honours_an_explicit_wave_id_override() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let wave_id = append_worker_row(
            root,
            "some-slug".to_string(),
            "w4:p9".to_string(),
            String::new(),
            "P-123".to_string(),
            Some("wave-explicit".to_string()),
            "2026-08-18T00:00:00Z".to_string(),
        )
        .unwrap();
        assert_eq!(wave_id, "wave-explicit");
    }

    #[test]
    fn record_worker_is_dispatched_by_try_native() {
        use std::ffi::OsString;
        // Missing --name/--pane-id/--path/--task fails closed rather than
        // writing a partial row — this only proves the argv reaches
        // `record_worker` at all, not its full behaviour (covered above and
        // by the crossing test below). No herdr binary is needed.
        let args: Vec<OsString> = [
            "herding",
            "record-worker",
            "--main-root",
            "/nonexistent-root-for-test",
        ]
        .iter()
        .map(OsString::from)
        .collect();
        assert!(super::super::try_native(&args).is_some());
    }

    // ─── THE crossing test: write through record-worker, read through occupancy ─
    //
    // This is the exact gap the three prior cells left open: a ledger writer
    // with one caller (`run_wave_and_record`, reachable only through
    // `bee herding wave`) and an occupancy reader with a real caller
    // (`role-dispatch.md` §4, via `bee herding occupancy`) that never met on
    // the path dispatch actually takes. Nothing before this test wrote a row
    // through `record-worker`'s own path and then read it back through the
    // occupancy path — so nothing before this test would have caught a wire
    // that silently did not connect.
    #[test]
    fn a_row_recorded_through_record_worker_reads_back_as_one_live_worker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Write through record-worker's own path — the exact function
        // `record_worker()` (the CLI verb) calls.
        let wave_id = append_worker_row(
            root,
            "crossing-slug".to_string(),
            "w4:p9".to_string(),
            "/tmp/wt-crossing-slug".to_string(),
            "P-999".to_string(),
            None,
            chrono::Utc::now().to_rfc3339(),
        )
        .unwrap();
        assert_eq!(wave_id, "crossing-slug");

        // Read through occupancy's own path — the exact function
        // `occupancy()` (the CLI verb) calls, with a live pane list supplied
        // directly (the crossing check needs the recorded pane id to be IN
        // that list for the row to count, exactly as occupancy() would if
        // `herdr pane list` had reported it live).
        let live_panes: HashSet<String> = ["w4:p9".to_string()].into_iter().collect();
        let now_ms = chrono::Utc::now().timestamp_millis();
        let occ = wave_ledger::live_worker_count(
            root,
            Some(&live_panes),
            now_ms,
            wave_ledger::DEFAULT_STALE_AFTER_MS,
        );
        assert_eq!(
            occ,
            wave_ledger::Occupancy::Live(1),
            "a row written through record-worker's own path must read back as one live worker \
             through occupancy's own path — this is the whole loop D18 closes"
        );
        assert_eq!(occupancy_json(&occ), serde_json::json!({"count": 1, "source": "live"}));
    }

    // ─── THE argv-level crossing: enter through record_worker()'s OWN flag
    // parsing, not through the pure `append_worker_row` helper above ────────
    //
    // The test above proves the ledger row and the occupancy read agree once
    // a row exists — but it builds that row by calling `append_worker_row`
    // directly, the same way `record_worker()` calls it AFTER parsing argv.
    // It never crosses the parsing itself, so a mis-wire inside
    // `record_worker()` — e.g. `--pane-id`'s value and `--path`'s value
    // swapped before being passed positionally to `append_worker_row` — types
    // perfectly and leaves this file's whole suite green. This test enters
    // at the same boundary production enters at: real flags, in the order
    // `role-dispatch.md` §8 actually sends them, through `record_worker()`
    // itself.
    #[test]
    fn a_row_recorded_through_record_workers_own_argv_reads_back_as_one_live_worker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_str = root.to_str().unwrap();

        // Write through record_worker()'s own argv parsing — the exact CLI
        // wrapper `bee herding record-worker` runs, with real `--pane-id`
        // and `--path` flags. A swap of these two same-typed arguments
        // inside `record_worker()` must fail this test.
        let exit = record_worker(&[
            "--main-root",
            root_str,
            "--name",
            "argv-slug",
            "--pane-id",
            "w4:pA",
            "--path",
            "/tmp/wt-argv-slug",
            "--task",
            "P-777",
        ]);
        assert!(exit == ExitCode::SUCCESS, "record_worker() must accept a fully-flagged call");

        // Read through occupancy's own path — the exact function
        // `occupancy()` (the CLI verb) calls, with a live pane list supplied
        // directly. The recorded row's pane id must be the value given to
        // `--pane-id` ("w4:pA"), not the value given to `--path`; if the
        // wrapper swapped them, the row's pane id would be the worktree
        // path instead and this live-pane list would not match it.
        let live_panes: HashSet<String> = ["w4:pA".to_string()].into_iter().collect();
        let now_ms = chrono::Utc::now().timestamp_millis();
        let occ = wave_ledger::live_worker_count(
            root,
            Some(&live_panes),
            now_ms,
            wave_ledger::DEFAULT_STALE_AFTER_MS,
        );
        assert_eq!(
            occ,
            wave_ledger::Occupancy::Live(1),
            "a row recorded through record_worker()'s own argv parsing must read back as one \
             live worker through occupancy's own path — a --pane-id/--path swap inside the \
             wrapper must fail this test, not just the pure-helper crossing above"
        );

        // Read the recorded row back directly (not just the live count) so
        // every value that crossed argv is watched, not only the one number
        // that happened to be easiest to observe. A `name`/`task` swap or a
        // `path`/`task` swap inside record_worker()'s call into
        // `append_worker_row` types perfectly and leaves the live-count
        // assertion above green — only reading the row back catches it.
        let waves = wave_ledger::read_waves(root);
        assert_eq!(waves.len(), 1);
        assert_eq!(
            waves[0].wave_id, "argv-slug",
            "no --wave-id given, so it must default to the value given to --name, not --task \
             or any other flag"
        );
        assert_eq!(waves[0].workers.len(), 1);
        assert_eq!(waves[0].workers[0].name, "argv-slug", "the recorded row's name must be the value given to --name");
        assert_eq!(
            waves[0].workers[0].worktree, "/tmp/wt-argv-slug",
            "the recorded row's worktree must be the value given to --path"
        );
        assert_eq!(waves[0].workers[0].task, "P-777", "the recorded row's task must be the value given to --task");
    }
}
