// the index computation and its drift check
//
// Split out of the single 4.4k-line verbs/knowledge.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── index (computeIndexFiles / knowledgeIndexDrift / renderKnowledgeIndexes)

pub(crate) const KNOWLEDGE_INDEX_HEADER: &str = "<!--\nGENERATED FILE — do not hand-edit.\nRendered by `bee knowledge index` from concept frontmatter inside docs/knowledge/ (okf-foundation D21).\nRegenerate: `bee knowledge index`. Check freshness: `bee knowledge index --check`.\nDeterministic: byte-identical for the same bundle contents — path-sorted entries, LF endings,\nnever a generation timestamp or any other wall-clock value.\n-->";

pub(crate) fn concept_entry_line(concept: &Concept, from_dir: &str) -> String {
    let target = if from_dir.is_empty() {
        concept.path.clone()
    } else {
        concept.path[from_dir.len() + 1..].to_string()
    };
    let base = concept.path.rsplit('/').next().unwrap_or(&concept.path);
    let title = str_field(&concept.data, "title").unwrap_or(base);
    match str_field(&concept.data, "description") {
        Some(desc) => format!("- [{title}]({target}) — {desc}"),
        None => format!("- [{title}]({target})"),
    }
}

/// computeIndexFiles(root) -> [(rel, content)] path-sorted. None => delegate.
pub(crate) fn compute_index_files(dir: &Path) -> Option<Vec<(String, String)>> {
    let concepts = collect_concepts(dir)?;

    let mut index_dirs: Vec<String> = vec![String::new()];
    for concept in &concepts {
        let segments: Vec<&str> = concept.path.split('/').collect();
        for i in 1..segments.len() {
            let d = segments[..i].join("/");
            if !index_dirs.contains(&d) {
                index_dirs.push(d);
            }
        }
    }
    let mut sorted_dirs = index_dirs.clone();
    sorted_dirs.sort();

    let mut files = Vec::new();
    for dir_rel in &sorted_dirs {
        let direct: Vec<&Concept> = concepts
            .iter()
            .filter(|c| dir_of(&c.path) == dir_rel.as_str())
            .collect();
        let child_dirs: Vec<&String> = {
            let mut v: Vec<&String> = index_dirs
                .iter()
                .filter(|d| {
                    !d.is_empty()
                        && if dir_rel.is_empty() {
                            !d.contains('/')
                        } else {
                            d.starts_with(&format!("{dir_rel}/")) && !d[dir_rel.len() + 1..].contains('/')
                        }
                })
                .collect();
            v.sort();
            v
        };

        let mut sections: Vec<String> = Vec::new();
        if !direct.is_empty() {
            let mut lines = vec!["## Concepts".to_string(), String::new()];
            lines.extend(direct.iter().map(|c| concept_entry_line(c, dir_rel)));
            sections.push(lines.join("\n"));
        }
        if !child_dirs.is_empty() {
            let mut lines = vec!["## Sections".to_string(), String::new()];
            for child in &child_dirs {
                let name = if dir_rel.is_empty() { child.as_str() } else { &child[dir_rel.len() + 1..] };
                let count = concepts.iter().filter(|c| c.path.starts_with(&format!("{child}/"))).count();
                lines.push(format!("- [{name}/]({name}/index.md) — {count} concept(s)"));
            }
            sections.push(lines.join("\n"));
        }
        if dir_rel.is_empty() {
            let critical: Vec<&Concept> = concepts
                .iter()
                .filter(|c| matches!(bee_of(&c.data).get("critical"), Some(Value::Bool(true))))
                .collect();
            let mut lines = vec!["## Critical patterns".to_string(), String::new()];
            if critical.is_empty() {
                lines.push("None.".to_string());
            } else {
                lines.extend(critical.iter().map(|c| concept_entry_line(c, "")));
            }
            sections.push(lines.join("\n"));
        }

        let heading = if dir_rel.is_empty() { "# Knowledge Bundle".to_string() } else { format!("# {dir_rel}/") };
        let mut body_parts = vec![heading];
        body_parts.extend(sections);
        let body = body_parts.join("\n\n");
        let frontmatter = if dir_rel.is_empty() {
            let mut fm = Map::new();
            fm.insert("okf_version".to_string(), Value::String(OKF_VERSION.to_string()));
            emit_frontmatter(&fm).ok()?
        } else {
            String::new()
        };
        let rel = if dir_rel.is_empty() { "index.md".to_string() } else { format!("{dir_rel}/index.md") };
        files.push((rel, format!("{frontmatter}{KNOWLEDGE_INDEX_HEADER}\n\n{body}\n")));
    }
    Some(files)
}
