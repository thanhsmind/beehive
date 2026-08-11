// onboard::skills — the per-target skill-sync stage (D1–D5, D9).
//
// Provenance: onboard_bee.mjs entryIdentity (l. 1008), detectAliasCollisions
// (l. 1016), detectNestedAlias (l. 1047), aliasBlockedItem (l. 1076),
// computeSkillItems (l. 1088), computeSkillSyncTarget (l. 1210),
// aggregateSkillBlocked (l. 1382), hostLibDowngradeBlock (l. 1412),
// computeLegacyGlobalRefresh (l. 1463), computeSkillSync (l. 1547),
// applySyncSkill (l. 1672), applyRemoveSkill (l. 1758) and
// blockedSourceIdentitySkillSync (l. 2980).

use super::render::{
    list_bee_skill_entries, manifest_fingerprint, render_skill_bytes, runtime_for_target_kind,
    validate_skill_tree_markers, walk_skill_tree, Walk,
};
use super::source::{
    read_host_version_strict, read_manifest_version_strict, read_version_strict,
    repo_target_segments_joined,
    skill_sync_targets, skills_target_root, unknown_versions_triple, compare_versions, Engine,
    ReleaseIdentity, VersionState,
};
use super::templates::SKILLS_VERSION_STAMP;
use super::util::{
    entry_identity, exists, is_under, join_rel, lstat_if_exists, path_resolve, realpath,
    write_file_atomic_random, EntryId,
};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

// ── plan items ─────────────────────────────────────────────────────────────

fn item(action: &str, skill: &str, path: &str, scope: &str) -> Value {
    let mut m = Map::new();
    m.insert("action".into(), json!(action));
    m.insert("skill".into(), json!(skill));
    m.insert("path".into(), json!(path));
    m.insert("scope".into(), json!(scope));
    Value::Object(m)
}

fn blocked_item(action: &str, skill: &str, path: &str, scope: &str, reason: String) -> Value {
    let mut v = item(action, skill, path, scope);
    v.as_object_mut().unwrap().insert("reason".into(), json!(reason));
    v
}

/// aliasBlockedItem (l. 1076): alias identity is always probed under
/// targetRoot, so `scope` is "installed".
fn alias_blocked_item(name: &str, detail: &str) -> Value {
    blocked_item(
        "blocked_alias",
        name,
        name,
        "installed",
        format!("installed {name} {detail} - blocked, never sync-then-delete"),
    )
}

fn with_target(mut v: Value, target: &str) -> Value {
    v.as_object_mut().unwrap().insert("target".into(), json!(target));
    v
}

// ── alias detection (win32 defect fix rides entry_identity, see util.rs) ───

/// detectAliasCollisions (l. 1016): probe every candidate name under the
/// target root; two DIFFERENT names on one physical identity collide.
fn detect_alias_collisions(source_names: &[String], target_root: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for n in source_names {
        if !names.contains(n) {
            names.push(n.clone());
        }
    }
    for entry in list_bee_skill_entries(target_root) {
        if !names.contains(&entry.name) {
            names.push(entry.name);
        }
    }
    let mut by_identity: Vec<(EntryId, Vec<String>)> = Vec::new();
    for name in names {
        let Some(id) = entry_identity(&target_root.join(&name)) else { continue };
        match by_identity.iter_mut().find(|(k, _)| *k == id) {
            Some((_, v)) => v.push(name),
            None => by_identity.push((id, vec![name])),
        }
    }
    let mut collided: Vec<String> = Vec::new();
    for (_, alias_names) in by_identity {
        if alias_names.len() > 1 {
            for n in alias_names {
                if !collided.contains(&n) {
                    collided.push(n);
                }
            }
        }
    }
    collided
}

/// detectNestedAlias (l. 1047): the same check inside ONE skill.
fn detect_nested_alias(
    target_dir: &Path,
    source_walk: &Walk,
    target_walk: &Walk,
) -> Option<(String, String)> {
    let mut rels: Vec<String> = Vec::new();
    let push = |r: &String, rels: &mut Vec<String>| {
        if !rels.contains(r) {
            rels.push(r.clone());
        }
    };
    for (r, _) in &source_walk.files {
        push(r, &mut rels);
    }
    for r in &source_walk.dirs {
        push(r, &mut rels);
    }
    for (r, _) in &target_walk.files {
        push(r, &mut rels);
    }
    for r in &target_walk.dirs {
        push(r, &mut rels);
    }
    let mut by_identity: Vec<(EntryId, String)> = Vec::new();
    for rel in rels {
        let Some(id) = entry_identity(&join_rel(target_dir, &rel)) else { continue };
        if let Some((_, existing)) = by_identity.iter().find(|(k, _)| *k == id) {
            if existing != &rel {
                return Some((existing.clone(), rel));
            }
        } else {
            by_identity.push((id, rel));
        }
    }
    None
}

// ── computeSkillItems ──────────────────────────────────────────────────────

