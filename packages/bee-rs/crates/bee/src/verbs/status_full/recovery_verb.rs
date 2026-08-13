// `bee recovery scan` — the releasing door (sweep-recovery-door srd-2)
//
// D3: this command releases every qualifying crashed-session claim ON
// INVOCATION. No `--release` flag, no confirmation step. `bee recovery
// window` stays unbuilt (D3) — this file never touches it; the registry
// keeps its `unavailable` marker.
//
// Nested inside `status_full` rather than a sibling `verbs::recovery` group:
// the REPORT pass below calls `detect_crash_candidates`
// (status_full/recovery.rs:264), which needs `&mut status_full::Ctx` — a
// type whose four fields are private and which exposes no constructor
// outside this module tree (status_full/mod.rs:159-196). Nesting here is
// smaller than widening `Ctx`, which four other files depend on staying
// closed (plan.md Discovery 5).
//
// Three independent passes, reported as three separate sets — never merged
// (plan.md "Approach"):
//   - RELEASE — `sweep_cells::sweep_expired_claims` with THIS door's own
//     resolved caller (R97 self-exclusion, R99 park-not-reopen, R100 store
//     boundary — all inherited unchanged).
//   - MARK (D2/D9) — an independent pass over `.bee/sessions/`: every
//     heartbeat-stale record is marked `status: "dead"` + `dead_at` in
//     place, re-judged under the SAME `sessions` store lock
//     `heartbeatSession` itself holds, so a record read stale before the
//     lock is not marked after its owner's heartbeat lands under it. A
//     session that died holding no claim is still marked — this pass never
//     derives its membership from the release pass above.
//   - REPORT — `detect_crash_candidates`'s trio-filtered, transcript-shaped
//     set: which sessions are worth mining. Named separately from RELEASE:
//     a cleanly-ended session holding an expired claim is released and
//     marked, but is never a reported candidate (Discovery 1).
//
// R98 is NOT inherited from the sweep (`sweep_expired_claims(control, now,
// None)` sweeps with no exclusion at all when handed `None` — that shape is
// deliberately exercised by `cells/tests.rs`). This door resolves its own
// caller exactly as `orient.rs:260-261` does and implements its own D6
// decline: when neither `resolve_session_flag_env` nor `resolve_session_adopt`
// resolves an identity, NOTHING is written — no release, no mark — and the
// command reports the counts it observed instead.

use super::*;
use crate::fsutil::write_json_atomic;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, RootsWt};
use crate::verbs::cells as sweep_cells;
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

/// Bridges `sweep_cells::Fail` (`MR<T> = Result<T, Fail>`) into this
/// module's own `Ex` (`R<T> = Result<T, Ex>`, mod.rs:157) — a fresh copy of
/// `orient.rs`'s own `bridge_sweep_fail` (`orient.rs:205-207`), not a shared
/// helper: the two error types cross a module boundary here exactly as they
/// do there, and each door maps its own, never unifies them.
fn bridge_sweep_fail(_: sweep_cells::Fail) -> Ex {
    Ex::Thrown
}

/// D6-decline preview (R98): counts, without touching anything, the claims
/// `sweep_expired_claims` would have released — TTL expired AND owning
/// session heartbeat-stale. A private copy of `orient.rs`'s own
/// `count_expired_claims` (see `bridge_sweep_fail`'s doc for why this file
/// keeps its own rather than reaching into `orient.rs`'s module-private
/// helpers).
fn count_expired_claims(control: &Path, now: f64) -> sweep_cells::MR<usize> {
    let dir = sweep_cells::claims_dir(control);
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(0) };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    let mut count = 0usize;
    for entry in names {
        let Some(cell) = entry.strip_suffix(".json") else { continue };
        let Some(claim) = sweep_cells::read_claim(control, cell)? else { continue };
        if !sweep_cells::claim_expired(&claim, now)? {
            continue;
        }
        if !sweep_cells::heartbeat_stale(sweep_cells::read_session_of_claim(control, &claim)?.as_ref(), now)? {
            continue;
        }
        count += 1;
    }
    Ok(count)
}

