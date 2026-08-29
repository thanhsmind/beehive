// onboard::apply — applyPlan (onboard_bee.mjs l. 3535–4018).
//
// Write-gate order is load-bearing and ported verbatim:
//   1. worktree-migration conflicts (msn-18d / advisor F4) — outranks
//      everything, zero mutations,
//   2. the codex-hybrid hook-write preflight (advisor R3, fail-closed),
//   2b. the hooks-merge validity preflight — a settings/hooks file that
//       exists but is malformed (bad JSON, a non-object `hooks` key, a
//       non-array event value) refuses by name, zero mutations, rather than
//       being treated as absent and silently rewritten,
//   3. the D3 blocked-first skill refusal (--force-downgrade overrides only
//      a fully-numeric version refusal).
// Only then does the item loop run, and only after IT does onboarding.json
// get its unconditional rewrite.

use super::agents::{compute_agents_sync_record, resolve_opencode_agent_model};
use super::hooks_wiring as hw;
use super::merge::{merge_agents_content, merge_gitignore_content};
use super::migration::{apply_worktree_migration, build_migration_conflict_reason, stranded_json};
use super::notices::compose_agents_header;
use super::plan::{compute_plan, ComputedPlan, Options};
use super::render::{build_render_sidecar, runtime_for_target_kind, source_skill_digest_entries};
use super::skills::{apply_remove_skill, apply_sync_skill};
use super::source::Engine;
use super::templates as T;
use super::util::{
    exists, join_rel, read_json_if_exists, read_text_if_exists, write_file_atomic,
};
use crate::jsjson;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

pub struct ApplyBlocked {
    pub status: String,
    pub reason: String,
    /// Absent for the migration-conflict and hook-collision refusals — Node
    /// leaves `versions`/`skills` `undefined` there and JSON.stringify drops
    /// the keys entirely.
    pub versions: Option<Value>,
    pub skills: Option<Value>,
    pub host_items: Option<Vec<Value>>,
    pub stranded: Option<Value>,
    pub bee_version: Option<String>,
}

pub struct ApplyOk {
    pub applied: Vec<Value>,
    pub onboarding: Value,
    pub bee_version: Option<String>,
    pub forced_downgrade: bool,
    pub forced_versions: Option<Value>,
    pub skills: Value,
}

pub enum ApplyOutcome {
    Blocked(Box<ApplyBlocked>),
    Ok(Box<ApplyOk>),
}

/// Whether a `remove_helper` plan item may unlink its target. Two admissible
/// sources (see plan.rs 3a/3b): a NAMED retired shim, or a helper the ledger
/// records that the source no longer ships. Both are flat `.mjs` files
/// directly under `.bee/bin/`.
///
/// The `.mjs` suffix is load-bearing, not decoration: `.bee/bin/` also holds
/// `bee.exe`/`bee`, the actual binary this process is running from. A plan item
/// that somehow named it must NEVER reach `remove_file` — bee would delete
/// itself, on every host, during a routine onboard.
fn remove_helper_admissible(rel: &str) -> bool {
    let name = posix_basename(rel);
    (T::RETIRED_HELPERS.contains(&name) || name.ends_with(".mjs"))
        && posix_dirname(rel) == ".bee/bin"
}

/// JS `path.dirname` on the POSIX-shaped `item.path` strings this script
/// constructs itself.
fn posix_dirname(p: &str) -> &str {
    match p.rfind('/') {
        Some(0) => &p[..1],
        Some(i) => &p[..i],
        None => ".",
    }
}

fn posix_basename(p: &str) -> &str {
    match p.rfind('/') {
        Some(i) => &p[i + 1..],
        None => p,
    }
}

/// utcNow(): `new Date().toISOString()`.
fn utc_now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

