// the models config layer
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

// ═══ models config (lib/state.mjs) ═════════════════════════════════════════

pub(crate) const EFFORT_LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

// opencode-support E4/S4: opencode joins claude/codex as a real `models.<rt>`
// key (docs/config-reference.md) rather than the silently-ignored third key
// it used to be — every function below is already keyed generically off this
// list (`models.get(rt)`, no struct field per runtime), so widening it is the
// whole fix for this reader.
// pi-support D5/D6: `pi` joins them for the same reason and with the same one
// line of change — `models.pi` is a real role table in the ONE config home,
// read generically like every other key. What is pi-specific lives at the
// dispatch door (`prepare::pi_requires_herding_refusal`), never here: this
// reader's job is "does the operator's config name this runtime", and a
// runtime absent from this list would silently resolve the CLAUDE table
// instead (see the `RUNTIMES.contains` fallbacks below).
pub(crate) const RUNTIMES: [&str; 4] = ["claude", "codex", "opencode", "pi"];

/// The names bee itself ships a built-in default for.
///
/// model-role-split D2 (store 06e49368) RETIRED this list as a membership
/// test: a role name is legal because `models.<runtime>` carries it, never
/// because it appears here. It gates neither normalization nor resolution any
/// more — `normalize_models` carries every key the config names, and
/// `resolve_role` asks "is this name configured", not "is this name one of
/// four words". What survives is its one honest job: naming the slots
/// `default_models` fills in, which is also the floor the last entry of a
/// role list resolves against. The tier-shaped readers left in `prepare.rs`
/// still import it; sweeping those is mrs-4's cell.
///
/// MODEL_NORMALIZE_SLOTS (this list plus `advisor`) is gone with the same
/// decision — it existed ONLY to bound the normalize overlay, which now walks
/// whatever the config names. `verbs/status_full/mod.rs` keeps a private copy
/// of the old four names for its own display path.
pub(crate) const CONFIGURABLE_SLOTS: [&str; 3] = ["extraction", "generation", "review"];

/// The role names BEE ITSELF asks for and seeds into a fresh config, but
/// ships no built-in model for.
///
/// model-role-split D3 (store 3c9d6262) / 561e1bda: every ordered role list
/// bee's own dispatch sites ask for ENDS with a historical name, and the
/// entries before that tail are `code` (cell execution) and `read` (a read
/// dispatch). mrs-10 seeds both into a FRESH host's config; a host that
/// onboarded earlier carries neither key, and neither has a `default_models`
/// entry — which is exactly the membership `warn_unknown_role` reads as "this
/// name is a typo".
///
/// So this list is a WARN-SUPPRESSION list and nothing else. It gates no
/// normalization and no resolution: adding either name to `default_models`
/// instead would seed it into the table `normalize_models` builds, where a
/// MID-list `code` would then RESOLVE and quietly outrank the `generation`
/// model every existing host has configured for years — the silent migration
/// 561e1bda exists to prevent. Absent from here, `code` would warn on every
/// single cell dispatch on every existing host, and a warning that always
/// fires is a warning nobody reads; a name the OPERATOR invented (`test`,
/// `design`) is in no table at all and still warns loudly.
///
/// mrs-24 (store `fef79243`) NARROWED that suppression to the case that
/// earned it. It is a MIGRATION WINDOW, not a standing exemption: it applies
/// only on a runtime whose table configures NEITHER of these names — a host
/// from before the roles existed, where falling through to the historical
/// name is the intended no-op upgrade and there is no better-fitting model to
/// have picked. The first asked name an operator configures closes the window
/// for that runtime, so a HALF-migrated config is loud about the sibling it
/// missed instead of silent about it forever. That is the line between
/// suppressing a warning nobody could act on and suppressing D2's
/// (store `06e49368`) whole point.
pub(crate) const ASKED_ROLES: [&str; 2] = ["code", "read"];

/// Has this runtime opted into bee's own role names at all?
///
/// The one question that bounds the `ASKED_ROLES` migration window. Asked of
/// the runtime's own table — never of `default_models`, which ships no entry
/// for either name on any runtime, so a positive answer can only ever mean
/// "the operator, or mrs-10's fresh-config seed, put it there".
pub(crate) fn host_opted_into_roles(models: &Map<String, Value>, runtime: &str) -> bool {
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    let Some(table) = models.get(rt) else { return false };
    ASKED_ROLES.iter().any(|name| table.get(name).is_some())
}

/// provenance: state.mjs DEFAULT_MODELS.
pub(crate) fn default_models(runtime: &str) -> Map<String, Value> {
    let mut m = Map::new();
    if runtime == "claude" {
        m.insert("extraction".into(), Value::String("haiku".into()));
        m.insert("generation".into(), Value::String("sonnet".into()));
        m.insert("review".into(), Value::String("opus".into()));
    } else {
        m.insert("extraction".into(), Value::Null);
        m.insert("generation".into(), Value::Null);
        m.insert("review".into(), Value::Null);
    }
    m
}

pub(crate) fn is_plain_object(v: &Value) -> bool {
    matches!(v, Value::Object(_))
}