/// computeSkillItems (l. 1088): D4/D5 drift plan items. Content difference IS
/// drift, at any version (D5); a bee-* skill absent from the anchored source
/// IS an intentional removal (D2).
pub fn compute_skill_items(source_root: &Path, target_root: &Path, runtime: &str) -> Vec<Value> {
    let mut items = Vec::new();
    let render = |buf: &[u8]| render_skill_bytes(buf, runtime);
    let source_entries = list_bee_skill_entries(source_root);
    let source_names: Vec<String> = source_entries.iter().map(|e| e.name.clone()).collect();
    let alias_collisions = detect_alias_collisions(&source_names, target_root);

    for entry in &source_entries {
        let name = &entry.name;
        if alias_collisions.contains(name) {
            items.push(alias_blocked_item(
                name,
                "shares one physical entry with a differently-named bee-* entry (case-insensitive alias)",
            ));
            continue;
        }
        if entry.is_symlink {
            items.push(blocked_item(
                "blocked_symlink",
                name,
                name,
                "source",
                format!("source {name} is a symlink - skipped, never followed"),
            ));
            continue;
        }
        if !entry.is_dir {
            continue; // stray bee-* file in source: not a skill dir
        }
        let source_walk = walk_skill_tree(&source_root.join(name), Some(&render));
        if let Some(b) = &source_walk.blocked {
            items.push(blocked_item(
                "blocked_symlink",
                name,
                &format!("{name}/{}", b.path),
                "source",
                format!("source {name} contains a {} at {} - skipped", b.reason, b.path),
            ));
            continue;
        }
        let target_dir = target_root.join(name);
        let target_stat = lstat_if_exists(&target_dir);
        if target_stat.is_some_and(|s| s.is_symlink) {
            items.push(blocked_item(
                "blocked_symlink",
                name,
                name,
                "installed",
                format!("installed {name} is a symlink (plausibly a live checkout) - skipped, never written through or unlinked"),
            ));
            continue;
        }
        if !target_stat.is_some_and(|s| s.is_dir) {
            items.push(item("sync_skill", name, name, "installed"));
            continue;
        }
        let target_walk = walk_skill_tree(&target_dir, None);
        if let Some(b) = &target_walk.blocked {
            items.push(blocked_item(
                "blocked_symlink",
                name,
                &format!("{name}/{}", b.path),
                "installed",
                format!("installed {name} contains a {} at {} - skipped, nothing inside it written or deleted", b.reason, b.path),
            ));
            continue;
        }
        if let Some((a, b)) = detect_nested_alias(&target_dir, &source_walk, &target_walk) {
            items.push(alias_blocked_item(
                name,
                &format!("has nested entries {a} and {b} resolving to one physical entry (case-insensitive alias)"),
            ));
            continue;
        }
        if manifest_fingerprint(&source_walk.files) != manifest_fingerprint(&target_walk.files) {
            items.push(item("sync_skill", name, name, "installed"));
        }
    }

    for entry in list_bee_skill_entries(target_root) {
        let name = &entry.name;
        if source_names.contains(name) {
            continue;
        }
        if alias_collisions.contains(name) {
            items.push(alias_blocked_item(
                name,
                "shares one physical entry with a differently-named bee-* entry (case-insensitive alias)",
            ));
            continue;
        }
        if entry.is_symlink {
            items.push(blocked_item(
                "blocked_symlink",
                name,
                name,
                "installed",
                format!("installed {name} is a symlink (plausibly a live checkout) - skipped, never unlinked"),
            ));
            continue;
        }
        if !entry.is_dir {
            continue; // deletion domain is /^bee-/ DIRECTORY entries only (D4)
        }
        let target_walk = walk_skill_tree(&target_root.join(name), None);
        if let Some(b) = &target_walk.blocked {
            items.push(blocked_item(
                "blocked_symlink",
                name,
                &format!("{name}/{}", b.path),
                "installed",
                format!("installed {name} contains a {} at {} - skipped, nothing deleted", b.reason, b.path),
            ));
            continue;
        }
        items.push(item("remove_skill", name, name, "installed"));
    }

    items
}

// ── per-target resolution ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TargetBlocked {
    pub status: String,
    pub reason: String,
    pub forceable: bool,
}

impl TargetBlocked {
    fn to_json(&self) -> Value {
        json!({"status": self.status, "reason": self.reason, "forceable": self.forceable})
    }
}

#[derive(Debug, Clone)]
pub struct SkillTarget {
    pub kind: String,
    pub target_root: PathBuf,
    pub mode: Option<&'static str>,
    pub versions: Option<Value>,
    pub blocked: Option<TargetBlocked>,
    pub items: Vec<Value>,
}

impl SkillTarget {
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("kind".into(), json!(self.kind));
        m.insert("target_root".into(), json!(self.target_root.to_string_lossy()));
        m.insert("mode".into(), self.mode.map(|s| json!(s)).unwrap_or(Value::Null));
        m.insert("versions".into(), self.versions.clone().unwrap_or(Value::Null));
        m.insert(
            "blocked".into(),
            self.blocked.as_ref().map(TargetBlocked::to_json).unwrap_or(Value::Null),
        );
        m.insert("items".into(), Value::Array(self.items.clone()));
        Value::Object(m)
    }
    /// The reduced shape main() emits for a blocked recheck (l. 4297).
    pub fn to_recheck_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("kind".into(), json!(self.kind));
        m.insert("target_root".into(), json!(self.target_root.to_string_lossy()));
        m.insert(
            "blocked".into(),
            self.blocked.as_ref().map(TargetBlocked::to_json).unwrap_or(Value::Null),
        );
        m.insert("versions".into(), self.versions.clone().unwrap_or(Value::Null));
        Value::Object(m)
    }
}

#[derive(Debug, Clone)]
pub struct AggBlocked {
    pub status: String,
    pub reason: String,
    pub forceable: bool,
    pub versions: Value,
}

#[derive(Debug, Clone, Default)]
pub struct LegacyRefresh {
    pub target_root: PathBuf,
    pub items: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct SkillSync {
    pub source_root: PathBuf,
    pub targets: Vec<SkillTarget>,
    pub blocked: Option<AggBlocked>,
    pub legacy_refresh: Option<LegacyRefresh>,
}

struct TargetInput<'a> {
    real_repo: &'a Path,
    source_root: &'a Path,
    real_source: &'a Path,
    source_version: &'a VersionState,
    host_version: &'a VersionState,
    kind: &'a str,
    target_root: &'a Path,
}

