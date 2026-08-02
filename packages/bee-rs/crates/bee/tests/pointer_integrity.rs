// pointer_integrity — the skill-reference pointer gate, ported to Rust.
//
// PROVENANCE. The Node original was `scripts/tests/test_skill_pointers.mjs`,
// deleted with the rest of the Node tree at the R6 cutover (2026-08-02). The
// contract it enforced is recorded in
// `docs/knowledge/areas/verify-pipeline/skill-reference-pointer-integrity.md`
// (router-cost D5/D8, tick-contract-inline T4) and that concept stayed
// `lifecycle: active` while nothing implemented it — the exact "what goes
// quiet?" failure the port's own learnings name. This file re-arms it.
//
// WHAT IT ENFORCES (R1-R7 of the concept):
//   R1  every pointer from an instruction document to a reference resolves
//   R2  a pointer naming a section is checked against that section
//   R3  only source documents are scanned; rendered projections never are
//   R4  negative controls ship with it and run every time
//   R7  a citation naming several headings points at EVERY heading it names
//
// R7 is where this port is STRICTER than the Node original. That gate bound
// only the first heading a citation named, leaving later ones unguarded — an
// open gap recorded in the concept ("Open Gaps", last bullet). Binding all of
// them is the whole point of the rule, so the port closes it rather than
// reproducing the limit.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    // crates/bee -> crates -> bee-rs -> packages -> beehive
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

/// Source instruction documents (R3). The rendered projections under
/// `.claude/`, `.agents/`, `.claude-plugin/` and `.codex-plugin/` are byte
/// copies of these, so scanning them would multiply one authoring mistake by
/// the number of delivery targets.
const SOURCE_FILES: &[&str] = &["AGENTS.md", "CLAUDE.md", "packages/bee/AGENTS.block.md"];
const SOURCE_TREE: &str = "skills";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    file: String,
    line: usize,
    text: String,
    problem: String,
}

/// One citation: the reference it names, plus every heading it names (R7).
#[derive(Debug)]
struct Citation {
    line: usize,
    text: String,
    target: String,
    headings: Vec<String>,
    /// The instruction set named just before a bare pointer, if any.
    named_set: Option<String>,
}

fn collect_sources(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = SOURCE_FILES
        .iter()
        .map(|r| root.join(r))
        .filter(|p| p.is_file())
        .collect();
    walk_md(&root.join(SOURCE_TREE), &mut out);
    out.sort();
    out
}

fn walk_md(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_md(&path, out);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            out.push(path);
        }
    }
}

/// Blank out fenced code blocks. A path inside a fence is prose about a path,
/// not a live promise to a reader ("Edge Cases Settled").
fn strip_fences(text: &str) -> Vec<String> {
    let mut fenced = false;
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                fenced = !fenced;
                return String::new();
            }
            if fenced { String::new() } else { line.to_string() }
        })
        .collect()
}

/// Headings quoted inside `s`, in order. Whitespace is normalised so a heading
/// wrapped across a line break still matches.
fn quoted_runs(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { break };
        let raw = &after[..close];
        let norm = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if !norm.is_empty() {
            out.push(norm);
        }
        rest = &after[close + 1..];
    }
    out
}

/// The window after a citation's closing backtick in which a heading may be
/// named. Three phrasings are in real use: a parenthetical `("A", "B")`, a
/// comma form `, "A" / "B"`, and a section sign `§ A`. The parenthetical is
/// depth-tracked so a nested `(` does not truncate its parent.
fn heading_window(after: &str) -> Option<&str> {
    let lead = after.trim_start_matches([' ', '\t', '\n', '\r']);
    let skipped = after.len() - lead.len();
    if lead.starts_with('(') {
        let bytes = lead.as_bytes();
        let mut depth = 0usize;
        for (i, b) in bytes.iter().enumerate() {
            match b {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&after[skipped + 1..skipped + i]);
                    }
                }
                _ => {}
            }
        }
        return None;
    }
    // Comma / § form: `path.md`, "A" / "B" — the headings follow the citation
    // directly. The window is the LEADING run of quotes and separators only. It
    // must not extend to the end of the paragraph: a quoted phrase later in the
    // same prose is a mention, and a mention is not a pointer.
    let mut idx = 0usize;
    let bytes = lead.as_bytes();
    if bytes.first() == Some(&b',') {
        idx = 1;
    } else if !lead.starts_with('§') && !lead.starts_with('"') {
        return None;
    }
    let mut end = idx;
    loop {
        let rest = &lead[end..];
        let trimmed = rest.trim_start_matches([' ', '\t', '\n', '\r', ',', '/']);
        let skipped = rest.len() - trimmed.len();
        if trimmed.starts_with('"') {
            let Some(close) = trimmed[1..].find('"') else { break };
            end += skipped + 1 + close + 1;
        } else if trimmed.starts_with('§') {
            let seg = &trimmed['§'.len_utf8()..];
            let cut = seg.find(['.', ',', ')', '\n']).unwrap_or(seg.len());
            end += skipped + '§'.len_utf8() + cut;
            break;
        } else {
            break;
        }
    }
    if end > idx { Some(&lead[..end]) } else { None }
}

