// bee dev plugin-distribution  <- packages/bee/scripts/plugin_distribution.mjs
//
// The last unported script. It is what both installers shelled `node` out for,
// and therefore the only reason installing bee needed a Node runtime at all.
//
// It answers one question for the installer: given a release manifest and what
// the client CLI reports about its installed `bee` plugin, is the plugin-first
// install real — and if it is, which repo-local fallback copies must now be
// removed so two copies of the same skill cannot disagree. `--apply` performs
// that removal as a transaction: every target is renamed aside first, and any
// failure renames everything back before the error propagates.
//
// FAITHFUL TO THE .mjs, with one deliberate divergence, marked DIVERGENCE
// below: the directory-alias check compares two canonical paths rather than a
// canonical path against a merely-absolute one. On Windows those differ for
// reasons that have nothing to do with aliasing (an 8.3 short component like
// `RUNNER~1` in TEMP resolves to its long form), and the .mjs would refuse a
// perfectly ordinary directory there.

use super::release_manifest::{mode_octal, observed_mode};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

/// A refusal. Every failure path in the .mjs is `fail(message)` — one error
/// code, one message, no partial work — and the CLI renders it as
/// `{"ok":false,"status":"blocked","error":<message>}` with exit 1.
#[derive(Debug)]
struct Refused(String);
type R<T> = Result<T, Refused>;

fn fail<T>(message: impl Into<String>) -> R<T> {
    Err(Refused(message.into()))
}

const PROJECT_SKILL_ROOTS: [&str; 3] = [".claude/skills", ".agents/skills", ".codex/skills"];
const PROJECT_HOOK_FILES: [&str; 2] = [".claude/settings.json", ".codex/hooks.json"];
/// GH #22 P0-1 / advisor R4 (self-erasure fix): the codex-hybrid path writes
/// bee entries into `.codex/hooks.json` that are byte-identical to the ones
/// this pass strips. Without the flag, a plugin-first install would delete the
/// only mechanical enforcement Codex sessions get, immediately after reporting
/// the apply successful.
const CODEX_HYBRID_EXEMPT_FILE: &str = ".codex/hooks.json";

/// D9/cnr2-12: the committed per-runtime rendered skill trees count as expected
/// package content alongside the canonical `plugin_skill` tree.
const PACKAGE_ROLES: [&str; 7] = [
    "plugin_skill",
    "plugin_skill_claude_render",
    "plugin_skill_codex_render",
    "plugin_hook",
    "plugin_manifest",
    "plugin_marketplace",
    "package_payload",
];

const BEE_HOOK_HANDLERS: [&str; 9] = [
    "bee-session-init.mjs",
    "bee-prompt-context.mjs",
    "bee-write-guard.mjs",
    "bee-model-guard.mjs",
    "bee-state-sync.mjs",
    "bee-chain-nudge.mjs",
    "bee-session-close.mjs",
    "bee-codex-subagent-audit.mjs",
    "bee-tools-logger.mjs",
];

fn sha256_bytes(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    format!("{:x}", h.finalize())
}

fn read_file(path: &Path, label: &str) -> R<Vec<u8>> {
    fs::read(path).map_err(|e| Refused(format!("{label} could not be read ({}): {e}", path.display())))
}

/// `path.relative(root, candidate)` containment, without resolving symlinks —
/// the .mjs checks the lexical relationship and so does this.
fn is_inside(root: &Path, candidate: &Path) -> bool {
    match candidate.strip_prefix(root) {
        Ok(rel) => !rel.components().any(|c| matches!(c, Component::ParentDir)),
        Err(_) => false,
    }
}

fn lstat_or_null(target: &Path) -> Option<fs::Metadata> {
    fs::symlink_metadata(target).ok()
}

/// DIVERGENCE (see the module header): the .mjs compares
/// `realpathSync.native(t)` with `path.resolve(t)`. Those disagree on Windows
/// whenever any ancestor carries an 8.3 short name, which is not aliasing. We
/// canonicalize the PARENT and rejoin the final component instead: a junction
/// or symlinked directory still resolves away from its own parent and is still
/// caught, while a short-named ancestor no longer produces a false refusal.
fn assert_plain_directory(target: &Path, label: &str, allow_missing: bool) -> R<bool> {
    let Some(stat) = lstat_or_null(target) else {
        if allow_missing {
            return Ok(false);
        }
        return fail(format!("{label} is missing: {}", target.display()));
    };
    if stat.file_type().is_symlink() || !stat.is_dir() {
        return fail(format!("{label} must be a plain directory: {}", target.display()));
    }
    let resolved = dunce::canonicalize(target)
        .map_err(|e| Refused(format!("{label} could not be resolved ({}): {e}", target.display())))?;
    let expected = match (target.parent(), target.file_name()) {
        (Some(parent), Some(name)) if !parent.as_os_str().is_empty() => {
            dunce::canonicalize(parent).map(|p| p.join(name)).unwrap_or_else(|_| target.to_path_buf())
        }
        _ => resolved.clone(),
    };
    if resolved != expected {
        return fail(format!("{label} aliases another path: {}", target.display()));
    }
    Ok(true)
}

fn assert_plain_file(target: &Path, label: &str) -> R<fs::Metadata> {
    match lstat_or_null(target) {
        Some(stat) if !stat.file_type().is_symlink() && stat.is_file() => Ok(stat),
        _ => fail(format!("{label} must be a plain file: {}", target.display())),
    }
}

/// Directory entries in `localeCompare` order, matching the .mjs sort.
fn sorted_entry_names(dir: &Path) -> R<Vec<String>> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map_err(|e| Refused(format!("cannot read directory {}: {e}", dir.display())))?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    if !super::sort_by_locale(&mut names, |n| n.as_str()) {
        names.sort();
    }
    Ok(names)
}

struct WalkedFile {
    rel: String,
    abs: PathBuf,
}

/// walkPlainFiles: every plain file under `root`, relative paths slash-joined,
/// refusing at the first symlink.
fn walk_plain_files(root: &Path, relative: &str, out: &mut Vec<WalkedFile>) -> R<()> {
    let current = if relative.is_empty() { root.to_path_buf() } else { root.join(relative) };
    let Some(stat) = lstat_or_null(&current) else { return Ok(()) };
    if stat.file_type().is_symlink() {
        return fail(format!("symlink is forbidden in inventory: {}", current.display()));
    }
    if stat.is_file() {
        out.push(WalkedFile { rel: relative.to_string(), abs: current });
        return Ok(());
    }
    if !stat.is_dir() {
        return fail(format!("unsupported inventory entry: {}", current.display()));
    }
    for name in sorted_entry_names(&current)? {
        let child_rel =
            if relative.is_empty() { name.clone() } else { format!("{relative}/{name}") };
        let child_abs = current.join(&name);
        if lstat_or_null(&child_abs).is_some_and(|s| s.file_type().is_symlink()) {
            return fail(format!("symlink is forbidden in inventory: {}", child_abs.display()));
        }
        walk_plain_files(root, &child_rel, out)?;
    }
    Ok(())
}

#[derive(Clone)]
struct FileRecord {
    sha256: String,
    /// The executable bit as this platform can observe it — `None` on
    /// Windows, where it cannot. An installed package has no git index, so
    /// this is the only reader available for the manifest's recorded mode.
    mode: Option<String>,
}