/// provenance: state.mjs normalizeTierValue. `None` == JS `undefined` (the
/// slot keeps its default); `Some(Value::Null)` == an explicit null slot.
pub(crate) fn normalize_tier_value(value: Option<&Value>) -> Option<Value> {
    let value = value?;
    match value {
        Value::String(s) if !js_trim(s).is_empty() => {
            return Some(Value::String(js_trim(s).to_string()))
        }
        Value::String(_) => return None,
        Value::Null => return Some(Value::Null),
        v if !is_plain_object(v) => return None,
        _ => {}
    }
    let obj = value.as_object().unwrap();
    // { kind: 'cli', command }
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "cli") {
        if let Some(Value::String(cmd)) = obj.get("command") {
            if !js_trim(cmd).is_empty() {
                let mut out = Map::new();
                out.insert("kind".into(), Value::String("cli".into()));
                out.insert("command".into(), Value::String(js_trim(cmd).to_string()));
                return Some(Value::Object(out));
            }
        }
    }
    // { kind: 'native', model, effort?, fork_turns?, agent_type? }
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "native") {
        if let Some(Value::String(model)) = obj.get("model") {
            if !js_trim(model).is_empty() {
                let mut out = Map::new();
                out.insert("kind".into(), Value::String("native".into()));
                out.insert("model".into(), Value::String(js_trim(model).to_string()));
                if let Some(Value::String(e)) = obj.get("effort") {
                    if EFFORT_LEVELS.contains(&js_trim(e)) {
                        out.insert("effort".into(), Value::String(js_trim(e).to_string()));
                    }
                }
                if let Some(Value::String(f)) = obj.get("fork_turns") {
                    if js_trim(f) == "none" {
                        out.insert("fork_turns".into(), Value::String("none".into()));
                    }
                }
                if let Some(Value::String(a)) = obj.get("agent_type") {
                    if !js_trim(a).is_empty() {
                        out.insert("agent_type".into(), Value::String(js_trim(a).to_string()));
                    }
                }
                return Some(Value::Object(out));
            }
        }
    }
    // { kind: 'herding', agent?, fallback? } — a router value, no other
    // fields required; unknown extras (e.g. a stray `command`) are dropped,
    // same as cli/native. herd-registry D2: `agent` names a
    // `herding.agents` registry entry by name — trimmed, empty/whitespace
    // dropped (same rule as every other string field on this leaf).
    // herding-review-slots D3: `fallback` recognizes exactly one value,
    // the literal string "default" — anything else (empty, mistyped,
    // non-string) is dropped, same exact-match posture as `fork_turns`
    // above.
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "herding") {
        let mut out = Map::new();
        out.insert("kind".into(), Value::String("herding".into()));
        if let Some(Value::String(a)) = obj.get("agent") {
            if !js_trim(a).is_empty() {
                out.insert("agent".into(), Value::String(js_trim(a).to_string()));
            }
        }
        if matches!(obj.get("fallback"), Some(Value::String(f)) if f == "default") {
            out.insert("fallback".into(), Value::String("default".into()));
        }
        return Some(Value::Object(out));
    }
    // Explicit-fallback composite: { primary: {kind:'native', model}, ... }
    if let Some(primary) = obj.get("primary") {
        if is_plain_object(primary) {
            let p = primary.as_object().unwrap();
            let native_primary = matches!(p.get("kind"), Some(Value::String(k)) if k == "native")
                && matches!(p.get("model"), Some(Value::String(m)) if !js_trim(m).is_empty());
            if native_primary {
                let mut out = Map::new();
                out.insert("primary".into(), normalize_tier_value(Some(primary))?);
                if matches!(obj.get("fallback_policy"), Some(Value::String(s)) if s == "explicit-only") {
                    out.insert("fallback_policy".into(), Value::String("explicit-only".into()));
                    if let Some(fb) = obj.get("fallback") {
                        if is_plain_object(fb) {
                            let f = fb.as_object().unwrap();
                            let cli = matches!(f.get("kind"), Some(Value::String(k)) if k == "cli");
                            if let (true, Some(Value::String(cmd))) = (cli, f.get("command")) {
                                if !js_trim(cmd).is_empty() {
                                    let mut fbo = Map::new();
                                    fbo.insert("kind".into(), Value::String("cli".into()));
                                    fbo.insert(
                                        "command".into(),
                                        Value::String(js_trim(cmd).to_string()),
                                    );
                                    out.insert("fallback".into(), Value::Object(fbo));
                                }
                            }
                        }
                    }
                }
                return Some(Value::Object(out));
            }
        }
    }
    // { model, effort? } — only when `kind` is absent.
    if obj.get("kind").is_none() {
        if let Some(Value::String(model)) = obj.get("model") {
            if !js_trim(model).is_empty() {
                let mut out = Map::new();
                out.insert("model".into(), Value::String(js_trim(model).to_string()));
                if let Some(Value::String(e)) = obj.get("effort") {
                    if EFFORT_LEVELS.contains(&js_trim(e)) {
                        out.insert("effort".into(), Value::String(js_trim(e).to_string()));
                    }
                }
                return Some(Value::Object(out));
            }
        }
    }
    None
}

/// provenance: state.mjs normalizeModels — defaults per runtime, overlaid by
/// the normalized value of every key the config names under that runtime.
///
/// model-role-split D2 (store 06e49368): the overlay used to walk
/// MODEL_NORMALIZE_SLOTS, so a config naming any other role
/// (`models.claude.test`) was read and then silently DROPPED — the role could
/// never resolve, and nothing said why. The open role set starts here: every
/// key whose value normalizes into a documented leaf shape is carried, and a
/// key whose value is junk is dropped exactly as a junk `generation` was.
pub(crate) fn normalize_models(raw: Option<&Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for rt in RUNTIMES {
        out.insert(rt.to_string(), Value::Object(default_models(rt)));
    }
    if let Some(raw) = raw {
        if is_plain_object(raw) {
            for rt in RUNTIMES {
                let Some(src) = raw.get(rt) else { continue };
                let Some(src) = src.as_object() else { continue };
                for (slot, raw_value) in src {
                    if let Some(value) = normalize_tier_value(Some(raw_value)) {
                        out.get_mut(rt)
                            .and_then(Value::as_object_mut)
                            .unwrap()
                            .insert(slot.clone(), value);
                    }
                }
            }
        }
    }
    out
}

