// onboard::plan — computePlan and the managed-hash ledger.
//
// Provenance: onboard_bee.mjs listTemplateHelpers (l. 1794),
// listTemplateLibModules (l. 1826), listSourceExpertise (l. 1848, RECURSIVE
// since expertise gained progressive disclosure), listTemplatePrompts
// (l. 1871), listTemplateStatusline (l. 1882), listPluginHooks (l. 2070),
// computePlan (l. 3006), coreChangesNeeded (l. 3389), buildHookVersions
// (l. 3397), buildManagedVersions (l. 3408) and subsetManaged (l. 3476).
//
// Plan-item key order is load-bearing: `--json` prints the plan verbatim
// (contract C2), so every item is built with an ordered map in the same
// order the JS object literal declares.

use super::agents::{compute_agent_file_plan, compute_opencode_agent_file_plan};
use super::hooks_wiring as hw;
use super::merge::{
    agents_block_present, claude_md_imports_agents, extract_agents_block,
    extract_gitignore_block, gitignore_block_present, host_shell_is_powershell,
    normalize_gitignore_for_compare, render_agents_block, render_gitignore_block,
};
use super::migration::{detect_worktree_migration, WorktreeMigration};
use super::notices::has_prose_outside_block;
use super::render::{
    render_skill_bytes, runtime_for_target_kind, validate_skill_markers, walk_skill_tree, Walk,
};
use super::skills::{blocked_source_identity_skill_sync, compute_skill_sync, SkillSync};
use super::source::{read_source_release_identity, Engine};
use super::templates as T;
use super::util::{
    exists, hash_file, is_under, join_rel, lstat_if_exists, read_dir_sorted,
    read_dir_sorted_checked, read_json_if_exists, read_text_if_exists, realpath, sha256_str,
};
use crate::jsjson;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Options {
    pub repo_hooks: bool,
    pub claude_md: bool,
    pub global_skills: bool,
    pub sync_skills: bool,
    pub force_downgrade: bool,
    pub plugin_source: bool,
    pub runtime: String,
}

pub struct ComputedPlan {
    pub plan: Vec<Value>,
    pub bee_version: Option<String>,
    pub rendered_block: String,
    pub rendered_gitignore_block: String,
    pub desired_managed: Value,
    pub skill_sync: SkillSync,
    pub codex_hybrid: bool,
    pub worktree_migration: WorktreeMigration,
}

fn plan_item(action: &str, path: &str) -> Value {
    let mut m = Map::new();
    m.insert("action".into(), json!(action));
    m.insert("path".into(), json!(path));
    Value::Object(m)
}

// ── source enumeration ─────────────────────────────────────────────────────

/// listTemplateHelpers (l. 1794): top-level `*.mjs` in packages/bee, sorted.
pub fn list_template_helpers(engine: &Engine) -> Vec<String> {
    list_files_by_suffix(&engine.templates_dir, ".mjs")
}

/// listTemplateLibModules (l. 1826).
pub fn list_template_lib_modules(engine: &Engine) -> Vec<String> {
    list_files_by_suffix(&engine.templates_lib_dir, ".mjs")
}

/// listTemplatePrompts (l. 1871).
pub fn list_template_prompts(engine: &Engine) -> Vec<String> {
    list_files_by_suffix(&engine.templates_prompts_dir, ".md")
}

/// listTemplateStatusline (l. 1882): every plain file, sorted.
pub fn list_template_statusline(engine: &Engine) -> Vec<String> {
    if !exists(&engine.templates_statusline_dir) {
        return Vec::new();
    }
    read_dir_sorted(&engine.templates_statusline_dir)
        .into_iter()
        .filter(|e| e.is_file)
        .map(|e| e.name)
        .collect()
}

/// opencode-support oc-13 (S5): the OpenCode guard plugin file(s) this
/// checkout ships at `.opencode/plugins/` — every plain `.ts` file, sorted.
/// Vendored into a host project the same "copy when missing or drifted" way
/// helpers/lib/prompts are, never through the rendered skill-tree pipeline
/// (that pipeline's `RENDER_RUNTIMES` deliberately excludes opencode — no
/// marketplace tree exists for it).
pub fn list_opencode_plugin_files(engine: &Engine) -> Vec<String> {
    if !exists(&engine.opencode_plugin_dir) {
        return Vec::new();
    }
    read_dir_sorted(&engine.opencode_plugin_dir)
        .into_iter()
        .filter(|e| e.is_file && e.name.ends_with(".ts"))
        .map(|e| e.name)
        .collect()
}

/// pi-support D1: the Pi guard extension file(s) this checkout ships at
/// `.pi/extensions/` — every plain `.ts` file, sorted. The exact shape of
/// `list_opencode_plugin_files` above, for the exact same reason: a
/// hand-written TypeScript belt is vendored "copy when missing or drifted",
/// never rendered by the skill-tree pipeline (which has no pi target).
pub fn list_pi_extension_files(engine: &Engine) -> Vec<String> {
    if !exists(&engine.pi_extension_dir) {
        return Vec::new();
    }
    read_dir_sorted(&engine.pi_extension_dir)
        .into_iter()
        .filter(|e| e.is_file && e.name.ends_with(".ts"))
        .map(|e| e.name)
        .collect()
}

fn list_files_by_suffix(dir: &Path, suffix: &str) -> Vec<String> {
    if !exists(dir) {
        return Vec::new();
    }
    let mut out: Vec<String> = read_dir_sorted(dir)
        .into_iter()
        .filter(|e| e.is_file && e.name.ends_with(suffix))
        .map(|e| e.name)
        .collect();
    out.sort();
    out
}

/// listSourceExpertise (l. 1848) — RECURSIVE. A guide may carry a
/// `<topic>/patterns/*.md` directory whose files it indexes with per-file
/// load triggers; a host that vendors only the top-level *.md gets an index
/// pointing at files it does not have. Names are POSIX-relative
/// ("tests/patterns/differential-testing.md") so every caller keeps treating
/// a name as the identity of ONE vendored guide.
pub fn list_source_expertise(engine: &Engine) -> Vec<String> {
    if !exists(&engine.expertise_dir) {
        return Vec::new();
    }
    let mut out = Vec::new();
    walk_md_tree(&engine.expertise_dir, "", &mut out);
    out.sort();
    out
}

