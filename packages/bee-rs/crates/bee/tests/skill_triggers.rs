use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const SKILLS_DIR: &str = "skills";
const SKILL_PREFIX: &str = "bee-";
const PRINCIPLE_PREFIX: &str = "bee-principle-";
const FIXTURE: &str = "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json";
const EVAL_ENV: &str = "BEE_SKILL_TRIGGER_EVAL";
const MIN_EXPECT: usize = 3;
const MIN_NEAR: usize = 2;
const EVAL_RUNS: usize = 3;
const EVAL_PASS_AT: usize = 2;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn front_matter(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn description(front: &str) -> Option<String> {
    let mut lines = front.lines();
    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix("description:") else { continue };
        let rest = rest.trim();
        if !rest.is_empty() && !rest.starts_with('>') && !rest.starts_with('|') {
            return Some(rest.trim_matches('"').trim_matches('\'').trim().to_string());
        }
        let mut folded: Vec<&str> = Vec::new();
        for cont in lines.by_ref() {
            if cont.trim().is_empty() {
                continue;
            }
            if !cont.starts_with(' ') && !cont.starts_with('\t') {
                break;
            }
            folded.push(cont.trim());
        }
        let joined = folded.join(" ");
        return if joined.is_empty() { None } else { Some(joined) };
    }
    None
}

fn skill_exists(name: &str) -> bool {
    repo_root().join(SKILLS_DIR).join(name).join("SKILL.md").is_file()
}

fn in_scope_skills() -> BTreeMap<String, String> {
    let dir = repo_root().join(SKILLS_DIR);
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot list {SKILLS_DIR}/: {e}"))
        .filter_map(Result::ok);

    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for entry in entries {
        if !entry.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let Ok(name) = entry.file_name().into_string() else { continue };
        if !name.starts_with(SKILL_PREFIX) || name.starts_with(PRINCIPLE_PREFIX) {
            continue;
        }
        let rel = format!("{SKILLS_DIR}/{name}/SKILL.md");
        let Ok(text) = std::fs::read_to_string(dir.join(&name).join("SKILL.md")) else { continue };
        let desc = front_matter(&text).and_then(description).unwrap_or_else(|| {
            panic!(
                "{rel} carries no readable `description:` in its front matter.\n\nThe description \
                 is the whole routing surface this fence guards: a skill without one is a skill \
                 no agent can route to. FIX: give it a `description:` line, as a quoted \
                 one-liner or a `>-` folded block."
            )
        });
        out.insert(name, desc);
    }

    assert!(
        !out.is_empty(),
        "no `{SKILLS_DIR}/{SKILL_PREFIX}*/` directory outside `{PRINCIPLE_PREFIX}*` carries a \
         SKILL.md, so this fence is checking an empty scope and cannot fail — the exact silence \
         it exists to break. FIX: restore the skills, or retire this fence with them."
    );
    out
}

struct Case {
    brief: String,
    expect: Vec<String>,
    near: Option<String>,
}

fn cases() -> Vec<Case> {
    let text = std::fs::read_to_string(repo_root().join(FIXTURE))
        .unwrap_or_else(|e| panic!("cannot read {FIXTURE}: {e}"));
    let value: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{FIXTURE} is not valid JSON: {e}"));
    let array = value
        .as_array()
        .unwrap_or_else(|| panic!("{FIXTURE} must hold a JSON array of trigger cases"));

    let mut out: Vec<Case> = Vec::new();
    for (index, item) in array.iter().enumerate() {
        let brief = item
            .get("brief")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("case {index} in {FIXTURE} has no string `brief`"))
            .to_string();
        let raw = item.get("expect").and_then(serde_json::Value::as_array).unwrap_or_else(|| {
            panic!("case {index} ({brief:?}) in {FIXTURE} has no `expect` array")
        });
        let expect: Vec<String> = raw
            .iter()
            .map(|v| {
                v.as_str()
                    .unwrap_or_else(|| {
                        panic!(
                            "case {index} ({brief:?}) in {FIXTURE} has a non-string entry in \
                             `expect`"
                        )
                    })
                    .to_string()
            })
            .collect();
        let near = match item.get("near") {
            None | Some(serde_json::Value::Null) => None,
            Some(v) => Some(
                v.as_str()
                    .unwrap_or_else(|| {
                        panic!("case {index} ({brief:?}) in {FIXTURE} has a non-string `near`")
                    })
                    .to_string(),
            ),
        };
        out.push(Case { brief, expect, near });
    }

    assert!(
        !out.is_empty(),
        "{FIXTURE} holds no case, so this fence read nothing and cannot fail. FIX: write the \
         trigger cases, or delete the fence with the fixture."
    );
    out
}

