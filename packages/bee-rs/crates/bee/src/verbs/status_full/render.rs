// renderStatusText
//
// Split out of the single 7k-line verbs/status_full.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, LinkedRoots, RootsWt};
use crate::state::{bypass_level, read_config_raw};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use crate::version::BEE_VERSION;

/// How many characters of `name=model` text the printed models line spends
/// before it stops naming roles and starts counting them.
///
/// model-role-split D2 (store 06e49368). The role set is OPEN, so this line
/// has no upper bound on entries and "print them all" is not automatically
/// the right answer: `bee status` is read at a glance and this is one line
/// of it. The number is MEASURED, not chosen — the longest role list bee
/// itself ships today is opencode's own seeded default (oc-14), which
/// renders 105 characters, so 120 keeps every default bee ships whole and
/// still leaves room for a role or two the operator added before it
/// truncates. Past it the line says `+N more` and points at
/// `bee status --json`, which carries every role unabridged; the human line
/// is a glance, the JSON is the record.
///
/// Bounding by WIDTH rather than by a count is what makes that promise hold,
/// and the difference is MEASURED on a real 12-role host, not argued:
///
/// * 12 claude roles (`generation=sonnet`, short names) render 153 characters
///   of role text — long, but no wall, and the budget trims it to 9 named
///   roles plus a count.
/// * the SAME 12 roles on opencode (`design=opencode/nemotron-3-ultra-free`)
///   render about 380, five wrapped terminal lines for one status row. The
///   budget renders 3 names plus `+9 more` at 143 characters total.
///
/// A count-based cap cannot tell those two apart, which is why this is not
/// one. The `+N more` suffix is kept SHORT for the same measured reason: at
/// its first spelling (`+N more (bee status --json for all)`) it cost 35
/// characters to drop 3 short claude entries worth 35 — a bound that bounded
/// nothing on exactly the host where names are cheap.
const MODELS_ROLE_LIST_BUDGET: usize = 120;

/// The roles one runtime configures, in the order the printed line names
/// them.
///
/// model-role-split D2 (store 06e49368): DERIVED from the config, never
/// listed. This line used to be a FIXED three-slot format string, so an
/// operator who followed this feature's own advice — add
/// `models.claude.test`, mark a cell `role: test` — saw `test` in
/// `bee status --json` and NOTHING in the line they actually read.
///
/// The ORDER is derived too, and it is load-bearing because the list can be
/// truncated: the slots bee's own dispatch door resolves come first, in
/// `DISPATCH_KINDS` order, so the names most dispatches land on can never be
/// the ones counted away; whatever else the host configures follows in map
/// order. That is the same rule the dispatch door itself orders by
/// (`hooks::model_guard::role_slot_display`) — one rule, two surfaces, so a
/// reader who compares them is comparing the same fact. It is also why the
/// printed order changed: `extraction` is no longer a slot any dispatch kind
/// resolves (`slot_for_kind` sends `gather` to `generation`), so it reads
/// with the rest instead of second.
///
/// `ceiling` is skipped: decision 0015 keeps it out of config and it names
/// the session model, not a role.
fn models_role_pairs(runtime_table: Option<&Value>) -> Vec<(String, String)> {
    use crate::verbs::drivers::{slot_for_kind, DISPATCH_KINDS, ESCALATION_WORD};
    let Some(Value::Object(table)) = runtime_table else { return Vec::new() };
    let mut order: Vec<&String> = Vec::new();
    for kind in DISPATCH_KINDS {
        if let Some(slot) = slot_for_kind(kind) {
            if let Some((name, _)) = table.get_key_value(slot) {
                if !order.iter().any(|n| n.as_str() == slot) {
                    order.push(name);
                }
            }
        }
    }
    for name in table.keys() {
        if name == ESCALATION_WORD || order.iter().any(|n| *n == name) {
            continue;
        }
        order.push(name);
    }
    order.into_iter().map(|name| (name.clone(), format_slot(table.get(name)))).collect()
}

/// The role list as the human line prints it — every configured role by name,
/// bounded by `MODELS_ROLE_LIST_BUDGET` and counting the remainder.
fn models_role_list(runtime_table: Option<&Value>) -> String {
    let pairs = models_role_pairs(runtime_table);
    if pairs.is_empty() {
        return "none configured".to_string();
    }
    let mut out = String::new();
    let mut shown = 0usize;
    for (name, value) in &pairs {
        let entry = format!("{name}={value}");
        // The first entry always prints: a budget must never render an empty
        // line, however long one model name is.
        if shown > 0 && out.chars().count() + 1 + entry.chars().count() > MODELS_ROLE_LIST_BUDGET {
            break;
        }
        if shown > 0 {
            out.push(' ');
        }
        out.push_str(&entry);
        shown += 1;
    }
    if shown < pairs.len() {
        out.push_str(&format!(" +{} more (--json)", pairs.len() - shown));
    }
    out
}

