// Routed craft principles — the ONE reader of the `## Principle homes` index.
//
// expertise-principles D2: `bee orient` selects the principles from the task's
// route class, and the index is never always-loaded. Two surfaces show the
// selection — the session preamble beside its `Route:` line, and `bee orient`'s
// text block — so the class filter is written HERE, once, and both callers push
// the lines this module returns. A filter written twice is the
// rule-living-in-N-places trap the parity fence exists to catch.
//
// The repo's own concept file wins when it exists; otherwise the copy compiled
// from that same file at build time answers, so no row is ever hand-written
// here.
//
// Silence beats an empty header. No recorded route, a class matching zero rows,
// and an absent section all return an EMPTY vector — the
// callers push nothing at all, so a session that triggers no principle pays no
// bytes.

use std::path::Path;

/// The single home of the principle index — the same file `rule_index_parity.rs`
/// pins the AGENTS.md rule rows in, one section further down.
pub(crate) const PRINCIPLE_INDEX: &str =
    "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md";

const BUILT_IN_INDEX: &str = include_str!(
    "../../../../../docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"
);

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
    pub(crate) signals: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Hints {
    pub(crate) lane: Option<String>,
    pub(crate) flags: Vec<String>,
    pub(crate) files: Vec<String>,
}

impl Hints {
    pub(crate) fn from_route(route: &serde_json::Value) -> Self {
        let lane = route.get("lane").and_then(serde_json::Value::as_str).map(str::to_string);
        let flags = match route.get("flags") {
            Some(serde_json::Value::Array(items)) => {
                items.iter().filter_map(|f| f.as_str().map(str::to_string)).collect()
            }
            _ => Vec::new(),
        };
        Hints { lane, flags, files: Vec::new() }
    }

    fn matches(&self, token: &str) -> bool {
        let token = token.trim();
        if token.is_empty() {
            return false;
        }
        self.lane.as_deref().is_some_and(|l| l.eq_ignore_ascii_case(token))
            || self.flags.iter().any(|f| f.eq_ignore_ascii_case(token))
            || self.files.iter().any(|f| path_glob(token, f))
    }
}

fn path_glob(pattern: &str, path: &str) -> bool {
    fn go(p: &[u8], s: &[u8]) -> bool {
        match p.first() {
            None => s.is_empty(),
            Some(b'*') if p.get(1) == Some(&b'*') => {
                let rest = p[2..].strip_prefix(b"/").unwrap_or(&p[2..]);
                (0..=s.len()).any(|i| go(rest, &s[i..]))
            }
            Some(b'*') => {
                let mut i = 0;
                loop {
                    if go(&p[1..], &s[i..]) {
                        return true;
                    }
                    if i == s.len() || s[i] == b'/' {
                        return false;
                    }
                    i += 1;
                }
            }
            Some(c) => s.first() == Some(c) && go(&p[1..], &s[1..]),
        }
    }
    let path = path.trim_start_matches("./");
    if pattern.contains('/') {
        go(pattern.as_bytes(), path.as_bytes())
    } else {
        go(pattern.as_bytes(), path.rsplit('/').next().unwrap_or(path).as_bytes())
    }
}

pub(crate) const PRINCIPLES_CAP_DEFAULT: usize = 7;

pub(crate) fn principles_cap(root: &Path) -> usize {
    match crate::state::read_config_raw(root).get("principles_cap") {
        Some(serde_json::Value::Number(n)) => n
            .as_u64()
            .filter(|c| *c >= 1)
            .map_or(PRINCIPLES_CAP_DEFAULT, |c| c as usize),
        _ => PRINCIPLES_CAP_DEFAULT,
    }
}

pub(crate) fn all_principles(root: &Path) -> Vec<Principle> {
    parse_principles(&index_body(root))
}

