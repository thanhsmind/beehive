// onboard::agents — the flat `.claude/agents/bee-*.md` managed-file sync.
//
// Provenance: onboard_bee.mjs listTemplateAgents (l. 1919),
// normalizeAgentTierValueLocal (l. 1958), resolveAgentTierModel (l. 1980),
// renderAgentTemplate (l. 2007), computeAgentFilePlan (l. 2018) and
// computeAgentsSyncRecord (l. 2044).
//
// AO11 asymmetry (Codex gets no agent files) is recorded inline in the
// sync record, never as a separate file.

use super::source::Engine;
use super::templates::{AGENT_TIER_BY_NAME, AGENT_TIER_DEFAULTS_CLAUDE, CODEX_AGENTS_NOTE};
use super::util::{exists, read_dir_sorted, read_json_if_exists, read_text_if_exists};
use serde_json::{json, Map, Value};
use std::path::Path;

/// listTemplateAgents (l. 1919): `*.md.tmpl`, sorted.
pub fn list_template_agents(engine: &Engine) -> Vec<String> {
    if !exists(&engine.templates_agents_dir) {
        return Vec::new();
    }
    read_dir_sorted(&engine.templates_agents_dir)
        .into_iter()
        .filter(|e| e.is_file && e.name.ends_with(".md.tmpl"))
        .map(|e| e.name)
        .collect()
}

fn tier_for_agent(agent_name: &str) -> Option<&'static str> {
    AGENT_TIER_BY_NAME.iter().find(|(n, _)| *n == agent_name).map(|(_, t)| *t)
}

/// normalizeAgentTierValueLocal (l. 1958). The three non-string outcomes stay
/// distinct: resolveAgentTierModel's review→generation fallback fires ONLY on
/// an explicit null, exactly like state.mjs resolveTier.
#[derive(Debug, Clone, PartialEq)]
enum Slot {
    Model(String),
    Null,
    Cli,
    /// "no override" — the default for that slot stands.
    Unset,
}

fn normalize_agent_tier_value(value: Option<&Value>) -> Slot {
    match value {
        Some(Value::String(s)) if !s.trim().is_empty() => Slot::Model(s.trim().to_string()),
        Some(Value::Null) => Slot::Null,
        Some(Value::Object(o)) => {
            match o.get("kind") {
                Some(Value::String(k)) if k == "cli" => Slot::Cli,
                None => match o.get("model") {
                    Some(Value::String(m)) if !m.trim().is_empty() => {
                        Slot::Model(m.trim().to_string())
                    }
                    _ => Slot::Unset,
                },
                _ => Slot::Unset,
            }
        }
        _ => Slot::Unset,
    }
}

/// resolveAgentTierModel (l. 1980): claude runtime only (AO11).
pub fn resolve_agent_tier_model(repo_root: &Path, tier: &str) -> Option<String> {
    let config = read_json_if_exists(&repo_root.join(".bee").join("config.json"));
    let raw_claude = config
        .as_ref()
        .filter(|c| c.is_object())
        .and_then(|c| c.get("models"))
        .filter(|m| m.is_object())
        .and_then(|m| m.get("claude"))
        .filter(|c| c.is_object())
        .cloned();

    let mut resolved: Vec<(&str, Slot)> = AGENT_TIER_DEFAULTS_CLAUDE
        .iter()
        .map(|(slot, model)| (*slot, Slot::Model((*model).to_string())))
        .collect();
    if let Some(raw) = raw_claude {
        for (slot, current) in resolved.iter_mut() {
            let value = normalize_agent_tier_value(raw.get(*slot));
            if value != Slot::Unset {
                *current = value;
            }
        }
    }
    let get = |name: &str| resolved.iter().find(|(s, _)| *s == name).map(|(_, v)| v.clone());
    let mut value = get(tier)?;
    if value == Slot::Null && tier == "review" {
        value = get("generation")?;
    }
    match value {
        Slot::Model(m) => Some(m),
        _ => None,
    }
}

/// renderAgentTemplate (l. 2007): `source.split("{{TIER_MODEL}}").join(model)`.
pub fn render_agent_template(engine: &Engine, agent_name: &str, model: &str) -> String {
    let source =
        read_text_if_exists(&engine.templates_agents_dir.join(format!("{agent_name}.md.tmpl")));
    source.replace("{{TIER_MODEL}}", model)
}

/// computeAgentFilePlan (l. 2018): byte-compare each rendered template
/// against the target; a tier that resolves to null skips the render and
/// removes a stale copy.
pub fn compute_agent_file_plan(engine: &Engine, repo_root: &Path) -> Vec<Value> {
    let mut items = Vec::new();
    for tmpl_name in list_template_agents(engine) {
        let agent_name = tmpl_name.trim_end_matches(".md.tmpl").to_string();
        let tier = tier_for_agent(&agent_name);
        let rel_path = format!(".claude/agents/{agent_name}.md");
        let target = repo_root.join(".claude").join("agents").join(format!("{agent_name}.md"));
        let model = tier.and_then(|t| resolve_agent_tier_model(repo_root, t));
        match model {
            Some(model) => {
                let rendered = render_agent_template(engine, &agent_name, &model);
                if read_text_if_exists(&target) != rendered {
                    items.push(
                        json!({"action": "sync_agent_file", "path": rel_path, "agent": agent_name}),
                    );
                }
            }
            None => {
                if exists(&target) {
                    items.push(
                        json!({"action": "remove_agent_file", "path": rel_path, "agent": agent_name}),
                    );
                }
            }
        }
    }
    items
}

