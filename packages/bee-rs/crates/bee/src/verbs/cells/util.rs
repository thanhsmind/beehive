// shared JS/string helpers and the store-file readers
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
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

// ═══════════════════════════════════════════════════════════════════════════
// MUTATING VERBS
// ═══════════════════════════════════════════════════════════════════════════

/// Handler failure: hand the whole command to Node (no output yet), or a
/// thrown-Error message (bee.mjs emitError bytes).
#[derive(Debug)]
pub(crate) enum Fail {
    Delegate,
    Thrown(String),
}

pub(crate) type MR<T> = Result<T, Fail>;

impl From<Delegate> for Fail {
    fn from(_: Delegate) -> Self {
        Fail::Delegate
    }
}

impl From<rsv::Exotic> for Fail {
    fn from(_: rsv::Exotic) -> Self {
        Fail::Delegate
    }
}

pub(crate) fn to_r2(out: MR<Out>) -> R2<Out> {
    match out {
        Ok(o) => Ok(o),
        Err(Fail::Delegate) => Err(Err2::Ex),
        Err(Fail::Thrown(m)) => Ok(Out::Thrown(m)),
    }
}

/// Routing for the mutating verbs. Flags parse via bee.mjs's own grammar
/// (rsv::parse_flags); anything Node's dispatcher/validate() answers itself
/// (unknown flags, missing required flags, bad enum/number values, --help)
/// returns None before any output.
pub(crate) fn try_mutating(verb: &str, rest: &[OsString], t0: Instant) -> Option<ExitCode> {
    let toks: Vec<&str> = rest.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // Node renders command-scoped help
    }
    let (flags, use_json) = rsv::parse_flags(&toks)?;
    match verb {
        "add" => run_add(flags, use_json, t0),
        "update" => run_update(flags, use_json, t0),
        "claim" => run_claim(flags, use_json, t0),
        "cap" => run_cap(false, flags, use_json, t0),
        "finish" => run_cap(true, flags, use_json, t0),
        "block" => run_block(flags, use_json, t0),
        "drop" => run_drop(flags, use_json, t0),
        "unclaim" => run_unclaim(flags, use_json, t0),
        "reopen" => run_reopen(flags, use_json, t0),
        "tier" => run_tier(flags, use_json, t0),
        "judge" => run_judge(flags, use_json, t0),
        "reset-budget" => run_reset_budget(flags, use_json, t0),
        "judge-record" => run_judge_record(flags, use_json, t0),
        "schedule" => run_schedule(flags, use_json, t0),
        "archive" => run_archive(flags, use_json, t0),
        "unarchive" => run_unarchive(flags, use_json, t0),
        "claim-next" => run_claim_next(flags, use_json, t0),
        // D9 (store `4eaf1b71`) — the one-time role backfill. Served here
        // and DECLARED in generated/registry_payload.json in the same
        // change: mrs-12 proved that a verb the dispatcher serves and the
        // registry does not describe is reported unknown by
        // `bee --help --all` and gives the CLI-shape guard no schema to
        // validate a call against.
        "backfill-roles" => run_backfill_roles(flags, use_json, t0),
        _ => None, // every unknown verb stays with Node
    }
}

/// Boolean-typed flag through validate(): absent/Present pass; "true"/"false"
/// strings pass validate but read as JS non-`true`; any other value is
/// validate()'s refusal — delegate (None).
pub(crate) fn bool_flag(flags: &rsv::Flags, name: &str) -> Option<bool> {
    match flags.get(name) {
        None => Some(false),
        Some(FlagV::Present) => Some(true),
        Some(FlagV::S(s)) if s == "true" || s == "false" => Some(false), // string !== true
        Some(FlagV::S(_)) => None,
    }
}