/// The same recursive `*.md` enumeration applied to a VENDORED tree — the
/// stale-removal domain of section 3d (l. 3179).
pub fn list_vendored_expertise(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    walk_md_tree(dir, "", &mut out);
    out.sort();
    out
}

fn walk_md_tree(dir: &Path, prefix: &str, out: &mut Vec<String>) {
    // Dirent typing never follows links, so a symlinked file or directory is
    // neither enumerated nor (later) unlinked.
    for entry in read_dir_sorted(dir) {
        let rel =
            if prefix.is_empty() { entry.name.clone() } else { format!("{prefix}/{}", entry.name) };
        if entry.is_dir {
            walk_md_tree(&dir.join(&entry.name), &rel, out);
        } else if entry.is_file && entry.name.ends_with(".md") {
            out.push(rel);
        }
    }
}

/// listPluginHooks (l. 2070): HOOK_FILENAMES order, filtered by existence.
pub fn list_plugin_hooks(engine: &Engine) -> Vec<String> {
    if !exists(&engine.plugin_hooks_dir) {
        return Vec::new();
    }
    T::HOOK_FILENAMES
        .iter()
        .filter(|name| exists(&engine.plugin_hooks_dir.join(name)))
        .map(|n| (*n).to_string())
        .collect()
}

// ── the managed ledger ─────────────────────────────────────────────────────

/// buildHookVersions (l. 3397): shared by managed.repo_hooks and
/// managed.codex_hooks so the two can never drift.
fn build_hook_versions(engine: &Engine) -> Value {
    let mut hooks = Map::new();
    for name in list_plugin_hooks(engine) {
        let h = hash_file(&engine.plugin_hooks_dir.join(&name)).unwrap_or_default();
        hooks.insert(name, json!(h));
    }
    hooks.insert(".codex/hooks.json".into(), json!(sha256_str(&hw::codex_hook_entries_json())));
    Value::Object(hooks)
}

/// Every managed-ledger group, paired with the HOST directory whose contents
/// it fingerprints — i.e. every tree whose hand-edit makes `.bee/onboarding.json`
/// a lie until onboarding re-runs. The two hash-a-string groups
/// (`agents_block`, `gitignore_block`) carry `None`: they cover rendered blocks
/// inside AGENTS.md / .gitignore, not a directory, and a hand-edit of those is
/// caught by onboarding's own block comparison rather than by a path-scoped
/// obligation.
///
/// `ledger_covered_roots()` projects this to the directory set that
/// `verbs/cells.rs`'s regen obligation guards.
///
/// R6 CUTOVER. That obligation used to derive the same set by PARSING
/// `scripts/ledger_parity.mjs` for its `checkGroup(managed.X, "<relDir>")`
/// calls. Parsing a second implementation was the only way to get decision D2's
/// property — the scope is DERIVED from the thing it guards, never pasted next
/// to it. The script is deleted (its runtime check now lives natively in
/// `verbs/status_full.rs`'s `compute_runtime_drift`, which runs on EVERY
/// `bee status` rather than only when someone remembers to invoke a script), so
/// the table is SHARED from the module that builds the ledger instead of
/// re-read from a copy of it: the same property with one fewer implementation.
///
/// `every_managed_group_is_classified` pins this table against
/// `build_managed_versions`, so adding a group without classifying it fails a
/// test rather than silently escaping the regen obligation.
const LEDGER_GROUPS: &[(&str, Option<&str>)] = &[
    ("agents_block", None),
    ("gitignore_block", None),
    ("helpers", Some(".bee/bin")),
    ("lib", Some(".bee/bin/lib")),
    ("expertise", Some(".bee/expertise")),
    ("prompts", Some(".bee/bin/prompts")),
    ("repo_hooks", Some(".bee/bin/hooks")),
    ("codex_hooks", Some(".bee/bin/hooks")),
    ("statusline", Some(".bee/bin/statusline")),
];

pub(crate) fn ledger_covered_roots() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for (_, dir) in LEDGER_GROUPS {
        if let Some(d) = dir {
            if !out.contains(d) {
                out.push(d);
            }
        }
    }
    out
}

/// buildManagedVersions (l. 3408).
pub fn build_managed_versions(
    engine: &Engine,
    rendered_block: &str,
    rendered_gitignore_block: &str,
    repo_hooks: bool,
    statusline: bool,
    codex_hybrid: bool,
) -> Value {
    let hash_map = |dir: &Path, names: Vec<String>| -> Value {
        let mut m = Map::new();
        for name in names {
            m.insert(name.clone(), json!(hash_file(&join_rel(dir, &name)).unwrap_or_default()));
        }
        Value::Object(m)
    };
    let mut managed = Map::new();
    managed.insert("agents_block".into(), json!(sha256_str(rendered_block)));
    managed.insert("gitignore_block".into(), json!(sha256_str(rendered_gitignore_block)));
    managed.insert(
        "helpers".into(),
        hash_map(&engine.templates_dir, list_template_helpers(engine)),
    );
    managed
        .insert("lib".into(), hash_map(&engine.templates_lib_dir, list_template_lib_modules(engine)));
    managed
        .insert("expertise".into(), hash_map(&engine.expertise_dir, list_source_expertise(engine)));
    managed.insert(
        "prompts".into(),
        hash_map(&engine.templates_prompts_dir, list_template_prompts(engine)),
    );
    if repo_hooks {
        managed.insert("repo_hooks".into(), build_hook_versions(engine));
    }
    if codex_hybrid {
        // Advisor R5: a DISTINCT key from repo_hooks.
        managed.insert("codex_hooks".into(), build_hook_versions(engine));
    }
    if statusline {
        let mut pair = Map::new();
        for name in list_template_statusline(engine) {
            let h = hash_file(&engine.templates_statusline_dir.join(&name)).unwrap_or_default();
            pair.insert(name, json!(h));
        }
        managed.insert("statusline".into(), Value::Object(pair));
    }
    Value::Object(managed)
}

