// The rule-id parity fence.
//
// A rule id lives in TWO places: one `<!-- rule: <id> -->` marker in the shipped
// doctrine — `packages/bee/LEADER.md` or a file under `packages/bee/playbooks/`
// (the thin `AGENTS.block.md` carries none, and its render in `AGENTS.md` with
// it) — and one row per rule in the `## AGENTS.md rule homes` section of the
// doctrine-layer concept, which names the file that carries the marker. `bee knowledge check` validates ref→home; nothing
// validated marker↔marker↔row, and nothing at all read the spoken line the
// invocation law depends on. A rule you can name out loud but cannot find in
// the index is a law with no address.
//
// Three things are pinned here.
//
//   1. The two AGENTS surfaces carry the SAME marker set. They move together
//      through `bee dev regen`; a set that split means one of them is stale.
//   2. That set equals the index's rows, both directions — a marked rule with
//      no row, and a row for a rule nobody marks.
//   3. Every row carries a non-empty `spoken:` line BEFORE its `applied_at:`
//      line. That order is not decoration: `parse_agents_rule_homes` reads the
//      indented block after `applied_at:` as paths, so a spoken line placed
//      there would be parsed as one.
//
// Shape, deliberately: pure filesystem, std only, and NOTHING imported from the
// bee crate — the model is `specs_fence.rs` and `route_class_parity.rs`. The
// marker scanner and the section boundary are re-derived from the files' own
// text rather than from a list held here. A fence keeping its own copy of the
// ten ids agrees with itself forever and catches nothing; that duplication has
// already happened once in this codebase (`FEATURE_ROUTE_LANE_CLASSES` in
// verbs/drivers/close.rs) and is the trap this file must not repeat.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// The generated operating block, and the source it is rendered from.
const AGENTS_MD: &str = "AGENTS.md";
const AGENTS_BLOCK: &str = "packages/bee/AGENTS.block.md";
const LEADER: &str = "packages/bee/LEADER.md";
const PLAYBOOKS_DIR: &str = "packages/bee/playbooks";
const THIN_BLOCK_LIMIT: usize = 3000;

/// The single home of the rule index.
const INDEX: &str =
    "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md";

/// The exact heading `parse_agents_rule_homes`
/// (`packages/bee-rs/crates/bee/src/verbs/knowledge/ownership.rs`) finds the
/// section by. Both it and this fence stop at the next `\n## `, so the two
/// cannot disagree about where the section ends.
const HEADING: &str = "## AGENTS.md rule homes";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// Rule ids opened by `<!-- rule: <id> -->`. A closing `<!-- /rule -->` carries
/// no id and is skipped, exactly as `extract_rule_markers` skips it.
fn marker_list(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = text[cursor..].find("<!--") {
        let start = cursor + rel + 4;
        let Some(rel_end) = text[start..].find("-->") else { break };
        let end = start + rel_end;
        if let Some(id) = text[start..end].trim().strip_prefix("rule:") {
            let id = id.trim();
            if !id.is_empty() && !id.starts_with('/') {
                out.push(id.to_string());
            }
        }
        cursor = end + 3;
    }
    out
}

fn markers(text: &str) -> BTreeSet<String> {
    marker_list(text).into_iter().collect()
}

fn doctrine_files() -> Vec<String> {
    let mut files = vec![AGENTS_BLOCK.to_string(), LEADER.to_string()];
    let mut playbooks: Vec<String> = std::fs::read_dir(repo_root().join(PLAYBOOKS_DIR))
        .unwrap_or_else(|e| panic!("cannot read {PLAYBOOKS_DIR}: {e}"))
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".md"))
        .map(|n| format!("{PLAYBOOKS_DIR}/{n}"))
        .collect();
    playbooks.sort();
    files.extend(playbooks);
    files
}

fn rule_files() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for file in doctrine_files() {
        for id in marker_list(&read(&file)) {
            out.push((id, file.clone()));
        }
    }
    out
}

