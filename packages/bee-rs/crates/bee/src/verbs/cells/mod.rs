// bee cells — natively served slice.
//
// READ-ONLY (unchanged from R3 wave 1): `cells list`, `cells ready`,
// `cells show` (flags: --json; --feature/--status on list; --feature on
// ready; --id on show).
//
// SELECTING (R6): `cells claim-next` — the sweep, resolvePipeline, the
// cross-lane pool and the hold filters, then the shared claim half.
//
// MUTATING (this wave): add, update, claim, cap, finish, block, drop,
// unclaim, reopen, tier, judge, reset-budget, judge-record, schedule,
// archive, unarchive — each mirroring bee.mjs's dispatch frame (root resolve
// -> manifest-drift check -> handler -> emit/emitError -> timing) and the
// lib/cells.mjs mutators behind it: the same `cells:<id>` /
// `cells-archive` / `decisions` store-lock names (crate::lock — identical
// name strings, so Node and Rust serialize against each other), the same
// claims-store O_EXCL protocol (claimCellFile: fence_epoch 1 at creation,
// session resolution flag -> BEE_SESSION_ID -> CLAUDE_CODE_SESSION_ID ->
// single-live-session adoption), and atomic cell writes through
// crate::fsutil::write_json_atomic via the writeCell funnel (brief
// 'cells-archive' acquire + archived-only re-check, typed
// CELLS_ARCHIVE_BUSY on contention).
//
// wf-1 — `cells finish` alone is worktree-native. Every mutating verb above
// (including `cap`) dispatches through the shared `dispatch`
// (handlers_write.rs) -> `rsv::prelude` -> `resolve_store_root`, the NARROW
// door (roots.rs "the three doors"): it refuses a granted linked worktree by
// name, naming the main checkout. `finish` alone routes through its own
// `run_finish` (handlers_close.rs), the FULL door
// (`resolve_store_root_worktree`): the cell record and its claim still
// resolve at `StoreRoots::main_root()` (one ledger — this is not a second
// per-worktree cell store), the declared test command's cwd is the calling
// worktree when granted, and reservation/hold release threads
// `StoreRoots::hold_topology()` (`finish_topology`, finish_support.rs).
// Running `finish` from the main checkout is unchanged: `roots.linked` is
// `None` there, so the FULL door answers exactly what the narrow one did.
//
// R6 — `cells claim-next` IS NOW NATIVE (the last cells debt). All four
// pieces the previous header listed as missing are ported, in one piece so
// the sweep never half-runs:
//   1. sweepExpiredClaims (claims.mjs): the per-claim `.adopting` gate, the
//      `sessions` store lock around the heartbeat re-verify, the claim-file
//      removal, the claimed->open reset under `cells:<id>` (trace stamped
//      swept_at/swept_from_session), and one best-effort logDecision row per
//      actual reset.
//   2. resolvePipeline (state.mjs) — session -> bound lane -> default, with
//      the four typed LANE_INVALID/LANE_MISSING/LANE_CORRUPT refusals.
//   3. the pooling pass — readState + listLanes + listSessionRecords/
//      heartbeatStale (GH#20 live-owner skip) + featureBacklogRank
//      (verbs/backlog.rs, both the docs/backlog.md Feature-column walk and
//      the PBI fold's `a.id.localeCompare(b.id)` arm) + the created_at
//      tiebreak.
//   4. the per-candidate filters — findSessionConflicts (path leases) and
//      findForeignHolds over resolveHoldTopology's ordinary arm.
// The old "a partial port cannot fall back to Node afterwards" objection is
// answered, not ignored: the sweep removes its own trigger, so a Node re-run
// after a mid-flight delegate re-derives the identical end state and bytes.
// See the `cells claim-next` section comment for the full argument.
//
// STILL DELEGATED (file-header contract):
//   - every argv shape any ported verb cannot PROVE: unknown flags, missing
//     required flags, --help, bad enum/number values (Node's validate()
//     speaks there), non-flag tokens, non-UTF-8 argv.
//   - JS-exotic store shapes this port cannot carry: an array where a cell/
//     claim/session/lane record is expected (`typeof [] === 'object'` lets
//     them through Node's guards into index-key spreads), a string/array
//     `trace`, a non-string `feature` feeding path math. These delegate
//     BEFORE any output or write — the drift-cache write is the one
//     sanctioned pre-None write, exactly like the read-only slice.
//
// CUTOVER (2026-08-01) — CORRUPT JSON IS NATIVE. Contract C2 (byte-identical
// output with Node) is retired with the Node runtime, so the arms that used
// to hand a corrupt store back to Node — because Node's readJson warning
// interpolated V8's own `JSON.parse` message — now do the work here:
//   - readJson-backed reads (`read_cell_json` / `read_store_json`) warn via
//     crate::fsutil::warn_corrupt_json and take the SAME fallback Node's
//     readJson took (null / {} / the caller's default). Fail-open stays
//     fail-open; nothing that refused before stops refusing.
//   - the strict readers (readLaneStrict, readCellStrictForUpdate,
//     recoverArchiveJournal) keep their own deterministic refusals, and the
//     unreadable-file branches that used to embed a libuv errno now carry the
//     Rust io error in the same sentence.
//   - lone-surrogate escapes (`\uD800`-`\uDFFF`), which V8's JSON.parse
//     accepted and serde refuses, are simply CORRUPT now: there is no second
//     parser to defer to, so each site takes its own not-valid-JSON path.
//   - |n| >= 1e21 no longer diverges at all — jsjson::js_f64_to_string
//     implements the spec's exponential forms, so those arms are gone.
// The pre-scan that walked the whole cell store just to make that delegation
// decision is gone with them (it would have warned about files the command
// never reads); `warn_corrupt_json_once` keeps the surviving probes from
// double-warning about a file the real flow reads again.
//
// DOCUMENTED RESIDUAL DIVERGENCES (all pathological, none reachable from
// well-formed bee stores; each noted again at its code site):
//   - hard mid-transaction filesystem failures (a failing rename inside the
//     archive loop, a failing final writeCell): Node embedded the libuv errno
//     message; the native error text carries the Rust io message instead.
//   - a store file that turns corrupt in the window between a surviving
//     probe and a post-test re-read (cap/finish only) warns once, not twice —
//     `warn_corrupt_json_once` dedupes per path.
//   - declared test commands producing > 64 MiB of combined output: Node's
//     spawnSync maxBuffer kills the child (spawn error); the native runner
//     captures it all.