/// JS `x || fallback`.
fn js_or(v: Option<&Value>, fallback: Value) -> Value {
    match v {
        Some(Value::Null) | None => fallback,
        Some(Value::Bool(false)) => fallback,
        Some(Value::String(s)) if s.is_empty() => fallback,
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => fallback,
        Some(other) => other.clone(),
    }
}

/// subsetManaged (l. 3476): compare only the parts THIS run manages. Note
/// (faithful to the original): `prompts` is deliberately NOT part of the
/// subset, so a prompt-only change never flips onboarding drift on its own.
pub fn subset_managed(
    managed: Option<&Value>,
    repo_hooks: bool,
    statusline: bool,
    codex_hybrid: bool,
) -> Value {
    let src = managed.filter(|m| m.is_object());
    let get = |k: &str| src.and_then(|m| m.get(k));
    let mut out = Map::new();
    out.insert("agents_block".into(), js_or(get("agents_block"), Value::Null));
    out.insert("gitignore_block".into(), js_or(get("gitignore_block"), Value::Null));
    out.insert("helpers".into(), js_or(get("helpers"), json!({})));
    out.insert("lib".into(), js_or(get("lib"), json!({})));
    out.insert("expertise".into(), js_or(get("expertise"), json!({})));
    if repo_hooks {
        out.insert("repo_hooks".into(), js_or(get("repo_hooks"), json!({})));
    }
    if codex_hybrid {
        out.insert("codex_hooks".into(), js_or(get("codex_hooks"), json!({})));
    }
    if statusline {
        out.insert("statusline".into(), js_or(get("statusline"), json!({})));
    }
    Value::Object(out)
}

/// coreChangesNeeded (l. 3389): the legacy-global refresh never drives
/// up_to_date/changes_needed.
pub fn core_changes_needed(plan: &[Value]) -> bool {
    plan.iter().any(|i| i["action"] != "refresh_legacy_global_skill")
}

// ── .bee/verify/ → every runtime skill home (verification-ships-to-hosts D3) ──
//
// `.bee/verify/<name>/` is SOURCE, and it belongs to the HOST repo: it is a
// `verify-<app>` skill an agent generated with `bee-verifying`, driving that
// repo's real product. Every runtime skill home gets a rendered copy, so ONE
// generation reaches Claude, Codex and OpenCode alike instead of the agent
// hand-writing three duplicates that drift on a teammate's fresh clone.
//
// This path CREATES and it UPDATES. It never removes anything, ever — there is
// no removal item, no removal verb and no removal code path anywhere in it,
// and that is the whole design (plan.md, "Render, never prune"). The `bee-*`
// sync may prune a target directory absent from source because bee MINTS those
// names and nobody else does; `verify-` is a generic English word whose source
// lives in the mutable host repo. Inheriting the mechanism without that
// ownership axiom would inherit only the deletions: a host-authored
// `verify-payments/` would vanish during a routine `bee onboard --apply`, with
// no trash and no journal. Staleness is owned instead by `bee-verify-upkeep`
// (D1), an agent pass running in a git working tree, where a removal is
// visible, reviewable and committable.
//
// It is also strictly ADDITIVE to bee's own skill sync (section 7): a missing,
// empty, unreadable or refused `.bee/verify/` yields zero items and blocks
// nothing, and a malformed host-authored SKILL.md reports only its own item —
// never the whole-tree `blocked_render` refusal `compute_skill_sync` raises
// for the engine's own tree. Host bytes must not be able to stop bee
// installing itself.

/// The `(kind, target root, POSIX relative root)` triple for every runtime
/// skill home, resolved from `REPO_SKILL_TARGETS` — never a hand-written list,
/// so a fourth runtime joins that table and this path follows for free.
pub fn verify_render_targets(repo_root: &Path) -> Vec<(&'static str, PathBuf, String)> {
    T::REPO_SKILL_TARGETS
        .iter()
        .map(|(kind, segments)| {
            let mut p = repo_root.to_path_buf();
            for s in *segments {
                p.push(s);
            }
            (*kind, p, segments.join("/"))
        })
        .collect()
}

/// `<repo>/.bee/verify` — the one source root this path reads.
pub fn verify_source_root(repo_root: &Path) -> PathBuf {
    repo_root.join(".bee").join("verify")
}

/// Root preflight, shared by the planner and the apply arm so the two can
/// never disagree: `Some(reason)` refuses the WHOLE root with zero items and
/// zero writes.
///
/// Absent is not an error — it is the common case, every repo that never
/// generated a verification skill. A symlinked root is refused because
/// rendering through it would write into a tree bee never resolved, and a root
/// resolving onto (or containing, or contained by) a runtime skill home is
/// refused because source and target would be the same tree.
pub fn verify_root_refusal(repo_root: &Path) -> Option<String> {
    let source_root = verify_source_root(repo_root);
    let Some(st) = lstat_if_exists(&source_root) else {
        return Some(".bee/verify is absent - nothing to render".into());
    };
    if st.is_symlink {
        return Some(".bee/verify is a symlink - refused, never followed or written through".into());
    }
    if !st.is_dir {
        return Some(".bee/verify is not a directory - refused, nothing rendered".into());
    }
    let real_source = realpath(&source_root).unwrap_or_else(|| source_root.clone());
    for (_, target_root, rel_root) in verify_render_targets(repo_root) {
        let real_target = realpath(&target_root).unwrap_or(target_root);
        if real_source == real_target
            || is_under(&real_source, &real_target)
            || is_under(&real_target, &real_source)
        {
            return Some(format!(
                ".bee/verify resolves onto {rel_root} - refused, a skill home is never its own source"
            ));
        }
    }
    None
}

