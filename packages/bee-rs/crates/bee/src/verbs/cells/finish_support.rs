// the test runner, the reservations-release half of finish, and the delegation pre-scans
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{self, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots, StoreRoots};
use crate::state as bstate;
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::{Err2, FlagV, Out, R2};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── test runner (lib/test-runner.mjs) ─────────────────────────────────────

pub(crate) const TEST_RESULTS_RELATIVE: &str = ".bee/logs/test-results.json";

pub(crate) fn test_results_path(root: &Path) -> PathBuf {
    root.join(".bee").join("logs").join("test-results.json")
}

// decision 13ce1858 (test-cadence-boundary D1): the cap's own copy of the
// declared-test runner (spawn_declared/run_declared_tests/CmdRun/TestsRun/
// tests_record_value/first_failure_line, plus this file's own posix_shell)
// is DELETED here — `cap_cell_from_flags` no longer runs any test command,
// so nothing in this crate called this copy anymore. Decisions 58ec9664 and
// 1f534837 later retired the doors' own copies too — `bee close` and
// `bee worktree merge` now call `crate::verbs::cells::feature_proof_check`
// (drivers/close.rs, worktree/phases.rs) and check the cap's recorded proof
// line instead of running the command themselves; no local door runs a test
// process any more, only CI does, on every push. `TEST_RESULTS_RELATIVE`/
// `test_results_path` above stay: other doors
// (the D2 red-base claim check, `cells finish --report`'s trace, the
// verify-none check) still read the last recorded record.

// ─── wfl-1 (docs/history/workflow-lessons/plan.md) — the structured worker
// Result form ─────────────────────────────────────────────────────────────
// `bee cells finish --report <json-string>` — the machine-readable
// counterpart to worker-cell.md's Result-form block, so tending reads the
// form instead of parsing prose. Exactly REPORT_KEYS, each required; an
// unknown key or a missing one is refused by name (frd-1's own "name the
// flag" posture). Absent `--report` never touches `trace.report` at all —
// old finish behavior stays byte-identical.

/// The Result form's five REQUIRED keys, in the order worker-cell.md
/// documents them.
pub(crate) const REPORT_KEYS: [&str; 5] = ["outcome", "commit", "files", "tests", "deviations"];

/// reflection-becomes-lesson: the Result form's optional keys — accepted,
/// never demanded.
///
/// `mistakes` is the worker's answer to "did anything go wrong": an array,
/// each entry one mistake in two parts. It is OPTIONAL here on purpose. Every
/// caller that shipped before it exists keeps working byte for byte, and the
/// cap adds no refusal of its own — the door that demands the answer is
/// `bee close`, which reads the capped cells and names the ones that never
/// answered. Making the key required here would refuse caps mid-flight for a
/// field their prompt never mentioned.
pub(crate) const REPORT_OPTIONAL_KEYS: [&str; 1] = ["mistakes"];

/// D8 (docs/history/test-doctrine/CONTEXT.md) proof-string separator —
/// three segments joined by `" — "` (space, em dash U+2014, space).
pub(crate) const PROOF_SEPARATOR: &str = " — ";

/// D1/D3 (docs/history/proof-strength-and-expiry/CONTEXT.md) — the closed
/// vocabulary a cap's proof line may record in its RESULT segment. Each
/// value's meaning is written beside it HERE and nowhere else: a meaning
/// restated in a second place is how a closed vocabulary decays back into
/// three free-text values. Checked on the WRITE path only, in
/// [`parse_report_flag`] — see the D2 note on [`parse_tests_proof`].
pub(crate) const PROOF_RESULT_VALUES: [&str; 3] = [
    // the real product or command was driven and its observable result inspected
    "green:live",
    // automated tests passed
    "green:unit",
    // it compiled, type-checked, linted, or a parity/pointer check passed,
    // with nothing executed
    "green:static",
];

/// parseTestsProof — splits a D8 proof string `<command> — <result> —
/// <scope reason>` into its three segments, splitting on the FIRST TWO
/// occurrences of [`PROOF_SEPARATOR`] only, so the reason segment may
/// itself contain the same separator. `None` when fewer than two
/// separators are found, or any segment trims to empty.
///
/// This parser checks SHAPE only and is deliberately blind to
/// [`PROOF_RESULT_VALUES`]. D2 (docs/history/proof-strength-and-expiry):
/// the READ path — `feature_proof_check` (proof.rs) — calls this same
/// function over already-capped cells, so a vocabulary check HERE would
/// retroactively refuse the ~200 historical caps that carry a bare
/// `green`. The vocabulary is checked in [`parse_report_flag`] instead, on
/// the tuple this returns, write path only. That inaction IS the write/read
/// split; do not "fix" it by moving the check down here.
pub(crate) fn parse_tests_proof(s: &str) -> Option<(String, String, String)> {
    let first = s.find(PROOF_SEPARATOR)?;
    let (command, rest) = s.split_at(first);
    let rest = &rest[PROOF_SEPARATOR.len()..];
    let second = rest.find(PROOF_SEPARATOR)?;
    let (result, reason) = rest.split_at(second);
    let reason = &reason[PROOF_SEPARATOR.len()..];
    let command = js_trim(command);
    let result = js_trim(result);
    let reason = js_trim(reason);
    if command.is_empty() || result.is_empty() || reason.is_empty() {
        return None;
    }
    Some((command.to_string(), result.to_string(), reason.to_string()))
}