fn file_record(abs: &Path) -> R<FileRecord> {
    let stat = assert_plain_file(abs, "inventory entry")?;
    let bytes = read_file(abs, "inventory entry")?;
    Ok(FileRecord { sha256: sha256_bytes(&bytes), mode: observed_mode(&stat) })
}

// ── release manifest inventory ─────────────────────────────────────────────

#[derive(Clone)]
struct InventoryRecord {
    path: String,
    sha256: String,
    mode: String,
    role: String,
}

fn load_package_inventory(manifest_path: &Path) -> R<Vec<InventoryRecord>> {
    assert_plain_file(manifest_path, "release manifest")?;
    let bytes = read_file(manifest_path, "release manifest")?;
    let manifest: Value = serde_json::from_slice(&bytes)
        .map_err(|e| Refused(format!("release manifest is not valid JSON: {e}")))?;
    let Some(files) = manifest.get("files").and_then(Value::as_array) else {
        return fail("release manifest has no files array");
    };
    let roles: BTreeSet<&str> = PACKAGE_ROLES.iter().copied().collect();
    let mut inventory: Vec<InventoryRecord> = Vec::new();
    for record in files {
        let role = record.get("role").and_then(Value::as_str).unwrap_or_default();
        if !roles.contains(role) {
            continue;
        }
        let path = record
            .get("packagePath")
            .and_then(Value::as_str)
            .or_else(|| record.get("path").and_then(Value::as_str))
            .unwrap_or_default()
            .to_string();
        inventory.push(InventoryRecord {
            path,
            sha256: record.get("sha256").and_then(Value::as_str).unwrap_or_default().to_string(),
            mode: record.get("mode").and_then(Value::as_str).unwrap_or_default().to_string(),
            role: role.to_string(),
        });
    }
    if !super::sort_by_locale(&mut inventory, |r| r.path.as_str()) {
        inventory.sort_by(|a, b| a.path.cmp(&b.path));
    }
    if inventory.is_empty() {
        return fail("release manifest has no package inventory records");
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for record in &inventory {
        let unsafe_path = record.path.is_empty()
            || js_is_absolute(&record.path)
            || record.path.split(['\\', '/']).any(|s| s == "..");
        if unsafe_path {
            return fail(format!("unsafe package path: {}", record.path));
        }
        let sha_ok = record.sha256.len() == 64
            && record.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());
        let mode_ok =
            record.mode.len() == 3 && record.mode.chars().all(|c| ('0'..='7').contains(&c));
        if !sha_ok || !mode_ok {
            return fail(format!("malformed package record: {}", record.path));
        }
        if !seen.insert(record.path.as_str()) {
            return fail(format!("duplicate package record: {}", record.path));
        }
    }
    Ok(inventory)
}

/// Node's `path.isAbsolute`, not Rust's. They disagree exactly where it
/// matters: on Windows, Rust reports `/etc/passwd` as RELATIVE (it carries no
/// drive), so a manifest naming a rooted POSIX path would sail through the
/// containment check on the one platform where the .mjs caught it.
fn js_is_absolute(p: &str) -> bool {
    let b = p.as_bytes();
    if matches!(b.first(), Some(b'/') | Some(b'\\')) {
        return true;
    }
    // `C:\x` / `C:/x` — a drive-qualified rooted path.
    matches!(b, [d, b':', s, ..] if d.is_ascii_alphabetic() && (*s == b'/' || *s == b'\\'))
}

fn is_safe_skill_name(name: &str) -> bool {
    name.strip_prefix("bee-").is_some_and(|rest| {
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    })
}

fn managed_skill_names(inventory: &[InventoryRecord]) -> R<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for record in inventory {
        if record.role != "plugin_skill" {
            continue;
        }
        let mut segments = record.path.split('/');
        let head = segments.next().unwrap_or_default();
        let name = segments.next().unwrap_or_default();
        if head != "skills" || name.is_empty() {
            return fail(format!(
                "release inventory has an unexpected plugin skill path: {}",
                record.path
            ));
        }
        if !is_safe_skill_name(name) {
            return fail(format!("release inventory names an unsafe managed skill: {name}"));
        }
        names.insert(name.to_string());
    }
    if names.is_empty() {
        return fail("release inventory names no managed plugin skills");
    }
    Ok(names)
}

// ── plugin discovery ───────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct PluginState {
    runtime: String,
    installed: bool,
    enabled: bool,
    root: Option<String>,
    version: Option<Value>,
    source_kind: Option<String>,
}

fn normalize_plugin_list(payload: &Value) -> Vec<Value> {
    if let Some(items) = payload.as_array() {
        return items.clone();
    }
    for key in ["plugins", "items", "data"] {
        if let Some(items) = payload.get(key).and_then(Value::as_array) {
            return items.clone();
        }
    }
    if payload.is_object() {
        vec![payload.clone()]
    } else {
        Vec::new()
    }
}

fn first_str<'a>(item: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| item.get(*k).and_then(Value::as_str))
}

fn discover_bee_plugin(payload: &Value, runtime: &str) -> PluginState {
    let candidates = normalize_plugin_list(payload);
    let plugin = candidates.into_iter().find(|item| {
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .or_else(|| item.get("id").and_then(Value::as_str))
            .or_else(|| item.pointer("/plugin/name").and_then(Value::as_str))
            .unwrap_or_default();
        name == "bee" || name.starts_with("bee@")
    });
    let Some(plugin) = plugin else {
        return PluginState { runtime: runtime.to_string(), ..Default::default() };
    };
    let state = first_str(&plugin, &["status", "state"]).unwrap_or_default().to_lowercase();
    let installed = plugin.get("installed") == Some(&Value::Bool(true))
        || !["removed", "not_installed"].contains(&state.as_str());
    let enabled = installed
        && (plugin.get("enabled") == Some(&Value::Bool(true))
            || ["enabled", "active"].contains(&state.as_str()));
    PluginState {
        runtime: runtime.to_string(),
        installed,
        enabled,
        root: first_str(
            &plugin,
            &["root", "path", "installPath", "install_path", "sourcePath", "source_path"],
        )
        .map(str::to_string),
        version: plugin
            .get("version")
            .cloned()
            .or_else(|| plugin.pointer("/plugin/version").cloned()),
        source_kind: first_str(&plugin, &["sourceKind", "source_kind", "provenance"])
            .map(str::to_string),
    }
}

#[derive(Debug)]
struct Proof {
    #[allow(dead_code)]
    root: PathBuf,
    #[allow(dead_code)]
    files: usize,
    #[allow(dead_code)]
    version: Option<Value>,
}