/// Per-skill marker grammar, checked against the HOST's bytes. Errors are
/// reported for this entry alone; they never reach the tree-wide refusal.
pub fn verify_marker_errors(source_dir: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let walk = walk_skill_tree(source_dir, None);
    if walk.blocked.is_some() {
        return errors; // the symlink policy answers this one, not the grammar
    }
    for (rel, _) in &walk.files {
        let Ok(buf) = std::fs::read(join_rel(source_dir, rel)) else { continue };
        if !super::render::buf_has_marker_bytes(&buf) {
            continue;
        }
        for e in validate_skill_markers(&String::from_utf8_lossy(&buf)) {
            errors.push(format!("{rel}: {e}"));
        }
    }
    errors
}

/// Drift, measured the copy-only way: every SOURCE file must be present in the
/// target with the rendered bytes, and every source directory must exist. A
/// file the target carries and the source does not is NOT drift — nothing
/// prunes it, so reporting it would plan a write that changes nothing and
/// break idempotence.
fn verify_skill_drifted(source_walk: &Walk, target_walk: &Walk) -> bool {
    source_walk.files.iter().any(|(rel, hash)| target_walk.file_hash(rel) != Some(hash.as_str()))
        || source_walk.dirs.iter().any(|d| !target_walk.dirs.contains(d))
}

fn verify_item(action: &str, skill: &str, path: &str, scope: &str, target: &str) -> Value {
    let mut m = Map::new();
    m.insert("action".into(), json!(action));
    m.insert("skill".into(), json!(skill));
    m.insert("path".into(), json!(path));
    m.insert("scope".into(), json!(scope));
    m.insert("target".into(), json!(target));
    Value::Object(m)
}

fn verify_blocked_item(skill: &str, path: &str, scope: &str, target: &str, reason: String) -> Value {
    let mut v = verify_item("blocked_verify_skill", skill, path, scope, target);
    v.as_object_mut().unwrap().insert("reason".into(), json!(reason));
    v
}

/// The copy-only planner: one `copy_verify_skill` item per (source entry,
/// runtime home) pair that is missing or drifted, and never a removal of any
/// kind.
pub fn compute_verify_skill_items(repo_root: &Path) -> Vec<Value> {
    let mut items: Vec<Value> = Vec::new();
    if verify_root_refusal(repo_root).is_some() {
        return items; // silent by design: bee's own sync is untouched
    }
    let source_root = verify_source_root(repo_root);
    // An UNREADABLE root reads as zero entries, not as an error that stops
    // onboarding: `read_dir_sorted_checked` separates "cannot open" from
    // "opened and empty" so a permission-denied directory cannot be mistaken
    // for a directory whose skills were all deleted.
    let Ok(entries) = read_dir_sorted_checked(&source_root) else {
        return items;
    };
    let targets = verify_render_targets(repo_root);
    for entry in entries {
        // Enumeration is the WHOLE root, deliberately not
        // `list_bee_skill_entries` — that helper filters on the `bee-` prefix,
        // which is bee's own deletion domain and says nothing about a host's
        // generated skill. Here the containing directory IS the namespace.
        if entry.is_symlink || !entry.is_dir {
            continue; // never followed; a stray file is not a skill directory
        }
        let name = &entry.name;
        let source_dir = source_root.join(name);
        let source_rel = format!(".bee/verify/{name}");
        if let Some(b) = walk_skill_tree(&source_dir, None).blocked {
            items.push(verify_blocked_item(
                name,
                &format!("{source_rel}/{}", b.path),
                "source",
                "source",
                format!("source {source_rel} contains a {} at {} - skipped", b.reason, b.path),
            ));
            continue;
        }
        let marker_errors = verify_marker_errors(&source_dir);
        if !marker_errors.is_empty() {
            items.push(verify_blocked_item(
                name,
                &source_rel,
                "source",
                "source",
                format!(
                    "source {source_rel} has malformed bee:only markers - skipped, nothing rendered: {}",
                    marker_errors.join("; ")
                ),
            ));
            continue;
        }
        for (kind, target_root, rel_root) in &targets {
            let runtime = runtime_for_target_kind(kind);
            let render = |buf: &[u8]| render_skill_bytes(buf, runtime);
            let source_walk = walk_skill_tree(&source_dir, Some(&render));
            let target_dir = target_root.join(name);
            let rel = format!("{rel_root}/{name}");
            match lstat_if_exists(&target_dir) {
                None => items.push(verify_item("copy_verify_skill", name, &rel, "installed", kind)),
                Some(st) if st.is_symlink => items.push(verify_blocked_item(
                    name,
                    &rel,
                    "installed",
                    kind,
                    format!("installed {rel} is a symlink (plausibly a live checkout) - skipped, never written through"),
                )),
                Some(st) if !st.is_dir => items.push(verify_blocked_item(
                    name,
                    &rel,
                    "installed",
                    kind,
                    format!("installed {rel} is not a directory - skipped, never removed"),
                )),
                Some(_) => {
                    let target_walk = walk_skill_tree(&target_dir, None);
                    if let Some(b) = &target_walk.blocked {
                        items.push(verify_blocked_item(
                            name,
                            &format!("{rel}/{}", b.path),
                            "installed",
                            kind,
                            format!(
                                "installed {rel} contains a {} at {} - skipped, nothing inside it written",
                                b.reason, b.path
                            ),
                        ));
                        continue;
                    }
                    if verify_skill_drifted(&source_walk, &target_walk) {
                        items.push(verify_item("copy_verify_skill", name, &rel, "installed", kind));
                    }
                }
            }
        }
    }
    items
}

// ── computePlan ────────────────────────────────────────────────────────────