/// parseReportFlag — `--report`'s raw JSON string validated against the
/// worker Result-form shape. `outcome`/`commit` are non-empty strings;
/// `files`/`deviations` are arrays (their own element shape is the
/// worker's business, not this gate's); `tests` is a D8 proof string
/// `<command> — <result> — <scope reason>` (three non-empty segments,
/// split on the FIRST TWO ` — ` separators only, so the reason may itself
/// carry the same separator) — never the retired `boundary`/`undeclared`
/// enum. A result segment reading `red` refuses the cap outright (D6's
/// spirit: a red is fix-first, never a done), and every other result
/// segment must be one of [`PROOF_RESULT_VALUES`] (D1). Every refusal names the
/// offending key so a cold reader fixes it without re-deriving the shape
/// from this function.
pub(crate) fn parse_report_flag(raw: &str) -> MR<Value> {
    let parsed: Value = serde_json::from_str(raw)
        .map_err(|e| Fail::Thrown(format!("cells finish: --report is not valid JSON: {e}")))?;
    let Value::Object(map) = parsed else {
        return Err(Fail::Thrown(format!(
            "cells finish: --report must be a JSON object with keys {}.",
            REPORT_KEYS.join(", ")
        )));
    };
    for key in map.keys() {
        if !REPORT_KEYS.contains(&key.as_str()) && !REPORT_OPTIONAL_KEYS.contains(&key.as_str()) {
            return Err(Fail::Thrown(format!(
                "cells finish: --report has unknown key \"{key}\" — only {} (plus the optional {}) are allowed.",
                REPORT_KEYS.join(", "),
                REPORT_OPTIONAL_KEYS.join(", ")
            )));
        }
    }
    for key in REPORT_KEYS {
        if !map.contains_key(key) {
            return Err(Fail::Thrown(format!(
                "cells finish: --report is missing required key \"{key}\"."
            )));
        }
    }
    match map.get("outcome") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"outcome\" must be a non-empty string.".to_string(),
            ))
        }
    }
    match map.get("commit") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"commit\" must be a non-empty string.".to_string(),
            ))
        }
    }
    match map.get("files") {
        Some(Value::Array(_)) => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"files\" must be an array.".to_string(),
            ))
        }
    }
    match map.get("deviations") {
        Some(Value::Array(_)) => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"deviations\" must be an array.".to_string(),
            ))
        }
    }
    // reflection-becomes-lesson: `mistakes` is optional, but a `mistakes` that
    // is not an array is a typo, not an answer — and the shape is refused here
    // rather than silently read as "no mistakes", which would turn a worker's
    // written mistake into the clean-run statement. This widens no existing
    // refusal: before this key existed, ANY spelling of it was refused
    // outright as an unknown key.
    match map.get("mistakes") {
        None | Some(Value::Array(_)) => {}
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"mistakes\" must be an array — one entry per mistake, each \"<what went wrong> — <what would have been better>\"; an empty array states that this cell hit none.".to_string(),
            ))
        }
    }
    // D8 (docs/history/test-doctrine/CONTEXT.md): a cap's own proof lives
    // WITH the cell, not in a fixed enum — `tests` is a proof string
    // `<command> — <result> — <scope reason>`, written by the agent that
    // ran it. The retired `boundary`/`undeclared` enum (decision 13ce1858,
    // test-cadence-boundary D1a — "a cap never claims green/red, the
    // boundary is the only place a test process runs") is refused by name
    // with a remedy teaching the new form, so a cold worker learns the
    // proof-string contract instead of guessing why the old value stopped
    // working. A well-formed proof string whose result segment reads
    // literally `red` still refuses — D6's spirit: a red is fix-first,
    // never a done.
    match map.get("tests") {
        Some(Value::String(s)) if s == "boundary" || s == "undeclared" => {
            return Err(Fail::Thrown(format!(
                "cells finish: --report key \"tests\" no longer accepts \"{s}\" — the boundary/undeclared enum is retired. Record a proof string instead: \"<command> — <result> — <scope reason>\" (e.g. \"cargo test -p bee — green:unit — touched close.rs\"). In a no-test-sentinel repo, name the command segment \"none\" and put the parity/docs proof used in the reason segment (e.g. \"none — green:static — regen parity check only\")."
            )))
        }
        Some(Value::String(s)) => match parse_tests_proof(s) {
            Some((_, result, _)) if result == "red" => {
                return Err(Fail::Thrown(
                    "cells finish: --report key \"tests\" result segment is \"red\" — a red is fix-first, never a cap. Fix the failure, re-run the proof, and cap with a passing result.".to_string(),
                ))
            }
            // D1 (docs/history/proof-strength-and-expiry/CONTEXT.md): the
            // result segment is closed over `PROOF_RESULT_VALUES`, checked
            // HERE — on the tuple, beside `red`, write path only — so the
            // read path keeps accepting historical bare-`green` caps (D2).
            // The refusal names the whole legal set, `ROUTE_CLASS_VALUES`
            // style, because this message is what a cold worker fixes from.
            Some((_, result, _)) if !PROOF_RESULT_VALUES.contains(&result.as_str()) => {
                return Err(Fail::Thrown(format!(
                    "cells finish: --report key \"tests\" result segment is \"{result}\" — a cap records HOW the change was shown to work, so the result must be one of {} (a bare \"green\" no longer says which).",
                    PROOF_RESULT_VALUES.join(", ")
                )))
            }
            Some(_) => {}
            None => {
                return Err(Fail::Thrown(
                    "cells finish: --report key \"tests\" must be a proof string \"<command> — <result> — <scope reason>\" — three non-empty segments separated by \" — \" (e.g. \"cargo test -p bee — green:unit — touched close.rs\")."
                        .to_string(),
                ))
            }
        },
        _ => {
            return Err(Fail::Thrown(
                "cells finish: --report key \"tests\" must be a proof string \"<command> — <result> — <scope reason>\"."
                    .to_string(),
            ))
        }
    }
    Ok(Value::Object(map))
}

