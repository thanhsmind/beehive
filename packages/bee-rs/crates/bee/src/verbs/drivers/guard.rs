// the dispatch-guard vocabulary, the cells read, and collation
//
// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::reservations::{
    finish, js_is_ws, parse_flags, prelude, pseudo_uuid_v4, truthy, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{
    release_reservations_for_agent, reserve_path_atomic, Err2, ReserveOutcome,
};
use serde_json::{Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

// ═══ dispatch-guard.mjs (the enforcement vocabulary) ═══════════════════════

pub(crate) const NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE: &str = "native_model_override";

pub(crate) const NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY: &str = "native_budget_only";

/// Every rendered bee agent, keyed by the ROLE it serves.
///
/// provenance: dispatch-guard.mjs PINNED_AGENT_TYPE (W3 pinned-type rule).
///
/// model-role-split D2/D3 (store 06e49368, 3c9d6262): the key used to be a
/// COST tier, and this table had a TWIN in `hooks/model_guard.rs` — the same
/// four pairs written out a second time, in exactly the drift shape the two
/// tier lists were already caught in (4 entries against 5, with nothing
/// intending it). This is the ONE table now; the guard hook asks it rather
/// than restating it, the same collapse D1 made for the config parser.
///
/// ORDER IS LOAD-BEARING. `generation` appears twice because two rendered
/// agents serve it — bee-gather reads, bee-build writes — and a role-only
/// lookup answers with the FIRST entry, the read-only one. That is the safe
/// answer when nothing else in the dispatch says which of the two is meant;
/// the one signal that CAN say so is `--kind cell`, and `prepare.rs` reads it
/// before it ever reaches this table.
pub(crate) const ROLE_AGENTS: [(&str, &str); 4] = [
    ("generation", "bee-gather"),
    ("generation", "bee-build"),
    ("extraction", "bee-extract"),
    ("review", "bee-review"),
];

/// The rendered bee agent a role is served by — `None` when the role has none
/// of its own.
///
/// `None` is a LEGAL answer, never a missing one. Under D2's open role set
/// most roles a host can configure (`test`, `docs`, `design`, and `advisor`
/// as shipped) have no rendered agent file at all, so answering one — or
/// falling back to a generic type — would name an agent that does not exist.
pub(crate) fn agent_for_role(role: &str) -> Option<&'static str> {
    ROLE_AGENTS.iter().find(|(r, _)| *r == role).map(|(_, agent)| *agent)
}

/// The role a rendered bee agent stands for. These files are generated FROM
/// the role's configured model at onboarding, so naming one IS a role
/// declaration in every sense that matters.
pub(crate) fn role_for_agent(agent: &str) -> Option<&'static str> {
    ROLE_AGENTS.iter().find(|(_, a)| *a == agent).map(|(role, _)| *role)
}

/// The `subagent_type` a PREPARED claude Agent payload carries for `role`.
///
/// The one caller that needs a TOTAL answer: an Agent payload must name some
/// type, so a role with no rendered agent of its own gets the runtime's own
/// generic. That is the deliberate answer for `advisor` — bee renders no
/// advisor agent, and an advisor's model comes from the advisor slot rather
/// than from an agent file. Every caller that CAN honour "this role has no
/// agent" — the model-guard's pinned-type repair above all — asks
/// `agent_for_role` instead, and skips the repair rather than rewriting a
/// dispatch onto a type that does not exist.
pub(crate) fn pinned_agent_type(role: &str) -> &'static str {
    agent_for_role(role).unwrap_or("general-purpose") // `PINNED_AGENT_TYPE[role] || 'general-purpose'`
}