pub fn compute_plan(engine: &Engine, repo_root: &Path, opts: &Options) -> ComputedPlan {
    let mut plan: Vec<Value> = Vec::new();
    let codex_hybrid = opts.plugin_source && hw::runtime_covers_codex(&opts.runtime);

    // 0. worktree-local coordination migration (msn-18d).
    let worktree_migration = detect_worktree_migration(repo_root);
    if worktree_migration.applicable
        && worktree_migration.conflicts.is_empty()
        && !worktree_migration.records.is_empty()
    {
        let mut m = Map::new();
        m.insert("action".into(), json!("migrate_worktree_records"));
        m.insert("path".into(), json!(".bee/ (worktree-local coordination stores)"));
        m.insert("count".into(), json!(worktree_migration.records.len()));
        plan.push(Value::Object(m));
    }

    let release_identity = read_source_release_identity(engine);
    if release_identity.blocked.is_some() {
        return ComputedPlan {
            plan,
            bee_version: None,
            rendered_block: String::new(),
            rendered_gitignore_block: String::new(),
            desired_managed: json!({}),
            skill_sync: blocked_source_identity_skill_sync(
                engine,
                repo_root,
                opts.sync_skills,
                opts.global_skills,
                &release_identity,
            ),
            codex_hybrid,
            worktree_migration,
        };
    }
    let bee_version = release_identity.version.clone();
    let rendered_block = render_agents_block(
        &engine.agents_block_template,
        host_shell_is_powershell(repo_root).then_some(engine.agents_windows_template.as_path()),
    );
    let rendered_gitignore_block = render_gitignore_block();

    // 1. AGENTS.md BEE block
    let agents_path = repo_root.join("AGENTS.md");
    let agents_text = read_text_if_exists(&agents_path);
    if agents_text.trim().is_empty() {
        plan.push(plan_item("create_agents_block", "AGENTS.md"));
    } else if !agents_block_present(&agents_text) {
        plan.push(plan_item("append_agents_block", "AGENTS.md"));
    } else if extract_agents_block(&agents_text).as_deref() != Some(rendered_block.as_str()) {
        plan.push(plan_item("update_agents_block", "AGENTS.md"));
    }

    // 1b. minimal header proposal (decision D4, propose-only)
    if !has_prose_outside_block(&agents_text) {
        plan.push(plan_item("propose_agents_header", "AGENTS.md"));
    }

    // 2. runtime files (create-if-missing only)
    for rel in [
        ".bee/state.json",
        ".bee/config.json",
        ".bee/reservations.json",
        ".bee/decisions.jsonl",
        ".bee/backlog.jsonl",
        // config-sample-herding D3: the annotated sample, embedded at
        // compile time (templates.rs::CONFIG_SAMPLE_JSON) so a fresh repo
        // gets full documentation without visiting the bee repo.
        ".bee/config-sample.json",
    ] {
        if !exists(&join_rel(repo_root, rel)) {
            plan.push(plan_item("create_runtime_file", rel));
        }
    }
    for rel_dir in [".bee/cells", ".bee/logs"] {
        if !exists(&join_rel(repo_root, rel_dir)) {
            plan.push(plan_item("create_dir", rel_dir));
        }
    }

    // 3. vendored helpers + lib (copy when missing or drifted)
    let onboarding = read_json_if_exists(&repo_root.join(".bee").join("onboarding.json"));
    for name in list_template_helpers(engine) {
        let source = read_text_if_exists(&engine.templates_dir.join(&name));
        let target = repo_root.join(".bee").join("bin").join(&name);
        if read_text_if_exists(&target) != source {
            plan.push(plan_item("copy_helper", &format!(".bee/bin/{name}")));
        }
    }
    // 3a. retired helper shims (D2) — the NAMED list.
    for name in T::RETIRED_HELPERS {
        if exists(&repo_root.join(".bee").join("bin").join(name)) {
            plan.push(plan_item("remove_helper", &format!(".bee/bin/{name}")));
        }
    }
    // 3b. STALE helpers, derived from the ledger diff (R6 cutover — new).
    //
    // The named list above only ever covered helpers retired ONE AT A TIME by
    // a decision that also added them to RETIRED_HELPERS. The R6 cutover
    // retires `bee.mjs` — the Node CLI entrypoint — by deleting the whole
    // source tree, and nothing adds it to a list. Without this, every host
    // that ever onboarded keeps `.bee/bin/bee.mjs` forever: a dispatcher whose
    // entire `lib/` closure `remove_lib` is about to delete out from under it,
    // still sitting at the path AGENTS.md tells agents to invoke
    // (`.bee/bin/bee …`). A half-working entrypoint is worse than a missing
    // one, so it is derived exactly the way 3c derives `remove_lib`: the
    // previous ledger's key set minus the current source's, intersected with
    // what is actually on disk.
    let current_helper_names = list_template_helpers(engine);
    let previous_helper_names: Vec<String> = onboarding
        .as_ref()
        .and_then(|o| o.get("managed"))
        .and_then(|m| m.get("helpers"))
        .and_then(Value::as_object)
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    for name in previous_helper_names {
        if T::RETIRED_HELPERS.contains(&name.as_str()) {
            continue; // already planned by 3a — never plan the same unlink twice
        }
        if name.contains('/') || name.contains('\\') {
            continue; // helpers are flat files directly under .bee/bin
        }
        if !current_helper_names.contains(&name)
            && exists(&repo_root.join(".bee").join("bin").join(&name))
        {
            plan.push(plan_item("remove_helper", &format!(".bee/bin/{name}")));
        }
    }
    let current_lib_names = list_template_lib_modules(engine);
    for name in &current_lib_names {
        let source = read_text_if_exists(&engine.templates_lib_dir.join(name));
        let target = repo_root.join(".bee").join("bin").join("lib").join(name);
        if read_text_if_exists(&target) != source {
            plan.push(plan_item("copy_lib", &format!(".bee/bin/lib/{name}")));
        }
    }
    // 3c. retired lib modules — derived from the ledger diff, not a list.
    let previous_lib_names: Vec<String> = onboarding
        .as_ref()
        .and_then(|o| o.get("managed"))
        .and_then(|m| m.get("lib"))
        .and_then(Value::as_object)
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    for name in previous_lib_names {
        if !current_lib_names.contains(&name)
            && exists(&repo_root.join(".bee").join("bin").join("lib").join(&name))
        {
            plan.push(plan_item("remove_lib", &format!(".bee/bin/lib/{name}")));
        }
    }

    // 3d. vendored expertise guides — recursive on both sides.
    let current_expertise_names = list_source_expertise(engine);
    for name in &current_expertise_names {
        let source = read_text_if_exists(&join_rel(&engine.expertise_dir, name));
        let target = join_rel(&repo_root.join(".bee").join("expertise"), name);
        if read_text_if_exists(&target) != source {
            plan.push(plan_item("copy_expertise", &format!(".bee/expertise/{name}")));
        }
    }
    if exists(&engine.expertise_dir) {
        let expertise_target_dir = repo_root.join(".bee").join("expertise");
        let mut stale: Vec<String> = if exists(&expertise_target_dir) {
            list_vendored_expertise(&expertise_target_dir)
                .into_iter()
                .filter(|name| !current_expertise_names.contains(name))
                .collect()
        } else {
            Vec::new()
        };
        stale.sort();
        for name in stale {
            plan.push(plan_item("remove_expertise", &format!(".bee/expertise/{name}")));
        }
    }

    // 3e. vendored prompt files (prompt-files spec §1)
    let current_prompt_names = list_template_prompts(engine);
    for name in &current_prompt_names {
        let source = read_text_if_exists(&engine.templates_prompts_dir.join(name));
        let target = repo_root.join(".bee").join("bin").join("prompts").join(name);
        if read_text_if_exists(&target) != source {
            plan.push(plan_item("copy_prompt", &format!(".bee/bin/prompts/{name}")));
        }
    }
    if exists(&engine.templates_prompts_dir) {
        let prompts_target_dir = repo_root.join(".bee").join("bin").join("prompts");
        let mut stale: Vec<String> = if exists(&prompts_target_dir) {
            read_dir_sorted(&prompts_target_dir)
                .into_iter()
                .filter(|e| e.is_file && e.name.ends_with(".md") && !current_prompt_names.contains(&e.name))
                .map(|e| e.name)
                .collect()
        } else {
            Vec::new()
        };
        stale.sort();
        for name in stale {
            plan.push(plan_item("remove_prompt", &format!(".bee/bin/prompts/{name}")));
        }
    }

    // 3b. statusline pair (opt-in sync)
    if hw::statusline_opt_in(repo_root) {
        for name in list_template_statusline(engine) {
            let source = read_text_if_exists(&engine.templates_statusline_dir.join(&name));
            let target = repo_root.join(".claude").join(&name);
            if read_text_if_exists(&target) != source {
                plan.push(plan_item("copy_statusline", &format!(".claude/{name}")));
            }
        }
    }

    // 3f. OpenCode guard plugin (opencode-support D2/D3, oc-13): copy when
    // missing or drifted, same shape as 3e's prompt files — this checkout's
    // OWN `.opencode/plugins/` tree is the source (see Engine::opencode_plugin_dir),
    // never a `packages/bee/` template. A belt that ships only in the source
    // checkout's working tree is inert everywhere else, so this is what makes
    // `bee onboard --apply` install it into a host project at all.
    let opencode_plugin_files = list_opencode_plugin_files(engine);
    if !opencode_plugin_files.is_empty()
        && !exists(&repo_root.join(".opencode").join("plugins"))
    {
        plan.push(plan_item("create_dir", ".opencode/plugins"));
    }
    for name in &opencode_plugin_files {
        let source = read_text_if_exists(&engine.opencode_plugin_dir.join(name));
        let target = repo_root.join(".opencode").join("plugins").join(name);
        if read_text_if_exists(&target) != source {
            plan.push(plan_item("copy_opencode_plugin", &format!(".opencode/plugins/{name}")));
        }
    }

    // 3g. Pi guard extension (pi-support D1): the fourth belt, vendored the
    // same "copy when missing or drifted" way 3f vendors the OpenCode plugin
    // — this checkout's OWN `.pi/extensions/` tree is the source (see
    // Engine::pi_extension_dir). Pi auto-discovers `<cwd>/.pi/extensions/`,
    // so copying the file into the host project IS the whole install: no
    // global directory, no user config, no hook JSON.
    let pi_extension_files = list_pi_extension_files(engine);
    if !pi_extension_files.is_empty() && !exists(&repo_root.join(".pi").join("extensions")) {
        plan.push(plan_item("create_dir", ".pi/extensions"));
    }
    for name in &pi_extension_files {
        let source = read_text_if_exists(&engine.pi_extension_dir.join(name));
        let target = repo_root.join(".pi").join("extensions").join(name);
        if read_text_if_exists(&target) != source {
            plan.push(plan_item("copy_pi_extension", &format!(".pi/extensions/{name}")));
        }
    }

    // 4. learnings stub
    if !exists(&join_rel(repo_root, "docs/history/learnings/critical-patterns.md")) {
        plan.push(plan_item("create_stub", "docs/history/learnings/critical-patterns.md"));
    }
    // 4a. state-layer skeletons
    for name in ["reading-map.md", "system-overview.md"] {
        if !exists(&join_rel(repo_root, &format!("docs/specs/{name}"))) {
            plan.push(plan_item("create_specs_stub", &format!("docs/specs/{name}")));
        }
    }

    // 4b. .gitignore managed block (D1)
    let gitignore_text = read_text_if_exists(&repo_root.join(".gitignore"));
    if gitignore_text.trim().is_empty() {
        plan.push(plan_item("create_gitignore_block", ".gitignore"));
    } else if !gitignore_block_present(&gitignore_text) {
        plan.push(plan_item("append_gitignore_block", ".gitignore"));
    } else if normalize_gitignore_for_compare(
        extract_gitignore_block(&gitignore_text).as_deref().unwrap_or(""),
    ) != rendered_gitignore_block
    {
        plan.push(plan_item("update_gitignore_block", ".gitignore"));
    }

    // 5-pre. STALE VENDORED HOOKS (R6 cutover — new action).
    //
    // Section 3c has always removed a lib module that left the source
    // (`remove_lib`, derived from the ledger diff, never a pasted list). The
    // hook tree had NO such action: `copy_repo_hook` writes, and nothing ever
    // unlinked. That was survivable while HOOK_FILENAMES only ever grew; it
    // stops being survivable the moment the whole `.mjs` hook set is deleted at
    // once, because every host that ever ran `--repo-hooks` or codex-hybrid
    // would keep eleven dead `.bee/bin/hooks/*.mjs` on disk FOREVER, with no
    // command able to reach them. `.codex/hooks.json` still points at that
    // directory on a host that owns its own catalog, so "dead files nobody
    // deletes" is not cosmetic — it is a hook tree that can still be launched.
    //
    // Derivation matches `remove_lib` exactly: the previous ledger's key set
    // minus the current source's, intersected with what is actually on disk. It
    // therefore fires for hosts that recorded the old hooks and stays silent
    // for hosts that never vendored any. The `.codex/hooks.json` ledger key is
    // excluded by name — `build_hook_versions` stores it alongside the hook
    // filenames, but it is not a file under `.bee/bin/hooks/` and must never be
    // unlinked by this rule.
    {
        let current_hook_names = list_plugin_hooks(engine);
        let mut previous_hook_names: Vec<String> = Vec::new();
        for ledger_key in ["repo_hooks", "codex_hooks"] {
            let names = onboarding
                .as_ref()
                .and_then(|o| o.get("managed"))
                .and_then(|m| m.get(ledger_key))
                .and_then(Value::as_object)
                .map(|o| o.keys().cloned().collect::<Vec<String>>())
                .unwrap_or_default();
            for name in names {
                if !previous_hook_names.contains(&name) {
                    previous_hook_names.push(name);
                }
            }
        }
        previous_hook_names.sort();
        for name in previous_hook_names {
            if name == ".codex/hooks.json" || name.contains('/') || name.contains('\\') {
                continue; // not a file in the vendored hook directory
            }
            if !current_hook_names.contains(&name)
                && exists(&repo_root.join(".bee").join("bin").join("hooks").join(&name))
            {
                plan.push(plan_item("remove_repo_hook", &format!(".bee/bin/hooks/{name}")));
            }
        }
    }

    // 5. repo hooks fallback (--repo-hooks only)
    if opts.repo_hooks {
        for name in list_plugin_hooks(engine) {
            let source = read_text_if_exists(&engine.plugin_hooks_dir.join(&name));
            let target = repo_root.join(".bee").join("bin").join("hooks").join(&name);
            if read_text_if_exists(&target) != source {
                plan.push(plan_item("copy_repo_hook", &format!(".bee/bin/hooks/{name}")));
            }
        }
        let settings_path = repo_root.join(".claude").join("settings.json");
        if hw::merge_needs_apply(&hw::merge_repo_settings(&settings_path)) {
            plan.push(plan_item("merge_repo_hook_settings", ".claude/settings.json"));
        }
        if !hw::repo_owns_hook_catalog(repo_root) {
            let codex_hooks_path = repo_root.join(".codex").join("hooks.json");
            if hw::merge_needs_apply(&hw::merge_codex_hooks(&codex_hooks_path)) {
                plan.push(plan_item("merge_codex_hooks", ".codex/hooks.json"));
            }
        }
    }

    // 5a. codex-hybrid hooks (GH #22 P0-1)
    if codex_hybrid && !hw::repo_owns_hook_catalog(repo_root) {
        for name in list_plugin_hooks(engine) {
            let source = read_text_if_exists(&engine.plugin_hooks_dir.join(&name));
            let target = repo_root.join(".bee").join("bin").join("hooks").join(&name);
            if read_text_if_exists(&target) != source {
                plan.push(plan_item("copy_repo_hook", &format!(".bee/bin/hooks/{name}")));
            }
        }
        let codex_hooks_path = repo_root.join(".codex").join("hooks.json");
        if hw::merge_needs_apply(&hw::merge_codex_hooks(&codex_hooks_path)) {
            plan.push(plan_item("merge_codex_hooks", ".codex/hooks.json"));
        }
    }

    // 5c. Codex user-config status line (machine-level, add-only)
    if hw::codex_statusline_missing() {
        plan.push(plan_item("ensure_codex_statusline", "~/.codex/config.toml"));
    }

    // 5b. CLAUDE.md @import fallback (D1, default)
    if opts.claude_md {
        let claude_md_path = repo_root.join("CLAUDE.md");
        if !exists(&claude_md_path) {
            plan.push(plan_item("create_claude_md", "CLAUDE.md"));
        } else if !claude_md_imports_agents(&read_text_if_exists(&claude_md_path)) {
            plan.push(plan_item("append_claude_md_import", "CLAUDE.md"));
        }
    }

    // 5d. bee agent files (config-rendered, AO10-safe flat sync)
    plan.extend(compute_agent_file_plan(engine, repo_root));

    // 5e. OpenCode worker agent files — same source of truth, own frontmatter
    // shape (opencode-support oc-14, D4).
    plan.extend(compute_opencode_agent_file_plan(engine, repo_root));

    // 6. onboarding.json drift (managed versions)
    let statusline = hw::statusline_opt_in(repo_root);
    let desired_managed = build_managed_versions(
        engine,
        &rendered_block,
        &rendered_gitignore_block,
        opts.repo_hooks,
        statusline,
        codex_hybrid,
    );
    let onboarding_current = onboarding.as_ref().is_some_and(|o| {
        o.get("schema_version").and_then(Value::as_str) == Some(T::ONBOARDING_SCHEMA_VERSION)
            && o.get("bee_version").and_then(Value::as_str) == bee_version.as_deref()
            && jsjson::stringify(&subset_managed(
                o.get("managed"),
                opts.repo_hooks,
                statusline,
                codex_hybrid,
            )) == jsjson::stringify(&subset_managed(
                Some(&desired_managed),
                opts.repo_hooks,
                statusline,
                codex_hybrid,
            ))
    });
    if !onboarding_current {
        plan.push(plan_item("write_onboarding", ".bee/onboarding.json"));
    }

    // 7. skill sync (D1-D5, per target)
    let skill_sync = if opts.sync_skills {
        compute_skill_sync(engine, repo_root, opts.global_skills)
    } else {
        SkillSync {
            source_root: engine.skills_root.join("bee-hive"),
            targets: Vec::new(),
            blocked: None,
            legacy_refresh: None,
        }
    };
    if skill_sync.blocked.is_none() {
        for target in &skill_sync.targets {
            plan.extend(target.items.iter().cloned());
        }
        if let Some(lr) = &skill_sync.legacy_refresh {
            plan.extend(lr.items.iter().cloned());
        }
    }

    // 8. host-generated verification skills (verification-ships-to-hosts D3).
    //
    // Appended AFTER section 7 and computed independently of it: these items
    // read the HOST's `.bee/verify/`, never the engine's skill tree, so
    // nothing a host wrote there can withhold, block or reorder bee's own
    // install. Gated on the same `sync_skills` switch, because a
    // `--plugin-source` run installs no skills at all.
    if opts.sync_skills {
        plan.extend(compute_verify_skill_items(repo_root));
    }

    ComputedPlan {
        plan,
        bee_version,
        rendered_block,
        rendered_gitignore_block,
        desired_managed,
        skill_sync,
        codex_hybrid,
        worktree_migration,
    }
}

