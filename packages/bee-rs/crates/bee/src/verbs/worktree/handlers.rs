// `worktree new` / `worktree merge` and routing
//
// Split out of the single 4.2k-line verbs/worktree.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_roots_core, Resolution};
use crate::verbs::reservations::{js_numberify, js_trim, now_iso, parse_flags, FlagV, Flags};
use crate::verbs::workspace_store as ws;
use crate::verbs::{emit_no_root_error, record_timing};
use crate::{jsjson, lock};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

// ─── worktree new / merge ─────────────────────────────────────────────────

pub(crate) fn bool_flag_ok(flags: &Flags, name: &str) -> bool {
    match flags.get(name) {
        None | Some(FlagV::Present) => true,
        Some(FlagV::S(s)) => s == "true" || s == "false",
    }
}

/// `flags.x === true` — a bare `--x` or an explicit `--x=true`.
pub(crate) fn bool_flag_true(flags: &Flags, name: &str) -> bool {
    match flags.get(name) {
        Some(FlagV::Present) => true,
        Some(FlagV::S(s)) => s == "true",
        None => false,
    }
}

pub(crate) fn run_new(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["feature", "base-ref", "with-companion"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "with-companion") {
        return None; // validate() rejects a non-boolean value first
    }
    let with_companion = bool_flag_true(&flags, "with-companion");
    // requireFlag(flags, 'feature') runs before the resolution check.
    let feature = match flags.get("feature") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    // `flags['base-ref'] !== undefined ? String(flags['base-ref']) : undefined`
    // — a bare `--base-ref` stringifies `true` into a ref name; unproven here.
    let base_ref: Option<String> = match flags.get("base-ref") {
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(FlagV::Present) => return None,
        None => None,
    };

    let ctx = match prelude("worktree new", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&format!(
            "\"bee worktree new\" must be run from inside the main checkout, not a \"{}\" checkout — run it from the main repo root, then open your next session inside the created worktree.",
            ctx.kind
        )));
    }
    let main_root = ctx.work_root.clone();

    // wcg-3 (D1a/D3/D4): the shared-nested-checkout guard, whole. Fires BEFORE
    // any mutation, hard fail-closed with no override (D3), and teaches the
    // paved road (D4 — a NEW companion-mounted worktree, never an in-place
    // conversion). Both of its shapes now run natively: the marker-verified
    // companion mount comes from THE write guard's own
    // `resolveVerifiedCompanionMountReal` and the nested-`.git` down-scan from
    // crate::nested_checkout (which is the only thing that module holds — see
    // its header for why the predicates are imported rather than re-derived).
    //
    // `!withCompanion && …` short-circuits in the .mjs, so `--with-companion`
    // never even runs the scan: the guard exists to push a caller toward that
    // flag, and is never a refusal for someone who already passed it.
    //
    // A solo checkout stays a pure no-op (D6): nobody else live, no scan.
    let main_root_s = p(&main_root);
    let ctrl_root = crate::verbs::reservations::control_root_for(&main_root_s).ok()?;
    let session_id = crate::verbs::reservations::resolve_session_id(None, &ctrl_root).ok()?;
    let shared_nested_found = if with_companion {
        false
    } else {
        match crate::nested_checkout::has_any_shared_nested_checkout(
            &main_root,
            &ctrl_root,
            session_id.as_deref(),
        ) {
            Ok(found) => found,
            // The detection check itself errored. Node's message interpolates
            // the caught error's V8 `.message`; this port supplies its own
            // deterministic reason in the same slot (crate::nested_checkout's
            // header documents the divergence). Everything else — fail CLOSED,
            // zero mutation, same wording, same exit — is Node's.
            Err(detect) => {
                return Some(ctx.fail(&format!(
                    "refusing to create a worktree: could not determine whether {main_root_s} holds a shared nested checkout another live session could reach — the detection check itself errored ({}). This guard fails CLOSED on a detection error rather than risk silently allowing an unguarded worktree. FIX: resolve the underlying filesystem error, then retry.",
                    detect.reason
                )))
            }
        }
    };
    if shared_nested_found {
        return Some(ctx.fail(&format!(
            "refusing to create a worktree: another session is concurrently live on {main_root_s} and it contains a shared nested checkout a companion mount must cover — running unguarded is how one session silently ate another's work. Re-run with \"bee worktree new --feature {feature} --with-companion\" so the shared checkout is mounted and tracked (the paved road for concurrent shared-checkout work — AGENTS.md rule 13). This creates a NEW companion-mounted worktree; it does not retrofit the checkout you are in."
        )));
    }

    // worktree-companion-hook: resolved HERE from readConfig(mainRoot).commands
    // and passed down as plain strings (worktree-store.mjs stays zero-deps),
    // and refused HERE — before any worktree is created — rather than surfacing
    // later as a confusing symlink failure.
    let mut companion_start: Option<String> = None;
    let mut companion_mount: Option<String> = None;
    if with_companion {
        let commands = read_worktree_commands(&main_root)?; // corrupt config -> Node
        companion_start = commands.companion_start;
        companion_mount = commands.companion_mount;
        if companion_start.is_none() {
            return Some(ctx.fail(
                "--with-companion requires commands.worktree_companion_start to be set in .bee/config.json.",
            ));
        }
        if companion_mount.is_none() {
            return Some(ctx.fail(
                "--with-companion requires commands.worktree_companion_mount to be set in .bee/config.json.",
            ));
        }
    }

    let mut lock_busy: Option<String> = None;
    let created = match create_feature_worktree(
        &main_root,
        &feature,
        base_ref.as_deref(),
        CompanionSpec {
            start_command: companion_start.as_deref(),
            mount_path: companion_mount.as_deref(),
        },
        &mut lock_busy,
    ) {
        Ok(c) => c,
        Err(CErr::Refuse(message)) => return Some(ctx.fail(&message)),
        Err(CErr::Ex) => match lock_busy {
            // LockBusyError is reached AFTER a lock attempt, so it is native
            // (campaign rule 2 — delegating would double the telemetry).
            Some(message) => return Some(ctx.fail(&message)),
            None => return None,
        },
    };

    // GH #31 (wux-1): the explicit session-boundary next step.
    let next_step = format!(
        "Open a new session with cwd={} to work the \"{feature}\" feature there — this session stays on main. Merge back later with \"bee worktree merge --id {}\".",
        p(&created.worktree_root),
        created.id
    );
    let (result, text) = new_result_and_text(&feature, &created, &next_step);
    Some(ctx.emit(&Value::Object(result), &text))
}

