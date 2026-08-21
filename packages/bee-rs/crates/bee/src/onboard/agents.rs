// onboard::agents — the flat `.claude/agents/bee-*.md` managed-file sync,
// plus its OpenCode sibling `.opencode/agent/bee-*.md` (opencode-support
// oc-14, D4).
//
// Provenance: onboard_bee.mjs listTemplateAgents (l. 1919),
// normalizeAgentTierValueLocal (l. 1958), resolveAgentTierModel (l. 1980),
// renderAgentTemplate (l. 2007), computeAgentFilePlan (l. 2018) and
// computeAgentsSyncRecord (l. 2044).
//
// AO11 asymmetry (Codex gets no agent files) is recorded inline in the
// sync record, never as a separate file.
//
// The OpenCode rendering (oc-14) shares its SOURCE with the Claude one byte
// for byte: same `packages/bee/agents/*.md.tmpl` file, same `description:`
// field, same body below the frontmatter. Only the frontmatter SHAPE
// differs, because that shape is what OpenCode's own agent-file schema
// requires (`mode`/`model`/`permission` instead of `name`/`tools`/`model`,
// verified live against opencode 1.18.16 — opencode-support oc-11/
// discovery.md). Before oc-14 the four `.opencode/agent/bee-*.md` files
// were hand-authored and only ever existed in this checkout; they now
// render into any onboarded host the same way the Claude ones do.

use super::source::Engine;
use super::templates::{
    AGENT_OPENCODE_PERMISSION_DENY, AGENT_TIER_BY_NAME, AGENT_TIER_DEFAULTS_CLAUDE,
    AGENT_TIER_DEFAULTS_OPENCODE, CODEX_AGENTS_NOTE,
};
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
    /// herding-tier D1: `{kind:"herding"}` — a router value, never a model
    /// name. AO11-shaped: like Cli, resolve_tier_model_generic reads this as
    /// "no model" (sync_agent_file no-ops, no file written) — the Claude
    /// subagent Task/Model tool has nothing to write here since cell
    /// dispatch on this slot routes through the herding-exec Bash payload,
    /// never a Task dispatch.
    Herding,
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
                Some(Value::String(k)) if k == "herding" => Slot::Herding,
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

