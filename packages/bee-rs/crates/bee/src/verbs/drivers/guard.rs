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

/// Every role name a dispatch on this runtime may legally declare.
///
/// DERIVED, never listed (model-role-split D2): the keys `models.<runtime>`
/// carries after `normalize_models` — the operator's own roles plus the
/// built-in defaults bee seeds there — and `ceiling`, which decision 0015
/// keeps out of config on purpose and which `resolve_tier` answers with
/// `Resolved::Inherit`. Every entry is a name something in bee can publish,
/// so the set cannot drift from the resolver the way the two deleted tier
/// lists drifted from each other.
///
/// It lives HERE rather than in the model-guard because both doors ask it
/// (T012a, store 8ff6e79e): the hook classifies a `[bee-tier: <name>]`
/// marker with it, and `bee dispatch prepare --role <name>` refuses with it.
/// Two copies of "is this role legal" is the defect this feature removes.
///
/// # Why the dispatch-door slots are NOT unioned in
///
/// This set used to add every `slot_for_kind` answer over `DISPATCH_KINDS`
/// unconditionally, so `advisor` was legal on a runtime that configured no
/// advisor. Two of those three slots (`generation`, `review`) are seeded into
/// every runtime table by `default_models`, so the union only ever ADDED one
/// name — `advisor` — and adding it was wrong in exactly the shape this
/// feature exists to close: `[bee-tier: advisor]` classified as
/// `Marker::Role` on a host with no advisor, skipped the unconfigured-role
/// refusal, resolved `Resolved::Budget`, and the subagent silently inherited
/// the session model — verbatim the outcome that refusal's own text says it
/// prevents. `bee dispatch prepare --role advisor` refused on the same host
/// (`resolve_advisor` never falls back — decision `4faf1de9`), so ONE question
/// had TWO answers through the two doors that share this predicate: the defect
/// D1 collapsed the parsers to remove, reappearing one layer up.
///
/// A slot is a legal role name when the host CONFIGURES it, and nothing else.
/// "Configures" is a question about the VALUE and not about the key, which is
/// what `role_is_declarable` below answers — the one place a null slot is told
/// apart from an absent one for a role NAME. `--kind advisor` never passes
/// through here at all (it resolves its slot through `slot_for_kind`), and it
/// no longer has to: both spellings now read a null advisor the same way.
pub(crate) fn known_roles(
    models: &Map<String, Value>,
    runtime: &str,
) -> std::collections::BTreeSet<String> {
    let mut set = std::collections::BTreeSet::new();
    if let Some(Value::Object(table)) = models.get(runtime) {
        set.extend(
            table.keys().filter(|name| role_is_declarable(models, runtime, name)).cloned(),
        );
    }
    set.insert(ESCALATION_WORD.to_string());
    set
}

/// The advisor slot, as a NAME rather than as a literal repeated at each site
/// that has to know the one role bee resolves through its own walk.
pub(crate) const ADVISOR_ROLE: &str = "advisor";

