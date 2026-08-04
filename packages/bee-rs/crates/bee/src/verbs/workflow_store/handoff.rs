// the handoff mailbox, gate stamps and listing order
//
// Split out of the single 2.7k-line verbs/workflow_store.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock::{self, LockGuard, MAX_ATTEMPTS};
use crate::verbs::reservations::{
    date_parse_val, jget, js_disp, js_disp_opt, js_trim, now_iso, pseudo_uuid_v4,
    truthy, Err2, Ex, Exotic,
};
use crate::verbs::state_group::{
    adopt_claim, coerce_legacy_phase, default_gates, handoff_path, io_read_reason, parse_json_v8,
    read_claim, read_state_peek, spread_gates, write_state, AdoptOutcome, ParsedJson,
};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

// ─── handoff mailbox (lib/state.mjs) ───────────────────────────────────────

/// state.mjs normalizeHandoffKind.
pub(crate) fn normalize_handoff_kind(kind: Option<&Value>) -> &'static str {
    match kind {
        Some(Value::String(s)) if s == "planned-next" => "planned-next",
        _ => "pause",
    }
}

/// state.mjs normalizeTargetRole.
pub(crate) fn normalize_target_role(role: Option<&str>) -> Option<String> {
    match role {
        Some(r) if !js_trim(r).is_empty() => Some(js_trim(r).to_string()),
        _ => None,
    }
}

pub(crate) fn normalize_target_role_val(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
        _ => None,
    }
}

/// state.mjs requireHandoffWorkflowId.
pub(crate) fn require_handoff_workflow_id(value: &str) -> Result<String, Err2> {
    let id = js_trim(value);
    if id.is_empty() {
        return Err(Err2::Msg("handoff mailbox: workflow id is required.".to_string()));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Err2::Msg(format!(
            "handoff mailbox: workflow id \"{id}\" must be a plain id (no path separators)."
        )));
    }
    Ok(id.to_string())
}

pub(crate) fn handoff_mailbox_dir(root: &Path, workflow_id: &str) -> Result<PathBuf, Err2> {
    Ok(root
        .join(".bee")
        .join("runtime")
        .join("handoffs")
        .join(require_handoff_workflow_id(workflow_id)?))
}

pub(crate) fn handoff_record_path(root: &Path, workflow_id: &str, seq: i64) -> Result<PathBuf, Err2> {
    Ok(handoff_mailbox_dir(root, workflow_id)?
        .join(format!("{:0width$}.json", seq, width = HANDOFF_SEQ_WIDTH)))
}

/// state.mjs listHandoffMailbox — oldest→newest by seq; `seq` attached
/// in-memory from the filename, `kind` normalized on read.
pub(crate) fn list_handoff_mailbox(root: &Path, workflow_id: &str) -> Ex<Vec<Map<String, Value>>> {
    let Ok(dir) = handoff_mailbox_dir(root, workflow_id) else {
        return Ok(Vec::new()); // the throw is not caught in Node — but every
        // caller here passes an id straight off a workflow record's directory
        // name, which already satisfies requireHandoffWorkflowId.
    };
    let Ok(rd) = std::fs::read_dir(&dir) else { return Ok(Vec::new()) };
    let mut records: Vec<Map<String, Value>> = Vec::new();
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".json") else { continue };
        let Some(seq) = js_parse_int(stem) else { continue };
        let file = dir.join(format!("{:0width$}.json", seq, width = HANDOFF_SEQ_WIDTH));
        let raw = match read_json(&file) {
            ReadJson::Missing => continue,
            // readJson's `null` fallback fails Node's `!raw` guard: the record
            // is skipped, the rest of the mailbox is listed as before.
            ReadJson::Corrupt => {
                crate::fsutil::warn_corrupt_json(&file);
                continue;
            }
            ReadJson::Parsed(v) => crate::verbs::reservations::js_numberify(&v)?,
        };
        let Value::Object(mut m) = raw else { continue };
        let kind = normalize_handoff_kind(m.get("kind"));
        m.insert("kind".into(), json!(kind));
        m.insert("seq".into(), json!(seq));
        records.push(m);
    }
    records.sort_by_key(|r| r.get("seq").and_then(Value::as_i64).unwrap_or(0));
    Ok(records)
}