/// resolveAgentTierModel (l. 1980), generalized over the runtime's
/// `models.<runtime_key>` config slice and its own default table — the same
/// resolution shape claude and opencode share, keyed differently.
fn resolve_tier_model_generic(
    repo_root: &Path,
    tier: &str,
    runtime_key: &str,
    defaults: &[(&str, &str)],
) -> Option<String> {
    let config = read_json_if_exists(&repo_root.join(".bee").join("config.json"));
    let raw_runtime = config
        .as_ref()
        .filter(|c| c.is_object())
        .and_then(|c| c.get("models"))
        .filter(|m| m.is_object())
        .and_then(|m| m.get(runtime_key))
        .filter(|c| c.is_object())
        .cloned();

    let mut resolved: Vec<(&str, Slot)> =
        defaults.iter().map(|(slot, model)| (*slot, Slot::Model((*model).to_string()))).collect();
    if let Some(raw) = raw_runtime {
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

/// resolveAgentTierModel (l. 1980): claude runtime only (AO11).
pub fn resolve_agent_tier_model(repo_root: &Path, tier: &str) -> Option<String> {
    resolve_tier_model_generic(repo_root, tier, "claude", AGENT_TIER_DEFAULTS_CLAUDE)
}

/// opencode-support oc-14: OpenCode's counterpart, keyed off
/// `models.opencode` and `AGENT_TIER_DEFAULTS_OPENCODE` instead of
/// `models.claude`/`AGENT_TIER_DEFAULTS_CLAUDE` — same fallback shape
/// (explicit-null review falls back to generation), different config slice
/// and different baked-in defaults (the free `opencode/*` provider, not
/// haiku/sonnet/opus).
pub fn resolve_opencode_agent_tier_model(repo_root: &Path, tier: &str) -> Option<String> {
    resolve_tier_model_generic(repo_root, tier, "opencode", AGENT_TIER_DEFAULTS_OPENCODE)
}

/// renderAgentTemplate (l. 2007): `source.split("{{TIER_MODEL}}").join(model)`.
pub fn render_agent_template(engine: &Engine, agent_name: &str, model: &str) -> String {
    let source =
        read_text_if_exists(&engine.templates_agents_dir.join(format!("{agent_name}.md.tmpl")));
    source.replace("{{TIER_MODEL}}", model)
}

/// Splits a `.md.tmpl` source into (frontmatter, body): the frontmatter is
/// the text strictly between the opening and closing `---` lines, the body
/// is everything from just past the closing `---` line onward (including
/// the blank line that conventionally follows it). A file with no closing
/// delimiter is treated as having no frontmatter at all — the whole source
/// is the body — so a malformed template degrades to "no fields found"
/// rather than panicking.
fn split_frontmatter(source: &str) -> (&str, &str) {
    let Some(after_open) = source.strip_prefix("---\n") else { return ("", source) };
    let Some(close_at) = after_open.find("\n---\n") else { return ("", source) };
    let front = &after_open[..close_at];
    let body = &after_open[close_at + "\n---\n".len()..];
    (front, body)
}

/// Reads one `key: value` line out of a frontmatter block (first match,
/// trimmed). Every template field this module reads (`description`) is a
/// single-line scalar, never folded YAML.
fn frontmatter_field<'a>(front: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}:");
    front.lines().find_map(|line| line.strip_prefix(&prefix)).map(str::trim)
}

/// opencode-support oc-14: renders `.opencode/agent/<name>.md` from the same
/// `.md.tmpl` source `render_agent_template` uses for Claude — same
/// `description:` value, same body — with OpenCode's own frontmatter shape
/// (`description`/`mode`/`model`/`permission`, not `name`/`tools`/`model`).
/// `None` when the agent has no known permission profile or the template
/// carries no `description:` field — a malformed/unknown source skips the
/// render rather than shipping a broken file.
pub fn render_opencode_agent_template(engine: &Engine, agent_name: &str, model: &str) -> Option<String> {
    let source =
        read_text_if_exists(&engine.templates_agents_dir.join(format!("{agent_name}.md.tmpl")));
    let (front, body) = split_frontmatter(&source);
    let description = frontmatter_field(front, "description")?;
    let deny = AGENT_OPENCODE_PERMISSION_DENY.iter().find(|(n, _)| *n == agent_name).map(|(_, d)| *d)?;
    let mut out = String::new();
    out.push_str("---\ndescription: ");
    out.push_str(description);
    out.push_str("\nmode: subagent\nmodel: ");
    out.push_str(model);
    out.push_str("\npermission:\n");
    for name in deny {
        out.push_str("  ");
        out.push_str(name);
        out.push_str(": deny\n");
    }
    out.push_str("---\n");
    out.push_str(body);
    Some(out)
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

/// opencode-support oc-14: OpenCode's counterpart to `compute_agent_file_plan`
/// — same template set, `.opencode/agent/<name>.md` targets, OpenCode's own
/// tier resolver and renderer. A tier that resolves to null (or a template
/// missing a `description:`/permission profile) skips the render and removes
/// a stale copy, same shape as the Claude side.
pub fn compute_opencode_agent_file_plan(engine: &Engine, repo_root: &Path) -> Vec<Value> {
    let mut items = Vec::new();
    for tmpl_name in list_template_agents(engine) {
        let agent_name = tmpl_name.trim_end_matches(".md.tmpl").to_string();
        let tier = tier_for_agent(&agent_name);
        let rel_path = format!(".opencode/agent/{agent_name}.md");
        let target = repo_root.join(".opencode").join("agent").join(format!("{agent_name}.md"));
        let model = tier.and_then(|t| resolve_opencode_agent_tier_model(repo_root, t));
        match model {
            Some(model) => match render_opencode_agent_template(engine, &agent_name, &model) {
                Some(rendered) if read_text_if_exists(&target) != rendered => {
                    items.push(json!({
                        "action": "sync_opencode_agent_file",
                        "path": rel_path,
                        "agent": agent_name,
                    }));
                }
                _ => {}
            },
            None => {
                if exists(&target) {
                    items.push(json!({
                        "action": "remove_opencode_agent_file",
                        "path": rel_path,
                        "agent": agent_name,
                    }));
                }
            }
        }
    }
    items
}

/// computeAgentsSyncRecord (l. 2044): computed POST-apply so `files` names
/// what is actually present. opencode-support oc-14 adds an `opencode` sibling
/// to the existing `codex` asymmetry note, now that OpenCode agent files are
/// rendered (not just hand-authored, oc-11) — same shape, own file set.
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
    let mut opencode_files: Vec<Value> = Vec::new();
    let mut opencode_rendered_from = Map::new();
    for tmpl_name in list_template_agents(engine) {
        let agent_name = tmpl_name.trim_end_matches(".md.tmpl").to_string();
        let tier = tier_for_agent(&agent_name);
        if let Some(model) = tier.and_then(|t| resolve_opencode_agent_tier_model(repo_root, t)) {
            opencode_files.push(json!(format!(".opencode/agent/{agent_name}.md")));
            opencode_rendered_from.insert(tier.unwrap().to_string(), json!(model));
        }
    }
    json!({
        "bee_version": bee_version,
        "files": files,
        "rendered_from": Value::Object(rendered_from),
        "codex": { "agents": [], "note": CODEX_AGENTS_NOTE },
        "opencode": {
            "files": opencode_files,
            "rendered_from": Value::Object(opencode_rendered_from),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

    /// A fixture carrying a `description:` field and all four agents (unlike
    /// `engine_with_agents` above, which the pre-existing Claude-only tests
    /// use and which deliberately omits `description:` since Claude's render
    /// never reads it) — what the OpenCode render needs.
    fn engine_with_all_agents(dir: &Path) -> Engine {
        let e = Engine::from_plugin_root(dir.to_path_buf());
        std::fs::create_dir_all(&e.templates_agents_dir).unwrap();
        for (name, body) in [
            (
                "bee-build.md.tmpl",
                "---\nname: bee-build\ndescription: build worker\ntools: Read, Edit\nmodel: {{TIER_MODEL}}\n---\n\nbuild body\n",
            ),
            (
                "bee-gather.md.tmpl",
                "---\nname: bee-gather\ndescription: gather worker\ntools: Read\nmodel: {{TIER_MODEL}}\n---\n\ngather body\n",
            ),
            (
                "bee-extract.md.tmpl",
                "---\nname: bee-extract\ndescription: extract worker\ntools: Read\nmodel: {{TIER_MODEL}}\n---\n\nextract body\n",
            ),
            (
                "bee-review.md.tmpl",
                "---\nname: bee-review\ndescription: review worker\ntools: Read, Bash\nmodel: {{TIER_MODEL}}\n---\n\nreview body\n",
            ),
        ] {
            std::fs::write(e.templates_agents_dir.join(name), body).unwrap();
        }
        e
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..").join("..")
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

    #[test]
    fn opencode_defaults_apply_without_config() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        assert_eq!(
            resolve_opencode_agent_tier_model(&repo, "extraction").as_deref(),
            Some("opencode/ling-3.0-tiny-free")
        );
        assert_eq!(
            resolve_opencode_agent_tier_model(&repo, "generation").as_deref(),
            Some("opencode/big-pickle")
        );
        assert_eq!(
            resolve_opencode_agent_tier_model(&repo, "review").as_deref(),
            Some("opencode/nemotron-3-ultra-free")
        );
    }

    #[test]
    fn opencode_explicit_null_review_falls_back_to_generation_but_cli_does_not() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(repo.join(".bee")).unwrap();
        let write = |cfg: Value| {
            std::fs::write(repo.join(".bee").join("config.json"), cfg.to_string()).unwrap()
        };
        write(json!({"models":{"opencode":{"review":null,"generation":"opencode/x"}}}));
        assert_eq!(resolve_opencode_agent_tier_model(&repo, "review").as_deref(), Some("opencode/x"));
        write(json!({"models":{"opencode":{"review":{"kind":"cli","command":"x"}}}}));
        assert_eq!(resolve_opencode_agent_tier_model(&repo, "review"), None);
    }

    #[test]
    fn render_opencode_uses_the_shared_description_and_body_with_its_own_frontmatter_shape() {
        let dir = tempfile::tempdir().unwrap();
        let engine = engine_with_all_agents(dir.path());
        let rendered = render_opencode_agent_template(&engine, "bee-build", "opencode/x").unwrap();
        assert_eq!(
            rendered,
            "---\ndescription: build worker\nmode: subagent\nmodel: opencode/x\npermission:\n  task: deny\n  todowrite: deny\n  webfetch: deny\n  websearch: deny\n  lsp: deny\n---\n\nbuild body\n"
        );
        let rendered_review =
            render_opencode_agent_template(&engine, "bee-review", "opencode/y").unwrap();
        assert_eq!(
            rendered_review,
            "---\ndescription: review worker\nmode: subagent\nmodel: opencode/y\npermission:\n  edit: deny\n  task: deny\n  todowrite: deny\n  webfetch: deny\n  websearch: deny\n  lsp: deny\n---\n\nreview body\n"
        );
    }

    #[test]
    fn opencode_plan_syncs_then_settles_then_removes_on_null() {
        let dir = tempfile::tempdir().unwrap();
        let engine = engine_with_all_agents(dir.path());
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();

        let items = compute_opencode_agent_file_plan(&engine, &repo);
        let paths: Vec<&str> = items.iter().map(|i| i["path"].as_str().unwrap()).collect();
        assert_eq!(
            paths,
            vec![
                ".opencode/agent/bee-build.md",
                ".opencode/agent/bee-extract.md",
                ".opencode/agent/bee-gather.md",
                ".opencode/agent/bee-review.md",
            ]
        );

        // Materialize them; the plan then settles.
        for it in &items {
            let agent = it["agent"].as_str().unwrap();
            let tier = tier_for_agent(agent).unwrap();
            let model = resolve_opencode_agent_tier_model(&repo, tier).unwrap();
            let target = repo.join(".opencode").join("agent").join(format!("{agent}.md"));
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(
                &target,
                render_opencode_agent_template(&engine, agent, &model).unwrap(),
            )
            .unwrap();
        }
        assert!(compute_opencode_agent_file_plan(&engine, &repo).is_empty());

        // A cli-shaped review slot removes its file.
        std::fs::create_dir_all(repo.join(".bee")).unwrap();
        std::fs::write(
            repo.join(".bee").join("config.json"),
            json!({"models":{"opencode":{"review":{"kind":"cli"}}}}).to_string(),
        )
        .unwrap();
        let items = compute_opencode_agent_file_plan(&engine, &repo);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["action"], "remove_opencode_agent_file");
        assert_eq!(items[0]["agent"], "bee-review");
    }

    #[test]
    fn sync_record_includes_the_opencode_sibling() {
        let dir = tempfile::tempdir().unwrap();
        let engine = engine_with_all_agents(dir.path());
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        let rec = compute_agents_sync_record(&engine, &repo, &json!("1.2.3"));
        assert_eq!(rec["opencode"]["files"].as_array().unwrap().len(), 4);
        assert_eq!(rec["opencode"]["rendered_from"]["generation"], "opencode/big-pickle");
        assert_eq!(rec["opencode"]["rendered_from"]["extraction"], "opencode/ling-3.0-tiny-free");
    }

    /// The pin: `render_opencode_agent_template` against the REAL, checked-in
    /// `packages/bee/agents/*.md.tmpl` source must reproduce the committed
    /// `.opencode/agent/bee-*.md` files byte for byte — the same "committed
    /// tree can drift from what the renderer produces" class of test
    /// `skills.rs`'s opencode sidecar pin guards against. Before oc-14 those
    /// four files were hand-authored with no renderer at all; this is the
    /// proof the renderer now agrees with the hand-authored baseline it
    /// replaces.
    #[test]
    fn opencode_render_matches_the_committed_projection_byte_for_byte() {
        let root = repo_root();
        let engine = Engine::from_plugin_root(root.clone());
        if !exists(&engine.templates_agents_dir) {
            return; // packaged build, not a source checkout — nothing to pin
        }
        for (agent, tier) in AGENT_TIER_BY_NAME {
            let model = resolve_opencode_agent_tier_model(&root, tier)
                .unwrap_or_else(|| panic!("no default opencode model for tier {tier}"));
            let rendered = render_opencode_agent_template(&engine, agent, &model)
                .unwrap_or_else(|| panic!("opencode render skipped for {agent}"));
            let committed =
                read_text_if_exists(&root.join(".opencode").join("agent").join(format!("{agent}.md")));
            assert_eq!(
                rendered, committed,
                "{agent}: render_opencode_agent_template drifted from the committed \
                 .opencode/agent/{agent}.md — re-render or update the template"
            );
        }
    }
}