fn index_body(root: &Path) -> String {
    std::fs::read_to_string(root.join(PRINCIPLE_INDEX))
        .unwrap_or_else(|_| BUILT_IN_INDEX.to_string())
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
    let mut signals: Vec<String> = Vec::new();

    // A row opens at column 0 (`- \`bee-principle-x\` (...)`); every
    // continuation — `spoken:`, `classes:`, and the `- applied_at:` block — is
    // indented, which is what keeps an applied_at bullet from reading as a new
    // row.
    let mut flush = |slug: &mut Option<String>,
                     spoken: &mut String,
                     classes: &mut Vec<String>,
                     signals: &mut Vec<String>| {
        if let Some(s) = slug.take()
            && !spoken.is_empty() && !classes.is_empty() {
                out.push(Principle {
                    slug: s,
                    spoken: std::mem::take(spoken),
                    classes: std::mem::take(classes),
                    signals: std::mem::take(signals),
                });
            }
        spoken.clear();
        classes.clear();
        signals.clear();
    };

    for line in section.lines() {
        let trimmed = line.trim();
        if line.starts_with("- ") || line.starts_with("* ") {
            flush(&mut slug, &mut spoken, &mut classes, &mut signals);
            slug = backticked(&trimmed[2..]).map(str::to_string);
        } else if let Some(value) = trimmed.strip_prefix("spoken:") {
            spoken = value.trim().to_string();
        } else if let Some(value) = trimmed.strip_prefix("classes:") {
            classes = value
                .split(',')
                .map(|c| c.trim().trim_matches('`').to_string())
                .filter(|c| !c.is_empty())
                .collect();
        } else if let Some(value) = trimmed.strip_prefix("signals:") {
            signals = value
                .split(',')
                .map(|c| c.trim().trim_matches('`').to_string())
                .filter(|c| !c.is_empty())
                .collect();
        }
    }
    flush(&mut slug, &mut spoken, &mut classes, &mut signals);

    out
}

/// The first backtick-quoted token of an index row, which is the skill slug.
fn backticked(item: &str) -> Option<&str> {
    let after = item.split_once('`')?.1;
    let (id, _) = after.split_once('`')?;
    let id = id.trim();
    (!id.is_empty()).then_some(id)
}

pub(crate) fn route_is_session_owned(session_bound: bool, lane_records: usize) -> bool {
    session_bound || lane_records == 0
}

/// The block every surface shows, or nothing at all.
///
/// `class` is the recorded route class. `None`, an empty class, a missing section, and zero matching rows all return an empty vector —
/// an empty header is worse than silence. Rows rank by signal matches against `hints`, then fewer classes, then file order, and at most `principles_cap` print.
pub(crate) fn principle_lines(root: &Path, class: Option<&str>, hints: &Hints) -> Vec<String> {
    let class = class.map(str::trim).filter(|c| !c.is_empty());
    let Some(class) = class else {
        return Vec::new();
    };
    render_lines(
        &index_body(root),
        class,
        crate::verbs::skillstation_group::principles_on_hub(root),
        hints,
        principles_cap(root),
    )
}

/// `principle_lines` with the index text already in hand — the seam the unit
/// tests drive, so they never need a repo tree on disk.
fn ranked_rows(body: &str, class: &str, hints: &Hints) -> Vec<Principle> {
    let mut matched: Vec<(usize, Principle)> = parse_principles(body)
        .into_iter()
        .filter(|p| p.classes.iter().any(|c| c.eq_ignore_ascii_case(class)))
        .map(|p| (p.signals.iter().filter(|t| hints.matches(t)).count(), p))
        .collect();
    matched.sort_by(|(a, pa), (b, pb)| b.cmp(a).then(pa.classes.len().cmp(&pb.classes.len())));
    matched.into_iter().map(|(_, p)| p).collect()
}

pub(crate) fn routed_slugs(root: &Path, class: Option<&str>, hints: &Hints) -> Vec<String> {
    let Some(class) = class.map(str::trim).filter(|c| !c.is_empty()) else {
        return Vec::new();
    };
    ranked_rows(&index_body(root), class, hints).into_iter().map(|p| p.slug).collect()
}