#[cfg(test)]
mod tests {
    /// LEDGER_GROUPS is what `verbs/cells.rs`'s regen obligation guards. A
    /// managed group that is not in the table is a tree a cell can hand-edit
    /// without ever being told to re-onboard — the silent no-op the R6 cutover
    /// had to avoid. Pin the table against the builder's real key set.
    #[test]
    fn every_managed_group_is_classified() {
        let dir = tempfile::tempdir().unwrap();
        let engine = super::Engine::from_plugin_root(dir.path().to_path_buf());
        // Every optional group ON, so the builder emits its widest key set.
        let managed = super::build_managed_versions(&engine, "block", "gitignore", true, true, true);
        let built: Vec<&String> = managed.as_object().unwrap().keys().collect();
        let classified: Vec<&str> = super::LEDGER_GROUPS.iter().map(|(k, _)| *k).collect();

        let unclassified: Vec<&&String> =
            built.iter().filter(|k| !classified.contains(&k.as_str())).collect();
        assert!(
            unclassified.is_empty(),
            "build_managed_versions emits managed group(s) {unclassified:?} that LEDGER_GROUPS \
             does not classify. Add them with the host directory they vendor into (or None if \
             they hash a rendered block, not a tree) — otherwise verbs/cells.rs's regen \
             obligation silently stops covering them."
        );
        let stale: Vec<&&str> =
            classified.iter().filter(|k| !built.iter().any(|b| b.as_str() == **k)).collect();
        assert!(
            stale.is_empty(),
            "LEDGER_GROUPS classifies {stale:?}, which build_managed_versions no longer emits — \
             the regen obligation would be guarding a tree nothing writes."
        );
    }