/// computeSkillSyncTarget (l. 1210): one target's resolution + the D3
/// three-version preflight. Fully read-only.
fn compute_skill_sync_target(input: TargetInput<'_>) -> SkillTarget {
    let mut target = SkillTarget {
        kind: input.kind.to_string(),
        target_root: input.target_root.to_path_buf(),
        mode: None,
        versions: None,
        blocked: None,
        items: Vec::new(),
    };
    let refuse = |mut t: SkillTarget, reason: String| -> SkillTarget {
        t.versions = Some(unknown_versions_triple());
        t.blocked = Some(TargetBlocked {
            status: "blocked_no_source".into(),
            reason,
            forceable: false,
        });
        t
    };

    // Never realpath a nonexistent target (absent target = fresh install);
    // ancestor overlap fails closed (F6).
    let target_exists = exists(input.target_root);
    let real_target = if target_exists {
        realpath(input.target_root).unwrap_or_else(|| input.target_root.to_path_buf())
    } else {
        path_resolve(&input.target_root.to_string_lossy())
    };

    if input.kind == "global" {
        if real_target == input.real_repo
            || is_under(input.real_repo, &real_target)
            || is_under(&real_target, input.real_repo)
        {
            return refuse(target, "repo root and the global skills root overlap (one contains the other) - a repo inside the managed skill target must never be touched by its own onboard, refusing fail-closed".into());
        }
    } else if target_exists && !is_under(&real_target, input.real_repo) {
        return refuse(
            target,
            format!(
                "managed in-repo skills root {} resolves outside the repo root - refusing fail-closed",
                repo_target_segments_joined(input.kind)
            ),
        );
    }

    if target_exists && input.real_source == real_target {
        target.mode = Some("noop"); // running the installed copy itself (D2)
    } else if is_under(&real_target, input.real_source) || is_under(input.real_source, &real_target)
    {
        return refuse(
            target,
            "source and target skill roots overlap (one contains the other) - refusing fail-closed"
                .into(),
        );
    } else {
        target.mode = Some(if target_exists { "sync" } else { "fresh" });
    }

    // Three-version preflight (D3), per target.
    let installed_hive = input.target_root.join("bee-hive");
    let mut installed_tree_exists = false;
    if target_exists {
        match super::util::read_dir_sorted_checked(input.target_root) {
            Ok(entries) => {
                installed_tree_exists = entries.iter().any(|e| e.name.starts_with("bee-"));
            }
            Err(()) => installed_tree_exists = true, // unreadable target: fail closed
        }
    }
    let stamp_path = input.target_root.join(SKILLS_VERSION_STAMP);
    let stamp_present = lstat_if_exists(&stamp_path).is_some();
    let installed_version = if target.mode == Some("noop") {
        input.source_version.clone()
    } else if stamp_present {
        read_manifest_version_strict(&stamp_path)
    } else {
        read_version_strict(
            &installed_hive.join("templates").join("lib").join("state.mjs"),
            installed_tree_exists,
            Some(input.target_root),
        )
    };
    target.versions = Some(json!({
        "source": input.source_version.label(),
        "host_helpers": input.host_version.label(),
        "installed_skills": installed_version.label(),
    }));

    let unknowns: Vec<&str> = [
        ("source", input.source_version),
        ("host_helpers", input.host_version),
        ("installed_skills", &installed_version),
    ]
    .into_iter()
    .filter(|(_, v)| v.is_unknown())
    .map(|(n, _)| n)
    .collect();
    if !unknowns.is_empty() {
        target.blocked = Some(TargetBlocked {
            status: "blocked_downgrade".into(),
            reason: format!(
                "version unresolvable for {}: tree exists but its version cannot be read - refusing (never forceable)",
                unknowns.join(", ")
            ),
            forceable: false,
        });
        return target;
    }
    let mut older: Vec<String> = Vec::new();
    if let (Some(sv), Some(hv)) = (input.source_version.value(), input.host_version.value()) {
        if compare_versions(sv, hv).is_lt() {
            older.push(format!("host_helpers {hv}"));
        }
    }
    if let (Some(sv), Some(iv)) = (input.source_version.value(), installed_version.value()) {
        if compare_versions(sv, iv).is_lt() {
            older.push(format!("installed_skills {iv}"));
        }
    }
    if !older.is_empty() {
        let all_numeric = input.source_version.is_resolved()
            && input.host_version.is_resolved()
            && installed_version.is_resolved();
        let tail = if all_numeric {
            " - refusing (--force-downgrade overrides after review)"
        } else {
            " - refusing (not forceable: not all versions resolved numeric)"
        };
        target.blocked = Some(TargetBlocked {
            status: "blocked_downgrade".into(),
            reason: format!(
                "source {} is older than {}{tail}",
                input.source_version.value().unwrap_or("unknown"),
                older.join(" and ")
            ),
            forceable: all_numeric,
        });
    }

    if matches!(target.mode, Some("sync") | Some("fresh"))
        && target.blocked.as_ref().is_none_or(|b| b.forceable)
    {
        target.items =
            compute_skill_items(input.source_root, input.target_root, runtime_for_target_kind(input.kind))
                .into_iter()
                .map(|i| with_target(i, input.kind))
                .collect();
    }
    target
}

/// aggregateSkillBlocked (l. 1382): blocked-first across targets.
fn aggregate_skill_blocked(targets: &[SkillTarget]) -> Option<AggBlocked> {
    let blocked: Vec<&SkillTarget> = targets.iter().filter(|t| t.blocked.is_some()).collect();
    if blocked.is_empty() {
        return None;
    }
    let multi = blocked.len() > 1 || targets.len() > 1;
    let reasons: Vec<String> = blocked
        .iter()
        .map(|t| {
            let b = t.blocked.as_ref().unwrap();
            if multi {
                format!("[{}] {}", t.kind, b.reason)
            } else {
                b.reason.clone()
            }
        })
        .collect();
    let first = blocked[0];
    Some(AggBlocked {
        status: first.blocked.as_ref().unwrap().status.clone(),
        reason: reasons.join("; "),
        forceable: blocked.iter().all(|t| t.blocked.as_ref().unwrap().forceable),
        versions: first.versions.clone().unwrap_or(Value::Null),
    })
}

