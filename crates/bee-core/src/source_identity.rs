//! source_identity — `classifySource`, ported from
//! `.bee/bin/lib/source-identity.mjs` (rust-port-20, CONTEXT.md D3). PURE:
//! only read probes (`exists`/`canonicalize`/`read_to_string`); never
//! mutates anything and never panics — any probe failure, unparseable
//! manifest, or ambiguity resolves to [`SourceKind::Unknown`] (fail-closed,
//! SRC-04), matching the mjs source's own contract.
//!
//! `.bee/bin/lib/source-identity.mjs` is FROZEN for the duration of the
//! rust-port feature (D1).

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

/// The render-provenance sidecar (D9): a skills root carrying it is a
/// rendered per-runtime PROJECTION, refused as an onboarding source for
/// any target.
pub const RENDER_SIDECAR: &str = ".bee-render.json";

fn realpath_or_none(p: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(p).ok()
}

fn exists_safe(p: &Path) -> bool {
    p.try_exists().unwrap_or(false)
}

/// `classifySource({hiveDir, homeDir})`. `hive_dir` is the running
/// launcher's `.../bee-hive` directory; `home_dir` is the user's home dir
/// (for the legacy global-root check), both caller-supplied — this
/// function never resolves either itself (same "caller supplies
/// already-resolved values" pattern as the rest of this cell). Returns
/// `{kind, root, markers}` as a raw [`Value`] (matching every other
/// bee-core reader's JSON-passthrough convention) rather than a typed
/// struct, since every consumer (the `bee status`/onboarding oracle) reads
/// it as JSON.
pub fn classify_source(hive_dir: Option<&Path>, home_dir: Option<&Path>) -> Value {
    let Some(hive_dir) = hive_dir else {
        return json!({"kind": "unknown", "root": Value::Null, "markers": {"reason": "no hiveDir"}});
    };

    // .../skills (or .agents/skills, .claude/skills)
    let Some(source_root) = hive_dir.parent() else {
        return json!({"kind": "unknown", "root": Value::Null, "markers": {"reason": "no hiveDir"}});
    };
    // the package / repo root
    let Some(plugin_root) = source_root.parent() else {
        return json!({"kind": "unknown", "root": Value::Null, "markers": {"reason": "no hiveDir"}});
    };

    let markers_base = json!({
        "source_root": source_root.to_string_lossy(),
        "plugin_root": plugin_root.to_string_lossy(),
    });

    // (0) rendered_projection FIRST — checked ahead of every other kind so
    // a rendered .claude/.agents root is refused as a source for ANY
    // target (D9 provenance).
    if exists_safe(&source_root.join(RENDER_SIDECAR)) {
        let mut markers = markers_base.clone();
        markers["render_sidecar"] = json!(true);
        return json!({"kind": "rendered_projection", "root": plugin_root.to_string_lossy(), "markers": markers});
    }

    // (1) legacy_global FIRST — the global ~/.claude/skills root also has
    // a `.claude` grandparent, so it would collide with project_projection
    // below; the realpath match to the true global root disambiguates it.
    if let Some(home) = home_dir {
        let global_root = home.join(".claude").join("skills");
        let rp = realpath_or_none(source_root);
        let rp_global = realpath_or_none(&global_root);
        if let (Some(rp), Some(rp_global)) = (rp, rp_global) {
            if rp == rp_global {
                let mut markers = markers_base.clone();
                markers["global_root"] = json!(global_root.to_string_lossy());
                return json!({"kind": "legacy_global", "root": plugin_root.to_string_lossy(), "markers": markers});
            }
        }
    }

    // (2) project_projection — launcher under a host's .agents/skills or
    // .claude/skills.
    let projection_parent = plugin_root.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    if projection_parent == ".agents" || projection_parent == ".claude" {
        let mut markers = markers_base.clone();
        markers["projection_parent"] = json!(projection_parent);
        return json!({"kind": "project_projection", "root": plugin_root.to_string_lossy(), "markers": markers});
    }

    // (3)/(4) a manifested package: .claude-plugin/plugin.json at the
    // package root.
    let plugin_manifest = plugin_root.join(".claude-plugin").join("plugin.json");
    if exists_safe(&plugin_manifest) {
        // SRC-04: an unparseable manifest is `unknown`, never a usable source.
        let parseable = std::fs::read_to_string(&plugin_manifest)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .is_some();
        if !parseable {
            let mut markers = markers_base.clone();
            markers["reason"] = json!("plugin.json unparseable");
            return json!({"kind": "unknown", "root": plugin_root.to_string_lossy(), "markers": markers});
        }
        if exists_safe(&plugin_root.join(".git")) {
            let mut markers = markers_base.clone();
            markers["plugin_manifest"] = json!(true);
            markers["git"] = json!(true);
            return json!({"kind": "source_checkout", "root": plugin_root.to_string_lossy(), "markers": markers});
        }
        // plugin.json without .git — a distributed snapshot. SRC-03: it
        // may source the same repo's runtime + projection, but is NEVER a
        // global/plugin-target authority.
        let mut markers = markers_base.clone();
        markers["plugin_manifest"] = json!(true);
        markers["git"] = json!(false);
        markers["can_target_global"] = json!(false);
        return json!({"kind": "plugin_package", "root": plugin_root.to_string_lossy(), "markers": markers});
    }

    // (5) unknown — no manifest, not a projection, not the global root: fail closed.
    let mut markers = markers_base;
    markers["reason"] = json!("no plugin manifest");
    json!({"kind": "unknown", "root": plugin_root.to_string_lossy(), "markers": markers})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_hive_dir_is_unknown() {
        let result = classify_source(None, None);
        assert_eq!(result["kind"], json!("unknown"));
    }

    #[test]
    fn source_checkout_needs_plugin_manifest_and_git() {
        let dir = tempfile::tempdir().unwrap();
        let hive = dir.path().join("skills").join("bee-hive");
        std::fs::create_dir_all(&hive).unwrap();
        std::fs::create_dir_all(dir.path().join(".claude-plugin")).unwrap();
        std::fs::write(dir.path().join(".claude-plugin").join("plugin.json"), "{}").unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let result = classify_source(Some(&hive), None);
        assert_eq!(result["kind"], json!("source_checkout"));
    }

    #[test]
    fn plugin_package_without_git() {
        let dir = tempfile::tempdir().unwrap();
        let hive = dir.path().join("skills").join("bee-hive");
        std::fs::create_dir_all(&hive).unwrap();
        std::fs::create_dir_all(dir.path().join(".claude-plugin")).unwrap();
        std::fs::write(dir.path().join(".claude-plugin").join("plugin.json"), "{}").unwrap();
        let result = classify_source(Some(&hive), None);
        assert_eq!(result["kind"], json!("plugin_package"));
    }

    #[test]
    fn rendered_projection_wins_over_everything() {
        let dir = tempfile::tempdir().unwrap();
        let hive = dir.path().join("skills").join("bee-hive");
        std::fs::create_dir_all(&hive).unwrap();
        std::fs::write(dir.path().join("skills").join(RENDER_SIDECAR), "{}").unwrap();
        std::fs::create_dir_all(dir.path().join(".claude-plugin")).unwrap();
        std::fs::write(dir.path().join(".claude-plugin").join("plugin.json"), "{}").unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let result = classify_source(Some(&hive), None);
        assert_eq!(result["kind"], json!("rendered_projection"));
    }

    #[test]
    fn project_projection_under_dot_claude() {
        let dir = tempfile::tempdir().unwrap();
        let hive = dir.path().join(".claude").join("skills").join("bee-hive");
        std::fs::create_dir_all(&hive).unwrap();
        let result = classify_source(Some(&hive), None);
        assert_eq!(result["kind"], json!("project_projection"));
    }

    #[test]
    fn unknown_manifest_is_unparseable() {
        let dir = tempfile::tempdir().unwrap();
        let hive = dir.path().join("skills").join("bee-hive");
        std::fs::create_dir_all(&hive).unwrap();
        std::fs::create_dir_all(dir.path().join(".claude-plugin")).unwrap();
        std::fs::write(dir.path().join(".claude-plugin").join("plugin.json"), "not json").unwrap();
        let result = classify_source(Some(&hive), None);
        assert_eq!(result["kind"], json!("unknown"));
        assert_eq!(result["markers"]["reason"], json!("plugin.json unparseable"));
    }
}