/// The `models` slice of readConfig(root). Delegates on the ONE readConfig
/// side effect this port still does not reproduce: normalizeDogfoodRepos'
/// per-dead-repo console.warn. (A corrupt config no longer delegates — it
/// warns and reads as "no config", readJson's own fallback.)
pub(crate) fn read_models(root: &Path) -> D<Map<String, Value>> {
    let config = read_config_raw(root);
    if let Some(Value::Array(items)) = config.get("dogfood_repos") {
        if !items.is_empty() {
            return Err(Delegate); // normalizeDogfoodRepos may warn to stderr
        }
    }
    Ok(normalize_models(config.get("models")))
}

/// provenance: state.mjs resolveTier / resolveAdvisor return shapes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Resolved {
    Inherit,
    Model {
        model: String,
        effort: Option<String>,
    },
    Budget,
    Cli {
        command: String,
    },
    Native {
        model: String,
        effort: Option<String>,
        fork_turns: String,
        agent_type: String,
        fallback: Option<String>,
    },
    Refused {
        slot: String,
    },
    /// herding-tier D1/D3, herding-review-slots D1 (widened to the full
    /// mapping): `{kind:"herding"}` on ANY slot/purpose — cell, gather,
    /// reviewer, advisor, extraction — turns into the `bee herding run`
    /// Bash payload (dispatch prepare's herding-exec arm); the operator
    /// owns the pane cost per slot.
    /// herd-registry D2: `agent` carries the optional `herding.agents` name
    /// named on the slot (`{kind:"herding", agent:"<name>"}`); prepare's
    /// herding-exec arm appends `--agent "<name>"` when present.
    /// herding-review-slots D3: `fallback` mirrors the normalized
    /// `"fallback": "default"` field verbatim (`Some("default".into())`) —
    /// dispatch prepare reads it to decide whether to add the payload's
    /// `fallback` object; absent when the slot never named a fallback.
    Herding { agent: Option<String>, fallback: Option<String> },
}

pub(crate) const CLI_REFUSAL_FIX: &str = "declare {for:\"gather\"} for a read-only gather; cli cell execution stays refused until a cell-execution dogfood is green (plan 2A/W9)";

/// provenance: state.mjs nativeResolved — normalize already trimmed/validated
/// the leaf; this only applies the resolved defaults.
pub(crate) fn native_resolved(value: &Map<String, Value>, fallback: Option<String>) -> Resolved {
    Resolved::Native {
        model: match value.get("model") {
            Some(Value::String(s)) => s.clone(),
            other => tpl(other),
        },
        effort: match value.get("effort") {
            None | Some(Value::Null) => None,
            Some(v) => Some(jsjson::js_to_string(v)),
        },
        fork_turns: match value.get("fork_turns") {
            None | Some(Value::Null) => "none".to_string(),
            Some(v) => jsjson::js_to_string(v),
        },
        agent_type: match value.get("agent_type") {
            None | Some(Value::Null) => "worker".to_string(),
            Some(v) => jsjson::js_to_string(v),
        },
        fallback,
    }
}

/// The composite `{primary, fallback_policy:'explicit-only', fallback}` arm
/// shared by resolveTier and resolveAdvisor.
pub(crate) fn composite_resolved(obj: &Map<String, Value>) -> Option<Resolved> {
    let primary = obj.get("primary")?;
    if !is_plain_object(primary) {
        return None;
    }
    let mut fallback = None;
    if matches!(obj.get("fallback_policy"), Some(Value::String(s)) if s == "explicit-only") {
        if let Some(fb) = obj.get("fallback") {
            if matches!(fb.get("kind"), Some(Value::String(k)) if k == "cli") {
                if let Some(Value::String(cmd)) = fb.get("command") {
                    fallback = Some(cmd.clone());
                }
            }
        }
    }
    Some(native_resolved(primary.as_object().unwrap(), fallback))
}

/// One name's own configuration, resolved. `None` means this name carries
/// NOTHING to resolve — it is unset, explicitly null, or shaped like nothing
/// bee documents — and the caller yields to the next name in its list.
///
/// `kind` is the dispatch-prepare purpose ("cell" | "gather" | "reviewer" |
/// "advisor" — DISPATCH_KINDS); the cli branch gates on
/// `purpose_is_gather(kind)`, byte-identical to before. herding-review-slots
/// D1 (widened to the full mapping): the herding branch has no gate on `kind`
/// at all — every purpose reads the same herding-shaped slot the same way.
fn resolve_configured(value: &Value, name: &str, kind: &str) -> Option<Resolved> {
    if value.is_null() {
        return None;
    }
    if let Value::String(model) = value {
        return Some(Resolved::Model { model: model.clone(), effort: None });
    }
    let obj = value.as_object()?;
    // cli purpose gate — unchanged: refused for a cell-execution dispatch,
    // served for gather/reviewer/advisor exactly as before. The refusal names
    // the slot actually READ, which under a role walk is the entry that
    // carried the cli value rather than the entry that was asked for.
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "cli") {
        if !purpose_is_gather(kind) {
            return Some(Resolved::Refused { slot: name.to_string() });
        }
        return Some(Resolved::Cli {
            command: truthy_str(obj.get("command")).unwrap_or_default().to_string(),
        });
    }
    // herding-review-slots D1 (widened to the full mapping): EVERY purpose
    // — cell, gather, reviewer, advisor — on a `{kind:"herding"}` slot
    // routes to the herding-exec pane (ht-3/hrv-1/hrv-3 build that payload).
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "herding") {
        let agent = match obj.get("agent") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        };
        let fallback = match obj.get("fallback") {
            Some(Value::String(f)) if f == "default" => Some(f.clone()),
            _ => None,
        };
        return Some(Resolved::Herding { agent, fallback });
    }
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "native") {
        return Some(native_resolved(obj, None));
    }
    if let Some(r) = composite_resolved(obj) {
        return Some(r);
    }
    if let Some(Value::String(model)) = obj.get("model") {
        return Some(Resolved::Model {
            model: model.clone(),
            effort: match obj.get("effort") {
                Some(v) if truthy(v) => Some(jsjson::js_to_string(v)),
                _ => None,
            },
        });
    }
    // An object matching no documented shape resolves nothing; the walk
    // yields, and a last entry lands on Budget.
    None
}