/// bee.mjs handleWorktreeNew's result/text, split out the same way
/// `merge_text_lines` is: it needs only `feature`, the `Created` answer, and
/// the already-formatted `next_step` line, never a live `ctx`/cwd — so it is
/// directly unit-testable without standing up a real linked worktree.
pub(crate) fn new_result_and_text(feature: &str, created: &Created, next_step: &str) -> (Map<String, Value>, String) {
    let mut result = Map::new();
    result.insert("id".into(), json!(created.id));
    result.insert("worktreeRoot".into(), json!(p(&created.worktree_root)));
    result.insert("branch".into(), json!(created.branch));
    result.insert(
        "baseRef".into(),
        created.base_ref.clone().map_or(Value::Null, Value::String),
    );
    result.insert(
        "baseRefSha".into(),
        created.base_ref_sha.clone().map_or(Value::Null, Value::String),
    );
    result.insert("skillsSync".into(), created.skills_sync.clone());
    // `companion: created.companion || null`.
    result.insert(
        "companion".into(),
        if js_truthy(&created.companion) {
            created.companion.clone()
        } else {
            Value::Null
        },
    );
    // review B-P2-7 / D-P3-1: carry the bootstrap report's `cellsSync` skip
    // note into `worktree new`'s own result — `run_register` already
    // surfaces it (both the map and the text), and a fresh-created worktree
    // can hit the exact same symlinked-path refusal.
    if let Some(sync) = created.bootstrap.get("cellsSync") {
        result.insert("cellsSync".into(), sync.clone());
    }
    result.insert("next_step".into(), json!(next_step));

    let skills_line = if created.skills_sync.get("applied") == Some(&Value::Bool(true)) {
        "  skills:      bee-* skill trees synced into the worktree.".to_string()
    } else {
        format!(
            "  skills:      NOT synced ({}) — bee* skills may be missing in a session opened there.",
            created
                .skills_sync
                .get("reason")
                .map(jsjson::js_to_string)
                .unwrap_or_default()
        )
    };
    let branch_line = match &created.base_ref_sha {
        Some(sha) => format!(
            "  branch:      {} (based on {}, resolved to {sha})",
            created.branch,
            jsjson::stringify(&created.base_ref.clone().map_or(Value::Null, Value::String))
        ),
        None => format!("  branch:      {}", created.branch),
    };
    let bootstrap_line = if created.bootstrap.get("created") == Some(&Value::Bool(true)) {
        format!(
            "  bootstrapped {} (phase idle, gates unapproved).",
            created
                .bootstrap
                .get("worktreeStoreRoot")
                .map(jsjson::js_to_string)
                .unwrap_or_default()
        )
    } else {
        format!(
            "  worktree .bee/state.json already existed — left untouched ({}).",
            created.bootstrap.get("reason").map(jsjson::js_to_string).unwrap_or_default()
        )
    };
    // `.filter((line) => line !== null)` — the companion line is present only
    // when a companion was actually mounted.
    let mut lines = vec![
        format!(
            "Created worktree for feature \"{feature}\": {}",
            p(&created.worktree_root)
        ),
        branch_line,
        bootstrap_line,
        skills_line,
    ];
    // review B-P2-7: the same one-line skip note `run_register` prints.
    if let Some(sync) = created.bootstrap.get("cellsSync") {
        let path = sync.get("path").map(jsjson::js_to_string).unwrap_or_default();
        let reason = sync.get("reason").map(jsjson::js_to_string).unwrap_or_default();
        lines.push(format!("  cells sync skipped — {path}: {reason}"));
    }
    if js_truthy(&created.companion) {
        let field = |k: &str| {
            created
                .companion
                .get(k)
                .map(jsjson::js_to_string)
                .unwrap_or_default()
        };
        let session = match created.companion.get("sessionId") {
            Some(v) if js_truthy(v) => format!(", session {}", jsjson::js_to_string(v)),
            _ => String::new(),
        };
        lines.push(format!(
            "  companion:   mounted at {} ({}{session}).",
            field("mountPath"),
            field("worktreePath")
        ));
    }
    lines.push(next_step.to_string());
    let text = lines.join("\n");
    (result, text)
}

