#![allow(unused_imports)]

use super::*;
use crate::verbs::reservations::{js_trim, keys_known, FlagV, Flags};
use crate::verbs::worktree::run_git;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

pub(crate) const CONFIRMED: &str = "confirmed";
pub(crate) const SUSPECT: &str = "suspect";
pub(crate) const ORPHAN: &str = "orphan";
pub(crate) const UNVERIFIED: &str = "unverified";

pub(crate) struct Grade {
    pub(crate) grade: &'static str,
    pub(crate) verified_at: Option<String>,
    pub(crate) verified_commit: Option<String>,
    pub(crate) anchors: Vec<String>,
    pub(crate) changed: Vec<String>,
    pub(crate) reason: Option<String>,
}

pub(crate) fn bundle_root(root: &Path, feature: Option<&str>) -> PathBuf {
    crate::verbs::drivers::docs_root_for_feature(root, feature.unwrap_or(""))
        .join("docs")
        .join("knowledge")
}

pub(crate) fn command_bundle(root: &Path) -> Option<PathBuf> {
    bundle_dir(root)?;
    Some(bundle_root(root, None))
}

pub(crate) fn checkout_of(dir: &Path) -> Option<&Path> {
    dir.parent()?.parent()
}

fn string_list(bee: &Map<String, Value>, key: &str) -> Vec<String> {
    match bee.get(key) {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(|s| js_trim(s).to_string()))
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn norm_anchor(anchor: &str) -> &str {
    anchor.trim_start_matches("./").trim_end_matches('/')
}

fn is_repo_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.split('/').any(|seg| seg == "..")
        && !path.contains(':')
}

fn is_sha(s: &str) -> bool {
    (7..=64).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(7)]
}

fn base_grade(bee: &Map<String, Value>) -> Grade {
    Grade {
        grade: UNVERIFIED,
        verified_at: str_field(bee, "verified_at").map(str::to_string),
        verified_commit: str_field(bee, "verified_commit").map(str::to_string),
        anchors: string_list(bee, "anchors"),
        changed: Vec::new(),
        reason: None,
    }
}

fn ancestry_reason(vc: &str, status: Option<i32>, stderr: &str) -> Option<String> {
    match status {
        Some(0) => None,
        Some(1) => Some(format!(
            "verified_commit {} is not an ancestor of HEAD (rewritten by squash or rebase, or from another branch)",
            short(vc)
        )),
        None => Some("git is unavailable".to_string()),
        Some(_) if stderr.contains("not a git repository") => Some("the checkout is not a git repository".to_string()),
        Some(_) => Some(format!("verified_commit {} is unknown to git", short(vc))),
    }
}

fn touched_commits(stdout: &str) -> Vec<(String, HashSet<String>)> {
    let mut commits: Vec<(String, HashSet<String>)> = Vec::new();
    for line in stdout.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let hash_line = matches!(line.len(), 40 | 64) && line.bytes().all(|b| b.is_ascii_hexdigit());
        if hash_line {
            commits.push((line.to_string(), HashSet::new()));
        } else if let Some(last) = commits.last_mut() {
            last.1.insert(line.to_string());
        }
    }
    commits
}

fn anchor_touched(files: &HashSet<String>, anchor: &str, is_dir: bool) -> bool {
    files.contains(anchor) || (is_dir && files.iter().any(|f| f.strip_prefix(anchor).is_some_and(|rest| rest.starts_with('/'))))
}

