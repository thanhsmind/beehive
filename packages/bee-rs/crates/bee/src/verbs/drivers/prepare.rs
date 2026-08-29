// dispatch prepare
//
// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::herding::{transport_kind_at, TransportKind};
use crate::state::read_config_raw;
use crate::verbs::knowledge;
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

// ═══ dispatch prepare ══════════════════════════════════════════════════════

/// pi-support D5 (store: the pi belt's dispatch door): `pi` joins codex and
/// claude as a legal `--runtime`, resolving `models.pi` in the ONE config home
/// every other runtime reads. It is a HERDING-ONLY door — see
/// `pi_requires_herding_refusal` — because Pi ships no Agent/subagent tool
/// surface for a payload to name.
///
/// This constant is one of THREE gates a runtime name passes: the shape guard
/// reads the `runtime` enum in `generated/registry_payload.json` (declared for
/// `dispatch.prepare` AND `dispatch.wave`) BEFORE the handler runs, this list
/// gates the handlers themselves, and `devtools::render_projection_text_for`
/// carries the label arm. All three move together or the door half-opens.
pub(crate) const DISPATCH_RUNTIMES: [&str; 3] = ["codex", "claude", "pi"];

pub(crate) const DISPATCH_KINDS: [&str; 4] = ["cell", "gather", "reviewer", "advisor"];

/// provenance: dispatch-prepare.mjs slotForKind (the PURPOSE MAP, advisor A1).
///
/// D2 (decision 06e49368) — the second silent-resolve site, closed. This map
/// used to end in a catch-all `_ => "advisor"`, so ANY kind without its own
/// arm resolved the advisor slot. `kind` is enum-gated against
/// `DISPATCH_KINDS` (at the flag parse and again at `prepare_dispatch`'s
/// entry), so a typo could never reach it — the hazard was a kind added to
/// `DISPATCH_KINDS` later with no arm here, which would have routed that work
/// to the advisor model with nothing red to show for it. Every kind now names
/// its slot EXPLICITLY; an unhandled kind returns `None` and its caller
/// refuses (`unmapped_kind_refusal`).
pub(crate) fn slot_for_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "cell" | "gather" => Some("generation"),
        "reviewer" => Some("review"),
        "advisor" => Some("advisor"),
        _ => None,
    }
}

/// The typed refusal a kind with no `slot_for_kind` arm earns: `{ok:false}`
/// naming its remedy, in the same shape as every other prepare refusal —
/// never a silent resolution onto some other consumer's model.
pub(crate) fn unmapped_kind_refusal(kind: &str) -> Value {
    let mut refusal = Map::new();
    refusal.insert("ok".into(), Value::Bool(false));
    refusal.insert("type".into(), Value::String("refused".into()));
    refusal.insert("reason".into(), Value::String("kind_slot_unmapped".into()));
    refusal.insert("kind".into(), Value::String(kind.to_string()));
    refusal.insert(
        "fix".into(),
        Value::String(format!(
            "dispatch kind \"{kind}\" has no slot mapping — add an explicit arm for it to slot_for_kind (verbs/drivers/prepare.rs) beside its DISPATCH_KINDS entry. An unmapped kind is refused, never resolved onto the advisor slot."
        )),
    );
    Value::Object(refusal)
}

/// The `reason` every pi-runtime refusal carries — ONE reason word, so a
/// caller at this door branches on one string rather than five.
pub(crate) const PI_HERDING_ONLY_REASON: &str = "pi_requires_herding";

/// What a slot resolved to, in the words the refusal reports it under.
/// `Herding` never reaches here (it is the one resolution the pi door
/// serves); `Refused` arrives from the cli purpose gate, which on pi is a
/// cli slot either way.
pub(crate) fn pi_resolution_word(resolved: &Resolved, escalated: bool) -> &'static str {
    if escalated {
        return "escalation";
    }
    match resolved {
        Resolved::Inherit => "escalation",
        Resolved::Model { .. } => "model",
        Resolved::Native { .. } => "native",
        Resolved::Cli { .. } | Resolved::Refused { .. } => "cli",
        Resolved::Budget => "budget",
        Resolved::Herding { .. } => "herding",
    }
}

/// pi-support D5, at FULL width: on runtime `pi`, EVERY slot resolution that
/// is not `Resolved::Herding` is refused BY NAME.
///
/// Pi has no Agent tool and no `spawn_agent` (store `7f9c8518`: no native
/// subagents), so an Agent payload, a codex `spawn_agent` payload, a plain
/// `model` parameter or a bare cli command emitted for the pi runtime would
/// dispatch NOTHING while the envelope read as a successful dispatch. The
/// herding pane is the one transport Pi can actually take, so the door emits
/// the herding-exec payload or it refuses.
///
/// The escalation arm (`Resolved::Inherit` — an escalated cell, or an explicit
/// `--role ceiling`) carries its own remedy: there is no subagent to run on
/// the session model, so the escalated cell runs INLINE in the session. Every
/// other arm names the slot and the herding shape to configure.
pub(crate) fn pi_requires_herding_refusal(slot: &str, resolved: &Resolved, escalated: bool) -> Value {
    let resolution = pi_resolution_word(resolved, escalated);
    let mut refusal = Map::new();
    refusal.insert("ok".into(), Value::Bool(false));
    refusal.insert("type".into(), Value::String("refused".into()));
    refusal.insert("reason".into(), Value::String(PI_HERDING_ONLY_REASON.into()));
    refusal.insert("runtime".into(), Value::String("pi".into()));
    refusal.insert("slot".into(), Value::String(slot.to_string()));
    refusal.insert("resolution".into(), Value::String(resolution.to_string()));
    let fix = if resolution == "escalation" {
        format!(
            "the \"{slot}\" dispatch asked for the session model (escalation), and the pi runtime has no session subagent to hand it to — Pi ships no Agent tool surface, so the payload would dispatch nothing. FIX: Pi has no subagent surface, run the escalated cell inline in the session. Every other cell on pi resolves a {{\"kind\":\"herding\"}} slot under models.pi."
        )
    } else {
        format!(
            "models.pi.{slot} resolved a \"{resolution}\" slot, and the pi runtime dispatches ONLY through herding — Pi ships no Agent tool surface, so any other payload would dispatch nothing. FIX: set models.pi.{slot} in .bee/config.json to {{\"kind\":\"herding\",\"agent\":\"<herding.agents name>\"}}."
        )
    };
    refusal.insert("fix".into(), Value::String(fix));
    Value::Object(refusal)
}

// ─── the LaneBrief carrier (slp-blind-lanes E1/E2, decision 5981246b D2) ───
//
// `--brief-file <path>` is the FIRST caller text that reaches a non-cell
// prompt body: every non-cell kind used to render with an empty vars slice.
// It exists so a blind lane can be handed one question, byte-identical across
// 2–3 parallel advisor dispatches, without a new dispatch kind and without a
// new store (decision f0f21142).

/// The cap on a carried brief, in bytes of the file as read from disk.
///
/// A brief is a QUESTION plus its constraints and read diet, not a document:
/// past this size the thing being carried is context, and context is what the
/// lane is supposed to go and read for itself.
pub(crate) const BRIEF_MAX_BYTES: usize = 8192;

/// The typed refusal shape every `--brief-file` failure takes — the same
/// `{ok:false, type:"refused", reason, …, fix}` `unmapped_kind_refusal`
/// returns, so a caller at this door parses ONE refusal shape, never two.
/// `fix` is inserted last so it reads at the end of the object, as it does
/// there.
pub(crate) fn brief_refusal(reason: &str, extra: &[(&str, Value)], fix: String) -> Value {
    let mut refusal = Map::new();
    refusal.insert("ok".into(), Value::Bool(false));
    refusal.insert("type".into(), Value::String("refused".into()));
    refusal.insert("reason".into(), Value::String(reason.to_string()));
    for (k, v) in extra {
        refusal.insert((*k).to_string(), v.clone());
    }
    refusal.insert("fix".into(), Value::String(fix));
    Value::Object(refusal)
}

/// Resolve `--brief-file` into the brief THIS dispatch carries.
///
/// * `Ok(None)` — no brief travels. Either the flag was absent, or the file
///   held nothing but whitespace: `{{#if}}` truthiness is `!v.is_empty()`
///   (`prompt.rs`), so a whitespace-only brief that reached the renderer
///   would splice an EMPTY block where today's bytes are. Trimmed-empty
///   therefore resolves to no brief at all, and the payload stays
///   byte-identical to a dispatch that passed no `--brief-file`.
/// * `Ok(Some(text))` — the trimmed brief, which is both what renders into
///   the prompt and what `brief_sha256` digests. Digesting the CARRIED bytes
///   rather than the file's is deliberate: equal digests then mean the lanes
///   received equal payloads, which is exactly what D2(b)'s byte-identity
///   claim is about.
/// * `Err(refusal)` — a typed `{ok:false, type:"refused", …}` value.
///
/// ADVISOR ONLY, and refused loudly everywhere else. `--kind cell` is D3: a
/// worker-cell prompt injects machine-assembled `learned_context` shared
/// across workers, which leaks the very thing blindness protects. `gather`
/// and `reviewer` carry no `{{#if brief}}` block at all, so accepting a brief
/// for them would swallow it silently — the one outcome this door must never
/// have.
///
/// The path is read AS GIVEN, relative to the process cwd, the same way every
/// other file-consuming flag in this CLI reads one (`read_file_text`,
/// verbs/cells/handlers_write.rs).
pub(crate) fn resolve_brief_file(kind: &str, path: Option<&str>) -> Result<Option<String>, Value> {
    let Some(path) = path else { return Ok(None) };
    if kind != "advisor" {
        return Err(brief_refusal(
            "brief_kind_not_advisor",
            &[("kind".into(), Value::String(kind.to_string()))],
            format!(
                "--brief-file is only valid with --kind advisor (got --kind {kind}). A blind lane never runs as --kind cell — the worker-cell prompt injects learned_context shared across workers, which leaks what blindness protects — and the gather/reviewer prompts carry no brief block, so a brief passed to them would be swallowed with nothing to show for it. Re-run the dispatch as --kind advisor."
            ),
        ));
    }
    let trimmed_path = js_trim(path);
    if trimmed_path.is_empty() {
        return Err(brief_refusal(
            "brief_file_unreadable",
            &[("path".into(), Value::String(path.to_string()))],
            "--brief-file needs the path of a readable file holding the brief; it was given no path at all. Pass --brief-file <path>.".to_string(),
        ));
    }
    let Ok(bytes) = std::fs::read(trimmed_path) else {
        return Err(brief_refusal(
            "brief_file_unreadable",
            &[("path".into(), Value::String(trimmed_path.to_string()))],
            format!(
                "--brief-file \"{trimmed_path}\" could not be read. The path is resolved from the current working directory — check the spelling, or pass an absolute path."
            ),
        ));
    };
    if bytes.len() > BRIEF_MAX_BYTES {
        return Err(brief_refusal(
            "brief_too_large",
            &[
                ("path".into(), Value::String(trimmed_path.to_string())),
                ("bytes".into(), Value::Number(Number::from(bytes.len()))),
                ("max_bytes".into(), Value::Number(Number::from(BRIEF_MAX_BYTES))),
            ],
            format!(
                "--brief-file \"{trimmed_path}\" is {} bytes; the cap is {BRIEF_MAX_BYTES} bytes. A brief is the QUESTION plus its constraints and read diet — anything longer is context, which the lane reads for itself from the paths the brief names. Cut it, or move the bulk into the read diet.",
                bytes.len()
            ),
        ));
    }
    let Ok(text) = String::from_utf8(bytes) else {
        return Err(brief_refusal(
            "brief_not_utf8",
            &[("path".into(), Value::String(trimmed_path.to_string()))],
            format!(
                "--brief-file \"{trimmed_path}\" is not valid UTF-8. It is refused rather than decoded lossily: a lossy decode would change the very bytes brief_sha256 exists to pin, and every lane would agree on the corrupted text. Save the brief as UTF-8."
            ),
        ));
    };
    let trimmed = js_trim(&text);
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed.to_string()))
}

/// slp-blind-lanes-procedure P1: a reading list may NEVER ride beside a
/// brief.
///
/// `--expertise` is prose that reaches the lane's prompt exactly as the brief
/// does, and two things break when the two travel together:
///
/// * the leaning guard reads BRIEF BYTES ONLY, by deliberate design
///   (`lint_brief` is handed one `&str` and nothing more), so a reading list
///   riding alongside is an unlinted prose channel straight into a blind
///   lane — the one channel `brief_lint.rs` exists to close;
/// * `brief_sha256` would keep proving the BRIEFS were byte-identical across
///   2–3 lanes while the payloads themselves diverged, which is precisely
///   what the digest is stamped to rule out.
///
/// So the combination is a typed refusal, in the same
/// `{ok:false, type:"refused", reason, fix}` shape `brief_kind_not_advisor`
/// takes, and its fix names the brief's OWN `## Read diet` section — a
/// required section of every LaneBrief, already checked against the reported
/// paths by `bee blind check` — as the one carrier. One list, one carrier,
/// one linted channel.
///
/// It speaks only for `advisor`: every other kind may not carry a brief at
/// all, and the kind arm below owns that refusal with its own remedy.
/// `expertise` here is the RENDERED block, so an empty or absent list carries
/// nothing and refuses nothing.
pub(crate) fn expertise_beside_brief_refusal(
    kind: &str,
    brief_path: Option<&str>,
    expertise: Option<&str>,
) -> Option<Value> {
    if kind != "advisor" {
        return None;
    }
    brief_path?;
    let expertise = expertise?;
    if expertise.is_empty() {
        return None;
    }
    Some(brief_refusal(
        "expertise_beside_brief",
        &[],
        "--expertise cannot travel with --brief-file. The leaning guard reads the brief's bytes and nothing else, so a reading list passed alongside reaches the lane unlinted, and brief_sha256 would keep certifying that the briefs matched while the payloads differed. A brief already carries its reading list: put the paths in the brief's own `## Read diet` section — the section `bee blind check` checks the reported paths against — and drop --expertise from this dispatch.".to_string(),
    ))
}

/// `resolve_brief_file` with the combination check in front of it — the
/// arity-adapter shape `prepare_dispatch_with_brief` already uses, so there
/// is ONE resolution order and a caller cannot accidentally read the brief
/// first. The refusal fires BEFORE any file is touched: an unreadable path
/// passed beside a reading list earns `expertise_beside_brief`, never
/// `brief_file_unreadable`.
pub(crate) fn resolve_brief_with_expertise(
    kind: &str,
    path: Option<&str>,
    expertise: Option<&str>,
) -> Result<Option<String>, Value> {
    if let Some(refusal) = expertise_beside_brief_refusal(kind, path, expertise) {
        return Err(refusal);
    }
    resolve_brief_file(kind, path)
}

/// A non-empty string field off a loaded cell record, or `None`. Written once
/// because the dispatch now reads TWO of them in precedence order (`role`,
/// then a pre-mrs-8 record's `tier`) and two hand-rolled copies of the same
/// "empty string is no value" rule is how the two drift.
pub(crate) fn recorded_str<'a>(cell: Option<&'a Value>, field: &str) -> Option<&'a str> {
    cell.and_then(|c| vget(c, field)).and_then(|v| match v {
        Value::String(s) if !s.is_empty() => Some(s.as_str()),
        _ => None,
    })
}

/// The ordered role list a CELL EXECUTION asks for, per store 561e1bda.
///
/// `[<the cell's own role>, code, generation]`, and for a read-shaped cell
/// `[read, extraction, generation]` — the read consumer's own literal list,
/// reached here because a cell is the only thing that can DECLARE itself a
/// read job (D9 backfills every `tier: extraction` cell to `role: read`).
///
/// THE TAIL IS LOAD-BEARING. Fall-through walks to the first name that
/// resolves, so a list ending at `code` would walk straight past the
/// `generation` model every existing host has configured for years and land
/// on bee's built-in default — a silent migration onto a different model, the
/// exact defect this feature exists to remove. The tail costs one array entry
/// and makes the upgrade a no-op for every host that has not opted in.
///
/// The advisor is deliberately NOT here: it keeps `resolve_advisor`, one name
/// and no fall-through at all (decision 4faf1de9).
///
/// lane-model-diversity D2 (store `23de5362`) does NOT change that. An
/// unconfigured SEAT role (`lane-2`, `hat-risks`) falls through to the advisor
/// at the `--kind advisor` door only, and it does so by REBINDING the role
/// before resolution — never by appending `advisor` to an ordered list, here
/// or anywhere. The advisor's tail is still its own floor-less one-name walk,
/// and a cell declaring a seat role keeps this exact list, byte for byte.
pub(crate) fn cell_role_list(role: &str) -> Vec<&str> {
    // role-surface-cleanup D1: the tail names skip anything already in the
    // list — a duplicate made the fall-through warn fire twice for one
    // dispatch, and its first copy named the very name that just failed.
    let tail: &[&str] =
        if role == "read" { &["extraction", "generation"] } else { &["code", "generation"] };
    let mut list = vec![role];
    for name in tail {
        if !list.contains(name) {
            list.push(name);
        }
    }
    list
}

/// provenance: dispatch-prepare.mjs purposeForKind — only 'cell' is
/// cell-execution; everything else is an explicit read-only gather.
pub(crate) fn purpose_is_gather(kind: &str) -> bool {
    kind != "cell"
}

pub(crate) fn is_cell_tier_configured(
    models: &Map<String, Value>,
    runtime: &str,
    tier: &str,
) -> bool {
    if !CONFIGURABLE_SLOTS.contains(&tier) {
        return false;
    }
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    let table = models.get(rt);
    if tier == "generation" {
        if rt == "claude" {
            table.and_then(|t| t.get("generation")).map(|v| !v.is_null()).unwrap_or(false)
        } else {
            true
        }
    } else if tier == "review" {
        let review_val = table.and_then(|t| t.get("review"));
        if review_val.map(|v| !v.is_null()).unwrap_or(false) {
            true
        } else if rt == "claude" {
            table.and_then(|t| t.get("generation")).map(|v| !v.is_null()).unwrap_or(false)
        } else {
            true
        }
    } else {
        table.and_then(|t| t.get(tier)).map(|v| !v.is_null()).unwrap_or(false)
    }
}

pub(crate) struct Ownership {
    pub(crate) ok: bool,
    pub(crate) code: Option<&'static str>,
    pub(crate) status: Value,
    pub(crate) owner: Value,
    pub(crate) reason: String,
}