#[test]
fn skill_triggers_fixture_is_complete() {
    let scope = in_scope_skills();
    let cases = cases();
    let mut findings: Vec<String> = Vec::new();

    let mut briefs: BTreeMap<&str, usize> = BTreeMap::new();
    for case in &cases {
        *briefs.entry(case.brief.trim()).or_default() += 1;
    }
    for (brief, count) in &briefs {
        if *count > 1 {
            findings.push(format!(
                "{count} cases share the brief {brief:?} — one brief, one expectation"
            ));
        }
    }

    for case in &cases {
        let lowered = case.brief.to_lowercase();
        let named = case.expect.iter().map(String::as_str).chain(case.near.as_deref());
        for name in named {
            if !skill_exists(name) {
                findings.push(format!(
                    "brief {:?} names skill {name:?}, but {SKILLS_DIR}/{name}/SKILL.md does not \
                     exist",
                    case.brief
                ));
            }
            if lowered.contains(&name.to_lowercase()) {
                findings.push(format!(
                    "brief {:?} spells out the slug {name:?} of a skill it expects or nears — \
                     that brief tests string matching, not routing",
                    case.brief
                ));
            }
        }
    }

    let mut expects: BTreeMap<&str, usize> = BTreeMap::new();
    let mut nears: BTreeMap<&str, usize> = BTreeMap::new();
    for case in &cases {
        for name in &case.expect {
            *expects.entry(name.as_str()).or_default() += 1;
        }
        if let Some(near) = case.near.as_deref() {
            *nears.entry(near).or_default() += 1;
        }
    }
    for name in scope.keys() {
        let opened = expects.get(name.as_str()).copied().unwrap_or(0);
        let missed = nears.get(name.as_str()).copied().unwrap_or(0);
        if opened < MIN_EXPECT {
            findings.push(format!(
                "{name}: {opened} brief(s) expect it, {MIN_EXPECT} required — a description \
                 nobody wrote briefs for is untested"
            ));
        }
        if missed < MIN_NEAR {
            findings.push(format!(
                "{name}: {missed} near miss(es), {MIN_NEAR} required — a description with no \
                 near miss is never tested for over-triggering"
            ));
        }
    }

    assert!(
        findings.is_empty(),
        "{FIXTURE} does not fence every skill description:\n\n  {}\n\nEvery skill under \
         {SKILLS_DIR}/{SKILL_PREFIX}* outside {PRINCIPLE_PREFIX}* needs {MIN_EXPECT} briefs that \
         must open it and {MIN_NEAR} that must not. FIX: add or correct the cases above in \
         {FIXTURE}.",
        findings.join("\n  "),
    );
}

fn first_json_object(text: &str) -> Option<String> {
    let start = text.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in text[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(text[start..start + offset + ch.len_utf8()].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn ask(command: &str, prompt: &str) -> Option<Vec<String>> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(prompt.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let object = first_json_object(&stdout)?;
    let value: serde_json::Value = serde_json::from_str(&object).ok()?;
    let listed = value.get("skills")?.as_array()?;
    Some(
        listed
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(|s| s.trim().to_lowercase())
            .collect(),
    )
}

fn answer_is_right(case: &Case, answer: &[String]) -> bool {
    if let Some(near) = case.near.as_deref() {
        let near = near.to_lowercase();
        if answer.iter().any(|s| *s == near) {
            return false;
        }
    }
    if case.expect.is_empty() {
        return answer.is_empty();
    }
    case.expect.iter().any(|want| {
        let want = want.to_lowercase();
        answer.iter().any(|s| *s == want)
    })
}

#[test]
#[ignore]
fn skill_triggers_real_agent_eval() {
    let Some(command) = std::env::var(EVAL_ENV).ok().filter(|c| !c.trim().is_empty()) else {
        println!(
            "skipped: {EVAL_ENV} is unset (set it to an agent command such as `claude -p` to run \
             the paid eval)"
        );
        return;
    };

    let scope = in_scope_skills();
    let cases = cases();
    let catalog = scope
        .iter()
        .map(|(name, desc)| format!("{name}: {desc}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut scores: Vec<usize> = Vec::with_capacity(cases.len());
    for case in &cases {
        let prompt = format!(
            "You route user requests to skills. Skills (name: description):\n{catalog}\n\nRequest: \
             {}\n\nAnswer with only JSON: {{\"skills\": [...]}} naming the skills that should \
             open, or [] for none.",
            case.brief
        );
        let mut right = 0usize;
        for _ in 0..EVAL_RUNS {
            if ask(&command, &prompt).is_some_and(|answer| answer_is_right(case, &answer)) {
                right += 1;
            }
        }
        scores.push(right);
    }

    for name in scope.keys() {
        let mine: Vec<usize> = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| {
                case.expect.iter().any(|e| e == name) || case.near.as_deref() == Some(name.as_str())
            })
            .map(|(index, _)| index)
            .collect();
        let passed = mine.iter().filter(|index| scores[**index] >= EVAL_PASS_AT).count();
        println!("{name}: {passed}/{}", mine.len());
    }
    let passed = scores.iter().filter(|s| **s >= EVAL_PASS_AT).count();
    println!("total: {passed}/{}", scores.len());

    let never: Vec<String> = cases
        .iter()
        .zip(&scores)
        .filter(|(_, score)| **score == 0)
        .map(|(case, _)| format!("{:?}", case.brief))
        .collect();
    assert!(
        never.is_empty(),
        "the agent routed these brief(s) wrong in all {EVAL_RUNS} runs:\n\n  {}\n\nA brief no run \
         got right is a description the agent cannot route by, not a flaky sample. FIX: rewrite \
         the named skill's `description:`, or correct the brief if it asks for something else.",
        never.join("\n  "),
    );
}