use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

/// Sentinel: this argv/store shape belongs to the Node runtime.
#[derive(Debug)]
pub(crate) struct Delegate;

#[derive(Clone, Copy, PartialEq)]
enum Verb {
    List,
    Ready,
    Show,
}

impl Verb {
    /// The dispatcher's timing label: `commandName.split('.').join(' ')`.
    fn cmd(self) -> &'static str {
        match self {
            Verb::List => "cells list",
            Verb::Ready => "cells ready",
            Verb::Show => "cells show",
        }
    }
}

#[derive(Default)]
struct Flags {
    json: bool,
    feature: Option<String>,
    status: Option<String>,
    id: Option<String>,
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "cells" {
        return None;
    }
    let verb = match args.get(1)?.to_str()? {
        "list" => Verb::List,
        "ready" => Verb::Ready,
        "show" => Verb::Show,
        other => return try_mutating(other, &args[2..], t0),
    };
    let flags = parse_flags(verb, &args[2..])?;
    if verb == Verb::Show {
        // Missing/empty --id takes Node's validate() "required, missing"
        // emission path (stdout-shaped, drift-line-bearing) — delegate it.
        if flags.id.as_deref().map(str::is_empty).unwrap_or(true) {
            return None;
        }
    }
    run(verb, flags, t0)
}