/// provenance: dispatch-prepare.mjs checkCellClaimOwnership (hardening-7) —
/// the CELL RECORD's own status/trace.worker, never the claims store.
pub(crate) fn check_cell_claim_ownership(cell: &Value, worker: &str) -> Ownership {
    let status = vget(cell, "status").cloned().unwrap_or(Value::Null);
    let status_str = tpl(vget(cell, "status"));
    let id = tpl(vget(cell, "id"));
    if !matches!(vget(cell, "status"), Some(Value::String(s)) if s == "claimed") {
        return Ownership {
            ok: false,
            code: Some("not_claimed"),
            status,
            owner: Value::Null,
            reason: format!(
                "cell \"{id}\" is \"{status_str}\", not \"claimed\" — dispatch prepare requires a claimed cell (run bee cells claim or bee cells claim-next first). Pass --force-ownership to override (audited)."
            ),
        };
    }
    let owner: Value = match vget(cell, "trace").and_then(|t| vget(t, "worker")) {
        Some(Value::String(w)) => Value::String(w.clone()),
        _ => Value::Null,
    };
    let owner_matches = matches!(&owner, Value::String(w) if w == worker);
    if !owner_matches {
        let shown = match &owner {
            Value::String(w) if !w.is_empty() => w.clone(),
            _ => "(unknown)".to_string(), // `owner || '(unknown)'`
        };
        return Ownership {
            ok: false,
            code: Some("not_owner"),
            status,
            owner,
            reason: format!(
                "cell \"{id}\" is claimed by worker \"{shown}\" — \"{worker}\" does not own this claim. Pass --force-ownership to override (audited)."
            ),
        };
    }
    Ownership { ok: true, code: None, status, owner, reason: String::new() }
}

/// provenance: dispatch-prepare.mjs PRIOR_ROUNDS_MAX_EVENT_LINES.
pub(crate) const PRIOR_ROUNDS_MAX_EVENT_LINES: usize = 12;

/// provenance: dispatch-prepare.mjs LEARNED_CONTEXT_MAX_LINES.
pub(crate) const LEARNED_CONTEXT_MAX_LINES: usize = 8;

/// How much of a cell's title the Agent-list row carries. Long enough to say
/// what the work is, short enough that the row still reads at a glance.
pub(crate) const DESCRIPTION_TITLE_MAX: usize = 60;

/// How much of the dispatch SUBJECT codex's `task_name` field carries. The
/// live-probed 0.145.0 schema (docs/knowledge/areas/hook-runtime/codex-spawn-
/// agent-dispatch-payload-schema-and-schema-agnosti.md) types `task_name` as
/// a plain required string with no documented charset or length limit, so
/// this exists to keep the row readable, not to satisfy a schema constraint —
/// kept as its own constant rather than reusing DESCRIPTION_TITLE_MAX in case
/// the two ever need to diverge.
pub(crate) const TASK_NAME_MAX: usize = 60;

/// provenance: dispatch-prepare.mjs priorRoundEventLines — the machine-
/// assembled digest of the cell record's own trace history, chronological
/// (ISO strings compare lexicographically; timeless events sink to the end in
/// insertion order, the sort being stable in both runtimes), capped at 12 with
/// one count line replacing the elided oldest.
pub(crate) fn prior_round_event_lines(cell: &Value) -> Vec<String> {
    let trace = match vget(cell, "trace") {
        Some(v) if is_plain_object(v) => v.clone(),
        _ => Value::Object(Map::new()),
    };
    let arr = |key: &str| -> Vec<Value> {
        match vget(&trace, key) {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        }
    };
    // (at, line) — `at` is `null` for a timeless event.
    let mut events: Vec<(Option<String>, String)> = Vec::new();
    let at_of = |v: &Value, key: &str| -> Option<String> {
        // `attempt.at || null` — a falsy `at` becomes null.
        match vget(v, key) {
            Some(x) if truthy(x) => Some(jsjson::js_to_string(x)),
            _ => None,
        }
    };

    for attempt in arr("attempts") {
        if !is_plain_object(&attempt) {
            continue;
        }
        let worker = truthy_str(vget(&attempt, "worker"))
            .map(str::to_string)
            .unwrap_or_else(|| "(unknown worker)".to_string());
        let verdict = vget(&attempt, "verdict");
        let sig = || match vget(&attempt, "failure_signature") {
            Some(v) if truthy(v) => jsjson::js_to_string(v),
            _ => "(none recorded)".to_string(),
        };
        if matches!(verdict, Some(Value::String(s)) if s == "blocked") {
            let note = one_line(vget(&attempt, "note"), 140);
            let reason = if note.is_empty() {
                format!("failure signature {}", sig())
            } else {
                note
            };
            events.push((at_of(&attempt, "at"), format!("- {worker} blocked: {reason}")));
        } else if matches!(verdict, Some(Value::String(s)) if s == "tests-red") {
            let note = one_line(vget(&attempt, "note"), 140);
            let note = if note.is_empty() { "(no excerpt recorded)".to_string() } else { note };
            events.push((at_of(&attempt, "at"), format!("- {worker} tests red: {note}")));
        } else if matches!(verdict, Some(Value::String(s)) if s == "fail") {
            events.push((
                at_of(&attempt, "at"),
                format!("- {worker} failed verify: failure signature {}", sig()),
            ));
        }
    }

    let capped_at = match vget(&trace, "capped_at") {
        Some(v) if truthy(v) => Some(jsjson::js_to_string(v)),
        _ => None,
    };
    for deviation in arr("deviations") {
        let Value::String(text) = &deviation else { continue };
        if js_trim(text).is_empty() {
            continue;
        }
        events.push((
            capped_at.clone(),
            format!("- (prior worker) deviation: {}", one_line(Some(&deviation), 140)),
        ));
    }

    for consult in arr("semantic_judge") {
        if !is_plain_object(&consult) {
            continue;
        }
        let judge = truthy_str(vget(&consult, "judge_model"))
            .map(str::to_string)
            .unwrap_or_else(|| "(judge)".to_string());
        let pointer = match vget(&consult, "failure_signature") {
            Some(v) if truthy(v) => {
                format!(" (failure signature {})", one_line(Some(v), 40))
            }
            _ => String::new(),
        };
        events.push((
            at_of(&consult, "recorded_at"),
            format!("- {judge} consult: {}{pointer}", tpl(vget(&consult, "verdict"))),
        ));
    }

    if let Some(Value::String(reason)) = vget(&trace, "reopened_reason") {
        if !js_trim(reason).is_empty() {
            events.push((
                at_of(&trace, "reopened_at"),
                format!(
                    "- (orchestrator) reopened: {}",
                    one_line(vget(&trace, "reopened_reason"), 140)
                ),
            ));
        }
    }
    if let Some(rework) = vget(&trace, "reopened_for_rework") {
        if truthy(rework) && is_plain_object(rework) {
            let reason = one_line(vget(rework, "reason"), 140);
            let reason = if reason.is_empty() {
                "NEEDS_REVISION verdict after cap".to_string()
            } else {
                reason
            };
            events.push((
                at_of(rework, "at"),
                format!("- (judge) reopened for rework: {reason}"),
            ));
        }
    }

    // Stable sort with Node's own comparator.
    events.sort_by(|a, b| match (&a.0, &b.0) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, _) => std::cmp::Ordering::Greater,
        (_, None) => std::cmp::Ordering::Less,
        (Some(x), Some(y)) => x.cmp(y),
    });
    let mut lines: Vec<String> = events.into_iter().map(|(_, line)| line).collect();
    if lines.len() > PRIOR_ROUNDS_MAX_EVENT_LINES {
        let kept = PRIOR_ROUNDS_MAX_EVENT_LINES - 1;
        let elided = lines.len() - kept;
        let tail = lines.split_off(lines.len() - kept);
        lines = std::iter::once(format!(
            "- ({elided} earlier event(s) elided — the cell record holds the rest)"
        ))
        .chain(tail)
        .collect();
    }
    lines
}

/// provenance: knowledge.mjs KNOWLEDGE_CONTEXT_LANE_BUDGETS /
/// KNOWLEDGE_CONTEXT_DEFAULT_BUDGET, read through `budgets[cell.lane] ?? default`.
pub(crate) fn lane_budget(lane: Option<&Value>) -> f64 {
    match lane {
        Some(Value::String(l)) => match l.as_str() {
            "tiny" => 8000.0,
            "small" => 12000.0,
            "standard" => 20000.0,
            "high-risk" => 30000.0,
            _ => 20000.0,
        },
        _ => 20000.0,
    }
}

/// provenance: dispatch-prepare.mjs bundleLearnedLines — the work-item
/// manifest first (every failure inside its try/catch falls through), then the
/// bundle index pointer. `Err(Delegate)` is NOT a JS-visible failure: it means
/// the lifted knowledge port cannot decide this bundle, so the whole command
/// re-runs under Node.
pub(crate) fn bundle_learned_lines(
    root: &Path,
    cell: &Value,
    read_first: &HashSet<String>,
) -> D<Vec<String>> {
    let Some(dir) = knowledge::bundle_dir(root) else { return Err(Delegate) };
    let budget = lane_budget(vget(cell, "lane"));
    let work = match vget(cell, "feature") {
        Some(Value::String(s)) => s.clone(),
        // A non-string `work` makes buildContextManifest throw missing_work
        // (`typeof work === 'string' ? work.trim() : ''`) -> the catch arm.
        _ => String::new(),
    };
    let manifest = if work.is_empty() {
        None
    } else {
        match knowledge::build_context_manifest(&dir, &work, budget, &knowledge::num(budget)) {
            knowledge::ManifestOut::Built(m) => Some(m),
            knowledge::ManifestOut::Thrown(_) => None, // caught by dispatch-prepare's try
            knowledge::ManifestOut::NeedsNode => return Err(Delegate),
        }
    };
    if let Some(manifest) = manifest {
        let Some(concepts) = knowledge::collect_concepts(&dir) else { return Err(Delegate) };
        // `new Map(...)`: last write wins per key (never hit for a real bundle,
        // where paths are unique).
        let mut titles: Vec<(String, Option<String>)> = Vec::new();
        for concept in &concepts {
            let key = format!("docs/knowledge/{}", concept.path);
            let title = match concept.data.get("title") {
                Some(Value::String(t)) if !t.is_empty() => Some(t.clone()),
                _ => None,
            };
            if let Some(slot) = titles.iter_mut().find(|(k, _)| *k == key) {
                slot.1 = title;
            } else {
                titles.push((key, title));
            }
        }
        let entries = match manifest.get("entries") {
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        let mut lines = Vec::new();
        for entry in &entries {
            let path = tpl(vget(entry, "path"));
            if read_first.contains(&path) {
                continue; // read_first stays authoritative — never duplicated
            }
            // `titles.get(entry.path) || entry.path.slice(lastIndexOf('/') + 1)`
            let title = titles
                .iter()
                .find(|(k, _)| *k == path)
                .and_then(|(_, t)| t.clone())
                .unwrap_or_else(|| match path.rfind('/') {
                    Some(p) => path[p + 1..].to_string(),
                    None => path.clone(),
                });
            // The manifest path is a raw filesystem-derived string: a
            // crafted bundle filename could carry a newline and forge extra
            // bullet lines inside the worker prompt's "Learned context"
            // block, so it gets the same whitespace-collapsing treatment as
            // the title (a generous cap — paths are not expected to be
            // long, but never truncate a real one in practice).
            lines.push(format!(
                "- {} — {}",
                one_line(Some(&Value::String(path.clone())), 200),
                one_line(Some(&Value::String(title)), 140)
            ));
        }
        if !lines.is_empty() {
            return Ok(lines);
        }
    }
    if dir.join("index.md").exists() && !read_first.contains("docs/knowledge/index.md") {
        return Ok(vec![
            "- docs/knowledge/index.md — Knowledge bundle index (see \"Critical patterns\")"
                .to_string(),
        ]);
    }
    Ok(Vec::new())
}

/// provenance: knowledge.mjs bundleMode — a DIRECTORY is not a bundle: at
/// least one non-reserved markdown file must parse as a strict OKF concept
/// carrying a non-empty string `type`.
pub(crate) fn bundle_mode(root: &Path) -> D<bool> {
    let Some(dir) = knowledge::bundle_dir(root) else { return Err(Delegate) };
    match std::fs::metadata(&dir) {
        Ok(m) if m.is_dir() => {}
        _ => return Ok(false),
    }
    let Some(rels) = knowledge::list_bundle_markdown(&dir) else { return Err(Delegate) };
    for rel in rels {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if knowledge::is_reserved_basename(base) {
            continue;
        }
        let Ok(text) = knowledge::read_file_lossy(&knowledge::join_rel(&dir, &rel)) else { continue };
        match knowledge::parse_frontmatter(&text) {
            knowledge::Fm::Parsed { data, .. } => {
                if matches!(data.get("type"), Some(Value::String(t)) if !t.is_empty()) {
                    return Ok(true);
                }
            }
            _ => {}
        }
    }
    Ok(false)
}

/// provenance: dispatch-prepare.mjs learnedContextLines — source resolution,
/// first hit wins, capped at LEARNED_CONTEXT_MAX_LINES.
pub(crate) fn learned_context_lines(root: &Path, cell: &Value) -> D<Vec<String>> {
    let mut read_first: HashSet<String> = HashSet::new();
    if let Some(Value::Array(items)) = vget(cell, "read_first") {
        for entry in items {
            if let Value::String(s) = entry {
                let normalized = s.replace('\\', "/");
                let normalized = normalized.strip_prefix("./").unwrap_or(&normalized);
                read_first.insert(normalized.to_string());
            }
        }
    }
    let mut lines = if bundle_mode(root)? {
        bundle_learned_lines(root, cell, &read_first)?
    } else if root
        .join("docs")
        .join("history")
        .join("learnings")
        .join("critical-patterns.md")
        .exists()
        && !read_first.contains("docs/history/learnings/critical-patterns.md")
    {
        vec![
            "- docs/history/learnings/critical-patterns.md — Critical patterns (hard-won learnings)"
                .to_string(),
        ]
    } else {
        Vec::new()
    };
    lines.truncate(LEARNED_CONTEXT_MAX_LINES);
    Ok(lines)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpertiseEntry {
    pub path: String,
    pub purpose: String,
    pub read_to: String,
}

pub(crate) fn parse_expertise(raw: &str) -> Result<Vec<ExpertiseEntry>, String> {
    let mut entries = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(" :: ").collect();
        if parts.len() != 3 || parts.iter().any(|s| s.trim().is_empty()) {
            return Err(format!(
                "malformed --expertise line (want '<path> :: <purpose> :: <read-to>'): {line}"
            ));
        }
        entries.push(ExpertiseEntry {
            path: parts[0].trim().to_string(),
            purpose: parts[1].trim().to_string(),
            read_to: parts[2].trim().to_string(),
        });
    }
    Ok(entries)
}

/// D5/D6 — the user's verbatim request, framed for a prompt, or `""`.
///
/// The framing is the intent anchor's OWN header and footer constants, reused
/// rather than retyped: the same "VERBATIM · DO NOT SUMMARIZE · DO NOT
/// PARAPHRASE" banner the compaction checkpoint already emits, so a worker
/// meets one framing for the user's words wherever it meets them.
///
/// `""` is falsy to `{{#if}}`, which drops the block WITH its own leading
/// newline — so a dispatch that resolves no anchor renders byte-identically
/// to what this door produced before the block existed.
pub(crate) fn original_request_block(root: &Path, feature: Option<&str>) -> String {
    match crate::verbs::intent_group::dispatch_original_request(root, feature) {
        Some(request) => format!(
            "{}\nORIGINAL REQUEST (verbatim):\n{request}\n{}",
            crate::verbs::intent_group::PRECOMPACT_HEADER,
            crate::verbs::intent_group::PRECOMPACT_FOOTER,
        ),
        None => String::new(),
    }
}

/// provenance: dispatch-prepare.mjs cellPromptBody / promptBodyFor.
pub(crate) fn prompt_body_for(
    root: &Path,
    kind: &str,
    cell: Option<&Value>,
    worker: Option<&str>,
    // `Some((worktree_root, control_root))` only when the cell's feature has
    // a granted worktree — see `worktree_location` in `prepare_dispatch`,
    // the ONE resolution this and the envelope both read. `None` (an
    // unworktreed feature, or a non-cell kind) renders byte-identically to
    // before this Location block existed: the `{{#if worktree_root}}` marker
    // strips to nothing when its var is empty.
    worktree_location: Option<(&str, &str)>,
    expertise: Option<&str>,
    // The carried LaneBrief, already resolved and trimmed by
    // `resolve_brief_file` (advisor kind only; `None` everywhere else).
    brief: Option<&str>,
) -> D<Result<String, String>> {
    if kind != "cell" {
        let Some(template) = load_prompt(kind) else { return Err(Delegate) };
        // Every var a non-cell template can carry. Absent renders the empty
        // string, which is falsy to `{{#if}}`, which drops the block WITH
        // its own leading newline — so a gather/reviewer/advisor payload
        // carrying neither is byte-identical to what this line produced when
        // it passed `&[]`.
        //
        // `expertise` is here because it was NOT: this arm rendered `brief`
        // alone, so a dispatcher's `--expertise` reading list was parsed,
        // rendered to a block and then silently dropped for every kind but
        // `cell`. Pass 2 refuses a `{{NAME}}` with no supplied value, so a
        // var declared in a template MUST appear in this slice or every
        // dispatch of that kind dies at the door.
        //
        // `original_request` is feature-keyed only, and a non-cell kind
        // carries no cell — so it resolves from the ACTIVE feature or not at
        // all. A gather/reviewer/advisor dispatch on an idle repo renders no
        // block even when a non-empty `.bee/intent/default.json` exists
        // (D5/D6: rendering the wrong request is worse than rendering none).
        let original_request = original_request_block(root, None);
        return Ok(render(
            &template,
            &[
                ("brief", brief.unwrap_or("")),
                ("expertise", expertise.unwrap_or("")),
                ("original_request", &original_request),
            ],
        ));
    }
    let cell = cell.expect("kind cell always carries a loaded cell");
    let Some(template) = load_prompt("worker-cell") else { return Err(Delegate) };
    let (worktree_root, control_root) = worktree_location.unwrap_or(("", ""));
    // The knowledge bundle (docs/knowledge/) and docs/history/ live in the
    // repo WORKING TREE, never the control store: when the cell's feature
    // carries a granted worktree, that worktree's own checkout is the
    // native root for the bundle read, exactly like a native verb invoked
    // from inside it (roots.rs's WIDE door) — never the control root's own
    // (possibly bundle-less, possibly stale) checkout.
    let bundle_root: &Path = if worktree_root.is_empty() { root } else { Path::new(worktree_root) };
    let learned = learned_context_lines(bundle_root, cell)?.join("\n");
    let prior = prior_round_event_lines(cell).join("\n");
    let cell_json = jsjson::stringify_pretty(cell);
    let feature = tpl(vget(cell, "feature"));
    let cell_id = tpl(vget(cell, "id"));
    // The cell's OWN feature keys the anchor read, then the active feature,
    // then nothing — never the `default` key.
    let original_request =
        original_request_block(root, if feature.is_empty() { None } else { Some(&feature) });
    Ok(render(
        &template,
        &[
            ("worker", worker.unwrap_or("undefined")),
            ("cell_id", &cell_id),
            ("feature", &feature),
            ("cell_json", &cell_json),
            ("learned_context", &learned),
            ("expertise", expertise.unwrap_or("")),
            ("prior_rounds", &prior),
            ("worktree_root", worktree_root),
            ("control_root", control_root),
            ("original_request", &original_request),
        ],
    ))
}

/// provenance: dispatch-prepare.mjs appendPrepareRecord — fail-open, exactly
/// like Node's try/catch: a log failure never blocks the payload.
pub(crate) fn append_prepare_record(root: &Path, record: &Map<String, Value>) {
    let mut line = Map::new();
    line.insert(
        "ts".into(),
        Value::String(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
    );
    line.insert("source".into(), Value::String("prepare".into()));
    for (k, v) in record {
        line.insert(k.clone(), v.clone());
    }
    let _ = crate::fsutil::append_jsonl(
        &root.join(".bee").join("logs").join("dispatch.jsonl"),
        &Value::Object(line),
    );
}

/// A prepareDispatch outcome: a returned value (envelope OR typed refusal), or
/// a thrown Error (malformed CALL).
pub(crate) enum Prepared {
    Value(Value),
    Thrown(String),
}

/// herding-reach D1: dispatch prepare reports herding transport reachability.
/// Probes HERDR_ENV (must be '1') and HERDR_PANE_ID (non-empty). The herdr-only
/// spelling every pre-tmux caller and test uses. Production reaches the probe
/// through `herding_transport_probe_for` with the configured kind, so this name
/// survives for its callers' sake (tests, and any herdr-only caller later).
#[allow(dead_code)]
pub(crate) fn herding_transport_probe(
    env: &dyn Fn(&str) -> Option<String>,
) -> (bool, String, Option<String>) {
    herding_transport_probe_for(TransportKind::Herdr, env)
}

/// tmux-herding-transport D1: the same probe for a KNOWN transport. `kind`
/// comes from `herding.transport`, never from the environment — with the key
/// absent this is the herdr arm and the tmux variables are never read.
pub(crate) fn herding_transport_probe_for(
    kind: TransportKind,
    env: &dyn Fn(&str) -> Option<String>,
) -> (bool, String, Option<String>) {
    match kind {
        TransportKind::Herdr => {
            let pane_id = env("HERDR_PANE_ID").filter(|s| !s.is_empty());
            let herdr_env = env("HERDR_ENV");
            match herdr_env.as_deref() {
                Some("1") => match pane_id {
                    Some(pane) => (
                        true,
                        format!("HERDR_ENV=1 and HERDR_PANE_ID={pane} are set"),
                        Some(pane),
                    ),
                    None => (
                        false,
                        "HERDR_PANE_ID is not set — this session is not inside a herdr pane".into(),
                        None,
                    ),
                },
                Some("") | None => (
                    false,
                    "HERDR_ENV is not set — this session is not inside a herdr pane".into(),
                    pane_id,
                ),
                Some(_) => (
                    false,
                    "HERDR_ENV is not 1 — this session is not inside a herdr pane".into(),
                    pane_id,
                ),
            }
        }
        TransportKind::Tmux => {
            // tmux exports $TMUX to every pane and $TMUX_PANE as the pane id —
            // both non-empty is "inside a pane".
            let pane_id = env("TMUX_PANE").filter(|s| !s.is_empty());
            match env("TMUX").filter(|s| !s.is_empty()) {
                Some(_) => match pane_id {
                    Some(pane) => (
                        true,
                        format!("TMUX and TMUX_PANE={pane} are set"),
                        Some(pane),
                    ),
                    None => (
                        false,
                        "TMUX_PANE is not set — this session is not inside a tmux pane".into(),
                        None,
                    ),
                },
                None => (
                    false,
                    "TMUX is not set — this session is not inside a tmux pane".into(),
                    pane_id,
                ),
            }
        }
    }
}

/// provenance: dispatch-prepare.mjs prepareDispatch(root, {...}). Throws only
/// on a malformed CALL; every legitimate cli-shaped / unconfigured-advisor /
/// native-unavailable / claim-ownership resolution is a typed {ok:false}
/// RETURN, not an exception.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_dispatch(
    root: &Path,
    runtime: &str,
    kind: &str,
    cell_id: Option<&str>,
    worker: Option<&str>,
    force_ownership: bool,
    classification: Option<&str>,
    purpose: Option<&str>,
    record_it: bool,
    expertise: Option<&str>,
) -> D<Prepared> {
    prepare_dispatch_with_role(
        root,
        runtime,
        kind,
        None,
        cell_id,
        worker,
        force_ownership,
        classification,
        purpose,
        record_it,
        expertise,
    )
}

/// `prepare_dispatch` plus T012a's explicit `--role` override (store
/// 8ff6e79e).
///
/// An ARITY ADAPTER, never a second implementation: the ten-argument
/// spelling above is one call into this body with `role: None`, so the
/// no-role path is the same code every existing caller already runs and
/// cannot drift from it. It exists because "role absent" is the overwhelming
/// majority of call sites and threading a `None` through each of them would
/// buy nothing but churn in files this change has no business touching.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_dispatch_with_role(
    root: &Path,
    runtime: &str,
    kind: &str,
    role: Option<&str>,
    cell_id: Option<&str>,
    worker: Option<&str>,
    force_ownership: bool,
    classification: Option<&str>,
    purpose: Option<&str>,
    record_it: bool,
    expertise: Option<&str>,
) -> D<Prepared> {
    prepare_dispatch_with_brief(
        root,
        runtime,
        kind,
        role,
        cell_id,
        worker,
        force_ownership,
        classification,
        purpose,
        record_it,
        expertise,
        None,
    )
}

