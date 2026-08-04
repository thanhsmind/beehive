// JS-semantics helpers and the flag plumbing every verb parses through
//
// Split out of the single 6.1k-line verbs/state_group.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{
    append_jsonl, ensure_dir, read_json, warn_corrupt_json, write_json_atomic, ReadJson,
};
use crate::jsjson;
use crate::lock::{self, AcquireOnce, LockGuard};
use crate::verbs::reservations::{
    date_parse_val, finish, iso_from_ms, jget, js_disp, js_disp_opt,
    js_numberify, js_trim, keys_known, now_iso, now_ms, parse_flags, prelude, truthy,
    Ctx, Err2, Ex, Exotic, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::reservations::{list_reservations, paths_overlap, rebuild_reservations_projection};
use crate::verbs::workspace_store as ws;
use crate::verbs::workflow_store::{
    acquire_named_lock, acquire_workflow_lock, adopt_mailbox_handoff, create_workflow,
    find_live_workflow, NewWorkflow,
    gates_patch_from_record, lane_lock_name, lane_path, list_lanes, list_workflows,
    newest_open_handoff_mailbox_record, projection_lock_name, read_lane_display, read_lane_strict,
    rebuild_handoff_projection, rebuild_handoff_projection_reporting, rebuild_lane_projection,
    rebuild_lane_projection_reporting, rebuild_state_projection,
    rebuild_state_projection_reporting, update_workflow, update_workflow_assuming_lock,
    update_workflow_assuming_lock_with, wf_id, workflows_list_sort, write_lane,
    write_mailbox_handoff, MailboxAdopt,
};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

/// `${from || 'idle'}` — the phase-door display value.
pub(crate) fn disp_or_idle(from: Option<&Value>) -> String {
    match from {
        Some(v) if truthy(v) => js_disp(v),
        _ => "idle".to_string(),
    }
}

/// JS Array.prototype.sort default comparator (UTF-16 code units).
pub(crate) fn js_sort(v: &mut [String]) {
    v.sort_by(|a, b| {
        a.encode_utf16()
            .collect::<Vec<_>>()
            .cmp(&b.encode_utf16().collect::<Vec<_>>())
    });
}

/// splitList (bee.mjs): split on ',', JS-trim, drop empties.
pub(crate) fn split_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(js_trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// `.bee<sep>rest...` display path (path.relative(root, file) for files under
/// root/.bee/) — Node uses the platform separator.
pub(crate) fn rel_bee(parts: &[&str]) -> String {
    let mut out = String::from(".bee");
    for p in parts {
        out.push(MAIN_SEPARATOR);
        out.push_str(p);
    }
    out
}

pub(crate) enum ParsedJson {
    Parsed(Value),
    Unparseable,
}

/// JSON.parse modeled: serde parse + js_numberify.
///
/// CUTOVER. This used to answer `Err(Exotic)` — "ask Node" — whenever serde
/// refused text containing a "\u" escape, because V8's JSON.parse accepts a
/// LONE SURROGATE escape (\uD800-\uDFFF unpaired) where serde_json does not.
/// There is no Node to ask any more, and nothing in this process can decode
/// such a string into a Rust `String`, so that input is now simply
/// UNPARSEABLE: every caller takes the same branch it takes for any other
/// corrupt file. The practical population is a hand-corrupted or truncated
/// record, which is corrupt either way.
pub(crate) fn parse_json_v8(text: &str) -> Ex<ParsedJson> {
    match serde_json::from_str::<Value>(text) {
        Ok(v) => Ok(ParsedJson::Parsed(js_numberify(&v)?)),
        Err(_) => Ok(ParsedJson::Unparseable),
    }
}

/// A portable, engine-free category for a failed `std::fs::read`, for the
/// refusals that used to interpolate a libuv errno code (`EISDIR`, `EPERM`,
/// …). The OS message string is deliberately NOT used: it varies by platform
/// and locale, so a refusal built from it would not be reproducible.
pub(crate) fn io_read_reason(file: &Path, err: &std::io::Error) -> &'static str {
    if file.is_dir() {
        return "the path is a directory";
    }
    match err.kind() {
        std::io::ErrorKind::PermissionDenied => "permission denied",
        std::io::ErrorKind::NotFound => "the file is missing",
        _ => "the file could not be read",
    }
}

// ─── flag plumbing (bee.mjs requireFlag / requireFlags / lane selectors) ───

/// The raw string value of a flag, or None for absent/boolean-true — the
/// shape `typeof flag === 'string'` accepts (resolveSessionId's own guard).
pub(crate) fn flag_value(flags: &Flags, name: &str) -> Option<String> {
    match flags.get(name) {
        Some(FlagV::S(s)) => Some(s.clone()),
        _ => None,
    }
}

/// `${sessionId}` where resolveSessionId answers `string | null`.
pub(crate) fn sid_disp(sid: &Option<String>) -> String {
    match sid {
        Some(s) => s.clone(),
        None => "null".to_string(),
    }
}

/// `String(flags[name])` when the flag is present (Present → "true").
pub(crate) fn flag_string(flags: &Flags, name: &str) -> Option<String> {
    match flags.get(name) {
        None => None,
        Some(FlagV::Present) => Some("true".to_string()),
        Some(FlagV::S(s)) => Some(s.clone()),
    }
}

/// requireFlag: undefined | '' | true → the thrown "Missing required flag".
pub(crate) fn require_flag(flags: &Flags, name: &str) -> Result<String, Err2> {
    match flags.get(name) {
        Some(FlagV::S(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(Err2::Msg(format!("Missing required flag --{name}."))),
    }
}

/// requireFlags (ce-1 batch refusal): every missing/invalid flag in one Error.
pub(crate) fn require_flags(
    flags: &Flags,
    spec: &[(&str, Option<&[&str]>)],
    example: &str,
) -> Result<Vec<String>, Err2> {
    let mut missing: Vec<String> = Vec::new();
    let mut invalid: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();
    for (name, allowed) in spec {
        match flags.get(name) {
            Some(FlagV::S(s)) if !s.is_empty() => {
                if let Some(a) = allowed {
                    if !a.contains(&s.as_str()) {
                        invalid.push(format!("--{name} \"{s}\" (must be one of {})", a.join(", ")));
                        values.push(String::new());
                        continue;
                    }
                }
                values.push(s.clone());
            }
            _ => {
                missing.push(format!("--{name}"));
                values.push(String::new());
            }
        }
    }
    if missing.is_empty() && invalid.is_empty() {
        return Ok(values);
    }
    let mut parts: Vec<String> = Vec::new();
    if !missing.is_empty() {
        parts.push(format!("missing required flag(s): {}", missing.join(", ")));
    }
    if !invalid.is_empty() {
        parts.push(format!("invalid flag(s): {}", invalid.join("; ")));
    }
    Err(Err2::Msg(format!("{}. Example: {example}", parts.join("; "))))
}

/// optionalLaneFlag: `--lane` bare/empty is a malformed call, never "no lane".
pub(crate) fn optional_lane_flag(flags: &Flags, verb: &str) -> Result<Option<String>, Err2> {
    match flags.get("lane") {
        None => Ok(None),
        Some(FlagV::Present) => Err(Err2::Msg(format!(
            "{verb}: --lane requires a value (the lane's feature name)."
        ))),
        Some(FlagV::S(s)) if s.is_empty() => Err(Err2::Msg(format!(
            "{verb}: --lane requires a value (the lane's feature name)."
        ))),
        Some(FlagV::S(s)) => Ok(Some(s.clone())),
    }
}

/// mutationLaneSelector (i54-closeout-7 D7).
pub(crate) fn mutation_lane_selector(flags: &Flags, verb: &str) -> Result<(Option<String>, bool), Err2> {
    let lane_feature = optional_lane_flag(flags, verb)?;
    let no_lane = flags.get("no-lane").is_some();
    if no_lane && lane_feature.is_some() {
        return Err(Err2::Msg(format!(
            "{verb}: --no-lane cannot be combined with --lane — --no-lane forces the default record, --lane names a lane record. Pick one."
        )));
    }
    Ok((lane_feature, no_lane))
}

/// validate() type check for a boolean-schema flag: bare, "true", or "false"
/// pass; any other =value is validate()'s own STDOUT refusal → delegate.
pub(crate) fn bool_flag_ok(flags: &Flags, name: &str) -> bool {
    match flags.get(name) {
        None | Some(FlagV::Present) => true,
        Some(FlagV::S(s)) => s == "true" || s == "false",
    }
}
