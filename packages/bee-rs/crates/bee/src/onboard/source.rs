// onboard::source — engine geometry, source identity, and the three-version
// resolution layer.
//
// Provenance: onboard_bee.mjs's module-scope geometry constants (l. 82–115),
// skillsTargetRoot (l. 398), skillSyncTargets (l. 404), unknownVersionsTriple
// (l. 429), readVersionStrict (l. 448), readManifestVersionStrict (l. 487),
// readSourceReleaseIdentity (l. 514), compareVersions (l. 607) and
// versionLabel (l. 618) — plus lib/source-identity.mjs classifySource, which
// this script imports.

use super::templates::REPO_SKILL_TARGETS;
use super::util::{exists, home_dir, lstat_if_exists, read_text_if_exists, realpath, split_lines};
use serde_json::Value;
use std::path::{Path, PathBuf};

// ── engine geometry ────────────────────────────────────────────────────────

/// The paths onboard_bee.mjs derives from `import.meta.url`. A Rust binary
/// has no script path, so the plugin root is DISCOVERED (see `locate`) and
/// every other path is the same arithmetic walked from it:
///   ENGINE_DIR = <root>/packages/bee, PLUGIN_ROOT = <root>,
///   SKILLS_ROOT = <root>/skills (never derived from ENGINE_DIR — that would
///   make sync walk packages/, not skills/).
#[derive(Debug, Clone)]
pub struct Engine {
    pub plugin_root: PathBuf,
    pub skills_root: PathBuf,
    pub plugin_hooks_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub templates_lib_dir: PathBuf,
    pub templates_prompts_dir: PathBuf,
    pub templates_statusline_dir: PathBuf,
    pub templates_agents_dir: PathBuf,
    pub expertise_dir: PathBuf,
    pub agents_block_template: PathBuf,
}

impl Engine {
    pub fn from_plugin_root(plugin_root: PathBuf) -> Self {
        let templates_dir = plugin_root.join("packages").join("bee");
        Self {
            skills_root: plugin_root.join("skills"),
            plugin_hooks_dir: templates_dir.join("hooks"),
            templates_lib_dir: templates_dir.join("lib"),
            templates_prompts_dir: templates_dir.join("prompts"),
            templates_statusline_dir: templates_dir.join("statusline"),
            templates_agents_dir: templates_dir.join("agents"),
            expertise_dir: plugin_root.join("expertise"),
            agents_block_template: templates_dir.join("AGENTS.block.md"),
            templates_dir,
            plugin_root,
        }
    }

    /// Locate the bee source checkout that plays the part of `SCRIPT_PATH`'s
    /// package root. Resolution order mirrors js_fallback::find_entry so both
    /// runtimes agree about which checkout is "this bee":
    ///   1. BEE_JS_ENTRY (the diff harness's own knob) — its grandparent's
    ///      parent is the package root.
    ///   2. Walk up from the executable's directory, then from cwd, for
    ///      `packages/bee/scripts/onboard_bee.mjs`.
    /// None ⇒ the probe declines the command (strangler routing): without a
    /// source tree there is nothing authoritative to vendor FROM.
    pub fn locate() -> Option<Self> {
        const MARKER: &str = "packages/bee/scripts/onboard_bee.mjs";
        if let Some(entry) = std::env::var_os("BEE_JS_ENTRY") {
            let entry = PathBuf::from(entry);
            if let Some(root) = entry.parent().and_then(Path::parent).and_then(Path::parent) {
                if super::util::join_rel(root, MARKER).is_file() {
                    return Some(Self::from_plugin_root(root.to_path_buf()));
                }
            }
        }
        let mut starts: Vec<PathBuf> = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                starts.push(dir.to_path_buf());
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            starts.push(cwd);
        }
        for start in starts {
            let mut dir = Some(start.as_path());
            while let Some(d) = dir {
                if super::util::join_rel(d, MARKER).is_file() {
                    return Some(Self::from_plugin_root(d.to_path_buf()));
                }
                dir = d.parent();
            }
        }
        None
    }
}

/// skillsTargetRoot (l. 398): the legacy global root.
pub fn skills_target_root() -> PathBuf {
    home_dir().join(".claude").join("skills")
}