pub(crate) fn grade_bundle(checkout: &Path, concepts: &[(&str, &Map<String, Value>)]) -> BTreeMap<String, Grade> {
    let mut out: BTreeMap<String, Grade> = BTreeMap::new();
    let mut candidates: Vec<&str> = Vec::new();
    for (rel, bee) in concepts {
        let mut grade = base_grade(bee);
        let missing: Vec<&String> = grade
            .anchors
            .iter()
            .filter(|a| !is_repo_relative(norm_anchor(a)) || !checkout.join(norm_anchor(a)).exists())
            .collect();
        if !missing.is_empty() {
            grade.grade = ORPHAN;
            grade.reason = Some(format!(
                "anchor path missing on disk: {}",
                missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            ));
        } else if grade.anchors.is_empty() {
            grade.reason = Some("no bee.anchors".to_string());
        } else if grade.verified_commit.is_none() {
            grade.reason = Some("no verified_commit".to_string());
        } else if !grade.verified_commit.as_deref().is_some_and(is_sha) {
            grade.reason = Some(format!(
                "verified_commit {} is not a commit sha",
                grade.verified_commit.as_deref().unwrap_or("")
            ));
        } else {
            candidates.push(rel);
        }
        out.insert((*rel).to_string(), grade);
    }
    if candidates.is_empty() {
        return out;
    }

    let mut distinct: Vec<String> = Vec::new();
    for rel in &candidates {
        let vc = out[*rel].verified_commit.clone().unwrap_or_default();
        if !distinct.contains(&vc) {
            distinct.push(vc);
        }
    }
    let mut ancestry: HashMap<String, Option<String>> = HashMap::new();
    for vc in &distinct {
        let result = run_git(checkout, &["merge-base", "--is-ancestor", vc, "HEAD"]);
        ancestry.insert(vc.clone(), ancestry_reason(vc, result.status, result.stderr.as_deref().unwrap_or("")));
    }
    let mut live: Vec<&String> = distinct.iter().filter(|vc| ancestry[*vc].is_none()).collect();
    live.sort();

    let mut window_reason: Option<String> = None;
    let mut commits: Vec<(String, HashSet<String>)> = Vec::new();
    if !live.is_empty() {
        let base = if live.len() == 1 {
            Some(live[0].clone())
        } else {
            let mut args: Vec<&str> = vec!["merge-base", "--octopus"];
            args.extend(live.iter().map(|s| s.as_str()));
            let result = run_git(checkout, &args);
            let text = result.stdout.as_deref().unwrap_or("").trim().to_string();
            (result.status == Some(0) && is_sha(&text)).then_some(text)
        };
        match base {
            None => window_reason = Some("git could not find a common base for the verified commits".to_string()),
            Some(base) => {
                let range = format!("{base}..HEAD");
                let result = run_git(
                    checkout,
                    &["-c", "core.quotepath=false", "log", "--format=%H", "--name-only", "--first-parent", "-m", &range],
                );
                if result.status == Some(0) {
                    commits = touched_commits(result.stdout.as_deref().unwrap_or(""));
                } else {
                    window_reason = Some("git log over the verified range failed".to_string());
                }
            }
        }
    }

    for rel in candidates {
        let Some(grade) = out.get_mut(rel) else { continue };
        let vc = grade.verified_commit.clone().unwrap_or_default();
        if let Some(reason) = ancestry.get(&vc).cloned().flatten() {
            grade.reason = Some(reason);
            continue;
        }
        if let Some(reason) = &window_reason {
            grade.reason = Some(reason.clone());
            continue;
        }
        let window_end = commits
            .iter()
            .position(|(hash, _)| *hash == vc || hash.starts_with(&vc))
            .unwrap_or(commits.len());
        let concept_path = format!("docs/knowledge/{rel}");
        let window = &commits[..window_end];
        let changed: Vec<String> = grade
            .anchors
            .iter()
            .filter(|anchor| {
                let norm = norm_anchor(anchor);
                let is_dir = checkout.join(norm).is_dir();
                window
                    .iter()
                    .any(|(_, files)| !files.contains(&concept_path) && anchor_touched(files, norm, is_dir))
            })
            .cloned()
            .collect();
        grade.grade = if changed.is_empty() { CONFIRMED } else { SUSPECT };
        grade.changed = changed;
    }
    out
}

pub(crate) fn grade_concept(checkout: &Path, rel: &str, bee: &Map<String, Value>) -> Grade {
    let mut graded = grade_bundle(checkout, &[(rel, bee)]);
    graded.remove(rel).unwrap_or_else(|| base_grade(bee))
}

pub(crate) fn grade_row(path: &str, grade: &Grade) -> Value {
    let opt = |v: &Option<String>| v.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null);
    json!({
        "path": path,
        "grade": grade.grade,
        "verified_at": opt(&grade.verified_at),
        "verified_commit": opt(&grade.verified_commit),
        "changed": grade.changed,
        "reason": opt(&grade.reason),
    })
}