/// bee.mjs parseFlags, narrowed to the three verbs' own registry flags.
///
/// Provenance (bee.mjs parseFlags + FLAG_ALONE_BOOLEANS): `--json` is
/// flag-alone (never consumes a value; `--json=<anything>` still just sets
/// json, and matches main()'s pre-parse `--json`/`--json=` scan, so
/// parsed.json == jsonRequested for every accepted shape). `--feature`/
/// `--status`/`--id` take a value: `--flag=value` inline, or the next token.
/// A next token starting with `--` WOULD be consumed as the value by Node —
/// that shape (and a missing value token, a bare positional, any unknown
/// flag such as `--help`, and non-UTF-8 argv) delegates instead. Repeated
/// flags keep Node's last-wins overwrite.
fn parse_flags(verb: Verb, tokens: &[OsString]) -> Option<Flags> {
    let mut out = Flags::default();
    let mut i = 0usize;
    while i < tokens.len() {
        let tok = tokens[i].to_str()?;
        if !tok.starts_with("--") {
            return None; // Node: parse error "unexpected argument" — delegate
        }
        let body = &tok[2..];
        let (name, inline) = match body.find('=') {
            Some(pos) => (&body[..pos], Some(body[pos + 1..].to_string())),
            None => (body, None),
        };
        if name == "json" {
            out.json = true;
            i += 1;
            continue;
        }
        let allowed = matches!(
            (verb, name),
            (Verb::List, "feature") | (Verb::List, "status") | (Verb::Ready, "feature") | (Verb::Show, "id")
        );
        if !allowed {
            return None; // unknown flag (incl. --help) — Node owns the refusal/help
        }
        let value = match inline {
            Some(v) => v,
            None => {
                let next = tokens.get(i + 1)?.to_str()?;
                if next.starts_with("--") {
                    return None; // Node would eat a flag token as the value — not proven here
                }
                i += 1;
                next.to_string()
            }
        };
        match name {
            "feature" => out.feature = Some(value),
            "status" => out.status = Some(value),
            "id" => out.id = Some(value),
            _ => unreachable!(),
        }
        i += 1;
    }
    Some(out)
}

fn run(verb: Verb, flags: Flags, t0: Instant) -> Option<ExitCode> {
    let cwd = std::env::current_dir().ok()?;
    let use_json = flags.json;

    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, verb.cmd(), use_json, t0, &why))
        }
        Roots::None => return Some(emit_no_root_error(&cwd, verb.cmd(), use_json, t0)),
    };

    let drift = check_manifest_drift(&root);

    // handleCellsList/Ready: `flags.feature ? String(flags.feature) : null` —
    // empty string is falsy, so it never filters.
    let feature = flags.feature.as_deref().filter(|s| !s.is_empty());
    let status = flags.status.as_deref().filter(|s| !s.is_empty());

    let outcome = match verb {
        Verb::List => handle_list(&root, feature, status),
        Verb::Ready => handle_ready(&root, feature),
        Verb::Show => handle_show(&root, flags.id.as_deref().unwrap_or("")),
    };

    match outcome {
        Err(Delegate) => None, // no output has happened — Node re-runs the command
        Ok(Handled::Emit { result, text }) => {
            // emit(): drift stderr line first, then the bare result on stdout.
            if drift.manifest_changed {
                eprintln!("manifest_changed: true — {}", drift.hint);
            }
            if use_json {
                println!("{}", jsjson::stringify_pretty(&result));
            } else {
                println!("{text}");
            }
            record_timing(&root, verb.cmd(), t0, true);
            Some(ExitCode::SUCCESS)
        }
        Ok(Handled::Error(message)) => {
            // emitError(): handler throw — NO drift line on this path (the
            // cache write above already happened, matching Node), --json gets
            // a compact {"error": ...} on stdout, text mode goes to stderr.
            if use_json {
                println!("{}", jsjson::stringify(&serde_json::json!({ "error": message })));
            } else {
                eprintln!("{message}");
            }
            record_timing(&root, verb.cmd(), t0, false);
            Some(ExitCode::FAILURE)
        }
    }
}

enum Handled {
    Emit { result: Value, text: String },
    Error(String),
}

// ─── bee.mjs handlers ──────────────────────────────────────────────────────

/// handleCellsList (bee.mjs): listCells with the two truthiness-normalized
/// filters; text is one summarizeCell line per cell or "No cells.".
fn handle_list(root: &Path, feature: Option<&str>, status: Option<&str>) -> Result<Handled, Delegate> {
    let cells = list_cells(root, feature, status)?;
    let text = if cells.is_empty() {
        "No cells.".to_string()
    } else {
        cells.iter().map(summarize_cell).collect::<Vec<_>>().join("\n")
    };
    Ok(Handled::Emit { result: Value::Array(cells), text })
}