/// skillSyncTargets (l. 404): stable order (repo-claude, repo-agents, then
/// global) — blocked-first aggregates surface the FIRST blocked target.
pub fn skill_sync_targets(repo_root: &Path, global_skills: bool) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = REPO_SKILL_TARGETS
        .iter()
        .map(|(kind, segments)| {
            let mut p = repo_root.to_path_buf();
            for s in *segments {
                p.push(s);
            }
            ((*kind).to_string(), p)
        })
        .collect();
    if global_skills {
        out.push(("global".to_string(), skills_target_root()));
    }
    out
}

/// The `.claude/skills` / `.agents/skills` segments joined the way the
/// out-of-repo refusal message prints them (`path.join(...segments)`).
pub fn repo_target_segments_joined(kind: &str) -> String {
    REPO_SKILL_TARGETS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, segs)| segs.join(&std::path::MAIN_SEPARATOR.to_string()))
        .unwrap_or_default()
}

// ── version resolution ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionState {
    Absent,
    Unknown,
    Resolved(String),
}

impl VersionState {
    /// versionLabel (l. 618).
    pub fn label(&self) -> &str {
        match self {
            VersionState::Absent => "absent",
            VersionState::Unknown => "unknown",
            VersionState::Resolved(v) => v,
        }
    }
    pub fn value(&self) -> Option<&str> {
        match self {
            VersionState::Resolved(v) => Some(v),
            _ => None,
        }
    }
    pub fn is_resolved(&self) -> bool {
        matches!(self, VersionState::Resolved(_))
    }
    pub fn is_unknown(&self) -> bool {
        matches!(self, VersionState::Unknown)
    }
}

/// unknownVersionsTriple (l. 429).
pub fn unknown_versions_triple() -> Value {
    serde_json::json!({"source": "unknown", "host_helpers": "unknown", "installed_skills": "unknown"})
}

/// `NUMERIC_RELEASE_VERSION_RE = /^\d+\.\d+\.\d+$/`
pub fn is_numeric_release(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// compareVersions (l. 607) — only ever called on NUMERIC_RELEASE-validated
/// values, so the parse cannot fail.
pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let pa: Vec<u64> = a.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    let pb: Vec<u64> = b.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    for i in 0..3 {
        let (x, y) = (pa.get(i).copied().unwrap_or(0), pb.get(i).copied().unwrap_or(0));
        if x != y {
            return x.cmp(&y);
        }
    }
    std::cmp::Ordering::Equal
}

/// `BEE_VERSION_LINE_RE = /^export const BEE_VERSION = ['"]([^'"]*)['"];?[ \t]*\r?$/gm`
/// — every line-anchored match in the text. NOTE (faithful to the original):
/// the opening and closing quote characters need not agree.
fn bee_version_matches(text: &str) -> Vec<String> {
    const PREFIX: &str = "export const BEE_VERSION = ";
    let mut out = Vec::new();
    for line in split_lines(text) {
        let Some(rest) = line.strip_prefix(PREFIX) else { continue };
        let mut chars = rest.chars();
        let Some(open) = chars.next() else { continue };
        if open != '\'' && open != '"' {
            continue;
        }
        let after_open = &rest[open.len_utf8()..];
        // `[^'"]*` then `['"]`
        let Some(close_idx) = after_open.find(['\'', '"']) else { continue };
        let value = &after_open[..close_idx];
        let close = after_open[close_idx..].chars().next().unwrap();
        let mut tail = &after_open[close_idx + close.len_utf8()..];
        if let Some(t) = tail.strip_prefix(';') {
            tail = t;
        }
        // `[ \t]*\r?$` — split_lines already removed a CRLF's `\r`, so only
        // horizontal whitespace may remain.
        if tail.chars().all(|c| c == ' ' || c == '\t') {
            out.push(value.to_string());
        }
    }
    out
}

/// `path.relative(componentRoot, stateFile)` for the only shape this script
/// uses: a marker constructed by joining under `componentRoot`. Anything that
/// is not literally under the root reads as "escaped" (None), which
/// readVersionStrict turns into `unknown` — the same fail-closed answer Node
/// gives for a `..`-prefixed or absolute relative path.
fn relative_under(root: &Path, file: &Path) -> Option<Vec<String>> {
    let r = root.to_string_lossy().into_owned();
    let f = file.to_string_lossy().into_owned();
    let sep = std::path::MAIN_SEPARATOR;
    let prefix = format!("{r}{sep}");
    let rest = f.strip_prefix(&prefix)?;
    if rest.is_empty() {
        return None;
    }
    Some(rest.split(sep).map(str::to_string).collect())
}

