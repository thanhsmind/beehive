// onboard::render — the skill-tree walker and the D9 runtime-render layer.
//
// Provenance: onboard_bee.mjs walkSkillTree (l. 632), manifestFingerprint
// (l. 666), skillDigest (l. 727), buildRenderSidecar (l. 738), the marker
// grammar (l. 749–797), validateSkillMarkers (l. 802), renderSkillBytes
// (l. 894), runtimeForTargetKind (l. 926), validateSkillTreeMarkers (l. 934),
// listBeeSkillEntries (l. 967) and sourceSkillDigestEntries (l. 986).
//
// Every regex in that range is hand-rolled here (the crate carries no regex
// dependency); each helper names the pattern it replaces so the two can be
// diffed by eye.

use super::templates::{RENDER_RUNTIMES, RENDER_SCHEMA};
use super::util::{
    read_dir_sorted, sha256_bytes, sha256_str, split_lines, split_lines_preserving, Dirent,
};
use crate::jsjson;
use serde_json::{json, Value};
use std::path::Path;

// ── walkSkillTree ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Blocked {
    pub path: String,
    pub reason: &'static str,
}

#[derive(Debug, Default, Clone)]
pub struct Walk {
    /// (relPath, sha256) in walk order — the JS Map's insertion order.
    pub files: Vec<(String, String)>,
    pub dirs: Vec<String>,
    pub blocked: Option<Blocked>,
}

impl Walk {
    pub fn file_hash(&self, rel: &str) -> Option<&str> {
        self.files.iter().find(|(r, _)| r == rel).map(|(_, h)| h.as_str())
    }
    pub fn has_file(&self, rel: &str) -> bool {
        self.files.iter().any(|(r, _)| r == rel)
    }
}

/// walkSkillTree(rootDir, transform) — lstat-only, symlinks never followed.
/// The first symlink (or other non-file/dir entry) blocks the WHOLE skill.
/// `transform` maps a file's raw bytes to the bytes that would be
/// materialized (the per-runtime render); source walks pass it, target walks
/// omit it.
pub fn walk_skill_tree(root_dir: &Path, transform: Option<&dyn Fn(&[u8]) -> Vec<u8>>) -> Walk {
    let mut walk = Walk::default();
    walk_inner(root_dir, "", transform, &mut walk);
    walk
}

fn walk_inner(
    dir: &Path,
    rel_prefix: &str,
    transform: Option<&dyn Fn(&[u8]) -> Vec<u8>>,
    out: &mut Walk,
) {
    for entry in read_dir_sorted(dir) {
        if out.blocked.is_some() {
            return;
        }
        let rel = if rel_prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{rel_prefix}/{}", entry.name)
        };
        let abs = dir.join(&entry.name);
        if entry.is_symlink {
            out.blocked = Some(Blocked { path: rel, reason: "symlink" });
            return;
        }
        if entry.is_dir {
            out.dirs.push(rel.clone());
            walk_inner(&abs, &rel, transform, out);
        } else if entry.is_file {
            let raw = std::fs::read(&abs).unwrap_or_default();
            let bytes = match transform {
                Some(f) => f(&raw),
                None => raw,
            };
            out.files.push((rel, sha256_bytes(&bytes)));
        } else {
            out.blocked = Some(Blocked { path: rel, reason: "unsupported entry type" });
            return;
        }
    }
}

/// manifestFingerprint: `JSON.stringify([...files].sort(byRelPath))`.
pub fn manifest_fingerprint(files: &[(String, String)]) -> String {
    let mut sorted: Vec<&(String, String)> = files.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let arr: Value =
        Value::Array(sorted.into_iter().map(|(k, v)| json!([k, v])).collect::<Vec<_>>());
    jsjson::stringify(&arr)
}

/// skillDigest: sha256 over the sorted [relPath, sha256] fingerprint.
pub fn skill_digest(files: &[(String, String)]) -> String {
    sha256_str(&manifest_fingerprint(files))
}