// ─── D6 (docs/history/hook-teeth/CONTEXT.md) — the cell commit trailer ────
//
// `cells finish` refuses to cap a cell whose files_changed is non-empty
// unless a commit carrying a `cell: <id>` trailer line exists in the RECENT
// history of the feature's own branch — bee's one-commit-per-cell rule
// (AGENTS.md "Care for the session"), made mechanical. `--commit-pending
// <reason>` escapes it, D2's `--fix-first` convention: the reason lands on
// the cap's own trace, never silently.
//
// Branch resolution matters here in a way it would not for an arbitrary
// feature: THIS very feature's `cells finish` calls typically run from the
// MAIN checkout (bee-swarming's worker convention dispatches a worker whose
// `bee cells finish` call reaches the shared store), while the worker's own
// commits land on the FEATURE's WORKTREE branch — never on main. Scanning
// `root`'s own HEAD history in that case would see the wrong branch
// entirely and refuse every legitimate cap. `find_granted_worktree_for_feature`
// (verbs/status_full/topology.rs, landed for ct-1's re-lane worktree block —
// see state_group/workflows.rs's `route_worktree_block`) is exactly the
// bidirectional gitdir walk that answers "does this feature have a granted
// worktree, and where is it" without a second implementation of that
// resolution, so it is reused here rather than re-derived. When no grant
// exists for the feature (an ordinary same-checkout feature, or one with no
// worktree split at all), the CURRENT checkout's own HEAD history is the
// right — and only — thing to scan.

/// The bounded window D6 scans, nearest-to-HEAD first (`git log`'s own
/// order): `cells finish` cap-checks a cell that was JUST claimed and
/// worked, so the qualifying commit is always near HEAD. Walking the WHOLE
/// branch history would cost real time on a long-lived repo for zero added
/// correctness — the same "bounded window" tradeoff D2's red-base read
/// (`.bee/logs/test-results.json`, a single record rather than a log) makes
/// in its own idiom.
pub(crate) const COMMIT_TRAILER_WINDOW: u32 = 50;

/// `cell: <id>` — the ONE trailer shape D6 recognizes, matched as an exact
/// (trimmed) whole line inside a commit's FULL message body, never a
/// substring match: a commit that merely mentions the cell id in prose
/// ("touch up bh-6 handling") must not satisfy this.
pub(crate) fn cell_commit_trailer(id: &str) -> String {
    format!("cell: {id}")
}

/// The history root D6 scans for `feature`: the granted worktree's HEAD
/// history when one exists for it, else `fallback_root`'s own HEAD history.
/// `fallback_root` is the store root `cap_cell_from_flags` already has in
/// hand — the ordinary-checkout case needs no extra resolution at all.
pub(crate) fn commit_trailer_history_root(fallback_root: &Path, feature: Option<&str>) -> PathBuf {
    if let Some(feature) = feature {
        if let Some((_, worktree_root)) =
            crate::verbs::status_full::find_granted_worktree_for_feature(fallback_root, feature)
        {
            return PathBuf::from(worktree_root);
        }
    }
    fallback_root.to_path_buf()
}

