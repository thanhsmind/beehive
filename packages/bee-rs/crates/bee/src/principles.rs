// Routed craft principles — the ONE reader of the `## Principle homes` index.
//
// expertise-principles D2: `bee orient` selects the principles from the task's
// route class, and the index is never always-loaded. Two surfaces show the
// selection — the session preamble beside its `Route:` line, and `bee orient`'s
// text block — so the class filter is written HERE, once, and both callers push
// the lines this module returns. A filter written twice is the
// rule-living-in-N-places trap the parity fence exists to catch.
//
// The index is read on demand from the concept file. It is never inlined as a
// constant here: a Rust copy of the rows would drift from the markdown home the
// moment somebody edits one and not the other, and the whole point of the
// section is that it is the single home.
//
// Silence beats an empty header. No recorded route, a class matching zero rows,
// an absent section, and an unreadable file all return an EMPTY vector — the
// callers push nothing at all, so a session that triggers no principle pays no
// bytes.

use std::path::Path;

/// The single home of the principle index — the same file `rule_index_parity.rs`
/// pins the AGENTS.md rule rows in, one section further down.
pub(crate) const PRINCIPLE_INDEX: &str =
    "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md";

/// The exact heading the section is found by. Bounded at the next `\n## `, the
/// same way `parse_agents_rule_homes` bounds its own section.
const HEADING: &str = "## Principle homes";

/// One row of the index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Principle {
    /// The skill slug, e.g. `bee-principle-red-before-green`.
    pub(crate) slug: String,
    /// The line a reader can say out loud to redirect the work.
    pub(crate) spoken: String,
    /// Route classes this principle fires for.
    pub(crate) classes: Vec<String>,
}

/// Rows of the `## Principle homes` section, in file order. A row with no
/// `spoken:` line or no `classes:` line is skipped: it can neither be spoken nor
/// routed, so it is not yet a principle.
pub(crate) fn parse_principles(body: &str) -> Vec<Principle> {
    let Some(start) = body.find(HEADING) else {
        return Vec::new();
    };
    let rest = &body[start + HEADING.len()..];
    let section = match rest.find("\n## ") {
        Some(end) => &rest[..end],
        None => rest,
    };

    let mut out: Vec<Principle> = Vec::new();
    let mut slug: Option<String> = None;
    let mut spoken = String::new();
    let mut classes: Vec<String> = Vec::new();

    // A row opens at column 0 (`- \`bee-principle-x\` (...)`); every
    // continuation — `spoken:`, `classes:`, and the `- applied_at:` block — is
    // indented, which is what keeps an applied_at bullet from reading as a new
    // row.
    let mut flush = |slug: &mut Option<String>, spoken: &mut String, classes: &mut Vec<String>| {
        if let Some(s) = slug.take() {
            if !spoken.is_empty() && !classes.is_empty() {
                out.push(Principle {
                    slug: s,
                    spoken: std::mem::take(spoken),
                    classes: std::mem::take(classes),
                });
            }
        }
        spoken.clear();
        classes.clear();
    };

    for line in section.lines() {
        let trimmed = line.trim();
        if line.starts_with("- ") || line.starts_with("* ") {
            flush(&mut slug, &mut spoken, &mut classes);
            slug = backticked(&trimmed[2..]).map(str::to_string);
        } else if let Some(value) = trimmed.strip_prefix("spoken:") {
            spoken = value.trim().to_string();
        } else if let Some(value) = trimmed.strip_prefix("classes:") {
            classes = value
                .split(',')
                .map(|c| c.trim().trim_matches('`').to_string())
                .filter(|c| !c.is_empty())
                .collect();
        }
    }
    flush(&mut slug, &mut spoken, &mut classes);

    out
}

/// The first backtick-quoted token of an index row, which is the skill slug.
fn backticked(item: &str) -> Option<&str> {
    let after = item.split_once('`')?.1;
    let (id, _) = after.split_once('`')?;
    let id = id.trim();
    (!id.is_empty()).then_some(id)
}

/// The block both surfaces show, or nothing at all.
///
/// `class` is the recorded route class. `None`, an empty class, a missing index
/// file, a missing section, and zero matching rows all return an empty vector —
/// an empty header is worse than silence.
pub(crate) fn principle_lines(root: &Path, class: Option<&str>) -> Vec<String> {
    let class = class.map(str::trim).filter(|c| !c.is_empty());
    let Some(class) = class else {
        return Vec::new();
    };
    let Ok(body) = std::fs::read_to_string(root.join(PRINCIPLE_INDEX)) else {
        return Vec::new();
    };
    render_lines(&body, class)
}