/// The MARK pass's own decline preview (R98's D9 analogue): how many
/// session records are heartbeat-stale and not already marked — the count
/// the MARK pass would have touched, had a caller identity resolved.
fn count_heartbeat_stale_sessions(control: &Path, now: f64) -> sweep_cells::MR<usize> {
    let mut count = 0usize;
    for record in sweep_cells::list_session_records(control)? {
        if matches!(record.get("status"), Some(Value::String(s)) if s == "dead") {
            continue; // already marked — not a fresh mark this pass would make
        }
        if sweep_cells::heartbeat_stale(Some(&record), now)? {
            count += 1;
        }
    }
    Ok(count)
}

/// D2/D9: an independent pass over `.bee/sessions/`, marking every
/// heartbeat-stale record dead in place (`status: "dead"`, `dead_at`) —
/// never deleted (D2). Staleness is re-judged under the SAME `sessions`
/// store lock `heartbeatSession` itself holds
/// (`cells::acquire_sessions_lock_bounded`, handlers_select.rs:111 — the
/// bounded-retry variant, never the hooks' single-attempt fail-open
/// helper: a dropped mark here is a silent miss, where a dropped heartbeat
/// touch is merely late), so a record read stale before the lock is NOT
/// marked after its owner's heartbeat lands under it.
///
/// `caller`'s own record is excluded, symmetric with R97's claim-sweep
/// exclusion: a live caller mid a long tool call can read heartbeat-stale
/// to its own `recovery scan` just as easily as to someone else's sweep,
/// and this door must never mark its own session dead.
///
/// An already-`"dead"` record is left untouched — idempotent, so its
/// `dead_at` stays the original death time, never overwritten by a later
/// scan; a session that revives clears the mark on its own heartbeat path
/// (D8, srd-3), not here.
pub(crate) fn mark_stale_sessions(
    control: &Path,
    now: f64,
    caller: &str,
) -> sweep_cells::MR<BTreeSet<String>> {
    let mut marked = BTreeSet::new();
    let dir = sweep_cells::sessions_dir(control);
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(marked) };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    for name in names {
        let Some(id) = name.strip_suffix(".json") else { continue };
        if id == caller {
            continue;
        }
        // Cheap pre-check outside the lock. A corrupt or id-mismatched
        // record warns (read_store_json, inside read_session) and is
        // skipped loudly — never aborts the pass (one bad record among
        // many session files must not stop the rest from being judged).
        let Some(record) = sweep_cells::read_session(control, id)? else { continue };
        if matches!(record.get("status"), Some(Value::String(s)) if s == "dead") {
            continue;
        }
        if !sweep_cells::heartbeat_stale(Some(&record), now)? {
            continue;
        }
        let Some(mut guard) = sweep_cells::acquire_sessions_lock_bounded(control) else {
            continue; // contention — never steal, same discipline as the claim sweep's own gate
        };
        let outcome = (|| -> sweep_cells::MR<bool> {
            let Some(fresh) = sweep_cells::read_session(control, id)? else { return Ok(false) };
            if matches!(fresh.get("status"), Some(Value::String(s)) if s == "dead") {
                return Ok(false);
            }
            if !sweep_cells::heartbeat_stale(Some(&fresh), now)? {
                // THE RACE: a heartbeat landed between the pre-check read
                // above and this lock. Re-judged fresh under the lock, so
                // it is NOT marked.
                return Ok(false);
            }
            let mut next = fresh;
            next.insert("status".into(), Value::String("dead".into()));
            next.insert("dead_at".into(), Value::String(to_iso(now)));
            let file = sweep_cells::sessions_dir(control).join(format!("{id}.json"));
            let value = Value::Object(next);
            sweep_cells::transient_fs_retry(|| write_json_atomic(&file, &value))
                .map_err(|e| sweep_cells::Fail::Thrown(format!("{e}")))?;
            Ok(true)
        })();
        guard.release();
        if outcome? {
            marked.insert(id.to_string());
        }
    }
    Ok(marked)
}

