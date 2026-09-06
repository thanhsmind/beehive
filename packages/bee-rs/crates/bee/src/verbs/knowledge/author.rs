// bee knowledge show and bee knowledge new — authoring and inspection verbs
// for the docs/knowledge/ OKF v0.1 bundle.
//
// Cell: ksn-1
//
// `show`: reads one concept by its bee.id, printing metadata, body,
// outbound links (relative .md targets in the body plus bee.required_context),
// and inbound referrers (other concepts whose body or required_context names
// this path). Unknown id is ctx.fail.
//
// `new`: writes a canonical concept file (pattern or area) and regenerates
// the bundle indexes via compute_index_files. Refuses if the path exists or
// the id is already claimed.

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

// ─── link extraction & path helpers ────────────────────────────────────────

/// Extracts relative `.md` link targets from a markdown text body (`[label](target)`).
pub(crate) fn extract_markdown_links(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some(open) = text[cursor..].find("](") {
        let target_start = cursor + open + 2;
        if let Some(close) = text[target_start..].find(')') {
            let target_raw = &text[target_start..target_start + close];
            let target = target_raw.split_whitespace().next().unwrap_or("");
            let path_part = target.split('#').next().unwrap_or(target);
            if path_part.ends_with(".md")
                && !path_part.starts_with('/')
                && !path_part.starts_with('\\')
                && !path_part.contains(':')
            {
                let cleaned = path_part.to_string();
                if !out.contains(&cleaned) {
                    out.push(cleaned);
                }
            }
            cursor = target_start + close + 1;
        } else {
            break;
        }
    }
    out
}

/// Normalizes a relative path with `/` separators, collapsing `.` and `..`.
pub(crate) fn normalize_rel_path(path: &str) -> String {
    let mut stack: Vec<&str> = Vec::new();
    for part in path.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            stack.pop();
        } else {
            stack.push(part);
        }
    }
    stack.join("/")
}

/// Resolves a link target relative to a concept's parent directory inside docs/knowledge/.
pub(crate) fn resolve_relative_target(base_dir: &str, link: &str) -> String {
    if base_dir.is_empty() {
        normalize_rel_path(link)
    } else {
        normalize_rel_path(&format!("{base_dir}/{link}"))
    }
}

// ─── show logic ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) struct ShowResult {
    pub(crate) path: String,
    pub(crate) data: Map<String, Value>,
    pub(crate) body: String,
    pub(crate) links_out: Vec<String>,
    pub(crate) links_in: Vec<String>,
    pub(crate) lines: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum ShowError {
    NotFound(String),
    BundleError,
}