/// bee.mjs's `WORKTREE_MERGE_SESSIONLESS_ID` (multisession-native-22): merge
/// has never required session identity to run solo, and lease-store only needs
/// SOME non-empty session_id string.
pub(crate) const WORKTREE_MERGE_SESSIONLESS_ID: &str = "bee-worktree-merge-sessionless";

/// readConfig(mainRoot).commands, narrowed to the five keys the worktree verbs
/// read. normalizeCommands trims every string value, drops empties, and keeps
/// `test`'s ARRAY shape distinct from its string shape — which matters,
/// because merge's verify fallback is `typeof commands.test === 'string'`, so
/// an array `test` is never spawned as one shell command.
pub(crate) struct WorktreeCommands {
    pub(crate) test_string: Option<String>,
    pub(crate) companion_start: Option<String>,
    pub(crate) companion_mount: Option<String>,
    pub(crate) companion_end: Option<String>,
}

pub(crate) fn read_worktree_commands(main_root: &Path) -> Option<WorktreeCommands> {
    let config = crate::state::read_config_raw(main_root);
    let mut out = WorktreeCommands {
        test_string: None,
        companion_start: None,
        companion_mount: None,
        companion_end: None,
    };
    let Some(Value::Object(raw)) = config.get("commands") else {
        return Some(out);
    };
    let trimmed = |key: &str| -> Option<String> {
        match raw.get(key) {
            Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
            _ => None,
        }
    };
    out.test_string = trimmed("test");
    out.companion_start = trimmed("worktree_companion_start");
    out.companion_mount = trimmed("worktree_companion_mount");
    out.companion_end = trimmed("worktree_companion_end");
    Some(out)
}