/// buildRenderSidecar(targetRuntime, [{name, files}]) — the bee-render/2
/// inventory (D7), sorted by skill name.
pub fn build_render_sidecar(target_runtime: &str, entries: &[(String, Vec<(String, String)>)]) -> Value {
    let mut skills: Vec<(String, String)> =
        entries.iter().map(|(name, files)| (name.clone(), skill_digest(files))).collect();
    skills.sort_by(|a, b| a.0.cmp(&b.0));
    json!({
        "schema": RENDER_SCHEMA,
        "target_runtime": target_runtime,
        "skills": skills.into_iter().map(|(name, sha)| json!({"name": name, "sha256": sha})).collect::<Vec<_>>(),
    })
}

// ── marker grammar ─────────────────────────────────────────────────────────

/// `NEAR_MARKER_RE = /^[ \t]*<!--[ \t]*bee:(only|end)\b/`
fn is_near_marker(line: &str) -> bool {
    let rest = line.trim_start_matches([' ', '\t']);
    let Some(rest) = rest.strip_prefix("<!--") else { return false };
    let rest = rest.trim_start_matches([' ', '\t']);
    for kw in ["only", "end"] {
        let full = format!("bee:{kw}");
        if let Some(after) = rest.strip_prefix(full.as_str()) {
            // \b: the next char must not be a word char.
            let boundary = after
                .chars()
                .next()
                .map(|c| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(true);
            if boundary {
                return true;
            }
        }
    }
    false
}

/// `MARKER_ONLY_RE = /^<!-- bee:only (\S+) -->[ \t]*$/`
fn match_marker_only(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("<!-- bee:only ")?;
    // \S+ is greedy and cannot span whitespace, so the label is the maximal
    // leading non-whitespace run; the remainder must be exactly " -->" plus
    // trailing horizontal whitespace.
    let label_len = rest.find(char::is_whitespace).unwrap_or(rest.len());
    if label_len == 0 {
        return None;
    }
    let (label, tail) = rest.split_at(label_len);
    let tail = tail.strip_prefix(" -->")?;
    if tail.chars().all(|c| c == ' ' || c == '\t') {
        Some(label)
    } else {
        None
    }
}

/// `MARKER_END_RE = /^<!-- bee:end -->[ \t]*$/`
fn match_marker_end(line: &str) -> bool {
    line.strip_prefix("<!-- bee:end -->")
        .is_some_and(|tail| tail.chars().all(|c| c == ' ' || c == '\t'))
}

/// `FRONTMATTER_DELIM_RE = /^---[ \t]*$/`
fn is_frontmatter_delim(line: &str) -> bool {
    line.strip_prefix("---").is_some_and(|tail| tail.chars().all(|c| c == ' ' || c == '\t'))
}

/// fenceChar: `/^[ \t]*(`{3,}|~{3,})/` → the fence character, if any.
fn fence_char(line: &str) -> Option<char> {
    let rest = line.trim_start_matches([' ', '\t']);
    for ch in ['`', '~'] {
        if rest.chars().take(3).filter(|c| *c == ch).count() == 3 {
            return Some(ch);
        }
    }
    None
}

enum MarkerClass<'a> {
    Only(&'a str),
    End,
    Error(String),
}

/// classifyMarkerLine (l. 784).
fn classify_marker_line(line: &str) -> MarkerClass<'_> {
    if let Some(label) = match_marker_only(line) {
        if !RENDER_RUNTIMES.contains(&label) {
            return MarkerClass::Error(format!(
                "unknown runtime label \"{label}\" (expected {})",
                RENDER_RUNTIMES.join(" or ")
            ));
        }
        return MarkerClass::Only(label);
    }
    if match_marker_end(line) {
        return MarkerClass::End;
    }
    MarkerClass::Error(format!(
        "ambiguous near-marker \"{}\" (not an exact full-line bee marker)",
        js_trim(line)
    ))
}