pub(crate) fn freshness_counts(rows: &[(String, Grade)]) -> [usize; 4] {
    let count = |g: &str| rows.iter().filter(|(_, grade)| grade.grade == g).count();
    [count(CONFIRMED), count(SUSPECT), count(ORPHAN), count(UNVERIFIED)]
}

pub(crate) fn freshness_line(rows: &[(String, Grade)]) -> String {
    let [confirmed, suspect, orphan, unverified] = freshness_counts(rows);
    format!("freshness: {confirmed} confirmed, {suspect} suspect, {orphan} orphan, {unverified} unverified")
}

pub(crate) fn freshness_json(rows: &[(String, Grade)]) -> Value {
    let [confirmed, suspect, orphan, unverified] = freshness_counts(rows);
    json!({
        "confirmed": confirmed,
        "suspect": suspect,
        "orphan": orphan,
        "unverified": unverified,
        "rows": rows.iter().map(|(path, grade)| grade_row(path, grade)).collect::<Vec<_>>(),
    })
}

pub(crate) fn grade_areas(checkout: &Path, concepts: &[(&str, Map<String, Value>)]) -> Vec<(String, Grade)> {
    let refs: Vec<(&str, &Map<String, Value>)> = concepts
        .iter()
        .filter(|(rel, _)| rel.starts_with("areas/"))
        .map(|(rel, bee)| (*rel, bee))
        .collect();
    grade_bundle(checkout, &refs).into_iter().collect()
}

pub(crate) fn stale_anchor_message(grade: &Grade, id: &str) -> String {
    let since = grade.verified_commit.as_deref().map(short).unwrap_or("");
    format!(
        "anchor(s) {} changed after verified_commit {since} — if the concept still holds run `bee knowledge confirm --id {id}`, otherwise edit it",
        grade.changed.join(", ")
    )
}

pub(crate) fn orphan_anchor_message(grade: &Grade) -> String {
    format!(
        "{} — fix bee.anchors (a confirm does not clear an orphan)",
        grade.reason.as_deref().unwrap_or("an anchor path is missing")
    )
}

pub(crate) struct AreaConcept {
    pub(crate) rel: String,
    pub(crate) abs: PathBuf,
    pub(crate) data: Map<String, Value>,
    pub(crate) body: String,
}

pub(crate) fn load_areas(dir: &Path) -> Option<Vec<AreaConcept>> {
    let mut out = Vec::new();
    for rel in list_bundle_markdown(dir)? {
        if !rel.starts_with("areas/") || is_reserved_basename(rel.rsplit('/').next().unwrap_or(&rel)) {
            continue;
        }
        let abs = join_rel(dir, &rel);
        let Ok(text) = read_file_lossy(&abs) else { continue };
        if let Fm::Parsed { data, body, .. } = parse_frontmatter(&text) {
            out.push(AreaConcept { rel, abs, data, body });
        }
    }
    Some(out)
}

fn write_concept(concept: &AreaConcept) -> Result<(), String> {
    let block = emit_frontmatter(&concept.data)
        .map_err(|()| format!("{} does not re-emit canonically", concept.rel))?;
    std::fs::write(&concept.abs, format!("{block}{}", concept.body)).map_err(|e| format!("{}: {e}", concept.rel))
}

fn bee_mut(data: &mut Map<String, Value>) -> Option<&mut Map<String, Value>> {
    data.get_mut("bee").and_then(Value::as_object_mut)
}

fn body_path_candidates(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut fenced = false;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let mut parts = line.split('`');
        parts.next();
        while let Some(span) = parts.next() {
            if parts.next().is_none() {
                break;
            }
            let Some(candidate) = dangling_source_candidate(span) else { continue };
            let candidate = match candidate.rsplit_once(':') {
                Some((head, tail)) if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) => head.to_string(),
                _ => candidate,
            };
            if !out.contains(&candidate) {
                out.push(candidate);
            }
        }
    }
    out
}