/// `Number(string)` — ECMA-262 StringNumericLiteral, whole: leading/trailing
/// whitespace stripped, an empty (or all-whitespace) string is 0, `Infinity`
/// with an optional sign, the 0x/0o/0b integer literals (no sign allowed on
/// those), and the ordinary decimal grammar with an optional exponent.
/// Anything else is NaN.
///
/// This is the FULL conversion, unlike verbs/reservations.rs's
/// `js_number_flag`, which models the same grammar but returns
/// `Number.parseInt`'s answer for its own call sites.
pub(crate) fn js_string_to_number(raw: &str) -> f64 {
    let t = js_trim(raw);
    if t.is_empty() {
        return 0.0; // Number('') === 0, Number('   ') === 0
    }
    // Radix literals: no sign, at least one digit.
    if let Some(rest) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return radix_value(rest, 16);
    }
    if let Some(rest) = t.strip_prefix("0o").or_else(|| t.strip_prefix("0O")) {
        return radix_value(rest, 8);
    }
    if let Some(rest) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
        return radix_value(rest, 2);
    }
    let (sign, body) = match t.strip_prefix('-') {
        Some(rest) => (-1.0, rest),
        None => (1.0, t.strip_prefix('+').unwrap_or(t)),
    };
    if body == "Infinity" {
        return sign * f64::INFINITY;
    }
    // [digits][.[digits]] | .digits, then optional [eE][+-]digits.
    let bytes = body.as_bytes();
    let mut i = 0usize;
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
        return f64::NAN;
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
            return f64::NAN;
        }
    }
    if i != bytes.len() {
        return f64::NAN;
    }
    // Rust's f64 parser accepts exactly this grammar and rounds the same way
    // (both are correctly-rounded IEEE-754); an out-of-range literal saturates
    // to ±Infinity in both.
    body.parse::<f64>().map(|v| sign * v).unwrap_or(f64::NAN)
}

pub(crate) fn radix_value(digits: &str, radix: u32) -> f64 {
    if digits.is_empty() {
        return f64::NAN;
    }
    let mut acc = 0.0f64;
    for c in digits.chars() {
        match c.to_digit(radix) {
            Some(d) => acc = acc * f64::from(radix) + f64::from(d),
            None => return f64::NAN,
        }
    }
    acc
}

/// wkm-1 (D1): `worktree_cleanup_on_merge` in `.bee/config.json`, read the
/// same shape `archive_on_close_enabled` reads `cells_archive_on_close`
/// (close.rs:827) — but the MEANING of "absent" flips here: a merge KEEPS
/// the worktree by default now, so an absent key reads as `false` (no
/// config opt-in), not `true`. A present-but-non-boolean value stays
/// REFUSED (`None`), same as before: a typo'd config value must never
/// resolve to cleanup running unasked, and it must never resolve to cleanup
/// being silently skipped either.
pub(crate) fn worktree_cleanup_on_merge_config(main_root: &Path) -> Option<bool> {
    match crate::state::read_config_raw(main_root).get("worktree_cleanup_on_merge") {
        None => Some(false),
        Some(Value::Bool(b)) => Some(*b),
        Some(_) => None,
    }
}

/// wkm-1 (D1): the merge's effective cleanup decision — KEEP by default.
/// Teardown runs only when `--cleanup` was passed for this one merge, OR
/// the repo's config explicitly opts in (`worktree_cleanup_on_merge:
/// true`). `--no-cleanup` is an explicit keep and wins over BOTH of those —
/// even a config `true`. Both flags are already validated boolean-shaped by
/// the caller (`bool_flag_ok`); `None` here means the CONFIG value was
/// invalid and the merge must refuse rather than guess.
pub(crate) fn resolve_cleanup_on_merge(
    main_root: &Path,
    cleanup_flag: bool,
    no_cleanup_flag: bool,
) -> Option<bool> {
    let config_enabled = worktree_cleanup_on_merge_config(main_root)?;
    Some(!no_cleanup_flag && (cleanup_flag || config_enabled))
}