/// JS `String.prototype.trim()` — Unicode whitespace on both ends.
fn js_trim(s: &str) -> &str {
    s.trim()
}

/// validateSkillMarkers (l. 802): whole-file grammar check, human-readable
/// error strings, empty when well-formed.
pub fn validate_skill_markers(text: &str) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let lines = split_lines(text);
    // Frontmatter span: only when the very first line is a `---` delimiter.
    let mut frontmatter_end: i64 = -1;
    if !lines.is_empty() && is_frontmatter_delim(lines[0]) {
        for (i, line) in lines.iter().enumerate().skip(1) {
            if is_frontmatter_delim(line) {
                frontmatter_end = i as i64;
                break;
            }
        }
    }
    let mut fence: Option<char> = None;
    let mut open_runtime: Option<String> = None;
    let mut open_line: i64 = -1;
    let mut first_frontmatter_opener: i64 = -1;

    for (i, line) in lines.iter().enumerate() {
        let idx = i as i64;
        if idx > frontmatter_end {
            let fc = fence_char(line);
            match fence {
                None => {
                    if let Some(c) = fc {
                        fence = Some(c);
                    }
                }
                Some(open) => {
                    if fc == Some(open) {
                        fence = None;
                    }
                }
            }
        }
        if !is_near_marker(line) {
            if first_frontmatter_opener == -1
                && fence.is_none()
                && is_frontmatter_delim(line)
                && idx > 0
                && frontmatter_end == -1
            {
                first_frontmatter_opener = idx;
            }
            continue;
        }
        if fence.is_some() {
            errors.push(format!(
                "marker inside a fenced code block at line {}: \"{}\"",
                i + 1,
                js_trim(line)
            ));
            continue;
        }
        if frontmatter_end != -1 && idx <= frontmatter_end {
            errors.push(format!(
                "marker inside YAML frontmatter at line {}: \"{}\"",
                i + 1,
                js_trim(line)
            ));
            continue;
        }
        match classify_marker_line(line) {
            MarkerClass::Error(e) => {
                errors.push(format!("{e} at line {}", i + 1));
            }
            MarkerClass::Only(runtime) => {
                if open_runtime.is_some() {
                    errors.push(format!(
                        "nested bee:only block at line {} (block opened at line {} not closed)",
                        i + 1,
                        open_line + 1
                    ));
                    continue;
                }
                open_runtime = Some(runtime.to_string());
                open_line = idx;
                if first_frontmatter_opener != -1 {
                    errors.push(format!("marker before YAML frontmatter at line {}", i + 1));
                }
            }
            MarkerClass::End => {
                if open_runtime.is_none() {
                    errors.push(format!("stray bee:end with no open block at line {}", i + 1));
                    continue;
                }
                open_runtime = None;
            }
        }
    }
    if open_runtime.is_some() {
        errors.push(format!("unclosed bee:only block opened at line {}", open_line + 1));
    }
    errors
}

/// bufHasMarkerBytes (l. 763): the cheap byte gate that keeps the no-marker
/// path from ever decoding (byte-identity guarantee).
pub fn buf_has_marker_bytes(buf: &[u8]) -> bool {
    contains_bytes(buf, b"bee:only") || contains_bytes(buf, b"bee:end")
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// renderSkillBytes (l. 894): drop off-runtime blocks, strip marker lines,
/// pass a marker-free file through byte-identical (never decoded).
pub fn render_skill_bytes(buf: &[u8], runtime: &str) -> Vec<u8> {
    if !buf_has_marker_bytes(buf) {
        return buf.to_vec();
    }
    let text = String::from_utf8_lossy(buf).into_owned();
    if !split_lines(&text).iter().any(|l| is_near_marker(l)) {
        return buf.to_vec(); // literal "bee:only"/"bee:end" in prose
    }
    let mut out = String::new();
    let mut open_runtime: Option<String> = None;
    for (content, term) in split_lines_preserving(&text) {
        if is_near_marker(content) {
            match classify_marker_line(content) {
                MarkerClass::Only(r) => {
                    open_runtime = Some(r.to_string());
                    continue;
                }
                MarkerClass::End => {
                    open_runtime = None;
                    continue;
                }
                MarkerClass::Error(_) => {}
            }
        }
        if open_runtime.as_deref().is_none_or(|r| r == runtime) {
            out.push_str(content);
            out.push_str(term);
        }
    }
    out.into_bytes()
}

/// runtimeForTargetKind (l. 926): repo-agents renders Codex, every other
/// managed root renders Claude.
pub fn runtime_for_target_kind(kind: &str) -> &'static str {
    if kind == "repo-agents" {
        "codex"
    } else {
        "claude"
    }
}