/// `prepare_dispatch_with_role` plus the carried LaneBrief (slp-blind-lanes
/// E1/E2).
///
/// The SAME arity-adapter shape the eleven-argument spelling above already
/// uses for `role`, and for the same reason: "no brief" is every existing
/// call site, so threading a `None` through each of them would buy nothing
/// but churn in files this change has no business touching. There is exactly
/// one implementation — this body — so the no-brief path cannot drift from
/// the brief path.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_dispatch_with_brief(
    root: &Path,
    runtime: &str,
    kind: &str,
    role: Option<&str>,
    cell_id: Option<&str>,
    worker: Option<&str>,
    force_ownership: bool,
    classification: Option<&str>,
    purpose: Option<&str>,
    record_it: bool,
    expertise: Option<&str>,
    // Already resolved and trimmed by `resolve_brief_file`, which refuses
    // every kind but `advisor` — so this is `None` for every other kind by
    // the time it reaches here.
    brief: Option<&str>,
) -> D<Prepared> {
    // The runtime/kind gates already fired in the probe (validate() owns those
    // bytes), so both are known-good here.
    debug_assert!(DISPATCH_RUNTIMES.contains(&runtime) && DISPATCH_KINDS.contains(&kind));

    let mut cell: Option<Value> = None;
    let mut ownership_override: Option<Value> = None;
    let mut resolved_worker: Option<String> = None;

    if kind == "cell" {
        let Some(cell_id) = cell_id else {
            return Ok(Prepared::Thrown(
                "dispatch prepare: --cell is required when --kind cell.".to_string(),
            ));
        };
        let Some(loaded) = read_cell(root, cell_id)? else {
            return Ok(Prepared::Thrown(format!(
                "dispatch prepare: cell \"{cell_id}\" not found."
            )));
        };
        let Some(worker) = worker.filter(|w| !js_trim(w).is_empty()) else {
            return Ok(Prepared::Thrown(
                "dispatch prepare: --worker is required when --kind cell.".to_string(),
            ));
        };
        let trimmed = js_trim(worker).to_string();
        let ownership = check_cell_claim_ownership(&loaded, &trimmed);
        if !ownership.ok && !force_ownership {
            let mut refusal = Map::new();
            refusal.insert("ok".into(), Value::Bool(false));
            refusal.insert("type".into(), Value::String("refused".into()));
            refusal.insert("reason".into(), Value::String("claim_ownership".into()));
            refusal.insert(
                "code".into(),
                ownership.code.map(|c| Value::String(c.into())).unwrap_or(Value::Null),
            );
            refusal.insert("status".into(), ownership.status);
            refusal.insert("owner".into(), ownership.owner);
            refusal.insert("fix".into(), Value::String(ownership.reason));
            return Ok(Prepared::Value(Value::Object(refusal)));
        }
        if force_ownership {
            let mut ov = Map::new();
            ov.insert("forced_by".into(), Value::String(trimmed.clone()));
            ov.insert("bypassed".into(), Value::Bool(!ownership.ok));
            ov.insert(
                "code".into(),
                if ownership.ok {
                    Value::Null
                } else {
                    ownership.code.map(|c| Value::String(c.into())).unwrap_or(Value::Null)
                },
            );
            ov.insert(
                "owner_bypassed".into(),
                if ownership.ok { Value::Null } else { ownership.owner.clone() },
            );
            ov.insert(
                "status_bypassed".into(),
                if ownership.ok { Value::Null } else { ownership.status.clone() },
            );
            ov.insert("transferred".into(), Value::Bool(false));
            ov.insert("note".into(), Value::String("advisory bypass only — cell.trace.worker (the actual claim owner) was NOT transferred; no correct transfer primitive exists on this ownership axis (see comment above).".into()));
            ownership_override = Some(Value::Object(ov));
        }
        // slp-contract S4 (D3, store 9c0104e0) — the SECOND door of the
        // contract-citation tripwire, running the SAME shared check the
        // claim body runs (`cells::contract_citation_refusal`), never a
        // second copy of the walk. Why both: a cell claimed BEFORE its
        // cited decision was superseded, or before a trigger keyed to it
        // reopened, slips a claim-only check entirely — the claim door
        // cannot see that window — and D3's letter names the dispatch.
        //
        // Placed AFTER the ownership refusal on the same ordering principle
        // the claim door uses: who owns the claim is a fact about the
        // caller and answers first; the citations are a fact about the
        // cell. `--force-ownership` deliberately does not reach here — that
        // flag is an ownership bypass and nothing else.
        if let Some(refusal) =
            crate::verbs::cells::contract_citation_refusal(root, cell_id, Some(&loaded))
        {
            let mut r = Map::new();
            r.insert("ok".into(), Value::Bool(false));
            r.insert("type".into(), Value::String("refused".into()));
            r.insert("reason".into(), Value::String(refusal.code.to_ascii_lowercase()));
            r.insert("code".into(), Value::String(refusal.code.to_string()));
            r.insert("cell".into(), Value::String(cell_id.to_string()));
            r.insert("fix".into(), Value::String(refusal.message));
            return Ok(Prepared::Value(Value::Object(r)));
        }
        resolved_worker = Some(trimmed);
        cell = Some(loaded);
    }

    // `find_granted_worktree_for_feature` (status_full/topology.rs) resolved
    // ONCE here feeds both the envelope (below) and the rendered prompt's
    // Location block (prompt_body_for) — one resolution, two destinations,
    // never a second lookup that could drift from the first. `root` here is
    // always the MAIN checkout: a granted worktree's own `dispatch prepare`
    // call already refused through the narrow door in run_dispatch_prepare
    // (Roots::Unsupported(GrantedWorktree)) before reaching this function.
    let worktree_location: Option<(String, String)> = cell
        .as_ref()
        .and_then(|c| match vget(c, "feature") {
            Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
        .and_then(|feature| {
            crate::verbs::status_full::find_granted_worktree_for_feature(root, &feature)
                .map(|(_id, worktree_root)| (worktree_root, root.to_string_lossy().into_owned()))
        });

    // D2 (06e49368): the default slot resolves ONCE, through an explicit arm.
    // A kind with no arm refuses here — it never falls through onto the
    // advisor slot. Unreachable today (both gates enum-check `kind` against
    // DISPATCH_KINDS); it exists so that adding a fifth kind without its
    // mapping is a typed refusal instead of silent work on the wrong model.
    let Some(default_slot) = slot_for_kind(kind) else {
        return Ok(Prepared::Value(unmapped_kind_refusal(kind)));
    };
    let models = read_models(root)?;
    // T012a: a `--role` naming a role nothing configures is REFUSED, never
    // resolved onto some other role's model. Same FIX shape as the
    // model-guard's marker refusal, and — the part that used to be a comment
    // rather than a fact — the SAME predicate, `known_role_named`, answer and
    // all. This door asked a plain `contains` while the guard asked
    // `eq_ignore_ascii_case`, so `--role Generation` was refused here while
    // `[bee-tier: Generation]` was admitted and resolved there: one typo, two
    // answers, through the two doors that share this predicate precisely so
    // that cannot happen.
    //
    // The answer carries the CONFIG's own spelling, and `role` is rebound to
    // it for the rest of this function. That is load-bearing rather than
    // cosmetic: every downstream read — the advisor branch's `role ==
    // Some(ADVISOR_ROLE)`, the resolved role list, the `[bee-tier: …]` marker
    // stamped on the payload, `economics.logical_tier` — has to be a key the
    // resolver can look up, exactly as `classify_marker` hands the guard one.
    // Admitting a spelling here without normalizing it would have moved the
    // two-answers defect one line down instead of closing it.
    //
    // lane-model-diversity D2 (store `23de5362`) — SEAT FALL-THROUGH, at the
    // ADVISOR door and nowhere else. A blind lane or a hat asks for its own
    // seat (`--role lane-3`, `--role hat-risks`); a host that has not
    // configured that seat still has to get an advisor rather than a refusal,
    // because the seats exist to let an operator OPT IN to model diversity and
    // an un-opted host must keep working exactly as it did.
    //
    // Three properties make this narrow rather than a hole in T012a:
    //
    //   * Only the eight `SEAT_ROLES` names (closed constant) fall through, so
    //     a typo'd `hat-risk` is still refused instead of quietly running on
    //     the advisor's model.
    //   * Only when the seat's own slot RESOLVES NOTHING — absent, null, or a
    //     shape `resolve_configured` reads as nothing. A configured seat
    //     resolves its own model and stamps its own marker (D4).
    //   * Only on the advisor kind. `default_slot` is `slot_for_kind(kind)`,
    //     and `advisor` is the one kind that maps to the advisor slot (see
    //     `slot_map_tests::only_the_advisor_arm_resolves_the_advisor_slot`), so
    //     `--kind gather --role lane-3` keeps T012a's refusal untouched.
    //
    // The fall-through REBINDS the canonical role to `advisor` rather than
    // walking `[seat, advisor]` through `resolve_role_named`: that walk's last
    // entry lands on the `Resolved::Budget` floor, which would hand an
    // unconfigured advisor the session model — verbatim the outcome
    // `4faf1de9` forbids. Rebinding lands on the advisor arm below, so the
    // advisor keeps its own one-name walk AND its `advisor_not_configured`
    // refusal for free.
    //
    // Nothing here reads a slot's `description`: that field is display-only
    // (`hooks/model_guard::role_slot_description`) and `normalize_models` has
    // already dropped it before `models` exists. D3's "a hat states its
    // purpose" is enforced as a `bee doctor` advisory over the RAW config
    // instead, so no resolution, guard, or dispatch decision depends on it.
    let fallen_through_seat: Option<&'static str> = match role {
        Some(declared) if default_slot == ADVISOR_ROLE => seat_role_named(declared)
            .filter(|_| !role_slot_resolves(&models, runtime, declared, kind)),
        _ => None,
    };
    let canonical_role: Option<String> = match role {
        Some(_) if fallen_through_seat.is_some() => Some(ADVISOR_ROLE.to_string()),
        Some(declared) => match known_role_named(&models, runtime, declared) {
            Some(canonical) => Some(canonical),
            None => {
                let roles = role_list(&models, runtime);
                let mut refusal = Map::new();
                refusal.insert("ok".into(), Value::Bool(false));
                refusal.insert("type".into(), Value::String("refused".into()));
                refusal.insert("reason".into(), Value::String("role_not_configured".into()));
                refusal.insert("role".into(), Value::String(declared.to_string()));
                refusal.insert("fix".into(), Value::String(format!(
                    "--role \"{declared}\" names a role nothing configures — models.{runtime} in .bee/config.json carries no \"{declared}\" entry, so the dispatch would select no model while the record asserted the caller had chosen one. FIX: name a configured role ({roles}), or configure this one — add \"{declared}\": \"<model>\" to models.{runtime} in .bee/config.json. Any role name you configure is legal; bee holds no fixed list."
                )));
                return Ok(Prepared::Value(Value::Object(refusal)));
            }
        },
        None => None,
    };
    // Unreachable as `None` when `role` was `Some` — that arm returned above.
    let role: Option<&str> = canonical_role.as_deref();
    // T012a (store 8ff6e79e): an explicit `--role` names the slot to resolve
    // OUTRIGHT, so neither the kind's default slot nor the cell's own
    // recorded value is consulted — the caller stating the job is the most
    // specific signal there is. Absent, this whole expression is the code
    // that stood here before, so every existing invocation keeps its bytes.
    //
    // What this buys is the reachability D12 asked for without D12's fifth
    // kind: `--kind gather --role extraction` resolves the extraction slot
    // and returns bee-extract, so every rendered bee agent is reachable
    // through the one door instead of one of them being rendered,
    // onboarded and documented while prepare could never return it.
    //
    // D3 (store 3c9d6262) — the WORK declares its job. A `--kind cell`
    // dispatch reads the cell's own `role` (required since mrs-8) and that
    // name HEADS the ordered list resolved below. Precedence, most specific
    // first: an explicit `--role` names the slot outright; else the cell's
    // declared role; else the kind's default slot.
    //
    // A role-LESS cell — every record written before mrs-8 — keeps the exact
    // path it had: its recorded `tier` still selects, still stamps
    // `tier_source: "cell"`, and still earns the `tier_not_configured`
    // refusal below. `tier` is not retired here (that is a later slice); this
    // changes which value a role-CARRYING cell resolves from, nothing else.
    // `from_role` is what tells the two apart downstream — the observable
    // `tier_source` vocabulary is unchanged at {flag, cell, default}.
    let (tier_token, tier_source, from_role) = match role {
        Some(role) => (role, "flag", false),
        None => {
            if kind == "cell" {
                match recorded_str(cell.as_ref(), "role") {
                    Some(r) => (r, "cell", true),
                    None => match recorded_str(cell.as_ref(), "tier") {
                        Some(t) => (t, "cell", false),
                        None => (default_slot, "default", false),
                    },
                }
            } else {
                (default_slot, "default", false)
            }
        }
    };
    // The advisor slot keeps its own resolver (never budget, never a tier
    // fallback). The condition reads the ROLE rather than the kind so an
    // explicit `--role advisor` reaches it too, and `--kind advisor --role
    // <other>` does not: with no `--role` it is exactly `kind == "advisor"`,
    // because `slot_for_kind("advisor")` is the advisor slot.
    //
    // Which name in the list actually WON. Set only on the cell-role path,
    // because every other path asks for one name (or `[review, generation]`,
    // whose fall-through predates this feature) and must keep stamping the
    // exact token it always stamped.
    let mut resolved_role: Option<&str> = None;
    // D5 (store `97ce5225`) — ESCALATION, read off the cell's flag rather
    // than off a tier value. `ceiling` used to arrive here as `tier_token`
    // and mean "run on the session model"; now the cell says so directly
    // (`crate::verbs::cells::cell_is_escalated` — the one predicate the 40%
    // ration reads too, so the cell that charges the budget and the cell
    // that spends it are the same set).
    //
    // Precedence is unchanged: an explicit `--role` names the slot OUTRIGHT,
    // so it still wins over the cell's own marking — including `--role
    // ceiling`, which stays the escalation door for a dispatch with no cell
    // behind it (a gather or a reviewer run on the session model). The flag
    // is read only on a `--kind cell` dispatch with no `--role`, which is
    // exactly the shape that used to read the cell's recorded `tier`.
    let escalated_cell = role.is_none()
        && kind == "cell"
        && cell.as_ref().map(crate::verbs::cells::cell_is_escalated).unwrap_or(false);
    // REVIEW P1-A — the escalation word is read off the CALLER or the FLAG,
    // never off a name the cell DECLARED.
    //
    // `tier_token` is one variable carrying three different provenances, and
    // `from_role` is the only thing that tells them apart. On the `from_role`
    // path it is the cell's own `role` string, and validation is deliberately
    // membership-blind there (D2, store `06e49368`: the role set is OPEN, so
    // any non-empty name is legal at add time). Testing `tier_token ==
    // ESCALATION_WORD` without this guard therefore let a cell escalate
    // ITSELF by declaring `role: "ceiling"` — `Resolved::Inherit`, the
    // session model, no `model` parameter — while `cell_is_escalated` read
    // false for the same record and every counter that enforces the 40%
    // ration (`escalation_share_after`, `role_mix`,
    // `ceiling_scarcity_warning`) missed it. Measured on a seven-cell feature
    // with six such cells: 86% escalated, and the refusal never fired.
    //
    // The two legitimate spellings both survive, because neither is
    // `from_role`: an explicit `--role ceiling` is `tier_source: "flag"`, and
    // a pre-migration record's `tier: "ceiling"` is already `escalated_cell`
    // through `cell_is_escalated`'s legacy read.
    //
    // Narrowing HERE rather than refusing the name at `verbs::cells::validate`
    // is the deliberate call: D5 took `ceiling` off the role axis entirely, so
    // a cell declaring it is just a role nothing configures, and this arm is
    // what makes `resolve_role_named`'s standing promise ("it warns and falls
    // through like any other") true of the shipped code. The alternative —
    // a reserved name validation rejects — would put one closed word back in
    // the open set D2 locks, and closes the hole only for cells authored
    // after it lands.
    let escalation_asked = tier_token == ESCALATION_WORD && !from_role;
    let (resolved, is_escalated) = if role == Some(ADVISOR_ROLE) || (role.is_none() && kind == "advisor")
    {
        let r = match resolve_advisor(&models, runtime) {
            Some(r) => r,
            None => {
                let mut refusal = Map::new();
                refusal.insert("ok".into(), Value::Bool(false));
                refusal.insert("reason".into(), Value::String("advisor_not_configured".into()));
                refusal.insert("fix".into(), Value::String(format!(
                    "set models.{runtime}.advisor in .bee/config.json to enable an advisor consult (resolveAdvisor never falls back to another tier)."
                )));
                return Ok(Prepared::Value(Value::Object(refusal)));
            }
        };
        (r, false)
    } else if escalated_cell || escalation_asked {
        // `Resolved::Inherit` IS "the session model": the payload built
        // below carries no `model` parameter and no herding command, on
        // either runtime. An escalated cell never walks the role list at
        // all — the escalation outranks whatever job name the cell declares,
        // which is what "run on the session model" has always meant.
        (Resolved::Inherit, true)
    } else {
        // A recorded TIER that nothing configures is still a refusal: a cost
        // word bee cannot resolve was never a fall-through, and pre-mrs-8
        // records keep that answer exactly. A recorded ROLE is the opposite
        // by construction — D2's open set says an unresolvable name YIELDS to
        // the next, and D3 bounds the cost of a bad guess at "ran on the
        // normal model", so `from_role` never reaches this door.
        if tier_source == "cell" && !from_role && !is_cell_tier_configured(&models, runtime, tier_token)
        {
            let mut refusal = Map::new();
            refusal.insert("ok".into(), Value::Bool(false));
            refusal.insert("type".into(), Value::String("refused".into()));
            refusal.insert("reason".into(), Value::String("tier_not_configured".into()));
            refusal.insert("tier".into(), Value::String(tier_token.to_string()));
            refusal.insert("fix".into(), Value::String(format!(
                "set models.{runtime}.{tier_token} in .bee/config.json to configure this tier."
            )));
            return Ok(Prepared::Value(Value::Object(refusal)));
        }
        // 561e1bda: the cell's own role heads the cell-execution list; every
        // other caller asks the tier-shaped question it always asked. Both
        // walk the ONE resolver, and the walk hands back the name that won.
        let roles =
            if from_role { cell_role_list(tier_token) } else { tier_role_list(tier_token) };
        let (winner, r) = resolve_role_named(&models, &roles, runtime, kind);
        if from_role {
            resolved_role = winner;
        }
        if let Resolved::Refused { slot } = &r {
            // pi-support D5: on pi a cli slot is refused for its RUNTIME, not
            // for its purpose — `{kind:"cli"}` has no herding pane behind it
            // on any kind, so the refusal that names the herding requirement
            // is the useful one for a gather as much as for a cell.
            if runtime == "pi" {
                return Ok(Prepared::Value(pi_requires_herding_refusal(slot, &r, false)));
            }
            let mut refusal = Map::new();
            refusal.insert("ok".into(), Value::Bool(false));
            refusal.insert("type".into(), Value::String("refused".into()));
            refusal.insert("reason".into(), Value::String("cli_tier_gather_only".into()));
            refusal.insert("slot".into(), Value::String(slot.clone()));
            refusal.insert("fix".into(), Value::String(CLI_REFUSAL_FIX.into()));
            return Ok(Prepared::Value(Value::Object(refusal)));
        }
        (r, false)
    };

    // THE NAME THE DISPATCH TRAVELS UNDER — the role that resolved, which on
    // every path but a fallen-through cell role is the token asked for.
    //
    // It matters because `hooks/model_guard.rs` reads the `[bee-tier: …]`
    // marker back and DENIES a name nothing configures: a cell declaring
    // `role: "code"` on a host that carries no `code` key runs on that host's
    // `generation` model, and a marker still saying "code" would have the
    // guard refuse the dispatch bee itself just prepared. The marker names
    // the model channel; `tier_source` still says who chose it, and the cell
    // record still carries the job the work declared.
    //
    // An ESCALATED dispatch travels under the escalation word, never under
    // the cell's job role. The payload's marker says `[bee-tier: ceiling]`
    // and `economics.logical_tier` has to say the same thing, or the guard's
    // audit line resolves one name while the marker carries another.
    let marker_role: &str =
        if is_escalated { ESCALATION_WORD } else { resolved_role.unwrap_or(tier_token) };

    // pi-support D5 — THE HERDING-ONLY DOOR, at full width. Placed here, after
    // the slot resolved and before a single byte of prompt is rendered:
    // resolution is what the refusal reports, and a refused dispatch has no
    // business paying for a prompt build. Every non-herding exit is covered by
    // construction — `Model`, `Native`, `Cli`, `Budget` and the escalation
    // path's `Inherit` all arrive here as `resolved`, so a transport arm added
    // to the payload match below cannot open a new hole in this rule.
    if runtime == "pi" && !matches!(resolved, Resolved::Herding { .. }) {
        return Ok(Prepared::Value(pi_requires_herding_refusal(
            marker_role,
            &resolved,
            is_escalated,
        )));
    }

    let prompt_body = match prompt_body_for(
        root,
        kind,
        cell.as_ref(),
        resolved_worker.as_deref(),
        worktree_location.as_ref().map(|(w, c)| (w.as_str(), c.as_str())),
        expertise,
        brief,
    )? {
        Ok(body) => body,
        Err(msg) => return Ok(Prepared::Thrown(msg)),
    };
    let requested_model = match &resolved {
        Resolved::Model { model, .. } => Some(model.clone()),
        _ => None,
    };
    // The generation tier carries TWO rendered agents — bee-build executes
    // a cell (reserves, writes, commits, caps), bee-gather reads and
    // reports (never writes, per .claude/agents/bee-gather.md). tier alone
    // cannot tell them apart (guard.rs's pinned_agent_type stays a tier-only
    // lookup, mirrored by hooks/model_guard.rs PINNED_AGENT_TYPE); `kind`
    // is the one signal that can, and only a --kind cell dispatch is a cell
    // execution (dp-2).
    let pinned_type =
        if kind == "cell" { "bee-build" } else { pinned_agent_type(marker_role) };

    // The dispatch SUBJECT — computed ONCE, here, before the transport match,
    // so every branch below (native override, native fallback, codex
    // spawn_agent, claude Agent) reads the identical value. It used to be
    // computed only inside the claude Agent arm, which is exactly how the
    // codex `task_name` branch was missed for a month (dispatch-label-
    // chokepoint plan.md — the fourth attempt at this one rule).
    //
    // `kind == "cell"`: subject is the cell's own "<id>: <title>" — the cell
    // is already loaded here for the prompt, and its title is what the row
    // is FOR. A whitespace-only or absent title is no title: the bare kind,
    // never a dangling "id: ".
    //
    // Every other kind: subject is "<kind>: <purpose>" when the caller passes
    // `--purpose`, and the bare kind otherwise — today's exact bytes when no
    // `--purpose` is given, so this stays back-compatible by construction.
    // Their purpose is the caller's to state; prepare never invents one.
    let subject = if kind == "cell" {
        cell.as_ref()
            .map(|c| (tpl(vget(c, "id")), one_line(vget(c, "title"), DESCRIPTION_TITLE_MAX)))
            .filter(|(_, title)| !title.trim().is_empty())
            .map(|(id, title)| format!("{id}: {title}"))
            .unwrap_or_else(|| kind.to_string())
    } else {
        match purpose.map(js_trim).filter(|p| !p.is_empty()) {
            Some(p) => format!(
                "{kind}: {}",
                one_line(Some(&Value::String(p.to_string())), DESCRIPTION_TITLE_MAX)
            ),
            None => kind.to_string(),
        }
    };

    let mut tool = String::new();
    let mut payload = Map::new();
    let mut channel = String::new();
    let mut refusal: Option<Value> = None;
    let mut native_confirmed = false;
    // envelopeExtra, kept as its two possible keys so the spread order below
    // stays byte-identical.
    let mut extra_transport: Option<&str> = None;
    let mut extra_fallback_reason: Option<&str> = None;

    if is_escalated {
        if runtime == "codex" {
            tool = "spawn_agent".into();
            payload.insert(
                "task_name".into(),
                Value::String(one_line(Some(&Value::String(subject.clone())), TASK_NAME_MAX)),
            );
            payload.insert(
                "message".into(),
                Value::String(format!("[bee-tier: {ESCALATION_WORD}]\n{prompt_body}")),
            );
            payload.insert("fork_turns".into(), Value::String("none".into()));
            channel = "session-model".into();
        } else {
            tool = "Agent".into();
            payload.insert("subagent_type".into(), Value::String(pinned_type.into()));
            payload.insert(
                "prompt".into(),
                Value::String(format!("[bee-tier: {ESCALATION_WORD}]\n{prompt_body}")),
            );
            payload.insert(
                "description".into(),
                Value::String(format!("{subject} ({ESCALATION_WORD})")),
            );
            channel = "session-model".into();
        }
    } else {
        match &resolved {
            Resolved::Native { model, effort, fallback, agent_type, .. } => {
                native_confirmed = classification == Some(NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE);
                if native_confirmed {
                    tool = "spawn_agent".into();
                    payload.insert(
                        "agent_type".into(),
                        Value::String(if agent_type.is_empty() {
                            "worker".to_string()
                        } else {
                            agent_type.clone()
                        }),
                    );
                    payload.insert(
                        "message".into(),
                        Value::String(format!("[bee-tier: {marker_role}]\n{prompt_body}")),
                    );
                    payload.insert("model".into(), Value::String(model.clone()));
                    payload.insert("fork_turns".into(), Value::String("none".into()));
                    if let Some(effort) = effort {
                        payload.insert("reasoning_effort".into(), Value::String(effort.clone()));
                    }
                    channel = "codex-native".into();
                    extra_transport = Some("native-override");
                } else if let Some(command) = fallback.as_ref().filter(|c| !c.is_empty()) {
                    // cli-exec: NO label field. This payload is `{command, stdin}`
                    // only — a recorded limit (dispatch-label-chokepoint plan.md
                    // "What this does not do"), not an oversight: no field exists
                    // on an external CLI-executor call to carry a subject.
                    tool = "Bash".into();
                    payload.insert("command".into(), Value::String(command.clone()));
                    payload.insert("stdin".into(), Value::String(prompt_body.clone()));
                    channel = "cli-exec".into();
                    extra_fallback_reason = Some("native_unavailable");
                } else {
                    let mut r = Map::new();
                    r.insert("ok".into(), Value::Bool(false));
                    r.insert("type".into(), Value::String("refused".into()));
                    r.insert("reason".into(), Value::String("native_unavailable".into()));
                    r.insert(
                        "detail".into(),
                        Value::String(
                            classification
                                .filter(|c| !c.is_empty())
                                .unwrap_or(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY)
                                .to_string(),
                        ),
                    );
                    refusal = Some(Value::Object(r));
                }
            }
            Resolved::Cli { command } => {
                // cli-exec: NO label field — see the identical comment on the
                // native-fallback Bash arm above; this is the same recorded
                // limit, not an oversight.
                tool = "Bash".into();
                payload.insert("command".into(), Value::String(command.clone()));
                payload.insert("stdin".into(), Value::String(prompt_body.clone()));
                channel = "cli-exec".into();
            }
            Resolved::Herding { agent, fallback } => {
                // herding-tier D4: mirrors the cli-exec Bash arm above
                // byte-for-byte in shape — argv cannot carry a long brief, so
                // the prompt travels on stdin, and this arm fires for EVERY
                // runtime (no codex/claude split): a herding pane is a Bash
                // subprocess call, never a native spawn_agent. D6: the payload
                // carries the brief only, never a bee verb for the worker — ALL
                // bee bookkeeping (claim, cap, close) stays the orchestrator's
                // after it reads the herding result (herding-executor D4).
                // herd-registry D2: a slot naming `agent:"<name>"` appends
                // `--agent "<name>"` after --cwd (quoted, same as --cwd); a slot
                // without an agent leaves the command byte-identical to before.
                tool = "Bash".into();
                let mut command = ".bee/bin/bee herding run --task-file - --json".to_string();
                if let Some((worktree_root, _control_root)) = &worktree_location {
                    if !worktree_root.is_empty() {
                        command.push_str(" --cwd \"");
                        command.push_str(worktree_root);
                        command.push('"');
                    }
                }
                if let Some(agent) = agent {
                    command.push_str(" --agent \"");
                    command.push_str(agent);
                    command.push('"');
                }
                payload.insert("command".into(), Value::String(command));
                payload.insert("stdin".into(), Value::String(prompt_body.clone()));
                // herding-reach D1: dispatch prepare reports herding transport
                // reachability. Probes the caller's environment and writes
                // transport_ready and transport_reason into the payload.
                // tmux-herding-transport D1: WHICH variables get probed comes
                // from `herding.transport` in the main checkout's config
                // (`root` here is always the MAIN checkout — see
                // `worktree_location` above), never from sniffing the env. A
                // bad value is reported as not-ready with the refusal text,
                // never a panic.
                let (transport_ready, transport_reason, _) = match transport_kind_at(root) {
                    Ok(kind) => herding_transport_probe_for(kind, &|k| std::env::var(k).ok()),
                    Err(reason) => (false, reason, None),
                };
                payload.insert("transport_ready".into(), Value::Bool(transport_ready));
                payload.insert("transport_reason".into(), Value::String(transport_reason));
                // herding-review-slots D3: `fallback:"default"` on the slot
                // names the runtime's own default model for this slot (the
                // same table a gather purpose used to fall back to silently,
                // pre-D1-widening) so the orchestrator can re-dispatch through
                // the Agent path on a failed herding run. Only CONFIGURABLE_SLOTS
                // members have a default-model table entry at all (advisor does
                // not — resolveAdvisor "NEVER a tier fallback" still holds); no
                // resolvable default leaves the payload byte-identical to a
                // slot with no `fallback` field.
                if fallback.is_some() {
                    if let Some(model) = CONFIGURABLE_SLOTS
                        .contains(&marker_role)
                        .then(|| default_models(runtime).get(marker_role).cloned())
                        .flatten()
                        .and_then(|v| match v {
                            Value::String(s) => Some(s),
                            _ => None,
                        })
                    {
                        let mut fb = Map::new();
                        fb.insert("model".into(), Value::String(model));
                        fb.insert(
                            "fallback_when".into(),
                            Value::String("transport_ready is false".into()),
                        );
                        payload.insert("fallback".into(), Value::Object(fb));
                    }
                }
                channel = "herding-exec".into();
            }
            _ if runtime == "codex" => {
                tool = "spawn_agent".into();
                // Carries the SAME subject as the claude Agent branch below,
                // instead of the bare cell id (or "bee-{kind}") it used to —
                // this arm is exactly the one the codex gap hid in (plan.md).
                // codex's `task_name` is a plain required string on the
                // live-probed 0.145.0 schema (see TASK_NAME_MAX); one-lined and
                // capped so a long subject cannot read like a paragraph.
                payload.insert(
                    "task_name".into(),
                    Value::String(one_line(Some(&Value::String(subject.clone())), TASK_NAME_MAX)),
                );
                payload.insert(
                    "message".into(),
                    Value::String(format!("[bee-tier: {marker_role}]\n{prompt_body}")),
                );
                payload.insert("fork_turns".into(), Value::String("none".into()));
                channel = "codex-native".into();
            }
            _ => {
                tool = "Agent".into();
                payload.insert("subagent_type".into(), Value::String(pinned_type.into()));
                payload.insert(
                    "prompt".into(),
                    Value::String(format!("[bee-tier: {marker_role}]\n{prompt_body}")),
                );
                // `requestedModel || tierToken`
                let model_tag = requested_model
                    .clone()
                    .filter(|m| !m.is_empty())
                    .unwrap_or_else(|| marker_role.to_string());
                // A description that is only a model name is a red flag
                // (work-visibility D2) — and it was the one prepare emitted, so
                // orchestrators wrote their own bare `Execute <id>` instead and
                // the agent list read as a column of ids. `subject` is computed
                // once, above, and every transport branch (including this one)
                // reads it.
                payload.insert("description".into(), Value::String(format!("{subject} ({model_tag})")));
                if let Resolved::Model { model, .. } = &resolved {
                    payload.insert("model".into(), Value::String(model.clone()));
                }
                channel = "claude-agent".into();
            }
        }
    }

    if let Some(refusal) = refusal {
        return Ok(Prepared::Value(refusal));
    }

    // model-role-split D10/D11 (store 50808d48), scoped for this codebase by
    // 51341f84: the runtime fallback chain is a contract bee PUBLISHES on the
    // payload, never a loop bee runs. prepare builds a payload and returns —
    // the orchestrator or the worker executes it — so bee never sees the 429,
    // the 5xx or the stall a chain step answers. What travels here is the
    // chain that applies to THIS dispatch plus D11's gate in both directions,
    // beside the model, in the same shape and the same neighbourhood as the
    // herding slot's `fallback` + `fallback_when` above.
    //
    // Gated on the payload ACTUALLY carrying a model, which is the literal
    // reading of "beside the model" and the honest one: an escalated dispatch
    // (session model, no `model` parameter), a cli-exec command and a herding
    // pane carry no model to fall FROM, so a list of model selectors would be
    // advice none of them could take. `marker_role` is the role the dispatch
    // travels under — the same name the `[bee-tier: …]` marker and
    // `economics.logical_tier` carry, so a role-keyed chain is keyed on the
    // channel that can actually fail rather than on a name that fell through.
    //
    // EXPLICIT-ONLY: with no `retry.fallbackChains` configured,
    // `read_fallback_chains` hands back an empty map, `resolve_fallback_chain`
    // answers None, and nothing at all is inserted — every payload stays
    // byte-identical to before this block existed, the advisor included.
    if let Some(model) = payload.get("model").and_then(Value::as_str).map(str::to_string) {
        if !model.is_empty() {
            let chains = read_fallback_chains(root);
            if let Some((key, steps)) = resolve_fallback_chain(&chains, marker_role, &model) {
                payload.insert("fallback_chain".into(), fallback_chain_payload(&key, &steps));
            }
        }
    }

    let param_model = match (&channel[..], &resolved) {
        ("claude-agent", Resolved::Model { model, .. }) => Some(model.clone()),
        _ => None,
    };
    // `logical_tier` audits the model channel this dispatch actually took, so
    // it reads the resolved role for the same reason the marker does — the
    // guard's own audit line resolves that name back and the two must agree.
    let mut economics = derive_economics(
        &channel,
        marker_role,
        param_model.as_deref(),
        &resolved,
        native_confirmed,
    );
    economics.insert("tier_source".into(), Value::String(tier_source.to_string()));
    // lane-model-diversity D2 — the seat that was ASKED FOR, beside the role
    // that RESOLVED. `logical_tier` and the `[bee-tier: …]` marker both name
    // `advisor` on a fallen-through seat (D4: the marker names the resolved
    // role), so without this key the record cannot tell three lanes that all
    // fell through apart from three plain advisor consults — and an operator
    // reading the log to find out whether their seats are wired would have
    // nothing to read. Present ONLY when a seat actually fell through: every
    // other dispatch's economics block, envelope and log row stay
    // byte-identical to before this key existed.
    //
    // It rides `economics` because that map is inserted into BOTH the returned
    // envelope and the appended dispatch-log record, one write reaching both
    // destinations rather than two that could drift.
    if let Some(seat) = fallen_through_seat {
        economics.insert("requested_role".into(), Value::String(seat.to_string()));
    }

    let dispatch_id = pseudo_uuid_v4();

    // slp-blind-lanes E2 (D2b). Byte-identity of the LaneBrief across 2–3
    // parallel lanes has to be CHECKABLE, and the dispatch record is the one
    // artifact prepare already returns AND already logs — so the digest goes
    // there rather than into a new store (decision f0f21142). Reuses the ONE
    // hasher this codebase has (`verbs/reservations/leases.rs`), never a
    // second one.
    //
    // Over the CARRIED brief, not the file: two lanes agreeing on this digest
    // agree on the bytes they were actually handed. Absent entirely when no
    // brief travelled, so every existing dispatch record — and every existing
    // envelope — stays byte-identical to before this key existed.
    let brief_sha256 = brief.map(crate::verbs::reservations::sha256_hex);

    let mut record = Map::new();
    record.insert("dispatch_id".into(), Value::String(dispatch_id.clone()));
    if let Some(digest) = &brief_sha256 {
        record.insert("brief_sha256".into(), Value::String(digest.clone()));
    }
    record.insert("kind".into(), Value::String(kind.to_string()));
    record.insert(
        "cell".into(),
        match &cell {
            Some(c) => vget(c, "id").cloned().unwrap_or(Value::Null),
            None => Value::Null,
        },
    );
    record.insert("runtime".into(), Value::String(runtime.to_string()));
    let classification_value = match classification {
        Some(c) if !c.is_empty() => Value::String(c.to_string()),
        _ => Value::Null,
    };
    if let Some(reason) = extra_fallback_reason {
        record.insert("native_fallback_reason".into(), Value::String(reason.into()));
        record.insert("native_classification".into(), classification_value.clone());
    }
    if extra_transport.is_some() {
        record.insert("native_classification".into(), classification_value);
    }
    if let Some(ov) = &ownership_override {
        record.insert("ownership_override".into(), ov.clone());
    }
    for (k, v) in &economics {
        record.insert(k.clone(), v.clone());
    }
    // `record_it` is false on the PROBE pass: run() builds the whole envelope
    // once to discover delegate-shaped inputs before a byte is produced, then
    // rebuilds it for real. Gating the append here means a command that ends
    // up delegating never leaves a prepare line behind, and one that is served
    // leaves exactly one — Node's count.
    if record_it {
        append_prepare_record(root, &record);
    }

    let mut envelope = Map::new();
    envelope.insert("tool".into(), Value::String(tool));
    envelope.insert("payload".into(), Value::Object(payload));
    envelope.insert("dispatch_id".into(), Value::String(dispatch_id));
    // The same digest the logged record carries, on the artifact the CALLER
    // holds: an orchestrator firing three lanes reads it straight off each
    // returned envelope instead of re-reading the log to find out what it
    // just sent. Present only when a brief travelled.
    if let Some(digest) = brief_sha256 {
        envelope.insert("brief_sha256".into(), Value::String(digest));
    }
    envelope.insert("economics".into(), Value::Object(economics));
    // Present only when the cell's feature has a granted worktree — see
    // `worktree_location` above. A feature with no worktree split (or a
    // non-cell dispatch) omits both keys, so the envelope stays
    // byte-identical to before this Location block existed.
    if let Some((worktree_root, control_root)) = &worktree_location {
        envelope.insert("worktree_root".into(), Value::String(worktree_root.clone()));
        envelope.insert("control_root".into(), Value::String(control_root.clone()));
    }
    if let Some(t) = extra_transport {
        envelope.insert("transport".into(), Value::String(t.into()));
    }
    if let Some(r) = extra_fallback_reason {
        envelope.insert("fallback_reason".into(), Value::String(r.into()));
    }
    if let Some(ov) = ownership_override {
        envelope.insert("ownership_override".into(), ov);
    }
    Ok(Prepared::Value(Value::Object(envelope)))
}

/// provenance: bee.mjs readNativeTransportClassification — the delegating
/// slice. An absent / unreadable / unparseable probe record and a
/// schema-mismatched one both short-circuit to native_budget_only with NO
/// subprocess; anything past that point shells out to codex-cli, so it
/// delegates.
pub(crate) const NATIVE_TRANSPORT_PROBE_SCHEMA: &str = "native-transport-probe/1";

pub(crate) fn native_transport_classification(root: &Path) -> D<&'static str> {
    let file = root.join(".bee").join("native-transport-probe.json");
    // doctorSafeReadJson: unreadable OR unparseable both yield null.
    let record = match std::fs::read(&file) {
        Err(_) => None,
        Ok(bytes) => serde_json::from_str::<Value>(&String::from_utf8_lossy(&bytes)).ok(),
    };
    match record {
        None => Ok(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY),
        Some(r) if !matches!(vget(&r, "schema"), Some(Value::String(s)) if s == NATIVE_TRANSPORT_PROBE_SCHEMA) => {
            Ok(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY)
        }
        // A live probe record: doctorRepoIdentity + `codex --version` +
        // `codex features list` + the config-scope hash all have to run.
        Some(_) => Err(Delegate),
    }
}