/// `git log -n <window> --format=%B%x00` over `cwd`'s HEAD. `%B` is the
/// full, UNWRAPPED message body (never `%s`/`%b`'s subject/body split, which
/// could fold a trailer line into the subject and hide it from a
/// line-by-line scan); the `%x00` separator keeps one commit's body from
/// bleeding into the next when splitting the combined output back apart. A
/// git that cannot be spawned, a `cwd` with no commits yet, or any other
/// non-zero exit answers `false` — fail CLOSED, matching this check's own
/// "unless --commit-pending" contract: an unprovable history is never
/// silently treated as satisfying the rule, it is proven or escaped.
///
/// Shells out through `crate::verbs::worktree::run_git` — the crate's own
/// `spawnSync('git', argv, {cwd})` pattern (worktree-store.mjs runGit, no
/// shell involved) — rather than a second ad-hoc git invocation style.
pub(crate) fn commit_trailer_present(cwd: &Path, id: &str) -> bool {
    let window = COMMIT_TRAILER_WINDOW.to_string();
    let out = crate::verbs::worktree::run_git(cwd, &["log", "-n", &window, "--format=%B%x00"]);
    if out.status != Some(0) {
        return false;
    }
    let trailer = cell_commit_trailer(id);
    out.stdout
        .unwrap_or_default()
        .split('\u{0}')
        .any(|body| body.lines().any(|line| js_trim(line) == trailer))
}

// ─── FULL-door topology (wf-1) ──────────────────────────────────────────────
// `cells finish`'s own split of `resolve_store_root_worktree`'s
// `StoreRoots`, per the logged decision: the cell record and its claim
// resolve at MAIN — one ledger, and the claim being validated already lives
// there; the hold-release topology is `StoreRoots::hold_topology()`
// unchanged (roots.rs:541-551). decision 13ce1858 (test-cadence-boundary
// D1) dropped this function's third return value, the calling worktree's
// own root — it existed only as the declared test command's cwd, and the
// cap no longer runs that command at all. A pure function of `StoreRoots`
// — no cwd read — so it is exercisable directly against a
// `resolve_store_root_worktree` fixture, the same shape
// reservations/tests.rs's `hold_topology_matches_node_for_every_checkout_kind`
// already uses.
pub(crate) fn finish_topology(roots: &StoreRoots) -> (PathBuf, Option<(PathBuf, String)>) {
    let cells_root = roots.main_root();
    let topo = roots.hold_topology();
    (cells_root, topo)
}

// ─── reservations-release subset (finish's release half) ───────────────────
// Provenance: lib/reservations.mjs release/listReservations + bee.mjs
// releaseReservationsForAgent, mirrored from verbs/reservations.rs's own
// release_exec (those fns are module-private there; this copy keeps cells.rs
// self-contained per the one-file rule).

pub(crate) const CROSS_WORKTREE_HOLDS_LOCK: &str = "cross-worktree-holds";

pub(crate) fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn holds_ledger_path(root: &Path) -> PathBuf {
    root.join(".bee").join("runtime").join("cross-worktree-holds.json")
}

/// worktree-holds.mjs readStore — `readJson(path, null)` then a shape check
/// that turns anything without an array `holds` into `{holds: []}`. A corrupt
/// ledger warns and takes that same `{holds: []}` fallback (Node's `null`
/// fallback reached it through `!store`). Null hold ENTRIES still delegate:
/// that is a JS-exotic shape, not a parse failure.
pub(crate) fn read_holds_store(root: &Path) -> MR<Value> {
    let ledger = holds_ledger_path(root);
    let store = match read_json(&ledger) {
        ReadJson::Missing => None,
        ReadJson::Corrupt => {
            warn_corrupt_json_once(&ledger);
            None
        }
        ReadJson::Parsed(v) => Some(rsv::js_numberify(&v).map_err(|_| Fail::Delegate)?),
    };
    let ok_shape = store
        .as_ref()
        .map(|s| matches!(s.get("holds"), Some(Value::Array(_))))
        .unwrap_or(false);
    if !ok_shape {
        return Ok(json!({ "holds": [] }));
    }
    let store = store.unwrap();
    if let Some(Value::Array(holds)) = store.get("holds") {
        if holds.iter().any(|h| h.is_null()) {
            return Err(Fail::Delegate);
        }
    }
    Ok(store)
}

