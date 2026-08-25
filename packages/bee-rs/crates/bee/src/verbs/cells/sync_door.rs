// sync_door — the cap-time sync door (D3, D4)
//
// Refuses a cap that skips an owned skill, an applied_at target, or whose
// affects_skills prediction mismatches touched skills/** files.

#![allow(unused_imports)]

use super::*;
use crate::verbs::knowledge::{load_ownership, matches_owned, AreaOwnership, RuleHome};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

/// sync_refusal — runs the three cap-time sync checks:
/// (a) ownership: touched code in an area with non-empty owns.skills must touch at least one owned skill
/// (b) applied_at: touched rule home must touch every applied_at file
/// (c) prediction: affects_skills prediction must equal touched skills/** paths (skipped on legacy cells)
///
/// Returns None when clean, Some(refusal_message) on failure.
pub(crate) fn sync_refusal(root: &Path, cell: &Map<String, Value>, touched: &[String]) -> Option<String> {
    let ownership = load_ownership(root);
    let id = cell.get("id").and_then(Value::as_str).unwrap_or("(unknown id)");

    // (a) ownership: for every area whose owns.code matches a touched path and whose owns.skills
    // is non-empty, at least one owns.skills path must be touched.
    let mut area_keys: Vec<&String> = ownership.areas.keys().collect();
    area_keys.sort();
    for area_name in area_keys {
        let area = &ownership.areas[area_name];
        if area.skills.is_empty() {
            continue;
        }
        let code_touched = touched.iter().any(|t| matches_owned(&area.code, t));
        if code_touched {
            let skill_touched = touched.iter().any(|t| matches_owned(&area.skills, t));
            if !skill_touched {
                return Some(format!(
                    "capCell: SYNC_DOOR — cell \"{id}\" touches code owned by area \"{}\", but touches none of its owned skills: {}. FIX: update at least one of the area's owned skills, or pass --sync-ack \"<reason>\" to cap anyway (the reason is recorded on trace.sync_ack and trace.deviations).",
                    area.area,
                    area.skills.join(", ")
                ));
            }
        }
    }

    // (b) applied_at: for every rule whose home is touched, every applied_at file must be touched.
    let mut rules: Vec<&RuleHome> = ownership.rules.iter().collect();
    rules.sort_by_key(|r| (&r.rule, &r.home));
    for rule in rules {
        let home_norm = normalize_cell_path(&rule.home);
        let home_touched = touched.iter().any(|t| normalize_cell_path(t) == home_norm);
        if home_touched {
            let mut untouched: Vec<String> = Vec::new();
            for app in &rule.applied_at {
                let app_norm = normalize_cell_path(app);
                let is_touched = touched.iter().any(|t| {
                    let t_norm = normalize_cell_path(t);
                    t_norm == app_norm || matches_owned(&[app.clone()], t)
                });
                if !is_touched {
                    untouched.push(app.to_string());
                }
            }
            if !untouched.is_empty() {
                return Some(format!(
                    "capCell: SYNC_DOOR — cell \"{id}\" touches rule home \"{}\" (rule \"{}\"), but does not touch all applied_at files. Missing: {}. FIX: update the missing applied_at files, or pass --sync-ack \"<reason>\" to cap anyway (the reason is recorded on trace.sync_ack and trace.deviations).",
                    rule.home,
                    rule.rule,
                    untouched.join(", ")
                ));
            }
        }
    }

    // (c) prediction: cell's affects_skills must equal the set of touched skills/** paths as a set.
    // Legacy cells (no affects_skills key) skip this check.
    if let Some(affects_skills_val) = cell.get("affects_skills") {
        let predicted: BTreeSet<String> = match affects_skills_val {
            Value::Array(arr) => arr
                .iter()
                .filter_map(Value::as_str)
                .map(normalize_cell_path)
                .filter(|s| !s.is_empty())
                .collect(),
            _ => BTreeSet::new(),
        };
        let touched_skills: BTreeSet<String> = touched
            .iter()
            .map(|s| normalize_cell_path(s))
            .filter(|p| path_under_root(p, "skills"))
            .collect();
        if predicted != touched_skills {
            let unpredicted: Vec<String> = touched_skills.difference(&predicted).cloned().collect();
            // The comparison is unchanged; only the WORDING is. A prediction
            // written as a bare skill name can never match a touched path, so
            // name it as the input error it is and print the path that would
            // have matched (belt and braces for cells written before `cells
            // add` began refusing the format).
            let unfulfilled: Vec<String> = predicted
                .difference(&touched_skills)
                .map(|p| match bare_skill_name_path(root, p) {
                    Some(path) => format!("{p} (a bare skill name, not a path — use \"{path}\")"),
                    None => p.clone(),
                })
                .collect();
            let mut diffs = Vec::new();
            if !unpredicted.is_empty() {
                diffs.push(format!("touched but unpredicted: {}", unpredicted.join(", ")));
            }
            if !unfulfilled.is_empty() {
                diffs.push(format!("predicted but untouched: {}", unfulfilled.join(", ")));
            }
            return Some(format!(
                "capCell: SYNC_DOOR — cell \"{id}\" affects_skills prediction does not match touched skills/** paths. Difference: {}. FIX: update affects_skills with `bee cells update --id {id} --stdin` (or touch the predicted skills), or pass --sync-ack \"<reason>\" to cap anyway (the reason is recorded on trace.sync_ack and trace.deviations).",
                diffs.join("; ")
            ));
        }
    }

    None
}
