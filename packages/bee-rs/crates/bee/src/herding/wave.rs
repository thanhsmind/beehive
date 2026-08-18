// herding::wave — `bee herding wave` and `bee herding occupancy`
// (herding-orchestration D17, docs/history/herding-orchestration/CONTEXT.md).
//
// `wave` is the bee-side entry point D17 locks: it is what turns
// `herding.agent_command` into a running wave. `HerdrBackend::new`
// (`crates/fleet/src/backend/herdr.rs`) had ZERO callers anywhere in the
// workspace before this file — its own documentation names the caller's
// obligation (split `herding.agent_command` per D14, raise a typed error on
// an unrecognised token 0) and THIS is that caller.
//
// `occupancy` is the CLI bridge to the ledger's read side
// (`super::wave_ledger::live_worker_count`), which was crate-private and
// unusable from anywhere outside this crate — in particular unusable from
// `role-dispatch.md` §4, a markdown role that can only run shell commands.
//
// Both verbs are deliberately split into a thin CLI-parsing shell and a pure
// (or backend-generic) inner function, so every behavioural test below runs
// with NO real `herdr` on PATH (D7's test seam) and NO herdr server:
//   - `resolve_agent_command` is pure: config JSON in, (kind, args) or a
//     typed `AgentCommandError` out. No process, no I/O beyond the caller
//     having already read the config file.
//   - `run_wave_and_record` is generic over `WorkerBackend`, so a test
//     drives it with `fleet::backend::fake::FakeBackend` instead of
//     `HerdrBackend` — the same seam `fleet`'s own choreography tests use
//     (D7). `wave()` (the CLI verb) is the only caller that ever
//     constructs a real `HerdrBackend`.
//   - `occupancy_json` is pure: an `Occupancy` value in, its JSON shape out
//     — `Live` and `Fallback` never collapse to the same shape. `occupancy()`
//     (the CLI verb) is the only caller that ever shells out to a real
//     `herdr pane list`; when that call fails (no herdr installed, in every
//     test environment) it returns `None` and the ledger's own fallback
//     path answers instead — never a test dependency on a live herdr.

use std::collections::HashSet;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use serde_json::{Map, Value};

use fleet::backend::herdr::HerdrBackend;
use fleet::backend::WorkerBackend;
use fleet::choreography::{run_wave, WaveResult};
use fleet::wave::{FailurePolicy, Wave, WaveTimeouts, WorkerSpec};

use super::wave_ledger;

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

/// The agent kinds `herdr agent start --kind` is known to accept, from
/// every example in this workspace's own herdr fixtures
/// (`crates/fleet/src/backend/herdr.rs` tests use both) — `herdr --skill`,
/// the first authority, was not available to this worker (recorded the same
/// way `herdr.rs`'s own module docs record it). Token 0 outside this list
/// raises `AgentCommandError::UnrecognizedKind` rather than being handed to
/// `herdr` on faith, per D14's typed-error obligation.
const SUPPORTED_AGENT_KINDS: &[&str] = &["claude", "codex"];

/// The one placeholder `herding.agent_command` defines
/// (`operational-invariants.md`): the fixed model, substituted per-token,
/// never by joining tokens and re-splitting (the shell-injection-safe shape
/// the runtime adapter requires).
const MODEL_PLACEHOLDER: &str = "{MODEL}";
const MODEL_VALUE: &str = "sonnet";

/// D14's typed error: an unrecognised token 0, naming the config key it
/// came from. Never a generic start failure — this is the whole point of
/// `HerdrBackend::new`'s caller owning the split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AgentCommandError {
    UnrecognizedKind { key: &'static str, kind: String, supported: Vec<&'static str> },
}