pub(crate) fn show_concept(dir: &Path, id: &str) -> Result<ShowResult, ShowError> {
    let concepts = collect_concepts(dir).ok_or(ShowError::BundleError)?;
    let target = concepts
        .iter()
        .find(|c| bee_of(&c.data).get("id").and_then(Value::as_str) == Some(id))
        .ok_or_else(|| ShowError::NotFound(id.to_string()))?;

    let body = concept_body(dir, &target.path).unwrap_or_default();
    let mut links_out = extract_markdown_links(&body);
    let bee = bee_of(&target.data);
    if let Some(Value::Array(reqs)) = bee.get("required_context") {
        for r in reqs {
            if let Some(s) = r.as_str() {
                let s_str = s.to_string();
                if !links_out.contains(&s_str) {
                    links_out.push(s_str);
                }
            }
        }
    }

    let mut links_in: Vec<String> = Vec::new();
    for other in &concepts {
        if other.path == target.path {
            continue;
        }
        let other_bee = bee_of(&other.data);
        let mut matched = false;

        // 1. Check required_context of other
        if let Some(Value::Array(reqs)) = other_bee.get("required_context") {
            for r in reqs {
                if let Some(s) = r.as_str() {
                    let s_clean = s.strip_prefix("docs/knowledge/").unwrap_or(s);
                    if s_clean == target.path || normalize_rel_path(s_clean) == target.path {
                        matched = true;
                        break;
                    }
                }
            }
        }

        // 2. Check body of other
        if !matched {
            let other_body = concept_body(dir, &other.path).unwrap_or_default();
            if other_body.contains(&target.path)
                || other_body.contains(&format!("docs/knowledge/{}", target.path))
            {
                matched = true;
            } else {
                let other_dir = dir_of(&other.path);
                let targets = extract_markdown_links(&other_body);
                for t in targets {
                    let resolved = resolve_relative_target(other_dir, &t);
                    if resolved == target.path {
                        matched = true;
                        break;
                    }
                }
            }
        }

        if matched {
            links_in.push(other.path.clone());
        }
    }
    links_in.sort();
    links_in.dedup();

    let display_path = format!("docs/knowledge/{}", target.path);
    let mut lines = Vec::new();
    lines.push(format!("path: {display_path}"));
    lines.push(format!("type: {}", str_field(&target.data, "type").unwrap_or("-")));
    lines.push(format!("title: {}", str_field(&target.data, "title").unwrap_or("-")));
    lines.push(format!(
        "description: {}",
        str_field(&target.data, "description").unwrap_or("-")
    ));
    if let Some(Value::Array(tags)) = target.data.get("tags") {
        let tag_strs: Vec<&str> = tags.iter().filter_map(Value::as_str).collect();
        lines.push(format!("tags: [{}]", tag_strs.join(", ")));
    } else {
        lines.push("tags: -".to_string());
    }
    lines.push(format!(
        "timestamp: {}",
        str_field(&target.data, "timestamp").unwrap_or("-")
    ));

    // every bee.* field
    let mut bee_keys: Vec<&String> = BEE_KEY_ORDER
        .iter()
        .filter_map(|k| bee.keys().find(|key| key.as_str() == *k))
        .collect();
    let mut unknown_bee: Vec<&String> = bee
        .keys()
        .filter(|k| !BEE_KEY_ORDER.contains(&k.as_str()))
        .collect();
    unknown_bee.sort();
    bee_keys.extend(unknown_bee);

    for key in bee_keys {
        if let Some(val) = bee.get(key) {
            let val_str = match val {
                Value::String(s) => s.clone(),
                Value::Array(arr) => {
                    let items: Vec<String> = arr
                        .iter()
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect();
                    format!("[{}]", items.join(", "))
                }
                other => other.to_string(),
            };
            lines.push(format!("bee.{key}: {val_str}"));
        }
    }

    if links_out.is_empty() {
        lines.push("links_out: (none)".to_string());
    } else {
        lines.push("links_out:".to_string());
        for l in &links_out {
            lines.push(format!("  - {l}"));
        }
    }

    if links_in.is_empty() {
        lines.push("links_in: (none)".to_string());
    } else {
        lines.push("links_in:".to_string());
        for l in &links_in {
            lines.push(format!("  - {l}"));
        }
    }

    if !body.is_empty() {
        lines.push(String::new());
        lines.push(body.clone());
    }

    Ok(ShowResult {
        path: display_path,
        data: target.data.clone(),
        body,
        links_out,
        links_in,
        lines,
    })
}

// ─── new logic ─────────────────────────────────────────────────────────────

pub(crate) struct NewArgs<'a> {
    pub(crate) c_type: &'a str,
    pub(crate) title: &'a str,
    pub(crate) summary: &'a str,
    pub(crate) area: &'a str,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) lifecycle: Option<&'a str>,
    pub(crate) body: Option<String>,
    pub(crate) today_iso: &'a str,
    pub(crate) today_nodash: &'a str,
}