pub(crate) fn run_merge(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(
        &flags,
        &["id", "cleanup", "no-cleanup", "queue-wait-ms", "skip-uat"],
    ) {
        return None;
    }
    // `--cleanup` (wkm-1, D1): re-armed. The default flipped to KEEP, so
    // this flag is what opts a single merge back into teardown — the same
    // one-merge opt-in `--no-cleanup` is an opt-out for.
    if !bool_flag_ok(&flags, "cleanup") {
        return None;
    }
    // `--no-cleanup` (D1a): the one-merge opt-out. A non-boolean value is
    // REFUSED outright here, never ignored and never defaulted to either
    // outcome — `bool_flag_true` would otherwise read a mis-parsed value as
    // "cleanup stays on", which is the destructive direction now that
    // cleanup is the default.
    if !bool_flag_ok(&flags, "no-cleanup") {
        return None;
    }
    // `--skip-uat` (uat-gate-before-merge D1): the one-merge opt-out of the
    // uat gate precondition. Validated the same fail-closed way as
    // `no-cleanup` — a non-boolean value refuses outright rather than
    // silently reading as either outcome.
    if !bool_flag_ok(&flags, "skip-uat") {
        return None;
    }
    // `--queue-wait-ms`: a registry `type:"number"` flag. validate() runs
    // first (typeMatches: a non-empty string whose `Number(...)` is FINITE),
    // then the handler keeps the value only when it is also POSITIVE —
    // anything else silently keeps DEFAULT_WAIT_BOUND_MS rather than refusing.
    let mut queue_wait_bound_ms = crate::integration_queue::DEFAULT_WAIT_BOUND_MS;
    match flags.get("queue-wait-ms") {
        None => {}
        // A bare `--queue-wait-ms` parses to `true`, which fails
        // typeMatches('number') — the dispatcher's own generic validate()
        // refusal, shared by every verb and not this flag's arm; it is the one
        // shape here that still returns before any output.
        Some(FlagV::Present) => return None,
        Some(FlagV::S(raw)) => {
            if js_trim(raw).is_empty() {
                return None; // validate(): `value.trim() !== ''`
            }
            let n = js_string_to_number(raw);
            if !n.is_finite() {
                return None; // validate(): `Number.isFinite(Number(value))`
            }
            if n > 0.0 {
                queue_wait_bound_ms = n;
            }
        }
    }
    let id = match flags.get("id") {
        Some(FlagV::S(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    let cleanup_flag = bool_flag_true(&flags, "cleanup");
    let no_cleanup_flag = bool_flag_true(&flags, "no-cleanup");
    let skip_uat_flag = bool_flag_true(&flags, "skip-uat");

    let ctx = match prelude("worktree merge", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&format!(
            "\"bee worktree merge\" must be run from inside the main checkout, not a \"{}\" checkout — a worktree, including the one being merged, cannot merge itself.",
            ctx.kind
        )));
    }
    let main_root = ctx.work_root.clone();

    // wkm-1 (D1): the worktree is KEPT by default; `--cleanup` or an
    // explicit `worktree_cleanup_on_merge: true` opts a merge into
    // teardown, and `--no-cleanup` always wins as an explicit keep. This is
    // decided before the first lock, same as every other gate below, and
    // refuses (rather than guesses) on an invalid config value.
    let cleanup = resolve_cleanup_on_merge(&main_root, cleanup_flag, no_cleanup_flag)?;

    // ── every delegation gate, decided BEFORE the first lock ──────────────
    // Campaign rule 2: lock.rs appends a `result: "acquired"` row to
    // .bee/logs/contention.jsonl on EVERY successful acquire, so a delegation
    // taken after one would leave a doubled row in the `.bee/` tree — a C1
    // break, not just noisy telemetry. Each probe below is read-only.

    let commands = read_worktree_commands(&main_root)?; // corrupt config -> Node
    // D7/D8 (docs/history/test-doctrine/CONTEXT.md, td-3): merge no longer
    // spawns `commands.test` itself — the tests door reads whether every
    // capped cell for the merging feature already carries a recorded D8
    // proof line instead (`merge_stage`'s own zero-mutation precondition,
    // verbs/cells/proof.rs `feature_proof_check`, the same helper `bee
    // close`'s tests door reads, td-2). `commands` is still read here only
    // for the companion-teardown command below.
    // worktree-companion-hook: resolved unconditionally (cheap) — there is no
    // `--with-companion` on the merge side, because the worktree's own marker
    // IS the signal. A worktree WITH a marker is torn down even when this
    // invocation opted in to nothing; there is nothing to opt in to.
    let companion_end_command = commands.companion_end.clone();

    // readGrants is consulted twice (P1's grant check, P3's fence); an
    // unparseable registry delegates here rather than from inside a hold.
    read_grants_strict(&main_root.join(".bee"))?;

    let main_root_s = p(&main_root);
    let ctrl_root_s = crate::verbs::reservations::control_root_for(&main_root_s).ok()?;
    let ctrl_root = PathBuf::from(&ctrl_root_s);
    let session_id = crate::verbs::reservations::resolve_session_id(None, &ctrl_root_s)
        .ok()?
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| WORKTREE_MERGE_SESSIONLESS_ID.to_string());

    // listQueueRecords reads each record with fsutil's WARNING readJson.
    if !crate::integration_queue::preflight_queue_readable(&ctrl_root) {
        return None;
    }
    // performCleanup's releaseAllForHolder reads the holds ledger the same
    // way. wkm-1 (D1): cleanup is now the OPT-IN outcome — `--cleanup` or an
    // explicit `worktree_cleanup_on_merge: true` are the only ways this
    // branch fires — so a corrupt ledger degrading gracefully still matters
    // on every merge that does opt in. `release_all_for_holder` itself
    // already treats an unparseable ledger as empty (best-effort), so this
    // probe matches that: it reads the bytes but never turns a parse
    // failure into a refusal.
    if cleanup {
        let ledger = main_root
            .join(".bee")
            .join("runtime")
            .join("cross-worktree-holds.json");
        if let Ok(bytes) = std::fs::read(&ledger) {
            let _ = serde_json::from_slice::<Value>(&bytes);
        }
    }

    // ── the drain (multisession-native-22 D8 stage 5) ─────────────────────
    let mut thrown: Option<String> = None;
    let drained = crate::integration_queue::run_through_queue(
        &ctrl_root,
        &id,
        &session_id,
        "main",
        queue_wait_bound_ms,
        crate::integration_queue::DEFAULT_POLL_INTERVAL_MS,
        crate::integration_queue::DEFAULT_PROCESSOR_TTL_SECONDS,
        crate::integration_queue::DEFAULT_RENEW_INTERVAL_MS,
        |hooks| {
            match merge_feature_worktree(
                &main_root,
                &id,
                cleanup,
                companion_end_command.as_deref(),
                skip_uat_flag,
                Some(hooks),
            ) {
                Ok(answer) => {
                    let ok = answer.ok;
                    Ok((Some(answer), ok))
                }
                // processAsOwner persists `error.message` into the queue
                // record and rethrows — both are deterministic here.
                Err(MErr::Thrown(message)) => Err(message),
                // The one late-delegation residual. Reachable ONLY if the
                // grants registry stops parsing BETWEEN the probe above and
                // P1's own read — a race, never an ordinary shape. Zero
                // mutations have happened to the repo, so the Node re-run
                // reproduces the same answer; the cost is one extra queue
                // record (this one, driven to 'done') plus its contention
                // rows, the same "idempotent steps, one duplicated
                // bookkeeping artifact" residual verbs/workspace_store.rs
                // and `worktree register` already document.
                Err(MErr::Ex) => Ok((None, true)),
            }
        },
    );

    let (result, ok, message_lines) = match drained {
        Err(crate::integration_queue::QErr::Msg(m))
        | Err(crate::integration_queue::QErr::LockBusy(m)) => {
            thrown = Some(m);
            (Map::new(), false, Vec::new())
        }
        Err(crate::integration_queue::QErr::Ex) => return None,
        Ok(crate::integration_queue::Drain::TimedOut(timeout)) => {
            // Advisor condition B: this text must UNAMBIGUOUSLY say the merge
            // did NOT run — never readable as success.
            let lines = vec![format!(
                "Merge of worktree {id} did NOT run: {}",
                timeout.message
            )];
            let map = match timeout.result {
                Value::Object(m) => m,
                _ => Map::new(),
            };
            (map, false, lines)
        }
        Ok(crate::integration_queue::Drain::Ran(None)) => return None, // MErr::Ex
        Ok(crate::integration_queue::Drain::Ran(Some(answer))) => {
            let lines = merge_text_lines(&id, &main_root, &answer);
            (answer.result, answer.ok, lines)
        }
    };
    if let Some(message) = thrown {
        return Some(ctx.fail(&message));
    }
    Some(ctx.emit_code(
        &Value::Object(result),
        &message_lines.join("\n"),
        if ok { 0 } else { 1 },
    ))
}