/// Number.parseInt(s, 10) restricted to the plain-decimal shapes a mailbox
/// filename can carry; NaN (→ skip) for everything else.
pub(crate) fn js_parse_int(s: &str) -> Option<i64> {
    let t = js_trim(s);
    let (sign, rest) = match t.strip_prefix('-') {
        Some(r) => (-1i64, r),
        None => (1i64, t.strip_prefix('+').unwrap_or(t)),
    };
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<i64>().ok().map(|n| sign * n)
}

pub(crate) fn record_seq(record: &Map<String, Value>) -> i64 {
    record.get("seq").and_then(Value::as_i64).unwrap_or(0)
}

/// state.mjs newestOpenHandoffMailboxRecord.
pub(crate) fn newest_open_handoff_mailbox_record(
    root: &Path,
    workflow_id: &str,
    target_role: Option<&str>,
) -> Ex<Option<Map<String, Value>>> {
    let role = normalize_target_role(target_role);
    let records = list_handoff_mailbox(root, workflow_id)?;
    for record in records.iter().rev() {
        if !matches!(record.get("status"), Some(Value::String(s)) if s == "open") {
            continue;
        }
        if normalize_target_role_val(record.get("target_role")) != role {
            continue;
        }
        return Ok(Some(record.clone()));
    }
    Ok(None)
}

/// A previous-cell id Node's path.join treats as a plain filename.
pub(crate) fn cell_path_modelable(id: &str) -> bool {
    !(id.contains(':') || id.starts_with('/') || id.starts_with('\\') || id.contains('\0'))
}

