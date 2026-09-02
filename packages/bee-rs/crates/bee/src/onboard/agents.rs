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
    AGENT_OPENCODE_PERMISSION_DENY, AGENT_ROLES_BY_NAME, AGENT_TIER_DEFAULTS_CLAUDE,
    AGENT_TIER_DEFAULTS_OPENCODE, CODEX_AGENTS_NOTE,
};
use super::util::{exists, read_dir_sorted, read_json_if_exists, read_text_if_exists};
use crate::verbs::drivers::{normalize_tier_value, resolve_role, Resolved};
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

/// The ordered ROLE LIST an agent declares (model-role-split D2/D3).
///
/// `None` is a template with no entry in `AGENT_ROLES_BY_NAME` at all — an
/// agent bee does not know — and its file is never rendered, exactly as an
/// unmapped tier was never rendered before.
pub fn roles_for_agent(agent_name: &str) -> Option<&'static [&'static str]> {
    AGENT_ROLES_BY_NAME.iter().find(|(n, _)| *n == agent_name).map(|(_, roles)| *roles)
}

/// The `models` map the SHARED resolver walks for one runtime's agent files.
///
/// model-role-split D1/D2 (store `cd72ec97`, `06e49368`): this module used to
/// carry its own `normalizeAgentTierValueLocal` + `resolveAgentTierModel`
/// pair — a second parser of the `models.<runtime>` shape, keyed to a closed
/// three-slot table. Both are gone. What is left is the one thing onboarding
/// legitimately owns: the SEED, bee's baked-in model per role for agent
/// FILES, which differs from `drivers::default_models` on opencode by design
/// (`AGENT_TIER_DEFAULTS_OPENCODE`). Everything above the seed — which value
/// shapes are legal, which resolve, which yield — is `resolve_role`'s.
///
/// The overlay carries EVERY key the config names under that runtime, not a
/// fixed slot list, which is what makes a host's own role name reach the
/// rendered file. `normalize_tier_value` returning `None` is a junk value:
/// the seed keeps standing, exactly as the old local normalizer's `Unset`
/// did. A key the config names as an explicit `null` is NOT junk — it
/// normalizes to `Null`, replaces the seed, and turns the role off, so
/// "absent" and "refused" stay different reads.
fn agent_models(repo_root: &Path, runtime_key: &str, seed: &[(&str, &str)]) -> Map<String, Value> {
    let config = read_json_if_exists(&repo_root.join(".bee").join("config.json"));
    let raw_runtime = config
        .as_ref()
        .filter(|c| c.is_object())
        .and_then(|c| c.get("models"))
        .filter(|m| m.is_object())
        .and_then(|m| m.get(runtime_key))
        .and_then(|c| c.as_object())
        .cloned();

    let mut slice: Map<String, Value> = seed
        .iter()
        .map(|(role, model)| ((*role).to_string(), Value::String((*model).to_string())))
        .collect();
    if let Some(raw) = raw_runtime {
        for (role, value) in &raw {
            if let Some(normalized) = normalize_tier_value(Some(value)) {
                slice.insert(role.clone(), normalized);
            }
        }
    }
    let mut models = Map::new();
    models.insert(runtime_key.to_string(), Value::Object(slice));
    models
}

/// Resolve an agent's declared roles into the model its file pins.
///
/// Only a resolved MODEL renders a file. A role that resolves to a cli
/// command, a herding router, an inherited session model or nothing at all is
/// not a model name, so the file is skipped and a stale copy removed — the
/// same outcome the old local resolver reached by returning `None` for
/// everything that was not `Slot::Model`.
///
/// The purpose passed to `resolve_role` is `"cell"`: these agent files are
/// what a cell dispatch runs as. It is not observable here — a cli slot maps
/// to no model under every purpose — but naming the honest purpose keeps the
/// call readable next to prepare's.
fn resolve_agent_model_generic(
    repo_root: &Path,
    roles: &[&str],
    runtime_key: &str,
    seed: &[(&str, &str)],
) -> Option<String> {
    let models = agent_models(repo_root, runtime_key, seed);
    match resolve_role(&models, roles, runtime_key, "cell") {
        Resolved::Model { model, .. } | Resolved::Native { model, .. } => Some(model),
        _ => None,
    }
}

