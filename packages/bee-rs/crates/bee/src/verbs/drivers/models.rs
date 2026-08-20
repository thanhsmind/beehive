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

/// CONFIGURABLE_SLOTS = [...CONFIGURABLE_TIERS, 'review'].
pub(crate) const CONFIGURABLE_SLOTS: [&str; 3] = ["extraction", "generation", "review"];

/// MODEL_NORMALIZE_SLOTS = [...CONFIGURABLE_SLOTS, 'advisor'].
pub(crate) const MODEL_NORMALIZE_SLOTS: [&str; 4] = ["extraction", "generation", "review", "advisor"];

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
    // { kind: 'herding', agent? } — a router value, no other fields
    // required; unknown extras (e.g. a stray `command`) are dropped, same
    // as cli/native. herd-registry D2: `agent` names a `herding.agents`
    // registry entry by name — trimmed, empty/whitespace dropped (same rule
    // as every other string field on this leaf).
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "herding") {
        let mut out = Map::new();
        out.insert("kind".into(), Value::String("herding".into()));
        if let Some(Value::String(a)) = obj.get("agent") {
            if !js_trim(a).is_empty() {
                out.insert("agent".into(), Value::String(js_trim(a).to_string()));
            }
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
/// the normalized value of each MODEL_NORMALIZE_SLOTS entry.
pub(crate) fn normalize_models(raw: Option<&Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for rt in RUNTIMES {
        out.insert(rt.to_string(), Value::Object(default_models(rt)));
    }
    if let Some(raw) = raw {
        if is_plain_object(raw) {
            for rt in RUNTIMES {
                let Some(src) = raw.get(rt) else { continue };
                if !is_plain_object(src) {
                    continue;
                }
                for slot in MODEL_NORMALIZE_SLOTS {
                    if let Some(value) = normalize_tier_value(src.get(slot)) {
                        out.get_mut(rt)
                            .and_then(Value::as_object_mut)
                            .unwrap()
                            .insert(slot.to_string(), value);
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
    /// herding-tier D1/D3, herding-review-slots D1/D2: `{kind:"herding"}` on
    /// a cell, reviewer, or advisor purpose slot — the dispatch seam (ht-3)
    /// turns this into the `bee herding run` Bash payload for every one of
    /// those three purposes. Never produced for a gather purpose (D3/D1
    /// route those to the runtime's default model instead).
    /// herd-registry D2: `agent` carries the optional `herding.agents` name
    /// named on the slot (`{kind:"herding", agent:"<name>"}`); prepare's
    /// herding-exec arm appends `--agent "<name>"` when present.
    Herding { agent: Option<String> },
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

/// provenance: state.mjs resolveTier(root, slot, runtime, purpose). `slot`
/// here is always a CONFIGURABLE_SLOTS member or 'advisor' (coerced to
/// 'generation' exactly like Node); `kind` is the dispatch-prepare purpose
/// ("cell" | "gather" | "reviewer" | "advisor" — DISPATCH_KINDS). The cli
/// branch below still gates on `purpose_is_gather(kind)`, byte-identical to
/// before; herding-review-slots D1/D2 widens the herding branch to route on
/// `kind` directly, since "cell purpose" and "herding purpose" are no
/// longer the same question.
pub(crate) fn resolve_tier(
    models: &Map<String, Value>,
    slot: &str,
    runtime: &str,
    kind: &str,
) -> Resolved {
    if slot == "ceiling" {
        return Resolved::Inherit;
    }
    let rt = if RUNTIMES.contains(&runtime) { runtime } else { "claude" };
    let s = if CONFIGURABLE_SLOTS.contains(&slot) { slot } else { "generation" };
    let table = models.get(rt);
    let mut value = table.and_then(|t| t.get(s)).cloned();
    if matches!(value, None | Some(Value::Null)) && s == "review" {
        value = table.and_then(|t| t.get("generation")).cloned();
    }
    let Some(value) = value.filter(|v| !v.is_null()) else {
        return Resolved::Budget;
    };
    if let Value::String(model) = &value {
        return Resolved::Model { model: model.clone(), effort: None };
    }
    let Some(obj) = value.as_object() else { return Resolved::Budget };
    // cli purpose gate — unchanged: refused for a cell-execution dispatch,
    // served for gather/reviewer/advisor exactly as before.
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "cli") {
        if !purpose_is_gather(kind) {
            return Resolved::Refused { slot: s.to_string() };
        }
        return Resolved::Cli {
            command: truthy_str(obj.get("command")).unwrap_or_default().to_string(),
        };
    }
    // herding-review-slots D1/D2 (widens herding-tier D1/D3's cell-only
    // scope): cell, reviewer, and advisor purposes route to the
    // herding-exec pane (ht-3/hrv-1 build that payload); only a gather
    // purpose keeps serving the runtime's own default model for this slot
    // (never Herding, never a refusal). A runtime whose default for this
    // slot is null (codex/opencode) reads Budget, same as an unconfigured
    // slot always does.
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "herding") {
        if kind != "gather" {
            let agent = match obj.get("agent") {
                Some(Value::String(s)) => Some(s.clone()),
                _ => None,
            };
            return Resolved::Herding { agent };
        }
        return match default_models(rt).get(s).cloned().filter(|v| !v.is_null()) {
            Some(Value::String(model)) => Resolved::Model { model, effort: None },
            _ => Resolved::Budget,
        };
    }
    if matches!(obj.get("kind"), Some(Value::String(k)) if k == "native") {
        return native_resolved(obj, None);
    }
    if let Some(r) = composite_resolved(obj) {
        return r;
    }
    if let Some(Value::String(model)) = obj.get("model") {
        return Resolved::Model {
            model: model.clone(),
            effort: match obj.get("effort") {
                Some(v) if truthy(v) => Some(jsjson::js_to_string(v)),
                _ => None,
            },
        };
    }
    Resolved::Budget
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
        return Some(Resolved::Herding { agent });
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