/// state.mjs writeMailboxHandoff. `input` is the CLI-built record (kind first,
/// then the optional fields, in bee.mjs's own insertion order); `target_role`
/// is passed separately because bee.mjs spreads it on last. Every precondition
/// is READ before the single write; the whole write runs under
/// `handoff:<workflow-id>`.
pub(crate) fn write_mailbox_handoff(
    root: &Path,
    workflow_id: &str,
    input: &Map<String, Value>,
    target_role: Option<&str>,
) -> Result<Map<String, Value>, Err2> {
    let wf_id_s = require_handoff_workflow_id(workflow_id)?;
    let kind = match input.get("kind") {
        Some(Value::String(s)) if s == "planned-next" || s == "pause" => s.clone(),
        other => {
            return Err(Err2::Msg(format!(
                "writeMailboxHandoff: --kind must be \"planned-next\" or \"pause\" (got {}) — D1 requires an explicit kind, never guessed. FIX: pass one of the two handoff kinds.",
                match other {
                    Some(v) => jsjson::stringify(v),
                    None => "undefined".to_string(),
                }
            )));
        }
    };
    let role = normalize_target_role(target_role);
    let now = now_iso();

    // `fields` — the per-kind body, in Node's exact key order.
    let mut fields: Map<String, Value> = Map::new();
    if kind == "pause" {
        for (k, v) in input {
            if k != "kind" && k != "target_role" && k != "from_session" {
                fields.insert(k.clone(), v.clone());
            }
        }
        fields.insert("kind".into(), json!("pause"));
        // `from_session` is never supplied by the CLI, so this is always null.
        let from = match input.get("from_session") {
            Some(Value::String(s)) if !js_trim(s).is_empty() => json!(js_trim(s)),
            _ => Value::Null,
        };
        fields.insert("from_session".into(), from);
    } else {
        let get_trim = |key: &str| -> String {
            match input.get(key) {
                Some(Value::String(s)) => js_trim(s).to_string(),
                _ => String::new(),
            }
        };
        let writer_session = get_trim("writer_session");
        let previous_cell = get_trim("previous_cell");
        let next_cell = get_trim("next_cell");
        if writer_session.is_empty() || previous_cell.is_empty() || next_cell.is_empty() {
            return Err(Err2::Msg(
                "writeMailboxHandoff: a planned-next handoff requires non-empty writer_session, previous_cell, and next_cell (D1) — FIX: pass all three.".to_string(),
            ));
        }
        if !cell_path_modelable(&previous_cell) {
            return Err(Err2::Ex);
        }
        let prev_file = root.join(".bee").join("cells").join(format!("{previous_cell}.json"));
        let previous = match read_json(&prev_file) {
            ReadJson::Missing => None,
            // readJson's `null` fallback: the cell reads as absent, so the
            // not-capped refusal below reports status "missing" unchanged.
            ReadJson::Corrupt => {
                crate::fsutil::warn_corrupt_json(&prev_file);
                None
            }
            ReadJson::Parsed(v) => Some(crate::verbs::reservations::js_numberify(&v)?),
        };
        let capped = previous
            .as_ref()
            .map(|v| truthy(v) && matches!(jget(v, "status"), Some(Value::String(s)) if s == "capped"))
            .unwrap_or(false);
        if !capped {
            let status_disp = match previous.as_ref().and_then(|v| jget(v, "status")) {
                None | Some(Value::Null) => "missing".to_string(),
                Some(v) => js_disp(v),
            };
            return Err(Err2::Msg(format!(
                "writeMailboxHandoff: refused — previous cell \"{previous_cell}\" is not capped (found status \"{status_disp}\"). A planned-next handoff may only follow a capped cell. FIX: finish \"{previous_cell}\" first (bee cells finish), then retry."
            )));
        }
        let claim = read_claim(root, &next_cell)?;
        let owned = claim
            .as_ref()
            .map(|c| {
                matches!(jget(c, "session"), Some(Value::String(s)) if *s == writer_session)
            })
            .unwrap_or(false);
        if !owned {
            let found = match &claim {
                None => "no claim".to_string(),
                Some(c) => format!("owner \"{}\"", js_disp_opt(jget(c, "session"))),
            };
            return Err(Err2::Msg(format!(
                "writeMailboxHandoff: refused — next cell \"{next_cell}\" has no claim owned by writer session \"{writer_session}\" (found {found}). The next cell must already be claimed by the writing session before a planned-next handoff carries it. FIX: claim \"{next_cell}\" as session \"{writer_session}\" first (bee cells claim), then retry."
            )));
        }
        let claim_epoch = match claim.as_ref().and_then(|c| jget(c, "fence_epoch")) {
            Some(Value::Number(n)) => Value::Number(n.clone()),
            _ => json!(1),
        };
        for (k, v) in input {
            if !matches!(
                k.as_str(),
                "kind" | "target_role" | "writer_session" | "from_session" | "previous_cell" | "next_cell"
            ) {
                fields.insert(k.clone(), v.clone());
            }
        }
        fields.insert("kind".into(), json!("planned-next"));
        fields.insert("writer_session".into(), json!(writer_session));
        fields.insert("from_session".into(), json!(writer_session));
        fields.insert("previous_cell".into(), json!(previous_cell));
        fields.insert("next_cell".into(), json!(next_cell));
        fields.insert("claim_epoch".into(), claim_epoch);
    }

    let guard = acquire_named_lock(root, &format!("handoff:{wf_id_s}"))?;
    let out = (|| -> Result<Map<String, Value>, Err2> {
        let existing = list_handoff_mailbox(root, &wf_id_s)?;
        let seq = existing.last().map(|r| record_seq(r) + 1).unwrap_or(1);
        // Auto-clear the previous OPEN record for this SAME (workflow, role).
        if let Some(prior) = newest_open_handoff_mailbox_record(root, &wf_id_s, target_role)? {
            let prior_seq = record_seq(&prior);
            let mut cleared = prior;
            cleared.shift_remove("seq");
            cleared.insert("status".into(), json!("cleared"));
            cleared.insert("cleared_at".into(), json!(now));
            write_json_atomic(
                &handoff_record_path(root, &wf_id_s, prior_seq)?,
                &Value::Object(cleared),
            )
            .map_err(|_| Err2::Ex)?;
        }
        let mut record: Map<String, Value> = Map::new();
        record.insert(
            "id".into(),
            json!(format!("{wf_id_s}-{:0width$}", seq, width = HANDOFF_SEQ_WIDTH)),
        );
        record.insert("workflow_id".into(), json!(wf_id_s));
        record.insert(
            "target_role".into(),
            role.as_ref().map(|r| json!(r)).unwrap_or(Value::Null),
        );
        record.insert("status".into(), json!("open"));
        record.insert("written_at".into(), json!(now));
        for (k, v) in &fields {
            record.insert(k.clone(), v.clone());
        }
        write_json_atomic(&handoff_record_path(root, &wf_id_s, seq)?, &Value::Object(record.clone()))
            .map_err(|_| Err2::Ex)?;
        let mut returned = record;
        returned.insert("seq".into(), json!(seq));
        Ok(returned)
    })();
    drop(guard);
    out
}