/// `flags[name] !== undefined ? String(flags[name]) : undefined` for a
/// value-typed flag. Present (boolean true) is impossible for non-flag-alone
/// names by the parser's grammar.
pub(crate) fn opt_string_flag(flags: &rsv::Flags, name: &str) -> Option<Option<String>> {
    match flags.get(name) {
        None => Some(None),
        Some(FlagV::S(s)) => Some(Some(s.clone())),
        Some(FlagV::Present) => None,
    }
}

// ─── shared JS/string helpers ──────────────────────────────────────────────

pub(crate) fn js_trim(s: &str) -> &str {
    rsv::js_trim(s)
}

pub(crate) fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub(crate) fn utc_now() -> String {
    rsv::now_iso()
}

/// Array.prototype.join(', ') with JS element coercion (null/undefined -> '').
pub(crate) fn js_join(values: &[Value], sep: &str) -> String {
    values
        .iter()
        .map(|v| match v {
            Value::Null => String::new(),
            other => jsjson::js_to_string(other),
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// JSON.stringify(value) rendered into a template literal; None (undefined)
/// renders "undefined" like JS does.
pub(crate) fn js_json_or_undefined(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => jsjson::stringify(v),
    }
}

/// JSON.parse with JS number semantics. `strip_bom` mirrors readJson's BOM
/// strip (the strict readers do NOT strip). Only two outcomes now:
/// - Value: parsed and JS-number-normalized.
/// - NotJson: the text is not JSON, and every caller takes its own
///   not-valid-JSON path for it.
///
/// CUTOVER: this used to have a third `Delegate` outcome for the two shapes
/// only V8 could settle — a `\uD800`-`\uDFFF` lone-surrogate escape (V8's
/// JSON.parse accepted it, serde refuses) and a |n| >= 1e21 number (jsjson
/// printed it differently than V8). The number case is fixed upstream
/// (`js_f64_to_string` does the spec's exponential forms, `js_numberify` no
/// longer rejects), and with no Node to defer to, a lone surrogate is just
/// input this CLI cannot parse — i.e. NotJson, exactly like any other text
/// serde refuses.
pub(crate) enum JsParse {
    Value(Value),
    NotJson,
}

pub(crate) fn parse_json_js(text: &str, strip_bom: bool) -> JsParse {
    let mut t = text;
    if strip_bom {
        if let Some(stripped) = t.strip_prefix('\u{feff}') {
            t = stripped;
        }
    }
    match serde_json::from_str::<Value>(t) {
        Ok(v) => match rsv::js_numberify(&v) {
            // Unreachable from JSON text (no NaN/Infinity literals), but a
            // parse helper must not panic on it either.
            Err(_) => JsParse::NotJson,
            Ok(v) => JsParse::Value(v),
        },
        // Includes lone-surrogate escapes — see the doc comment above.
        Err(_) => JsParse::NotJson,
    }
}

// ─── store-file readers with JS-number normalization ───────────────────────

/// readJson-backed store read (BOM-stripped): Missing -> None; Corrupt ->
/// warn + None, which is exactly readJson's fail-open `fallback` for every
/// caller here (each one maps a missing file and its fallback to the same
/// branch). Parsed values are JS-number-normalized.
pub(crate) fn read_store_json(file: &Path) -> Result<Option<Value>, Delegate> {
    match read_json(file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => {
            warn_corrupt_json_once(file);
            Ok(None)
        }
        ReadJson::Parsed(v) => match rsv::js_numberify(&v) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Err(Delegate), // unreachable from JSON — see js_numberify
        },
    }
}

/// readCell with JS-number normalization for the mutators (the read-only
/// slice keeps its own un-normalized reader above).
pub(crate) fn read_cell_norm(root: &Path, id: &str) -> Result<Option<Value>, Delegate> {
    match read_cell(root, id)? {
        None => Ok(None),
        Some(v) => match rsv::js_numberify(&v) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Err(Delegate),
        },
    }
}

pub(crate) fn cell_file(root: &Path, id: &str) -> PathBuf {
    cells_dir(root).join(format!("{id}.json"))
}