fn section_sign_headings(window: &str) -> Vec<String> {
    window
        .split('§')
        .skip(1)
        .filter_map(|seg| {
            let cut = seg.find(['.', ',', ')', '\n']).unwrap_or(seg.len());
            let norm = seg[..cut].split_whitespace().collect::<Vec<_>>().join(" ");
            if norm.is_empty() { None } else { Some(norm) }
        })
        .collect()
}

/// Extract every live citation naming a `references/*.md` document. A live
/// pointer is backticked; a retired name written unquoted reads as history and
/// is not a promise.
fn citations(lines: &[String]) -> Vec<Citation> {
    let joined = lines.join("\n");
    let mut out = Vec::new();
    let mut cursor = 0usize;
    let mut prev_span: Option<(usize, String)> = None;
    while let Some(open_rel) = joined[cursor..].find('`') {
        let open = cursor + open_rel;
        let after_open = open + 1;
        let Some(close_rel) = joined[after_open..].find('`') else { break };
        let close = after_open + close_rel;
        let span = &joined[after_open..close];
        cursor = close + 1;
        if !span.contains("references/") || !span.ends_with(".md") {
            if span.starts_with("bee-") && !span.contains('/') && !span.contains(' ') {
                prev_span = Some((close, span.to_string()));
            }
            continue;
        }
        let named_set = prev_span
            .as_ref()
            .filter(|(end, _)| open.saturating_sub(*end) <= 8)
            .map(|(_, name)| name.clone());
        let window = heading_window(&joined[cursor..]);
        let mut headings = window.map(quoted_runs).unwrap_or_default();
        if let Some(w) = window {
            headings.extend(section_sign_headings(w));
        }
        let line = joined[..open].matches('\n').count() + 1;
        let text = lines.get(line - 1).cloned().unwrap_or_default();
        out.push(Citation {
            line,
            text: text.trim().to_string(),
            target: span.to_string(),
            headings,
            named_set,
        });
    }
    out
}

/// Bare pointers resolve against the citing document's own directory;
/// qualified pointers (`<skill>/references/...`) against the instruction set
/// they name. Resolution is deterministic — a bare pointer whose own directory
/// has no such file is a finding, never a cue to search elsewhere.
fn resolve(root: &Path, citing: &Path, target: &str, named_set: Option<&str>) -> PathBuf {
    if !target.starts_with("references/") {
        return root.join(SOURCE_TREE).join(target);
    }
    // A bare pointer carried by a document that NAMES its instruction set in
    // the same breath (``bee-hive` → `references/…``) is qualified by that
    // name, not by where the citing file happens to sit.
    if let Some(set) = named_set {
        let via_set = root.join(SOURCE_TREE).join(set).join(target);
        if via_set.is_file() {
            return via_set;
        }
    }
    let Some(dir) = citing.parent() else { return root.join(target) };
    let local = dir.join(target);
    if local.is_file() {
        return local;
    }
    // A document that already lives inside `references/` names its siblings the
    // same way its skill root does, so the skill root is the second and last
    // place resolution looks. It never searches further: guessing would hide
    // the error this gate exists to surface.
    if dir.file_name().map(|n| n == "references").unwrap_or(false) {
        if let Some(skill_root) = dir.parent() {
            return skill_root.join(target);
        }
    }
    local
}