/// A name bee has never heard of — absent from `models.<runtime>` AND from
/// the built-in defaults — is the typo case, and the ONE thing it must never
/// do is quietly hand back some other role's model. It says so, on stderr,
/// naming what it fell through to.
///
/// A name that IS present but null or unresolvable is deliberately unset (or
/// null by built-in default, as every codex slot ships), not a mistake, so it
/// yields without a word. Keeping those two apart is what stops a
/// per-dispatch warning storm on an unconfigured runtime while leaving the
/// misspelling loud.
/// The exact question `resolve_role_named` asks before it warns — a name
/// nothing has heard of: absent from `models.<runtime>` and absent from the
/// built-in defaults, with ONE bounded exception.
///
/// Public because the warn itself goes to stderr, which an in-process test
/// cannot read: this is how "no dispatch on an existing host warns routinely"
/// is provable over a REAL config and the real ordered lists, rather than by
/// re-asserting the constant against itself. One home, one answer — the
/// resolver below asks this same function.
///
/// mrs-24 (store `fef79243`): the `ASKED_ROLES` exception is a MIGRATION
/// WINDOW and is therefore tested LAST — after the configured check, and only
/// against a runtime that has opted into neither asked name. The order is the
/// fix. Asking "is this name configured" first is what makes a half-migrated
/// host loud about the sibling key it missed, and what keeps this function's
/// answer a fact about THIS host instead of a fact about two hard-coded
/// words. The FIX text in `verbs/cells/validate.rs` states this one silent
/// case out loud rather than promising a warn bee does not give.
pub(crate) fn role_is_unknown(models: &Map<String, Value>, runtime: &str, name: &str) -> bool {
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    if models.get(rt).and_then(|t| t.get(name)).is_some() || default_models(rt).contains_key(name) {
        return false;
    }
    // The pre-roles window. Every ordered list bee asks for puts `code` or
    // `read` ahead of the historical name, so warning here would fire on
    // every dispatch of an un-migrated host — over bee's own plumbing, not
    // over anything the operator wrote. Configure either name and the window
    // shuts: from then on the missing sibling warns like any other name.
    !(ASKED_ROLES.contains(&name) && !host_opted_into_roles(models, rt))
}

fn warn_unknown_role(name: &str, runtime: &str, next: Option<&str>) {
    let tail = match next {
        Some(next) => format!(" — falling through to \"{next}\""),
        None => " — nothing after it in the list, so no model is selected".to_string(),
    };
    eprintln!(
        "bee: model role \"{name}\" is not configured in models.{runtime} of .bee/config.json{tail}"
    );
}

/// D5 (store `97ce5225`) — the escalation word, in ONE place.
///
/// It is not a role and not a tier: it is the marker `dispatch prepare`
/// stamps on an escalated dispatch (`[bee-tier: ceiling]`) and the model
/// guard reads back to grant a session-model run with no `model` parameter.
/// The cell-side spelling of the same fact is the boolean escalation flag
/// (`verbs::cells::ESCALATE_FIELD`); this constant is what keeps the two
/// halves naming one word instead of two literals drifting apart.
pub(crate) const ESCALATION_WORD: &str = "ceiling";

/// Resolve an ORDERED LIST of role names against `models.<runtime>`.
///
/// model-role-split D2 (store 06e49368). The consumer names the roles it will
/// accept, best first; the first name that carries a resolvable configuration
/// wins; an unset or unresolvable name yields to the next. The LAST entry
/// always resolves, because it falls back to the runtime's built-in
/// `default_models` entry and then to `Resolved::Budget` — so a walk cannot
/// dead-end, and an empty list cannot panic (it is Budget: no name asked, no
/// model selected).
///
/// What this replaces is the coercion that stood at the old `resolve_tier`'s
/// third line: `if CONFIGURABLE_SLOTS.contains(&slot) { slot } else
/// { "generation" }`. Under a closed four-word set that read as a harmless
/// normalization; under D2's open set it is a wrong-model dispatch that
/// completes clean — a cell whose role is `tset` would have been handed the
/// generation model while `prepare.rs` stamped `tier_source: "cell"`, so the
/// record asserted the cell had chosen it. No name may resolve a model the
/// config does not carry for that name, and no name bee does not know
/// resolves silently.
///
/// Falling through on an ABSENT configuration is not a downgrade: decision
/// `72f3d6dd` licenses a fallback "ONLY when that tier is unconfigured", and
/// a configured role is still obeyed exactly.
pub(crate) fn resolve_role(
    models: &Map<String, Value>,
    roles: &[&str],
    runtime: &str,
    kind: &str,
) -> Resolved {
    resolve_role_named(models, roles, runtime, kind).1
}

