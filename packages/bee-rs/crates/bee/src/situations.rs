use std::path::Path;

const HEADING: &str = "## Situations";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Situation {
    pub(crate) name: String,
    pub(crate) triggers: String,
    pub(crate) deliverable: String,
}

const BUILT_IN: [(&str, &str); 9] = [
    ("bugfix", include_str!("../../../../../skills/bee-planning/playbooks/bugfix.md")),
    ("content", include_str!("../../../../../skills/bee-planning/playbooks/content.md")),
    ("docs", include_str!("../../../../../skills/bee-planning/playbooks/docs.md")),
    ("feature", include_str!("../../../../../skills/bee-planning/playbooks/feature.md")),
    ("perf", include_str!("../../../../../skills/bee-planning/playbooks/perf.md")),
    ("refactor", include_str!("../../../../../skills/bee-planning/playbooks/refactor.md")),
    ("release", include_str!("../../../../../skills/bee-planning/playbooks/release.md")),
    ("research", include_str!("../../../../../skills/bee-planning/playbooks/research.md")),
    ("spike", include_str!("../../../../../skills/bee-planning/playbooks/spike.md")),
];

pub(crate) fn parse_situations(body: &str) -> Vec<Situation> {
    let Some(start) = body.find(HEADING) else {
        return Vec::new();
    };
    let rest = &body[start + HEADING.len()..];
    let section = match rest.find("\n## ") {
        Some(end) => &rest[..end],
        None => rest,
    };
    section.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<Situation> {
    let item = line.strip_prefix("- `")?;
    let (name, rest) = item.split_once('`')?;
    let valid = !name.is_empty()
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid {
        return None;
    }
    let rest = rest.strip_prefix(" — ")?;
    let (triggers, deliverable) = rest.split_once(" — delivers: ")?;
    let (triggers, deliverable) = (triggers.trim(), deliverable.trim());
    if triggers.is_empty() || deliverable.is_empty() {
        return None;
    }
    Some(Situation {
        name: name.to_string(),
        triggers: triggers.to_string(),
        deliverable: deliverable.to_string(),
    })
}

pub(crate) fn class_situations(root: &Path, class: &str) -> Vec<Situation> {
    let path = root.join(format!("skills/bee-planning/playbooks/{class}.md"));
    match std::fs::read_to_string(path) {
        Ok(body) => parse_situations(&body),
        Err(_) => BUILT_IN
            .iter()
            .find(|(c, _)| *c == class)
            .map(|(_, body)| parse_situations(body))
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, triggers: &str, deliverable: &str) -> Situation {
        Situation {
            name: name.to_string(),
            triggers: triggers.to_string(),
            deliverable: deliverable.to_string(),
        }
    }

    fn write_index(root: &Path, class: &str, body: &str) {
        let path = root.join(format!("skills/bee-planning/playbooks/{class}.md"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn a_well_formed_section_yields_one_row_per_line_and_stops_at_the_next_heading() {
        let body = "\
# Class

## Situations

- `bug-fix` — bug, lỗi — delivers: the root cause and the fix
- `slow-2` — slow, chậm — delivers: a measured speedup

## Steps

- `not-a-row` — after the section — delivers: nothing
";
        assert_eq!(
            parse_situations(body),
            vec![
                row("bug-fix", "bug, lỗi", "the root cause and the fix"),
                row("slow-2", "slow, chậm", "a measured speedup"),
            ]
        );
    }

    #[test]
    fn a_malformed_line_is_skipped_and_its_neighbours_still_parse() {
        let body = "\
## Situations

Prose the parser walks past.
- `Bad-Name` — upper case — delivers: x
- `` — empty name — delivers: x
- `no-delivers` — triggers only
- `empty-triggers` —  — delivers: x
- `empty-deliverable` — t — delivers: 
- no-backticks — t — delivers: x
- `good` — t — delivers: d
";
        assert_eq!(parse_situations(body), vec![row("good", "t", "d")]);
    }

    #[test]
    fn an_absent_section_yields_no_rows() {
        assert!(parse_situations("# Class\n\n## Steps\n\n- `x` — t — delivers: d\n").is_empty());
        assert!(parse_situations("").is_empty());
    }

    #[test]
    fn the_repo_copy_wins_over_the_compiled_in_copy() {
        let tmp = tempfile::tempdir().unwrap();
        write_index(tmp.path(), "bugfix", "## Situations\n\n- `local-only` — here — delivers: there\n");
        assert_eq!(class_situations(tmp.path(), "bugfix"), vec![row("local-only", "here", "there")]);
        write_index(tmp.path(), "bugfix", "# No section\n");
        assert!(class_situations(tmp.path(), "bugfix").is_empty());
    }

    #[test]
    fn a_repo_with_no_skills_tree_reads_the_compiled_in_copy() {
        let tmp = tempfile::tempdir().unwrap();
        let built_in = BUILT_IN.iter().find(|(c, _)| *c == "bugfix").unwrap().1;
        let expected = parse_situations(built_in);
        assert!(expected.iter().any(|s| s.name == "bug-fix"), "{expected:?}");
        assert_eq!(class_situations(tmp.path(), "bugfix"), expected);
        assert!(class_situations(tmp.path(), "no-such-class").is_empty());
    }
}
