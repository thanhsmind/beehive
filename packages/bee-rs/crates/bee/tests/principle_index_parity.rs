// The principle-index parity fence.
//
// A craft principle lives in THREE places: a `skills/principle-<slug>/`
// directory that ships it, one row in the `## Principle homes` section of the
// doctrine-layer concept, and the `expertise/` guide section that holds its
// depth. `principles.rs` reads the rows and hands them to `bee orient` and the
// session preamble — silently. Every failure mode of that reader is quiet: a
// row whose skill was never shipped, a skill nobody indexed, a `classes:` value
// that is not a route class, a missing `spoken:` line, an anchor pointing at a
// heading that moved. Each one costs the agent a principle it never learns it
// lost.
//
// Four things are pinned here.
//
//   1. Every `skills/principle-*/` directory has exactly one row, and every row
//      names a directory that exists — both directions.
//   2. Every value on a row's `classes:` line is a real route class, read from
//      `ROUTE_CLASS_VALUES` in `workflows.rs`. A class value nobody routes is a
//      row `bee orient` can never select.
//   3. Every row carries a non-empty `spoken:` line BEFORE its `applied_at:`
//      line. Order is not decoration: `parse_agents_rule_homes` reads the
//      indented block after `applied_at:` as paths, so a line placed there is
//      parsed as one.
//   4. Every row's guide anchor names a file under `expertise/` that exists and
//      a heading that appears in it — the pointer the skill's depth rests on.
//
// Shape, deliberately: pure filesystem, std only, and NOTHING imported from the
// bee crate — the model is `rule_index_parity.rs` and `route_class_parity.rs`.
// This file declares no list it guards: the principle names come from the
// `skills/` listing, the class vocabulary is read out of `workflows.rs` as TEXT
// (`ROUTE_CLASS_VALUES` is `pub(crate)` and invisible to an integration test).
// A fence holding its own copy of the truth it guards agrees with itself
// forever and catches nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// The single home of the principle index — the file `principles.rs` reads.
const INDEX: &str =
    "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md";

/// The exact heading `parse_principles`
/// (`packages/bee-rs/crates/bee/src/principles.rs`) finds the section by. Both
/// it and this fence stop at the next `\n## `, so the two cannot disagree about
/// where the section ends.
const HEADING: &str = "## Principle homes";

/// Where the principle skills ship from, and the slug prefix that marks one.
const SKILLS_DIR: &str = "skills";
const SLUG_PREFIX: &str = "principle-";

/// The single home of the route-class vocabulary, read as text.
const WORKFLOWS_RS: &str = "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs";

/// Where a principle's depth lives (`expertise-principles` D1).
const GUIDE_DIR: &str = "expertise";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

// ── the index rows ─────────────────────────────────────────────────────────

struct Row {
    slug: String,
    /// The parenthetical after the slug, e.g. `expertise/tests.md § Heading`.
    /// `None` when the row opened without one.
    anchor: Option<String>,
    /// The `spoken:` text found BEFORE this row's `applied_at:` line.
    spoken: Option<String>,
    classes: Vec<String>,
}

/// A row opens at column 0 with ``- `<slug>` ``, the same bullet
/// `parse_principles` treats as the start of a new row; every continuation is
/// indented, which is what keeps an `applied_at:` bullet from reading as a row.
fn row_open(line: &str) -> Option<(String, Option<String>)> {
    let rest = line.strip_prefix("- `")?;
    let (slug, rest) = rest.split_once('`')?;
    let slug = slug.trim();
    if slug.is_empty() {
        return None;
    }
    let rest = rest.trim();
    let anchor = rest
        .strip_prefix('(')
        .and_then(|inner| inner.rfind(')').map(|close| inner[..close].trim().to_string()));
    Some((slug.to_string(), anchor))
}

/// The principle-homes section only, bounded the way `parse_principles` bounds
/// it.
fn section(text: &str) -> &str {
    let start = text.find(HEADING).unwrap_or_else(|| {
        panic!(
            "{INDEX} no longer carries the heading {HEADING:?}.\n\n`parse_principles` finds the \
             section by that exact string, so losing it empties the router's principle selection \
             AND this fence in silence. FIX: restore the heading, or move the index and point \
             both this fence and that parser at its new home."
        )
    });
    let rest = &text[start + HEADING.len()..];
    match rest.find("\n## ") {
        Some(end) => &rest[..end],
        None => rest,
    }
}

fn rows(section: &str) -> Vec<Row> {
    let mut out: Vec<Row> = Vec::new();
    let mut in_applied_at = false;

    for line in section.lines() {
        if let Some((slug, anchor)) = row_open(line) {
            out.push(Row { slug, anchor, spoken: None, classes: Vec::new() });
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
            } else if let Some(classes) = trimmed.strip_prefix("classes:") {
                row.classes = classes
                    .split(',')
                    .map(|c| c.trim().trim_matches('`').to_string())
                    .filter(|c| !c.is_empty())
                    .collect();
            }
        }
    }
    out
}