/// May a dispatch on this runtime legally DECLARE this name?
///
/// THE ONE PLACE null-versus-absent is decided for a role name — asked by
/// `known_roles` above and therefore by BOTH doors at once, rather than
/// re-tested at each reader. This feature has now been bitten three times by
/// that distinction being made ad hoc (the pre-roles migration window, the
/// dispatch-door slot union, and this), and every one of them was a second
/// reader inventing its own answer.
///
/// The rule is neither "the key is present" nor "the value is non-null". It
/// is: **the name is declarable when the resolution a dispatch would actually
/// perform for it yields something.**
///
/// For every ordinary role that answer is yes with a null value as with a
/// string one, because `resolve_role_named` ends its walk on a
/// `Resolved::Budget` FLOOR. That floor is the documented meaning of an unset
/// slot (`.bee/config-sample.json`: "A missing/null slot = prompt-budget — the
/// marker alone carries the role"), and on codex it is the ONLY meaning there
/// is: `default_models("codex")` seeds every one of that runtime's slots null,
/// so a blanket non-null test here would refuse every codex dispatch bee makes.
///
/// The advisor is the one name with NO floor (decision `4faf1de9`):
/// `resolve_advisor` walks one name, never falls through, and answers `None`
/// for a null value exactly as it does for an absent key. That is what made
/// ONE question get TWO answers on a host spelling the advisor off the way bee
/// itself documents (`"advisor": null` — "Set null to skip the advisor line"):
/// `--role advisor` and `--kind advisor` refused `advisor_not_configured`,
/// while `[bee-tier: advisor]` classified as `Marker::Role`, skipped the
/// unconfigured-role refusal, resolved `Resolved::Budget` and let the subagent
/// inherit the session model — verbatim the outcome that refusal's own text
/// says it prevents. `c2ef2f9f` closed the ABSENT case by dropping the
/// dispatch-door slot union; this closes the NULL case, which is the spelling
/// `.bee/config-sample.json` teaches.
///
/// Asking `resolve_advisor` rather than re-testing the value here is the whole
/// point: the advisor's null-versus-absent rule stays in exactly one place —
/// its own resolver — so this door cannot drift from the door that resolves.
pub(crate) fn role_is_declarable(models: &Map<String, Value>, runtime: &str, name: &str) -> bool {
    // role-edge-hardening D1: the advisor arm matches like the doors around
    // it — `known_role_named` folds case, so an exact-match arm here let a
    // mis-cased "Advisor" key fall to `contains_key`, enter `known_roles`,
    // and answer differently at the marker door than at `--kind advisor`.
    if name.eq_ignore_ascii_case(ADVISOR_ROLE) {
        return resolve_advisor(models, runtime).is_some();
    }
    models.get(runtime).and_then(Value::as_object).is_some_and(|t| t.contains_key(name))
}

/// The CONFIG's own spelling of a declared role name — `None` when this
/// runtime configures no such role.
///
/// THE predicate both doors ask, answer and all. `known_roles` says which
/// names are legal; this says whether ONE name is, and hands back the spelling
/// the config carries so every downstream read (`resolve_role`, the audit
/// line, the FIX text) gets a key it can look up.
///
/// Case-insensitive, because the marker door has always been: `[BEE-TIER:
/// Generation]` declares the `generation` role. `bee dispatch prepare --role`
/// used to ask a plain `contains` instead, so `--role Generation` was refused
/// at the door while `[bee-tier: Generation]` was admitted and resolved — one
/// typo, two answers, through the two doors that share this predicate
/// precisely so that cannot happen. Sharing the ANSWER and not just the SET is
/// what makes the sharing true.
pub(crate) fn known_role_named(
    models: &Map<String, Value>,
    runtime: &str,
    name: &str,
) -> Option<String> {
    known_roles(models, runtime).into_iter().find(|k| k.eq_ignore_ascii_case(name))
}

/// The configured roles as one FIX-line fragment.
pub(crate) fn role_list(models: &Map<String, Value>, runtime: &str) -> String {
    known_roles(models, runtime).into_iter().collect::<Vec<_>>().join("/")
}

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

/// What a rendered bee agent DOES, in one clause — the sentence a refusal
/// needs when it has to tell a caller which of a role's agents to name.
///
/// provenance: each agent template's own `description` frontmatter
/// (`packages/bee/agents/<agent>.md.tmpl`). It is keyed on the AGENT, never on
/// a role, so it cannot drift the way the agent-to-role tables drifted: adding
/// a spelling or a role changes nothing here, and an agent with no clause
/// answers `None` and is named bare rather than described wrongly.
pub(crate) fn agent_job_summary(agent: &str) -> Option<&'static str> {
    match agent {
        "bee-build" => Some("executes a cell (reserves, writes, commits, caps)"),
        "bee-gather" => Some("reads and reports (never writes)"),
        "bee-extract" => Some("pulls one already-scoped fact and reports (never writes)"),
        "bee-review" => Some("checks a claim read-only and reports (never edits)"),
        _ => None,
    }
}