/// What one `recovery scan` run did, or why it did nothing (D6).
pub(crate) enum ScanOutcome {
    /// Neither `resolve_session_flag_env` nor `resolve_session_adopt`
    /// resolved this door's own caller identity — nothing was released or
    /// marked; these are the counts it observed instead.
    Declined { expired_claims: usize, stale_sessions: usize },
    Ran {
        caller: String,
        released: BTreeSet<String>,
        parked: BTreeSet<String>,
        unreachable: BTreeSet<String>,
        marked_dead: BTreeSet<String>,
        candidates: Vec<Value>,
        /// `detect_crash_candidates` degraded (a Thrown it caught locally,
        /// per `build_recovery_block`'s own fail-open) — `candidates` is
        /// then an empty, not exhaustive, list.
        candidates_degraded: bool,
    },
}

/// The three passes (RELEASE, MARK, REPORT) — see the module header.
pub(crate) fn run_scan(ctx: &mut Ctx) -> R<ScanOutcome> {
    let control = sweep_cells::control_root(&ctx.root).map_err(bridge_sweep_fail)?;
    let now = now_ms();
    // Exactly orient.rs:260-261's own resolution order — this door's own
    // decline (D6/R98), never inherited from the sweep it calls.
    let caller = sweep_cells::resolve_session_flag_env(None)
        .or_else(|| sweep_cells::resolve_session_adopt(&control).ok().flatten());
    let Some(caller) = caller else {
        let expired_claims = count_expired_claims(&control, now).map_err(bridge_sweep_fail)?;
        let stale_sessions = count_heartbeat_stale_sessions(&control, now).map_err(bridge_sweep_fail)?;
        return Ok(ScanOutcome::Declined { expired_claims, stale_sessions });
    };
    let summary = sweep_cells::sweep_expired_claims(&control, now, Some(caller.as_str()))
        .map_err(bridge_sweep_fail)?;
    let marked_dead = mark_stale_sessions(&control, now, &caller).map_err(bridge_sweep_fail)?;
    let projects_root = claude_projects_root();
    let (candidates, candidates_degraded) = match detect_crash_candidates(ctx, &projects_root) {
        Ok(v) => (v, false),
        Err(Ex::Thrown) => (Vec::new(), true),
        Err(e @ Ex::Bail) => return Err(e),
    };
    Ok(ScanOutcome::Ran {
        caller,
        released: summary.released,
        parked: summary.parked,
        unreachable: summary.unreachable,
        marked_dead,
        candidates,
        candidates_degraded,
    })
}

fn ids_or_none(ids: &BTreeSet<String>) -> String {
    if ids.is_empty() {
        "none".to_string()
    } else {
        ids.iter().cloned().collect::<Vec<_>>().join(", ")
    }
}

pub(crate) fn scan_outcome_to_json(outcome: &ScanOutcome) -> Value {
    let mut m = Map::new();
    match outcome {
        ScanOutcome::Declined { expired_claims, stale_sessions } => {
            m.insert("declined".into(), json!(true));
            m.insert("caller_session".into(), Value::Null);
            m.insert("expired_claims_observed".into(), json!(expired_claims));
            m.insert("heartbeat_stale_sessions_observed".into(), json!(stale_sessions));
            m.insert("released".into(), json!([]));
            m.insert("parked".into(), json!([]));
            m.insert("unreachable".into(), json!([]));
            m.insert("marked_dead".into(), json!([]));
            m.insert("candidates".into(), json!([]));
            m.insert("candidates_degraded".into(), json!(false));
        }
        ScanOutcome::Ran {
            caller,
            released,
            parked,
            unreachable,
            marked_dead,
            candidates,
            candidates_degraded,
        } => {
            m.insert("declined".into(), json!(false));
            m.insert("caller_session".into(), json!(caller));
            m.insert("released".into(), json!(released.iter().cloned().collect::<Vec<_>>()));
            m.insert("parked".into(), json!(parked.iter().cloned().collect::<Vec<_>>()));
            m.insert(
                "unreachable".into(),
                json!(unreachable.iter().cloned().collect::<Vec<_>>()),
            );
            m.insert(
                "marked_dead".into(),
                json!(marked_dead.iter().cloned().collect::<Vec<_>>()),
            );
            m.insert("candidates".into(), Value::Array(candidates.clone()));
            m.insert("candidates_degraded".into(), json!(candidates_degraded));
        }
    }
    Value::Object(m)
}