/// bee.mjs's `claimAndReserveForDispatch` — the claim, then one reserve per
/// declared file IN DECLARATION ORDER, stopping at the first conflict and
/// unwinding everything this call created, in REVERSE (reservations first,
/// then the claim), so the refusal can truthfully say the repo is back in its
/// pre-call state.
///
/// Both doors are the SHARED ones (`cells::claim_cell_from_flags`,
/// `reservations::reserve_path_atomic`), never a second copy — re-deriving
/// them would fork the store-mutation logic C1 exists to protect.
///
/// The outer `Err` carries `(Err2, claim_taken)` — dispatch review delta
/// (hpf-3): the claim door's own exotic-shape delegate fires BEFORE any
/// claim mutation (`handlers_write.rs:904`/`915` — a truthy non-object cell
/// or a truthy non-array `deps`, checked ahead of `claimCellCrossSession`),
/// so `claim_taken` is `false` there always. Every OTHER `?`-propagated
/// unproven shape below runs only after the claim door already returned
/// `Ok`, so `claim_taken` is `true` there. A caller deciding whether to
/// force-unclaim on refusal MUST read this flag first: forcing through an
/// ownership guard over a claim this call never took would write straight
/// through whatever the cell already held — open, or another live agent's
/// claim.
pub(crate) fn claim_and_reserve_for_dispatch(
    root: &Path,
    topo: Option<(&Path, &str)>,
    cell_id: &str,
    worker: &str,
    session_flag: Option<&str>,
) -> Result<Result<(Value, Vec<Value>, bool, Option<String>), String>, (Err2, bool)> {
    use crate::verbs::cells;
    // ttl/isolate are structurally absent: bee.mjs builds `{id, worker}` plus
    // `session-id` only when one was passed.
    let door = match cells::claim_cell_from_flags(root, cell_id, worker, session_flag, None) {
        Ok(d) => d,
        Err(cells::Fail::Delegate) => return Err((Err2::Ex, false)),
        Err(cells::Fail::Thrown(m)) => return Ok(Err(m)),
    };
    // The claim STANDS from here on — wrap the remainder so any further
    // unproven-shape error reports `claim_taken: true`, never folded into
    // the pre-claim `false` case above.
    (move || -> R2<Result<(Value, Vec<Value>, bool, Option<String>), String>> {
        let cell = door.cell;
        let claimed_id = match cell.get("id") {
            Some(Value::String(s)) => s.clone(),
            other => jsjson::js_to_string(other.unwrap_or(&Value::Null)),
        };
        // `Array.isArray(cell.files) ? cell.files.filter(f => typeof f === 'string' && f) : []`
        let files: Vec<String> = match cell.get("files") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|f| match f {
                    Value::String(s) if !s.is_empty() => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };

        let mut reserved: Vec<Value> = Vec::new();
        for file_path in &files {
            let section =
                reserve_path_atomic(topo, root.to_str().ok_or(Err2::Ex)?, worker, &claimed_id, file_path)?;
            let conflict_lines: Vec<String> = match section {
                ReserveOutcome::Reserved(reservation) => {
                    // The NORMALIZED path off the lease record, not files[i].
                    reserved.push(reservation.get("path").cloned().unwrap_or(Value::Null));
                    continue;
                }
                ReserveOutcome::Thrown(m) => return Ok(Err(m)),
                ReserveOutcome::ForeignHold(hold) => {
                    let or_unknown = |k: &str| match hold.get(k) {
                        Some(v) if truthy(v) => jsjson::js_to_string(v),
                        _ => "unknown".to_string(),
                    };
                    vec![format!(
                        "- checkout \"{}\" holds \"{}\" (cross-worktree hold, feature {}, cell {})",
                        hold.get("holder").map_or("undefined".into(), jsjson::js_to_string),
                        hold.get("path").map_or("undefined".into(), jsjson::js_to_string),
                        or_unknown("feature"),
                        or_unknown("cell"),
                    )]
                }
                ReserveOutcome::Conflicts(conflicts) => conflicts
                    .iter()
                    .map(|c| {
                        format!(
                            "- {} holds \"{}\" (cell {})",
                            c.get("agent").map_or("undefined".into(), jsjson::js_to_string),
                            c.get("path").map_or("undefined".into(), jsjson::js_to_string),
                            c.get("cell").map_or("undefined".into(), jsjson::js_to_string),
                        )
                    })
                    .collect(),
            };

            // Unwind, in reverse. Both rungs read stores this same call has
            // already probed (reserve_prechecks, and the claim door's own
            // prescans), so the Exotic arm below is unreachable in practice —
            // recorded as this branch's one accepted residual.
            let mut unwind_note = "the claim was unwound and state restored as found".to_string();
            let unwound = (|| -> R2<Result<(), String>> {
                if !reserved.is_empty() {
                    if let Out::Thrown(m) = release_reservations_for_agent(
                        topo,
                        root.to_str().ok_or(Err2::Ex)?,
                        worker,
                        Some(&claimed_id),
                    )? {
                        return Ok(Err(m));
                    }
                }
                match cells::unclaim_cell(root, &claimed_id, door.session_id.as_deref(), false) {
                    Ok(_) => Ok(Ok(())),
                    Err(cells::Fail::Delegate) => Err(Err2::Ex),
                    Err(cells::Fail::Thrown(m)) => Ok(Err(m)),
                }
            })()?;
            if let Err(message) = unwound {
                unwind_note = format!(
                    "UNWIND FAILED ({message}) — restore by hand: bee reservations release --agent {worker} --cell {claimed_id} --json ; bee cells unclaim --id {claimed_id} --json"
                );
            }
            let mut lines = vec![format!(
                "dispatch prepare --claim: reservation conflict on cell \"{claimed_id}\" — nothing dispatched; {unwind_note}:"
            )];
            lines.extend(conflict_lines);
            return Ok(Err(lines.join("\n")));
        }

        // dp-r1: the claim (and every reservation) stands from here on — register
        // the claiming worker against the cell it now owns, THE SAME record shape
        // `bee state worker add --nickname <w> --cell <id> --tier <t> --status
        // running` writes (state_group::register_worker_for_cell reuses that
        // door's own write path). A registration failure never unwinds the claim
        // — it is real state the caller already holds — it only travels back as
        // `(worker_registered: false, Some(registration_error))` for the payload
        // to name loudly.
        let tier = match cell.get("tier") {
            Some(Value::String(t)) if !t.is_empty() => Some(t.clone()),
            _ => None,
        };
        let (worker_registered, registration_error) =
            match crate::verbs::state_group::register_worker_for_cell(root, worker, &claimed_id, tier.as_deref()) {
                Ok(()) => (true, None),
                Err(message) => (false, Some(message)),
            };
        Ok(Ok((cell, reserved, worker_registered, registration_error)))
    })()
    .map_err(|e| (e, true))
}