/// `resolve_role` plus the ONE fact its caller cannot recompute afterwards:
/// WHICH name in the list actually won.
///
/// model-role-split D3 (store 3c9d6262) puts the cell's own role at the head
/// of the list, so the name a dispatch ASKS for and the name that RESOLVES
/// are no longer the same word — a cell declaring `role: "test"` on a host
/// that configures no `test` runs on the `generation` model. The
/// `[bee-tier: <role>]` marker that dispatch stamps has to name the resolved
/// one: `hooks/model_guard.rs` classifies the marker against `known_roles`
/// and DENIES any name nothing configures, so a marker carrying the
/// unresolved head would refuse every cell dispatch on every host that has
/// not opted into the new names — a louder spelling of the same silent
/// migration 561e1bda's tail exists to prevent.
///
/// It is the same walk, never a second one: `resolve_role` is this function
/// with the name dropped, so the two cannot drift.
pub(crate) fn resolve_role_named<'a>(
    models: &Map<String, Value>,
    roles: &[&'a str],
    runtime: &str,
    kind: &str,
) -> (Option<&'a str>, Resolved) {
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    let table = models.get(rt);
    for (i, name) in roles.iter().enumerate() {
        let name = *name;
        // No carve-out stands here any more. `ceiling` used to short-circuit
        // this walk to `Resolved::Inherit` (decision 0015), which made the
        // "open" role set carry exactly one closed word. D5 (store
        // `97ce5225`) removes the word from this axis entirely: escalation is
        // a FLAG on the cell, never a role, so the open set needs no
        // exception and a cell that declares `role: "ceiling"` is just a role
        // nothing configures — it warns and falls through like any other.
        // The escalation word survives ONE layer up, in `resolve_tier`, as
        // the tier-shaped marker `[bee-tier: ceiling]` that `dispatch
        // prepare` stamps and `hooks/model_guard.rs` reads back.
        //
        // REVIEW P1-A: that paragraph described the intent and not the
        // shipped code for one release — `prepare.rs` short-circuited on
        // `tier_token == ESCALATION_WORD` BEFORE this walk was reached, and
        // on the cell-role path `tier_token` is the cell's declared role, so
        // a cell saying `role: "ceiling"` never arrived here at all. The
        // caller now guards that arm with `!from_role`, and the promise is
        // held by test rather than by comment:
        // `verbs::drivers::tests::the_ration_and_the_dispatch_agree_on_which_cells_are_escalated`
        // asserts the ration's predicate and the dispatch's channel agree
        // for every marking a cell can carry.
        let entry = table.and_then(|t| t.get(name));
        if let Some(resolved) = entry.and_then(|v| resolve_configured(v, name, kind)) {
            return (Some(name), resolved);
        }
        let defaults = default_models(rt);
        // `ASKED_ROLES`: bee's OWN tail names (`code`, `read`) are absent from
        // both tables on every host that onboarded before mrs-10, and warning
        // on them would fire on every dispatch — see the constant. An
        // operator-invented name still warns.
        if role_is_unknown(models, rt, name) {
            warn_unknown_role(name, rt, roles.get(i + 1).copied());
        }
        if i + 1 == roles.len() {
            // The floor, so the walk cannot dead-end. `default_models` is
            // consulted ONLY for a name the table does not carry at all: a
            // present-but-null entry is a slot somebody turned OFF (or one
            // `default_models` itself seeded null, as every codex slot is),
            // and answering it with the built-in default would resurrect the
            // very model the config just cleared. Absent and refused are not
            // the same read.
            if entry.is_none() {
                return (
                    Some(name),
                    defaults
                        .get(name)
                        .and_then(|v| resolve_configured(v, name, kind))
                        .unwrap_or(Resolved::Budget),
                );
            }
            return (Some(name), Resolved::Budget);
        }
    }
    // An empty list: no name asked, so no name won and no model is selected.
    (None, Resolved::Budget)
}

/// provenance: state.mjs resolveTier(root, slot, runtime, purpose) — kept as
/// the single-name spelling of `resolve_role` for the tier-shaped callers
/// mrs-3 (the guard) and mrs-4 (`slot_for_kind`) still have to sweep. It
/// carries no resolution logic of its own, so the D2 fix above reaches every
/// one of them today rather than waiting for the sweep.
///
/// The one rule it keeps as a list: an unset `review` yields to `generation`,
/// which used to be a special case inside the resolver and is now just what
/// fall-through means.
pub(crate) fn resolve_tier(
    models: &Map<String, Value>,
    slot: &str,
    runtime: &str,
    kind: &str,
) -> Resolved {
    // D5 (store `97ce5225`) — the ESCALATION WORD, and the one layer that
    // still knows it. `ceiling` is not a role and never resolves a model: it
    // means "run on the session model", which `Resolved::Inherit` is the
    // spelling of. It lives HERE rather than in `resolve_role_named` because
    // the callers that can still hand it over are the tier-shaped ones — the
    // model guard classifying a `[bee-tier: ceiling]` marker
    // (`hooks/model_guard.rs`) and `verbs/drivers/guard.rs`'s economics
    // audit — and that marker is the wire word between `dispatch prepare`
    // and the guard, not a name any config carries.
    if slot == ESCALATION_WORD {
        return Resolved::Inherit;
    }
    resolve_role(models, &tier_role_list(slot), runtime, kind)
}