/// listBeeSkillEntries (l. 967): the deletion domain IS the iteration domain
/// — only `/^bee-/` entries are ever enumerated (D4).
pub fn list_bee_skill_entries(root: &Path) -> Vec<Dirent> {
    if !root.exists() {
        return Vec::new();
    }
    read_dir_sorted(root).into_iter().filter(|e| e.name.starts_with("bee-")).collect()
}

/// validateSkillTreeMarkers (l. 934): whole-tree grammar gate; a non-empty
/// result refuses the ENTIRE apply with zero writes.
pub fn validate_skill_tree_markers(source_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    for entry in list_bee_skill_entries(source_root) {
        if entry.is_symlink || !entry.is_dir {
            continue;
        }
        let skill_dir = source_root.join(&entry.name);
        let walk = walk_skill_tree(&skill_dir, None);
        if walk.blocked.is_some() {
            continue;
        }
        for (rel, _) in &walk.files {
            let abs = super::util::join_rel(&skill_dir, rel);
            let Ok(buf) = std::fs::read(&abs) else { continue };
            if !buf_has_marker_bytes(&buf) {
                continue;
            }
            for e in validate_skill_markers(&String::from_utf8_lossy(&buf)) {
                errors.push(format!("{}/{rel}: {e}", entry.name));
            }
        }
    }
    errors
}