pub fn apply_plan(engine: &Engine, repo_root: &Path, opts: &Options) -> ApplyOutcome {
    let ComputedPlan {
        mut plan,
        bee_version,
        rendered_block,
        rendered_gitignore_block,
        desired_managed,
        skill_sync,
        codex_hybrid,
        worktree_migration,
    } = compute_plan(engine, repo_root, opts);

    // 1. worktree-migration preflight — ALL-OR-NOTHING.
    if !worktree_migration.conflicts.is_empty() {
        return ApplyOutcome::Blocked(Box::new(ApplyBlocked {
            status: "blocked_worktree_migration_conflict".into(),
            reason: build_migration_conflict_reason(&worktree_migration.conflicts),
            versions: None,
            skills: None,
            host_items: None,
            stranded: Some(stranded_json(&worktree_migration.conflicts)),
            bee_version,
        }));
    }

    // 2. codex-hybrid write preflight (fail-closed).
    if codex_hybrid && !hw::repo_owns_hook_catalog(repo_root) {
        if let Some((status, reason)) = hw::codex_hook_write_blocker(repo_root) {
            return ApplyOutcome::Blocked(Box::new(ApplyBlocked {
                status,
                reason,
                versions: None,
                skills: None,
                host_items: None,
                stranded: None,
                bee_version,
            }));
        }
    }

    // 2b. hooks-merge validity preflight: a settings/hooks file that exists
    // but fails the merge shape check (malformed JSON, a non-object "hooks"
    // key, a non-array event value) refuses BEFORE any item touches disk —
    // never silently treated as absent and clobbered with a bare
    // {"hooks": …}. Checked for whichever of the two merge items compute_plan
    // actually queued.
    for item in &plan {
        let action = item["action"].as_str().unwrap_or("");
        let rel = item["path"].as_str().unwrap_or("");
        let result = match action {
            "merge_repo_hook_settings" => Some(hw::merge_repo_settings(&join_rel(repo_root, rel))),
            "merge_codex_hooks" => Some(hw::merge_codex_hooks(&join_rel(repo_root, rel))),
            _ => None,
        };
        if let Some(Err(reason)) = result {
            return ApplyOutcome::Blocked(Box::new(ApplyBlocked {
                status: "blocked_hooks_merge".into(),
                reason,
                versions: None,
                skills: None,
                host_items: None,
                stranded: None,
                bee_version,
            }));
        }
    }

    // 3. D3 preflight: blocked-first across targets.
    let mut forced_downgrade = false;
    if let Some(blocked) = skill_sync.blocked.clone() {
        if opts.force_downgrade && blocked.forceable {
            forced_downgrade = true;
            // computePlan withholds ALL targets' items while the stage is
            // blocked — restore every target's computed items for the force.
            for target in &skill_sync.targets {
                plan.extend(target.items.iter().cloned());
            }
            if let Some(lr) = &skill_sync.legacy_refresh {
                plan.extend(lr.items.iter().cloned());
            }
        } else {
            // Review P1-6 / D2: the refused response carries every target's
            // computed items so a human sees the blast radius BEFORE forcing.
            let host_items = if blocked.forceable {
                Some(
                    plan.iter()
                        .filter(|i| {
                            matches!(
                                i["action"].as_str(),
                                Some("copy_lib")
                                    | Some("copy_helper")
                                    | Some("copy_expertise")
                                    | Some("copy_prompt")
                            )
                        })
                        .cloned()
                        .collect(),
                )
            } else {
                None
            };
            return ApplyOutcome::Blocked(Box::new(ApplyBlocked {
                status: blocked.status.clone(),
                reason: blocked.reason.clone(),
                versions: Some(blocked.versions.clone()),
                skills: Some(json!({
                    "source_root": skill_sync.source_root.to_string_lossy(),
                    "targets": skill_sync.targets.iter().map(|t| t.to_json()).collect::<Vec<_>>(),
                })),
                host_items,
                stranded: None,
                bee_version,
            }));
        }
    }

    let mut target_root_by_kind: Vec<(String, PathBuf)> =
        skill_sync.targets.iter().map(|t| (t.kind.clone(), t.target_root.clone())).collect();
    if let Some(lr) = &skill_sync.legacy_refresh {
        target_root_by_kind.push(("legacy-global".into(), lr.target_root.clone()));
    }
    let root_for = |kind: &str| -> Option<PathBuf> {
        target_root_by_kind.iter().find(|(k, _)| k == kind).map(|(_, p)| p.clone())
    };

    let mut applied: Vec<Value> = Vec::new();
    let mut skipped_skills: Vec<Value> = Vec::new();

    // Compose the header BEFORE any mergeAgentsContent call (decision D4).
    let propose_header = plan.iter().any(|i| i["action"] == "propose_agents_header");
    let header_text = if propose_header { compose_agents_header(repo_root) } else { String::new() };
    let mut header_applied = false;

    for item in &plan {
        let action = item["action"].as_str().unwrap_or("");
        let rel = item["path"].as_str().unwrap_or("");
        let target = join_rel(repo_root, rel);
        match action {
            "create_agents_block" | "append_agents_block" | "update_agents_block" => {
                let merged = merge_agents_content(
                    &format!("{header_text}{}", read_text_if_exists(&target)),
                    &rendered_block,
                );
                let _ = write_file_atomic(&target, merged.text.as_bytes());
                header_applied = propose_header;
            }
            "propose_agents_header" => {
                if header_applied {
                    applied.push(item.clone());
                    continue;
                }
                let merged = merge_agents_content(
                    &format!("{header_text}{}", read_text_if_exists(&target)),
                    &rendered_block,
                );
                let _ = write_file_atomic(&target, merged.text.as_bytes());
                header_applied = true;
            }
            "create_gitignore_block" | "append_gitignore_block" | "update_gitignore_block" => {
                let merged =
                    merge_gitignore_content(&read_text_if_exists(&target), &rendered_gitignore_block);
                let _ = write_file_atomic(&target, merged.text.as_bytes());
            }
            "create_runtime_file" => {
                if !exists(&target) {
                    let content = if rel.ends_with("state.json") {
                        format!("{}\n", jsjson::stringify_pretty(&T::default_state()))
                    } else if rel.ends_with("config.json") {
                        format!("{}\n", jsjson::stringify_pretty(&T::default_config()))
                    } else if rel.ends_with("reservations.json") {
                        format!("{}\n", jsjson::stringify_pretty(&T::default_reservations()))
                    } else if rel.ends_with("config-sample.json") {
                        T::CONFIG_SAMPLE_JSON.to_string()
                    } else {
                        String::new()
                    };
                    let _ = write_file_atomic(&target, content.as_bytes());
                }
            }
            "create_dir" => {
                let _ = std::fs::create_dir_all(&target);
            }
            "copy_helper" => {
                let name = posix_basename(rel);
                let _ = write_file_atomic(
                    &target,
                    read_text_if_exists(&engine.templates_dir.join(name)).as_bytes(),
                );
            }
            "remove_helper" => {
                // Never a generic rm. Two admissible sources now (see plan.rs
                // 3a/3b): a NAMED retired shim, or a helper the ledger records
                // that the source no longer ships. Both are flat `.mjs` files
                // directly under `.bee/bin/`.
                //
                // The `.mjs` suffix is load-bearing, not decoration:
                // `.bee/bin/` also holds `bee.exe`/`bee`, the actual binary
                // this process is running from. A ledger key that somehow named
                // it must NEVER reach `remove_file` — bee would delete itself,
                // on every host, during a routine onboard.
                if remove_helper_admissible(rel) {
                    let _ = std::fs::remove_file(&target);
                }
            }
            "copy_lib" => {
                let name = posix_basename(rel);
                let _ = write_file_atomic(
                    &target,
                    read_text_if_exists(&engine.templates_lib_dir.join(name)).as_bytes(),
                );
            }
            "remove_lib" => {
                if posix_dirname(rel) == ".bee/bin/lib" {
                    let _ = std::fs::remove_file(&target);
                }
            }
            "copy_expertise" => {
                // The name is the path RELATIVE to .bee/expertise — not the
                // basename: a pattern file is vendored as
                // "tests/patterns/<file>.md", and a basename would both read
                // the wrong source and flatten the tree.
                let name = rel.strip_prefix(".bee/expertise/").unwrap_or(rel);
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = write_file_atomic(
                    &target,
                    read_text_if_exists(&join_rel(&engine.expertise_dir, name)).as_bytes(),
                );
            }
            "remove_expertise" => {
                // Prefix-based containment: a vendored guide may sit in a
                // `<topic>/patterns/` subdirectory, and an exact-dirname
                // check would silently decline to clean those.
                if format!("{rel}/").starts_with(".bee/expertise/") && !rel.contains("..") {
                    let _ = std::fs::remove_file(&target);
                }
            }
            "copy_prompt" => {
                let name = posix_basename(rel);
                let _ = write_file_atomic(
                    &target,
                    read_text_if_exists(&engine.templates_prompts_dir.join(name)).as_bytes(),
                );
            }
            "remove_prompt" => {
                if posix_dirname(rel) == ".bee/bin/prompts" {
                    let _ = std::fs::remove_file(&target);
                }
            }
            // R6 cutover: the mirror of copy_repo_hook. Containment is the
            // same shape remove_lib uses — an exact dirname match, so a
            // crafted `path` can never reach outside the vendored hook dir.
            "remove_repo_hook" => {
                if posix_dirname(rel) == ".bee/bin/hooks" {
                    let _ = std::fs::remove_file(&target);
                }
            }
            "copy_repo_hook" => {
                let name = posix_basename(rel);
                let _ = write_file_atomic(
                    &target,
                    read_text_if_exists(&engine.plugin_hooks_dir.join(name)).as_bytes(),
                );
            }
            "copy_opencode_plugin" => {
                // opencode-support D2/D3, oc-13: source is this checkout's
                // OWN `.opencode/plugins/` tree, not a `packages/bee/`
                // template (see Engine::opencode_plugin_dir).
                let name = posix_basename(rel);
                let _ = write_file_atomic(
                    &target,
                    read_text_if_exists(&engine.opencode_plugin_dir.join(name)).as_bytes(),
                );
            }
            "copy_pi_extension" => {
                // pi-support D1: source is this checkout's OWN
                // `.pi/extensions/` tree, not a `packages/bee/` template
                // (see Engine::pi_extension_dir) — the same vendoring the
                // OpenCode plugin arm above does.
                let name = posix_basename(rel);
                let _ = write_file_atomic(
                    &target,
                    read_text_if_exists(&engine.pi_extension_dir.join(name)).as_bytes(),
                );
            }
            "copy_statusline" => {
                let name = posix_basename(rel);
                let _ = write_file_atomic(
                    &target,
                    read_text_if_exists(&engine.templates_statusline_dir.join(name)).as_bytes(),
                );
            }
            "create_stub" => {
                let _ = write_file_atomic(&target, T::CRITICAL_PATTERNS_STUB.as_bytes());
            }
            "create_specs_stub" => {
                // create-only: scribing owns these files.
                if !exists(&target) {
                    let body = if rel.ends_with("reading-map.md") {
                        T::READING_MAP_STUB
                    } else {
                        T::SYSTEM_OVERVIEW_STUB
                    };
                    let _ = write_file_atomic(&target, body.as_bytes());
                }
            }
            "create_claude_md" => {
                let _ = write_file_atomic(&target, T::claude_md_template().as_bytes());
            }
            "append_claude_md_import" => {
                let existing = read_text_if_exists(&target);
                let separator = if existing.ends_with('\n') { "\n" } else { "\n\n" };
                let _ = write_file_atomic(
                    &target,
                    format!("{existing}{separator}{}", T::CLAUDE_MD_IMPORT_SECTION).as_bytes(),
                );
            }
            "merge_repo_hook_settings" => {
                // The 2b preflight already refused on an Err before the loop
                // started; this Ok-only branch is a defensive fallback for a
                // plan-to-apply race, not the primary refusal path.
                if let Ok(merged) = hw::merge_repo_settings(&target) {
                    if exists(&target) {
                        let mut bak = target.clone().into_os_string();
                        bak.push(".bak");
                        let _ = std::fs::copy(&target, PathBuf::from(bak));
                    }
                    let _ = write_file_atomic(&target, merged.text.as_bytes());
                }
            }
            "merge_codex_hooks" => {
                if let Ok(merged) = hw::merge_codex_hooks(&target) {
                    if exists(&target) {
                        let mut bak = target.clone().into_os_string();
                        bak.push(".bak");
                        let _ = std::fs::copy(&target, PathBuf::from(bak));
                    }
                    let _ = write_file_atomic(&target, merged.text.as_bytes());
                }
            }
            "ensure_codex_statusline" => {
                // Machine-level target: NEVER the repoRoot-joined path above.
                let config_path = hw::codex_user_config_path();
                if !hw::codex_statusline_missing() {
                    applied.push(item.clone());
                    continue; // plan-to-apply race: someone added it meanwhile
                }
                let text = read_text_if_exists(&config_path);
                let mut bak = config_path.clone().into_os_string();
                bak.push(".bak");
                let _ = std::fs::copy(&config_path, PathBuf::from(bak));
                let _ = write_file_atomic(
                    &config_path,
                    hw::codex_statusline_next_text(&text).as_bytes(),
                );
            }
            "sync_agent_file" => {
                let agent = item["agent"].as_str().unwrap_or("");
                // agent-model-unpin D1: no model resolve — the file carries
                // no pin; the dispatch payload's model param is the authority.
                if let Some(rendered) = super::agents::render_claude_agent_file(engine, agent) {
                    let _ = write_file_atomic(&target, rendered.as_bytes());
                }
            }
            "remove_agent_file" => {
                let _ = std::fs::remove_file(&target);
            }
            "sync_opencode_agent_file" => {
                let agent = item["agent"].as_str().unwrap_or("");
                if let Some(model) = resolve_opencode_agent_model(repo_root, agent) {
                    if let Some(rendered) =
                        super::agents::render_opencode_agent_template(engine, agent, &model)
                    {
                        let _ = write_file_atomic(&target, rendered.as_bytes());
                    }
                }
            }
            "remove_opencode_agent_file" => {
                let _ = std::fs::remove_file(&target);
            }
            "write_onboarding" => {
                // handled after the loop so managed versions reflect the
                // final state
            }
            "migrate_worktree_records" => {
                apply_worktree_migration(&worktree_migration);
            }
            "sync_skill" => {
                let kind = item["target"].as_str().unwrap_or("");
                let skill = item["skill"].as_str().unwrap_or("");
                let Some(root) = root_for(kind) else {
                    applied.push(item.clone());
                    continue;
                };
                if let Some(reason) =
                    apply_sync_skill(&skill_sync.source_root, &root, skill, runtime_for_target_kind(kind))
                {
                    skipped_skills.push(json!({"skill": skill, "target": kind, "reason": reason}));
                    continue; // skipped loudly, not applied
                }
            }
            "refresh_legacy_global_skill" => {
                let kind = item["target"].as_str().unwrap_or("");
                let skill = item["skill"].as_str().unwrap_or("");
                let Some(root) = root_for(kind) else {
                    applied.push(item.clone());
                    continue;
                };
                // Honor "already exists" at apply time too (plan-to-apply
                // race): never create a copy that vanished.
                let st = super::util::lstat_if_exists(&root.join(skill));
                if !st.is_some_and(|s| !s.is_symlink && s.is_dir) {
                    skipped_skills.push(json!({
                        "skill": skill, "target": kind,
                        "reason": "legacy global skill is absent or not a plain directory - skipped, never created"
                    }));
                    continue;
                }
                if let Some(reason) =
                    apply_sync_skill(&skill_sync.source_root, &root, skill, runtime_for_target_kind(kind))
                {
                    skipped_skills.push(json!({"skill": skill, "target": kind, "reason": reason}));
                    continue;
                }
            }
            "remove_skill" => {
                let kind = item["target"].as_str().unwrap_or("");
                let skill = item["skill"].as_str().unwrap_or("");
                let Some(root) = root_for(kind) else {
                    applied.push(item.clone());
                    continue;
                };
                if let Some(reason) = apply_remove_skill(&root, skill) {
                    skipped_skills.push(json!({"skill": skill, "target": kind, "reason": reason}));
                    continue;
                }
            }
            "blocked_symlink" | "blocked_alias" => {
                skipped_skills.push(json!({
                    "skill": item["skill"], "target": item["target"], "reason": item["reason"]
                }));
                continue;
            }
            _ => {}
        }
        applied.push(item.clone());
    }

    // D9/D7 provenance stamps.
    if opts.sync_skills {
        let mut sidecar_by_runtime: Vec<(&'static str, Value)> = Vec::new();
        for t in &skill_sync.targets {
            let target_synced = matches!(t.mode, Some("sync") | Some("fresh"));
            let hive_skipped = skipped_skills.iter().any(|s| {
                s["target"].as_str() == Some(t.kind.as_str()) && s["skill"].as_str() == Some("bee-hive")
            });
            if target_synced && !hive_skipped {
                let payload = json!({
                    "version": bee_version.as_ref().map(|v| json!(v)).unwrap_or(Value::Null)
                });
                let _ = write_file_atomic(
                    &t.target_root.join(T::SKILLS_VERSION_STAMP),
                    format!("{}\n", jsjson::stringify_pretty(&payload)).as_bytes(),
                );
            }
            if t.blocked.is_some() || !target_synced {
                continue;
            }
            let runtime = runtime_for_target_kind(&t.kind);
            if !sidecar_by_runtime.iter().any(|(r, _)| *r == runtime) {
                let entries = source_skill_digest_entries(&skill_sync.source_root, runtime);
                sidecar_by_runtime.push((runtime, build_render_sidecar(runtime, &entries)));
            }
            let sidecar = sidecar_by_runtime.iter().find(|(r, _)| *r == runtime).unwrap().1.clone();
            let _ = write_file_atomic(
                &t.target_root.join(T::RENDER_SIDECAR),
                format!("{}\n", jsjson::stringify_pretty(&sidecar)).as_bytes(),
            );
        }
    }

    // Always (re)write onboarding.json on apply.
    let onboarding_path = repo_root.join(".bee").join("onboarding.json");
    let previous = read_json_if_exists(&onboarding_path).unwrap_or(json!({}));
    let mut managed = desired_managed.as_object().cloned().unwrap_or_default();
    // Advisor R6 / point 6: a --plugin-source apply lets a prior
    // --repo-hooks record LAPSE rather than silently carrying it forward.
    if !opts.repo_hooks && !opts.plugin_source {
        if let Some(prev) = previous.get("managed").and_then(|m| m.get("repo_hooks")) {
            if is_truthy(prev) {
                managed.insert("repo_hooks".into(), prev.clone());
            }
        }
    }
    let mut payload = Map::new();
    payload.insert("schema_version".into(), json!(T::ONBOARDING_SCHEMA_VERSION));
    payload.insert(
        "bee_version".into(),
        bee_version.as_ref().map(|v| json!(v)).unwrap_or(Value::Null),
    );
    payload.insert("managed".into(), Value::Object(managed));
    payload.insert(
        "agents_sync".into(),
        compute_agents_sync_record(
            engine,
            repo_root,
            &bee_version.as_ref().map(|v| json!(v)).unwrap_or(Value::Null),
        ),
    );
    let created_at = match previous.get("created_at") {
        Some(v) if is_truthy(v) => v.clone(),
        _ => json!(utc_now()),
    };
    payload.insert("created_at".into(), created_at);
    payload.insert("updated_at".into(), json!(utc_now()));
    let onboarding_payload = Value::Object(payload);
    let _ = write_file_atomic(
        &onboarding_path,
        format!("{}\n", jsjson::stringify_pretty(&onboarding_payload)).as_bytes(),
    );

    ApplyOutcome::Ok(Box::new(ApplyOk {
        applied,
        onboarding: onboarding_payload,
        bee_version,
        forced_downgrade,
        forced_versions: skill_sync.blocked.as_ref().map(|b| b.versions.clone()),
        skills: json!({
            "source_root": skill_sync.source_root.to_string_lossy(),
            "targets": skill_sync.targets.iter().map(|t| t.to_json()).collect::<Vec<_>>(),
            "skipped": skipped_skills,
        }),
    }))
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null | Value::Bool(false) => false,
        Value::String(s) => !s.is_empty(),
        Value::Number(n) => n.as_f64() != Some(0.0),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posix_path_helpers_match_node() {
        assert_eq!(posix_dirname(".bee/bin/lib/state.mjs"), ".bee/bin/lib");
        assert_eq!(posix_dirname(".bee/bin/x.mjs"), ".bee/bin");
        assert_eq!(posix_dirname("AGENTS.md"), ".");
        assert_eq!(posix_basename(".bee/expertise/tests/patterns/x.md"), "x.md");
    }

    /// THE containment proof for the R6 ledger-derived helper removal.
    #[test]
    fn remove_helper_never_reaches_the_bee_binary_beside_the_helpers() {
        // Admissible: the two sources plan.rs 3a/3b produce.
        assert!(remove_helper_admissible(".bee/bin/bee.mjs")); // ledger-derived
        assert!(remove_helper_admissible(".bee/bin/bee_state.mjs")); // RETIRED_HELPERS

        // THE case this guard exists for: bee lives in the same directory.
        assert!(!remove_helper_admissible(".bee/bin/bee.exe"));
        assert!(!remove_helper_admissible(".bee/bin/bee"));

        // Everything else in or near that directory stays out of reach.
        assert!(!remove_helper_admissible(".bee/bin/lib/state.mjs")); // remove_lib's job
        assert!(!remove_helper_admissible(".bee/bin/hooks/x.mjs")); // remove_repo_hook's job
        assert!(!remove_helper_admissible("../../outside.mjs"));
        assert!(!remove_helper_admissible(".bee/state.json"));
    }

    #[test]
    fn expertise_containment_guard_accepts_nested_and_rejects_escapes() {
        let ok = |p: &str| format!("{p}/").starts_with(".bee/expertise/") && !p.contains("..");
        assert!(ok(".bee/expertise/tests.md"));
        assert!(ok(".bee/expertise/tests/patterns/differential-testing.md"));
        assert!(!ok(".bee/expertise/../bin/bee.mjs"));
        assert!(!ok(".bee/bin/lib/state.mjs"));
    }

    #[test]
    fn utc_now_has_the_iso_millisecond_shape() {
        let s = utc_now();
        assert_eq!(s.len(), 24, "{s}");
        assert!(s.ends_with('Z'));
        assert_eq!(&s[10..11], "T");
    }
}