    use super::*;

    #[test]
    fn subset_managed_ignores_prompts_and_optional_maps() {
        let managed = json!({
            "agents_block": "a", "gitignore_block": "g",
            "helpers": {"x":"1"}, "lib": {}, "expertise": {}, "prompts": {"p":"9"},
            "repo_hooks": {"h":"1"}, "statusline": {"s":"1"}, "codex_hooks": {"c":"1"}
        });
        let bare = subset_managed(Some(&managed), false, false, false);
        let keys: Vec<&str> = bare.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["agents_block", "gitignore_block", "helpers", "lib", "expertise"]);
        let full = subset_managed(Some(&managed), true, true, true);
        let keys: Vec<&str> = full.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(
            keys,
            vec![
                "agents_block",
                "gitignore_block",
                "helpers",
                "lib",
                "expertise",
                "repo_hooks",
                "codex_hooks",
                "statusline"
            ]
        );
        // Missing input falls back the way `||` does.
        let empty = subset_managed(None, false, false, false);
        assert!(empty["agents_block"].is_null());
        assert_eq!(empty["helpers"], json!({}));
    }

    #[test]
    fn core_changes_needed_ignores_legacy_refresh_only_plans() {
        let refresh = json!({"action": "refresh_legacy_global_skill"});
        assert!(!core_changes_needed(&[refresh.clone()]));
        assert!(core_changes_needed(&[refresh, json!({"action": "copy_lib"})]));
        assert!(!core_changes_needed(&[]));
    }

    #[test]
    fn expertise_enumeration_is_recursive_and_posix_sorted() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::from_plugin_root(dir.path().to_path_buf());
        std::fs::create_dir_all(engine.expertise_dir.join("tests").join("patterns")).unwrap();
        std::fs::write(engine.expertise_dir.join("tests.md"), "T").unwrap();
        std::fs::write(engine.expertise_dir.join("review.md"), "R").unwrap();
        std::fs::write(engine.expertise_dir.join("notes.txt"), "skip").unwrap();
        std::fs::write(
            engine.expertise_dir.join("tests").join("patterns").join("differential-testing.md"),
            "D",
        )
        .unwrap();
        assert_eq!(
            list_source_expertise(&engine),
            vec![
                "review.md".to_string(),
                "tests.md".to_string(),
                "tests/patterns/differential-testing.md".to_string(),
            ]
        );
    }

    #[test]
    fn vendored_expertise_enumeration_matches_the_source_shape() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join(".bee").join("expertise").join("tests").join("patterns");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("orphan.md"), "O").unwrap();
        std::fs::write(dir.path().join(".bee").join("expertise").join("tests.md"), "T").unwrap();
        assert_eq!(
            list_vendored_expertise(&dir.path().join(".bee").join("expertise")),
            vec!["tests.md".to_string(), "tests/patterns/orphan.md".to_string()]
        );
    }
}