/// hostLibDowngradeBlock (l. 1412): target-independent runtime-lib guard
/// (VER-02..06).
/// VER-04's downgrade guard. R6 CUTOVER: `host_version` is now read from
/// `.bee/onboarding.json`'s `bee_version` rather than the deleted
/// `.bee/bin/lib/state.mjs`, so the messages name the marker that actually
/// exists. The GUARD is unchanged: an installed host that is NEWER than the
/// source refuses (forceably), and an installed host whose version cannot be
/// resolved refuses outright.
fn host_lib_downgrade_block(
    source_version: &VersionState,
    host_version: &VersionState,
) -> Option<AggBlocked> {
    if matches!(host_version, VersionState::Absent) {
        return None; // VER-04: fresh install
    }
    let versions = json!({
        "source": source_version.label(),
        "host_helpers": host_version.label(),
        "installed_skills": host_version.label(),
    });
    if !source_version.is_resolved() || !host_version.is_resolved() {
        return Some(AggBlocked {
            status: "blocked_downgrade".into(),
            reason: format!(
                "installed bee version in .bee/onboarding.json unresolvable (source {}, installed {}) - refusing (never forceable)",
                source_version.label(),
                host_version.label()
            ),
            forceable: false,
            versions,
        });
    }
    if compare_versions(source_version.value().unwrap(), host_version.value().unwrap()).is_lt() {
        return Some(AggBlocked {
            status: "blocked_downgrade".into(),
            reason: format!(
                "source {} is older than the installed bee {} recorded in .bee/onboarding.json - refusing (--force-downgrade overrides after review)",
                source_version.value().unwrap(),
                host_version.value().unwrap()
            ),
            forceable: true,
            versions,
        });
    }
    None
}

/// computeLegacyGlobalRefresh (l. 1463): strictly additive best-effort pass
/// over ~/.claude/skills. Never creates, never deletes, never blocks.
fn compute_legacy_global_refresh(
    source_root: &Path,
    real_source: &Path,
    real_repo: &Path,
    source_version: &VersionState,
) -> LegacyRefresh {
    let global_root = skills_target_root();
    let mut out = LegacyRefresh { target_root: global_root.clone(), items: Vec::new() };
    if !exists(&global_root) {
        return out; // nothing installed there -> never create
    }
    let Some(real_global) = realpath(&global_root) else { return out };
    if real_source == real_global {
        return out; // never self-copy
    }
    if real_repo == real_global
        || is_under(real_repo, &real_global)
        || is_under(&real_global, real_repo)
        || is_under(&real_global, real_source)
        || is_under(real_source, &real_global)
    {
        return out;
    }
    let global_stamp = global_root.join(SKILLS_VERSION_STAMP);
    let installed_version = if lstat_if_exists(&global_stamp).is_some() {
        read_manifest_version_strict(&global_stamp)
    } else {
        read_version_strict(
            &global_root.join("bee-hive").join("templates").join("lib").join("state.mjs"),
            true,
            Some(&global_root),
        )
    };
    if let (Some(sv), Some(iv)) = (source_version.value(), installed_version.value()) {
        if compare_versions(sv, iv).is_lt() {
            return out; // never downgrade the legacy global
        }
    }
    for it in compute_skill_items(source_root, &global_root, "claude") {
        let action = it["action"].as_str().unwrap_or("").to_string();
        if action == "remove_skill" {
            continue; // never delete from the legacy global root
        }
        let skill = it["skill"].as_str().unwrap_or("").to_string();
        let Some(st) = lstat_if_exists(&global_root.join(&skill)) else { continue };
        if action == "sync_skill" {
            if st.is_symlink || !st.is_dir {
                continue;
            }
            let mut refreshed = it.clone();
            let obj = refreshed.as_object_mut().unwrap();
            obj.insert("action".into(), json!("refresh_legacy_global_skill"));
            obj.insert("scope".into(), json!("installed"));
            obj.insert("target".into(), json!("legacy-global"));
            out.items.push(refreshed);
            continue;
        }
        out.items.push(with_target(it, "legacy-global"));
    }
    out
}