/// `principle_lines` with the index text already in hand — the seam the unit
/// tests drive, so they never need a repo tree on disk.
fn render_lines(body: &str, class: &str) -> Vec<String> {
    let matched: Vec<Principle> = parse_principles(body)
        .into_iter()
        .filter(|p| p.classes.iter().any(|c| c.eq_ignore_ascii_case(class)))
        .collect();
    if matched.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![format!(
        "- Principles (class={class}) — name each one you apply and the decision it changed:"
    )];
    lines.extend(
        matched.iter().map(|p| format!("  - `{}` — {}", p.slug, p.spoken)),
    );
    lines
}

/// The index shape every surface's tests drive. It lives at module level, not
/// inside `mod tests`, because the two CALLERS test against it too — one
/// fixture, so a caller's test can never pass against a row shape this parser
/// never sees.
#[cfg(test)]
pub(crate) const TEST_INDEX: &str = "\
# Concept

## Principle homes

Prose the parser walks past.

- `bee-principle-red-before-green` (expertise/tests.md § Red before green):
  spoken: watch it fail for the reported reason before you fix it
  classes: feature, bugfix, refactor
  - applied_at:
    - `skills/bee-principle-red-before-green/SKILL.md`
- `bee-principle-one-home` (expertise/architecture.md § One home):
  spoken: a rule lives in one place and every other surface points at it
  classes: refactor
  - applied_at:
    - `skills/bee-principle-one-home/SKILL.md`

## Open Gaps

- `bee-principle-not-a-principle` (past the section boundary):
  spoken: never read
  classes: feature
";

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const INDEX: &str = TEST_INDEX;

    #[test]
    fn a_class_with_matches_names_each_principle_and_its_spoken_line() {
        let lines = render_lines(INDEX, "bugfix");
        assert_eq!(
            lines,
            vec![
                "- Principles (class=bugfix) — name each one you apply and the decision it changed:"
                    .to_string(),
                "  - `bee-principle-red-before-green` — watch it fail for the reported reason before you fix it"
                    .to_string(),
            ]
        );
    }

    #[test]
    fn a_row_listing_several_classes_matches_every_one_of_them() {
        for class in ["feature", "bugfix", "refactor"] {
            let lines = render_lines(INDEX, class);
            assert!(
                lines.iter().any(|l| l.contains("bee-principle-red-before-green")),
                "class {class} lost the three-class row: {lines:?}"
            );
        }
        // `refactor` is the only class both rows carry.
        assert_eq!(render_lines(INDEX, "refactor").len(), 3);
        assert_eq!(render_lines(INDEX, "feature").len(), 2);
    }

    #[test]
    fn a_class_matching_zero_rows_emits_no_block_at_all() {
        assert!(render_lines(INDEX, "docs").is_empty());
    }

    #[test]
    fn a_missing_section_emits_no_block_at_all() {
        assert!(parse_principles("# Concept\n\n## Open Gaps\n\n- nothing here\n").is_empty());
        assert!(render_lines("# Concept\n\n## Open Gaps\n\n- nothing\n", "feature").is_empty());
    }

    #[test]
    fn no_route_recorded_emits_no_block_at_all() {
        let root = std::env::temp_dir();
        assert!(principle_lines(&root, None).is_empty());
        assert!(principle_lines(&root, Some("")).is_empty());
        assert!(principle_lines(&root, Some("   ")).is_empty());
    }

    #[test]
    fn a_missing_index_file_emits_no_block_at_all() {
        let root = std::env::temp_dir().join("bee-principles-absent-root");
        assert!(principle_lines(&root, Some("feature")).is_empty());
    }

    /// The section boundary is the next `## `, exactly where the fence stops.
    #[test]
    fn rows_past_the_section_boundary_are_not_read() {
        let slugs: Vec<String> = parse_principles(INDEX).into_iter().map(|p| p.slug).collect();
        assert_eq!(slugs, vec!["bee-principle-red-before-green", "bee-principle-one-home"]);
    }

    /// This repo's own index must parse — a fixture that cannot diverge from
    /// the real file proves nothing about the real file.
    #[test]
    fn the_repos_own_index_parses_into_rows() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf();
        let body = std::fs::read_to_string(root.join(PRINCIPLE_INDEX))
            .unwrap_or_else(|e| panic!("cannot read {PRINCIPLE_INDEX}: {e}"));
        let rows = parse_principles(&body);
        assert!(!rows.is_empty(), "{PRINCIPLE_INDEX} carries no parseable principle rows");
        for row in &rows {
            assert!(row.slug.starts_with("bee-principle-"), "row slug is not a skill slug: {row:?}");
            assert!(!row.spoken.is_empty(), "row has no spoken line: {row:?}");
            assert!(!row.classes.is_empty(), "row has no classes: {row:?}");
        }
    }
}