pub(crate) fn run_dispatch_prepare(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(
        &flags,
        &[
            "runtime",
            "kind",
            "cell",
            "worker",
            "force-ownership",
            "claim",
            "session-id",
            "purpose",
            "expertise",
            "role",
            "brief-file",
        ],
    ) {
        return None;
    }
    // `flags.claim === true`: a bare `--claim` or `--claim=true`. Any other
    // spelling is validate()'s to answer.
    let claim = match flags.get("claim") {
        None => false,
        Some(FlagV::Present) => true,
        Some(FlagV::S(s)) if s == "true" => true,
        Some(FlagV::S(s)) if s == "false" => false,
        Some(FlagV::S(_)) => return None,
    };
    // `--session-id` is documented as ignored WITHOUT --claim; a caller that
    // passes it anyway is an unproven shape here. With --claim it is the
    // claim door's own `sessionFlag`.
    let session_flag: Option<String> = match (claim, flags.get("session-id")) {
        (_, None) => None,
        (false, Some(_)) => return None,
        (true, Some(FlagV::S(s))) => Some(s.clone()),
        (true, Some(FlagV::Present)) => return None, // String(true) — unproven
    };
    // validate(): boolean-typed --force-ownership given as =value.
    match flags.get("force-ownership") {
        None | Some(FlagV::Present) => {}
        Some(FlagV::S(s)) if s == "true" || s == "false" => {}
        Some(FlagV::S(_)) => return None,
    }
    // validate(): runtime/kind required + enum-checked.
    let runtime = flags.req_str("runtime")?.to_string();
    let kind = flags.req_str("kind")?.to_string();
    if !DISPATCH_RUNTIMES.contains(&runtime.as_str()) || !DISPATCH_KINDS.contains(&kind.as_str()) {
        return None; // validate()'s enum message
    }
    // `typeof flags.cell === 'string' && flags.cell ? flags.cell : null`
    let cell_id = flags.truthy_str("cell").map(str::to_string);
    let worker = flags.truthy_str("worker").map(str::to_string);
    // Only read for a non-cell kind (prepare_dispatch itself ignores it for
    // `--kind cell`); `one_line` inside prepare_dispatch collapses it to a
    // single line, so no separate "<one line>" validation is needed here.
    let purpose = flags.truthy_str("purpose").map(str::to_string);
    // T012a: the role the caller declares for this dispatch. `truthy_str`
    // reads an empty or boolean-shaped `--role` as absent, which is the right
    // answer — a role that names nothing is no role at all, and the whole
    // point of the flag is that it is EXPLICIT. A name nothing configures is
    // refused inside `prepare_dispatch_with_role`, beside the FIX that lists
    // the roles this runtime can resolve.
    let role = flags.truthy_str("role").map(|r| js_trim(r).to_string()).filter(|r| !r.is_empty());
    let force_ownership = matches!(flags.get("force-ownership"), Some(FlagV::Present));

    let expertise_flag = match flags.get("expertise") {
        Some(FlagV::S(s)) => Some(s.as_str()),
        Some(FlagV::Present) => Some(""),
        None => None,
    };
    let expertise_entries = match expertise_flag {
        Some(raw) => match parse_expertise(raw) {
            Ok(entries) => Ok(entries),
            Err(e) => Err(e),
        },
        None => Ok(Vec::new()),
    };
    // slp-blind-lanes E1: the LaneBrief path. `FlagV::Present` (a bare
    // `--brief-file` with no value) is deliberately NOT a delegation like the
    // boolean-shaped flags above — the Node runtime it would delegate to was
    // deleted at the R6 cutover, so it resolves to the empty path and earns
    // `brief_file_unreadable`, a refusal that names its own remedy. An
    // argument shape this door cannot honour is refused, never shrugged at.
    let brief_flag: Option<String> = match flags.get("brief-file") {
        None => None,
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(FlagV::Present) => Some(String::new()),
    };
    // The rendered reading list, resolved BEFORE the brief: the two may not
    // travel together (`expertise_beside_brief_refusal`), and that refusal
    // has to fire before the brief file is read.
    let (expertise_arg_error, expertise_block) = match expertise_entries {
        Ok(entries) => {
            let block = if entries.is_empty() {
                None
            } else {
                let lines: Vec<String> = entries
                    .iter()
                    .map(|e| format!("- {} — {}. Read it to {}.", e.path, e.purpose, e.read_to))
                    .collect();
                Some(lines.join("\n"))
            };
            (None, block)
        }
        Err(err) => (Some(err), None),
    };
    // Resolved here, before anything is built, so a refused brief never
    // reaches the payload build at all. A typed refusal, never a Thrown: the
    // caller of this door is an orchestrator firing 2–3 lanes, and it reads
    // `{ok:false, reason}` off every other refusal this command returns.
    //
    // The LEANING GUARD runs here too, on the resolved brief bytes and NOTHING
    // else — never `--purpose`, never `--expertise`, never the cell record. A
    // false fire on those would refuse the advisor consult Gate 3 itself
    // requires (`high_risk_advisor_refusal`) and deadlock the high-risk
    // workflow that approves guards, so the guard is handed one `&str` and the
    // door hands it nothing more. Its list and its arms live in
    // `brief_lint.rs` alone, so `bee blind check` can re-run the SAME rule
    // over a dossier's recorded brief without a second copy to drift.
    //
    // `resolve_brief_with_expertise`, never `resolve_brief_file`: a reading
    // list beside a brief is refused here, before the brief file is read,
    // because a list that reached a lane's payload would ride around the
    // guard on this very line.
    let (brief_text, brief_arg_refusal) = match resolve_brief_with_expertise(
        &kind,
        brief_flag.as_deref(),
        expertise_block.as_deref(),
    ) {
        Ok(Some(brief)) => match lint_brief(&brief) {
            Ok(()) => (Some(brief), None),
            Err(refusal) => (None, Some(refusal)),
        },
        Ok(None) => (None, None),
        Err(refusal) => (None, Some(refusal)),
    };

    // ── everything that can still delegate happens BEFORE prelude: its
    //    drift-cache write would otherwise swallow the Node re-run's
    //    manifest_changed line. ─────────────────────────────────────────────
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, "dispatch prepare", use_json, t0, &why));
        }
        Roots::None => {
            return Some(emit_no_root_error(&cwd, "dispatch prepare", use_json, t0));
        }
    };
    let prompt_name = if kind == "cell" { "worker-cell" } else { kind.as_str() };
    if !prompts_match_disk(&root, prompt_name) {
        return None; // prompt skew ⇒ delegate (C4)
    }
    let classification = if runtime == "codex" {
        Some(native_transport_classification(&root).ok()?)
    } else {
        None
    };

    // ── --claim's own argument refusals. bee.mjs throws all three BEFORE
    //    prepareDispatch is ever called, so they are resolved here and the
    //    dry-run below is skipped entirely when one fires. ─────────────────
    let claim_arg_error: Option<String> = if !claim {
        None
    } else if kind != "cell" {
        Some(format!(
            "dispatch prepare: --claim is only valid with --kind cell (got --kind {kind}) — claiming and reserving are cell-execution moves; gather/reviewer/advisor dispatches never own a cell."
        ))
    } else if cell_id.is_none() {
        Some("dispatch prepare: --cell is required when --kind cell.".to_string())
    } else if worker.is_none() {
        Some("dispatch prepare: --worker is required when --kind cell.".to_string())
    } else {
        None
    };

    let arg_error = expertise_arg_error.or(claim_arg_error);

    // Dry-run the whole build to surface every delegate-shaped input before a
    // single byte (or the prepare-time log line) is produced. The build is
    // free of side effects apart from appendPrepareRecord, which is applied on
    // the SECOND pass only.
    //
    // With --claim this pass is a DELEGATION PROBE ONLY: bee.mjs sequences the
    // claim BEFORE the payload build, so a Thrown produced here (over the
    // still-unclaimed cell) is not necessarily the message the real,
    // post-claim build produces. Its verdict is discarded; only "would this
    // delegate?" is kept, and it is kept because after the claim's O_EXCL
    // write nothing may delegate at all (campaign rule 2).
    let prepared = if arg_error.is_some() || brief_arg_refusal.is_some() {
        Prepared::Value(Value::Null) // unused — the refusal short-circuits below
    } else {
        prepare_dispatch_with_brief(
            &root,
            &runtime,
            &kind,
            role.as_deref(),
            cell_id.as_deref(),
            worker.as_deref(),
            force_ownership,
            classification,
            purpose.as_deref(),
            false,
            expertise_block.as_deref(),
            brief_text.as_deref(),
        )
        .ok()?
    };
    // The cross-worktree hold topology reservePathAtomic resolves for itself.
    // `prelude` above already narrowed this to an ORDINARY checkout, so this
    // is always `(workRoot, "main")` — a linked worktree delegated earlier.
    let topology = match claim {
        false => None,
        true => match crate::roots::resolve_store_root_worktree(&cwd) {
            crate::roots::RootsWt::Go(r) => r.hold_topology(),
            _ => return None,
        },
    };

    let ctx = match prelude("dispatch prepare", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    if let Some(message) = arg_error {
        return finish(&ctx, Ok(Out::Thrown(message)));
    }
    // The `--brief-file` refusals, in the shape every other prepare refusal
    // takes: emitted as the command's own JSON, exit 0, `{ok:false}` saying
    // which arm fired and how to fix it. Nothing was claimed, reserved or
    // logged before this point, so a refusal here leaves no state behind.
    if let Some(refusal) = brief_arg_refusal {
        let text = jsjson::stringify_pretty(&refusal);
        return finish(&ctx, Ok(Out::Emit(refusal, text, 0)));
    }

    // ── the claim + reserve gesture, before the payload build (Node's order).
    let mut claim_outcome: Option<Vec<Value>> = None;
    let mut worker_registered = false;
    let mut registration_error: Option<String> = None;
    if claim {
        let topo = topology.as_ref().map(|(m, h)| (m.as_path(), h.as_str()));
        match claim_and_reserve_for_dispatch(
            &ctx.root,
            topo,
            cell_id.as_deref().unwrap_or(""),
            worker.as_deref().unwrap_or(""),
            session_flag.as_deref(),
        ) {
            Ok(Ok((_cell, reserved, registered, reg_err))) => {
                claim_outcome = Some(reserved);
                worker_registered = registered;
                registration_error = reg_err;
            }
            Ok(Err(message)) => return finish(&ctx, Ok(Out::Thrown(message))),
            // The single-cell path never unwinds here either way (it simply
            // delegates the whole command to Node on an unproven shape), so
            // `claim_taken` carries no decision to make here — only the
            // Err2 payload does, byte-identical to before this door started
            // reporting it.
            Err((e, _claim_taken)) => return finish(&ctx, Err(e)),
        }
    }

    let out: R2<Out> = match prepared {
        Prepared::Thrown(msg) if !claim => Ok(Out::Thrown(msg)),
        _ => {
            // Re-run for real so the prepare-time record is appended exactly
            // once, with a freshly minted dispatch_id/ts like Node's.
            match prepare_dispatch_with_brief(
                &ctx.root,
                &runtime,
                &kind,
                role.as_deref(),
                cell_id.as_deref(),
                worker.as_deref(),
                force_ownership,
                classification,
                purpose.as_deref(),
                true,
                expertise_block.as_deref(),
                brief_text.as_deref(),
            ) {
                Ok(Prepared::Value(result)) => {
                    // `claimOutcome ? {...out, claimed:true, reserved} : out`
                    // — the claim/reservations are real state either way, so
                    // the result names them beside the payload.
                    let mut result = match &claim_outcome {
                        None => result,
                        Some(reserved) => {
                            let mut m = match result {
                                Value::Object(m) => m,
                                other => return finish(&ctx, Ok(Out::Emit(other, String::new(), 0))),
                            };
                            m.insert("claimed".into(), Value::Bool(true));
                            m.insert("reserved".into(), Value::Array(reserved.clone()));
                            // dp-r1: named loudly either way — the payload IS
                            // the printed text below, json mode or not.
                            m.insert("worker_registered".into(), Value::Bool(worker_registered));
                            if let Some(err) = &registration_error {
                                m.insert("registration_error".into(), Value::String(err.clone()));
                            }
                            Value::Object(m)
                        }
                    };

                    // dp-r2: a claim-less `--kind cell` prepare of a cell
                    // `--worker` already owns registers the worker too — the
                    // SAME write dp-r1 (above) makes after a fresh `--claim`,
                    // so `bee state worker list` and the B44 close-time door
                    // (`registered_worker_for_cell`) see the worker whether
                    // or not this particular call claimed anything. Re-reads
                    // the SAME cell record `prepare_dispatch`'s own
                    // ownership gate just read to build `result`, so the two
                    // never disagree: an ownership refusal there already
                    // returned a refusal envelope (`Ok(false)`, this arm
                    // never reached) before this branch runs, and registers
                    // nothing here either. A registration failure never
                    // turns the prepare itself into a failure — only the
                    // payload names it, same as dp-r1.
                    if claim_outcome.is_none() && kind == "cell" {
                        if let (Some(id), Some(w)) = (cell_id.as_deref(), worker.as_deref()) {
                            let trimmed = js_trim(w).to_string();
                            if let Ok(Some(loaded)) = read_cell(&ctx.root, id) {
                                let ownership = check_cell_claim_ownership(&loaded, &trimmed);
                                if ownership.ok {
                                    let tier = match loaded.get("tier") {
                                        Some(Value::String(t)) if !t.is_empty() => Some(t.clone()),
                                        _ => None,
                                    };
                                    let (registered, reg_err) =
                                        match crate::verbs::state_group::register_worker_for_cell(
                                            &ctx.root,
                                            &trimmed,
                                            id,
                                            tier.as_deref(),
                                        ) {
                                            Ok(()) => (true, None),
                                            Err(message) => (false, Some(message)),
                                        };
                                    let mut m = match result {
                                        Value::Object(m) => m,
                                        other => return finish(&ctx, Ok(Out::Emit(other, String::new(), 0))),
                                    };
                                    m.insert("worker_registered".into(), Value::Bool(registered));
                                    if let Some(err) = &reg_err {
                                        m.insert("registration_error".into(), Value::String(err.clone()));
                                    }
                                    result = Value::Object(m);
                                }
                            }
                        }
                    }

                    let text = jsjson::stringify_pretty(&result);
                    Ok(Out::Emit(result, text, 0))
                }
                Ok(Prepared::Thrown(msg)) => Ok(Out::Thrown(msg)),
                // Accepted residual, recorded in the module header: a
                // delegate-shaped input that the pre-claim probe did not see.
                // Unreachable in practice — record=true differs from the
                // probe only by appendPrepareRecord.
                Err(_) => Err(Err2::Ex),
            }
        }
    };
    finish(&ctx, out)
}