/// `record[key] = value` where `None` is JS `undefined` — the key is dropped
/// by JSON.stringify rather than written as null.
pub(crate) fn set_or_drop(record: &mut Map<String, Value>, key: &str, value: &Option<Value>) {
    match value {
        Some(v) => {
            record.insert(key.to_string(), v.clone());
        }
        None => {
            record.shift_remove(key);
        }
    }
}

pub(crate) enum MailboxAdopt {
    Fail { reason: String },
    Ok {
        claim: Option<Value>,
        previous_owner: Option<Value>,
        next_cell: String,
        workflow_id: String,
        seq: i64,
    },
}

/// state.mjs adoptMailboxHandoff — newest open-OR-adopted record for
/// (workflow, role); adoptClaim then mark 'cleared', with the crash-between
/// self-heal. Runs under `handoff:<workflow-id>`.
pub(crate) fn adopt_mailbox_handoff(
    root: &Path,
    workflow_id: &str,
    session_id: &str,
    target_role: Option<&str>,
) -> Result<MailboxAdopt, Err2> {
    let wf_id_s = require_handoff_workflow_id(workflow_id)?;
    let role = normalize_target_role(target_role);
    let guard = acquire_named_lock(root, &format!("handoff:{wf_id_s}"))?;
    let out = (|| -> Result<MailboxAdopt, Err2> {
        let records = list_handoff_mailbox(root, &wf_id_s)?;
        let mut candidate: Option<Map<String, Value>> = None;
        for record in records.iter().rev() {
            if normalize_target_role_val(record.get("target_role")) != role {
                continue;
            }
            match record.get("status") {
                Some(Value::String(s)) if s == "open" || s == "adopted" => {
                    candidate = Some(record.clone());
                    break;
                }
                _ => {}
            }
        }
        let Some(candidate) = candidate else {
            let role_note = role.as_ref().map(|r| format!(" (role \"{r}\")")).unwrap_or_default();
            return Ok(MailboxAdopt::Fail {
                reason: format!(
                    "no open handoff in workflow \"{wf_id_s}\"'s mailbox{role_note} to adopt."
                ),
            });
        };
        if !matches!(candidate.get("kind"), Some(Value::String(s)) if s == "planned-next") {
            return Ok(MailboxAdopt::Fail {
                reason: format!(
                    "handoff kind \"{}\" is not \"planned-next\" — a pause handoff is never adopted, it must be surfaced and WAITED on (D1).",
                    js_disp_opt(candidate.get("kind"))
                ),
            });
        }
        let next_cell = match candidate.get("next_cell") {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        };
        if next_cell.is_empty() {
            return Ok(MailboxAdopt::Fail {
                reason: "planned-next handoff has no next_cell to adopt.".to_string(),
            });
        }
        let seq = record_seq(&candidate);
        let mut rest = candidate.clone();
        rest.shift_remove("seq");
        let was_open = matches!(candidate.get("status"), Some(Value::String(s)) if s == "open");
        // `candidate.adopted_previous_owner ?? null` — a definite null (the key
        // IS emitted). The open branch below overwrites it with adoptClaim's
        // own `previous_owner`, which may be undefined (key DROPPED), hence
        // Option<Value> rather than a bare Value.
        let mut previous_owner: Option<Value> =
            Some(candidate.get("adopted_previous_owner").cloned().unwrap_or(Value::Null));
        let claim: Option<Value>;
        if was_open {
            match adopt_claim(root, &next_cell, session_id)? {
                AdoptOutcome::Fail { reason } => return Ok(MailboxAdopt::Fail { reason }),
                AdoptOutcome::Adopted { claim: c, previous_owner: prev } => {
                    previous_owner = prev;
                    let epoch = c.get("fence_epoch").cloned().unwrap_or(Value::Null);
                    claim = Some(Value::Object(c));
                    let mut adopted = rest.clone();
                    adopted.insert("status".into(), json!("adopted"));
                    adopted.insert("claim_epoch".into(), epoch);
                    adopted.insert("adopted_by".into(), json!(session_id));
                    adopted.insert("adopted_at".into(), json!(now_iso()));
                    set_or_drop(&mut adopted, "adopted_previous_owner", &previous_owner);
                    write_json_atomic(
                        &handoff_record_path(root, &wf_id_s, seq)?,
                        &Value::Object(adopted),
                    )
                    .map_err(|_| Err2::Ex)?;
                }
            }
        } else {
            // Self-heal: the claim already moved on a crashed-before-clear call.
            claim = read_claim(root, &next_cell)?;
        }
        let claim_epoch = match claim.as_ref().and_then(|c| jget(c, "fence_epoch")) {
            Some(Value::Number(n)) => Value::Number(n.clone()),
            _ => candidate.get("claim_epoch").cloned().unwrap_or(Value::Null),
        };
        let adopted_at = match candidate.get("adopted_at") {
            None | Some(Value::Null) => json!(now_iso()),
            Some(v) => v.clone(),
        };
        let mut cleared = rest;
        cleared.insert("status".into(), json!("cleared"));
        cleared.insert("claim_epoch".into(), claim_epoch);
        cleared.insert("adopted_by".into(), json!(session_id));
        cleared.insert("adopted_at".into(), adopted_at);
        set_or_drop(&mut cleared, "adopted_previous_owner", &previous_owner);
        cleared.insert("cleared_at".into(), json!(now_iso()));
        write_json_atomic(&handoff_record_path(root, &wf_id_s, seq)?, &Value::Object(cleared))
            .map_err(|_| Err2::Ex)?;
        Ok(MailboxAdopt::Ok {
            claim,
            previous_owner,
            next_cell,
            workflow_id: wf_id_s.clone(),
            seq,
        })
    })();
    drop(guard);
    out
}