fn prove_installed_package(state: &PluginState, expected: &[InventoryRecord]) -> R<Proof> {
    if !state.installed || !state.enabled {
        let who = if state.runtime.is_empty() { "runtime" } else { &state.runtime };
        return fail(format!("{who} bee plugin is not installed and enabled"));
    }
    let kind = state.source_kind.clone().unwrap_or_default().to_lowercase();
    if ["source_checkout", "checkout", "repository"].contains(&kind.as_str()) {
        return fail("source checkout cannot substitute for an installed plugin package");
    }
    let Some(root) = state.root.as_deref().filter(|r| Path::new(r).is_absolute()) else {
        return fail("enabled plugin did not report an absolute installed package root");
    };
    let package_root = PathBuf::from(root);
    assert_plain_directory(&package_root, "installed package root", false)?;

    let expected_map: BTreeMap<&str, &InventoryRecord> =
        expected.iter().map(|r| (r.path.as_str(), r)).collect();
    let mut prefixes: Vec<&str> =
        expected.iter().filter_map(|r| r.path.split('/').next()).collect();
    prefixes.dedup();
    let prefixes: BTreeSet<&str> = prefixes.into_iter().collect();

    let mut actual: BTreeMap<String, FileRecord> = BTreeMap::new();
    for prefix in prefixes {
        let prefix_path = package_root.join(prefix);
        let Some(stat) = lstat_or_null(&prefix_path) else { continue };
        if stat.file_type().is_symlink() {
            return fail(format!("installed package prefix is a symlink: {prefix}"));
        }
        if stat.is_file() {
            actual.insert(prefix.to_string(), file_record(&prefix_path)?);
        } else {
            let mut walked = Vec::new();
            walk_plain_files(&prefix_path, "", &mut walked)?;
            for item in walked {
                let rel = if item.rel.is_empty() {
                    prefix.to_string()
                } else {
                    format!("{prefix}/{}", item.rel)
                };
                actual.insert(rel, file_record(&item.abs)?);
            }
        }
    }

    let missing: Vec<&str> =
        expected_map.keys().copied().filter(|k| !actual.contains_key(*k)).collect();
    let unexpected: Vec<&str> = actual
        .keys()
        .map(String::as_str)
        .filter(|k| !expected_map.contains_key(*k))
        .collect();
    let changed: Vec<&str> = expected_map
        .iter()
        .filter(|(k, record)| {
            actual.get(**k).is_some_and(|v| {
                v.sha256 != record.sha256
                    // A platform that cannot observe the executable bit does
                    // not get to call it changed.
                    || v.mode.as_ref().is_some_and(|m| m != &record.mode)
            })
        })
        .map(|(k, _)| *k)
        .collect();
    if !missing.is_empty() || !unexpected.is_empty() || !changed.is_empty() {
        let join = |v: Vec<&str>| if v.is_empty() { "none".to_string() } else { v.join(",") };
        return fail(format!(
            "installed package inventory mismatch (missing={}; unexpected={}; changed={})",
            join(missing),
            join(unexpected),
            join(changed)
        ));
    }
    Ok(Proof { root: package_root, files: actual.len(), version: state.version.clone() })
}

fn prove_plugin_inactive(states: &[PluginState]) -> R<()> {
    let active: Vec<&str> = states
        .iter()
        .filter(|s| s.installed || s.enabled)
        .map(|s| s.runtime.as_str())
        .collect();
    if !active.is_empty() {
        return fail(format!("bee plugin remains active for: {}", active.join(", ")));
    }
    Ok(())
}

// ── hook configuration cleanup ─────────────────────────────────────────────

fn recognized_bee_command(command: Option<&str>) -> bool {
    let Some(command) = command else { return false };
    let normalized = command.replace('\\', "/");
    let has_handler =
        BEE_HOOK_HANDLERS.iter().any(|name| normalized.contains(&format!("/hooks/{name}")));
    if !has_handler {
        return false;
    }
    normalized.contains("CLAUDE_PLUGIN_ROOT")
        || normalized.contains("CLAUDE_PROJECT_DIR")
        || normalized.contains("/.bee/bin/hooks/")
        || normalized.contains("/hooks/bee-")
}

struct PlannedWrite {
    path: PathBuf,
    before: Vec<u8>,
    after: Vec<u8>,
    #[allow(dead_code)]
    removed: usize,
}

fn clean_hook_config(abs: &Path) -> R<Option<PlannedWrite>> {
    if lstat_or_null(abs).is_none() {
        return Ok(None);
    }
    assert_plain_file(abs, "hook configuration")?;
    let before = read_file(abs, "hook configuration")?;
    let mut json: Value = serde_json::from_slice(&before)
        .map_err(|e| Refused(format!("malformed hook configuration {}: {e}", abs.display())))?;
    if !json.is_object() {
        return fail(format!("hook configuration must be an object: {}", abs.display()));
    }
    if json.get("hooks").is_none() {
        return Ok(None);
    }
    if !json.get("hooks").is_some_and(Value::is_object) {
        return fail(format!("hooks must be an object: {}", abs.display()));
    }

    let mut removed = 0usize;
    let hooks = json.get("hooks").cloned().unwrap();
    let mut next_hooks = Map::new();
    for (event, groups) in hooks.as_object().unwrap() {
        let Some(groups) = groups.as_array() else {
            return fail(format!("hook event {event} must be an array: {}", abs.display()));
        };
        let mut kept_groups: Vec<Value> = Vec::new();
        for group in groups {
            let Some(group_obj) = group.as_object() else {
                return fail(format!("hook group {event} is malformed: {}", abs.display()));
            };
            let Some(entries) = group_obj.get("hooks").and_then(Value::as_array) else {
                return fail(format!("hook group {event} is malformed: {}", abs.display()));
            };
            let mut kept: Vec<Value> = Vec::new();
            for hook in entries {
                if !hook.is_object() {
                    return fail(format!("hook entry {event} is malformed: {}", abs.display()));
                }
                let recognized = hook.get("type").and_then(Value::as_str) == Some("command")
                    && recognized_bee_command(hook.get("command").and_then(Value::as_str));
                if recognized {
                    removed += 1;
                } else {
                    kept.push(hook.clone());
                }
            }
            if !kept.is_empty() {
                let mut next_group = group_obj.clone();
                next_group.insert("hooks".into(), Value::Array(kept));
                kept_groups.push(Value::Object(next_group));
            }
        }
        if !kept_groups.is_empty() {
            next_hooks.insert(event.clone(), Value::Array(kept_groups));
        }
    }

    if removed == 0 {
        return Ok(None);
    }
    let obj = json.as_object_mut().unwrap();
    if next_hooks.is_empty() {
        obj.remove("hooks");
    } else {
        obj.insert("hooks".into(), Value::Object(next_hooks));
    }
    let mut after = serde_json::to_vec_pretty(&json)
        .map_err(|e| Refused(format!("could not re-render {}: {e}", abs.display())))?;
    after.push(b'\n');
    Ok(Some(PlannedWrite { path: abs.to_path_buf(), before, after, removed }))
}

// ── project + user cleanup targets ─────────────────────────────────────────

struct ProjectCleanup {
    dirs: Vec<PathBuf>,
    configs: Vec<PlannedWrite>,
}

fn collect_project_cleanup(
    repo_root: &Path,
    managed_skills: &BTreeSet<String>,
    codex_hybrid: bool,
) -> R<ProjectCleanup> {
    assert_plain_directory(repo_root, "repository root", false)?;
    if managed_skills.is_empty() {
        return fail("project cleanup requires the managed release skill set");
    }
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    for relative_root in PROJECT_SKILL_ROOTS {
        let root = repo_root.join(relative_root.replace('/', std::path::MAIN_SEPARATOR_STR));
        if lstat_or_null(&root).is_none() {
            continue;
        }
        assert_plain_directory(&root, "project skill root", false)?;
        if !is_inside(repo_root, &root) {
            return fail(format!("project skill root escapes repository: {}", root.display()));
        }
        for name in sorted_entry_names(&root)? {
            if !managed_skills.contains(&name) {
                continue;
            }
            let target = root.join(&name);
            let stat = lstat_or_null(&target);
            if !stat.is_some_and(|s| !s.file_type().is_symlink() && s.is_dir()) {
                return fail(format!(
                    "managed cleanup target must be a direct plain directory: {}",
                    target.display()
                ));
            }
            assert_plain_directory(&target, "managed cleanup target", false)?;
            let real = dunce::canonicalize(&target).unwrap_or_else(|_| target.clone());
            if !seen.insert(real) {
                return fail(format!("duplicate cleanup target alias: {}", target.display()));
            }
            dirs.push(target);
        }
    }
    dirs.sort();

    let mut configs = Vec::new();
    for relative in PROJECT_HOOK_FILES {
        if codex_hybrid && relative == CODEX_HYBRID_EXEMPT_FILE {
            continue;
        }
        let abs = repo_root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(write) = clean_hook_config(&abs)? {
            configs.push(write);
        }
    }
    Ok(ProjectCleanup { dirs, configs })
}