// ═══ dispatch wave ══════════════════════════════════════════════════════════
//
// `bee dispatch wave` — no bee.mjs counterpart (workflow-lessons wfl-4): the
// current schedule wave (`cells::compute_schedule`, the SAME door `cells
// schedule` reads), claimed and prepared in one call instead of one `dispatch
// prepare --cell <id> --worker <name> --claim` per cell. Every cell in the
// wave runs through the identical shared doors `dispatch prepare --claim`
// uses — `claim_and_reserve_for_dispatch` then `prepare_dispatch` — so a
// wave payload is byte-identical to what the single-cell command would have
// emitted for that cell; this command only saves the orchestrator the
// per-cell round trips. One cell's refusal (already claimed, a reservation
// conflict) never aborts the rest of the wave — it is recorded in `skipped`
// with a typed reason and the loop continues.
//
// dispatch review P1: this door MUTATES the shared control plane (claims,
// reservations, worker rows), unlike its read-only sibling `cells schedule`
// — whose all-features default it had inherited by construction. The
// resolved feature is exactly one of: an explicit `--feature`, the calling
// session's bound lane, or the default record's own `feature` (the same
// three-step `resolve_mutation_lock_scope` every mutating `state` verb
// already resolves against); nothing resolving is a typed refusal, never a
// silent every-feature grab. `--limit <n>` (a positive integer) caps how
// many cells of the current wave are actually claimed — the rest of the
// wave is simply left untouched, not reported — bounding a speculative
// batch by what the caller can actually spawn workers for; omitted, the
// whole wave stands as before.

/// The worker nickname `dispatch wave` claims each cell under when the
/// caller does not hand one down: derived from the cell id alone, so two
/// runs against the same still-open cell (or a human reading the claim
/// trace) see a stable, self-explaining name — never a counter that drifts
/// with call order.
pub(crate) fn auto_wave_worker_name(cell_id: &str) -> String {
    format!("w-{cell_id}")
}

/// Classifies a `claim_and_reserve_for_dispatch` refusal string into the
/// named `skipped` reasons the cell calls out — "an unwind of that same
/// refusal itself failed", "already claimed", and "a reservation conflict"
/// — falling back to a generic `claim_refused` for any other typed refusal
/// that door can produce (budget caps, an unapproved execution gate, and the
/// like). Message-sniffing is the one option here: the door returns a
/// rendered String, not a typed enum, by the same design `dispatch prepare
/// --claim`'s own single-cell caller already accepts.
///
/// `UNWIND FAILED` is checked FIRST (review P2): the door's own reservation-
/// conflict message already embeds its unwind note in the same string
/// (`"...reservation conflict on cell ...; UNWIND FAILED (...)..."`), so a
/// naive "reservation conflict" match alone would bury a leaked claim behind
/// the ordinary, already-handled `reservation_conflict` reason instead of
/// surfacing the worse, still-open state.
pub(crate) fn wave_skip_reason(message: &str) -> &'static str {
    if message.contains("UNWIND FAILED") {
        "unwind_failed"
    } else if message.contains("reservation conflict") {
        "reservation_conflict"
    } else if message.contains("not \"open\"") {
        "already_claimed"
    } else {
        "claim_refused"
    }
}

fn wave_skip(id: &str, reason: &'static str, detail: String) -> Value {
    let mut s = Map::new();
    s.insert("id".into(), Value::String(id.to_string()));
    s.insert("reason".into(), Value::String(reason.to_string()));
    s.insert("detail".into(), Value::String(detail));
    Value::Object(s)
}

/// dp-r1's registration counterpart: strips the `(nickname, cell)` worker
/// row a wave claim registered, before `unwind_wave_claim` undoes the claim
/// itself (dispatch review, carried: prepare.rs:1021-1025 registers this
/// exact row, but the unwind never removed it). `state worker remove`
/// (`state_group/workers.rs::run_worker_remove`) exists but is nickname-only
/// — it would strip EVERY cell that nickname ever held, wrong here since a
/// worker legitimately holding a different cell must be left standing — so
/// this goes through the SAME `worker_mutate` lock+read+write frame
/// `register_worker_for_cell` writes through, scoped to the exact
/// `(nickname, cell)` pair a claim ever registers. Idempotent: a row that
/// was never registered (the claim failed before dp-r1 ran) leaves nothing
/// to remove, same as `release_reservations_for_agent` against an
/// agent/cell pair holding no reservations — `None` on success either way.
fn unregister_worker_for_cell(root: &Path, nickname: &str, cell: &str) -> Option<String> {
    let nickname_v = Value::String(nickname.to_string());
    let cell_v = Value::String(cell.to_string());
    match crate::verbs::state_group::worker_mutate(root, move |workers| {
        workers.retain(|w| {
            !(truthy(w) && w.get("nickname") == Some(&nickname_v) && w.get("cell") == Some(&cell_v))
        });
        Ok(String::new())
    }) {
        Ok(_) => None,
        Err(Err2::Msg(m)) => Some(m),
        Err(Err2::Ex) => Some("removing the worker row hit an unsupported store shape".to_string()),
    }
}

/// Best-effort unwind for a wave-loop claim that must be undone before its
/// cell lands in `skipped`: either a claim+reserve that SUCCEEDED,
/// immediately followed by a `prepare_dispatch` that failed to build a
/// payload over the very cell it just loaded (kind is always "cell", the
/// worker is always non-empty, and ownership always matches the claim this
/// same call just took — see `prepare_dispatch`'s Thrown arms); or (review
/// P2) a `claim_and_reserve_for_dispatch` call whose OWN internal unwind
/// never ran because its failure propagated out through a bare `?` on
/// `reserve_path_atomic`, mid-loop, after the claim itself already stood —
/// the wave loop cannot see how many of that cell's files were reserved
/// before the failure, so it cannot hand this function an accurate
/// `reserved` list. Always attempts every step (release, unclaim, worker-row
/// removal) regardless of what the caller can prove happened —
/// `release_reservations_for_agent` and `unregister_worker_for_cell` are
/// both no-ops when nothing matches, so calling them on a claim that never
/// got that far costs nothing. Mirrors `claim_and_reserve_for_dispatch`'s
/// own unwind (reservations first, then the claim, in reverse), with
/// `force_ownership` on the unclaim since this call is undoing its own,
/// still-fresh claim rather than resolving a contested one — the caller MUST
/// only reach here when `claim_taken` is true (see
/// `claim_and_reserve_for_dispatch`'s own doc comment); calling this over a
/// claim the wave never took would force through an ownership guard it does
/// not own.
pub(crate) fn unwind_wave_claim(root: &Path, topo: Option<(&Path, &str)>, worker: &str, id: &str) -> String {
    let mut note = "the claim, its reservations and its worker row were unwound".to_string();
    let release_failed =
        match release_reservations_for_agent(topo, root.to_str().unwrap_or(""), worker, Some(id)) {
            Ok(Out::Thrown(m)) => Some(m),
            Err(_) => Some("release_reservations_for_agent hit an unproven shape".to_string()),
            Ok(_) => None,
        };
    let unclaim_failed = match crate::verbs::cells::unclaim_cell(root, id, None, true) {
        Ok(_) => None,
        Err(crate::verbs::cells::Fail::Delegate) => {
            Some("unclaim_cell hit an unproven shape".to_string())
        }
        Err(crate::verbs::cells::Fail::Thrown(m)) => Some(m),
    };
    let worker_row_failed = unregister_worker_for_cell(root, worker, id);
    if release_failed.is_some() || unclaim_failed.is_some() || worker_row_failed.is_some() {
        note = format!(
            "UNWIND FAILED (release: {}; unclaim: {}; worker row: {}) — restore by hand: bee reservations release --agent {worker} --cell {id} --json ; bee cells unclaim --id {id} --force-ownership --json ; bee state worker remove --nickname {worker} --json",
            release_failed.as_deref().unwrap_or("ok"),
            unclaim_failed.as_deref().unwrap_or("ok"),
            worker_row_failed.as_deref().unwrap_or("ok"),
        );
    }
    note
}