fn seedable(checkout: &Path, candidate: &str) -> bool {
    let norm = candidate.trim_start_matches("./");
    is_repo_relative(norm)
        && !norm.starts_with(".bee/")
        && !norm.starts_with("docs/")
        && !format!("/{norm}").contains("/generated/")
        && checkout.join(norm).exists()
}

fn baseline_commit(checkout: &Path, rel: &str) -> Option<(String, String)> {
    let path = format!("docs/knowledge/{rel}");
    let result = run_git(checkout, &["log", "-1", "--first-parent", "--format=%H%n%cs", "--", &path]);
    if result.status != Some(0) {
        return None;
    }
    let stdout = result.stdout.unwrap_or_default();
    let mut lines = stdout.lines();
    let sha = lines.next()?.trim().to_string();
    let date = lines.next()?.trim().to_string();
    (is_sha(&sha) && !date.is_empty()).then_some((sha, date))
}

pub(crate) fn run_seed_anchors(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["dry-run"]) {
        return None;
    }
    let dry_run = js_bool_flag(&flags, "dry-run")?;
    let ctx = match g_prelude("knowledge seed-anchors", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = command_bundle(&ctx.root)?;
    let checkout = checkout_of(&dir)?.to_path_buf();
    let mut concepts = load_areas(&dir)?;

    let mut lines: Vec<String> = Vec::new();
    let mut written: Vec<Value> = Vec::new();
    let (mut zero, mut existing) = (0usize, 0usize);
    for concept in concepts.iter_mut() {
        let Some(bee) = bee_mut(&mut concept.data) else {
            zero += 1;
            continue;
        };
        if bee.contains_key("anchors") {
            existing += 1;
            continue;
        }
        let anchors: Vec<String> = body_path_candidates(&concept.body)
            .into_iter()
            .map(|c| c.trim_start_matches("./").to_string())
            .filter(|c| seedable(&checkout, c))
            .collect();
        if anchors.is_empty() {
            zero += 1;
            continue;
        }
        let count = anchors.len();
        bee.insert("anchors".into(), Value::Array(anchors.into_iter().map(Value::String).collect()));
        let mut baseline: Option<(String, String)> = None;
        if !bee.contains_key("verified_commit") && !bee.contains_key("verified_at") {
            baseline = baseline_commit(&checkout, &concept.rel);
            if let Some((sha, date)) = &baseline {
                bee.insert("verified_at".into(), Value::String(date.clone()));
                bee.insert("verified_commit".into(), Value::String(sha.clone()));
            }
        }
        if !dry_run {
            if let Err(message) = write_concept(concept) {
                return Some(ctx.fail(&message));
            }
        }
        lines.push(match &baseline {
            Some((sha, date)) => format!("{}: {count} anchor(s), baseline {} {date}", concept.rel, short(sha)),
            None => format!("{}: {count} anchor(s), no baseline commit", concept.rel),
        });
        written.push(json!({
            "path": concept.rel,
            "anchors": count,
            "verified_commit": baseline.as_ref().map(|(sha, _)| sha.clone()),
            "verified_at": baseline.as_ref().map(|(_, date)| date.clone()),
        }));
    }
    lines.push(format!(
        "seed-anchors{}: {} concept(s) {}, {zero} with zero anchors, {existing} already carry bee.anchors",
        if dry_run { " --dry-run" } else { "" },
        written.len(),
        if dry_run { "would be written" } else { "written" },
    ));
    let result = json!({
        "dry_run": dry_run,
        "written": written,
        "zero_anchors": zero,
        "already_anchored": existing,
    });
    Some(ctx.emit(&result, &lines.join("\n"), 0))
}

fn repeated_ids(toks: &[&str]) -> Vec<String> {
    let mut ids = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        if toks[i] == "--id" {
            if let Some(v) = toks.get(i + 1) {
                ids.push((*v).to_string());
            }
            i += 2;
            continue;
        }
        if let Some(v) = toks[i].strip_prefix("--id=") {
            ids.push(v.to_string());
        }
        i += 1;
    }
    ids
}