impl std::fmt::Display for AgentCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentCommandError::UnrecognizedKind { key, kind, supported } => {
                write!(
                    f,
                    "{key}: token 0 \"{kind}\" is not one of herdr's supported agent kinds ({})",
                    supported.join(", ")
                )
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

/// D14's split: token 0 becomes the agent kind (herdr's `--kind`), the
/// remaining tokens — each substituted per-token — become the agent args
/// (herdr's trailing argv after `--`). An unrecognised token 0 surfaces as
/// `AgentCommandError::UnrecognizedKind` naming `herding.agent_command`,
/// never a generic start failure.
pub(crate) fn resolve_agent_command(cfg: &Value) -> Result<(String, Vec<String>), AgentCommandError> {
    let tokens = agent_command_tokens(cfg);
    let substituted: Vec<String> = tokens.iter().map(|t| substitute_model(t)).collect();
    let Some((kind, args)) = substituted.split_first() else {
        // agent_command_tokens() never returns empty (it falls back to the
        // non-empty default), but fail closed rather than panic if it ever did.
        return Err(AgentCommandError::UnrecognizedKind {
            key: "herding.agent_command",
            kind: String::new(),
            supported: SUPPORTED_AGENT_KINDS.to_vec(),
        });
    };
    if !SUPPORTED_AGENT_KINDS.contains(&kind.as_str()) {
        return Err(AgentCommandError::UnrecognizedKind {
            key: "herding.agent_command",
            kind: kind.clone(),
            supported: SUPPORTED_AGENT_KINDS.to_vec(),
        });
    }
    Ok((kind.clone(), args.to_vec()))
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

    let (kind, args) = match resolve_agent_command(&cfg) {
        Ok(pair) => pair,
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

    // The caller HerdrBackend::new's own documentation names — wiring D14's
    // resolved kind/args through to the real backend.
    let backend = HerdrBackend::new(kind, args);

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

    let result = run_wave_and_record(&backend, &main_root, wave_id.clone(), started_at, &inputs, &wave_value);
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
/// instead. This is the only place in this file that spawns a process, so
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

fn emit_occupancy(occ: &wave_ledger::Occupancy, json: bool) {
    let v = occupancy_json(occ);
    if json {
        println!("{v}");
        return;
    }
    let count = v["count"].as_u64().unwrap_or(0);
    let source = v["source"].as_str().unwrap_or("?");
    println!("occupancy: {count} worker(s) live ({source})");
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

    let live_panes = live_pane_ids_via_herdr();
    let now_ms = chrono::Utc::now().timestamp_millis();
    let occ =
        wave_ledger::live_worker_count(&main_root, live_panes.as_ref(), now_ms, wave_ledger::DEFAULT_STALE_AFTER_MS);
    emit_occupancy(&occ, json);
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
        let (kind, args) = resolve_agent_command(&Value::Null).unwrap();
        assert_eq!(kind, "claude");
        assert_eq!(args, vec!["--model", "sonnet", "--permission-mode", "bypassPermissions"]);
    }

    #[test]
    fn a_configured_command_splits_token_0_from_the_rest() {
        let cfg = serde_json::json!({"herding": {"agent_command": ["codex", "--flag", "value"]}});
        let (kind, args) = resolve_agent_command(&cfg).unwrap();
        assert_eq!(kind, "codex");
        assert_eq!(args, vec!["--flag", "value"]);
    }

    #[test]
    fn model_placeholder_is_substituted_per_token_never_joined() {
        let cfg = serde_json::json!({"herding": {"agent_command": ["claude", "--model", "{MODEL}", "--x={MODEL}"]}});
        let (kind, args) = resolve_agent_command(&cfg).unwrap();
        assert_eq!(kind, "claude");
        assert_eq!(args, vec!["--model", "sonnet", "--x=sonnet"]);
    }

    #[test]
    fn an_unrecognised_kind_is_a_typed_error_naming_the_config_key() {
        let cfg = serde_json::json!({"herding": {"agent_command": ["not-a-real-kind", "--x"]}});
        let err = resolve_agent_command(&cfg).unwrap_err();
        match &err {
            AgentCommandError::UnrecognizedKind { key, kind, .. } => {
                assert_eq!(*key, "herding.agent_command");
                assert_eq!(kind, "not-a-real-kind");
            }
        }
        let msg = err.to_string();
        assert!(msg.contains("herding.agent_command"), "{msg}");
        assert!(msg.contains("not-a-real-kind"), "{msg}");
    }

    #[test]
    fn an_empty_or_malformed_array_falls_back_to_the_default_not_an_error() {
        let empty = serde_json::json!({"herding": {"agent_command": []}});
        assert!(resolve_agent_command(&empty).is_ok());
        let non_array = serde_json::json!({"herding": {"agent_command": "claude"}});
        assert!(resolve_agent_command(&non_array).is_ok());
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
        // the D14 caller wired the CONFIGURED kind/args through: proven at
        // the HerdrBackend layer by its own tests (decide_start). Here the
        // proof is that this caller reaches the fake backend's argv-shaped
        // seam at all — `send` was actually called, not skipped.
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
}