/// The ordered list a SINGLE-SLOT (tier-shaped) caller asks for — today's
/// exact bytes, kept in ONE place so `resolve_tier` and the caller that needs
/// the resolved NAME back (`prepare.rs`) ask the identical question. 561e1bda
/// names this list for the review consumer: `[review, generation]`. A second
/// hand-written copy beside this one is exactly the drift D1 collapsed the
/// two parsers to remove.
///
/// gather-reads-the-read-slot D1 (decision 295f3d03) adds the SECOND rule:
/// `read` — the slot `--kind gather` resolves — walks `[read, generation]`.
/// NOT through `extraction`. Extraction was the tier era's cheapest slot and
/// never the gather slot, so a legacy host that configures `extraction` +
/// `generation` and no `read` must keep its gathers on `generation` byte for
/// byte; a walk through `extraction` would move every one of them onto the
/// cheapest model silently — the exact defect 561e1bda's tail rule exists to
/// prevent. The read-shaped CELL list (`prepare::cell_role_list("read")`) DOES
/// pass through `extraction`, because a read-shaped cell is a backfilled
/// `tier: extraction` record; the two lists differ on purpose and each says
/// why at its own door.
pub(crate) fn tier_role_list(slot: &str) -> Vec<&str> {
    if slot == "review" || slot == "read" {
        return vec![slot, "generation"];
    }
    vec![slot]
}

/// The parallel-advisor SEATS, as role names — lane-model-diversity D1
/// (store `23de5362`).
///
/// A blind-lane run fires 2–3 lanes at once and a hat wave fires five hats at
/// once; before this they all resolved the ONE `advisor` slot, so every
/// parallel brainstorm agent ran the same model and the diversity the fan-out
/// exists for was thrown away at the door. Each seat now has its own name in
/// the ONE `models.<runtime>` table (D1 keeps the single config home,
/// `4a6e38be`), so an operator can point lane-2 at a different model from
/// lane-1 by writing one key.
///
/// It lives HERE, beside `tier_role_list`, for the same reason that list does:
/// the names a consumer asks for belong with the resolver that answers them,
/// never hand-copied at each door. `skills/bee-hive/references/
/// gates-and-delegation.md` cites this constant as the list of record (D5).
///
/// CLOSED on purpose, and that is the whole safety of D2's fall-through
/// below: only these eight names may resolve the advisor's model when their
/// own slot is empty. A typo (`hat-risk`) is not a seat, so it keeps T012a's
/// refusal rather than silently running on the advisor model.
pub(crate) const SEAT_ROLES: [&str; 8] = [
    "lane-1",
    "lane-2",
    "lane-3",
    "hat-facts-gaps",
    "hat-risks",
    "hat-value",
    "hat-alternatives",
    "hat-user-impact",
];

/// The prefix that marks a seat as a HAT rather than a lane.
///
/// D3 asks a hat slot — and only a hat slot — to carry a `description`: the
/// five hats have distinct purposes an operator has to be able to read back,
/// while lanes are interchangeable seats with no per-seat purpose to state.
/// One home for the prefix so `bee doctor`'s advisory and this list cannot
/// drift on what counts as a hat.
pub(crate) const HAT_ROLE_PREFIX: &str = "hat-";

/// The SEAT this name spells, in `SEAT_ROLES`' own spelling — `None` when the
/// name is no seat at all.
///
/// Case-folded, because every other role door in this codebase already is:
/// `known_role_named` and `role_is_declarable` both match a role name with
/// `eq_ignore_ascii_case`, so an exact-match test here would make `--role
/// Lane-2` a seat at one door and a stranger at the next — the one-question-
/// two-answers shape `3c9d6262` and `role-edge-hardening` D1 each closed once
/// already.
pub(crate) fn seat_role_named(name: &str) -> Option<&'static str> {
    SEAT_ROLES.iter().copied().find(|seat| seat.eq_ignore_ascii_case(name))
}

/// Does this role's own slot carry ANYTHING a dispatch could resolve?
///
/// The exact question D2's fall-through turns on, and it is deliberately not
/// "is the key present". Three configurations mean the same thing to a
/// dispatch — the key is absent, the key is explicitly `null`, or the value is
/// an object matching no documented shape — and all three leave the seat with
/// no model of its own. `resolve_role_named` would answer the last two with
/// its `Resolved::Budget` FLOOR (the last entry in a walk always resolves), so
/// a seat spelled `"lane-3": null` would have inherited the session model
/// instead of falling through to the advisor: the fall-through would have had
/// a hole in exactly the spelling `.bee/config-sample.json` teaches for "off".
///
/// Asking `resolve_configured` — the one leaf parser — rather than re-testing
/// the value here is what keeps this door's answer identical to the door that
/// resolves. The key is found case-insensitively for the reason
/// `resolve_advisor` finds its own key that way.
pub(crate) fn role_slot_resolves(
    models: &Map<String, Value>,
    runtime: &str,
    name: &str,
    kind: &str,
) -> bool {
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    models
        .get(rt)
        .and_then(Value::as_object)
        .and_then(|t| t.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)))
        .and_then(|(k, v)| resolve_configured(v, k, kind))
        .is_some()
}