/// provenance: none — new native command (workflow-lessons wfl-4).
pub(crate) fn run_dispatch_wave(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(
        &flags,
        &["runtime", "feature", "session-id", "limit"],
    ) {
        return None;
    }
    let runtime = flags.req_str("runtime")?.to_string();
    if !DISPATCH_RUNTIMES.contains(&runtime.as_str()) {
        return None; // validate()'s enum message equivalent
    }
    let feature_flag = flags.truthy_str("feature").map(str::to_string);
    let session_flag = flags.truthy_str("session-id").map(str::to_string);
    // `--limit`: shape only here (a bare `--limit` with no value is
    // unproven); the positive-integer value check waits for `ctx` below so
    // a bad value gets the same typed refusal every other value-shape
    // problem in this door gets, not a bare "unsupported command shape".
    let limit_flag = match flags.get("limit") {
        None => None,
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(FlagV::Present) => return None,
    };

    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, "dispatch wave", use_json, t0, &why));
        }
        Roots::None => {
            return Some(emit_no_root_error(&cwd, "dispatch wave", use_json, t0));
        }
    };
    // Every wave cell renders the "worker-cell" prompt (kind is always
    // "cell") — the same skew guard `dispatch prepare --claim` applies.
    if !prompts_match_disk(&root, "worker-cell") {
        return None;
    }
    let classification = if runtime == "codex" {
        Some(native_transport_classification(&root).ok()?)
    } else {
        None
    };
    // Claiming is not optional for `dispatch wave` (there is no read-only
    // mode), so the hold topology is always resolved up front — the same
    // narrow-door check `dispatch prepare --claim` makes.
    let topology: Option<(PathBuf, String)> =
        match crate::roots::resolve_store_root_worktree(&cwd) {
            crate::roots::RootsWt::Go(r) => r.hold_topology(),
            _ => return None,
        };

    let ctx = match prelude("dispatch wave", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };

    // dispatch review P1: resolve the ONE feature this wave targets — an
    // explicit `--feature`, else the calling session's bound lane, else the
    // default record's own `feature` — the identical three-step resolution
    // `resolve_mutation_lock_scope` gives every mutating `state` verb.
    // Nothing resolving is a typed refusal: a mutating wave never falls back
    // to the read-only `cells schedule` default of every feature at once.
    let scope = match crate::verbs::state_group::resolve_mutation_lock_scope(
        &ctx.root,
        feature_flag.as_deref(),
        false,
    ) {
        Ok(s) => s,
        Err(_) => return finish(&ctx, Err(Err2::Ex)),
    };
    let Some(feature) = scope.feature else {
        return finish(
            &ctx,
            Err(Err2::Msg(
                "dispatch wave: refused — no feature resolved (no --feature given, the \
                 calling session has no bound lane, and the default record names none). \
                 FIX: pass --feature <name> naming the pipeline to dispatch."
                    .to_string(),
            )),
        );
    };
    let limit: Option<usize> = match &limit_flag {
        None => None,
        Some(raw) => match raw.trim().parse::<u64>() {
            Ok(n) if n > 0 => Some(n as usize),
            _ => {
                return finish(
                    &ctx,
                    Err(Err2::Msg(
                        "dispatch wave: --limit must be a positive integer.".to_string(),
                    )),
                );
            }
        },
    };

    let cells = match crate::verbs::cells::list_cells(&ctx.root, Some(feature.as_str()), None) {
        Ok(cells) => cells,
        Err(crate::verbs::cells::Delegate) => return finish(&ctx, Err(Err2::Ex)),
    };
    // Same exotic-id guard `cells schedule` applies before scheduling.
    for cell in &cells {
        let schedulable =
            matches!(cell.get("status"), Some(Value::String(s)) if s == "open" || s == "claimed");
        if schedulable && !matches!(cell.get("id"), Some(Value::String(_))) {
            return finish(&ctx, Err(Err2::Ex));
        }
    }
    let schedule = crate::verbs::cells::compute_schedule(&cells);
    let mut wave_ids: Vec<String> = schedule.waves.first().cloned().unwrap_or_default();
    if let Some(n) = limit {
        wave_ids.truncate(n);
    }

    let topo = topology.as_ref().map(|(m, h)| (m.as_path(), h.as_str()));
    let mut wave_payloads: Vec<Value> = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();
    let mut economics: Vec<Value> = Vec::new();

    for id in &wave_ids {
        let worker = auto_wave_worker_name(id);
        match claim_and_reserve_for_dispatch(&ctx.root, topo, id, &worker, session_flag.as_deref())
        {
            Err((_e, claim_taken)) if claim_taken => {
                // dispatch review P2: `claim_and_reserve_for_dispatch`'s own
                // `?`-propagated Err never reaches ITS internal unwind, so
                // the claim (and any reservations it already took) may
                // still stand — best-effort undo it here rather than leak
                // it, and name the cell in the detail either way.
                let note = unwind_wave_claim(&ctx.root, topo, &worker, id);
                let unwind_failed = note.contains("UNWIND FAILED");
                skipped.push(wave_skip(
                    id,
                    if unwind_failed { "unwind_failed" } else { "unsupported" },
                    format!(
                        "cell \"{id}\" hit an unproven shape mid-wave — run \
                         `bee dispatch prepare --cell {id} --worker {worker} --runtime {runtime} --claim` \
                         for it directly. ({note})"
                    ),
                ));
            }
            Err((_e, _claim_taken)) => {
                // dispatch review delta (hpf-3): the claim door itself never
                // took this cell — an exotic-shape delegate fired ahead of
                // the claim (handlers_write.rs:904/915), possibly over a
                // cell ALREADY claimed by a live agent. Nothing here to
                // unwind, and force-unclaiming would write straight through
                // whatever the cell already held (open, or that other
                // agent's claim) rather than something THIS call put there.
                // Report the real reason; no unwind note — there is nothing
                // to report an unwind of.
                skipped.push(wave_skip(
                    id,
                    "claim_refused",
                    format!(
                        "cell \"{id}\" hit an unproven shape mid-wave before any claim was \
                         taken — run `bee dispatch prepare --cell {id} --worker {worker} \
                         --runtime {runtime} --claim` for it directly."
                    ),
                ));
            }
            Ok(Err(message)) => {
                let reason = wave_skip_reason(&message);
                skipped.push(wave_skip(id, reason, message));
            }
            Ok(Ok((_cell, reserved, worker_registered, registration_error))) => {
                match prepare_dispatch(
                    &ctx.root,
                    &runtime,
                    "cell",
                    Some(id.as_str()),
                    Some(worker.as_str()),
                    false,
                    classification,
                    None,
                    true,
                    None,
                ) {
                    // pi-support D5 in the WAVE door: the same refusal
                    // `dispatch prepare` emits, and the claim this loop just
                    // took is unwound rather than left standing on a cell
                    // nothing can dispatch. Every cell of a pi wave resolves
                    // the same `models.pi` table, so this fires for the whole
                    // wave or for none of it — the operator gets a `skipped`
                    // row per cell naming the slot and the herding shape,
                    // instead of a `wave` array of refusals holding claims.
                    Ok(Prepared::Value(result))
                        if result.get("reason").and_then(Value::as_str)
                            == Some(PI_HERDING_ONLY_REASON) =>
                    {
                        let note = unwind_wave_claim(&ctx.root, topo, &worker, id);
                        let fix =
                            result.get("fix").and_then(Value::as_str).unwrap_or_default();
                        skipped.push(wave_skip(
                            id,
                            PI_HERDING_ONLY_REASON,
                            format!("cell \"{id}\": {fix} — {note}"),
                        ));
                    }
                    Ok(Prepared::Value(result)) => {
                        let mut m = match result {
                            Value::Object(m) => m,
                            // kind "cell" always renders an object envelope
                            // (either the full payload or a typed refusal);
                            // this arm is unreached in practice.
                            other => {
                                wave_payloads.push(other);
                                continue;
                            }
                        };
                        m.insert("claimed".into(), Value::Bool(true));
                        m.insert("reserved".into(), Value::Array(reserved));
                        m.insert("worker_registered".into(), Value::Bool(worker_registered));
                        if let Some(err) = &registration_error {
                            m.insert("registration_error".into(), Value::String(err.clone()));
                        }
                        if let Some(Value::Object(econ)) = m.get("economics") {
                            let mut e = econ.clone();
                            e.insert("id".into(), Value::String(id.clone()));
                            economics.push(Value::Object(e));
                        }
                        wave_payloads.push(Value::Object(m));
                    }
                    Ok(Prepared::Thrown(msg)) => {
                        let note = unwind_wave_claim(&ctx.root, topo, &worker, id);
                        skipped.push(wave_skip(
                            id,
                            "prepare_failed",
                            format!("cell \"{id}\": {msg} — {note}"),
                        ));
                    }
                    Err(Delegate) => {
                        let note = unwind_wave_claim(&ctx.root, topo, &worker, id);
                        skipped.push(wave_skip(
                            id,
                            "prepare_failed",
                            format!(
                                "cell \"{id}\": prepare_dispatch hit an unproven shape mid-wave — {note}"
                            ),
                        ));
                    }
                }
            }
        }
    }

    let mut result = Map::new();
    result.insert("wave".into(), Value::Array(wave_payloads));
    result.insert("skipped".into(), Value::Array(skipped));
    result.insert("economics".into(), Value::Array(economics));
    let result = Value::Object(result);
    let text = jsjson::stringify_pretty(&result);
    finish(&ctx, Ok(Out::Emit(result, text, 0)))
}

// ═══ tests — the slot map's explicit arms (D2 / 06e49368) ══════════════════

#[cfg(test)]
mod slot_map_tests {
    use super::*;

    /// The four live kinds keep exactly the slots they resolved before the
    /// catch-all went away — cell/gather → generation, reviewer → review,
    /// advisor → advisor.
    #[test]
    fn every_dispatch_kind_keeps_its_slot() {
        assert_eq!(slot_for_kind("cell"), Some("generation"));
        assert_eq!(slot_for_kind("gather"), Some("generation"));
        assert_eq!(slot_for_kind("reviewer"), Some("review"));
        assert_eq!(slot_for_kind("advisor"), Some("advisor"));
        // Every declared kind has an arm: no member of DISPATCH_KINDS may
        // reach the unmapped branch.
        for kind in DISPATCH_KINDS {
            assert!(slot_for_kind(kind).is_some(), "kind {kind:?} has no slot arm");
        }
    }

    /// `advisor` is the one gate onto the advisor slot. Nothing else — not a
    /// live kind, not a name that could plausibly be added later — resolves
    /// it by falling through.
    #[test]
    fn only_the_advisor_arm_resolves_the_advisor_slot() {
        for kind in ["cell", "gather", "reviewer", "extract", "judge", "", "advisorr", "ADVISOR"] {
            assert_ne!(
                slot_for_kind(kind),
                Some("advisor"),
                "kind {kind:?} resolved the advisor slot"
            );
        }
        assert_eq!(slot_for_kind("advisor"), Some("advisor"));
    }

    /// An unhandled kind resolves NOTHING: it returns None so the caller can
    /// refuse, rather than silently landing on some other consumer's model.
    #[test]
    fn an_unhandled_kind_resolves_no_slot_at_all() {
        for kind in ["extract", "judge", "scribe", "", "  ", "Cell"] {
            assert_eq!(slot_for_kind(kind), None, "kind {kind:?} resolved a slot");
        }
    }

    /// The refusal is typed and names its remedy — the shape a caller can
    /// branch on, not free prose.
    #[test]
    fn the_unmapped_kind_refusal_is_typed_and_names_its_remedy() {
        let refusal = unmapped_kind_refusal("extract");
        assert_eq!(refusal.get("ok"), Some(&Value::Bool(false)));
        assert_eq!(refusal.get("type"), Some(&Value::String("refused".into())));
        assert_eq!(
            refusal.get("reason"),
            Some(&Value::String("kind_slot_unmapped".into()))
        );
        assert_eq!(refusal.get("kind"), Some(&Value::String("extract".into())));
        let fix = refusal.get("fix").and_then(|v| v.as_str()).unwrap_or_default();
        assert!(fix.contains("extract"), "fix must name the kind: {fix}");
        assert!(fix.contains("slot_for_kind"), "fix must name the remedy: {fix}");
    }
}

// ═══ tests — the explicit --role override (T012a / 8ff6e79e) ══════════════

#[cfg(test)]
mod role_flag_tests {
    use super::*;
    use serde_json::json;