/// computeSkillSync (l. 1547): D2 resolution over ALL sync targets.
pub fn compute_skill_sync(engine: &Engine, repo_root: &Path, global_skills: bool) -> SkillSync {
    let source_root = engine.skills_root.clone();
    let target_specs = skill_sync_targets(repo_root, global_skills);

    let block_all = |reason: String, status: &str| -> SkillSync {
        let blocked = TargetBlocked {
            status: status.to_string(),
            reason: reason.clone(),
            forceable: false,
        };
        SkillSync {
            source_root: source_root.clone(),
            targets: target_specs
                .iter()
                .map(|(kind, root)| SkillTarget {
                    kind: kind.clone(),
                    target_root: root.clone(),
                    mode: None,
                    versions: Some(unknown_versions_triple()),
                    blocked: Some(blocked.clone()),
                    items: Vec::new(),
                })
                .collect(),
            blocked: Some(AggBlocked {
                status: status.to_string(),
                reason,
                forceable: false,
                versions: unknown_versions_triple(),
            }),
            legacy_refresh: None,
        }
    };

    // Identity anchor (F2): the engine and skills tree must share one
    // legitimate package root.
    let identity_ok = (|| {
        let bee_hive = realpath(&source_root.join("bee-hive"))?;
        let real_plugin_root = realpath(&engine.plugin_root)?;
        let contained = bee_hive != real_plugin_root && is_under(&bee_hive, &real_plugin_root);
        // R6 CUTOVER: the anchor payload used to be `packages/bee/lib/state.mjs`.
        // It is now the AGENTS block template — the same tree (`templates_dir`),
        // the same "this root really is a bee engine" question, and the file
        // `Engine::locate` already keys on, so the two can never disagree about
        // what counts as a package root.
        let payload_readable = exists(&engine.agents_block_template);
        Some(contained && payload_readable)
    })()
    .unwrap_or(false);
    if !identity_ok {
        return block_all(
            "no authoritative skill source: the engine and skills tree do not share one legitimate package root".into(),
            "blocked_no_source",
        );
    }

    // Whole-tree marker-grammar gate (D9).
    let marker_errors = validate_skill_tree_markers(&source_root);
    if !marker_errors.is_empty() {
        return block_all(
            format!(
                "skill source markers are malformed - refusing to render, zero writes: {}",
                marker_errors.join("; ")
            ),
            "blocked_render",
        );
    }

    let real_source = realpath(&source_root).unwrap_or_else(|| source_root.clone());
    let real_repo =
        realpath(repo_root).unwrap_or_else(|| path_resolve(&repo_root.to_string_lossy()));

    // R6 CUTOVER: source version from `.claude-plugin/plugin.json`, host
    // version from `.bee/onboarding.json` — see onboard::source for why the
    // two `.mjs` markers these replace could not simply be dropped.
    let source_version = read_manifest_version_strict(
        &engine.plugin_root.join(".claude-plugin").join("plugin.json"),
    );
    let host_version = read_host_version_strict(repo_root);

    let mut targets = Vec::new();
    for (kind, target_root) in &target_specs {
        targets.push(compute_skill_sync_target(TargetInput {
            real_repo: &real_repo,
            source_root: &source_root,
            real_source: &real_source,
            source_version: &source_version,
            host_version: &host_version,
            kind,
            target_root,
        }));
    }
    let mut blocked = aggregate_skill_blocked(&targets);
    if blocked.is_none() {
        blocked = host_lib_downgrade_block(&source_version, &host_version);
    }
    let legacy_refresh = if global_skills {
        None
    } else {
        Some(compute_legacy_global_refresh(
            &source_root,
            &real_source,
            &real_repo,
            &source_version,
        ))
    };
    SkillSync { source_root, targets, blocked, legacy_refresh }
}

/// blockedSourceIdentitySkillSync (l. 2980).
pub fn blocked_source_identity_skill_sync(
    engine: &Engine,
    repo_root: &Path,
    sync_skills: bool,
    global_skills: bool,
    identity: &ReleaseIdentity,
) -> SkillSync {
    let target_specs =
        if sync_skills { skill_sync_targets(repo_root, global_skills) } else { Vec::new() };
    let versions = json!({
        "source": identity.components[0].1.label(),
        "host_helpers": "unknown",
        "installed_skills": "unknown",
    });
    let b = identity.blocked.as_ref().expect("called only for a blocked identity");
    let per_target =
        TargetBlocked { status: b.status.into(), reason: b.reason.clone(), forceable: b.forceable };
    SkillSync {
        source_root: engine.skills_root.clone(),
        targets: target_specs
            .into_iter()
            .map(|(kind, target_root)| SkillTarget {
                kind,
                target_root,
                mode: None,
                versions: Some(versions.clone()),
                blocked: Some(per_target.clone()),
                items: Vec::new(),
            })
            .collect(),
        blocked: Some(AggBlocked {
            status: b.status.into(),
            reason: b.reason.clone(),
            forceable: b.forceable,
            versions,
        }),
        legacy_refresh: None,
    }
}

// ── apply ──────────────────────────────────────────────────────────────────

/// applySyncSkill (l. 1672): mirror one bee-* skill dir into the target,
/// re-verifying the symlink/alias policy at apply time so plan-to-apply races
/// fail closed.
pub fn apply_sync_skill(
    source_root: &Path,
    target_root: &Path,
    name: &str,
    runtime: &str,
) -> Option<String> {
    let render = |buf: &[u8]| render_skill_bytes(buf, runtime);
    let source_dir = source_root.join(name);
    let source_stat = lstat_if_exists(&source_dir);
    if !source_stat.is_some_and(|s| !s.is_symlink && s.is_dir) {
        return Some(format!("source {name} is not a plain directory - skipped"));
    }
    let source_walk = walk_skill_tree(&source_dir, Some(&render));
    if let Some(b) = &source_walk.blocked {
        return Some(format!("source {name} contains a {} at {} - skipped", b.reason, b.path));
    }
    if detect_alias_collisions(&[name.to_string()], target_root).iter().any(|n| n == name) {
        return Some(format!("installed {name} shares one physical entry with a differently-named bee-* entry (case-insensitive alias) - skipped, never sync-then-delete"));
    }
    let target_dir = target_root.join(name);
    let mut target_stat = lstat_if_exists(&target_dir);
    if target_stat.is_some_and(|s| s.is_symlink) {
        return Some(format!("installed {name} is a symlink (plausibly a live checkout) - skipped, never written through or unlinked"));
    }
    let mut target_walk = Walk::default();
    if target_stat.is_some_and(|s| s.is_dir) {
        let walked = walk_skill_tree(&target_dir, None);
        if let Some(b) = &walked.blocked {
            return Some(format!("installed {name} contains a {} at {} - skipped, nothing inside it written or deleted", b.reason, b.path));
        }
        target_walk = walked;
    } else if target_stat.is_some() {
        // non-link type collision: remove the entry, write the source shape
        let _ = std::fs::remove_file(&target_dir);
        target_stat = None;
    }
    let _ = target_stat;
    if let Some((a, b)) = detect_nested_alias(&target_dir, &source_walk, &target_walk) {
        return Some(format!("installed {name} has nested entries {a} and {b} resolving to one physical entry (case-insensitive alias) - skipped, never sync-then-delete"));
    }
    // Node lets a failing mkdir throw out to main()'s catch; the port reports
    // it as a loud per-skill skip instead (documented divergence — a V8
    // message is never reproduced, and a silent success would be worse).
    if std::fs::create_dir_all(&target_dir).is_err() {
        return Some(format!("installed {name} could not be created - skipped"));
    }
    // Phase 1 — cleanup, deepest-first, BEFORE materializing anything
    // (review P1-3).
    let mut stale: Vec<(String, bool)> = Vec::new();
    for (rel, _) in &target_walk.files {
        if !source_walk.has_file(rel) {
            stale.push((rel.clone(), false));
        }
    }
    for rel in &target_walk.dirs {
        if !source_walk.dirs.contains(rel) {
            stale.push((rel.clone(), true));
        }
    }
    // JS: sort((a,b) => b.depth - a.depth) — a stable descending-depth sort.
    stale.sort_by(|a, b| {
        b.0.split('/').count().cmp(&a.0.split('/').count())
    });
    for (rel, recursive) in stale {
        let p = join_rel(&target_dir, &rel);
        if recursive {
            let _ = std::fs::remove_dir_all(&p);
        } else {
            let _ = std::fs::remove_file(&p);
        }
    }
    // Phase 2 — materialize the source shape onto the cleaned target.
    for rel in &source_walk.dirs {
        let _ = std::fs::create_dir_all(join_rel(&target_dir, rel));
    }
    for (rel, hash) in &source_walk.files {
        if target_walk.file_hash(rel) == Some(hash.as_str()) {
            continue; // already byte-identical
        }
        let raw = std::fs::read(join_rel(&source_dir, rel)).unwrap_or_default();
        let _ = write_file_atomic_random(&join_rel(&target_dir, rel), &render(&raw));
    }
    None
}