/// The rule-homes section only, bounded the way the parser bounds it.
fn rule_homes_section(text: &str) -> &str {
    let start = text.find(HEADING).unwrap_or_else(|| {
        panic!(
            "{INDEX} no longer carries the heading {HEADING:?}.\n\n`parse_agents_rule_homes` \
             finds the section by that exact string, so losing it empties the parser AND this \
             fence in silence. FIX: restore the heading, or move the index and point both this \
             fence and that parser at its new home."
        )
    });
    let rest = &text[start + HEADING.len()..];
    match rest.find("\n## ") {
        Some(end) => &rest[..end],
        None => rest,
    }
}

struct Row {
    id: String,
    home: Option<String>,
    /// The `spoken:` text found before this row's `applied_at:` line, if any.
    spoken: Option<String>,
}

/// A row opens with ``- `<id>` (`` — the same `- ` bullet
/// `parse_agents_rule_homes` treats as the start of a new rule.
fn row_id(line: &str) -> Option<String> {
    let rest = line.strip_prefix("- `")?;
    let close = rest.find('`')?;
    rest[close + 1..].starts_with(" (").then(|| rest[..close].to_string())
}

fn row_home(line: &str) -> Option<String> {
    let open = line.find(" (")?;
    let inner = &line[open + 2..];
    let end = inner.find(" § ").or_else(|| inner.find(')'))?;
    Some(inner[..end].trim().to_string())
}

fn rows(section: &str) -> Vec<Row> {
    let mut out: Vec<Row> = Vec::new();
    let mut in_applied_at = false;

    for line in section.lines() {
        if let Some(id) = row_id(line) {
            out.push(Row { home: row_home(line), id, spoken: None });
            in_applied_at = false;
            continue;
        }
        let Some(row) = out.last_mut() else { continue };
        let trimmed = line.trim().trim_start_matches("- ");
        if trimmed.starts_with("applied_at:") {
            in_applied_at = true;
        } else if !in_applied_at {
            if let Some(spoken) = trimmed.strip_prefix("spoken:") {
                row.spoken = Some(spoken.trim().to_string());
            }
        }
    }
    out
}

// ── the shipped doctrine carries each rule marker once, and the block is thin ──

#[test]
fn the_rendered_block_carries_the_same_markers_as_its_source() {
    let rendered = markers(&read(AGENTS_MD));
    let source = markers(&read(AGENTS_BLOCK));

    let missing_from_source: Vec<&str> =
        rendered.difference(&source).map(String::as_str).collect();
    let missing_from_rendered: Vec<&str> =
        source.difference(&rendered).map(String::as_str).collect();

    assert!(
        missing_from_source.is_empty() && missing_from_rendered.is_empty(),
        "the two AGENTS surfaces mark different rules.\n\n  in {AGENTS_MD}, absent from \
         {AGENTS_BLOCK}: [{}]\n  in {AGENTS_BLOCK}, absent from {AGENTS_MD}: [{}]\n\n{AGENTS_MD} \
         is RENDERED from {AGENTS_BLOCK}. FIX: edit the marker in {AGENTS_BLOCK} and run \
         `.bee/bin/bee dev regen`; never hand-edit the rendered copy.",
        missing_from_source.join(" "),
        missing_from_rendered.join(" "),
    );
}

#[test]
fn the_agents_block_stays_thin_and_carries_no_rule_marker() {
    let block = read(AGENTS_BLOCK);
    assert!(
        block.chars().count() < THIN_BLOCK_LIMIT,
        "{AGENTS_BLOCK} is {} chars; the block every worker loads must stay under \
         {THIN_BLOCK_LIMIT}. FIX: move leader-only doctrine into {LEADER} or a playbook.",
        block.chars().count()
    );
    assert!(
        markers(&block).is_empty(),
        "{AGENTS_BLOCK} carries a rule marker; rule text lives in {LEADER} or {PLAYBOOKS_DIR}/"
    );
}