/// The two spellings of one JOB: the word an operator now writes, and the
/// historical word the table above is keyed on.
///
/// mrs-29, from an independent audit of this feature. A rendered bee agent
/// serves a JOB — bee-extract reads, bee-build writes — and while the
/// historical cost words were the only vocabulary bee spoke, keying the table
/// on them made a role-to-agent lookup total. It is not any more: `read` and
/// `code` are what `default_config` seeds into a fresh host
/// (`onboard/templates.rs`), what D8's recommended vocabulary teaches, and
/// what `cell_role_list` puts at the HEAD of every ordered list bee's own
/// dispatch sites ask for. Both vocabularies stay legal indefinitely — the
/// historical name is the deliberate TAIL of those same lists, so no existing
/// host's upgrade moves it onto a different model — so a lookup that
/// understands only one of them is wrong for somebody either way.
///
/// What that cost, measured on the release binary before this fix, is that
/// ONE request got TWO answers decided by the host's migration state rather
/// than by the request: `dispatch prepare --runtime claude --kind gather
/// --role read` returned `subagent_type: "general-purpose"` on a host seeded
/// by today's `default_config`, while the same call on a pre-roles host was
/// refused `role_not_configured` and only `--role extraction` reached
/// `bee-extract` there. One rendered read agent, reachable only through
/// whichever word the host happened to speak.
///
/// So the LOOKUP keys on the job, not on the spelling. `read` and
/// `extraction` are one job; `code` and `generation` are one job. `review` is
/// already a job word and needs no second spelling.
///
/// The TABLE is deliberately left alone. Its rows stay agent-unique because
/// `role_for_agent` is the inverse lookup and its answer is fed straight back
/// into `resolve_tier` (`hooks/model_guard.rs`'s pinned-type branch), where
/// it must keep naming the role that resolves a MODEL on every host —
/// migrated or not — and the historical name is the one that does. Aliasing
/// is a property of the question "which agent serves this job", never of the
/// answer "which role was this agent rendered from".
pub(crate) const ROLE_ALIASES: [(&str, &str); 2] =
    [("read", "extraction"), ("code", "generation")];

/// The name `ROLE_AGENTS` keys a job on, given either of the job's spellings.
/// A name that is nobody's alias is its own key, so an operator-invented role
/// (`test`, `design`) reaches the table exactly as it did.
pub(crate) fn canonical_role(role: &str) -> &str {
    ROLE_ALIASES
        .iter()
        .find(|(job, _)| *job == role)
        .map(|(_, keyed)| *keyed)
        .unwrap_or(role)
}

/// The rendered bee agent a role is served by — `None` when the role has none
/// of its own.
///
/// `None` is a LEGAL answer, never a missing one. Under D2's open role set
/// most roles a host can configure (`test`, `docs`, `design`, and `advisor`
/// as shipped) have no rendered agent file at all, so answering one — or
/// falling back to a generic type — would name an agent that does not exist.
///
/// mrs-29: the role is normalized to its job first, so both spellings of one
/// job answer with the one agent that serves it. `code` answers `bee-gather`
/// for the same reason `generation` does — the FIRST entry, the read-only one,
/// is the safe answer when nothing else in the dispatch says which of the two
/// generation agents is meant, and `--kind cell` is still the one signal that
/// can say so before the lookup is ever reached.
pub(crate) fn agent_for_role(role: &str) -> Option<&'static str> {
    agents_for_role(role).first().copied()
}