/// applyRemoveSkill (l. 1758).
pub fn apply_remove_skill(target_root: &Path, name: &str) -> Option<String> {
    if !name.starts_with("bee-") {
        return Some(format!("refusing to remove {name}: outside the bee-* namespace"));
    }
    let target_dir = target_root.join(name);
    let Some(st) = lstat_if_exists(&target_dir) else { return None }; // already gone
    if st.is_symlink {
        return Some(format!(
            "installed {name} is a symlink (plausibly a live checkout) - skipped, never unlinked"
        ));
    }
    if !st.is_dir {
        return Some(format!(
            "installed {name} is not a directory - outside the deletion domain, skipped"
        ));
    }
    if detect_alias_collisions(&[name.to_string()], target_root).iter().any(|n| n == name) {
        return Some(format!("installed {name} shares one physical entry with a differently-named bee-* entry (case-insensitive alias) - skipped, never sync-then-delete"));
    }
    let walked = walk_skill_tree(&target_dir, None);
    if let Some(b) = &walked.blocked {
        return Some(format!(
            "installed {name} contains a {} at {} - skipped, nothing deleted",
            b.reason, b.path
        ));
    }
    let _ = std::fs::remove_dir_all(&target_dir);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(p: &Path, body: &str) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    #[test]
    fn fresh_target_plans_one_sync_per_source_skill() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("skills");
        write(&src.join("bee-hive").join("SKILL.md"), "hi");
        write(&src.join("bee-planning").join("SKILL.md"), "plan");
        write(&src.join("not-bee").join("SKILL.md"), "no");
        let target = dir.path().join("target");
        std::fs::create_dir_all(&target).unwrap();

        let items = compute_skill_items(&src, &target, "claude");
        let actions: Vec<&str> = items.iter().map(|i| i["action"].as_str().unwrap()).collect();
        assert_eq!(actions, vec!["sync_skill", "sync_skill"]);
        assert_eq!(items[0]["skill"], "bee-hive");
        assert_eq!(items[0]["scope"], "installed");
        assert_eq!(items[1]["skill"], "bee-planning");
    }

    #[test]
    fn identical_target_plans_nothing_and_drift_replans() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("skills");
        write(&src.join("bee-hive").join("SKILL.md"), "hi");
        let target = dir.path().join("target");
        write(&target.join("bee-hive").join("SKILL.md"), "hi");
        assert!(compute_skill_items(&src, &target, "claude").is_empty());

        write(&target.join("bee-hive").join("SKILL.md"), "drifted");
        let items = compute_skill_items(&src, &target, "claude");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["action"], "sync_skill");
    }

    #[test]
    fn foreign_bee_skill_in_target_is_removed_but_non_bee_is_untouchable() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("skills");
        write(&src.join("bee-hive").join("SKILL.md"), "hi");
        let target = dir.path().join("target");
        write(&target.join("bee-hive").join("SKILL.md"), "hi");
        write(&target.join("bee-legacy").join("SKILL.md"), "old");
        write(&target.join("my-own-skill").join("SKILL.md"), "mine");

        let items = compute_skill_items(&src, &target, "claude");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["action"], "remove_skill");
        assert_eq!(items[0]["skill"], "bee-legacy");
    }

    #[test]
    fn apply_sync_mirrors_and_prunes_then_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("skills");
        write(&src.join("bee-hive").join("SKILL.md"), "hi");
        write(&src.join("bee-hive").join("references").join("a.md"), "A");
        let target = dir.path().join("target");
        write(&target.join("bee-hive").join("stale.md"), "junk");
        write(&target.join("bee-hive").join("old").join("deep.md"), "junk");

        assert_eq!(apply_sync_skill(&src, &target, "bee-hive", "claude"), None);
        assert_eq!(
            std::fs::read_to_string(target.join("bee-hive").join("SKILL.md")).unwrap(),
            "hi"
        );
        assert_eq!(
            std::fs::read_to_string(target.join("bee-hive").join("references").join("a.md")).unwrap(),
            "A"
        );
        assert!(!target.join("bee-hive").join("stale.md").exists());
        assert!(!target.join("bee-hive").join("old").exists());
        // Idempotent: a second apply plans nothing.
        assert!(compute_skill_items(&src, &target, "claude").is_empty());
        assert_eq!(apply_sync_skill(&src, &target, "bee-hive", "claude"), None);
        assert!(compute_skill_items(&src, &target, "claude").is_empty());
    }

    #[test]
    fn remove_skill_refuses_outside_the_bee_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        write(&target.join("mine").join("x.md"), "x");
        let blocked = apply_remove_skill(&target, "mine").unwrap();
        assert!(blocked.contains("outside the bee-* namespace"));
        assert!(target.join("mine").exists());
    }

    #[test]
    fn codex_target_renders_the_codex_arm() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("skills");
        write(
            &src.join("bee-hive").join("SKILL.md"),
            "x\n<!-- bee:only claude -->\nC\n<!-- bee:end -->\n<!-- bee:only codex -->\nK\n<!-- bee:end -->\n",
        );
        let target = dir.path().join("agents");
        assert_eq!(apply_sync_skill(&src, &target, "bee-hive", "codex"), None);
        assert_eq!(
            std::fs::read_to_string(target.join("bee-hive").join("SKILL.md")).unwrap(),
            "x\nK\n"
        );
        // And the drift check agrees (no perpetual re-sync).
        assert!(compute_skill_items(&src, &target, "codex").is_empty());
        // The claude render differs, so a claude-runtime compare shows drift.
        assert_eq!(compute_skill_items(&src, &target, "claude").len(), 1);
    }

    /// D1 (opencode-support oc-4): the ONBOARDING SYNC PATH's per-target
    /// writer was already generic on `runtime` (proven above for "codex");
    /// this pins that "opencode" is a real third arm, not just an
    /// unvalidated string that happened to fall through.
    #[test]
    fn opencode_target_renders_the_opencode_arm() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("skills");
        write(
            &src.join("bee-hive").join("SKILL.md"),
            "x\n<!-- bee:only claude -->\nC\n<!-- bee:end -->\n<!-- bee:only opencode -->\nO\n<!-- bee:end -->\n",
        );
        let target = dir.path().join("opencode-skills");
        assert_eq!(apply_sync_skill(&src, &target, "bee-hive", "opencode"), None);
        assert_eq!(
            std::fs::read_to_string(target.join("bee-hive").join("SKILL.md")).unwrap(),
            "x\nO\n"
        );
        assert!(compute_skill_items(&src, &target, "opencode").is_empty());
        // Neither claude nor codex would have rendered the opencode-only
        // block, so a runtime-mismatched compare still shows drift.
        assert_eq!(compute_skill_items(&src, &target, "claude").len(), 1);
        assert_eq!(compute_skill_items(&src, &target, "codex").len(), 1);
    }

    #[test]
    fn host_lib_downgrade_guard_states() {
        let older = VersionState::Resolved("1.0.0".into());
        let newer = VersionState::Resolved("2.0.0".into());
        assert!(host_lib_downgrade_block(&newer, &VersionState::Absent).is_none());
        assert!(host_lib_downgrade_block(&newer, &older).is_none());
        let b = host_lib_downgrade_block(&older, &newer).unwrap();
        assert!(b.forceable);
        assert!(b.reason.contains("is older than the installed bee"));
        let b = host_lib_downgrade_block(&older, &VersionState::Unknown).unwrap();
        assert!(!b.forceable);
        assert!(b.reason.contains("never forceable"));
    }

    #[test]
    fn aggregate_names_every_blocked_target() {
        let mk = |kind: &str, reason: &str, forceable: bool| SkillTarget {
            kind: kind.into(),
            target_root: PathBuf::from("x"),
            mode: None,
            versions: Some(unknown_versions_triple()),
            blocked: Some(TargetBlocked {
                status: "blocked_downgrade".into(),
                reason: reason.into(),
                forceable,
            }),
            items: vec![],
        };
        let agg = aggregate_skill_blocked(&[mk("repo-claude", "r1", true), mk("repo-agents", "r2", false)])
            .unwrap();
        assert_eq!(agg.reason, "[repo-claude] r1; [repo-agents] r2");
        assert!(!agg.forceable, "one non-forceable target makes the aggregate non-forceable");
    }

    // ── the real .opencode/skills/ projection (oc-4) ────────────────────────
    //
    // Mirrors devtools::skill_trees's committed-tree pin: re-render the
    // canonical skills/ source for "opencode" and byte-compare against the
    // tree this cell committed, catching future drift the same way that
    // pipeline's own pin catches drift in `.claude-plugin/`/`.codex-plugin/`.

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..").join("..")
    }

    fn opencode_sidecar_bytes(source_root: &Path) -> Vec<u8> {
        let entries = super::super::render::source_skill_digest_entries(source_root, "opencode");
        let sidecar = super::super::render::build_render_sidecar("opencode", &entries);
        format!("{}\n", crate::jsjson::stringify_pretty(&sidecar)).into_bytes()
    }

    /// Regen entry point for `.opencode/skills/` (oc-4, S2). `bee onboard
    /// --apply` (oc-13, S5: `REPO_SKILL_TARGETS`'s `repo-opencode` entry) now
    /// drives this same runtime-agnostic writer for real host repos,
    /// including this checkout itself — running it from inside a bee source
    /// checkout is the ordinary regen path. This test remains as the
    /// lower-ceremony one-liner for regenerating the COMMITTED tree below
    /// without going through the full CLI plan/apply cycle. `#[ignore]`d
    /// because it writes the REAL checkout: run explicitly with `cargo test
    /// --manifest-path packages/bee-rs/Cargo.toml
    /// onboard::skills::tests::regen_opencode_skills_tree -- --ignored`
    /// whenever the canonical `skills/` source changes.
    #[test]
    #[ignore]
    fn regen_opencode_skills_tree() {
        let root = repo_root();
        let source_root = root.join("skills");
        assert!(source_root.is_dir(), "not a bee source checkout");
        let target_root = root.join(".opencode").join("skills");
        std::fs::create_dir_all(&target_root).unwrap();
        for entry in list_bee_skill_entries(&source_root) {
            if entry.is_symlink || !entry.is_dir {
                continue;
            }
            let skipped = apply_sync_skill(&source_root, &target_root, &entry.name, "opencode");
            assert_eq!(skipped, None, "opencode sync skipped {}: {skipped:?}", entry.name);
        }
        super::super::util::write_file_atomic(
            &target_root.join(super::super::templates::RENDER_SIDECAR),
            &opencode_sidecar_bytes(&source_root),
        )
        .unwrap();
        // oc-13: real `bee onboard --apply` also stamps SKILLS_VERSION_STAMP
        // (apply.rs's "D9/D7 provenance stamps" section) — without it, the
        // three-version preflight cannot resolve `installed_skills` for this
        // target and blocks EVERY future `--apply` against this checkout
        // with "version unresolvable ... refusing (never forceable)". The
        // interim regen path skipped it; a real onboard apply never would.
        let engine = super::super::source::Engine::from_plugin_root(root.clone());
        let version = super::super::source::read_source_release_identity(&engine).version;
        let payload = serde_json::json!({
            "version": version.as_ref().map(|v| serde_json::json!(v)).unwrap_or(serde_json::Value::Null)
        });
        super::super::util::write_file_atomic(
            &target_root.join(super::super::templates::SKILLS_VERSION_STAMP),
            format!("{}\n", crate::jsjson::stringify_pretty(&payload)).as_bytes(),
        )
        .unwrap();
    }

    /// THE PIN: re-renders skills/ for "opencode" and byte-compares against
    /// the committed `.opencode/skills/` tree.
    #[test]
    fn opencode_projection_matches_the_committed_tree() {
        let root = repo_root();
        let source_root = root.join("skills");
        if !source_root.is_dir() {
            return; // not a source checkout
        }
        let target_root = root.join(".opencode").join("skills");
        assert!(
            target_root.is_dir(),
            ".opencode/skills/ must exist — run `regen_opencode_skills_tree` (see its doc comment)"
        );
        assert!(
            compute_skill_items(&source_root, &target_root, "opencode").is_empty(),
            ".opencode/skills/ has drifted from skills/ — re-run the opencode regen"
        );
        let committed_sidecar =
            std::fs::read(target_root.join(super::super::templates::RENDER_SIDECAR)).expect("sidecar");
        assert_eq!(
            committed_sidecar,
            opencode_sidecar_bytes(&source_root),
            "opencode sidecar drifted"
        );
    }

    // ── the skills-version stamp trap (compounding-batch cmp-2) ─────────────
    //
    // A rendered skills root that EXISTS without `SKILLS_VERSION_STAMP` puts
    // every future `bee onboard --apply` against it into `blocked_downgrade`
    // (the three-version preflight above, ~l. 420-458: `installed_skills`
    // cannot resolve without either the stamp or a readable `bee-hive/
    // templates/lib/state.mjs`, and an unresolvable version blocks with
    // "refusing (never forceable)"). Before this test, the only place that
    // even NAMED the trap was a comment inside `regen_opencode_skills_tree`
    // above's `#[ignore]`d body (oc-13) — nothing asserted it, for that
    // target or the other two. This test is NOT `#[ignore]`d: it checks
    // every `REPO_SKILL_TARGETS` root present in THIS checkout at once,
    // never a hand-picked one.

    /// For every `REPO_SKILL_TARGETS` root present in this checkout (all
    /// three are git-tracked here, so a normal clone carries all three), its
    /// rendered tree must carry `SKILLS_VERSION_STAMP`. A root that has
    /// never been rendered anywhere (a fresh, pre-onboard host repo) is
    /// skipped — there is no tree to trap yet — but this checkout's own
    /// three roots are exactly the case the trap bites, so `present == 0`
    /// here means the derivation itself broke, not a legitimately empty run.
    #[test]
    fn every_present_repo_skill_target_carries_its_version_stamp() {
        let root = repo_root();
        let mut missing: Vec<String> = Vec::new();
        let mut present = 0usize;
        for (kind, segments) in super::super::templates::REPO_SKILL_TARGETS {
            let mut target_root = root.clone();
            for seg in *segments {
                target_root = target_root.join(seg);
            }
            if !target_root.is_dir() {
                continue; // never rendered here — nothing to trap
            }
            present += 1;
            let stamp = target_root.join(super::super::templates::SKILLS_VERSION_STAMP);
            if !stamp.is_file() {
                missing.push(format!("{kind} ({})", target_root.display()));
            }
        }
        assert!(
            present > 0,
            "expected at least one REPO_SKILL_TARGETS root ({:?}) to exist under {} — the \
             derivation likely broke, or this is not a source checkout",
            super::super::templates::REPO_SKILL_TARGETS,
            root.display()
        );
        assert!(
            missing.is_empty(),
            "REPO_SKILL_TARGETS root(s) present without their version stamp \
             ({}) — every future `bee onboard --apply` against them blocks with \
             blocked_downgrade, never forceable (skills.rs's three-version preflight, ~l. 420-458): {}",
            super::super::templates::SKILLS_VERSION_STAMP,
            missing.join(", ")
        );
    }
}