struct LedgerCleanup {
    dirs: Vec<PathBuf>,
    update: Option<PlannedWrite>,
}

fn read_ownership_ledger(
    ledger_path: Option<&Path>,
    requested_roots: &[String],
) -> R<LedgerCleanup> {
    if requested_roots.is_empty() {
        return Ok(LedgerCleanup { dirs: Vec::new(), update: None });
    }
    let Some(ledger_path) = ledger_path else {
        return fail("user-root cleanup requires an ownership ledger");
    };
    assert_plain_file(ledger_path, "ownership ledger")?;
    let before = read_file(ledger_path, "ownership ledger")?;
    let mut ledger: Value = serde_json::from_slice(&before)
        .map_err(|e| Refused(format!("ownership ledger is not valid JSON: {e}")))?;
    let shape_ok = ledger.get("schemaVersion").and_then(Value::as_i64) == Some(1)
        && ledger.get("roots").is_some_and(Value::is_array);
    if !shape_ok {
        return fail("ownership ledger has an unsupported shape");
    }

    let requested: Vec<PathBuf> = requested_roots
        .iter()
        .map(|r| std::path::absolute(r).unwrap_or_else(|_| PathBuf::from(r)))
        .collect();
    if requested.iter().collect::<BTreeSet<_>>().len() != requested.len() {
        return fail("user skill roots contain a duplicate or alias");
    }

    let roots = ledger.get("roots").and_then(Value::as_array).unwrap().clone();
    let mut dirs = Vec::new();
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    for root in &requested {
        assert_plain_directory(root, "user skill root", false)?;
        let matches: Vec<&Value> = roots
            .iter()
            .filter(|entry| {
                entry
                    .get("path")
                    .and_then(Value::as_str)
                    .and_then(|p| std::path::absolute(p).ok())
                    .is_some_and(|p| &p == root)
            })
            .collect();
        let skills = matches.first().and_then(|m| m.get("skills")).and_then(Value::as_array);
        let (Some(skills), 1) = (skills, matches.len()) else {
            return fail(format!(
                "ownership ledger does not exactly name user root: {}",
                root.display()
            ));
        };
        for name in skills {
            let Some(name) = name.as_str().filter(|n| is_safe_skill_name(n)) else {
                return fail(format!(
                    "ownership ledger contains unsafe skill name: {}",
                    name.as_str().unwrap_or("<non-string>")
                ));
            };
            let target = root.join(name);
            if !is_inside(root, &target) {
                return fail(format!("ledger target escapes root: {}", target.display()));
            }
            if lstat_or_null(&target).is_none() {
                continue;
            }
            assert_plain_directory(&target, "ledger-owned skill", false)?;
            let real = dunce::canonicalize(&target).unwrap_or_else(|_| target.clone());
            if !seen.insert(real) {
                return fail(format!("duplicate ledger target alias: {}", target.display()));
            }
            dirs.push(target);
        }
    }

    if let Some(next_roots) = ledger.get_mut("roots").and_then(Value::as_array_mut) {
        for entry in next_roots.iter_mut() {
            let is_requested = entry
                .get("path")
                .and_then(Value::as_str)
                .and_then(|p| std::path::absolute(p).ok())
                .is_some_and(|p| requested.contains(&p));
            if is_requested {
                if let Some(obj) = entry.as_object_mut() {
                    obj.insert("skills".into(), Value::Array(Vec::new()));
                }
            }
        }
    }
    let mut after = serde_json::to_vec_pretty(&ledger)
        .map_err(|e| Refused(format!("could not re-render the ownership ledger: {e}")))?;
    after.push(b'\n');
    Ok(LedgerCleanup {
        dirs,
        update: Some(PlannedWrite {
            path: ledger_path.to_path_buf(),
            before,
            after,
            removed: 0,
        }),
    })
}

// ── preflight snapshots ────────────────────────────────────────────────────

fn snapshot_rows(current: &Path, relative: &str, rows: &mut Vec<Value>) -> R<()> {
    let Some(stat) = lstat_or_null(current) else {
        return fail(format!("snapshot target is missing or symlinked: {}", current.display()));
    };
    if stat.file_type().is_symlink() {
        return fail(format!("snapshot target is missing or symlinked: {}", current.display()));
    }
    let mode = mode_octal(&stat);
    if stat.is_file() {
        let bytes = read_file(current, "snapshot target")?;
        rows.push(json!([relative, "file", mode, sha256_bytes(&bytes)]));
        return Ok(());
    }
    if !stat.is_dir() {
        return fail(format!("snapshot target has an unsupported entry: {}", current.display()));
    }
    if !relative.is_empty() {
        rows.push(json!([relative, "dir", mode]));
    }
    let mut names: Vec<String> = fs::read_dir(current)
        .map_err(|e| Refused(format!("cannot read directory {}: {e}", current.display())))?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    for name in names {
        let child_rel =
            if relative.is_empty() { name.clone() } else { format!("{relative}/{name}") };
        snapshot_rows(&current.join(&name), &child_rel, rows)?;
    }
    Ok(())
}

fn snapshot_tree(root: &Path) -> R<String> {
    let mut rows = Vec::new();
    snapshot_rows(root, "", &mut rows)?;
    let encoded = serde_json::to_vec(&Value::Array(rows))
        .map_err(|e| Refused(format!("could not encode a snapshot: {e}")))?;
    Ok(sha256_bytes(&encoded))
}

struct Snapshot {
    dirs: Vec<(PathBuf, String)>,
    writes: Vec<(PathBuf, String)>,
}

fn snapshot_targets(dirs: &[PathBuf], writes: &[PlannedWrite]) -> R<Snapshot> {
    let mut dir_rows = Vec::new();
    for target in dirs {
        dir_rows.push((target.clone(), snapshot_tree(target)?));
    }
    let write_rows =
        writes.iter().map(|w| (w.path.clone(), sha256_bytes(&w.before))).collect::<Vec<_>>();
    Ok(Snapshot { dirs: dir_rows, writes: write_rows })
}

fn revalidate_snapshot(snapshot: &Snapshot) -> R<()> {
    for (path, digest) in &snapshot.dirs {
        assert_plain_directory(path, "cleanup target", false)?;
        if &snapshot_tree(path)? != digest {
            return fail(format!("cleanup target changed after preflight: {}", path.display()));
        }
    }
    for (path, digest) in &snapshot.writes {
        assert_plain_file(path, "planned configuration write")?;
        if &sha256_bytes(&read_file(path, "planned configuration write")?) != digest {
            return fail(format!("configuration changed after preflight: {}", path.display()));
        }
    }
    Ok(())
}

// ── plan ───────────────────────────────────────────────────────────────────