pub(crate) fn render_scan_text(outcome: &ScanOutcome) -> String {
    match outcome {
        ScanOutcome::Declined { expired_claims, stale_sessions } => format!(
            "recovery scan declined: could not resolve its own caller session (BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID unset, and no single fresh session record to adopt) — {expired_claims} expired claim(s) and {stale_sessions} heartbeat-stale session record(s) observed; nothing released or marked. Pass --session-id to `bee cells claim-next` (which sweeps too) from an identified session, or set BEE_SESSION_ID, then retry."
        ),
        ScanOutcome::Ran {
            caller,
            released,
            parked,
            unreachable,
            marked_dead,
            candidates,
            candidates_degraded,
        } => {
            let mut lines = Vec::new();
            lines.push(format!("recovery scan (caller {caller}):"));
            lines.push(format!("released: {}", ids_or_none(released)));
            lines.push(format!("parked: {}", ids_or_none(parked)));
            lines.push(format!("unreachable: {}", ids_or_none(unreachable)));
            lines.push(format!("marked dead: {}", ids_or_none(marked_dead)));
            if *candidates_degraded {
                lines.push("candidates: degraded (a transcript read failed) — none reported".to_string());
            } else if candidates.is_empty() {
                lines.push("candidates: none".to_string());
            } else {
                let ids: Vec<String> = candidates.iter().map(|c| tpl(vget(c, "session_id"))).collect();
                lines.push(format!("candidates: {}", ids.join(", ")));
            }
            lines.join("\n")
        }
    }
}

// ─── entry point ───────────────────────────────────────────────────────────

pub(crate) fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    let strs: Vec<&str> = args.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    // Exactly two shapes served — `verbs/timings.rs:46-58`'s own argv
    // matching discipline. Any other `recovery …` shape (including
    // `recovery window …`) falls through to the catalog, which still
    // answers it with the `unavailable` refusal (D3: `recovery window`
    // stays unbuilt).
    let use_json = match strs.as_slice() {
        ["recovery", "scan"] => false,
        ["recovery", "scan", "--json"] => true,
        _ => return None,
    };
    run(use_json, t0)
}

fn run(use_json: bool, t0: Instant) -> Option<ExitCode> {
    let cwd = std::env::current_dir().ok()?;
    let cmd = "recovery scan";
    // The FULL door (roots.rs:35 names it FULL; :645-653 reserves WIDE for
    // `resolve_store_root_any`, forbidden here — this door reads sessions
    // and claims). Matched `RootsWt::Go` exactly as `orient`/`status` do.
    let roots = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r,
        RootsWt::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, cmd, use_json, t0, &why))
        }
        RootsWt::None => return Some(emit_no_root_error(&cwd, cmd, use_json, t0)),
    };
    let root = roots.root;
    let drift = check_manifest_drift(&root);
    let mut ctx = Ctx { root, cwd, linked: roots.linked, stderr: RefCell::new(Vec::new()) };
    let outcome = match run_scan(&mut ctx) {
        Ok(o) => o,
        // JS-exotic store data (Ex::Bail), or a Thrown that escaped every
        // local catch (mod.rs's own error-plumbing note: "a Thrown that
        // would escape further instead bails") — refuse OUT LOUD rather
        // than silently drop the run (router.rs's own rule: silence is the
        // one outcome forbidden).
        Err(_) => {
            let msg = format!(
                "bee {cmd}: could not complete — unexpected store data; see stderr for any warnings already emitted."
            );
            if use_json {
                println!("{}", jsjson::stringify(&json!({ "error": msg })));
            } else {
                eprintln!("{msg}");
            }
            record_timing(&ctx.root, cmd, t0, false);
            return Some(ExitCode::FAILURE);
        }
    };
    let payload = scan_outcome_to_json(&outcome);
    let text = render_scan_text(&outcome);
    // Emission order (per stream), matching orient/status: handler
    // warnings, then the drift line on stderr; the payload on stdout; the
    // timing line last.
    for line in ctx.stderr.borrow().iter() {
        eprintln!("{line}");
    }
    if drift.manifest_changed {
        eprintln!("manifest_changed: true — {}", drift.hint);
    }
    if use_json {
        println!("{}", jsjson::stringify_pretty(&payload));
    } else {
        println!("{text}");
    }
    record_timing(&ctx.root, cmd, t0, true);
    Some(ExitCode::SUCCESS)
}
