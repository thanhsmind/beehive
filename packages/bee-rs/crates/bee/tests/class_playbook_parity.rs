// The class-PLAYBOOK parity fence.
//
// `route_class_parity.rs` pins the class VOCABULARY: the four documents that
// spell the eight values out all spell out the same eight. It never asks the
// next question — whether a class a plan can actually route to has a PROCEDURE
// to cite. It did not, for four of the eight, and nothing anywhere went red:
// `expertise-principles` routed as `class=feature`, found no playbook, and had
// to record a named deviation in place of the steps it should have cited.
//
// A vocabulary is a promise that each value means something. A value with no
// playbook breaks that promise SILENTLY — the router accepts it, the plan cites
// nothing, and the hole is visible only to whoever happens to read both files
// side by side.
//
// Two things are pinned here, both directions of the same map:
//
//   1. Every value in `ROUTE_CLASS_VALUES` has exactly one `<class>.md` file
//      in `skills/bee-planning/playbooks/`.
//   2. Every `.md` file in that directory names a real class value.
//
// Shape, deliberately: pure filesystem, std only, and NOTHING imported from the
// bee crate — the model is `route_class_parity.rs`, whose header explains why.
// `ROUTE_CLASS_VALUES` is `pub(crate)`, so an integration test cannot see it as
// a symbol; this file reads `workflows.rs` as TEXT instead.
//
// The one thing this file must never do is re-declare the class list. A fence
// holding its own private copy of the truth it guards agrees with itself
// forever and catches nothing.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The single home of the class vocabulary.
const WORKFLOWS_RS: &str = "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs";

const PLAYBOOKS_DIR: &str = "skills/bee-planning/playbooks";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel))
        .unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// Read `const <name>: [&str; N] = [ ... ];` out of Rust source TEXT.