#[derive(Debug)]
pub(crate) struct NewResult {
    pub(crate) path: String,
    pub(crate) id: String,
    pub(crate) written: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum NewError {
    EmptySlug,
    InvalidType(String),
    PathExists(String),
    IdClaimed { id: String, claimed_by: String },
    WriteError(String),
}

pub(crate) fn new_concept(dir: &Path, args: &NewArgs) -> Result<NewResult, NewError> {
    let slug = slug_from_stem(args.title);
    if slug.is_empty() {
        return Err(NewError::EmptySlug);
    }

    let (type_name, rel_path, id) = match args.c_type {
        "pattern" => {
            let id = format!("pattern-{}-{slug}", args.today_nodash);
            let rel_path = format!("patterns/{}-{slug}.md", args.today_nodash);
            ("bee.pattern", rel_path, id)
        }
        "area" => {
            let id = format!("{}-{slug}", args.area);
            let rel_path = format!("areas/{}/{slug}.md", args.area);
            ("bee.area", rel_path, id)
        }
        other => return Err(NewError::InvalidType(other.to_string())),
    };

    let abs_path = join_rel(dir, &rel_path);
    if abs_path.exists() {
        return Err(NewError::PathExists(format!("docs/knowledge/{rel_path}")));
    }

    let concepts = collect_concepts(dir).unwrap_or_default();
    if let Some(existing) = concepts
        .iter()
        .find(|c| bee_of(&c.data).get("id").and_then(Value::as_str) == Some(&id))
    {
        return Err(NewError::IdClaimed {
            id,
            claimed_by: format!("docs/knowledge/{}", existing.path),
        });
    }

    let lifecycle = args.lifecycle.unwrap_or("active");

    let mut bee = Map::new();
    bee.insert("id".to_string(), Value::String(id.clone()));
    bee.insert("lifecycle".to_string(), Value::String(lifecycle.to_string()));
    bee.insert(
        "areas".to_string(),
        Value::Array(vec![Value::String(args.area.to_string())]),
    );

    let mut data = Map::new();
    data.insert("type".to_string(), Value::String(type_name.to_string()));
    data.insert("title".to_string(), Value::String(args.title.to_string()));
    data.insert(
        "description".to_string(),
        Value::String(args.summary.to_string()),
    );
    if let Some(ref tags) = args.tags {
        data.insert(
            "tags".to_string(),
            Value::Array(tags.iter().map(|t| Value::String(t.clone())).collect()),
        );
    }
    data.insert(
        "timestamp".to_string(),
        Value::String(args.today_iso.to_string()),
    );
    data.insert("bee".to_string(), Value::Object(bee));

    let frontmatter = emit_frontmatter(&data)
        .map_err(|()| NewError::WriteError("cannot emit frontmatter".into()))?;

    let raw_body = match &args.body {
        Some(b) => b.clone(),
        None => format!("# {}\n", args.title),
    };
    let clean_body = raw_body.trim_start_matches(['\r', '\n']);
    let full_content = if clean_body.is_empty() {
        frontmatter
    } else if clean_body.ends_with('\n') {
        format!("{frontmatter}\n{clean_body}")
    } else {
        format!("{frontmatter}\n{clean_body}\n")
    };

    if let Some(parent) = abs_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| NewError::WriteError(e.to_string()))?;
    }
    std::fs::write(&abs_path, &full_content).map_err(|e| NewError::WriteError(e.to_string()))?;

    let mut written = Vec::new();
    if let Some(expected) = compute_index_files(dir) {
        for (rel, content) in &expected {
            let abs = join_rel(dir, rel);
            if let Some(parent) = abs.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::write(&abs, content).map_err(|e| NewError::WriteError(e.to_string()))?;
            written.push(format!("docs/knowledge/{rel}"));
        }
    }

    Ok(NewResult {
        path: format!("docs/knowledge/{rel_path}"),
        id,
        written,
    })
}

// ─── routing handlers ──────────────────────────────────────────────────────

pub(crate) fn run_show(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["id"]) {
        return None;
    }
    let id = match flags.get("id") {
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => js_trim(s).to_string(),
        _ => return None,
    };

    let ctx = match g_prelude("knowledge show", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;

    match show_concept(&dir, &id) {
        Ok(res) => {
            let mut result = Map::new();
            result.insert("path".into(), Value::String(res.path));
            result.insert("data".into(), Value::Object(res.data));
            result.insert("body".into(), Value::String(res.body));
            result.insert(
                "links_out".into(),
                Value::Array(res.links_out.into_iter().map(Value::String).collect()),
            );
            result.insert(
                "links_in".into(),
                Value::Array(res.links_in.into_iter().map(Value::String).collect()),
            );
            Some(ctx.emit(&Value::Object(result), &res.lines.join("\n"), 0))
        }
        Err(ShowError::NotFound(id)) => Some(ctx.fail(&format!("concept not found: {id}"))),
        Err(ShowError::BundleError) => None,
    }
}