/// The citable names inside a reference: its headings, AND its bolded handles.
/// Both count because the repository's own authoring contract says so —
/// `expertise/README.md`: "Every rule carries a citable name (its heading, or a
/// bolded handle in prose)". A gate that bound only headings would report a
/// perfectly reachable rule as broken, and the fix for that finding would be to
/// damage a document to satisfy the checker.
///
/// A handle is bold at the START of a line (after list markers), which is how
/// they are written. A bold span mid-sentence is emphasis, not a name.
fn citable_names(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in strip_fences(text) {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix('#') {
            let body = rest.trim_start_matches('#').trim();
            if !body.is_empty() {
                out.push(body.to_string());
            }
            continue;
        }
        // Strip list/quote markers only in their `marker + space` form. A bare
        // trim of `*` would eat the opening `**` of the handle itself.
        let mut handle = t;
        loop {
            let next = handle
                .strip_prefix("- ")
                .or_else(|| handle.strip_prefix("* "))
                .or_else(|| handle.strip_prefix("+ "))
                .or_else(|| handle.strip_prefix("> "));
            match next {
                Some(rest) => handle = rest.trim_start(),
                None => break,
            }
        }
        if let Some(rest) = handle.strip_prefix("**") {
            if let Some(end) = rest.find("**") {
                let name = rest[..end].trim().trim_end_matches(':').trim();
                if !name.is_empty() {
                    out.push(name.to_string());
                }
            }
        }
    }
    out
}

/// Scan one tree and return (pointers examined, findings). Reporting a count of
/// what was looked at is part of the contract: zero broken means nothing next
/// to zero examined.
fn scan(root: &Path, files: &[PathBuf]) -> (usize, Vec<Finding>) {
    let mut examined = 0usize;
    let mut findings = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(file) else { continue };
        let lines = strip_fences(&text);
        let rel = file.strip_prefix(root).unwrap_or(file).display().to_string().replace('\\', "/");
        for cite in citations(&lines) {
            examined += 1;
            let target = resolve(root, file, &cite.target, cite.named_set.as_deref());
            let Ok(target_text) = std::fs::read_to_string(&target) else {
                findings.push(Finding {
                    file: rel.clone(),
                    line: cite.line,
                    text: cite.text.clone(),
                    problem: format!("target does not exist: {}", cite.target),
                });
                continue;
            };
            let present = citable_names(&target_text);
            for named in &cite.headings {
                if !present.iter().any(|h| h.contains(named.as_str())) {
                    findings.push(Finding {
                        file: rel.clone(),
                        line: cite.line,
                        text: cite.text.clone(),
                        problem: format!("{} has no section \"{}\"", cite.target, named),
                    });
                }
            }
        }
    }
    findings.sort();
    (examined, findings)
}

fn report(findings: &[Finding]) -> String {
    findings
        .iter()
        .map(|f| format!("  {}:{} — {}\n      {}", f.file, f.line, f.problem, f.text))
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── the live gate (R1, R2, R3, R7) ───────────────────────────────────────

#[test]
fn every_instruction_pointer_resolves_to_its_reference_and_section() {
    let root = repo_root();
    let files = collect_sources(&root);
    assert!(
        files.len() > 10,
        "scan set collapsed to {} file(s) — a gate that scans nothing reports green",
        files.len()
    );
    let (examined, findings) = scan(&root, &files);
    assert!(
        examined > 20,
        "only {examined} pointer(s) found across {} source document(s) — the matcher stopped matching",
        files.len()
    );
    assert!(
        findings.is_empty(),
        "{} broken pointer(s) out of {examined} examined:\n{}",
        findings.len(),
        report(&findings)
    );
}

// ─── negative controls (R4): both failure modes proven detectable, every run ──

struct Fixture {
    dir: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("skills/demo/references")).unwrap();
        Self { dir }
    }
    fn write(&self, rel: &str, body: &str) -> PathBuf {
        let path = self.dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
        path
    }
    fn scan(&self, citing: &Path) -> (usize, Vec<Finding>) {
        scan(self.dir.path(), &[citing.to_path_buf()])
    }
}

#[test]
fn selftest_a_missing_target_file_is_detected() {
    let fx = Fixture::new();
    let citing = fx.write("skills/demo/SKILL.md", "See `references/gone.md` for detail.\n");
    let (examined, findings) = fx.scan(&citing);
    assert_eq!(examined, 1);
    assert_eq!(findings.len(), 1, "a pointer to a missing file must be a finding");
    assert!(findings[0].problem.contains("does not exist"), "{:?}", findings[0]);
}

#[test]
fn selftest_a_vanished_section_is_detected() {
    let fx = Fixture::new();
    fx.write("skills/demo/references/real.md", "# Real\n\n## Kept Heading\n\nbody\n");
    let citing = fx.write("skills/demo/SKILL.md", "See `references/real.md` (\"Renamed Away\").\n");
    let (_, findings) = fx.scan(&citing);
    assert_eq!(findings.len(), 1, "a correct file with a vanished heading is still broken");
    assert!(findings[0].problem.contains("Renamed Away"), "{:?}", findings[0]);
}