/// provenance: dispatch-guard.mjs deriveEconomics — the ONE honest
/// pinned/unverified/inherited-or-unknown/native-requested split. Key order is
/// frozen: {logical_tier, requested_model, effective_model,
/// effective_model_status, channel, enforcement}.
pub(crate) fn derive_economics(
    channel: &str,
    tier: &str,
    param_model: Option<&str>,
    resolved: &Resolved,
    native_confirmed: bool,
) -> Map<String, Value> {
    let is_native_confirmed =
        channel == "codex-native" && matches!(resolved, Resolved::Native { .. }) && native_confirmed;
    let resolved_model: Option<String> = match resolved {
        Resolved::Model { model, .. } | Resolved::Native { model, .. } => Some(model.clone()),
        _ => None,
    };

    let enforcement = if channel == "cli-exec" {
        "cli-command"
    } else if channel == "herding-exec" {
        "herding-command"
    } else if channel == "session-model" {
        "session-model"
    } else if is_native_confirmed {
        "native-model-param"
    } else if channel == "codex-native" {
        "prompt-budget"
    } else if param_model.is_some() {
        "model-param"
    } else {
        "prompt-budget"
    };

    let mut effective_model = Value::Null;
    let effective_model_status = if channel == "session-model" {
        "inherited-or-unknown"
    } else if is_native_confirmed {
        "native-requested"
    } else if channel == "codex-native" {
        "inherited-or-unknown"
    } else if channel == "cli-exec" || channel == "herding-exec" {
        "unverified"
    } else if let Some(pm) = param_model {
        effective_model = Value::String(pm.to_string());
        "pinned"
    } else {
        "unverified"
    };

    let requested_model = if channel == "cli-exec" || channel == "herding-exec" || channel == "session-model" {
        Value::Null
    } else {
        match param_model.map(str::to_string).or(resolved_model) {
            Some(m) => Value::String(m),
            None => Value::Null,
        }
    };

    let mut out = Map::new();
    out.insert("logical_tier".into(), Value::String(tier.to_string()));
    out.insert("requested_model".into(), requested_model);
    out.insert("effective_model".into(), effective_model);
    out.insert(
        "effective_model_status".into(),
        Value::String(effective_model_status.to_string()),
    );
    out.insert("channel".into(), Value::String(channel.to_string()));
    out.insert("enforcement".into(), Value::String(enforcement.to_string()));
    out
}

// ═══ cells (lib/cells.mjs; Rust port: verbs/cells.rs) ══════════════════════

pub(crate) fn cells_dir(root: &Path) -> PathBuf {
    root.join(".bee").join("cells")
}

/// provenance: cells.mjs ARCHIVE_DIR_NAME (verbs/cells.rs:330).
pub(crate) const ARCHIVE_DIR_NAME: &str = "archive";

