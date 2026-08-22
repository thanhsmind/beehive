// Ownership loader and glob matching for knowledge areas and homed rules (D1/D4)
#![allow(unused_imports)]

use super::*;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AreaOwnership {
    pub(crate) area: String,
    pub(crate) code: Vec<String>,
    pub(crate) skills: Vec<String>,
    pub(crate) tests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct RuleHome {
    pub(crate) rule: String,
    pub(crate) home: String,
    pub(crate) areas: Vec<String>,
    pub(crate) applied_at: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Ownership {
    pub(crate) areas: HashMap<String, AreaOwnership>,
    pub(crate) rules: Vec<RuleHome>,
}

/// Walks docs/knowledge/ once and returns, per area, the owns.code / owns.skills /
/// owns.tests lists from areas/<area>/overview.md, plus every rule home.
pub(crate) fn load_ownership(repo_root: &Path) -> Ownership {
    let mut ownership = Ownership::default();
    let knowledge_dir = repo_root.join("docs").join("knowledge");
    let Some(files) = list_bundle_markdown(&knowledge_dir) else {
        return ownership;
    };

    let mut agents_homes: Vec<RuleHome> = Vec::new();

    for rel in files {
        let base = rel.rsplit('/').next().unwrap_or(&rel);
        if is_reserved_basename(base) {
            continue;
        }
        let full_path = join_rel(&knowledge_dir, &rel);
        let Ok(text) = read_file_lossy(&full_path) else {
            continue;
        };

        let Fm::Parsed { data, body, .. } = parse_frontmatter(&text) else {
            continue;
        };

        let bee = bee_of(&data);
        let body = &body;

        // 1. Area ownership from areas/<area>/overview.md
        let is_area_overview = rel.starts_with("areas/")
            && (rel.ends_with("/overview.md") || base == "overview.md")
            && rel.matches('/').count() == 2;

        if is_area_overview {
            let area_name = match bee.get("areas") {
                Some(Value::Array(arr)) if !arr.is_empty() => {
                    arr[0].as_str().map(str::to_string).unwrap_or_else(|| {
                        rel.split('/').nth(1).unwrap_or("").to_string()
                    })
                }
                _ => rel.split('/').nth(1).unwrap_or("").to_string(),
            };

            let code = match bee.get("owns.code") {
                Some(Value::Array(arr)) => {
                    arr.iter().filter_map(Value::as_str).map(str::to_string).collect()
                }
                _ => Vec::new(),
            };
            let skills = match bee.get("owns.skills") {
                Some(Value::Array(arr)) => {
                    arr.iter().filter_map(Value::as_str).map(str::to_string).collect()
                }
                _ => Vec::new(),
            };
            let tests = match bee.get("owns.tests") {
                Some(Value::Array(arr)) => {
                    arr.iter().filter_map(Value::as_str).map(str::to_string).collect()
                }
                _ => Vec::new(),
            };

            ownership.areas.insert(
                area_name.clone(),
                AreaOwnership {
                    area: area_name,
                    code,
                    skills,
                    tests,
                },
            );
        }

        // 2. Rule homes from concept
        let file_areas: Vec<String> = match bee.get("areas") {
            Some(Value::Array(arr)) => {
                arr.iter().filter_map(Value::as_str).map(str::to_string).collect()
            }
            _ => Vec::new(),
        };
        let applied_at: Vec<String> = match bee.get("applied_at") {
            Some(Value::Array(arr)) => {
                arr.iter().filter_map(Value::as_str).map(str::to_string).collect()
            }
            _ => Vec::new(),
        };

        let markers = extract_rule_markers(body);
        let home_path = format!("docs/knowledge/{}", rel.replace('\\', "/"));
        for marker in markers {
            ownership.rules.push(RuleHome {
                rule: marker,
                home: home_path.clone(),
                areas: file_areas.clone(),
                applied_at: applied_at.clone(),
            });
        }

        // 3. AGENTS.md rule homes section
        if body.contains("## AGENTS.md rule homes") {
            let parsed_agents = parse_agents_rule_homes(body, &file_areas);
            agents_homes.extend(parsed_agents);
        }
    }

    // Merge AGENTS.md homes
    for home in agents_homes {
        if !ownership
            .rules
            .iter()
            .any(|r| r.rule == home.rule && r.home == home.home)
        {
            ownership.rules.push(home);
        }
    }

    ownership
}

fn parse_agents_rule_homes(body: &str, default_areas: &[String]) -> Vec<RuleHome> {
    let mut out = Vec::new();
    let Some(start_pos) = body.find("## AGENTS.md rule homes") else {
        return out;
    };
    let rest = &body[start_pos + "## AGENTS.md rule homes".len()..];
    let section_text = if let Some(end_pos) = rest.find("\n## ") {
        &rest[..end_pos]
    } else {
        rest
    };

    let mut current_rule: Option<String> = None;
    let mut current_applied_at: Vec<String> = Vec::new();
    let mut in_applied_at = false;

    for line in section_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if line.starts_with("- ")
            || line.starts_with("* ")
            || (!line.starts_with("  ")
                && !line.starts_with("    ")
                && !line.starts_with('\t')
                && trimmed.starts_with("- "))
        {
            if let Some(rule_id) = current_rule.take() {
                out.push(RuleHome {
                    rule: rule_id,
                    home: "AGENTS.md".to_string(),
                    areas: default_areas.to_vec(),
                    applied_at: current_applied_at,
                });
                current_applied_at = Vec::new();
            }
            in_applied_at = false;

            let item_content = trimmed
                .strip_prefix("- ")
                .or_else(|| trimmed.strip_prefix("* "))
                .unwrap_or(trimmed);
            let rule_id = if let Some(start) = item_content.find('`') {
                if let Some(end) = item_content[start + 1..].find('`') {
                    &item_content[start + 1..start + 1 + end]
                } else {
                    item_content.split_whitespace().next().unwrap_or("")
                }
            } else {
                item_content.split([' ', '(', ':']).next().unwrap_or("")
            };
            let rule_id = rule_id.trim();
            if !rule_id.is_empty() {
                current_rule = Some(rule_id.to_string());
            }
        } else if trimmed.starts_with("- applied_at:")
            || trimmed.starts_with("* applied_at:")
            || trimmed == "applied_at:"
        {
            in_applied_at = true;
        } else if in_applied_at && (trimmed.starts_with("- ") || trimmed.starts_with("* ")) {
            let item = trimmed
                .strip_prefix("- ")
                .or_else(|| trimmed.strip_prefix("* "))
                .unwrap()
                .trim();
            let cleaned = item
                .trim_matches('`')
                .trim_matches('"')
                .trim_matches('\'')
                .trim();
            if !cleaned.is_empty() {
                current_applied_at.push(cleaned.to_string());
            }
        }
    }

    if let Some(rule_id) = current_rule.take() {
        out.push(RuleHome {
            rule: rule_id,
            home: "AGENTS.md".to_string(),
            areas: default_areas.to_vec(),
            applied_at: current_applied_at,
        });
    }

    out
}

#[allow(dead_code)]
pub(crate) fn matches_owned(list: &[String], path: &str) -> bool {
    let norm_path = path.replace('\\', "/");
    let norm_path = norm_path.trim_start_matches('/');
    for item in list {
        let norm_item = item.replace('\\', "/");
        let norm_item = norm_item.trim_start_matches('/');
        if matches_single_pattern(norm_item, norm_path) {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
fn matches_single_pattern(pat: &str, path: &str) -> bool {
    if pat == path {
        return true;
    }
    if let Some(prefix) = pat.strip_suffix("/**") {
        if path == prefix || path.starts_with(&format!("{prefix}/")) {
            return true;
        }
    }
    if let Some(prefix) = pat.strip_suffix("/*") {
        if path == prefix || path.starts_with(&format!("{prefix}/")) {
            return true;
        }
    }
    let pat_segs: Vec<&str> = pat.split('/').filter(|s| !s.is_empty()).collect();
    let path_segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    glob_match_segs(&pat_segs, &path_segs)
}

#[allow(dead_code)]
fn glob_match_segs(pat_segs: &[&str], path_segs: &[&str]) -> bool {
    if pat_segs.is_empty() {
        return path_segs.is_empty();
    }
    if pat_segs[0] == "**" {
        if pat_segs.len() == 1 {
            return true;
        }
        for i in 0..=path_segs.len() {
            if glob_match_segs(&pat_segs[1..], &path_segs[i..]) {
                return true;
            }
        }
        return false;
    }
    if path_segs.is_empty() {
        return false;
    }
    if match_glob_segment(pat_segs[0], path_segs[0]) {
        return glob_match_segs(&pat_segs[1..], &path_segs[1..]);
    }
    false
}