struct DistributionPlan {
    status: &'static str,
    dirs: Vec<PathBuf>,
    writes: Vec<PlannedWrite>,
    snapshot: Snapshot,
    repo_copy: bool,
    #[allow(dead_code)]
    proofs: Vec<Proof>,
}

struct PlanInput<'a> {
    mode: &'a str,
    runtimes: &'a [String],
    repo_root: &'a Path,
    plugin_states: &'a [PluginState],
    inventory: &'a [InventoryRecord],
    ledger_path: Option<&'a Path>,
    user_skill_roots: &'a [String],
    codex_hybrid: bool,
}

fn build_distribution_plan(input: PlanInput<'_>) -> R<DistributionPlan> {
    if !["plugin-first", "repo-copy"].contains(&input.mode) {
        return fail(format!("unknown distribution mode: {}", input.mode));
    }
    if input.runtimes.is_empty() {
        return fail("at least one runtime is required");
    }
    let selected: Vec<PluginState> = input
        .runtimes
        .iter()
        .map(|runtime| {
            input
                .plugin_states
                .iter()
                .find(|s| &s.runtime == runtime)
                .cloned()
                .unwrap_or(PluginState { runtime: runtime.clone(), ..Default::default() })
        })
        .collect();

    if input.mode == "repo-copy" {
        prove_plugin_inactive(&selected)?;
        return Ok(DistributionPlan {
            status: "ready_for_onboarding",
            dirs: Vec::new(),
            writes: Vec::new(),
            snapshot: Snapshot { dirs: Vec::new(), writes: Vec::new() },
            repo_copy: true,
            proofs: Vec::new(),
        });
    }

    let managed_skills = managed_skill_names(input.inventory)?;
    let mut proofs = Vec::new();
    for state in &selected {
        proofs.push(prove_installed_package(state, input.inventory)?);
    }
    let repo_root =
        std::path::absolute(input.repo_root).unwrap_or_else(|_| input.repo_root.to_path_buf());
    let project = collect_project_cleanup(&repo_root, &managed_skills, input.codex_hybrid)?;
    let user = read_ownership_ledger(input.ledger_path, input.user_skill_roots)?;

    let mut dirs = project.dirs;
    dirs.extend(user.dirs);
    let reals: Vec<PathBuf> =
        dirs.iter().map(|t| dunce::canonicalize(t).unwrap_or_else(|_| t.clone())).collect();
    if reals.iter().collect::<BTreeSet<_>>().len() != reals.len() {
        return fail("cleanup plan contains duplicate or aliased targets");
    }
    let mut writes = project.configs;
    writes.extend(user.update);

    let snapshot = snapshot_targets(&dirs, &writes)?;
    let status = if dirs.is_empty() && writes.is_empty() { "up_to_date" } else { "changes_needed" };
    Ok(DistributionPlan { status, dirs, writes, snapshot, repo_copy: false, proofs })
}

/// A per-run suffix for the quarantine paths. The .mjs uses `randomUUID`; the
/// property that matters is only that a concurrent or crashed run's leftovers
/// cannot be mistaken for this run's, and every quarantine path is checked for
/// prior existence before it is used.
fn transaction_token() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!("{nanos:x}-{:x}", std::process::id())
}

struct ApplyOutcome {
    status: &'static str,
    removed: usize,
    updated: usize,
}

fn apply_distribution_plan(plan: &DistributionPlan) -> R<ApplyOutcome> {
    revalidate_snapshot(&plan.snapshot)?;
    if plan.repo_copy {
        return Ok(ApplyOutcome { status: "ready_for_onboarding", removed: 0, updated: 0 });
    }
    if plan.dirs.is_empty() && plan.writes.is_empty() {
        return Ok(ApplyOutcome { status: "up_to_date", removed: 0, updated: 0 });
    }

    let token = transaction_token();
    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut config_moves: Vec<(&PlannedWrite, PathBuf, PathBuf)> = Vec::new();

    // Everything below runs inside one transaction: the first failure unwinds
    // every rename already made, in reverse, before the error escapes.
    let result = (|| -> R<()> {
        for target in &plan.dirs {
            let quarantine = sibling_suffixed(target, &format!(".bee-cleanup-{token}"));
            if lstat_or_null(&quarantine).is_some() {
                return fail(format!("quarantine path already exists: {}", quarantine.display()));
            }
            fs::rename(target, &quarantine).map_err(|e| {
                Refused(format!("could not quarantine {}: {e}", target.display()))
            })?;
            moved.push((target.clone(), quarantine));
        }
        for item in &plan.writes {
            let temp = sibling_suffixed(&item.path, &format!(".bee-write-{token}"));
            let backup = sibling_suffixed(&item.path, &format!(".bee-cleanup-{token}"));
            if lstat_or_null(&temp).is_some() || lstat_or_null(&backup).is_some() {
                return fail(format!(
                    "configuration transaction path already exists: {}",
                    item.path.display()
                ));
            }
            let original = fs::metadata(&item.path).map_err(|e| {
                Refused(format!("could not stat {}: {e}", item.path.display()))
            })?;
            fs::rename(&item.path, &backup).map_err(|e| {
                Refused(format!("could not back up {}: {e}", item.path.display()))
            })?;
            config_moves.push((item, temp.clone(), backup));
            fs::write(&temp, &item.after)
                .map_err(|e| Refused(format!("could not write {}: {e}", temp.display())))?;
            copy_permissions(&original, &temp);
            fs::rename(&temp, &item.path).map_err(|e| {
                Refused(format!("could not install {}: {e}", item.path.display()))
            })?;
        }
        Ok(())
    })();

    if let Err(error) = result {
        for (item, temp, backup) in config_moves.iter().rev() {
            let _ = fs::remove_file(temp);
            let _ = fs::remove_file(&item.path);
            if lstat_or_null(backup).is_some() {
                let _ = fs::rename(backup, &item.path);
            }
        }
        for (target, quarantine) in moved.iter().rev() {
            if lstat_or_null(quarantine).is_some() && lstat_or_null(target).is_none() {
                let _ = fs::rename(quarantine, target);
            }
        }
        return Err(error);
    }

    for (_, quarantine) in &moved {
        let _ = fs::remove_dir_all(quarantine);
    }
    for (_, _, backup) in &config_moves {
        let _ = fs::remove_file(backup);
    }
    Ok(ApplyOutcome { status: "applied", removed: moved.len(), updated: config_moves.len() })
}

/// `${target}${suffix}` as a sibling path — the .mjs builds these by string
/// concatenation, so the suffix lands on the final component, not inside it.
fn sibling_suffixed(target: &Path, suffix: &str) -> PathBuf {
    let mut s = target.as_os_str().to_os_string();
    s.push(suffix);
    PathBuf::from(s)
}

fn copy_permissions(from: &fs::Metadata, to: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = from.permissions().mode() & 0o777;
        let _ = fs::set_permissions(to, fs::Permissions::from_mode(mode));
    }
    #[cfg(not(unix))]
    {
        let _ = fs::set_permissions(to, from.permissions());
    }
}

// ── CLI ────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct Args {
    apply: bool,
    codex_hybrid: bool,
    user_skill_roots: Vec<String>,
    named: BTreeMap<String, String>,
}

