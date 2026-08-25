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
pub(crate) const RUNTIMES: [&str; 3] = ["claude", "codex", "opencode"];

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
pub(crate) const ASKED_ROLES: [&str; 2] = ["code", "read"];

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
/// nothing has heard of: absent from `models.<runtime>`, absent from the
/// built-in defaults, and not one of bee's own `ASKED_ROLES` tail names.
///
/// Public because the warn itself goes to stderr, which an in-process test
/// cannot read: this is how "no dispatch on an existing host warns routinely"
/// is provable over a REAL config and the real ordered lists, rather than by
/// re-asserting the constant against itself. One home, one answer — the
/// resolver below asks this same function.
pub(crate) fn role_is_unknown(models: &Map<String, Value>, runtime: &str, name: &str) -> bool {
    if ASKED_ROLES.contains(&name) {
        return false;
    }
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    models.get(rt).and_then(|t| t.get(name)).is_none() && !default_models(rt).contains_key(name)
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
        // decision 0015: `ceiling` is the escalation word, never a slot — it
        // is not configurable, so it terminates the walk on its own.
        if name == "ceiling" {
            return (Some(name), Resolved::Inherit);
        }
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
    resolve_role(models, &tier_role_list(slot), runtime, kind)
}

/// The ordered list a SINGLE-SLOT (tier-shaped) caller asks for — today's
/// exact bytes, kept in ONE place so `resolve_tier` and the caller that needs
/// the resolved NAME back (`prepare.rs`) ask the identical question. 561e1bda
/// names this list for the review consumer: `[review, generation]`. A second
/// hand-written copy beside this one is exactly the drift D1 collapsed the
/// two parsers to remove.
pub(crate) fn tier_role_list(slot: &str) -> Vec<&str> {
    if slot == "review" {
        return vec![slot, "generation"];
    }
    vec![slot]
}

/// provenance: state.mjs resolveAdvisor — NEVER budget, NEVER a tier fallback;
/// `None` unambiguously means "no advisor".
pub(crate) fn resolve_advisor(models: &Map<String, Value>, runtime: &str) -> Option<Resolved> {
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    let value = models.get(rt).and_then(|t| t.get("advisor"))?;
    if value.is_null() {
        return None;
    }
    if let Value::String(model) = value {
        return Some(Resolved::Model { model: model.clone(), effort: None });
    }
    let obj = value.as_object()?;
    // herding-review-slots D1/D2: an advisor is one task in, one result out
    // — the same shape as the herding-exec pane's own read-only job — so a
    // herding-shaped advisor slot now resolves to Resolved::Herding
    // (widening herding-tier D1's cell-only scope) instead of reading as
    // "no advisor".
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
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "cli") {
        return Some(Resolved::Cli {
            command: match obj.get("command") {
                Some(Value::String(c)) => c.clone(),
                _ => return None, // `{type:'cli', command: undefined}` never reaches here post-normalize
            },
        });
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
    None
}