/// The rows of this repo's index, with the "the fence read nothing" guard every
/// test below shares.
fn index_rows() -> Vec<Row> {
    let text = read(INDEX);
    let rows = rows(section(&text));
    assert!(
        !rows.is_empty(),
        "the {HEADING:?} section of {INDEX} holds no principle row, so this fence read nothing — \
         a check that cannot fail is the drift it is meant to catch"
    );
    rows
}

// ── the shipped skills ─────────────────────────────────────────────────────

/// Every `skills/principle-*/` directory name. This listing IS the principle
/// set; the fence states no set of its own.
fn shipped_slugs() -> BTreeSet<String> {
    let dir = repo_root().join(SKILLS_DIR);
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot list {SKILLS_DIR}/: {e}"))
        .filter_map(Result::ok);

    let out: BTreeSet<String> = entries
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.starts_with(SLUG_PREFIX))
        .collect();

    assert!(
        !out.is_empty(),
        "no `{SKILLS_DIR}/{SLUG_PREFIX}*/` directory exists, so this fence is comparing against \
         an empty set and cannot fail — the exact silence it exists to break. FIX: ship the \
         principle skills, or retire the {HEADING:?} index with them."
    );
    out
}

// ── the class vocabulary, read as text ─────────────────────────────────────

/// Read `const <name>: [&str; N] = [ ... ];` out of Rust source TEXT.
///
/// The declared arity `N` is checked against what was found: if this parser
/// ever grabs the wrong bracket pair, the count disagrees and the test says so
/// instead of fencing against a half-read list.
fn const_str_array(src: &str, name: &str) -> BTreeSet<String> {
    let decl = format!("const {name}: [&str; ");
    let at = src.find(&decl).unwrap_or_else(|| {
        panic!(
            "`{name}` is no longer declared as `{decl}...` in {WORKFLOWS_RS}.\n\nThis fence reads \
             that constant as text because it is pub(crate) and cannot be imported. FIX: point \
             this parser at the new declaration — do NOT paste the values in here."
        )
    });
    let rest = &src[at + decl.len()..];

    let arity_end = rest.find(']').expect("the `[&str; N]` type is unterminated");
    let arity: usize = rest[..arity_end]
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("`{name}` has a non-numeric arity in its type"));

    let eq = rest.find('=').expect("the declaration has no `=`");
    let open = eq + rest[eq..].find('[').expect("the initializer has no `[`");
    let close = open + rest[open..].find(']').expect("the initializer has no `]`");

    let values: BTreeSet<String> = rest[open + 1..close]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    assert_eq!(
        values.len(),
        arity,
        "read {} distinct value(s) out of `{name}` but its type declares {arity} — this fence \
         misparsed the source and would guard the wrong list",
        values.len()
    );
    values
}

// ── every skill is indexed, and every row is shipped ───────────────────────

#[test]
fn every_principle_skill_has_exactly_one_row_and_every_row_a_skill() {
    let shipped = shipped_slugs();

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in index_rows() {
        *counts.entry(row.slug).or_default() += 1;
    }

    let duplicated: Vec<&str> =
        counts.iter().filter(|(_, n)| **n > 1).map(|(slug, _)| slug.as_str()).collect();
    assert!(
        duplicated.is_empty(),
        "principle(s) [{}] carry more than one row in {INDEX} § {HEADING:?}.\n\n`bee orient` \
         would name the principle twice, and an edit to one row would leave the other stale. \
         FIX: merge each set of rows above into one.",
        duplicated.join(" "),
    );

    let indexed: BTreeSet<String> = counts.into_keys().collect();
    let unindexed: Vec<&str> = shipped.difference(&indexed).map(String::as_str).collect();
    let unshipped: Vec<&str> = indexed.difference(&shipped).map(String::as_str).collect();

    assert!(
        unindexed.is_empty(),
        "principle skill(s) [{}] exist under {SKILLS_DIR}/ but have no row in {INDEX} § \
         {HEADING:?}.\n\nThe index is the ONLY thing `bee orient` reads, so an unindexed skill \
         is never routed to anybody — it ships and is never selected. FIX: add a row for each \
         slug above.",
        unindexed.join(" "),
    );
    assert!(
        unshipped.is_empty(),
        "row(s) [{}] in {INDEX} § {HEADING:?} name no `{SKILLS_DIR}/<slug>/` directory.\n\n`bee \
         orient` would name a skill the agent cannot load. FIX: ship the skill, or drop the \
         stale row.",
        unshipped.join(" "),
    );
}