/// R7, and the gap the Node gate left open: the SECOND heading must be bound
/// too. This control fails on any implementation that reads only the first.
#[test]
fn selftest_every_heading_in_a_multi_heading_citation_is_bound() {
    let fx = Fixture::new();
    fx.write("skills/demo/references/real.md", "## First Heading\n\n## Second Heading\n");
    let good = fx.write("skills/demo/a.md", "See `references/real.md` (\"First Heading\", \"Second Heading\").\n");
    assert!(fx.scan(&good).1.is_empty(), "both headings exist — must be clean");

    let bad = fx.write("skills/demo/b.md", "See `references/real.md` (\"First Heading\", \"Missing Second\").\n");
    let (_, findings) = fx.scan(&bad);
    assert_eq!(findings.len(), 1, "the second heading of a multi-heading citation must be checked");
    assert!(findings[0].problem.contains("Missing Second"), "{:?}", findings[0]);
}

#[test]
fn selftest_a_citation_inside_a_fence_is_not_a_pointer() {
    let fx = Fixture::new();
    let citing = fx.write(
        "skills/demo/SKILL.md",
        "Example:\n\n```\nSee `references/gone.md` for detail.\n```\n\nEnd.\n",
    );
    let (examined, findings) = fx.scan(&citing);
    assert_eq!(examined, 0, "a path inside a fence is prose, not a promise");
    assert!(findings.is_empty());
}

#[test]
fn selftest_a_qualified_pointer_resolves_against_the_named_instruction_set() {
    let fx = Fixture::new();
    fx.write("skills/other/references/target.md", "## Anchor\n");
    let good = fx.write("skills/demo/SKILL.md", "See `other/references/target.md` (\"Anchor\").\n");
    assert!(fx.scan(&good).1.is_empty(), "qualified pointer must resolve against skills/<set>/");

    let bad = fx.write("skills/demo/b.md", "See `references/target.md` (\"Anchor\").\n");
    let (_, findings) = fx.scan(&bad);
    assert_eq!(findings.len(), 1, "a bare pointer resolves locally and must not search elsewhere");
}

#[test]
fn selftest_a_heading_citation_matches_a_longer_real_heading() {
    let fx = Fixture::new();
    fx.write("skills/demo/references/real.md", "### Judgment contract — rails for workers\n");
    let citing = fx.write("skills/demo/SKILL.md", "See `references/real.md` (\"Judgment contract\").\n");
    assert!(fx.scan(&citing).1.is_empty(), "a citation may name the stable prefix of a heading");
}

/// The repository's authoring contract makes a bolded handle a citable name.
/// Both halves are controlled: a real handle resolves, a vanished one does not.
#[test]
fn selftest_a_bolded_handle_is_a_citable_name() {
    let fx = Fixture::new();
    fx.write(
        "skills/demo/references/real.md",
        "## Section\n\n**Parallel by default:** cells fan out.\n",
    );
    let good = fx.write("skills/demo/a.md", "See `references/real.md` (\"Parallel by default\").\n");
    assert!(fx.scan(&good).1.is_empty(), "a bolded handle is a citable name");

    let bad = fx.write("skills/demo/b.md", "See `references/real.md` (\"Serial by default\").\n");
    assert_eq!(fx.scan(&bad).1.len(), 1, "a handle that is not there is still a broken promise");
}

#[test]
fn selftest_mid_sentence_bold_is_emphasis_not_a_name() {
    let fx = Fixture::new();
    fx.write(
        "skills/demo/references/real.md",
        "## Section\n\nthe rule is **never silently** applied.\n",
    );
    let citing = fx.write("skills/demo/a.md", "See `references/real.md` (\"never silently\").\n");
    assert_eq!(fx.scan(&citing).1.len(), 1, "emphasis mid-sentence is not a citable name");
}

#[test]
fn selftest_the_section_sign_form_is_recognised() {
    let fx = Fixture::new();
    fx.write("skills/demo/references/real.md", "## Communication contract\n");
    let good = fx.write("skills/demo/a.md", "See `references/real.md` § Communication contract.\n");
    assert!(fx.scan(&good).1.is_empty(), "the § phrasing is one of the three in use");

    let bad = fx.write("skills/demo/b.md", "See `references/real.md` § Vanished Section.\n");
    assert_eq!(fx.scan(&bad).1.len(), 1, "the § phrasing must be checked, not merely parsed");
}