fn parse_args(argv: &[&str]) -> R<Args> {
    let mut options = Args::default();
    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i];
        if arg == "--apply" {
            options.apply = true;
        } else if arg == "--codex-hybrid" {
            // Boolean, like --apply: kept out of the generic "--" fallback
            // below (which always consumes a value) so it never eats the next
            // token. GH #22 P0-1.
            options.codex_hybrid = true;
        } else if arg == "--user-skill-root" {
            i += 1;
            let Some(value) = argv.get(i) else {
                return fail("--user-skill-root needs a value");
            };
            options.user_skill_roots.push((*value).to_string());
        } else if let Some(name) = arg.strip_prefix("--") {
            i += 1;
            let Some(value) = argv.get(i) else {
                return fail(format!("{arg} needs a value"));
            };
            options.named.insert(name.replace('-', "_"), (*value).to_string());
        } else {
            return fail(format!("unexpected argument: {arg}"));
        }
        i += 1;
    }
    Ok(options)
}

fn parse_runtime(value: Option<&String>) -> R<Vec<String>> {
    match value.map(String::as_str) {
        Some("both") => Ok(vec!["claude".into(), "codex".into()]),
        Some(v @ ("claude" | "codex")) => Ok(vec![v.to_string()]),
        _ => fail("--runtime must be claude, codex, or both"),
    }
}

fn run_cli(argv: &[&str]) -> R<Value> {
    let args = parse_args(argv)?;
    let runtimes = parse_runtime(args.named.get("runtime"))?;
    let (mode, repo_root, release_manifest, plugin_state_file) = match (
        args.named.get("mode"),
        args.named.get("repo_root"),
        args.named.get("release_manifest"),
        args.named.get("plugin_state_file"),
    ) {
        (Some(m), Some(r), Some(rm), Some(ps)) => (m, r, rm, ps),
        _ => {
            return fail(
                "--mode, --runtime, --repo-root, --release-manifest, and --plugin-state-file are required",
            )
        }
    };

    // Strip a leading UTF-8 BOM before parsing: install.ps1 writes this state
    // file with PowerShell `Set-Content -Encoding UTF8`, which on PS 5.1
    // prepends one, and a bare parse then fails on it — surfaced as a broken
    // Windows install (#9).
    let raw = read_file(Path::new(plugin_state_file), "plugin state file")?;
    let text = String::from_utf8_lossy(&raw);
    let payload: Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|e| Refused(format!("plugin state file is not valid JSON: {e}")))?;

    let plugin_states: Vec<PluginState> = runtimes
        .iter()
        .map(|runtime| {
            let scoped = payload.get(runtime).unwrap_or(&payload);
            discover_bee_plugin(scoped, runtime)
        })
        .collect();

    let inventory = load_package_inventory(Path::new(release_manifest))?;
    let ledger_path = args.named.get("ledger").map(PathBuf::from);
    let plan = build_distribution_plan(PlanInput {
        mode,
        runtimes: &runtimes,
        repo_root: Path::new(repo_root),
        plugin_states: &plugin_states,
        inventory: &inventory,
        ledger_path: ledger_path.as_deref(),
        user_skill_roots: &args.user_skill_roots,
        codex_hybrid: args.codex_hybrid,
    })?;

    let (status, removed, updated) = if args.apply {
        let outcome = apply_distribution_plan(&plan)?;
        (outcome.status.to_string(), outcome.removed, outcome.updated)
    } else {
        (plan.status.to_string(), plan.dirs.len(), plan.writes.len())
    };

    Ok(json!({
        "ok": true,
        "mode": mode,
        "runtimes": runtimes,
        "dryRun": !args.apply,
        "status": status,
        "removed": removed,
        "updated": updated,
    }))
}

/// Was the bee plugin installed for this runtime, per a client's own listing?
/// Shared with `install-support plugin-installed`, which the installer uses to
/// decide the inverse transition during rollback — one discovery rule, not two.
pub(super) fn discover_plugin_installed(payload: &Value) -> bool {
    discover_bee_plugin(payload, "").installed
}