///
/// The declared arity `N` is parsed too and checked against what was found: if
/// this parser ever silently grabs the wrong bracket pair, the count disagrees
/// and the test says so instead of fencing against a half-read list.
fn const_str_array(src: &str, name: &str) -> Vec<String> {
    let decl = format!("const {name}: [&str; ");
    let at = src.find(&decl).unwrap_or_else(|| {
        panic!(
            "`{name}` is no longer declared as `{decl}...` in {WORKFLOWS_RS}.\n\nFIX: this fence \
             reads that constant as text because it is pub(crate) and cannot be imported. If the \
             declaration moved or changed shape, point this parser at the new one — do NOT paste \
             the values in here."
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

    let values: Vec<String> = rest[open + 1..close]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    assert_eq!(
        values.len(),
        arity,
        "read {} value(s) out of `{name}` but its type declares {arity} — this fence misparsed \
         the source and would guard the wrong list",
        values.len()
    );
    values
}

fn playbook_names() -> Vec<String> {
    let dir = std::fs::read_dir(repo_root().join(PLAYBOOKS_DIR)).unwrap_or_else(|e| {
        panic!(
            "the directory {PLAYBOOKS_DIR:?} cannot be read: {e}.\n\nThat directory is where the \
             playbooks live and where a plan is told to cite them. FIX: restore it, or point this \
             fence at wherever the playbooks now live."
        )
    });

    let names: Vec<String> = dir
        .map(|entry| entry.unwrap_or_else(|e| panic!("cannot read an entry of {PLAYBOOKS_DIR}: {e}")))
        .filter_map(|entry| {
            entry.file_name().to_str().and_then(|n| n.strip_suffix(".md")).map(str::to_string)
        })
        .collect();

    assert!(
        !names.is_empty(),
        "no `<class>.md` file sits in {PLAYBOOKS_DIR:?}, so this fence would pass on an empty \
         playbook set"
    );
    names
}

// ── every class has a playbook, and every playbook has a class ─────────────

#[test]
fn every_route_class_has_exactly_one_playbook() {
    let classes = const_str_array(&read(WORKFLOWS_RS), "ROUTE_CLASS_VALUES");
    let names = playbook_names();

    let missing: Vec<&str> = classes
        .iter()
        .filter(|c| !names.contains(c))
        .map(|s| s.as_str())
        .collect();
    assert!(
        missing.is_empty(),
        "route class(es) [{}] have no `<class>.md` file in {PLAYBOOKS_DIR:?}.\n\nA plan routed to one of those classes is told to cite a playbook that \
         does not exist, so it cites nothing and the gap never shows up as a red.\n\nFIX: write \
         the missing playbook(s) in the voice of the ones already there — numbered ACTION steps, \
         then a closing line naming the thing that is NOT a result. Deleting the class from \
         `ROUTE_CLASS_VALUES` ({WORKFLOWS_RS}) also settles it; hiding the class from this fence \
         does not.",
        missing.join(" ")
    );

    let duplicated: BTreeSet<&str> = classes
        .iter()
        .filter(|c| names.iter().filter(|n| n == c).count() > 1)
        .map(|s| s.as_str())
        .collect();
    assert!(
        duplicated.is_empty(),
        "route class(es) [{}] carry MORE THAN ONE `<class>.md` file in {PLAYBOOKS_DIR:?}.\n\n\
         A class with two playbooks has none: a plan citing \"the\" playbook by name does not say \
         which steps it followed. FIX: merge them into one file.",
        duplicated.into_iter().collect::<Vec<_>>().join(" ")
    );
}

#[test]
fn every_playbook_names_a_real_route_class() {
    let classes: BTreeSet<String> =
        const_str_array(&read(WORKFLOWS_RS), "ROUTE_CLASS_VALUES").into_iter().collect();

    let orphans: Vec<String> =
        playbook_names().into_iter().filter(|n| !classes.contains(n)).collect();
    assert!(
        orphans.is_empty(),
        "playbook file(s) [{}] in {PLAYBOOKS_DIR:?} name no value in `ROUTE_CLASS_VALUES` \
         ({WORKFLOWS_RS}).\n\nNo route can reach those steps, so nobody reads them and nobody \
         notices when they rot.\n\nFIX: rename the file to the `<class>.md` it means, add the \
         class to the constant, or delete the file.",
        orphans.join(" ")
    );
}

const SITUATIONS_HEADING: &str = "## Situations";
const SESSION_DIR: &str = "session";

fn situations_section(body: &str) -> Option<&str> {
    let start = body.find(SITUATIONS_HEADING)?;
    let rest = &body[start + SITUATIONS_HEADING.len()..];
    Some(match rest.find("\n## ") {
        Some(end) => &rest[..end],
        None => rest,
    })
}

fn situation_line_name(line: &str) -> Option<&str> {
    let item = line.strip_prefix("- `")?;
    let (name, rest) = item.split_once('`')?;
    let named = !name.is_empty()
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    let (triggers, deliverable) = rest.strip_prefix(" — ")?.split_once(" — delivers: ")?;
    (named && !triggers.trim().is_empty() && !deliverable.trim().is_empty()).then_some(name)
}

fn listed_situations(index: &Path, findings: &mut Vec<String>) -> BTreeSet<String> {
    let label = index.display();
    let Ok(body) = std::fs::read_to_string(index) else {
        findings.push(format!("{label}: cannot be read, so its situations have no index"));
        return BTreeSet::new();
    };
    let Some(section) = situations_section(&body) else {
        findings.push(format!("{label}: has no `{SITUATIONS_HEADING}` section"));
        return BTreeSet::new();
    };
    let mut names = BTreeSet::new();
    for line in section.lines().filter(|l| l.starts_with("- ")) {
        match situation_line_name(line) {
            Some(name) => {
                names.insert(name.to_string());
            }
            None => findings.push(format!(
                "{label}: situation line {line:?} is not of the form - `<name>` — <triggers> — \
                 delivers: <deliverable>, with <name> in [a-z0-9-]+"
            )),
        }
    }
    names
}

fn md_stems(dir: &Path, skip: &str) -> BTreeSet<String> {
    std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            entry.file_type().ok()?.is_file().then_some(())?;
            let name = entry.file_name().to_str()?.to_string();
            let stem = name.strip_suffix(".md")?;
            (name != skip).then(|| stem.to_string())
        })
        .collect()
}

fn compare(
    label: &str,
    index: &Path,
    dir: &Path,
    listed: &BTreeSet<String>,
    files: &BTreeSet<String>,
    findings: &mut Vec<String>,
) {
    for name in files.difference(listed) {
        findings.push(format!(
            "{label}: {}/{name}.md is not listed in {}, so no router can reach it",
            dir.display(),
            index.display()
        ));
    }
    for name in listed.difference(files) {
        findings.push(format!(
            "{label}: {} lists `{name}` but {}/{name}.md does not exist",
            index.display(),
            dir.display()
        ));
    }
}

fn situation_findings(playbooks: &Path, classes: &[String]) -> Vec<String> {
    let mut findings = Vec::new();
    let mut subdirs: Vec<String> = std::fs::read_dir(playbooks)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", playbooks.display()))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            entry.file_type().ok()?.is_dir().then_some(())?;
            entry.file_name().to_str().map(str::to_string)
        })
        .collect();
    subdirs.sort();

    for sub in &subdirs {
        if sub != SESSION_DIR && !classes.contains(sub) {
            findings.push(format!(
                "{sub}/: {}/{sub} is neither a route class in `ROUTE_CLASS_VALUES` \
                 ({WORKFLOWS_RS}) nor `{SESSION_DIR}`",
                playbooks.display()
            ));
        }
    }

    for class in classes {
        let index = playbooks.join(format!("{class}.md"));
        let dir = playbooks.join(class);
        let has_dir = subdirs.contains(class);
        let sectioned = std::fs::read_to_string(&index)
            .is_ok_and(|body| situations_section(&body).is_some());
        let listed = if has_dir || sectioned {
            listed_situations(&index, &mut findings)
        } else {
            BTreeSet::new()
        };
        let files = if has_dir { md_stems(&dir, "") } else { BTreeSet::new() };
        compare(class, &index, &dir, &listed, &files, &mut findings);
    }

    if subdirs.iter().any(|s| s == SESSION_DIR) {
        let dir = playbooks.join(SESSION_DIR);
        let index = dir.join("index.md");
        let listed = listed_situations(&index, &mut findings);
        let files = md_stems(&dir, "index.md");
        compare(SESSION_DIR, &index, &dir, &listed, &files, &mut findings);
    }
    findings
}