/// bee.mjs handleWorktreeMerge's text block, for every non-timeout outcome.
pub(crate) fn merge_text_lines(id: &str, main_root: &Path, answer: &MergeAnswer) -> Vec<String> {
    let r = &answer.result;
    let s = |key: &str| -> String { r.get(key).map(jsjson::js_to_string).unwrap_or_default() };
    let mut lines: Vec<String> = Vec::new();
    let code = r.get("code").and_then(Value::as_str).unwrap_or("");
    if answer.ok {
        if code == "ALREADY_UP_TO_DATE" {
            lines.push(format!(
                "Worktree {id} (branch {}) is already up to date with {} — nothing to merge; no commit was made.",
                s("branch"),
                p(main_root)
            ));
        } else {
            lines.push(format!(
                "Merged worktree {id} (branch {}) into {}.",
                s("branch"),
                p(main_root)
            ));
            lines.push(format!("  verify: {}", s("verify")));
        }
        // The companion block, shared by BOTH ok outcomes (issues-46-53 D3).
        if let Some(companion) = r.get("companion").filter(|v| js_truthy(v)) {
            lines.push(match companion.get("warning").filter(|w| js_truthy(w)) {
                Some(warning) => {
                    format!("  companion: WARNING — {}", jsjson::js_to_string(warning))
                }
                None => {
                    let session = match companion.get("sessionId").filter(|v| js_truthy(v)) {
                        Some(v) => format!(" (session {})", jsjson::js_to_string(v)),
                        None => String::new(),
                    };
                    format!("  companion: ended{session}.")
                }
            });
        }
        if let Some(warning) = r.get("warning") {
            lines.push(format!(
                "  WARNING ({}): {}",
                warning.get("code").map(jsjson::js_to_string).unwrap_or_default(),
                warning.get("message").map(jsjson::js_to_string).unwrap_or_default()
            ));
        }
        if let Some(cleanup) = r.get("cleanup") {
            lines.push(if cleanup.get("ok") == Some(&Value::Bool(true)) {
                "  cleanup: worktree removed, branch deleted.".to_string()
            } else {
                format!(
                    "  cleanup: refused ({}) — {}",
                    cleanup.get("code").map(jsjson::js_to_string).unwrap_or_default(),
                    cleanup.get("reason").map(jsjson::js_to_string).unwrap_or_default()
                )
            });
            if let Some(w) = cleanup.get("warning") {
                lines.push(format!("  WARNING: {}", jsjson::js_to_string(w)));
            }
        } else if let Some(cmd) = r.get("cleanup_suggested_command") {
            lines.push(format!(
                "  cleanup: run `{}` when ready.",
                jsjson::js_to_string(cmd)
            ));
        }
        // staging-lane D0a (trigger 3): the one nudge line, present only
        // when a staging record exists (phases.rs's merge_finish is the
        // sole writer of this key).
        if let Some(cmd) = r.get("staging_rebuild_suggested") {
            lines.push(format!(
                "  staging: main moved — run `{}` to catch staging up.",
                jsjson::js_to_string(cmd)
            ));
        }
    } else if code == "MERGE_VERIFY_RED" {
        lines.push(format!(
            "Merge of worktree {id} (branch {}) was TEXTUALLY CLEAN, but verify is RED (semantic-conflict alarm).",
            s("branch")
        ));
        lines.push(format!(
            "The merge was aborted — {} was left byte-untouched; no merge commit exists. Fix-first before release, then retry the merge.",
            p(main_root)
        ));
        lines.push("--- verify output tail ---".to_string());
        lines.push(s("output_tail"));
    } else {
        lines.push(format!(
            "Merge of worktree {id} hit a textual conflict — the merge was aborted and {} was left byte-untouched; bee does not auto-resolve a textual conflict. Resolve it in the worktree and retry.",
            p(main_root)
        ));
    }
    lines
}

// ─── routing ──────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "worktree" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> = args[2..]
        .iter()
        .map(|a| a.to_str())
        .collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // Node renders command-scoped help
    }
    let (flags, use_json) = parse_flags(&toks)?;
    match verb {
        "list" => run_list(flags, use_json, t0),
        "register" => run_register(flags, use_json, t0),
        "unregister" => run_unregister(flags, use_json, t0),
        "new" => run_new(flags, use_json, t0),
        "merge" => run_merge(flags, use_json, t0),
        "prune" => run_prune(flags, use_json, t0),
        _ => None,
    }
}