// ─── renderStatusText (bee.mjs ~1081-1206) ─────────────────────────────────

/// bee.mjs formatSlot.
pub(crate) fn format_slot(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "null".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(v) => {
            if str_eq(vget(v, "kind"), "cli") {
                let command = tpl(vget(v, "command"));
                let first = if command.starts_with(|c: char| c.is_whitespace()) {
                    ""
                } else {
                    command.split_whitespace().next().unwrap_or("")
                };
                return format!("cli({first})");
            }
            // herding-tier D1/D3: a router value, never a model name — the
            // dispatch seam turns this into the herding-exec Bash payload
            // for cell purpose; a gather on the same slot keeps the
            // runtime default (resolved before this display point).
            if str_eq(vget(v, "kind"), "herding") {
                return "herding".to_string();
            }
            if opt_truthy(vget(v, "model")) {
                let model = tpl(vget(v, "model"));
                if opt_truthy(vget(v, "effort")) {
                    return format!("{model}@{}", tpl(vget(v, "effort")));
                }
                return model;
            }
            "null".to_string()
        }
    }
}

/// bee.mjs formatLaneRow.
pub(crate) fn format_lane_row(l: &Value) -> String {
    let gates = GATE_NAMES
        .iter()
        .map(|g| {
            let approved = opt_truthy(vget(l, "approved_gates").and_then(|ag| vget(ag, g)));
            format!("{g}={}", if approved { "approved" } else { "pending" })
        })
        .collect::<Vec<_>>()
        .join(" ");
    let bound = match vget(l, "bound_sessions") {
        Some(Value::Array(items)) if !items.is_empty() => {
            format!(" sessions={}", js_join(items, ","))
        }
        _ => String::new(),
    };
    format!("{} [{}] {gates}{bound}", tpl(vget(l, "feature")), tpl(vget(l, "phase")))
}

/// bee.mjs formatLaneSummaryLine — None = no line at all.
pub(crate) fn format_lane_summary_line(summary: &Value) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if opt_truthy(vget(summary, "active")) {
        parts.push(format!("active: {}", format_lane_row(vget(summary, "active").unwrap())));
    }
    if let Some(Value::Array(ids)) = vget(summary, "ids") {
        if !ids.is_empty() {
            let counts_str = match vget(summary, "counts") {
                Some(Value::Object(counts)) => counts
                    .iter()
                    .map(|(phase, n)| format!("{phase}={}", jsjson::js_to_string(n)))
                    .collect::<Vec<_>>()
                    .join(" "),
                _ => String::new(),
            };
            parts.push(format!(
                "{} other lane(s) [{counts_str}] (ids: {})",
                ids.len(),
                js_join(ids, ", ")
            ));
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("Lanes: {}", parts.join(" | ")))
    }
}