fn real_classes() -> Vec<String> {
    const_str_array(&read(WORKFLOWS_RS), "ROUTE_CLASS_VALUES")
}

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir()
            .join(format!("class-playbook-parity-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Sandbox(dir)
    }

    fn write(&self, rel: &str, body: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn with_clean_bugfix(tag: &str) -> Self {
        let sandbox = Sandbox::new(tag);
        sandbox.write(
            "bugfix.md",
            "# bugfix\n\n## Situations\n\n- `bug-fix` — bug, lỗi — delivers: the fix\n\nProof floor: x\n\n## Next\n\n- `not-a-situation` — x — delivers: y\n",
        );
        sandbox.write("bugfix/bug-fix.md", "# bug-fix\n");
        sandbox.write("feature.md", "# feature\n");
        sandbox
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn assert_one_finding(findings: &[String], needle: &str) {
    assert!(
        findings.len() == 1 && findings[0].contains(needle),
        "expected exactly one finding containing {needle:?}, got {findings:#?}"
    );
}

#[test]
fn every_situation_file_matches_its_index() {
    let findings = situation_findings(&repo_root().join(PLAYBOOKS_DIR), &real_classes());
    assert!(
        findings.is_empty(),
        "the situation playbooks disagree with their indexes:\n  {}\n\nA situation file no \
         index lists is never routed to, and a listed name with no file sends the router to \
         nothing. FIX: add the line - `<name>` — <triggers, English and Vietnamese> — delivers: \
         <deliverable> under `{SITUATIONS_HEADING}` in the class index (or session/index.md), \
         write the missing file, or delete the stray one.",
        findings.join("\n  ")
    );
}

#[test]
fn a_clean_sandbox_without_session_passes() {
    let sandbox = Sandbox::with_clean_bugfix("clean");
    let findings = situation_findings(&sandbox.0, &real_classes());
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn an_unlisted_situation_file_fails_the_fence() {
    let sandbox = Sandbox::with_clean_bugfix("unlisted");
    sandbox.write("bugfix/flaky-test.md", "# flaky\n");
    let findings = situation_findings(&sandbox.0, &real_classes());
    assert_one_finding(&findings, "flaky-test.md is not listed");
}

#[test]
fn a_listed_name_with_no_file_fails_the_fence() {
    let sandbox = Sandbox::with_clean_bugfix("missing");
    std::fs::remove_file(sandbox.0.join("bugfix/bug-fix.md")).unwrap();
    let findings = situation_findings(&sandbox.0, &real_classes());
    assert_one_finding(&findings, "lists `bug-fix` but");
}

#[test]
fn a_subdirectory_that_is_neither_a_class_nor_session_fails_the_fence() {
    let sandbox = Sandbox::with_clean_bugfix("stray");
    std::fs::create_dir_all(sandbox.0.join("hotfix")).unwrap();
    let findings = situation_findings(&sandbox.0, &real_classes());
    assert_one_finding(&findings, "hotfix is neither a route class");
}

#[test]
fn a_malformed_situation_line_fails_the_fence() {
    let sandbox = Sandbox::with_clean_bugfix("malformed");
    sandbox.write("bugfix.md", "## Situations\n\n- `bug-fix` — bug — the fix\n");
    let findings = situation_findings(&sandbox.0, &real_classes());
    assert!(
        findings.iter().any(|f| f.contains("is not of the form")),
        "{findings:#?}"
    );
}

#[test]
fn session_files_and_session_index_agree_both_ways() {
    let sandbox = Sandbox::with_clean_bugfix("session");
    sandbox.write(
        "session/index.md",
        "## Situations\n\n- `pickup` — resume, tiếp tục — delivers: the next step\n- `pause` — pause, tạm dừng — delivers: a safe stop\n",
    );
    sandbox.write("session/pickup.md", "# pickup\n");
    sandbox.write("session/orphan.md", "# orphan\n");
    let findings = situation_findings(&sandbox.0, &real_classes());
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings.iter().any(|f| f.contains("orphan.md is not listed")), "{findings:#?}");
    assert!(findings.iter().any(|f| f.contains("lists `pause` but")), "{findings:#?}");
}