/// provenance: state.mjs resolveAdvisor — NEVER budget, NEVER a tier fallback;
/// `None` unambiguously means "no advisor".
///
/// D1 (store `cd72ec97`) — COLLAPSED onto the one parser (mrs-24). What made
/// this a standalone reader was never the value shapes: it read the identical
/// string / cli / herding / native / composite / `{model, effort}` set that
/// `resolve_configured` reads, in a different order, and the two had already
/// been edited in lockstep twice (herding-tier D1, then herding-review-slots
/// D1/D2) — the exact lockstep D1 exists to end. What is genuinely the
/// advisor's own is the LIST it walks, and that list is one name with no
/// fall-through and no `Budget` floor (decision `4faf1de9`): the advisor
/// deliberately does not go through `resolve_role`, which always resolves its
/// last entry. So this is the one-name walk spelled out, and the leaf parse
/// is shared.
///
/// `Resolved::Refused` cannot come back: that arm fires only for a cli slot
/// on a cell-execution purpose (`purpose_is_gather`), and an advisor consult
/// is a gather purpose by definition.
pub(crate) fn resolve_advisor(models: &Map<String, Value>, runtime: &str) -> Option<Resolved> {
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    // role-edge-hardening D1: the key is found case-insensitively, matching
    // `role_is_declarable`'s advisor arm — an exact `get("advisor")` read
    // made a mis-cased "Advisor" key resolve at the marker door (which folds
    // case) while `--kind advisor` refused, one question with two answers.
    let value = models.get(rt).and_then(Value::as_object).and_then(|t| {
        t.iter().find(|(k, _)| k.eq_ignore_ascii_case("advisor")).map(|(_, v)| v)
    })?;
    resolve_configured(value, "advisor", "advisor")
}

// ═══ the runtime fallback chain (model-role-split D10/D11) ═════════════════
//
// WHAT THIS LAYER IS, AND WHAT IT IS NOT. Decision 51341f84 scopes store
// 50808d48 for this codebase: bee never executes a dispatch. `dispatch
// prepare` builds a payload and RETURNS; the orchestrator or the worker runs
// it. So bee cannot observe the quota wall, the 5xx or the stream stall a
// chain step answers, and nothing below is a retry loop, an error classifier
// or any code that decides a step has been earned. What bee owns is the
// CONTRACT: parse the config, resolve the chain that applies to THIS
// dispatch, and publish it — with the gate that says when a step is
// earned — beside the model on the payload. Advancing a step, and recording
// the step that was taken, belong wherever the dispatch is actually executed.
//
// The shape is the one `prepare.rs` already uses for the herding slot's
// `fallback` (herding-review-slots D3) and `fallback_when` (af17e217): the
// condition travels WITH the fallback, never as a rule the caller re-derives.

/// D11 (store 50808d48) — the error classes that MAY advance a chain step.
/// Transient and infrastructural, every one of them: the failure happened
/// BEFORE the model got to be wrong.
pub(crate) const CHAIN_ADVANCE_ON: [&str; 6] = [
    "quota_or_rate_limit",
    "provider_auth_or_policy_rejection",
    "empty_response",
    "malformed_tool_call_replay_safe",
    "stream_stall_or_connection_reset",
    "server_error_5xx",
];

/// D11 — the classes that may NEVER advance a step. Every one is a SEMANTIC
/// failure: the model was reached, answered, and answered badly. Falling to
/// another model there would hide the defect, which is the one thing bee's
/// loud posture exists to refuse. Published as its own list rather than left
/// as "everything not in `advance_on`", because the negative is the half a
/// caller gets wrong.
pub(crate) const CHAIN_NEVER_ADVANCE_ON: [&str; 4] =
    ["tool_error", "wrong_or_unwanted_result", "failed_proof", "red_test"];

/// The condition, in one line, carried beside the chain — the `fallback_when`
/// precedent (af17e217) applied to D11's gate.
pub(crate) const CHAIN_FALLBACK_WHEN: &str = "the dispatch failed with one of advance_on; a never_advance_on failure stays loud and never advances a step";

fn chain_class_list(classes: &[&str]) -> Value {
    Value::Array(classes.iter().map(|c| Value::String((*c).to_string())).collect())
}

fn warn_fallback_chain(key: &str, why: &str) {
    eprintln!("bee: retry.fallbackChains[\"{key}\"] in .bee/config.json is ignored — {why}");
}

/// Parse and validate `retry.fallbackChains`: a map whose KEY names a role, a
/// concrete model selector, or a `provider/*` wildcard, and whose VALUE is an
/// ordered list of model selectors.
///
/// EXPLICIT-ONLY (D10). There is no built-in chain for any role, so this
/// function answers an absent config with an EMPTY map and every dispatch
/// payload stays byte-identical to a bee that had never heard of chains. A
/// `default` key is refused out loud for the same reason: the source product
/// this shape is adapted from lets every role inherit a `default` chain, and
/// inheriting one is exactly what D10 declined — it would change advisor
/// behaviour that decision 4faf1de9 settled by live evidence, without the
/// owner asking.
///
/// Junk drops rather than throws, the same posture `normalize_tier_value`
/// holds one screen up — but it drops LOUDLY here. A mistyped chain is not a
/// slot somebody deliberately turned off; it is a safety net the operator
/// believes is under them.
pub(crate) fn normalize_fallback_chains(raw: Option<&Value>) -> Map<String, Value> {
    let mut out = Map::new();
    let Some(Value::Object(src)) = raw else { return out };
    for (raw_key, value) in src {
        let key = js_trim(raw_key);
        if key.is_empty() {
            continue;
        }
        if key == "default" {
            warn_fallback_chain(
                key,
                "bee has no default chain and no role inherits one (model-role-split D10); key a chain by role name, by concrete model selector, or by a \"provider/*\" wildcard",
            );
            continue;
        }
        let Some(items) = value.as_array() else {
            warn_fallback_chain(key, "a chain must be an ordered ARRAY of model selectors");
            continue;
        };
        let mut steps: Vec<Value> = Vec::new();
        for item in items {
            let Value::String(step) = item else { continue };
            let step = js_trim(step);
            // A step naming the key's own model is not a step: a chain that
            // loops to its own head would have the executor "advance" onto
            // the model that just hit the wall.
            if step.is_empty() || step == key {
                continue;
            }
            if steps.iter().any(|s| s.as_str() == Some(step)) {
                continue;
            }
            steps.push(Value::String(step.to_string()));
        }
        if steps.is_empty() {
            warn_fallback_chain(key, "it names no model selector bee can use as a step");
            continue;
        }
        out.insert(key.to_string(), Value::Array(steps));
    }
    out
}