pub(crate) fn confirm_targets(concepts: &[AreaConcept], ids: &[String], path: Option<&str>) -> Result<Vec<usize>, String> {
    let mut targets: Vec<usize> = Vec::new();
    if let Some(path) = path {
        for (i, concept) in concepts.iter().enumerate() {
            if string_list(&bee_of(&concept.data), "anchors").iter().any(|a| norm_anchor(a) == path) {
                targets.push(i);
            }
        }
        if targets.is_empty() {
            return Err(format!("no concept anchors {path}"));
        }
        return Ok(targets);
    }
    for id in ids {
        let found = concepts.iter().position(|c| str_field(&bee_of(&c.data), "id") == Some(id.as_str()));
        let Some(i) = found else {
            return Err(format!("no concept with bee.id {id} — bee knowledge list"));
        };
        if string_list(&bee_of(&concepts[i].data), "anchors").is_empty() {
            return Err(format!(
                "docs/knowledge/{} has no bee.anchors — run bee knowledge seed-anchors or add anchors",
                concepts[i].rel
            ));
        }
        if !targets.contains(&i) {
            targets.push(i);
        }
    }
    Ok(targets)
}

pub(crate) fn run_confirm(flags: Flags, json: bool, pre_json: bool, t0: Instant, toks: &[&str]) -> Option<ExitCode> {
    if !keys_known(&flags, &["id", "path"]) {
        return None;
    }
    let ids: Vec<String> = repeated_ids(toks).into_iter().filter(|id| !js_trim(id).is_empty()).collect();
    let path = flags.req_str("path").map(|p| norm_anchor(js_trim(p)).to_string());
    if ids.is_empty() == path.is_none() {
        return None;
    }
    let ctx = match g_prelude("knowledge confirm", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = command_bundle(&ctx.root)?;
    let checkout = checkout_of(&dir)?.to_path_buf();
    let mut concepts = load_areas(&dir)?;

    let targets = match confirm_targets(&concepts, &ids, path.as_deref()) {
        Ok(t) => t,
        Err(message) => return Some(ctx.fail(&message)),
    };

    let head = run_git(&checkout, &["rev-parse", "HEAD"]);
    let head_sha = head.stdout.as_deref().unwrap_or("").trim().to_string();
    if head.status != Some(0) || !is_sha(&head_sha) {
        return Some(ctx.fail("cannot read HEAD of this checkout — git is unavailable or the repository has no commits"));
    }
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    for i in &targets {
        let concept = &mut concepts[*i];
        let Some(bee) = bee_mut(&mut concept.data) else {
            return Some(ctx.fail(&format!("docs/knowledge/{} has no bee map", concept.rel)));
        };
        bee.insert("verified_at".into(), Value::String(today.clone()));
        bee.insert("verified_commit".into(), Value::String(head_sha.clone()));
        if let Err(message) = write_concept(concept) {
            return Some(ctx.fail(&message));
        }
    }

    let bees: Vec<(String, Map<String, Value>)> =
        targets.iter().map(|i| (concepts[*i].rel.clone(), bee_of(&concepts[*i].data))).collect();
    let refs: Vec<(&str, &Map<String, Value>)> = bees.iter().map(|(rel, bee)| (rel.as_str(), bee)).collect();
    let graded = grade_bundle(&checkout, &refs);

    let mut lines: Vec<String> = Vec::new();
    let mut rows: Vec<Value> = Vec::new();
    for (rel, bee) in &bees {
        let grade = &graded[rel];
        lines.push(if grade.grade == ORPHAN {
            format!(
                "docs/knowledge/{rel}: stamped {} but the grade stays orphan — {}",
                short(&head_sha),
                orphan_anchor_message(grade)
            )
        } else {
            format!("docs/knowledge/{rel}: confirmed at {} on {today}", short(&head_sha))
        });
        let mut row = grade_row(rel, grade);
        row["id"] = bee.get("id").cloned().unwrap_or(Value::Null);
        rows.push(row);
    }
    let result = json!({
        "verified_at": today,
        "verified_commit": head_sha,
        "confirmed": rows,
    });
    Some(ctx.emit(&result, &lines.join("\n"), 0))
}