/// provenance: cells.mjs ID_PATTERN /^[A-Za-z0-9][A-Za-z0-9._-]*$/
/// (verbs/cells.rs:333 id_pattern_ok).
pub(crate) fn id_pattern_ok(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// provenance: fsutil.mjs readJson(file, null) (verbs/cells.rs:347
/// read_cell_json).
///
/// CUTOVER: corrupt used to be Node's V8-warning path and delegated. It now
/// warns once and returns readJson's own `null` fallback, so every caller
/// sees the record exactly as it saw an absent one — which is what Node's
/// `!cell` / `?? null` guards did with that same fallback.
pub(crate) fn rj(file: &Path) -> D<Option<Value>> {
    match read_json(file) {
        ReadJson::Missing => Ok(None),
        ReadJson::Corrupt => {
            crate::fsutil::warn_corrupt_json(file);
            Ok(None)
        }
        ReadJson::Parsed(Value::Null) => Ok(None),
        ReadJson::Parsed(v) => Ok(Some(v)),
    }
}

/// provenance: cells.mjs readCell (verbs/cells.rs:419 read_cell) — the active
/// file wins, then every `.bee/cells/archive/<feature>/` dir in readdir order.
pub(crate) fn read_cell(root: &Path, id: &str) -> D<Option<Value>> {
    if id.is_empty() || !id_pattern_ok(id) {
        return Ok(None);
    }
    if let Some(v) = rj(&cells_dir(root).join(format!("{id}.json")))? {
        return Ok(Some(v));
    }
    let Ok(entries) = std::fs::read_dir(cells_dir(root).join(ARCHIVE_DIR_NAME)) else {
        return Ok(None);
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        if let Some(v) = rj(&entry.path().join(format!("{id}.json")))? {
            return Ok(Some(v));
        }
    }
    Ok(None)
}

/// provenance: cells.mjs listCells(root, {feature, status}) — the active scan
/// only (verbs/status_full.rs:1571 list_cells). The sort is LOAD-BEARING here:
/// scribingDebt maps the result to ids and close joins them into the
/// scribing-debt door detail, so the order reaches an emitted byte (caught by
/// a live diff against the beehive repo itself, where a plain byte sort put
/// "rust-port-5" after "rust-port-23").
pub(crate) fn list_cells(root: &Path, feature: &str, status: &str) -> D<Vec<Value>> {
    let mut cells: Vec<Value> = Vec::new();
    let Ok(entries) = std::fs::read_dir(cells_dir(root)) else {
        return Ok(cells);
    };
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.ends_with(".json") {
            continue;
        }
        let Some(cell) = rj(&entry.path())? else { continue };
        if !matches!(cell, Value::Object(_) | Value::Array(_)) {
            continue; // `typeof cell !== 'object'`
        }
        if !matches!(vget(&cell, "feature"), Some(Value::String(f)) if f == feature) {
            continue;
        }
        if !matches!(vget(&cell, "status"), Some(Value::String(s)) if s == status) {
            continue;
        }
        cells.push(cell);
    }
    cells.sort_by(|a, b| locale_cmp(&tpl(vget(a, "id")), &tpl(vget(b, "id")), true));
    Ok(cells)
}

/// Archive-aware sibling of `list_cells` above, for debt-door-archive dda-1:
/// `bee close` archives a feature's cells on a green close
/// (`.bee/cells/archive/<feature>/*.json`), so a debt counter that only
/// walks the live store the way `list_cells` does goes structurally silent
/// the moment its own feature closes. This reads the live store (exactly as
/// `list_cells` does) THEN every file directly under
/// `.bee/cells/archive/<feature>/`, deduplicating by id with the LIVE copy
/// winning on a duplicate — the exact live-copy-wins pattern
/// `verbs/knowledge/promote.rs:353-376` (`read_capped_cell_traces`) already
/// uses and `verbs/knowledge/tests.rs:682` already pins. `list_cells` itself
/// is untouched and stays active-only: every other caller (`bee cells list`,
/// `bee cells ready`, …) keeps its current behavior. Only
/// `close::scribing_debt` calls this variant.
pub(crate) fn list_cells_including_archive(root: &Path, feature: &str, status: &str) -> D<Vec<Value>> {
    let mut cells = list_cells(root, feature, status)?;
    let mut seen_ids: HashSet<String> = cells.iter().map(|c| tpl(vget(c, "id"))).collect();
    let archive_dir = cells_dir(root).join(ARCHIVE_DIR_NAME).join(feature);
    let Ok(entries) = std::fs::read_dir(&archive_dir) else {
        return Ok(cells);
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue; // a stray nested dir under the feature's archive slot
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.ends_with(".json") {
            continue;
        }
        let Some(cell) = rj(&entry.path())? else { continue };
        if !matches!(cell, Value::Object(_) | Value::Array(_)) {
            continue; // `typeof cell !== 'object'`
        }
        if !matches!(vget(&cell, "feature"), Some(Value::String(f)) if f == feature) {
            continue;
        }
        if !matches!(vget(&cell, "status"), Some(Value::String(s)) if s == status) {
            continue;
        }
        let id = tpl(vget(&cell, "id"));
        if !seen_ids.insert(id) {
            continue; // the live copy above already claimed this id
        }
        cells.push(cell);
    }
    cells.sort_by(|a, b| locale_cmp(&tpl(vget(a, "id")), &tpl(vget(b, "id")), true));
    Ok(cells)
}