pub(crate) fn render_status_text(status: &JMap) -> String {
    let s = |k: &str| status.get(k);
    let mut lines: Vec<String> = Vec::new();
    if opt_truthy(s("worktree_notice")) {
        lines.push(tpl(s("worktree_notice")));
    }
    lines.push(format!("bee status (plugin v{BEE_VERSION})"));
    {
        let onboarding = s("onboarding").cloned().unwrap_or(Value::Null);
        let installed = opt_truthy(vget(&onboarding, "installed"));
        let base = if installed {
            format!("installed (bee {})", tpl(vget(&onboarding, "bee_version")))
        } else {
            "MISSING".to_string()
        };
        let drift = if opt_truthy(vget(&onboarding, "drift")) {
            let detail = if opt_truthy(vget(&onboarding, "drift_detail")) {
                let n = vget(&onboarding, "drift_detail")
                    .and_then(|d| d.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                format!(": {n} file(s)")
            } else {
                String::new()
            };
            format!(" [drift{detail}]")
        } else {
            String::new()
        };
        lines.push(format!("Onboarding: {base}{drift}"));
    }
    lines.push(format!(
        "Phase: {} | Mode: {} | Feature: {}",
        tpl(s("phase")),
        if nullish(s("mode")) { "none".to_string() } else { tpl(s("mode")) },
        if nullish(s("feature")) { "none".to_string() } else { tpl(s("feature")) },
    ));
    lines.push(format!(
        "Gates: {}",
        GATE_NAMES
            .iter()
            .map(|g| {
                let approved = opt_truthy(s("gates").and_then(|gs| vget(gs, g)));
                format!("{g}={}", if approved { "approved" } else { "pending" })
            })
            .collect::<Vec<_>>()
            .join(" ")
    ));
    // D3/D7: one short line per gate whose persisted RECORD state is not
    // "approved" — distinguishing a gate nobody has acted on ("pending")
    // from one an actor refused ("rejected"), which the boolean line above
    // cannot: both show as "pending" there. Scoped to a live feature — an
    // idle repo's fallback default (every gate "pending", nobody having
    // acted) is not news worth a line.
    if opt_truthy(s("feature")) {
        if let Some(Value::Object(gate_records)) = s("gate_records") {
            for g in GATE_NAMES {
                let Some(entry) = gate_records.get(g) else { continue };
                let state = vget(entry, "state").and_then(|v| v.as_str()).unwrap_or("pending");
                if state == "approved" {
                    continue;
                }
                let mut line = format!("Gate {g}: {state}");
                if opt_truthy(vget(entry, "actor")) {
                    line.push_str(&format!(" actor={}", tpl(vget(entry, "actor"))));
                }
                if opt_truthy(vget(entry, "at")) {
                    line.push_str(&format!(" at={}", tpl(vget(entry, "at"))));
                }
                if opt_truthy(vget(entry, "bypass_level")) {
                    line.push_str(&format!(" bypass={}", tpl(vget(entry, "bypass_level"))));
                }
                if opt_truthy(vget(entry, "reason")) {
                    line.push_str(&format!(
                        " reason={}",
                        jsjson::stringify(vget(entry, "reason").unwrap())
                    ));
                }
                lines.push(line);
            }
        }
    }
    let level = s("gate_bypass_level");
    if opt_truthy(level) && !str_eq(level, "off") {
        lines.push(bypass_banner(&tpl(level)).to_string());
    }
    lines.push(format!(
        "Handoff: {}",
        if opt_truthy(s("handoff")) { "PRESENT — surface it and WAIT" } else { "none" }
    ));
    // D1 (awaiting-human): the live wait mark, one line naming WHAT the run
    // is waiting on (a gate name or the question asked) — silent when there
    // is none, matching Handoff's own "PRESENT | absent" shape just above.
    if opt_truthy(s("waiting_on")) {
        let w = s("waiting_on").unwrap();
        lines.push(format!(
            "Waiting on human: {} — {} (asked {})",
            tpl(vget(w, "kind")),
            tpl(vget(w, "subject")),
            tpl(vget(w, "asked_at")),
        ));
    }
    {
        let cells = s("cells").cloned().unwrap_or(Value::Null);
        let archived = vget(&cells, "archived").cloned().unwrap_or(Value::Null);
        let capped = vget(&cells, "capped").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
        let arch_capped = vget(&archived, "capped").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
        lines.push(format!(
            "Cells: open={} claimed={} capped={} blocked={} archived={} (total capped={})",
            tpl(vget(&cells, "open")),
            tpl(vget(&cells, "claimed")),
            tpl(vget(&cells, "capped")),
            tpl(vget(&cells, "blocked")),
            tpl(vget(&archived, "total")),
            jsjson::js_f64_to_string(capped + arch_capped),
        ));
    }
    match s("lanes") {
        Some(Value::Array(rows)) => {
            if !rows.is_empty() {
                lines.push(format!(
                    "Lanes: {}",
                    rows.iter().map(format_lane_row).collect::<Vec<_>>().join(" | ")
                ));
            }
        }
        Some(summary) => {
            if let Some(line) = format_lane_summary_line(summary) {
                lines.push(line);
            }
        }
        None => {}
    }
    let unreviewed = s("review")
        .and_then(|r| vget(r, "candidates"))
        .and_then(|c| vget(c, "unreviewed"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let phase_post_exec = s("phase")
        .and_then(|v| v.as_str())
        .map(|p| POST_EXECUTION_REVIEW_PHASES.contains(&p))
        .unwrap_or(false);
    if phase_post_exec && unreviewed > 0.0 {
        lines.push(format!(
            "Completed and verified; independent review not requested; {} candidate(s) awaiting review.",
            tpl(s("review").and_then(|r| vget(r, "candidates")).and_then(|c| vget(c, "unreviewed")))
        ));
    }
    {
        // The retirement nudge. `bee close` retires the feature it closes, so
        // this only ever speaks for work that never went through close — and
        // it stays quiet until the backlog is actually worth a command, since
        // a line that shows up at one stale feature is a line readers learn
        // to skip.
        let a = s("cells").and_then(|c| vget(c, "archivable"));
        let features = a.and_then(|v| vget(v, "features")).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let cells = a.and_then(|v| vget(v, "cells")).and_then(|v| v.as_f64()).unwrap_or(0.0);
        if features >= ARCHIVABLE_NUDGE_FLOOR {
            let ids = match a.and_then(|v| vget(v, "ids")) {
                Some(Value::Array(arr)) if !arr.is_empty() => {
                    let shown = js_join(arr, ", ");
                    if features > arr.len() as f64 {
                        format!(" ({shown}, …)")
                    } else {
                        format!(" ({shown})")
                    }
                }
                _ => String::new(),
            };
            lines.push(format!(
                "Finished features not retired: {} feature(s), {} cell(s) still in the active scan{ids} — every status and orient parses them. Retire: bee cells archive --all-but-active",
                tpl(a.and_then(|v| vget(v, "features"))),
                tpl(a.and_then(|v| vget(v, "cells"))),
            ));
            let _ = cells;
        }
    }
    {
        let sd = s("scribing_debt");
        if opt_truthy(sd) && vget(sd.unwrap(), "count").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0 {
            let sd = sd.unwrap();
            let cells = match vget(sd, "cells") {
                Some(Value::Array(a)) => js_join(a, ", "),
                _ => String::new(),
            };
            lines.push(format!(
                "Capture pending: {} behavior_change cell(s) uncaptured ({cells}) — run bee-capturing when you choose (decision c8e25271; batching features is fine)",
                tpl(vget(sd, "count"))
            ));
        }
    }
    {
        let cq = s("capture_queue");
        if opt_truthy(cq) && vget(cq.unwrap(), "count").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0 {
            lines.push(format!(
                "Capture queue pending: {} stub(s) awaiting flush — run bee-capturing when you choose (decision c8e25271), and before compact/clear",
                tpl(vget(cq.unwrap(), "count"))
            ));
        }
    }
    if opt_truthy(s("pbi")) {
        let pbi = s("pbi").unwrap();
        lines.push(format!(
            "PBI: {} done / {} in-flight / {} proposed",
            tpl(vget(pbi, "done")),
            tpl(vget(pbi, "in_flight")),
            tpl(vget(pbi, "proposed"))
        ));
    }
    {
        let commands = s("commands");
        let parts: Vec<String> = COMMAND_KEYS
            .iter()
            .filter(|key| opt_truthy(commands.and_then(|c| vget(c, key))))
            .map(|key| format!("{key}={}", tpl(commands.and_then(|c| vget(c, key)))))
            .collect();
        let joined = parts.join(" | ");
        lines.push(format!(
            "Standard commands: {}",
            if joined.is_empty() { "none recorded" } else { joined.as_str() }
        ));
    }
    lines.push(format!(
        "Active reservations: {}",
        s("active_reservations").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0)
    ));
    lines.push(format!(
        "Active workers: {}",
        s("workers").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0)
    ));
    if opt_truthy(s("contention")) {
        let c = s("contention").unwrap();
        let top = match vget(c, "top_locks") {
            Some(Value::Array(locks)) => locks
                .iter()
                .map(|l| format!("{}×{}", tpl(vget(l, "lock_name")), tpl(vget(l, "busy_count"))))
                .collect::<Vec<_>>()
                .join(", "),
            _ => String::new(),
        };
        let worst_lock = if opt_truthy(vget(c, "worst_wait_lock")) {
            format!(" on \"{}\"", tpl(vget(c, "worst_wait_lock")))
        } else {
            String::new()
        };
        lines.push(format!(
            "Contention: {} LOCK_BUSY event(s) recently (top: {top}); worst wait {}ms{worst_lock}",
            tpl(vget(c, "busy_count")),
            tpl(vget(c, "worst_wait_ms"))
        ));
    }
    lines.push(format!(
        "Critical patterns file: {}",
        if opt_truthy(s("critical_patterns_present")) { "present" } else { "absent" }
    ));
    if opt_truthy(s("models")) {
        let models = s("models");
        lines.push(format!(
            "Models (claude): {} · ceiling = the session model (keep it scarce; decisions 0012/0015/0021)",
            models_role_list(models.and_then(|m| vget(m, "claude")))
        ));
        // opencode-support oc-13/oc-14: printed unconditionally, same as
        // claude's line above — opencode now carries a built-in default too
        // (the free `opencode/*` provider names baked into every rendered
        // `.opencode/agent/bee-*.md`, oc-14), so an unconfigured repo has a
        // real answer to print, not an all-null line nobody asked for.
        lines.push(format!(
            "Models (opencode): {}",
            models_role_list(models.and_then(|m| vget(m, "opencode")))
        ));
        // model-role-split (mrs-27): codex had NO line at all. Two runtimes
        // were printed by hand out of the three `RUNTIMES` names status
        // itself carries, so an operator who configured `models.codex` read a
        // status page that reported on everything except the runtime they
        // configured — the same invisibility as the closed role list, one
        // axis over. It is guarded rather than unconditional because codex,
        // unlike claude and opencode, ships no built-in default: `bee status`
        // gains a line only on a host that actually configured one, so no
        // existing host reads a new all-null row it never asked for.
        let codex = models.and_then(|m| vget(m, "codex"));
        let codex_configured = matches!(codex, Some(Value::Object(t))
            if t.values().any(|v| !matches!(v, Value::Null)));
        if codex_configured {
            lines.push(format!("Models (codex): {}", models_role_list(codex)));
        }
    }
    // D6: the human line counts ROLES, and the escalation share is reported
    // beside them rather than folded into a fourth count. The role set is
    // open (D2), so this renders whatever roles the store actually holds —
    // it never names a fixed list that a configured role could fall out of.
    {
        let rm = s("role_mix");
        if opt_truthy(rm) && vget(rm.unwrap(), "cells").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0
        {
            let rm = rm.unwrap();
            let counts = vget(rm, "counts").cloned().unwrap_or(Value::Null);
            let share = vget(rm, "escalationShare").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let mut parts: Vec<String> = Vec::new();
            if let Value::Object(map) = &counts {
                for (role, n) in map {
                    parts.push(format!("{role}={}", tpl(Some(n))));
                }
            }
            lines.push(format!(
                "Role mix: {} (escalated {}/{}, {}%)",
                parts.join(" "),
                tpl(vget(rm, "escalated")),
                tpl(vget(rm, "cells")),
                jsjson::js_f64_to_string(js_round(share * 100.0))
            ));
        }
    }
    if opt_truthy(s("ceiling_scarcity")) {
        let cs = s("ceiling_scarcity").unwrap();
        lines.push(format!(
            "⚠ Ceiling scarcity: {}/{} cells escalated onto the session model ({}%) — drop the escalation on routine cells (decision 0012)",
            tpl(vget(cs, "ceiling")),
            tpl(vget(cs, "cells")),
            tpl(vget(cs, "pct"))
        ));
    }
    let high_risk = s("review")
        .and_then(|r| vget(r, "high_risk_unreviewed"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    if high_risk > 0.0 {
        lines.push(format!(
            "⚠ High-risk unreviewed: {} high-risk candidate(s) have not passed independent review — bee will not auto-dispatch reviewers; request review before merge/release.",
            tpl(s("review").and_then(|r| vget(r, "high_risk_unreviewed")))
        ));
    }
    if let Some(Value::Array(decisions)) = s("recent_decisions") {
        if !decisions.is_empty() {
            lines.push("Recent decisions:".to_string());
            for d in decisions {
                lines.push(format!("- {} ({})", tpl(vget(d, "decision")), tpl(vget(d, "date"))));
            }
        }
    }
    if let Some(Value::Array(warnings)) = s("staleness_warnings") {
        if !warnings.is_empty() {
            lines.push("Staleness warnings:".to_string());
            for w in warnings {
                lines.push(format!("- {}", jsjson::js_to_string(w)));
            }
        }
    }
    // D4 (wayfinding-flow): open discovery maps — resume must be
    // un-missable. Guarded on `open_maps` carrying at least one effort or
    // one unreadable MAP.md; absent or empty `docs/discovery/` prints
    // nothing at all.
    if let Some(om) = s("open_maps") {
        let efforts = vget(om, "efforts").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let unreadable = vget(om, "unreadable").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        if !efforts.is_empty() || !unreadable.is_empty() {
            for e in &efforts {
                lines.push(format!(
                    "### Open discovery map(s): {} — {} frontier ticket(s)",
                    tpl(vget(e, "name")),
                    tpl(vget(e, "frontier"))
                ));
            }
            for u in &unreadable {
                lines.push(format!("unreadable {} — remedy: fix or delete", tpl(Some(u))));
            }
        }
    }
    lines.push(format!("Recommended next: {}", tpl(s("recommended_next"))));
    lines.join("\n")
}