pub(crate) fn list_path_lease_records(root: &Path) -> MR<Vec<Map<String, Value>>> {
    let control = control_root(root)?;
    let leases_root = control.join(".bee").join("runtime").join("leases");
    let mut out = Vec::new();
    for dir in [leases_root.join("cells"), leases_root.join("paths")] {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            if !is_file {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(dir.join(&name)) {
                if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                    // readLeaseSafe: corrupt silently skipped (no warn in Node).
                    if let Value::Object(m) = rsv::js_numberify(&parsed).map_err(|_| Fail::Delegate)? {
                        let is_path =
                            matches!(m.get("resource"), Some(Value::String(s)) if s.starts_with("path:"));
                        if is_path {
                            out.push(m);
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

pub(crate) fn path_lease_file(control_root: &Path, raw_path_id: &str) -> PathBuf {
    let canonical = rsv::res_normalize_path(raw_path_id);
    let resource_key = format!("path:{canonical}");
    control_root
        .join(".bee")
        .join("runtime")
        .join("leases")
        .join("paths")
        .join(format!("{}.json", sha256_hex(&resource_key)))
}

pub(crate) fn lease_record_expired(rec: &Map<String, Value>, now: f64) -> MR<bool> {
    match rec.get("expires_at") {
        None | Some(Value::Null) => Ok(false),
        Some(v) => match rsv::date_parse_val(Some(v)).map_err(|_| Fail::Delegate)? {
            None => Ok(false),
            Some(ms) => Ok(ms <= now),
        },
    }
}

pub(crate) struct ResvLite {
    pub(crate) agent: Option<Value>,
    pub(crate) cell: Option<Value>,
    pub(crate) path: String,
    pub(crate) session: Option<Value>,
}

pub(crate) fn lease_to_resv_lite(rec: &Map<String, Value>) -> MR<ResvLite> {
    let resource = match rec.get("resource") {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(Fail::Delegate),
    };
    let agent = rec.get("workspace_id").map(|w| match w {
        Value::String(s) if s.starts_with("agent:") => Value::String(s["agent:".len()..].to_string()),
        other => other.clone(),
    });
    let session = match rec.get("session_id") {
        Some(v) if js_truthy(v) && !matches!(v, Value::String(s) if s == rsv::SESSIONLESS_SESSION_ID) => {
            Some(v.clone())
        }
        _ => None,
    };
    Ok(ResvLite {
        agent,
        cell: rec.get("workflow_id").cloned(),
        path: resource["path:".len()..].to_string(),
        session,
    })
}

pub(crate) struct ReleaseOutcome {
    pub(crate) paths: Vec<String>,
    /// Mirrors Node's holdsReleased (reservations-release parity); the finish
    /// text/result never surfaces it — kept for the ledger write's own count.
    #[allow(dead_code)]
    pub(crate) holds_released: u64,
}

/// bee.mjs releaseReservationsForAgent(root, agent, cell) — matched-rows
/// derivation, local lease release, {cell, session}-scoped ledger release.
///
/// `topo` is `StoreRoots::hold_topology()` (wf-1) — `(main_root, holder)`,
/// `None` for an ungranted linked worktree — replacing the ordinary-only
/// assumption (ledger at `root`, holder hardcoded `"main"`) this used to
/// carry unconditionally, matching `verbs/reservations/release.rs`'s own
/// `release_exec` topology gate: no topology, no lock, no ledger read, the
/// ledger step skipped entirely rather than silently guarding an empty one.
/// The local lease release (`control_root(root)`) is unaffected — leases
/// always live off `root`, independent of the ledger's own home.
pub(crate) fn release_reservations_for_agent(
    topo: Option<(&Path, &str)>,
    root: &Path,
    agent: &str,
    cell_id: &str,
) -> MR<ReleaseOutcome> {
    let now = rsv::now_ms();
    let records = list_path_lease_records(root)?;
    let mut matched: Vec<ResvLite> = Vec::new();
    for rec in &records {
        if lease_record_expired(rec, now)? {
            continue; // activeOnly
        }
        let resv = lease_to_resv_lite(rec)?;
        let agent_match = matches!(&resv.agent, Some(Value::String(s)) if s == agent);
        let cell_match =
            matches!(&resv.cell, Some(v) if v == &Value::String(cell_id.to_string()));
        if agent_match && cell_match {
            matched.push(resv);
        }
    }
    let mut pairs: Vec<(Value, Option<Value>)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for r in &matched {
        let Some(cell_v) = r.cell.as_ref().filter(|c| js_truthy(c)) else { continue };
        let session_v = r.session.as_ref().filter(|s| js_truthy(s)).cloned();
        let key = format!(
            "{}::{}",
            jsjson::js_to_string(cell_v),
            session_v.as_ref().map(|s| jsjson::js_to_string(s)).unwrap_or_default()
        );
        if !seen.contains(&key) {
            seen.push(key);
            pairs.push((cell_v.clone(), session_v));
        }
    }

    // reservations.mjs release(root, {agent, cell}).
    let control = control_root(root)?;
    let trimmed_agent = js_trim(agent);
    for rec in &records {
        let lease_agent = match rec.get("workspace_id") {
            Some(Value::String(s)) if s.starts_with("agent:") => {
                Value::String(s["agent:".len()..].to_string())
            }
            Some(other) => other.clone(),
            None => continue,
        };
        if !matches!(&lease_agent, Value::String(s) if s == trimmed_agent) {
            continue;
        }
        let matches_cell = matches!(
            rec.get("workflow_id"),
            Some(v) if v == &Value::String(cell_id.to_string())
        );
        if !matches_cell {
            continue;
        }
        let resource = match rec.get("resource") {
            Some(Value::String(s)) => s.clone(),
            _ => continue,
        };
        let file = path_lease_file(&control, &resource["path:".len()..]);
        match std::fs::remove_file(&file) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Fail::Thrown(format!("{e}"))),
        }
    }

    // xwh-2/gfb-1: ledger release per {cell, session} pair, gated on a
    // hold-worthy topology (wf-1). `None` — an ungranted linked worktree —
    // skips the whole ledger step, same as reservations::release_exec.
    let mut holds_released: u64 = 0;
    if let Some((main_root, holder)) = topo {
        for (cell_v, session_v) in &pairs {
            // hha-2 (docs/history/hold-holder-attribution/plan.md): the rows
            // this cap clears belong to whoever owns the CELL, not to the
            // checkout typing `cells finish` — and that command typically
            // runs from MAIN for a cell whose work happened inside a granted
            // worktree (see the D6 note above), which is exactly the row hha-1
            // now stamps with that worktree. Filtering by the acting holder
            // here would make those rows ones main could never clear.
            let owner =
                rsv::cell_hold_owner(main_root, holder, &jsjson::js_to_string(cell_v))
                    .map_err(|_| Fail::Delegate)?;
            let mut guard = acquire_named_lock(main_root, CROSS_WORKTREE_HOLDS_LOCK)?;
            let outcome = (|| -> MR<u64> {
                let mut store = read_holds_store(main_root)?;
                let released_at = utc_now();
                let mut count: u64 = 0;
                if let Some(Value::Array(holds)) = store.get_mut("holds") {
                    for hold in holds.iter_mut() {
                        let unreleased = matches!(hold.get("released_at"), None | Some(Value::Null));
                        if !unreleased {
                            continue;
                        }
                        if !matches!(hold.get("holder"), Some(Value::String(s)) if s == &owner.holder)
                        {
                            continue;
                        }
                        if let Some(s) = session_v {
                            if !matches!(hold.get("session"), Some(v) if v == s) {
                                continue;
                            }
                        }
                        if !matches!(hold.get("cell"), Some(v) if v == cell_v) {
                            continue;
                        }
                        if let Value::Object(m) = hold {
                            m.insert("released_at".into(), Value::String(released_at.clone()));
                        }
                        count += 1;
                    }
                }
                if count > 0 {
                    write_json_atomic(&holds_ledger_path(main_root), &store)
                        .map_err(|e| Fail::Thrown(format!("{e}")))?;
                }
                Ok(count)
            })();
            guard.release();
            holds_released += outcome?;
        }
    }

    let mut paths: Vec<String> = Vec::new();
    for r in &matched {
        if !paths.contains(&r.path) {
            paths.push(r.path.clone());
        }
    }
    Ok(ReleaseOutcome { paths, holds_released })
}

// RETIRED at the R6 cutover: the cap-time impact-registry cross-check (E1).
// It queried `scripts/impact-registry.json` — a suite-impact graph derived
// by parsing `scripts/run_verify.mjs` and the `.mjs` import closure — to warn
// when a cell's verify command missed a direct-edge Node suite. Both the
// registry and the graph it was derived from are gone with the Node tree, and
// the cargo suite that replaced them runs whole in ~20s, so there is no
// filtering left to advise about. `trace.warnings` keeps its slot (existing
// capped cells carry it) but this producer no longer exists.

// ─── fa-1: diff-vs-test advisory ────────────────────────────────────────────
// A NEW producer for the `trace.warnings` slot the E1 retirement above left
// empty: a large commit that touches no test-shaped path is worth a nudge.
// Runs at the GREEN-cap path of `cells finish` alone (the caller gates this
// on `finish`, mirroring D6's own "finish only" scoping) — every earlier
// door (test-green, commit-trailer, lane/worker checks) already let the cap
// through by the time this producer runs, so it can only ever ADD a line to
// `trace.warnings` and print that SAME line once to stderr; it never touches
// the exit code, the cap outcome, or any other part of the result shape.
// Every git failure — no git on PATH, HEAD's body doesn't carry THIS cell's
// own `cell: <id>` trailer, numstat unreadable — is a silent skip: this is
// best-effort telemetry, never a second gate alongside D6's real one.

/// The default for config key `finish.advisory_untested_lines` —
/// `{finish: {advisory_untested_lines: N}}` in `.bee/config.json`, the same
/// nested-object shape `guards.write_policy`
/// (write_guard/checks.rs `resolve_write_policy_mode`) already establishes
/// for a dotted doc-facing key name. `0` is the documented disable, never a
/// "run always" typo — the producer checks for it explicitly before doing
/// any git work.
pub(crate) const DEFAULT_ADVISORY_UNTESTED_LINES: u64 = 150;

/// advisoryUntestedLinesThreshold — absent/null/non-numeric/negative all
/// fall back to the default silently (`resolve_write_policy_mode`'s own
/// posture for a single-scalar nested key, not `capture_queue_threshold`'s
/// warn-and-fallback: a malformed advisory threshold is worth exactly one
/// nudge line, never a stderr warning of its own).
pub(crate) fn advisory_untested_lines_threshold(config: &Map<String, Value>) -> u64 {
    match config.get("finish").and_then(|f| f.get("advisory_untested_lines")) {
        Some(Value::Number(n)) => match n.as_u64() {
            Some(v) => v,
            None => DEFAULT_ADVISORY_UNTESTED_LINES,
        },
        _ => DEFAULT_ADVISORY_UNTESTED_LINES,
    }
}

/// headCommitCarriesTrailer — unlike [`commit_trailer_present`]'s
/// `COMMIT_TRAILER_WINDOW`-commit scan (D6 only needs to prove a qualifying
/// commit exists SOMEWHERE near HEAD), the advisory describes the diff of
/// the ONE commit the one-commit-per-cell convention names as THIS cell's
/// own: HEAD, and HEAD alone. A HEAD whose body does not carry the exact
/// trailer line (an unrelated commit landed on top, `--commit-pending`
/// escaped D6's own check entirely, this checkout has no commits yet, git
/// itself is missing) answers `false` — the producer only ever describes a
/// commit it is confident belongs to `id`.
pub(crate) fn head_commit_carries_trailer(cwd: &Path, id: &str) -> bool {
    let out = crate::verbs::worktree::run_git(cwd, &["log", "-1", "--format=%B"]);
    if out.status != Some(0) {
        return false;
    }
    let trailer = cell_commit_trailer(id);
    out.stdout.unwrap_or_default().lines().any(|line| js_trim(line) == trailer)
}

/// One `git show --numstat` row: `<added>\t<deleted>\t<path>`.
pub(crate) struct NumstatRow {
    pub(crate) added: u64,
    pub(crate) deleted: u64,
    pub(crate) path: String,
}

/// headCommitNumstat — `git show -1 --numstat --format=` HEAD: one row per
/// changed path, no commit-message preamble (`--format=` empty — the same
/// "spawn once, parse exactly what was asked for" posture
/// [`commit_trailer_present`]'s `%B`-only spawn already takes, rather than a
/// second ad-hoc parse of git's default log format). A binary file's
/// `-\t-\tpath` row parses both fields as `0`: line counts are not
/// observable for it, and treating the dash as "large" would be a false
/// positive this producer has no way to justify. `None` on any non-zero git
/// exit (no git, no HEAD, a detached/empty repo) — the caller treats that
/// exactly like every other silent-skip path.
pub(crate) fn head_commit_numstat(cwd: &Path) -> Option<Vec<NumstatRow>> {
    let out = crate::verbs::worktree::run_git(cwd, &["show", "-1", "--numstat", "--format="]);
    if out.status != Some(0) {
        return None;
    }
    let mut rows = Vec::new();
    for line in out.stdout.unwrap_or_default().lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(3, '\t');
        let (Some(added), Some(deleted), Some(path)) = (parts.next(), parts.next(), parts.next()) else {
            continue; // an unparsable row is skipped, not fatal to the whole read
        };
        rows.push(NumstatRow {
            added: added.parse().unwrap_or(0),
            deleted: deleted.parse().unwrap_or(0),
            path: path.to_string(),
        });
    }
    Some(rows)
}

/// pathLooksLikeTest — four shapes, any one of which qualifies a changed
/// path as test-shaped: a path SEGMENT (a `/`-split component, so
/// `contest/a.rs` and `latest.rs` never false-positive on a `test`
/// substring) named exactly `test` or `tests`; the bare filename
/// `tests.rs`; or a filename carrying `_test.` / `.test.` anywhere before
/// its extension (`foo_test.rs`, `foo.test.ts`).
pub(crate) fn path_looks_like_test(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let segments: Vec<&str> = normalized.split('/').collect();
    if segments.iter().any(|s| *s == "test" || *s == "tests") {
        return true;
    }
    let filename = segments.last().copied().unwrap_or("");
    filename == "tests.rs" || filename.contains("_test.") || filename.contains(".test.")
}

/// advisoryUntestedLinesLine — the ONE stderr line this producer ever
/// prints, and byte-identical to the ONE line appended to `trace.warnings`
/// (a single representation, never a stderr copy plus a differently-worded
/// trace copy).
pub(crate) fn advisory_untested_lines_line(total_lines: u64, threshold: u64) -> String {
    format!(
        "advisory: this cap's commit changes {total_lines} line(s) (over the {threshold}-line finish.advisory_untested_lines threshold) but touches no test-shaped path — consider adding test coverage."
    )
}

/// diffVsTestAdvisory — the producer itself. `None` on every silent-skip
/// path: the threshold is `0` (disabled), HEAD doesn't carry `id`'s own
/// trailer, numstat is unreadable, at least one changed path already looks
/// test-shaped, or the total changed-line count does not exceed the
/// threshold. `Some(line)` is the one line the caller both prints to
/// stderr and appends to `trace.warnings` — this function has no other
/// side effect and never returns an `Err`.
pub(crate) fn diff_vs_test_advisory(cwd: &Path, id: &str, threshold: u64) -> Option<String> {
    if threshold == 0 {
        return None;
    }
    if !head_commit_carries_trailer(cwd, id) {
        return None;
    }
    let rows = head_commit_numstat(cwd)?;
    if rows.iter().any(|r| path_looks_like_test(&r.path)) {
        return None;
    }
    let total: u64 = rows.iter().map(|r| r.added + r.deleted).sum();
    if total <= threshold {
        return None;
    }
    Some(advisory_untested_lines_line(total, threshold))
}

// ─── delegation pre-scans ──────────────────────────────────────────────────
// The mutators must never return None after an output or a write, so the
// JS-exotic store shapes that still delegate (an array where a record is
// expected, a string/array `trace`) are probed up front; Thrown-class
// outcomes are ignored here (the real flow reproduces them at Node's own
// point in the order).
//
// CUTOVER: `prescan_cells_store` used to walk EVERY active and archived cell
// file so a corrupt one could delegate before any output. Corrupt JSON is
// native now, and that walk would have warned about cell files the command
// never reads — so it is deleted rather than kept as a second, louder read.
// The probes that survive read exactly the file the flow reads, and
// `warn_corrupt_json_once` keeps that from printing twice.

pub(crate) fn delegate_only<T>(result: MR<T>) -> MR<()> {
    match result {
        Err(Fail::Delegate) => Err(Fail::Delegate),
        _ => Ok(()),
    }
}

pub(crate) fn prescan_claim(root: &Path, id: &str) -> MR<()> {
    let control = control_root(root)?;
    if id_pattern_ok(id) {
        delegate_only(read_claim(&control, id))?;
    }
    Ok(())
}

// ─── slp-advisor-nudge an-3: the cap path's advisor-nudge arm (9e5eda5b) ───

/// Pinned prefix of the cap-path advisor-nudge refusal headline
/// (message-contract test: `cap_refuses_while_an_advisor_nudge_for_the_cells_
/// feature_is_unanswered`, verbs/cells/tests.rs). It reads `capCell: …` like
/// every other refusal this door emits, rather than borrowing the close
/// door's own `CLOSE_ADVISOR_NUDGE_DEBT_PREFIX`: a reader looking at a
/// refused cap must see WHICH verb refused, in its own voice.
pub(crate) const CAP_ADVISOR_NUDGE_DEBT_PREFIX: &str = "capCell: advisor nudge debt for";

/// The cap door's advisor-nudge refusal text, or `None` when this cell's
/// feature owes nothing. 9e5eda5b arms the debt at the CELL level too, and
/// this is that tooth: the cap is where the obligation bites first, long
/// before `bee close` or `bee worktree merge` ever run.
///
/// The count is `feature_advisor_nudge_debt` (verbs/supervisor.rs) and
/// nothing else — the SAME function both boundary doors call. One obligation
/// read three ways would be three obligations; what is shared is the count,
/// and each door writes its own prose.
///
/// An empty feature name owes nothing: a nudge row carries no feature when
/// its target held no claim (423871d7), so "no name" already means "counts
/// against nothing" in the store, and asking the debt about `""` must not
/// invent an answer the record cannot support.
pub(crate) fn advisor_nudge_cap_refusal(root: &Path, id: &str, feature: &str) -> MR<Option<String>> {
    if feature.is_empty() {
        return Ok(None);
    }
    let debt = crate::verbs::supervisor::feature_advisor_nudge_debt(root, feature)
        .map_err(|_: crate::verbs::drivers::Delegate| Fail::Delegate)?;
    if debt.count == 0 {
        return Ok(None);
    }
    Ok(Some(
        [
            format!(
                "{CAP_ADVISOR_NUDGE_DEBT_PREFIX} \"{feature}\" — cell \"{id}\" cannot cap: {} advisor nudge(s) with no consult and no recorded decline ({}).",
                debt.count,
                crate::verbs::drivers::js_join(&debt.ids, ", ")
            ),
            "remedy: run the advisor consult for each row above, then record what came of it with bee decisions log --tags advisor-nudge — or record a reasoned decline the same way. The decision text must NAME the row id; one decision answers one row, and a decision naming no row clears nothing.".to_string(),
            format!("next: settle the advisor-nudge debt above, then re-run the cap for {id}"),
        ]
        .join("\n"),
    ))
}