/// The model `.claude/agents/<agent>.md` pins, resolved from the roles that
/// agent declares. Replaces `resolveAgentTierModel` (l. 1980).
pub fn resolve_agent_model(repo_root: &Path, agent_name: &str) -> Option<String> {
    let roles = roles_for_agent(agent_name)?;
    resolve_agent_model_generic(repo_root, roles, "claude", AGENT_TIER_DEFAULTS_CLAUDE)
}

/// opencode-support oc-14's counterpart: same roles, same shared resolver,
/// keyed off `models.opencode` and seeded with the free `opencode/*` names
/// instead of haiku/sonnet/opus.
pub fn resolve_opencode_agent_model(repo_root: &Path, agent_name: &str) -> Option<String> {
    let roles = roles_for_agent(agent_name)?;
    resolve_agent_model_generic(repo_root, roles, "opencode", AGENT_TIER_DEFAULTS_OPENCODE)
}

/// renderAgentTemplate (l. 2007): `source.split("{{TIER_MODEL}}").join(model)`.
pub fn render_agent_template(engine: &Engine, agent_name: &str, model: &str) -> String {
    let source =
        read_text_if_exists(&engine.templates_agents_dir.join(format!("{agent_name}.md.tmpl")));
    source.replace("{{TIER_MODEL}}", model)
}