/// The `retry.fallbackChains` slice of the repo config. Reads the raw config
/// directly rather than through `read_models`, so a repo shape that makes
/// `read_models` delegate is not this reader's business, and an absent
/// `retry` key costs one map lookup.
pub(crate) fn read_fallback_chains(root: &Path) -> Map<String, Value> {
    let config = read_config_raw(root);
    let chains = normalize_fallback_chains(config.get("retry").and_then(|r| r.get("fallbackChains")));
    // role-edge-hardening D1: a chain key that names neither a wildcard, a
    // configured role, nor any model a dispatch can actually resolve is
    // DEAD — resolve_fallback_chain matches on the RESOLVED role and model,
    // so a role-keyed chain for a role nothing configures can never fire.
    // The chain still publishes (junk-drop stays for structural junk only);
    // the operator just hears about it instead of nothing.
    if !chains.is_empty() {
        let models = normalize_models(config.get("models"));
        for key in dead_chain_keys(&chains, &models) {
            warn_fallback_chain(
                &key,
                "no runtime configures a role or model by this name, so no dispatch ever travels under it and the chain can never fire — key it by a configured role, the concrete model that would fail, or a \"provider/*\" wildcard",
            );
        }
    }
    chains
}

/// role-edge-hardening D1: the chain keys no dispatch can ever travel under.
/// `resolve_fallback_chain` matches on the RESOLVED role and model, so a key
/// is live only when it is a `provider/*` wildcard, a role name some runtime
/// configures or seeds by default, or a model string some slot resolves to.
/// Everything else is accepted, published, and matches nothing — this is the
/// one word the operator gets about it.
pub(crate) fn dead_chain_keys(
    chains: &Map<String, Value>,
    models: &Map<String, Value>,
) -> Vec<String> {
    let mut live: std::collections::HashSet<String> = std::collections::HashSet::new();
    for rt in RUNTIMES {
        for (name, value) in default_models(rt)
            .iter()
            .chain(models.get(rt).and_then(Value::as_object).into_iter().flatten())
        {
            live.insert(name.to_string());
            if let Value::String(m) = value {
                live.insert(js_trim(m).to_string());
            }
            if let Some(m) = value.get("model").and_then(Value::as_str) {
                live.insert(js_trim(m).to_string());
            }
        }
    }
    chains
        .keys()
        .filter(|k| !k.contains('/') && !live.contains(k.as_str()))
        .cloned()
        .collect()
}

/// WHICH chain applies to THIS dispatch — most specific key first.
///
/// 1. the concrete model selector this dispatch carries,
/// 2. the `provider/*` wildcard that selector falls under,
/// 3. the role the dispatch travels under.
///
/// A model-keyed chain outranks a role-keyed one because D10 says a
/// model-keyed chain "follows that model wherever it is assigned": it is
/// keyed on the thing that actually failed, and it survives the model being
/// reassigned to another role. A wildcard sits between the two — it is the
/// same model axis, one step wider.
///
/// The FIRST key that matches answers, and the walk stops there even when its
/// steps clean away to nothing (a chain naming only the model already in
/// hand). Continuing would let a broader key overrule the more specific one
/// the operator wrote, which is the opposite of "most specific wins".
pub(crate) fn resolve_fallback_chain(
    chains: &Map<String, Value>,
    role: &str,
    model: &str,
) -> Option<(String, Vec<String>)> {
    if chains.is_empty() {
        return None;
    }
    let model = js_trim(model);
    let role = js_trim(role);
    let mut keys: Vec<String> = Vec::new();
    if !model.is_empty() {
        keys.push(model.to_string());
        if let Some((provider, _)) = model.split_once('/') {
            if !provider.is_empty() {
                keys.push(format!("{provider}/*"));
            }
        }
    }
    if !role.is_empty() && !keys.iter().any(|k| k == role) {
        keys.push(role.to_string());
    }
    for key in keys {
        let Some(Value::Array(items)) = chains.get(&key) else { continue };
        let steps: Vec<String> = items
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|s| *s != model)
            .map(|s| s.to_string())
            .collect();
        return if steps.is_empty() { None } else { Some((key, steps)) };
    }
    None
}

/// The published contract, as one payload field: the chain, the key it was
/// resolved by, and D11's gate in both directions. The executor is TOLD which
/// failures may advance a step and which may not — an unpublished rule is one
/// every caller invents differently.
pub(crate) fn fallback_chain_payload(key: &str, steps: &[String]) -> Value {
    let mut out = Map::new();
    out.insert("key".into(), Value::String(key.to_string()));
    out.insert(
        "chain".into(),
        Value::Array(steps.iter().map(|s| Value::String(s.clone())).collect()),
    );
    out.insert("fallback_when".into(), Value::String(CHAIN_FALLBACK_WHEN.to_string()));
    out.insert("advance_on".into(), chain_class_list(&CHAIN_ADVANCE_ON));
    out.insert("never_advance_on".into(), chain_class_list(&CHAIN_NEVER_ADVANCE_ON));
    Value::Object(out)
}