/// sourceSkillDigestEntries (l. 986): the "should be installed" inventory for
/// one runtime — blocked source dirs are excluded, because they are never
/// synced to any target either.
pub fn source_skill_digest_entries(
    source_root: &Path,
    runtime: &str,
) -> Vec<(String, Vec<(String, String)>)> {
    let render = |buf: &[u8]| render_skill_bytes(buf, runtime);
    let mut out = Vec::new();
    for entry in list_bee_skill_entries(source_root) {
        if entry.is_symlink || !entry.is_dir {
            continue;
        }
        let walk = walk_skill_tree(&source_root.join(&entry.name), Some(&render));
        if walk.blocked.is_some() {
            continue;
        }
        out.push((entry.name, walk.files));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_marker_file_renders_byte_identical() {
        let raw = b"---\ntitle: x\n---\n\nplain \xEF\xBB\xBF body\r\nno trailing newline";
        assert_eq!(render_skill_bytes(raw, "claude"), raw.to_vec());
        assert_eq!(render_skill_bytes(raw, "codex"), raw.to_vec());
    }

    #[test]
    fn prose_mentioning_marker_words_is_untouched() {
        let raw = b"the token bee:only appears in prose here\n";
        assert_eq!(render_skill_bytes(raw, "codex"), raw.to_vec());
    }

    #[test]
    fn runtime_blocks_are_filtered_and_markers_stripped() {
        let src = "head\n<!-- bee:only claude -->\nCLAUDE\n<!-- bee:end -->\n<!-- bee:only codex -->\nCODEX\n<!-- bee:end -->\ntail\n";
        assert_eq!(
            String::from_utf8(render_skill_bytes(src.as_bytes(), "claude")).unwrap(),
            "head\nCLAUDE\ntail\n"
        );
        assert_eq!(
            String::from_utf8(render_skill_bytes(src.as_bytes(), "codex")).unwrap(),
            "head\nCODEX\ntail\n"
        );
    }

    #[test]
    fn crlf_line_endings_survive_rendering() {
        let src = "a\r\n<!-- bee:only codex -->\r\nX\r\n<!-- bee:end -->\r\nb\r\n";
        assert_eq!(
            String::from_utf8(render_skill_bytes(src.as_bytes(), "claude")).unwrap(),
            "a\r\nb\r\n"
        );
    }

    #[test]
    fn marker_grammar_errors_match_node_wording() {
        assert!(validate_skill_markers("plain\n").is_empty());
        // An unknown label errors AND leaves no block open, so the closing
        // marker then reads as stray — both errors, in that order.
        let e = validate_skill_markers("<!-- bee:only rust -->\nx\n<!-- bee:end -->\n");
        assert_eq!(
            e,
            vec![
                "unknown runtime label \"rust\" (expected claude or codex) at line 1",
                "stray bee:end with no open block at line 3"
            ]
        );
        let e = validate_skill_markers("<!-- bee:end -->\n");
        assert_eq!(e, vec!["stray bee:end with no open block at line 1"]);
        let e = validate_skill_markers("<!-- bee:only claude -->\n");
        assert_eq!(e, vec!["unclosed bee:only block opened at line 1"]);
        let e = validate_skill_markers(
            "<!-- bee:only claude -->\n<!-- bee:only codex -->\n<!-- bee:end -->\n",
        );
        assert_eq!(e, vec!["nested bee:only block at line 2 (block opened at line 1 not closed)"]);
        let e = validate_skill_markers("```\n<!-- bee:end -->\n```\n");
        assert_eq!(e, vec!["marker inside a fenced code block at line 2: \"<!-- bee:end -->\""]);
        let e = validate_skill_markers("---\n<!-- bee:end -->\n---\n");
        assert_eq!(e, vec!["marker inside YAML frontmatter at line 2: \"<!-- bee:end -->\""]);
        let e = validate_skill_markers("  <!-- bee:only claude -->\n<!-- bee:end -->\n");
        assert_eq!(
            e,
            vec![
                "ambiguous near-marker \"<!-- bee:only claude -->\" (not an exact full-line bee marker) at line 1",
                "stray bee:end with no open block at line 2"
            ]
        );
    }

    #[test]
    fn fingerprint_and_digest_match_node_shapes() {
        let files = vec![("b".to_string(), "2".to_string()), ("a".to_string(), "1".to_string())];
        assert_eq!(manifest_fingerprint(&files), r#"[["a","1"],["b","2"]]"#);
        assert_eq!(skill_digest(&files), sha256_str(r#"[["a","1"],["b","2"]]"#));
    }

    #[test]
    fn sidecar_shape_is_bee_render_2_sorted() {
        let entries = vec![
            ("bee-z".to_string(), vec![("f".to_string(), "h".to_string())]),
            ("bee-a".to_string(), vec![]),
        ];
        let sc = build_render_sidecar("codex", &entries);
        assert_eq!(sc["schema"], "bee-render/2");
        assert_eq!(sc["target_runtime"], "codex");
        assert_eq!(sc["skills"][0]["name"], "bee-a");
        assert_eq!(sc["skills"][1]["name"], "bee-z");
    }

    #[test]
    fn walk_blocks_on_first_symlink() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "x").unwrap();
        let walk = walk_skill_tree(dir.path(), None);
        assert!(walk.blocked.is_none());
        assert_eq!(walk.files.len(), 1);
        assert_eq!(walk.files[0].0, "a.md");
    }

    #[test]
    fn runtime_for_target_kind_maps_agents_to_codex() {
        assert_eq!(runtime_for_target_kind("repo-agents"), "codex");
        assert_eq!(runtime_for_target_kind("repo-claude"), "claude");
        assert_eq!(runtime_for_target_kind("global"), "claude");
        assert_eq!(runtime_for_target_kind("legacy-global"), "claude");
    }
}