/// agent-model-unpin D1: the Claude agent file carries NO model pin — the
/// dispatch door's `model` param is the one model authority (it overrides
/// frontmatter in the harness), so the render needs no resolved model and a
/// slot's shape (herding/cli/null) can no longer remove the file. `None` is
/// only an agent bee does not know (`AGENT_ROLES_BY_NAME` has no entry) —
/// that template is never rendered, exactly as before. A `{{TIER_MODEL}}`
/// line in a stale template copy is dropped rather than shipped verbatim.
pub fn render_claude_agent_file(engine: &Engine, agent_name: &str) -> Option<String> {
    roles_for_agent(agent_name)?;
    let source =
        read_text_if_exists(&engine.templates_agents_dir.join(format!("{agent_name}.md.tmpl")));
    if !source.contains("{{TIER_MODEL}}") {
        return Some(source);
    }
    let mut out = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        if !line.contains("{{TIER_MODEL}}") {
            out.push_str(line);
        }
    }
    Some(out)
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
/// against the target; a role list that resolves to no model skips the render
/// and removes a stale copy.
pub fn compute_agent_file_plan(engine: &Engine, repo_root: &Path) -> Vec<Value> {
    let mut items = Vec::new();
    for tmpl_name in list_template_agents(engine) {
        let agent_name = tmpl_name.trim_end_matches(".md.tmpl").to_string();
        let rel_path = format!(".claude/agents/{agent_name}.md");
        let target = repo_root.join(".claude").join("agents").join(format!("{agent_name}.md"));
        // agent-model-unpin D1/D2: a known agent renders UNCONDITIONALLY —
        // no model resolve, so a herded/cli/null slot keeps the file (the
        // dispatch payload's model param is the authority). Removal is only
        // for a template bee does not know.
        match render_claude_agent_file(engine, &agent_name) {
            Some(rendered) => {
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
/// seed and renderer over the same shared role resolver. A role list that
/// resolves to no model (or a template missing a `description:`/permission
/// profile) skips the render and removes a stale copy, same shape as the
/// Claude side.
pub fn compute_opencode_agent_file_plan(engine: &Engine, repo_root: &Path) -> Vec<Value> {
    let mut items = Vec::new();
    for tmpl_name in list_template_agents(engine) {
        let agent_name = tmpl_name.trim_end_matches(".md.tmpl").to_string();
        let rel_path = format!(".opencode/agent/{agent_name}.md");
        let target = repo_root.join(".opencode").join("agent").join(format!("{agent_name}.md"));
        let model = resolve_opencode_agent_model(repo_root, &agent_name);
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
///
/// `rendered_from` is keyed by the role the agent ASKS FOR — the head of its
/// declared list — not by the entry fall-through happened to land on. That is
/// what the record said when the key was a tier (a `bee-review` rendered off
/// a null review slot recorded `review`, not `generation`), and it is the
/// honest read either way: the key names the request, the value names what
/// answered it.
pub fn compute_agents_sync_record(engine: &Engine, repo_root: &Path, bee_version: &Value) -> Value {
    let mut files: Vec<Value> = Vec::new();
    let mut rendered_from = Map::new();
    for tmpl_name in list_template_agents(engine) {
        let agent_name = tmpl_name.trim_end_matches(".md.tmpl").to_string();
        let head_role = roles_for_agent(&agent_name).and_then(|r| r.first().copied());
        if let Some(model) = resolve_agent_model(repo_root, &agent_name) {
            files.push(json!(format!(".claude/agents/{agent_name}.md")));
            if let Some(role) = head_role {
                rendered_from.insert(role.to_string(), json!(model));
            }
        }
    }
    let mut opencode_files: Vec<Value> = Vec::new();
    let mut opencode_rendered_from = Map::new();
    for tmpl_name in list_template_agents(engine) {
        let agent_name = tmpl_name.trim_end_matches(".md.tmpl").to_string();
        let head_role = roles_for_agent(&agent_name).and_then(|r| r.first().copied());
        if let Some(model) = resolve_opencode_agent_model(repo_root, &agent_name) {
            opencode_files.push(json!(format!(".opencode/agent/{agent_name}.md")));
            if let Some(role) = head_role {
                opencode_rendered_from.insert(role.to_string(), json!(model));
            }
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
        assert_eq!(resolve_agent_model(&repo, "bee-extract").as_deref(), Some("haiku"));
        assert_eq!(resolve_agent_model(&repo, "bee-gather").as_deref(), Some("sonnet"));
        assert_eq!(resolve_agent_model(&repo, "bee-build").as_deref(), Some("sonnet"));
        assert_eq!(resolve_agent_model(&repo, "bee-review").as_deref(), Some("opus"));
    }

    /// An agent bee has no role list for renders nothing — the `None` arm of
    /// `roles_for_agent`, which used to be the unmapped-tier arm.
    #[test]
    fn an_unknown_agent_resolves_no_model() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        assert_eq!(resolve_agent_model(&repo, "bee-nope"), None);
        assert_eq!(resolve_opencode_agent_model(&repo, "bee-nope"), None);
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
        assert_eq!(resolve_agent_model(&repo, "bee-review").as_deref(), Some("sonnet-x"));
        write(json!({"models":{"claude":{"review":{"kind":"cli","command":"x"}}}}));
        assert_eq!(resolve_agent_model(&repo, "bee-review"), None);
        write(json!({"models":{"claude":{"generation":{"model":"  m  "}}}}));
        assert_eq!(resolve_agent_model(&repo, "bee-gather").as_deref(), Some("m"));
        // An invalid shape leaves the default standing.
        write(json!({"models":{"claude":{"generation":42}}}));
        assert_eq!(resolve_agent_model(&repo, "bee-gather").as_deref(), Some("sonnet"));
        // bee-extract declares ONE role and must NOT fall through: a null
        // extraction slot removes its file rather than handing it the
        // generation model.
        write(json!({"models":{"claude":{"extraction":null,"generation":"sonnet-x"}}}));
        assert_eq!(resolve_agent_model(&repo, "bee-extract"), None);
    }

    /// The capability the whole rebase exists for: a host configures a role
    /// name of its own under `models.<runtime>`, an agent declares it, and
    /// the rendered agent file pins that model — no bee code knows the name.
    /// The role list is declared here rather than in `AGENT_ROLES_BY_NAME`
    /// because WHICH names bee publishes is a separate decision (D3); the
    /// mechanism this test pins is the resolver path, not the name list.
    #[test]
    fn a_host_configured_role_name_reaches_the_rendered_model() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(repo.join(".bee")).unwrap();
        std::fs::write(
            repo.join(".bee").join("config.json"),
            json!({"models":{"claude":{"docs":"a-model-good-at-docs"},
                             "opencode":{"docs":"opencode/docs-model"}}})
            .to_string(),
        )
        .unwrap();
        assert_eq!(
            resolve_agent_model_generic(&repo, &["docs"], "claude", AGENT_TIER_DEFAULTS_CLAUDE)
                .as_deref(),
            Some("a-model-good-at-docs")
        );
        assert_eq!(
            resolve_agent_model_generic(
                &repo,
                &["docs"],
                "opencode",
                AGENT_TIER_DEFAULTS_OPENCODE
            )
            .as_deref(),
            Some("opencode/docs-model")
        );
        // Unconfigured heads yield to the next name; the walk is the shared
        // resolver's, not a second one here.
        assert_eq!(
            resolve_agent_model_generic(
                &repo,
                &["test", "docs"],
                "claude",
                AGENT_TIER_DEFAULTS_CLAUDE
            )
            .as_deref(),
            Some("a-model-good-at-docs")
        );
    }

    #[test]
    fn plan_renders_unconditionally_and_never_removes_a_known_agent() {
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

        // agent-model-unpin D1: the render needs no model, and a legacy
        // `model: {{TIER_MODEL}}` template line is dropped, never shipped.
        for it in &items {
            let agent = it["agent"].as_str().unwrap();
            let rendered = render_claude_agent_file(&engine, agent).unwrap();
            assert!(
                !rendered.contains("model:"),
                "rendered {agent} still carries a model pin: {rendered}"
            );
            let target = repo.join(".claude").join("agents").join(format!("{agent}.md"));
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(&target, rendered).unwrap();
        }
        assert!(compute_agent_file_plan(&engine, &repo).is_empty());

        // agent-model-unpin D2: a cli-shaped review slot used to remove its
        // file; the file no longer names a model, so the slot's shape is
        // irrelevant and the plan settles.
        std::fs::create_dir_all(repo.join(".bee")).unwrap();
        std::fs::write(
            repo.join(".bee").join("config.json"),
            json!({"models":{"claude":{"review":{"kind":"cli","command":"x"}}}}).to_string(),
        )
        .unwrap();
        assert!(compute_agent_file_plan(&engine, &repo).is_empty());

        // The regression that motivated D2: a herded generation slot keeps
        // bee-gather.md (it removed it before, stranding the still-native
        // code/test roles without their execution agent).
        std::fs::write(
            repo.join(".bee").join("config.json"),
            json!({"models":{"claude":{"generation":{"kind":"herding","agent":"agy-flash"}}}})
                .to_string(),
        )
        .unwrap();
        assert!(compute_agent_file_plan(&engine, &repo).is_empty());

        // gather-reads-the-read-slot D6: the same holds for the slot
        // bee-gather now asks for FIRST. A herding-shaped `read` slot opens a
        // pane instead of a subagent, and it still must not remove the
        // rendered bee-gather file — that file carries the read-only tool
        // permissions no bare general-purpose type can express.
        std::fs::write(
            repo.join(".bee").join("config.json"),
            json!({"models":{"claude":{"read":{"kind":"herding","agent":"agy-flash"}}}})
                .to_string(),
        )
        .unwrap();
        assert!(compute_agent_file_plan(&engine, &repo).is_empty());
        assert!(repo.join(".claude").join("agents").join("bee-gather.md").exists());

        // An unknown template (no AGENT_ROLES_BY_NAME entry) still renders
        // nothing, and a stale copy of it is still removed.
        std::fs::write(
            engine.templates_agents_dir.join("bee-mystery.md.tmpl"),
            "---\n---\nmystery\n",
        )
        .unwrap();
        let target = repo.join(".claude").join("agents").join("bee-mystery.md");
        std::fs::write(&target, "stale").unwrap();
        let items = compute_agent_file_plan(&engine, &repo);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["action"], "remove_agent_file");
        assert_eq!(items[0]["agent"], "bee-mystery");
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
            resolve_opencode_agent_model(&repo, "bee-extract").as_deref(),
            Some("opencode/ling-3.0-tiny-free")
        );
        assert_eq!(
            resolve_opencode_agent_model(&repo, "bee-gather").as_deref(),
            Some("opencode/big-pickle")
        );
        assert_eq!(
            resolve_opencode_agent_model(&repo, "bee-build").as_deref(),
            Some("opencode/big-pickle")
        );
        assert_eq!(
            resolve_opencode_agent_model(&repo, "bee-review").as_deref(),
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
        assert_eq!(resolve_opencode_agent_model(&repo, "bee-review").as_deref(), Some("opencode/x"));
        write(json!({"models":{"opencode":{"review":{"kind":"cli","command":"x"}}}}));
        assert_eq!(resolve_opencode_agent_model(&repo, "bee-review"), None);
    }

    /// gather-reads-the-read-slot D4: `bee-gather` declares `["read",
    /// "generation"]`, so a host that configures a `read` slot sees it in the
    /// rendered agent file, and a host that never heard of `read` renders
    /// exactly the bytes it rendered before. The walk is the shared
    /// resolver's — nothing here re-implements the fall-through.
    #[test]
    fn bee_gather_pins_the_read_slot_and_falls_through_to_generation() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(repo.join(".bee")).unwrap();
        let write = |cfg: Value| {
            std::fs::write(repo.join(".bee").join("config.json"), cfg.to_string()).unwrap()
        };

        // (a) a configured read slot pins that model.
        write(json!({"models":{"opencode":{"read":"opencode/reader","generation":"opencode/x"},
                                "claude":{"read":"haiku-r","generation":"sonnet-x"}}}));
        assert_eq!(
            resolve_opencode_agent_model(&repo, "bee-gather").as_deref(),
            Some("opencode/reader")
        );
        assert_eq!(resolve_agent_model(&repo, "bee-gather").as_deref(), Some("haiku-r"));
        // bee-build is untouched by the read slot — it declares generation.
        assert_eq!(resolve_opencode_agent_model(&repo, "bee-build").as_deref(), Some("opencode/x"));
        assert_eq!(resolve_agent_model(&repo, "bee-build").as_deref(), Some("sonnet-x"));

        // (b) no `read` key at all — the legacy host — renders generation's
        // model, byte for byte what it rendered before D4.
        write(json!({"models":{"opencode":{"extraction":"opencode/tiny","generation":"opencode/x"},
                                "claude":{"extraction":"haiku","generation":"sonnet-x"}}}));
        assert_eq!(resolve_opencode_agent_model(&repo, "bee-gather").as_deref(), Some("opencode/x"));
        assert_eq!(resolve_agent_model(&repo, "bee-gather").as_deref(), Some("sonnet-x"));

        // (c) an explicitly null read slot falls through the same way a null
        // review slot does — it is a two-name list, not bee-extract's one.
        write(json!({"models":{"opencode":{"read":null,"generation":"opencode/x"}}}));
        assert_eq!(resolve_opencode_agent_model(&repo, "bee-gather").as_deref(), Some("opencode/x"));

        // (d) a read slot that names no model (cli) removes the opencode
        // file, exactly as a cli generation slot does today.
        write(json!({"models":{"opencode":{"read":{"kind":"cli","command":"x"},
                                            "generation":"opencode/x"}}}));
        assert_eq!(resolve_opencode_agent_model(&repo, "bee-gather"), None);
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
            let model = resolve_opencode_agent_model(&repo, agent).unwrap();
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
            json!({"models":{"opencode":{"review":{"kind":"cli","command":"x"}}}}).to_string(),
        )
        .unwrap();
        let items = compute_opencode_agent_file_plan(&engine, &repo);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["action"], "remove_opencode_agent_file");
        assert_eq!(items[0]["agent"], "bee-review");

        // The Claude side's note applies here too: a command-less
        // `{kind:"cli"}` is junk to the one shared parser, so the seed stands
        // and the plan settles.
        std::fs::write(
            repo.join(".bee").join("config.json"),
            json!({"models":{"opencode":{"review":{"kind":"cli"}}}}).to_string(),
        )
        .unwrap();
        assert!(compute_opencode_agent_file_plan(&engine, &repo).is_empty());
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
        for (agent, _roles) in AGENT_ROLES_BY_NAME {
            let model = resolve_opencode_agent_model(&root, agent)
                .unwrap_or_else(|| panic!("no default opencode model for agent {agent}"));
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