#[test]
fn every_rule_marker_is_homed_exactly_once_across_the_shipped_doctrine() {
    let found = rule_files();
    assert!(
        !found.is_empty(),
        "{LEADER} and {PLAYBOOKS_DIR}/ carry no `<!-- rule: <id> -->` marker at all, so this fence \
         would compare nothing"
    );
    let mut seen: std::collections::BTreeMap<&str, Vec<&str>> = Default::default();
    for (id, file) in &found {
        seen.entry(id.as_str()).or_default().push(file.as_str());
    }
    let duplicated: Vec<String> = seen
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(id, files)| format!("{id} in {}", files.join(", ")))
        .collect();
    assert!(
        duplicated.is_empty(),
        "rule marker(s) homed more than once: [{}]. A rule has exactly one home.",
        duplicated.join("; ")
    );
}

// ── every marked rule is indexed, and every indexed rule is marked ─────────

#[test]
fn every_marked_rule_has_an_index_row_naming_its_file_and_every_row_a_marker() {
    let found = rule_files();
    let marked: BTreeSet<String> = found.iter().map(|(id, _)| id.clone()).collect();
    let index = read(INDEX);
    let index_rows = rows(rule_homes_section(&index));
    let indexed: BTreeSet<String> = index_rows.iter().map(|r| r.id.clone()).collect();

    assert!(
        !indexed.is_empty(),
        "the {HEADING:?} section of {INDEX} holds no rule row, so this fence read nothing — a \
         check that cannot fail is the drift it is meant to catch"
    );

    let unindexed: Vec<&str> = marked.difference(&indexed).map(String::as_str).collect();
    let unmarked: Vec<&str> = indexed.difference(&marked).map(String::as_str).collect();

    assert!(
        unindexed.is_empty(),
        "rule(s) [{}] are marked in {LEADER} or {PLAYBOOKS_DIR}/ but have no row in {INDEX} § \
         {HEADING:?}.\n\nAn unindexed rule cannot be invoked by name: the index is where a \
         spoken id resolves to its file and its spoken form. FIX: add a row for each id above.",
        unindexed.join(" "),
    );
    assert!(
        unmarked.is_empty(),
        "rule(s) [{}] have a row in {INDEX} § {HEADING:?} but no `<!-- rule: <id> -->` marker in \
         {LEADER} or {PLAYBOOKS_DIR}/.\n\nThe row points at prose that no longer claims the id. \
         FIX: mark the rule and regen, or drop the stale row.",
        unmarked.join(" "),
    );

    let wrong_file: Vec<String> = found
        .iter()
        .filter_map(|(id, file)| {
            let row = index_rows.iter().find(|r| &r.id == id)?;
            (row.home.as_deref() != Some(file.as_str())).then(|| {
                format!("{id}: row names {:?}, marker is in {file}", row.home.as_deref().unwrap_or(""))
            })
        })
        .collect();
    assert!(
        wrong_file.is_empty(),
        "rule row(s) in {INDEX} name the wrong file:\n  {}\n\nFIX: put the marker's file, as \
         `(<path> § <section>)`, on each row.",
        wrong_file.join("\n  "),
    );
}

// ── every row says its rule out loud ───────────────────────────────────────

#[test]
fn every_index_row_carries_a_non_empty_spoken_line() {
    let index = read(INDEX);
    let rows = rows(rule_homes_section(&index));

    assert!(
        !rows.is_empty(),
        "the {HEADING:?} section of {INDEX} holds no rule row, so no spoken line was checked"
    );

    let silent: Vec<&str> = rows
        .iter()
        .filter(|r| r.spoken.as_deref().unwrap_or("").is_empty())
        .map(|r| r.id.as_str())
        .collect();

    assert!(
        silent.is_empty(),
        "rule row(s) [{}] in {INDEX} § {HEADING:?} carry no non-empty `spoken:` line before \
         their `applied_at:` line.\n\nThe spoken form is what lets a user invoke the rule in \
         their own words; without it the id is an annotation nobody can say. FIX: add one \
         indented `spoken: <the rule in plain words>` line to each row above, ABOVE its \
         `applied_at:` line — a `- ` sibling bullet would be parsed as an eleventh rule, and a \
         line under `applied_at:` would be parsed as a path.",
        silent.join(" "),
    );
}