/// readVersionStrict (l. 448): fallback-free. `treeExists == false` ⇒
/// "absent" (fresh install, proceed); an EXISTING tree whose version cannot
/// be read ⇒ "unknown" (refuse, never forceable). With `component_root`,
/// EVERY path component is lstat'ed — a symlinked directory on the way is as
/// untrusted as a symlinked marker.
pub fn read_version_strict(
    state_file: &Path,
    tree_exists: bool,
    component_root: Option<&Path>,
) -> VersionState {
    if !tree_exists {
        return VersionState::Absent;
    }
    let mut components: Vec<PathBuf> = Vec::new();
    if let Some(root) = component_root {
        let Some(parts) = relative_under(root, state_file) else {
            return VersionState::Unknown; // marker escaped the managed root
        };
        let mut current = root.to_path_buf();
        for part in parts {
            current = current.join(part);
            components.push(current.clone());
        }
    } else {
        components.push(state_file.to_path_buf());
    }
    let last = components.len().saturating_sub(1);
    for (i, comp) in components.iter().enumerate() {
        let is_marker = i == last;
        match lstat_if_exists(comp) {
            None => return VersionState::Unknown,
            Some(st) => {
                if st.is_symlink || (is_marker && !st.is_file) || (!is_marker && !st.is_dir) {
                    return VersionState::Unknown;
                }
            }
        }
    }
    let Ok(bytes) = std::fs::read(state_file) else { return VersionState::Unknown };
    let text = String::from_utf8_lossy(&bytes);
    let matches = bee_version_matches(&text);
    if matches.len() != 1 || !is_numeric_release(&matches[0]) {
        return VersionState::Unknown;
    }
    VersionState::Resolved(matches[0].clone())
}

/// readManifestVersionStrict (l. 487): a REGULAR, non-symlinked JSON file
/// whose `version` is a numeric release string.
pub fn read_manifest_version_strict(manifest_path: &Path) -> VersionState {
    let Some(st) = lstat_if_exists(manifest_path) else { return VersionState::Unknown };
    if st.is_symlink || !st.is_file {
        return VersionState::Unknown;
    }
    let Ok(bytes) = std::fs::read(manifest_path) else { return VersionState::Unknown };
    let Ok(parsed) = serde_json::from_str::<Value>(&String::from_utf8_lossy(&bytes)) else {
        return VersionState::Unknown;
    };
    match parsed.get("version").and_then(Value::as_str) {
        Some(v) if parsed.is_object() && is_numeric_release(v) => VersionState::Resolved(v.into()),
        _ => VersionState::Unknown,
    }
}

// ── source identity (lib/source-identity.mjs) ──────────────────────────────

/// classifySource({hiveDir, homeDir}).kind — pure read probes, never throws,
/// every ambiguity resolves to "unknown" (SRC-04).
pub fn classify_source_kind(hive_dir: &Path, home: &Path) -> &'static str {
    let Some(source_root) = hive_dir.parent() else { return "unknown" };
    let Some(plugin_root) = source_root.parent() else { return "unknown" };

    // (0) rendered_projection FIRST (D9 provenance).
    if exists(&source_root.join(super::templates::RENDER_SIDECAR)) {
        return "rendered_projection";
    }
    // (1) legacy_global — realpath match against <home>/.claude/skills.
    if !home.as_os_str().is_empty() {
        let global_root = home.join(".claude").join("skills");
        if let (Some(rp), Some(rpg)) = (realpath(source_root), realpath(&global_root)) {
            if rp == rpg {
                return "legacy_global";
            }
        }
    }
    // (2) project_projection — launcher under a host's .agents/.claude.
    let parent_name = plugin_root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    if parent_name == ".agents" || parent_name == ".claude" {
        return "project_projection";
    }
    // (3)/(4) a manifested package.
    let manifest = plugin_root.join(".claude-plugin").join("plugin.json");
    if exists(&manifest) {
        let text = read_text_if_exists(&manifest);
        if serde_json::from_str::<Value>(&text).is_err() {
            return "unknown";
        }
        if exists(&plugin_root.join(".git")) {
            return "source_checkout";
        }
        return "plugin_package";
    }
    "unknown"
}

// ── readSourceReleaseIdentity ──────────────────────────────────────────────

pub struct BlockedSource {
    pub status: &'static str,
    pub reason: String,
    pub forceable: bool,
}