/// handleCellsReady (bee.mjs): readyCells = listCells({status:'open'})
/// filtered to cells whose depsAllCapped list is empty.
fn handle_ready(root: &Path, feature: Option<&str>) -> Result<Handled, Delegate> {
    let mut ready = Vec::new();
    for cell in list_cells(root, feature, Some("open"))? {
        if deps_all_capped_is_empty(root, &cell)? {
            ready.push(cell);
        }
    }
    let text = if ready.is_empty() {
        "No ready cells.".to_string()
    } else {
        ready.iter().map(summarize_cell).collect::<Vec<_>>().join("\n")
    };
    Ok(Handled::Emit { result: Value::Array(ready), text })
}

/// handleCellsShow (bee.mjs): readCell -> not-found throw (byte-matched
/// message) -> withVerifyOwner -> {result: annotated, text: pretty JSON}.
/// Both output modes print the identical JSON.stringify(annotated, null, 2).
fn handle_show(root: &Path, id: &str) -> Result<Handled, Delegate> {
    let cell = match read_cell(root, id)? {
        None => return Ok(Handled::Error(format!("Cell \"{id}\" not found."))),
        Some(v) => v,
    };
    // A truthy non-object cell file (number/string/bool/array JSON) takes
    // Object.entries()/! coercion paths whose renders are JS-exotic — Node's.
    let Value::Object(map) = cell else { return Err(Delegate) };
    let annotated = Value::Object(with_verify_owner(&map));
    let text = jsjson::stringify_pretty(&annotated);
    Ok(Handled::Emit { result: annotated, text })
}

/// bee.mjs VERIFY_OWNER_ANNOTATION (vo-1, R82 main-verifies).
const VERIFY_OWNER_ANNOTATION: &str = "main (feature close) — the worker never runs this";

/// bee.mjs withVerifyOwner: re-build the object inserting `verify_owner`
/// immediately after the `verify` key; append at the end when the cell has
/// no `verify` key at all. Key order is otherwise the file's own (JS
/// insertion order == serde_json preserve_order).
fn with_verify_owner(cell: &Map<String, Value>) -> Map<String, Value> {
    let mut out = Map::new();
    let mut inserted = false;
    for (key, value) in cell {
        out.insert(key.clone(), value.clone());
        if key == "verify" {
            out.insert("verify_owner".into(), Value::String(VERIFY_OWNER_ANNOTATION.into()));
            inserted = true;
        }
    }
    if !inserted {
        out.insert("verify_owner".into(), Value::String(VERIFY_OWNER_ANNOTATION.into()));
    }
    out
}

/// bee.mjs summarizeCell: `${cell.id} [${cell.status}] (${cell.lane})
/// ${cell.title}` — template-literal coercion, so an absent field renders
/// "undefined", an object "[object Object]", an array its comma-join.
fn summarize_cell(cell: &Value) -> String {
    format!(
        "{} [{}] ({}) {}",
        js_string_or_undefined(cell.get("id")),
        js_string_or_undefined(cell.get("status")),
        js_string_or_undefined(cell.get("lane")),
        js_string_or_undefined(cell.get("title"))
    )
}

#[cfg(test)]
mod tests;

mod read;
mod util;
mod claims;
mod trace;
mod audit;
mod obligation;
mod validate;
mod judge;
mod schedule;
mod finish_support;
mod handlers_write;
mod handlers_select;
mod handlers_close;
mod handlers_meta;
pub(crate) use self::read::*;
pub(crate) use self::util::*;
pub(crate) use self::claims::*;
pub(crate) use self::trace::*;
pub(crate) use self::audit::*;
pub(crate) use self::obligation::*;
pub(crate) use self::validate::*;
pub(crate) use self::judge::*;
pub(crate) use self::schedule::*;
pub(crate) use self::finish_support::*;
pub(crate) use self::handlers_write::*;
pub(crate) use self::handlers_select::*;
pub(crate) use self::handlers_close::*;
pub(crate) use self::handlers_meta::*;