// ─── gate stamps + workflow listing order (bee.mjs) ────────────────────────

/// bee.mjs findGateStamp — normalizes the single/array/absent shapes to the
/// one entry (if any) naming `name`.
pub(crate) fn find_gate_stamp<'a>(
    stamps: &'a [(String, Value)],
    name: &str,
) -> Option<&'a (String, Value)> {
    stamps.iter().find(|(n, _)| n == name)
}

/// The `gates` half of writeLaneRecordThroughProjection /
/// writeStateRecordThroughProjection's updateWorkflowAssumingLock patch.
pub(crate) fn gates_patch_from_record(
    updated: &Map<String, Value>,
    stamps: &[(String, Value)],
) -> Value {
    let mut gates = Map::new();
    for name in GATE_NAMES {
        let mut entry = Map::new();
        let approved = updated
            .get("approved_gates")
            .filter(|v| truthy(v))
            .and_then(|v| jget(v, name))
            .map(|v| v == &Value::Bool(true))
            .unwrap_or(false);
        entry.insert("approved".into(), Value::Bool(approved));
        if let Some((_, rev)) = find_gate_stamp(stamps, name) {
            entry.insert("approved_for_plan_rev".into(), rev.clone());
        }
        gates.insert(name.to_string(), Value::Object(entry));
    }
    Value::Object(gates)
}

/// bee.mjs workflowsListSort — created_at descending (newest first), stable.
pub(crate) fn workflows_list_sort(records: &mut [Map<String, Value>]) -> Ex<()> {
    let mut keyed: Vec<f64> = Vec::with_capacity(records.len());
    for r in records.iter() {
        let stamp = match r.get("created_at") {
            Some(v) if truthy(v) => date_parse_val(Some(v))?.unwrap_or(f64::NAN),
            _ => 0.0,
        };
        keyed.push(stamp);
    }
    let mut idx: Vec<usize> = (0..records.len()).collect();
    idx.sort_by(|&a, &b| {
        let d = keyed[b] - keyed[a];
        if d.is_nan() || d == 0.0 {
            // A comparator returning NaN is treated as 0 by V8's sort.
            std::cmp::Ordering::Equal
        } else if d < 0.0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    let sorted: Vec<Map<String, Value>> = idx.into_iter().map(|i| records[i].clone()).collect();
    records.clone_from_slice(&sorted);
    Ok(())
}