pub struct ReleaseIdentity {
    pub version: Option<String>,
    /// (component name, resolved version state) — component[0] is always the
    /// runtime marker, which blockedSourceIdentitySkillSync reads for its
    /// `source` label.
    pub components: Vec<(String, VersionState)>,
    pub blocked: Option<BlockedSource>,
}

/// readSourceReleaseIdentity (l. 514): the canonical runtime marker plus both
/// package manifests are ONE authoritative source identity.
pub fn read_source_release_identity(engine: &Engine) -> ReleaseIdentity {
    let runtime_component = (
        "packages/bee/lib/state.mjs".to_string(),
        read_version_strict(&engine.templates_lib_dir.join("state.mjs"), true, None),
    );
    let source_kind = classify_source_kind(&engine.skills_root.join("bee-hive"), &home_dir());

    if source_kind == "rendered_projection" {
        return ReleaseIdentity {
            version: None,
            components: vec![runtime_component],
            blocked: Some(BlockedSource {
                status: "blocked_no_source",
                reason: "onboarding source is a rendered per-runtime projection (carries the render provenance marker) - a projection is never an authoritative source for any target; use the canonical checkout or plugin package".to_string(),
                forceable: false,
            }),
        };
    }

    if source_kind == "project_projection" || source_kind == "legacy_global" {
        if !runtime_component.1.is_resolved() {
            return ReleaseIdentity {
                version: None,
                components: vec![runtime_component],
                blocked: Some(BlockedSource {
                    status: "blocked_no_source",
                    reason: "projection runtime version is missing, unreadable, or non-numeric"
                        .to_string(),
                    forceable: false,
                }),
            };
        }
        let v = runtime_component.1.value().unwrap().to_string();
        return ReleaseIdentity { version: Some(v), components: vec![runtime_component], blocked: None };
    }

    let components = vec![
        runtime_component,
        (
            ".claude-plugin/plugin.json".to_string(),
            read_manifest_version_strict(
                &engine.plugin_root.join(".claude-plugin").join("plugin.json"),
            ),
        ),
        (
            ".codex-plugin/plugin.json".to_string(),
            read_manifest_version_strict(
                &engine.plugin_root.join(".codex-plugin").join("plugin.json"),
            ),
        ),
    ];
    let unreadable: Vec<&String> =
        components.iter().filter(|(_, v)| !v.is_resolved()).map(|(n, _)| n).collect();
    if !unreadable.is_empty() {
        let names: Vec<String> = unreadable.into_iter().cloned().collect();
        return ReleaseIdentity {
            version: None,
            blocked: Some(BlockedSource {
                status: "blocked_no_source",
                reason: format!(
                    "authoritative source release tuple is invalid: missing, unreadable, or non-numeric {}",
                    names.join(", ")
                ),
                forceable: false,
            }),
            components,
        };
    }
    let mut distinct: Vec<&str> = Vec::new();
    for (_, v) in &components {
        let val = v.value().unwrap();
        if !distinct.contains(&val) {
            distinct.push(val);
        }
    }
    if distinct.len() != 1 {
        let detail: Vec<String> = components
            .iter()
            .map(|(n, v)| format!("{n}={}", v.value().unwrap()))
            .collect();
        return ReleaseIdentity {
            version: None,
            blocked: Some(BlockedSource {
                status: "blocked_no_source",
                reason: format!(
                    "authoritative source release tuple is invalid: tuple members disagree ({})",
                    detail.join(", ")
                ),
                forceable: false,
            }),
            components,
        };
    }
    let version = components[0].1.value().unwrap().to_string();
    ReleaseIdentity { version: Some(version), components, blocked: None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bee_version_line_matching_is_line_anchored() {
        assert_eq!(bee_version_matches("export const BEE_VERSION = '1.2.3';\n"), vec!["1.2.3"]);
        assert_eq!(bee_version_matches("export const BEE_VERSION = \"1.2.3\"\r\n"), vec!["1.2.3"]);
        // A comment decoy (substring, not line-anchored) never resolves.
        assert!(bee_version_matches("// export const BEE_VERSION = '9.9.9';\n").is_empty());
        // Trailing prose after the declaration is not a match.
        assert!(bee_version_matches("export const BEE_VERSION = '1.2.3'; // x\n").is_empty());
        // Two declarations → the caller refuses (len != 1).
        assert_eq!(
            bee_version_matches("export const BEE_VERSION = '1.0.0';\nexport const BEE_VERSION = '2.0.0';\n").len(),
            2
        );
    }

    #[test]
    fn numeric_release_gate() {
        assert!(is_numeric_release("0.1.23"));
        assert!(!is_numeric_release("1.2"));
        assert!(!is_numeric_release("1.2.3-beta"));
        assert!(!is_numeric_release("v1.2.3"));
    }

    #[test]
    fn version_ordering() {
        use std::cmp::Ordering::*;
        assert_eq!(compare_versions("1.2.3", "1.2.3"), Equal);
        assert_eq!(compare_versions("1.2.3", "1.10.0"), Less);
        assert_eq!(compare_versions("2.0.0", "1.99.99"), Greater);
    }

    #[test]
    fn read_version_strict_states() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("state.mjs");
        assert_eq!(read_version_strict(&marker, false, None), VersionState::Absent);
        assert_eq!(read_version_strict(&marker, true, None), VersionState::Unknown);
        std::fs::write(&marker, "export const BEE_VERSION = '3.4.5';\n").unwrap();
        assert_eq!(
            read_version_strict(&marker, true, None),
            VersionState::Resolved("3.4.5".into())
        );
        std::fs::write(&marker, "export const BEE_VERSION = 'nope';\n").unwrap();
        assert_eq!(read_version_strict(&marker, true, None), VersionState::Unknown);
    }

    #[test]
    fn read_version_strict_walks_components_under_root() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("bee-hive").join("templates").join("lib");
        std::fs::create_dir_all(&nested).unwrap();
        let marker = nested.join("state.mjs");
        std::fs::write(&marker, "export const BEE_VERSION = '1.0.0';\n").unwrap();
        assert_eq!(
            read_version_strict(&marker, true, Some(dir.path())),
            VersionState::Resolved("1.0.0".into())
        );
        // A marker outside the managed root is never trusted.
        let outside = tempfile::tempdir().unwrap();
        let other = outside.path().join("state.mjs");
        std::fs::write(&other, "export const BEE_VERSION = '1.0.0';\n").unwrap();
        assert_eq!(read_version_strict(&other, true, Some(dir.path())), VersionState::Unknown);
    }

    #[test]
    fn manifest_version_strict_rejects_non_numeric() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("plugin.json");
        std::fs::write(&f, r#"{"version":"1.2.3"}"#).unwrap();
        assert_eq!(read_manifest_version_strict(&f), VersionState::Resolved("1.2.3".into()));
        std::fs::write(&f, r#"{"version":"1.2"}"#).unwrap();
        assert_eq!(read_manifest_version_strict(&f), VersionState::Unknown);
        std::fs::write(&f, "not json").unwrap();
        assert_eq!(read_manifest_version_strict(&f), VersionState::Unknown);
        assert_eq!(read_manifest_version_strict(&dir.path().join("gone")), VersionState::Unknown);
    }

    #[test]
    fn classify_source_detects_projection_and_package() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();

        // rendered projection: sidecar at the skills root.
        let proj = dir.path().join("pkg").join("skills");
        std::fs::create_dir_all(proj.join("bee-hive")).unwrap();
        std::fs::write(proj.join(".bee-render.json"), "{}").unwrap();
        assert_eq!(classify_source_kind(&proj.join("bee-hive"), &home), "rendered_projection");
        std::fs::remove_file(proj.join(".bee-render.json")).unwrap();

        // no manifest → unknown
        assert_eq!(classify_source_kind(&proj.join("bee-hive"), &home), "unknown");

        // plugin package (manifest, no .git)
        std::fs::create_dir_all(dir.path().join("pkg").join(".claude-plugin")).unwrap();
        std::fs::write(
            dir.path().join("pkg").join(".claude-plugin").join("plugin.json"),
            r#"{"version":"1.0.0"}"#,
        )
        .unwrap();
        assert_eq!(classify_source_kind(&proj.join("bee-hive"), &home), "plugin_package");

        // source checkout (manifest + .git)
        std::fs::create_dir_all(dir.path().join("pkg").join(".git")).unwrap();
        assert_eq!(classify_source_kind(&proj.join("bee-hive"), &home), "source_checkout");

        // project projection
        let host = dir.path().join("host").join(".claude").join("skills");
        std::fs::create_dir_all(host.join("bee-hive")).unwrap();
        assert_eq!(classify_source_kind(&host.join("bee-hive"), &home), "project_projection");
    }
}
