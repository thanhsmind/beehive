use super::*;
use crate::verbs::reservations::{js_trim, keys_known, FlagV, Flags};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub(crate) const THIN_LAYER_RULES: [&str; 3] = [
    "Every entry anchors to code: list the files it describes in the concept's bee.anchors.",
    "Never restate what the code shows: keep the why, the boundary and the pitfall, drop the how.",
    "Extend the concept that owns the subject; never file a near-duplicate beside it.",
];

pub(crate) fn delivery_rel(feature: &str) -> String {
    format!("docs/history/{feature}/delivery.md")
}

pub(crate) fn feature_knowledge_files(store_root: &Path, feature: &str) -> Vec<String> {
    let mut files = BTreeSet::new();
    for cell in read_capped_cell_traces(store_root, feature).unwrap_or_default() {
        for file in &cell.files_changed {
            let path = file.trim_start_matches("./");
            if path.starts_with("docs/knowledge/") {
                files.insert(path.to_string());
            }
        }
    }
    files.into_iter().collect()
}

pub(crate) fn build_consolidation(store_root: &Path, docs_root: &Path, feature: &str) -> Result<Value, String> {
    let proposal = match bundle_dir(docs_root) {
        None => json!({"area_updates": [], "pattern_candidates": [], "cells": []}),
        Some(dir) => match build_promotion_split(store_root, docs_root, &dir, feature) {
            None => json!({"area_updates": [], "pattern_candidates": [], "cells": []}),
            Some(Promo::Thrown(msg)) => {
                return Err(msg.replace("knowledge promote:", "knowledge consolidate:"));
            }
            Some(Promo::Ok(p)) => p,
        },
    };
    let delivery = delivery_rel(feature);
    let exists = docs_root.join(&delivery).is_file();
    Ok(json!({
        "feature": feature,
        "proposal": proposal,
        "knowledge_files": feature_knowledge_files(store_root, feature),
        "delivery": {"path": delivery, "exists": exists},
        "rules": THIN_LAYER_RULES,
        "writes": [],
    }))
}

pub(crate) fn consolidation_text(value: &Value) -> String {
    let feature = value["feature"].as_str().unwrap_or_default();
    let mut lines = vec![format!("Consolidation brief for \"{feature}\" (writes nothing)."), String::new()];
    lines.push("Knowledge files the feature's cells changed:".to_string());
    match value["knowledge_files"].as_array() {
        Some(files) if !files.is_empty() => {
            for file in files {
                lines.push(format!("- {}", file.as_str().unwrap_or_default()));
            }
        }
        _ => lines.push("- none".to_string()),
    }
    lines.push(String::new());
    let delivery = &value["delivery"];
    lines.push(format!(
        "Delivery record: {} ({})",
        delivery["path"].as_str().unwrap_or_default(),
        if delivery["exists"].as_bool() == Some(true) { "exists" } else { "to write" }
    ));
    lines.push(String::new());
    lines.push("Thin-layer rules:".to_string());
    for rule in THIN_LAYER_RULES {
        lines.push(format!("- {rule}"));
    }
    lines.push(String::new());
    lines.push(promote_text(&value["proposal"]));
    lines.join("\n")
}

pub(crate) fn run_consolidate(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["feature"]) {
        return None;
    }
    let feature = match flags.get("feature") {
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => return None,
    };
    let ctx = match g_prelude("knowledge consolidate", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let store_root = control_root_of(&ctx.root);
    match build_consolidation(&store_root, &ctx.root, &feature) {
        Err(msg) => Some(ctx.fail(&msg)),
        Ok(value) => {
            let text = consolidation_text(&value);
            Some(ctx.emit(&value, &text, 0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn fixture(root: &Path) {
        put(
            root,
            "docs/knowledge/areas/demo/rules.md",
            "---\ntype: bee.area\ntitle: \"Demo rules\"\nbee:\n  id: demo-rules\n  areas: [demo]\n---\n\n# Demo rules\n",
        );
        put(root, "docs/history/demo/CONTEXT.md", "# Demo — Context\n\nThe demo feature.\n");
        put(
            root,
            ".bee/cells/d-1.json",
            &json!({
                "id": "d-1",
                "feature": "demo",
                "title": "Change the demo",
                "status": "capped",
                "trace": {
                    "behavior_change": true,
                    "capped_at": "2026-11-09T00:00:00.000Z",
                    "files_changed": ["src/demo.rs", "docs/knowledge/areas/demo/rules.md"],
                    "deviations": ["the demo needed a second pass"]
                }
            })
            .to_string(),
        );
        put(
            root,
            ".bee/cells/archive/demo/d-2.json",
            &json!({
                "id": "d-2",
                "feature": "demo",
                "title": "Archive the demo",
                "status": "capped",
                "trace": {"files_changed": ["./docs/knowledge/areas/demo/rules.md", "docs/knowledge/patterns/p.md"]}
            })
            .to_string(),
        );
    }

    #[test]
    fn knowledge_consolidate_lists_the_features_knowledge_files_from_live_and_archived_cells() {
        let tmp = tempfile::tempdir().unwrap();
        fixture(tmp.path());
        let value = build_consolidation(tmp.path(), tmp.path(), "demo").unwrap();
        assert_eq!(
            value["knowledge_files"],
            json!(["docs/knowledge/areas/demo/rules.md", "docs/knowledge/patterns/p.md"])
        );
        assert_eq!(value["writes"], json!([]));
        assert_eq!(value["delivery"], json!({"path": "docs/history/demo/delivery.md", "exists": false}));
        assert_eq!(value["rules"].as_array().unwrap().len(), 3);
        assert!(value["proposal"]["pattern_candidates"].as_array().is_some_and(|p| !p.is_empty()), "{value}");
    }

    #[test]
    fn knowledge_consolidate_writes_nothing_and_names_an_existing_delivery() {
        let tmp = tempfile::tempdir().unwrap();
        fixture(tmp.path());
        put(tmp.path(), "docs/history/demo/delivery.md", "# Delivery\n");
        let before: Vec<_> = walk(tmp.path());
        let value = build_consolidation(tmp.path(), tmp.path(), "demo").unwrap();
        assert_eq!(value["delivery"]["exists"], json!(true));
        assert_eq!(walk(tmp.path()), before);
        let text = consolidation_text(&value);
        assert!(text.contains("Delivery record: docs/history/demo/delivery.md (exists)"), "{text}");
        assert!(text.contains("- docs/knowledge/areas/demo/rules.md"), "{text}");
        assert!(text.contains(THIN_LAYER_RULES[0]), "{text}");
    }

    #[test]
    fn knowledge_consolidate_refuses_an_unknown_feature_with_its_own_name() {
        let tmp = tempfile::tempdir().unwrap();
        fixture(tmp.path());
        let err = build_consolidation(tmp.path(), tmp.path(), "nowhere").unwrap_err();
        assert!(err.starts_with("knowledge consolidate:"), "{err}");
        assert!(err.contains("unknown_work"), "{err}");
    }

    fn walk(root: &Path) -> Vec<String> {
        let mut out = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    out.push(path.strip_prefix(root).unwrap().to_string_lossy().into_owned());
                }
            }
        }
        out.sort();
        out
    }
}
