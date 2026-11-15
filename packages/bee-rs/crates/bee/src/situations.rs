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