pub fn run(flags: &[&str]) -> Option<ExitCode> {
    match run_cli(flags) {
        Ok(value) => {
            println!("{value}");
            Some(ExitCode::SUCCESS)
        }
        Err(Refused(message)) => {
            println!("{}", json!({"ok": false, "status": "blocked", "error": message}));
            Some(ExitCode::from(1))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    fn sha_of(body: &str) -> String {
        sha256_bytes(body.as_bytes())
    }

    /// A manifest naming one skill file, plus the package payload the skill
    /// tree lives under, matching what release_manifest emits.
    fn manifest_for(files: Vec<(&str, &str, &str)>) -> String {
        let records: Vec<Value> = files
            .into_iter()
            .map(|(path, role, body)| {
                json!({"path": path, "role": role, "sha256": sha_of(body), "mode": "644"})
            })
            .collect();
        json!({"files": records}).to_string()
    }

    // ── inventory ──────────────────────────────────────────────────────────

    #[test]
    fn inventory_keeps_only_package_roles_and_refuses_malformed_records() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        write(root, "m.json", &manifest_for(vec![
            ("skills/bee-hive/SKILL.md", "plugin_skill", "a"),
            ("docs/notes.md", "documentation", "b"),
        ]));
        let inv = load_package_inventory(&root.join("m.json")).unwrap();
        assert_eq!(inv.len(), 1, "a non-package role must not enter the inventory");
        assert_eq!(inv[0].path, "skills/bee-hive/SKILL.md");

        // An absolute path, a `..` segment, a short sha and a bad mode are each
        // their own refusal — these are the guards that keep a hostile manifest
        // from steering the cleanup outside the package root.
        for bad in [
            json!({"path": "/etc/passwd", "role": "plugin_skill", "sha256": "a".repeat(64), "mode": "644"}),
            json!({"path": "skills/../../x", "role": "plugin_skill", "sha256": "a".repeat(64), "mode": "644"}),
            json!({"path": "skills/bee-hive/x", "role": "plugin_skill", "sha256": "nope", "mode": "644"}),
            json!({"path": "skills/bee-hive/x", "role": "plugin_skill", "sha256": "a".repeat(64), "mode": "9999"}),
        ] {
            write(root, "bad.json", &json!({"files": [bad]}).to_string());
            assert!(load_package_inventory(&root.join("bad.json")).is_err());
        }

        write(root, "dup.json", &json!({"files": [
            {"path": "skills/bee-hive/x", "role": "plugin_skill", "sha256": "a".repeat(64), "mode": "644"},
            {"path": "skills/bee-hive/x", "role": "plugin_skill", "sha256": "a".repeat(64), "mode": "644"},
        ]}).to_string());
        assert!(load_package_inventory(&root.join("dup.json")).is_err(), "duplicate path");

        write(root, "empty.json", &json!({"files": []}).to_string());
        assert!(load_package_inventory(&root.join("empty.json")).is_err(), "no package records");
    }

    #[test]
    fn managed_skill_names_reads_the_second_segment_and_refuses_unsafe_names() {
        let ok = vec![
            InventoryRecord { path: "skills/bee-hive/SKILL.md".into(), sha256: String::new(), mode: String::new(), role: "plugin_skill".into() },
            InventoryRecord { path: "skills/bee-swarming/x.md".into(), sha256: String::new(), mode: String::new(), role: "plugin_skill".into() },
            // Not a plugin_skill: contributes no name.
            InventoryRecord { path: "hooks/h.json".into(), sha256: String::new(), mode: String::new(), role: "plugin_hook".into() },
        ];
        let names = managed_skill_names(&ok).unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains("bee-hive") && names.contains("bee-swarming"));

        // `../` as a skill name is the attack this regex exists to stop.
        let evil = vec![InventoryRecord { path: "skills/../etc/x".into(), sha256: String::new(), mode: String::new(), role: "plugin_skill".into() }];
        assert!(managed_skill_names(&evil).is_err());
        let wrong_head = vec![InventoryRecord { path: "hooks/bee-hive/x".into(), sha256: String::new(), mode: String::new(), role: "plugin_skill".into() }];
        assert!(managed_skill_names(&wrong_head).is_err());
    }

    // ── plugin discovery ───────────────────────────────────────────────────

    #[test]
    fn discovery_matches_bee_by_name_or_version_prefix_and_maps_state_words() {
        let payload = json!({"plugins": [
            {"name": "other", "status": "enabled"},
            {"name": "bee@2.1.0", "status": "enabled", "root": "/pkg", "version": "2.1.0"},
        ]});
        let s = discover_bee_plugin(&payload, "claude");
        assert!(s.installed && s.enabled, "an `enabled` status means installed and enabled");
        assert_eq!(s.root.as_deref(), Some("/pkg"));

        // Absent entirely: neither installed nor enabled, and no root.
        let s = discover_bee_plugin(&json!({"plugins": []}), "codex");
        assert!(!s.installed && !s.enabled && s.root.is_none());

        // `removed` reads as not installed, which also forces not-enabled.
        let s = discover_bee_plugin(&json!([{"name": "bee", "status": "removed"}]), "claude");
        assert!(!s.installed && !s.enabled);

        // Installed but not enabled is a real state and must not collapse to enabled.
        let s = discover_bee_plugin(&json!([{"name": "bee", "installed": true, "status": "disabled"}]), "claude");
        assert!(s.installed && !s.enabled);
    }

    #[test]
    fn a_source_checkout_can_never_stand_in_for_an_installed_package() {
        let state = PluginState {
            runtime: "claude".into(),
            installed: true,
            enabled: true,
            root: Some(if cfg!(windows) { "C:\\pkg".into() } else { "/pkg".into() }),
            version: None,
            source_kind: Some("source_checkout".into()),
        };
        let err = prove_installed_package(&state, &[]).unwrap_err().0;
        assert!(err.contains("source checkout cannot substitute"), "{err}");
    }

    #[test]
    fn an_installed_package_must_match_the_inventory_byte_for_byte() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("pkg");
        write(&pkg, "skills/bee-hive/SKILL.md", "hive");

        let mut inv = vec![InventoryRecord {
            path: "skills/bee-hive/SKILL.md".into(),
            sha256: sha_of("hive"),
            mode: mode_octal(&std::fs::metadata(pkg.join("skills/bee-hive/SKILL.md")).unwrap()),
            role: "plugin_skill".into(),
        }];
        let state = PluginState {
            runtime: "claude".into(),
            installed: true,
            enabled: true,
            root: Some(pkg.to_string_lossy().into_owned()),
            version: None,
            source_kind: None,
        };
        assert!(prove_installed_package(&state, &inv).is_ok(), "a matching tree proves");

        // Content drift is caught, and the message names the file.
        inv[0].sha256 = sha_of("tampered");
        let err = prove_installed_package(&state, &inv).unwrap_err().0;
        assert!(err.contains("changed=skills/bee-hive/SKILL.md"), "{err}");

        // A file present on disk but absent from the manifest is `unexpected` —
        // the half that catches a package carrying something it should not.
        inv[0].sha256 = sha_of("hive");
        write(&pkg, "skills/bee-hive/EXTRA.md", "extra");
        let err = prove_installed_package(&state, &inv).unwrap_err().0;
        assert!(err.contains("unexpected=skills/bee-hive/EXTRA.md"), "{err}");
    }

    // ── hook config cleaning ───────────────────────────────────────────────

    #[test]
    fn only_bee_hook_entries_are_stripped_and_emptied_containers_disappear() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let cfg = json!({
            "otherKey": {"kept": true},
            "hooks": {
                "PreToolUse": [
                    {"matcher": "Edit", "hooks": [
                        {"type": "command", "command": "$CLAUDE_PLUGIN_ROOT/hooks/bee-write-guard.mjs"},
                        {"type": "command", "command": "my-own-linter"}
                    ]},
                    {"matcher": "Write", "hooks": [
                        {"type": "command", "command": "/x/.bee/bin/hooks/bee-session-init.mjs"}
                    ]}
                ]
            }
        });
        write(root, ".claude/settings.json", &serde_json::to_string_pretty(&cfg).unwrap());

        let plan = clean_hook_config(&root.join(".claude/settings.json")).unwrap().unwrap();
        assert_eq!(plan.removed, 2);
        let after: Value = serde_json::from_slice(&plan.after).unwrap();
        assert_eq!(after["otherKey"]["kept"], json!(true), "unrelated keys survive verbatim");
        let groups = after["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(groups.len(), 1, "a group whose entries were all bee entries is dropped");
        assert_eq!(groups[0]["hooks"].as_array().unwrap().len(), 1);
        assert_eq!(groups[0]["hooks"][0]["command"], json!("my-own-linter"));

        // A config with nothing of ours plans no write at all.
        write(root, ".codex/hooks.json", r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"other"}]}]}}"#);
        assert!(clean_hook_config(&root.join(".codex/hooks.json")).unwrap().is_none());

        // An absent file is not an error — it is simply nothing to do.
        assert!(clean_hook_config(&root.join(".claude/missing.json")).unwrap().is_none());
    }

    #[test]
    fn a_bee_named_handler_still_needs_a_bee_shaped_path() {
        // The handler name alone is not enough: the command must also look like
        // it resolves through a bee root, or an unrelated tool that happens to
        // mention the filename would be stripped.
        assert!(recognized_bee_command(Some("$CLAUDE_PLUGIN_ROOT/hooks/bee-write-guard.mjs")));
        assert!(recognized_bee_command(Some("C:\\x\\.bee\\bin\\hooks\\bee-state-sync.mjs")));
        assert!(!recognized_bee_command(Some("/opt/tool/hooks/not-a-bee-hook.mjs")));
        assert!(!recognized_bee_command(Some("echo bee-write-guard.mjs")));
        assert!(!recognized_bee_command(None));
    }

    #[test]
    fn the_codex_hybrid_flag_exempts_exactly_one_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let bee_entry = r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"/x/.bee/bin/hooks/bee-write-guard.mjs"}]}]}}"#;
        write(root, ".claude/settings.json", bee_entry);
        write(root, ".codex/hooks.json", bee_entry);
        let skills: BTreeSet<String> = ["bee-hive".to_string()].into_iter().collect();

        let normal = collect_project_cleanup(root, &skills, false).unwrap();
        assert_eq!(normal.configs.len(), 2, "both hook files are cleaned by default");

        // GH #22 P0-1: the codex-hybrid projection is the only enforcement
        // Codex sessions get, so this pass must not delete what onboarding
        // just wrote.
        let hybrid = collect_project_cleanup(root, &skills, true).unwrap();
        assert_eq!(hybrid.configs.len(), 1);
        assert!(hybrid.configs[0].path.ends_with("settings.json"));
    }

    #[test]
    fn only_release_managed_skill_directories_are_collected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, ".claude/skills/bee-hive/SKILL.md", "x");
        write(root, ".claude/skills/my-own-skill/SKILL.md", "x");
        write(root, ".agents/skills/bee-hive/SKILL.md", "x");
        let skills: BTreeSet<String> = ["bee-hive".to_string()].into_iter().collect();

        let out = collect_project_cleanup(root, &skills, false).unwrap();
        assert_eq!(out.dirs.len(), 2, "one per skill root, and never the user's own skill");
        assert!(out.dirs.iter().all(|d| d.ends_with("bee-hive")));
    }

    // ── the transaction ────────────────────────────────────────────────────

    fn plugin_state_file(root: &Path, pkg: &Path) -> PathBuf {
        let state = json!({"claude": {"plugins": [
            {"name": "bee", "status": "enabled", "root": pkg.to_string_lossy(), "version": "2.1.0"}
        ]}});
        write(root, "state.json", &state.to_string());
        root.join("state.json")
    }

    /// The whole CLI, end to end: a real package, a real repo with fallback
    /// copies, dry run, then apply.
    #[test]
    fn plugin_first_dry_run_then_apply_removes_the_repo_fallbacks() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let pkg = root.join("pkg");
        let repo = root.join("repo");

        write(&pkg, "skills/bee-hive/SKILL.md", "hive");
        let manifest = manifest_for(vec![("skills/bee-hive/SKILL.md", "plugin_skill", "hive")]);
        // The manifest's mode must match what this filesystem actually reports.
        let real_mode = mode_octal(&std::fs::metadata(pkg.join("skills/bee-hive/SKILL.md")).unwrap());
        let manifest = manifest.replace("\"mode\":\"644\"", &format!("\"mode\":\"{real_mode}\""));
        write(root, "manifest.json", &manifest);

        write(&repo, ".claude/skills/bee-hive/SKILL.md", "stale copy");
        write(&repo, ".claude/settings.json", r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"/r/.bee/bin/hooks/bee-write-guard.mjs"}]}]}}"#);

        let state = plugin_state_file(root, &pkg);
        let repo_s = repo.to_string_lossy().into_owned();
        let manifest_s = root.join("manifest.json").to_string_lossy().into_owned();
        let state_s = state.to_string_lossy().into_owned();
        let argv = vec![
            "--mode", "plugin-first", "--runtime", "claude",
            "--repo-root", &repo_s,
            "--release-manifest", &manifest_s,
            "--plugin-state-file", &state_s,
        ];

        let dry = run_cli(&argv).unwrap();
        assert_eq!(dry["status"], json!("changes_needed"));
        assert_eq!(dry["dryRun"], json!(true));
        assert_eq!(dry["removed"], json!(1));
        assert_eq!(dry["updated"], json!(1));
        assert!(repo.join(".claude/skills/bee-hive").exists(), "a dry run writes nothing");

        let mut apply_argv = argv.clone();
        apply_argv.push("--apply");
        let applied = run_cli(&apply_argv).unwrap();
        assert_eq!(applied["status"], json!("applied"));
        assert_eq!(applied["removed"], json!(1));
        assert!(!repo.join(".claude/skills/bee-hive").exists(), "the stale copy is gone");
        let settings: Value =
            serde_json::from_slice(&std::fs::read(repo.join(".claude/settings.json")).unwrap()).unwrap();
        assert!(settings.get("hooks").is_none(), "the emptied hooks key is dropped: {settings}");

        // No quarantine litter survives a successful run.
        let leftovers: Vec<String> = std::fs::read_dir(repo.join(".claude/skills"))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".bee-cleanup-") || n.contains(".bee-write-"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");

        // Second run is idempotent.
        let again = run_cli(&argv).unwrap();
        assert_eq!(again["status"], json!("up_to_date"));
    }

    #[test]
    fn repo_copy_mode_refuses_while_the_plugin_is_still_active() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let pkg = root.join("pkg");
        write(&pkg, "skills/bee-hive/SKILL.md", "hive");
        write(root, "manifest.json", &manifest_for(vec![("skills/bee-hive/SKILL.md", "plugin_skill", "hive")]));
        let state = plugin_state_file(root, &pkg);

        let repo_s = root.to_string_lossy().into_owned();
        let manifest_s = root.join("manifest.json").to_string_lossy().into_owned();
        let state_s = state.to_string_lossy().into_owned();
        let argv = vec![
            "--mode", "repo-copy", "--runtime", "claude",
            "--repo-root", &repo_s,
            "--release-manifest", &manifest_s,
            "--plugin-state-file", &state_s,
        ];
        let err = run_cli(&argv).unwrap_err().0;
        assert!(err.contains("bee plugin remains active for: claude"), "{err}");

        // With the plugin gone, repo-copy is the go-ahead for onboarding.
        write(root, "state.json", &json!({"claude": {"plugins": []}}).to_string());
        assert_eq!(run_cli(&argv).unwrap()["status"], json!("ready_for_onboarding"));
    }

    #[test]
    fn a_utf8_bom_on_the_state_file_is_stripped() {
        // install.ps1 writes this file with PowerShell `Set-Content -Encoding
        // UTF8`, which on PS 5.1 prepends a BOM. Parsing it raw broke Windows
        // installs (#9), reported only as "Distribution preflight refused".
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let pkg = root.join("pkg");
        write(&pkg, "skills/bee-hive/SKILL.md", "hive");
        write(root, "manifest.json", &manifest_for(vec![("skills/bee-hive/SKILL.md", "plugin_skill", "hive")]));
        write(root, "state.json", &format!("\u{feff}{}", json!({"claude": {"plugins": []}})));

        let repo_s = root.to_string_lossy().into_owned();
        let manifest_s = root.join("manifest.json").to_string_lossy().into_owned();
        let state_s = root.join("state.json").to_string_lossy().into_owned();
        let out = run_cli(&[
            "--mode", "repo-copy", "--runtime", "claude",
            "--repo-root", &repo_s,
            "--release-manifest", &manifest_s,
            "--plugin-state-file", &state_s,
        ]).unwrap();
        assert_eq!(out["status"], json!("ready_for_onboarding"));
    }

    #[test]
    fn codex_hybrid_is_boolean_and_never_eats_the_following_token() {
        // GH #22 P0-1: it sits outside the generic `--flag value` fallback.
        let args = parse_args(&["--codex-hybrid", "--mode", "plugin-first"]).unwrap();
        assert!(args.codex_hybrid);
        assert_eq!(args.named.get("mode").map(String::as_str), Some("plugin-first"));

        let args = parse_args(&["--user-skill-root", "/a", "--user-skill-root", "/b"]).unwrap();
        assert_eq!(args.user_skill_roots, vec!["/a".to_string(), "/b".to_string()]);

        assert!(parse_args(&["stray"]).is_err(), "a positional is a refusal");
    }

    #[test]
    fn runtime_both_expands_in_a_fixed_order() {
        assert_eq!(parse_runtime(Some(&"both".to_string())).unwrap(), vec!["claude", "codex"]);
        assert_eq!(parse_runtime(Some(&"codex".to_string())).unwrap(), vec!["codex"]);
        assert!(parse_runtime(Some(&"emacs".to_string())).is_err());
        assert!(parse_runtime(None).is_err());
    }

    #[test]
    fn missing_required_flags_refuse_before_any_filesystem_work() {
        let err = run_cli(&["--runtime", "claude"]).unwrap_err().0;
        assert!(err.contains("--mode, --runtime, --repo-root"), "{err}");
    }
}