/// EVERY rendered bee agent that serves a role's job, in `ROLE_AGENTS` order.
///
/// The question `agent_for_role` cannot answer: how MANY agents a role has.
/// `generation` has two — bee-gather reads, bee-build writes — so a dispatch
/// naming the role and `general-purpose` has stated no agent at all, and the
/// guard refuses rather than guessing (`hooks/model_guard.rs`, the pinned-type
/// rule). That refusal used to be keyed on the literal `"generation"`, which
/// every freshly onboarded host now spells `code`: the alias walked past the
/// check and was repaired onto the FIRST entry, the read-only agent, so an
/// execution dispatch died later at the write guard with the audit line naming
/// the wrong agent. Asking the table how many agents serve the job cannot go
/// stale that way — not when a spelling is added, and not when a third agent
/// is rendered for a role that has one today.
pub(crate) fn agents_for_role(role: &str) -> Vec<&'static str> {
    let key = canonical_role(role);
    ROLE_AGENTS.iter().filter(|(r, _)| *r == key).map(|(_, agent)| *agent).collect()
}

/// The role a rendered bee agent stands for. These files are generated FROM
/// the role's configured model at onboarding, so naming one IS a role
/// declaration in every sense that matters.
///
/// NOT alias-normalized, and that is the point of the split: this answer is
/// resolved against `models.<runtime>` by the caller, so it has to be the
/// name the agent was rendered from — the historical one every host still
/// carries. See `ROLE_ALIASES`.
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
///
/// `logical_tier` KEEPS ITS NAME under the model-role split (D4/D6), and that
/// is a decision rather than an oversight. The VALUE has already moved — it
/// carries the declared ROLE now, which is why the tests below assert
/// `logical_tier: "test"` and `logical_tier: "review"` beside the older
/// `"generation"`. The key does not follow it, for three reasons:
///
/// 1. Its destination is `.bee/logs/dispatch.jsonl`, an APPEND-ONLY log.
///    Renaming the key splits that file into two schemas with no version
///    marker, so every reader — `docs/decisions/ab-tiny-protocol.md`'s
///    measurement, documented at `bee-swarming/references/swarming-reference.md`
///    — would have to handle both spellings forever. One key that spans the
///    whole log is strictly easier to read correctly than two.
/// 2. There is a SECOND writer of this key, `hooks/model_guard.rs:994`.
///    Renaming here and not there splits one field into two; renaming both
///    is a change to a file this cell does not own, made for tidiness.
/// 3. "logical" always meant "what the dispatch DECLARED", as against the
///    `effective_model` it was actually observed to get. A role is exactly
///    that declaration, so the word is still true of its contents.
///
/// Consumers, named so a later rename is a decision and not a surprise:
/// `.bee/logs/dispatch.jsonl` readers, `hooks/model_guard.rs`'s verdict
/// output, and `verbs/drivers/tests.rs`'s economics assertions.
pub(crate) fn derive_economics(
    channel: &str,
    // The DECLARED role (or `ceiling`, the escalation word). Named for what it
    // now carries; the emitted key stays `logical_tier` — see above.
    role: &str,
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
    out.insert("logical_tier".into(), Value::String(role.to_string()));
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
/// slp-dissent-stop-and-ask sd-4: `status` is `Option<&str>` — `None` means
/// EVERY status, the same shape `verbs/cells/read.rs list_cells` already
/// uses. The dissent-debt door counts a dissent on a cell in ANY status (a
/// `blocker` dissent leaves it `blocked`, a `consider` dissent leaves it
/// `open` or `claimed`), so a single hard-coded status string would have made
/// that door blind to exactly the cells a blocker dissent parks.
pub(crate) fn list_cells(root: &Path, feature: &str, status: Option<&str>) -> D<Vec<Value>> {
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
        if let Some(status) = status {
            if !matches!(vget(&cell, "status"), Some(Value::String(s)) if s == status) {
                continue;
            }
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
pub(crate) fn list_cells_including_archive(root: &Path, feature: &str, status: Option<&str>) -> D<Vec<Value>> {
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
        if let Some(status) = status {
            if !matches!(vget(&cell, "status"), Some(Value::String(s)) if s == status) {
                continue;
            }
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