    /// A repo root with nothing but the two files prepare reads. Kept local
    /// rather than imported: the drivers test module's own `repo` fixture is
    /// private to its `#[cfg(test)] mod tests`.
    fn repo(tmp: &tempfile::TempDir, config: &str) -> PathBuf {
        let root = tmp.path().to_path_buf();
        for (rel, body) in [(".bee/onboarding.json", "{\"version\":1}"), (".bee/config.json", config)]
        {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, body).unwrap();
        }
        root
    }

    const THREE_SLOTS: &str =
        r#"{"models":{"claude":{"extraction":"haiku","generation":"sonnet","review":"opus"}}}"#;

    fn envelope(root: &Path, kind: &str, role: Option<&str>) -> Value {
        let out =
            prepare_dispatch_with_role(root, "claude", kind, role, None, None, false, None, None, false, None)
                .unwrap();
        let Prepared::Value(v) = out else { panic!("expected an envelope for {kind}/{role:?}") };
        v
    }

    /// THE reachability truth (decision 8dad7c2e's stated defect, closed):
    /// bee-extract is rendered, onboarded and documented, and now the one
    /// door can return it. A read-shaped dispatch names the read-shaped role
    /// and gets the reader — no fifth dispatch kind, no remapped `gather`.
    #[test]
    fn a_read_shaped_role_returns_the_bee_extract_worker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, THREE_SLOTS);
        let v = envelope(&root, "gather", Some("extraction"));
        assert_eq!(v.get("tool"), Some(&json!("Agent")));
        let p = v.get("payload").unwrap();
        assert_eq!(p.get("subagent_type"), Some(&json!("bee-extract")));
        assert_eq!(p.get("model"), Some(&json!("haiku")), "the ROLE picked the model");
        // The marker the guard will read back names the same role, so the
        // audit line and the model that actually runs cannot disagree.
        let prompt = p.get("prompt").and_then(Value::as_str).unwrap_or_default();
        assert!(prompt.starts_with("[bee-tier: extraction]"), "{prompt}");
        let e = v.get("economics").unwrap();
        assert_eq!(e.get("logical_tier"), Some(&json!("extraction")));
        assert_eq!(e.get("tier_source"), Some(&json!("flag")), "the caller chose it, and the record says so");
    }

    /// Every rendered bee agent is reachable through the one door. This is
    /// the invariant the docs half of D12 (mrs-7) rewrites against: an agent
    /// name is what prepare RETURNS, never something a caller hand-picks.
    #[test]
    fn every_rendered_agent_is_reachable_through_the_one_door() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, THREE_SLOTS);
        let mut returned: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for (kind, role) in
            [("gather", None), ("gather", Some("extraction")), ("reviewer", None)]
        {
            let v = envelope(&root, kind, role);
            let t = v.get("payload").and_then(|p| p.get("subagent_type")).cloned().unwrap();
            returned.insert(t.as_str().unwrap().to_string());
        }
        // bee-build is the cell-execution agent: `--kind cell` names it, and
        // that is deliberately NOT role-driven — a cell execution reserves,
        // writes and caps whatever job the cell declares (`--role` selects
        // the MODEL, per D4, never the capability).
        w_cell(&root, "c-1");
        let Prepared::Value(v) = prepare_dispatch_with_role(
            &root, "claude", "cell", None, Some("c-1"), Some("w"), false, None, None, false, None,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        returned.insert(
            v.get("payload")
                .and_then(|p| p.get("subagent_type"))
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
        );
        for (_, agent) in crate::verbs::drivers::ROLE_AGENTS {
            assert!(
                returned.contains(agent),
                "{agent} is rendered and onboarded but the door never returns it; door returned {returned:?}"
            );
        }
    }

    fn w_cell(root: &Path, id: &str) {
        let path = root.join(".bee").join("cells").join(format!("{id}.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            format!(r#"{{"id":"{id}","feature":"f","status":"claimed","trace":{{"worker":"w"}}}}"#),
        )
        .unwrap();
    }

    /// Absent `--role`, nothing moved. Every kind resolves the slot it always
    /// resolved, and the envelope is byte-identical to the no-role spelling
    /// every existing caller uses.
    #[test]
    fn absent_role_every_dispatch_is_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, THREE_SLOTS);
        for (kind, agent, model) in [
            ("gather", "bee-gather", "sonnet"),
            ("reviewer", "bee-review", "opus"),
        ] {
            let with_none = envelope(&root, kind, None);
            let p = with_none.get("payload").unwrap();
            assert_eq!(p.get("subagent_type"), Some(&json!(agent)), "{kind}");
            assert_eq!(p.get("model"), Some(&json!(model)), "{kind}");
            assert_eq!(
                with_none.get("economics").and_then(|e| e.get("tier_source")),
                Some(&json!("default"))
            );
            // The ten-argument spelling and the eleven-argument one are the
            // same code: an adapter, not a second implementation.
            let old = prepare_dispatch(&root, "claude", kind, None, None, false, None, None, false, None)
                .unwrap();
            let Prepared::Value(old) = old else { panic!("expected an envelope") };
            assert_eq!(
                jsjson::stringify(&strip_volatile(old)),
                jsjson::stringify(&strip_volatile(with_none)),
                "{kind}: --role absent must change nothing"
            );
        }
    }

    /// `dispatch_id`/`ts` are freshly minted per call; everything else must
    /// match byte for byte.
    fn strip_volatile(v: Value) -> Value {
        let Value::Object(mut m) = v else { return v };
        m.remove("dispatch_id");
        m.remove("ts");
        Value::Object(m)
    }

    /// An explicit role beats the cell's own recorded value: the caller
    /// stating the job at the door is the most specific signal there is.
    #[test]
    fn an_explicit_role_outranks_the_cells_recorded_value() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, THREE_SLOTS);
        w_cell(&root, "c-1");
        let Prepared::Value(v) = prepare_dispatch_with_role(
            &root,
            "claude",
            "cell",
            Some("review"),
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        let e = v.get("economics").unwrap();
        assert_eq!(e.get("logical_tier"), Some(&json!("review")));
        assert_eq!(e.get("tier_source"), Some(&json!("flag")));
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("opus")));
        // …and the execution agent is unchanged: `--role` selects the model,
        // never what the worker is allowed to do.
        assert_eq!(
            v.get("payload").and_then(|p| p.get("subagent_type")),
            Some(&json!("bee-build"))
        );
    }

    /// A typo is refused, never resolved onto some other role's model — the
    /// same answer the model-guard gives a marker naming an unconfigured
    /// role, because both doors ask one `known_roles`.
    #[test]
    fn a_role_nothing_configures_is_refused_with_the_configured_list() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, THREE_SLOTS);
        let v = envelope(&root, "gather", Some("extractoin"));
        assert_eq!(v.get("ok"), Some(&json!(false)));
        assert_eq!(v.get("type"), Some(&json!("refused")));
        assert_eq!(v.get("reason"), Some(&json!("role_not_configured")));
        assert_eq!(v.get("role"), Some(&json!("extractoin")));
        let fix = v.get("fix").and_then(Value::as_str).unwrap_or_default();
        assert!(fix.contains("extractoin"), "the FIX names the typo: {fix}");
        for configured in ["extraction", "generation", "review"] {
            assert!(fix.contains(configured), "the FIX lists {configured}: {fix}");
        }
        assert!(fix.contains("models.claude"), "the FIX names where to add it: {fix}");
        // A role the operator invented and CONFIGURED is legal — the open set
        // is open (D2), so this is not a four-word allowlist wearing a new
        // name.
        let tmp2 = tempfile::tempdir().unwrap();
        let open = repo(
            &tmp2,
            r#"{"models":{"claude":{"extraction":"haiku","generation":"sonnet","review":"opus","docs":"haiku"}}}"#,
        );
        let v = envelope(&open, "gather", Some("docs"));
        assert_eq!(v.get("ok"), None, "a configured role is not refused: {v}");
        assert_eq!(
            v.get("economics").and_then(|e| e.get("logical_tier")),
            Some(&json!("docs"))
        );
        // No rendered agent for `docs`, so the payload carries the runtime's
        // own generic rather than an invented name.
        assert_eq!(
            v.get("payload").and_then(|p| p.get("subagent_type")),
            Some(&json!("general-purpose"))
        );
    }

    /// `--role advisor` reaches the advisor's own resolver (never budget,
    /// never a tier fallback), and `--kind advisor --role <other>` no longer
    /// silently reads the advisor slot instead of the role that was asked
    /// for.
    #[test]
    fn the_advisor_slot_follows_the_role_not_the_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"extraction":"haiku","generation":"sonnet","review":"opus","advisor":"opus"}}}"#,
        );
        let v = envelope(&root, "gather", Some("advisor"));
        assert_eq!(v.get("economics").and_then(|e| e.get("logical_tier")), Some(&json!("advisor")));
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("opus")));
        let v = envelope(&root, "advisor", Some("extraction"));
        assert_eq!(
            v.get("economics").and_then(|e| e.get("logical_tier")),
            Some(&json!("extraction")),
            "the role the caller named, not the kind's slot"
        );
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("haiku")));
    }

    /// A null-valued `advisor` is the OFF spelling bee itself teaches
    /// (`.bee/config-sample.json`: "Set null to skip the advisor line"), and
    /// EVERY door has to read it as off — not only the two that happen to
    /// resolve through the advisor's own floor-less walk.
    ///
    /// `c2ef2f9f` closed the case where the key is ABSENT. A present-but-null
    /// key is still a key, so `known_roles` kept handing the name out as
    /// legal: `[bee-tier: advisor]` classified as a configured role, skipped
    /// the unconfigured-role refusal, resolved `Resolved::Budget` and let the
    /// subagent inherit the session model, while `--role advisor` and `--kind
    /// advisor` on the SAME host refused. One question, two doors, two
    /// answers — through the reachable configuration bee ships.
    #[test]
    /// role-edge-hardening D1: a mis-cased "Advisor" config key answers the
    /// SAME at both doors, in both directions. Before the case-fold,
    /// `role_is_declarable` matched the advisor arm exactly and fell through
    /// to `contains_key`, so "Advisor": null entered `known_roles`, resolved
    /// as Budget at the marker door, and refused at `--kind advisor` — one
    /// question, two answers, reopened by a typo.
    #[test]
    fn a_mis_cased_advisor_key_answers_the_same_at_both_doors() {
        // Direction one: "Advisor": null — OFF at both doors.
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"extraction":"haiku","generation":"sonnet","Advisor":null}}}"#,
        );
        let models = read_models(&root).unwrap();
        assert!(
            !known_roles(&models, "claude").iter().any(|k| k.eq_ignore_ascii_case("advisor")),
            "a null advisor is off whatever the key's case"
        );
        let v = envelope(&root, "gather", Some("Advisor"));
        assert_eq!(v.get("reason"), Some(&json!("role_not_configured")));
        let v = envelope(&root, "advisor", None);
        assert_eq!(v.get("reason"), Some(&json!("advisor_not_configured")));

        // Direction two: "Advisor": "fable" — ON at both doors.
        let tmp2 = tempfile::tempdir().unwrap();
        let on = repo(
            &tmp2,
            r#"{"models":{"claude":{"extraction":"haiku","generation":"sonnet","Advisor":"fable"}}}"#,
        );
        let v = envelope(&on, "advisor", None);
        assert_eq!(v.get("reason"), None, "--kind advisor must resolve the mis-cased key: {v}");
        let models = read_models(&on).unwrap();
        assert!(
            known_roles(&models, "claude").iter().any(|k| k.eq_ignore_ascii_case("advisor")),
            "and the marker door sees the same configured advisor"
        );
    }

    #[test]
    fn a_null_advisor_is_off_at_every_door_not_only_at_the_ones_that_resolve() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"extraction":"haiku","generation":"sonnet","review":"opus","advisor":null}}}"#,
        );
        // The derivation both doors share no longer calls it a role at all,
        // which is what the model guard reads when it classifies the marker.
        let models = read_models(&root).unwrap();
        assert!(
            !known_roles(&models, "claude").contains("advisor"),
            "a null advisor is a slot switched OFF, not a declarable role"
        );
        assert!(known_role_named(&models, "claude", "advisor").is_none());
        assert!(!role_list(&models, "claude").contains("advisor"), "nor is it offered as a FIX");
        // …so the flag door refuses…
        let v = envelope(&root, "gather", Some("advisor"));
        assert_eq!(v.get("reason"), Some(&json!("role_not_configured")));
        // …and case-insensitivity does not smuggle the off slot back in.
        let v = envelope(&root, "gather", Some("ADVISOR"));
        assert_eq!(v.get("reason"), Some(&json!("role_not_configured")));
        // …exactly as `--kind advisor` already did, through `resolve_advisor`.
        let v = envelope(&root, "advisor", None);
        assert_eq!(v.get("reason"), Some(&json!("advisor_not_configured")));

        // Every OTHER role keeps its documented prompt-budget floor, null
        // value and all: a blanket non-null rule here would refuse every
        // codex dispatch bee makes, since `default_models` seeds every codex
        // slot null.
        let tmp2 = tempfile::tempdir().unwrap();
        let off = repo(&tmp2, r#"{"models":{"claude":{"generation":null},"codex":{}}}"#);
        let models = read_models(&off).unwrap();
        for name in ["generation", "extraction", "review"] {
            assert!(known_roles(&models, "codex").contains(name), "codex {name}");
        }
        assert!(known_roles(&models, "claude").contains("generation"));
        let v = envelope(&off, "gather", Some("generation"));
        assert_eq!(v.get("ok"), None, "a null ordinary slot is prompt-budget, not a refusal: {v}");

        // A CONFIGURED advisor is untouched by all of it.
        let tmp3 = tempfile::tempdir().unwrap();
        let on = repo(&tmp3, r#"{"models":{"claude":{"generation":"sonnet","advisor":"fable"}}}"#);
        assert_eq!(
            envelope(&on, "advisor", None).get("payload").and_then(|p| p.get("model")),
            Some(&json!("fable"))
        );
    }

    /// One typo, one answer. The guard's marker door has always matched a
    /// role name case-insensitively (`[BEE-TIER: Generation]` declares the
    /// `generation` role); this door asked a case-SENSITIVE `contains`, so
    /// `--role Generation` was refused here while the marker was admitted and
    /// resolved there — the two doors sharing a derivation but not a
    /// predicate. Nothing exercised a mixed-case `--role` before.
    #[test]
    fn a_mixed_case_role_is_admitted_and_normalized_to_the_configs_spelling() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, THREE_SLOTS);
        let v = envelope(&root, "gather", Some("Generation"));
        assert_eq!(v.get("ok"), None, "the guard admits this spelling, so this door does: {v}");
        // The CONFIG's spelling is what travels, never the caller's: the
        // marker the guard reads back, the audit line and the model that
        // actually runs all name one key the resolver can look up.
        assert_eq!(
            v.get("economics").and_then(|e| e.get("logical_tier")),
            Some(&json!("generation"))
        );
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("sonnet")));
        let prompt =
            v.get("payload").and_then(|p| p.get("prompt")).and_then(Value::as_str).unwrap_or_default();
        assert!(prompt.starts_with("[bee-tier: generation]"), "{prompt}");
        // A name nothing configures is still refused in the spelling the
        // caller typed, so the FIX names what they wrote.
        let v = envelope(&root, "gather", Some("Generatoin"));
        assert_eq!(v.get("reason"), Some(&json!("role_not_configured")));
        assert_eq!(v.get("role"), Some(&json!("Generatoin")));
    }

    // ═══ lane-model-diversity — the SEAT roles (D1/D2/D4) ══════════════════

    /// A host that configures the advisor and nothing seat-shaped: every one
    /// of the eight seats is unconfigured here, which is the state every
    /// existing host is in the moment this ships.
    const ADVISOR_ONLY: &str =
        r#"{"models":{"claude":{"generation":"sonnet","advisor":"opus"}}}"#;

    /// D1 — the eight seats, as the two procedures name them. This asserts the
    /// CONTENT of the list, not just that a list exists: the constant is what
    /// `gates-and-delegation.md` cites, so a name silently dropped from it
    /// would make the doc wrong with nothing red to say so.
    #[test]
    fn the_seat_roles_are_the_three_lanes_and_the_five_hats() {
        assert_eq!(
            SEAT_ROLES.to_vec(),
            vec![
                "lane-1",
                "lane-2",
                "lane-3",
                "hat-facts-gaps",
                "hat-risks",
                "hat-value",
                "hat-alternatives",
                "hat-user-impact",
            ]
        );
        // Membership folds case, like every other role door.
        assert_eq!(seat_role_named("lane-2"), Some("lane-2"));
        assert_eq!(seat_role_named("Lane-2"), Some("lane-2"));
        assert_eq!(seat_role_named("HAT-RISKS"), Some("hat-risks"));
        // …and it is CLOSED: a near-miss is not a seat, which is what keeps
        // D2's fall-through from swallowing typos.
        for stranger in ["hat-risk", "lane-4", "lane", "hat-", "advisor", "generation", ""] {
            assert_eq!(seat_role_named(stranger), None, "{stranger} must not be a seat");
        }
        // Every hat is prefixed, no lane is — the predicate doctor's advisory
        // reads.
        for seat in SEAT_ROLES {
            assert_eq!(
                seat.starts_with(HAT_ROLE_PREFIX),
                seat.starts_with("hat"),
                "{seat} disagrees with the hat prefix"
            );
        }
    }

    /// D2 — the fall-through, in the two spellings of "this seat carries
    /// nothing": the key is ABSENT, and the key is present but `null`.
    ///
    /// The null case is the one that had a hole. `resolve_role_named`'s last
    /// entry always resolves, so a walk over `["lane-3"]` would have answered
    /// `Resolved::Budget` — no model parameter, the subagent inherits the
    /// SESSION model — for the exact spelling `.bee/config-sample.json`
    /// teaches for "off". Both spellings must reach the advisor's model.
    #[test]
    fn an_unconfigured_seat_resolves_the_advisor_at_the_advisor_door() {
        for (label, config) in [
            ("absent", ADVISOR_ONLY),
            (
                "null",
                r#"{"models":{"claude":{"generation":"sonnet","advisor":"opus","lane-3":null}}}"#,
            ),
            (
                "shapeless",
                r#"{"models":{"claude":{"generation":"sonnet","advisor":"opus","lane-3":{"nonsense":1}}}}"#,
            ),
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let root = repo(&tmp, config);
            let v = envelope(&root, "advisor", Some("lane-3"));
            assert_eq!(v.get("ok"), None, "{label}: a seat must not be refused: {v}");
            assert_eq!(
                v.get("payload").and_then(|p| p.get("model")),
                Some(&json!("opus")),
                "{label}: the advisor's model, never the session model"
            );
            // D4 — the marker names the RESOLVED role, so the guard reading it
            // back sees a name this host configures.
            let prompt = v
                .get("payload")
                .and_then(|p| p.get("prompt"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            assert!(prompt.starts_with("[bee-tier: advisor]"), "{label}: {prompt}");
            let e = v.get("economics").unwrap();
            assert_eq!(e.get("logical_tier"), Some(&json!("advisor")), "{label}");
            // …and the seat that was ASKED for is on the record, or three
            // fallen-through lanes read as three plain advisor consults.
            assert_eq!(e.get("requested_role"), Some(&json!("lane-3")), "{label}");
        }
    }

    /// A CONFIGURED seat is the whole point: it resolves its OWN model and
    /// travels under its OWN name, so two lanes can run two models.
    #[test]
    fn a_configured_seat_resolves_its_own_model_and_names_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":"sonnet","advisor":"opus","lane-2":"fable","hat-risks":{"model":"haiku","description":"what could go wrong"}}}}"#,
        );
        let v = envelope(&root, "advisor", Some("lane-2"));
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("fable")));
        let e = v.get("economics").unwrap();
        assert_eq!(e.get("logical_tier"), Some(&json!("lane-2")));
        assert_eq!(e.get("requested_role"), None, "nothing fell through, so nothing to record");
        let prompt = v
            .get("payload")
            .and_then(|p| p.get("prompt"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert!(prompt.starts_with("[bee-tier: lane-2]"), "{prompt}");
        // An object-shaped hat carrying D3's description resolves exactly like
        // any other object slot: the description is display-only and the
        // resolver never sees it.
        let v = envelope(&root, "advisor", Some("hat-risks"));
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("haiku")));
        assert_eq!(
            v.get("economics").and_then(|e| e.get("logical_tier")),
            Some(&json!("hat-risks"))
        );
        // A string-shaped hat carries no description and STILL resolves —
        // doctor's advisory flags it, the door does not.
        let tmp2 = tempfile::tempdir().unwrap();
        let bare = repo(
            &tmp2,
            r#"{"models":{"claude":{"generation":"sonnet","advisor":"opus","hat-risks":"fable"}}}"#,
        );
        let v = envelope(&bare, "advisor", Some("hat-risks"));
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("fable")));
        assert_eq!(
            v.get("economics").and_then(|e| e.get("requested_role")),
            None,
            "a described-or-not configured seat is configured"
        );
    }

    /// P2-2 — case folding, in both directions. `--role Lane-2` must resolve a
    /// configured `lane-2` (the config's spelling travels), and fall through
    /// when the seat is unconfigured. An exact-match seat test would have made
    /// the mixed-case spelling a stranger and refused it, which is the
    /// one-question-two-answers defect this codebase already closed twice.
    #[test]
    fn a_mixed_case_seat_resolves_and_falls_through_like_its_lowercase_spelling() {
        let tmp = tempfile::tempdir().unwrap();
        let on = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":"sonnet","advisor":"opus","lane-2":"fable"}}}"#,
        );
        let v = envelope(&on, "advisor", Some("Lane-2"));
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("fable")));
        assert_eq!(
            v.get("economics").and_then(|e| e.get("logical_tier")),
            Some(&json!("lane-2")),
            "the CONFIG's spelling travels, never the caller's"
        );

        let tmp2 = tempfile::tempdir().unwrap();
        let off = repo(&tmp2, ADVISOR_ONLY);
        let v = envelope(&off, "advisor", Some("Lane-2"));
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("opus")));
        assert_eq!(
            v.get("economics").and_then(|e| e.get("requested_role")),
            Some(&json!("lane-2")),
            "the record names the seat in the constant's own spelling"
        );
    }

    /// The fall-through is bounded on BOTH axes it claims to be bounded on.
    ///
    /// By KIND: every non-advisor kind keeps T012a's refusal, so an
    /// unconfigured seat cannot borrow the advisor's model through a `gather`
    /// or a `reviewer` dispatch.
    ///
    /// By NAME: a typo'd seat is refused on the advisor kind too. Falling
    /// through for any unconfigured name would let `hat-risk` run silently on
    /// the advisor model, which is the outcome the refusal exists to prevent.
    #[test]
    fn the_seat_fall_through_is_bounded_by_kind_and_by_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, ADVISOR_ONLY);
        for kind in ["gather", "reviewer"] {
            for seat in SEAT_ROLES {
                let v = envelope(&root, kind, Some(seat));
                assert_eq!(
                    v.get("reason"),
                    Some(&json!("role_not_configured")),
                    "{kind}/{seat}: T012a is unchanged off the advisor door"
                );
                assert_eq!(v.get("role"), Some(&json!(seat)), "{kind}/{seat}");
            }
        }
        for typo in ["hat-risk", "lane-4", "lanes-1", "hat-value-add"] {
            let v = envelope(&root, "advisor", Some(typo));
            assert_eq!(
                v.get("reason"),
                Some(&json!("role_not_configured")),
                "{typo} is not a seat, so it is refused rather than resolved"
            );
        }
    }

    /// P3-2 — when the advisor is ALSO off, the seat's fall-through lands on
    /// the advisor's own refusal and says so by its own name. Reporting
    /// `role_not_configured` here would send the operator to configure the
    /// seat when what the host actually lacks is the advisor.
    #[test]
    fn a_seat_falling_through_to_a_missing_advisor_refuses_as_the_advisor() {
        for config in [
            r#"{"models":{"claude":{"generation":"sonnet"}}}"#,
            r#"{"models":{"claude":{"generation":"sonnet","advisor":null}}}"#,
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let root = repo(&tmp, config);
            let v = envelope(&root, "advisor", Some("lane-1"));
            assert_eq!(v.get("ok"), Some(&json!(false)), "{config}");
            assert_eq!(
                v.get("reason"),
                Some(&json!("advisor_not_configured")),
                "{config}: the missing thing is the advisor, and the refusal names it"
            );
            let fix = v.get("fix").and_then(Value::as_str).unwrap_or_default();
            assert!(fix.contains("models.claude.advisor"), "{fix}");
        }
    }

    /// Nothing moved for anyone who never names a seat. A `--kind advisor`
    /// dispatch with no `--role`, and every other kind's default path, are
    /// byte-identical to the pre-seat spelling.
    #[test]
    fn a_dispatch_that_names_no_seat_is_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, ADVISOR_ONLY);
        for kind in DISPATCH_KINDS {
            if kind == "cell" {
                continue; // a cell dispatch needs a claimed cell; covered above
            }
            let v = envelope(&root, kind, None);
            let e = v.get("economics").unwrap();
            assert_eq!(e.get("requested_role"), None, "{kind}: no seat, no key");
        }
        let v = envelope(&root, "advisor", None);
        assert_eq!(v.get("payload").and_then(|p| p.get("model")), Some(&json!("opus")));
        assert_eq!(
            v.get("economics").and_then(|e| e.get("tier_source")),
            Some(&json!("default"))
        );
    }
}