// ─── String.prototype.localeCompare('en', {numeric:true}) ──────────────────
//
// VERBATIM LIFT of verbs/status_full.rs:429-503 (char_class_key + locale_cmp),
// whose own provenance is the measured V8/ICU behavior on the id/feature
// alphabet ([A-Za-z0-9._-] plus ISO timestamps):
//   primary:  class order _ < - < . < (other punct) < digits < letters
//             (letters case-folded; numeric mode compares digit runs BY VALUE,
//              so "01" == "1" with no length tiebreak, matching ICU)
//   tertiary: first case difference, lowercase before uppercase.
// R6 debt: promote to a shared module alongside the kctx lift.

pub(crate) fn char_class_key(c: char) -> (u8, u32) {
    if c.is_whitespace() {
        return (0, c as u32);
    }
    match c {
        '_' => (1, 0),
        '-' => (1, 1),
        ',' => (1, 2),
        ';' => (1, 3),
        ':' => (1, 4),
        '!' => (1, 5),
        '?' => (1, 6),
        '.' => (1, 7),
        _ if c.is_ascii_digit() => (2, c as u32 - '0' as u32),
        _ if c.is_alphabetic() => (3, c.to_lowercase().next().unwrap_or(c) as u32),
        _ => (1, 100 + c as u32),
    }
}

pub(crate) fn locale_cmp(a: &str, b: &str, numeric: bool) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let (mut i, mut j) = (0usize, 0usize);
    while i < av.len() && j < bv.len() {
        let (ca, cb) = (av[i], bv[j]);
        if numeric && ca.is_ascii_digit() && cb.is_ascii_digit() {
            let si = i;
            while i < av.len() && av[i].is_ascii_digit() {
                i += 1;
            }
            let sj = j;
            while j < bv.len() && bv[j].is_ascii_digit() {
                j += 1;
            }
            let ra: String = av[si..i].iter().collect();
            let rb: String = bv[sj..j].iter().collect();
            let ta = ra.trim_start_matches('0');
            let tb = rb.trim_start_matches('0');
            let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
            if ord != Ordering::Equal {
                return ord;
            }
            continue;
        }
        let ord = char_class_key(ca).cmp(&char_class_key(cb));
        if ord != Ordering::Equal {
            return ord;
        }
        i += 1;
        j += 1;
    }
    let ord = (av.len() - i).cmp(&(bv.len() - j));
    if ord != Ordering::Equal {
        return ord;
    }
    // Tertiary (case) pass — only when primary-equal.
    let (mut i, mut j) = (0usize, 0usize);
    while i < av.len() && j < bv.len() {
        let (ca, cb) = (av[i], bv[j]);
        if numeric && ca.is_ascii_digit() && cb.is_ascii_digit() {
            while i < av.len() && av[i].is_ascii_digit() {
                i += 1;
            }
            while j < bv.len() && bv[j].is_ascii_digit() {
                j += 1;
            }
            continue;
        }
        if ca != cb && ca.is_alphabetic() && cb.is_alphabetic() {
            let (la, lb) = (ca.is_lowercase(), cb.is_lowercase());
            if la != lb {
                return if la { Ordering::Less } else { Ordering::Greater };
            }
        }
        i += 1;
        j += 1;
    }
    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_economics_herding_exec() {
        let e = derive_economics("herding-exec", "generation", None, &Resolved::Budget, false);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":null,"effective_model":null,"effective_model_status":"unverified","channel":"herding-exec","enforcement":"herding-command"}"#
        );
    }

    #[test]
    fn derive_economics_session_model() {
        let e = derive_economics("session-model", "ceiling", None, &Resolved::Inherit, false);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"ceiling","requested_model":null,"effective_model":null,"effective_model_status":"inherited-or-unknown","channel":"session-model","enforcement":"session-model"}"#
        );
    }
}