/// lib/cells.mjs resolveCellFile — existence-only (never parses).
pub(crate) fn resolve_cell_file(root: &Path, id: &str) -> Option<PathBuf> {
    if id.is_empty() || !id_pattern_ok(id) {
        return None;
    }
    let active = cell_file(root, id);
    if active.exists() {
        return Some(active);
    }
    let archive_root = cells_dir(root).join(ARCHIVE_DIR_NAME);
    let entries = std::fs::read_dir(&archive_root).ok()?;
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let candidate = entry.path().join(format!("{id}.json"));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// lib/cells.mjs CellArchivedError message.
pub(crate) fn cell_archived_error(verb: &str, id: &str) -> String {
    format!(
        "{verb}: cell \"{id}\" is archived — unarchive its feature first (bee cells unarchive --feature <feature>)."
    )
}

/// lib/cells.mjs assertNotArchived: no-op for malformed ids, active cells,
/// and genuinely missing ids; throws CELL_ARCHIVED for archived-only ids.
pub(crate) fn assert_not_archived(root: &Path, verb: &str, id: &str) -> MR<()> {
    if id.is_empty() || !id_pattern_ok(id) {
        return Ok(());
    }
    if cell_file(root, id).exists() {
        return Ok(());
    }
    if resolve_cell_file(root, id).is_some() {
        return Err(Fail::Thrown(cell_archived_error(verb, id)));
    }
    Ok(())
}

/// lock-holder rendering shared by CellsArchiveBusyError and
/// DecisionsLockBusyError: `pid=<p> session=<s> since <ts>` with ?? 'unknown'.
pub(crate) fn holder_who(holder: &Option<Value>) -> String {
    match holder {
        Some(Value::Object(h)) => {
            let field = |k: &str| match h.get(k) {
                Some(Value::Null) | None => "unknown".to_string(),
                Some(v) => jsjson::js_to_string(v),
            };
            format!("pid={} session={} since {}", field("pid"), field("session"), field("ts"))
        }
        _ => "unknown holder".to_string(),
    }
}

/// lib/cells.mjs writeCell — the single write funnel: valid-id check, brief
/// 'cells-archive' single-attempt acquire (typed CELLS_ARCHIVE_BUSY on
/// contention), archived-only re-check under that lock, atomic write.
pub(crate) fn write_cell(root: &Path, cell: &Value) -> MR<()> {
    let id = match cell.get("id") {
        Some(Value::String(s)) if !s.is_empty() && id_pattern_ok(s) => s.clone(),
        other => {
            return Err(Fail::Thrown(format!(
                "writeCell: cell needs a valid id (got {}).",
                js_json_or_undefined(other)
            )))
        }
    };
    match lock::acquire_store_lock_once(root, "cells-archive") {
        lock::AcquireOnce::Busy { holder } => Err(Fail::Thrown(format!(
            "writeCell: cell \"{id}\" write refused — the \"cells-archive\" lock is held by {} (a live archive/unarchive transaction). Retry once it completes.",
            holder_who(&holder)
        ))),
        lock::AcquireOnce::Acquired(mut guard) => {
            let result = (|| -> MR<()> {
                let active = cell_file(root, &id);
                if !active.exists() && resolve_cell_file(root, &id).is_some() {
                    return Err(Fail::Thrown(cell_archived_error("writeCell", &id)));
                }
                // Node throws the raw fs error here (libuv bytes) — residual
                // divergence on a hard write failure, documented in the header.
                write_json_atomic(&active, cell)
                    .map_err(|e| Fail::Thrown(format!("writeCell: {e}")))
            })();
            guard.release();
            result
        }
    }
}

/// withStoreLock(root, name, ..) — Node's async bounded-retry acquisition;
/// a LockBusyError's message surfaces via emitError.
pub(crate) fn acquire_named_lock(root: &Path, name: &str) -> MR<lock::LockGuard> {
    lock::acquire_store_lock(root, name, lock::MAX_ATTEMPTS).map_err(|busy| Fail::Thrown(busy.message()))
}