// ── every row routes to real classes ───────────────────────────────────────

#[test]
fn every_row_routes_only_to_known_class_values() {
    let known = const_str_array(&read(WORKFLOWS_RS), "ROUTE_CLASS_VALUES");
    let rows = index_rows();

    let classless: Vec<&str> =
        rows.iter().filter(|r| r.classes.is_empty()).map(|r| r.slug.as_str()).collect();
    assert!(
        classless.is_empty(),
        "row(s) [{}] in {INDEX} § {HEADING:?} carry no `classes:` line before their `applied_at:` \
         line.\n\n`parse_principles` skips a row with no classes, so the principle is never \
         routed and this fence has nothing to check against the enum. FIX: add one indented \
         `classes: <comma-separated route classes>` line to each row above, ABOVE its \
         `applied_at:` line.",
        classless.join(" "),
    );

    let unknown: Vec<String> = rows
        .iter()
        .flat_map(|r| {
            r.classes
                .iter()
                .filter(|c| !known.contains(*c))
                .map(move |c| format!("{}: {c}", r.slug))
        })
        .collect();

    assert!(
        unknown.is_empty(),
        "row(s) in {INDEX} § {HEADING:?} name class value(s) that are not in \
         `ROUTE_CLASS_VALUES` ({WORKFLOWS_RS}):\n\n  {}\n\nThe enum holds: {}\n\nA class the \
         router never computes can never match, so the principle is silently unroutable. FIX: \
         correct the value(s) above; adding a class to the enum is a separate, deliberate \
         change.",
        unknown.join("\n  "),
        known.iter().cloned().collect::<Vec<_>>().join(" "),
    );
}

// ── every row says its principle out loud ──────────────────────────────────

#[test]
fn every_row_carries_a_non_empty_spoken_line_before_its_applied_at() {
    let silent: Vec<String> = index_rows()
        .iter()
        .filter(|r| r.spoken.as_deref().unwrap_or("").is_empty())
        .map(|r| r.slug.clone())
        .collect();

    assert!(
        silent.is_empty(),
        "principle row(s) [{}] in {INDEX} § {HEADING:?} carry no non-empty `spoken:` line before \
         their `applied_at:` line.\n\nThe spoken line is what `bee orient` prints beside the \
         slug, and what a user says to invoke the principle mid-run; without it the row routes a \
         name with no rule. FIX: add one indented `spoken: <the principle in plain words>` line \
         to each row above, ABOVE its `applied_at:` line — a `- ` sibling bullet would be parsed \
         as another principle, and a line under `applied_at:` would be parsed as a path.",
        silent.join(" "),
    );
}

// ── every row anchors a guide section that exists ──────────────────────────

#[test]
fn every_row_anchors_an_existing_guide_heading() {
    let root = repo_root();
    let mut broken: Vec<String> = Vec::new();

    for row in index_rows() {
        let slug = &row.slug;
        let Some(anchor) = row.anchor.as_deref() else {
            broken.push(format!(
                "{slug}: the row opens with no `(<guide> § <heading>)` anchor at all"
            ));
            continue;
        };
        let Some((rel, heading)) = anchor.split_once('§') else {
            broken.push(format!("{slug}: anchor {anchor:?} has no `§` separating guide from \
                                 heading"));
            continue;
        };
        let (rel, heading) = (rel.trim().trim_matches('`'), heading.trim());

        if !rel.starts_with(&format!("{GUIDE_DIR}/")) {
            broken.push(format!("{slug}: anchor points at {rel:?}, which is not under \
                                 {GUIDE_DIR}/"));
            continue;
        }
        let Ok(guide) = std::fs::read_to_string(root.join(rel)) else {
            broken.push(format!("{slug}: guide {rel} does not exist"));
            continue;
        };
        if heading.is_empty() {
            broken.push(format!("{slug}: anchor names {rel} but no heading after the `§`"));
            continue;
        }
        let found = guide
            .lines()
            .filter(|l| l.starts_with('#'))
            .any(|l| l.trim_start_matches('#').trim() == heading);
        if !found {
            broken.push(format!("{slug}: {rel} carries no heading {heading:?}"));
        }
    }

    assert!(
        broken.is_empty(),
        "principle row(s) in {INDEX} § {HEADING:?} anchor a guide section that is not \
         there:\n\n  {}\n\nThe anchor is where the principle's depth lives \
         (`expertise-principles` D1): the skill is a handle, the guide section is the rule. A \
         broken anchor strands the depth. FIX: repoint each anchor above at the heading's \
         current spelling, or restore the heading in the guide.",
        broken.join("\n  "),
    );
}