fn render_lines(body: &str, class: &str, hub: bool, hints: &Hints, cap: usize) -> Vec<String> {
    let matched = ranked_rows(body, class, hints);
    if matched.is_empty() {
        return Vec::new();
    }
    let mut header =
        format!("- Principles (class={class}) — name each one you apply and the decision it changed:");
    if hub {
        header.push_str(" Load each one with skillstation skill show <slug> --json (its content field); the skillstation MCP tool skill_get, the slug as skill_id, is the fallback.");
    }
    let mut lines = vec![header];
    let shown = matched.len().min(cap.max(1));
    lines.extend(
        matched[..shown].iter().map(|p| format!("  - `{}` — {}", p.slug, p.spoken)),
    );
    if shown < matched.len() {
        let rest: Vec<String> = matched[shown..].iter().map(|p| format!("`{}`", p.slug)).collect();
        lines.push(format!(
            "  - {} more of this class, ranked lower and still yours to apply by name: {}",
            rest.len(),
            rest.join(", ")
        ));
    }
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
        let lines = render_lines(INDEX, "bugfix", false, &Hints::default(), 7);
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
    fn hub_mode_extends_only_the_header_line() {
        let plain = render_lines(INDEX, "bugfix", false, &Hints::default(), 7);
        let hub = render_lines(INDEX, "bugfix", true, &Hints::default(), 7);
        assert_eq!(
            hub[0],
            format!("{} Load each one with skillstation skill show <slug> --json (its content field); the skillstation MCP tool skill_get, the slug as skill_id, is the fallback.", plain[0])
        );
        assert_eq!(hub[1..], plain[1..]);
    }

    #[test]
    fn principle_lines_reads_hub_mode_from_the_repo_config() {
        let dir = tempfile::tempdir().unwrap();
        let idx = dir.path().join(PRINCIPLE_INDEX);
        std::fs::create_dir_all(idx.parent().unwrap()).unwrap();
        std::fs::write(&idx, INDEX).unwrap();
        let plain = principle_lines(dir.path(), Some("bugfix"), &Hints::default());
        assert!(!plain[0].contains("skill_get"));
        std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
        std::fs::write(
            dir.path().join(".bee/config.json"),
            r#"{"skillstation":{"principles":"hub"}}"#,
        )
        .unwrap();
        assert!(principle_lines(dir.path(), Some("bugfix"), &Hints::default())[0].contains("skill_get"));
    }

    #[test]
    fn hub_header_names_the_skillstation_cli_before_skill_get_and_local_names_neither() {
        let dir = tempfile::tempdir().unwrap();
        let idx = dir.path().join(PRINCIPLE_INDEX);
        std::fs::create_dir_all(idx.parent().unwrap()).unwrap();
        std::fs::write(&idx, INDEX).unwrap();
        let local = principle_lines(dir.path(), Some("bugfix"), &Hints::default());
        assert!(!local[0].contains("skillstation skill show"), "{}", local[0]);
        assert!(!local[0].contains("skill_get"), "{}", local[0]);
        std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
        std::fs::write(dir.path().join(".bee/config.json"), r#"{"skillstation":{"principles":"hub"}}"#).unwrap();
        let hub = principle_lines(dir.path(), Some("bugfix"), &Hints::default());
        let cli = hub[0].find("skillstation skill show").expect("hub header lacks the CLI load");
        let mcp = hub[0].find("skill_get").expect("hub header lacks the skill_get fallback");
        assert!(cli < mcp, "CLI at {cli} must precede skill_get at {mcp}: {}", hub[0]);
        assert_eq!(hub[1..], local[1..]);
    }

    #[test]
    fn a_row_listing_several_classes_matches_every_one_of_them() {
        for class in ["feature", "bugfix", "refactor"] {
            let lines = render_lines(INDEX, class, false, &Hints::default(), 7);
            assert!(
                lines.iter().any(|l| l.contains("bee-principle-red-before-green")),
                "class {class} lost the three-class row: {lines:?}"
            );
        }
        // `refactor` is the only class both rows carry.
        assert_eq!(render_lines(INDEX, "refactor", false, &Hints::default(), 7).len(), 3);
        assert_eq!(render_lines(INDEX, "feature", false, &Hints::default(), 7).len(), 2);
    }

    #[test]
    fn a_class_matching_zero_rows_emits_no_block_at_all() {
        assert!(render_lines(INDEX, "docs", false, &Hints::default(), 7).is_empty());
    }

    #[test]
    fn a_missing_section_emits_no_block_at_all() {
        assert!(parse_principles("# Concept\n\n## Open Gaps\n\n- nothing here\n").is_empty());
        assert!(render_lines("# Concept\n\n## Open Gaps\n\n- nothing\n", "feature", false, &Hints::default(), 7).is_empty());
    }

    #[test]
    fn no_route_recorded_emits_no_block_at_all() {
        let root = std::env::temp_dir();
        assert!(principle_lines(&root, None, &Hints::default()).is_empty());
        assert!(principle_lines(&root, Some(""), &Hints::default()).is_empty());
        assert!(principle_lines(&root, Some("   "), &Hints::default()).is_empty());
    }

    #[test]
    fn a_repo_without_the_index_file_gets_the_built_in_copy() {
        let dir = tempfile::tempdir().unwrap();
        let lines = principle_lines(dir.path(), Some("bugfix"), &Hints::default());
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("bee-principle-red-before-green")), "{lines:?}");
        assert!(!lines[0].contains("skill_get"));
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