pub(crate) fn run_new(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["type", "title", "summary", "area", "tags", "lifecycle", "file"]) {
        return None;
    }
    let c_type = match flags.get("type") {
        Some(FlagV::S(s)) if s == "pattern" || s == "area" => s.as_str(),
        _ => return None,
    };
    let title = match flags.get("title") {
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => js_trim(s),
        _ => return None,
    };
    let summary = match flags.get("summary") {
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => js_trim(s),
        _ => return None,
    };
    let area = match flags.get("area") {
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => js_trim(s),
        _ => return None,
    };
    let lifecycle = match flags.get("lifecycle") {
        Some(FlagV::S(s)) if !js_trim(s).is_empty() => Some(js_trim(s)),
        _ => None,
    };
    let tags = match flags.get("tags") {
        Some(FlagV::S(s)) => {
            let list: Vec<String> = s
                .split(',')
                .map(js_trim)
                .filter(|t| !t.is_empty())
                .map(str::to_string)
                .collect();
            Some(list)
        }
        _ => None,
    };

    let ctx = match g_prelude("knowledge new", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;

    let body = match flags.get("file") {
        Some(FlagV::S(file_path)) => {
            let abs_file = ctx.root.join(file_path);
            let raw = match read_file_lossy(&abs_file) {
                Ok(content) => content,
                Err(_) => match read_file_lossy(Path::new(file_path)) {
                    Ok(content) => content,
                    Err(e) => return Some(ctx.fail(&format!("could not read file \"{file_path}\": {e}"))),
                },
            };
            Some(strip_leading_frontmatter(&raw).to_string())
        }
        _ => None,
    };

    let today = chrono::Utc::now();
    let today_iso = today.format("%Y-%m-%d").to_string();
    let today_nodash = today.format("%Y%m%d").to_string();

    let args = NewArgs {
        c_type,
        title,
        summary,
        area,
        tags,
        lifecycle,
        body,
        today_iso: &today_iso,
        today_nodash: &today_nodash,
    };

    match new_concept(&dir, &args) {
        Ok(res) => {
            let mut result = Map::new();
            result.insert("path".into(), Value::String(res.path.clone()));
            result.insert("id".into(), Value::String(res.id.clone()));
            result.insert(
                "written".into(),
                Value::Array(res.written.iter().map(|s| Value::String(s.clone())).collect()),
            );
            let text = format!(
                "Created {} ({})\nRendered {} index file(s).",
                res.path,
                res.id,
                res.written.len()
            );
            Some(ctx.emit(&Value::Object(result), &text, 0))
        }
        Err(NewError::PathExists(p)) => Some(ctx.fail(&format!("concept file already exists: {p}"))),
        Err(NewError::IdClaimed { id, claimed_by }) => {
            Some(ctx.fail(&format!("concept id \"{id}\" is already claimed by {claimed_by}")))
        }
        Err(NewError::EmptySlug) => {
            Some(ctx.fail("title carries no letters or digits to slug a concept from"))
        }
        Err(NewError::InvalidType(t)) => Some(ctx.fail(&format!("unsupported concept type \"{t}\""))),
        Err(NewError::WriteError(e)) => Some(ctx.fail(&format!("write error: {e}"))),
    }
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        (tmp, dir)
    }

    fn write_concept(dir: &Path, rel: &str, text: &str) {
        let abs = join_rel(dir, rel);
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(abs, text).unwrap();
    }

    #[test]
    fn show_finds_concept_and_emits_fields_and_links() {
        let (_tmp, dir) = bundle();
        write_concept(
            &dir,
            "areas/auth/login.md",
            "---\ntype: bee.area\ntitle: Auth Login\ndescription: Login flow\ntags: [auth, security]\ntimestamp: 2026-08-01\nbee:\n  id: auth-login\n  lifecycle: active\n  areas: [auth]\n  required_context: [patterns/20260101-token-leak.md]\n---\n\n# Auth Login\n\nSee [details](mfa.md).\n",
        );
        write_concept(
            &dir,
            "areas/auth/mfa.md",
            "---\ntype: bee.area\ntitle: Auth MFA\ndescription: MFA flow\ntimestamp: 2026-08-02\nbee:\n  id: auth-mfa\n  lifecycle: active\n  areas: [auth]\n---\n\n# Auth MFA\n\nBack to [login](login.md).\n",
        );
        write_concept(
            &dir,
            "patterns/20260101-token-leak.md",
            "---\ntype: bee.pattern\ntitle: Token Leak\ndescription: Token leak pattern\ntimestamp: 2026-01-01\nbee:\n  id: pattern-20260101-token-leak\n  lifecycle: active\n  required_context: [areas/auth/login.md]\n---\n\n# Token Leak\n\nReferenced by login.\n",
        );

        let res = show_concept(&dir, "auth-login").expect("show auth-login");
        assert_eq!(res.path, "docs/knowledge/areas/auth/login.md");
        assert_eq!(str_field(&res.data, "title"), Some("Auth Login"));
        assert_eq!(str_field(&res.data, "description"), Some("Login flow"));
        assert!(res.body.contains("See [details](mfa.md)."));

        // Outbound links: relative .md targets in body plus bee.required_context
        assert!(res.links_out.contains(&"mfa.md".to_string()));
        assert!(res.links_out.contains(&"patterns/20260101-token-leak.md".to_string()));

        // Inbound referrers: mfa.md links to login.md, pattern names areas/auth/login.md in required_context
        assert!(res.links_in.contains(&"areas/auth/mfa.md".to_string()));
        assert!(res.links_in.contains(&"patterns/20260101-token-leak.md".to_string()));
    }

    #[test]
    fn show_not_found_returns_error() {
        let (_tmp, dir) = bundle();
        let err = show_concept(&dir, "non-existent-id").unwrap_err();
        assert!(matches!(err, ShowError::NotFound(id) if id == "non-existent-id"));
    }

    #[test]
    fn new_writes_canonical_pattern_and_rerenders_indexes() {
        let (_tmp, dir) = bundle();
        let args = NewArgs {
            c_type: "pattern",
            title: "Cache Stampede",
            summary: "Avoid concurrent cache misses with single-flight locks",
            area: "performance",
            tags: Some(vec!["cache".into(), "concurrency".into()]),
            lifecycle: None,
            body: None,
            today_iso: "2026-09-06",
            today_nodash: "20260906",
        };

        let res = new_concept(&dir, &args).expect("new pattern");
        assert_eq!(res.id, "pattern-20260906-cache-stampede");
        assert_eq!(res.path, "docs/knowledge/patterns/20260906-cache-stampede.md");

        let file_path = join_rel(&dir, "patterns/20260906-cache-stampede.md");
        assert!(file_path.exists());

        // Check canonical form via check_bundle
        let report = check_bundle(&dir, true).expect("check_bundle");
        assert!(report.ok, "check_bundle must pass: {:?}", report.warnings);
        assert!(report.okf_errors.is_empty(), "OKF errors: {:?}", report.okf_errors);
        assert!(report.profile_errors.is_empty(), "Profile errors: {:?}", report.profile_errors);
        assert!(report.warnings.is_empty(), "Warnings (incl not_canonical): {:?}", report.warnings);

        // Indexes must be re-rendered
        assert!(dir.join("index.md").exists());
        assert!(dir.join("patterns").join("index.md").exists());
        assert!(res.written.contains(&"docs/knowledge/index.md".to_string()));
        assert!(res.written.contains(&"docs/knowledge/patterns/index.md".to_string()));
    }

    #[test]
    fn new_writes_canonical_area_concept() {
        let (_tmp, dir) = bundle();
        let args = NewArgs {
            c_type: "area",
            title: "Auth Gateway",
            summary: "Centralized entrypoint for auth requests",
            area: "auth",
            tags: None,
            lifecycle: Some("draft"),
            body: Some("# Auth Gateway\n\nDetailed gateway design.\n".into()),
            today_iso: "2026-09-06",
            today_nodash: "20260906",
        };

        let res = new_concept(&dir, &args).expect("new area");
        assert_eq!(res.id, "auth-auth-gateway");
        assert_eq!(res.path, "docs/knowledge/areas/auth/auth-gateway.md");

        let file_path = join_rel(&dir, "areas/auth/auth-gateway.md");
        assert!(file_path.exists());
        let text = read_file_lossy(&file_path).unwrap();
        assert!(text.contains("Detailed gateway design."));

        let report = check_bundle(&dir, true).expect("check_bundle");
        assert!(report.ok, "check_bundle must pass: {:?}", report.warnings);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn new_refuses_duplicate_path_and_duplicate_id() {
        let (_tmp, dir) = bundle();
        let args = NewArgs {
            c_type: "area",
            title: "Audit Trail",
            summary: "Audit logging",
            area: "compliance",
            tags: None,
            lifecycle: None,
            body: None,
            today_iso: "2026-09-06",
            today_nodash: "20260906",
        };

        let res = new_concept(&dir, &args).expect("first write succeeds");
        assert_eq!(res.id, "compliance-audit-trail");

        // 1. Same arguments -> path exists refusal
        let err1 = new_concept(&dir, &args).unwrap_err();
        assert!(matches!(err1, NewError::PathExists(_)));

        // 2. Different title that generates the same slug -> path exists
        let args_same_slug = NewArgs {
            c_type: "area",
            title: "Audit-Trail",
            summary: "Audit logging again",
            area: "compliance",
            tags: None,
            lifecycle: None,
            body: None,
            today_iso: "2026-09-06",
            today_nodash: "20260906",
        };
        let err2 = new_concept(&dir, &args_same_slug).unwrap_err();
        assert!(matches!(err2, NewError::PathExists(_)));

        // 3. Pre-existing concept claiming the id at a different path -> id claimed refusal
        write_concept(
            &dir,
            "areas/other/diff.md",
            "---\ntype: bee.area\ntitle: Other\ndescription: Other\ntimestamp: 2026-09-06\nbee:\n  id: compliance-audit-two\n  lifecycle: active\n---\n",
        );
        let args_id_conflict = NewArgs {
            c_type: "area",
            title: "Audit Two",
            summary: "Collision on id",
            area: "compliance",
            tags: None,
            lifecycle: None,
            body: None,
            today_iso: "2026-09-06",
            today_nodash: "20260906",
        };
        let err3 = new_concept(&dir, &args_id_conflict).unwrap_err();
        assert!(matches!(err3, NewError::IdClaimed { ref id, .. } if id == "compliance-audit-two"));
    }
}