/// computeAgentsSyncRecord (l. 2044): computed POST-apply so `files` names
/// what is actually present.
pub fn compute_agents_sync_record(engine: &Engine, repo_root: &Path, bee_version: &Value) -> Value {
    let mut files: Vec<Value> = Vec::new();
    let mut rendered_from = Map::new();
    for tmpl_name in list_template_agents(engine) {
        let agent_name = tmpl_name.trim_end_matches(".md.tmpl").to_string();
        let tier = tier_for_agent(&agent_name);
        if let Some(model) = tier.and_then(|t| resolve_agent_tier_model(repo_root, t)) {
            files.push(json!(format!(".claude/agents/{agent_name}.md")));
            rendered_from.insert(tier.unwrap().to_string(), json!(model));
        }
    }
    json!({
        "bee_version": bee_version,
        "files": files,
        "rendered_from": Value::Object(rendered_from),
        "codex": { "agents": [], "note": CODEX_AGENTS_NOTE },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_with_agents(dir: &Path) -> Engine {
        let e = Engine::from_plugin_root(dir.to_path_buf());
        std::fs::create_dir_all(&e.templates_agents_dir).unwrap();
        for (name, body) in [
            ("bee-gather.md.tmpl", "---\nmodel: {{TIER_MODEL}}\n---\ngather\n"),
            ("bee-extract.md.tmpl", "---\nmodel: {{TIER_MODEL}}\n---\nextract\n"),
            ("bee-review.md.tmpl", "---\nmodel: {{TIER_MODEL}}\n---\nreview\n"),
        ] {
            std::fs::write(e.templates_agents_dir.join(name), body).unwrap();
        }
        e
    }

    #[test]
    fn defaults_apply_without_config() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        assert_eq!(resolve_agent_tier_model(&repo, "extraction").as_deref(), Some("haiku"));
        assert_eq!(resolve_agent_tier_model(&repo, "generation").as_deref(), Some("sonnet"));
        assert_eq!(resolve_agent_tier_model(&repo, "review").as_deref(), Some("opus"));
    }

    #[test]
    fn explicit_null_review_falls_back_to_generation_but_cli_does_not() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(repo.join(".bee")).unwrap();
        let write = |cfg: Value| {
            std::fs::write(repo.join(".bee").join("config.json"), cfg.to_string()).unwrap()
        };
        write(json!({"models":{"claude":{"review":null,"generation":"sonnet-x"}}}));
        assert_eq!(resolve_agent_tier_model(&repo, "review").as_deref(), Some("sonnet-x"));
        write(json!({"models":{"claude":{"review":{"kind":"cli","command":"x"}}}}));
        assert_eq!(resolve_agent_tier_model(&repo, "review"), None);
        write(json!({"models":{"claude":{"generation":{"model":"  m  "}}}}));
        assert_eq!(resolve_agent_tier_model(&repo, "generation").as_deref(), Some("m"));
        // An invalid shape leaves the default standing.
        write(json!({"models":{"claude":{"generation":42}}}));
        assert_eq!(resolve_agent_tier_model(&repo, "generation").as_deref(), Some("sonnet"));
    }

    #[test]
    fn plan_syncs_then_settles_then_removes_on_null() {
        let dir = tempfile::tempdir().unwrap();
        let engine = engine_with_agents(dir.path());
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();

        let items = compute_agent_file_plan(&engine, &repo);
        let paths: Vec<&str> = items.iter().map(|i| i["path"].as_str().unwrap()).collect();
        assert_eq!(
            paths,
            vec![
                ".claude/agents/bee-extract.md",
                ".claude/agents/bee-gather.md",
                ".claude/agents/bee-review.md"
            ]
        );

        // Materialize them; the plan then settles.
        for it in &items {
            let agent = it["agent"].as_str().unwrap();
            let tier = tier_for_agent(agent).unwrap();
            let model = resolve_agent_tier_model(&repo, tier).unwrap();
            let target = repo.join(".claude").join("agents").join(format!("{agent}.md"));
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(&target, render_agent_template(&engine, agent, &model)).unwrap();
        }
        assert!(compute_agent_file_plan(&engine, &repo).is_empty());

        // A cli-shaped review slot removes its file.
        std::fs::create_dir_all(repo.join(".bee")).unwrap();
        std::fs::write(
            repo.join(".bee").join("config.json"),
            json!({"models":{"claude":{"review":{"kind":"cli"}}}}).to_string(),
        )
        .unwrap();
        let items = compute_agent_file_plan(&engine, &repo);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["action"], "remove_agent_file");
        assert_eq!(items[0]["agent"], "bee-review");
    }

    #[test]
    fn sync_record_names_present_files_and_the_codex_note() {
        let dir = tempfile::tempdir().unwrap();
        let engine = engine_with_agents(dir.path());
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        let rec = compute_agents_sync_record(&engine, &repo, &json!("1.2.3"));
        assert_eq!(rec["bee_version"], "1.2.3");
        assert_eq!(rec["files"].as_array().unwrap().len(), 3);
        assert_eq!(rec["rendered_from"]["extraction"], "haiku");
        assert_eq!(rec["codex"]["agents"].as_array().unwrap().len(), 0);
        assert!(rec["codex"]["note"].as_str().unwrap().contains("AO11"));
    }
}
