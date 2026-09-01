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
//   1. Every value in `ROUTE_CLASS_VALUES` has exactly one `### <class>`
//      section under `## Class playbooks`.
//   2. Every `### ` section under that heading names a real class value.
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
use std::path::PathBuf;

/// The single home of the class vocabulary.
const WORKFLOWS_RS: &str = "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs";

/// The single home of the playbooks.
const PLAYBOOKS_MD: &str = "skills/bee-planning/references/planning-reference.md";

/// The heading that opens the playbook set. The section runs to the next `## `.
const PLAYBOOKS_HEADING: &str = "## Class playbooks";

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

/// The `### ` section names under `## Class playbooks`, in document order,
/// duplicates kept — telling one section from two is this fence's job.
fn playbook_names() -> Vec<String> {
    let text = read(PLAYBOOKS_MD);
    let at = text.find(&format!("\n{PLAYBOOKS_HEADING}\n")).unwrap_or_else(|| {
        panic!(
            "the heading {PLAYBOOKS_HEADING:?} is gone from {PLAYBOOKS_MD}.\n\nThat heading is \
             where the playbooks live and where a plan is told to cite them. FIX: restore it, or \
             point this fence at wherever the playbooks now live."
        )
    }) + 1;
    let body = &text[at + PLAYBOOKS_HEADING.len()..];
    let end = body.find("\n## ").map(|i| i + 1).unwrap_or(body.len());

    let names: Vec<String> = body[..end]
        .lines()
        .filter_map(|l| l.strip_prefix("### "))
        .map(|n| n.trim().to_string())
        .collect();

    assert!(
        !names.is_empty(),
        "no `### ` section sits under {PLAYBOOKS_HEADING:?} in {PLAYBOOKS_MD}, so this fence \
         would pass on an empty playbook set"
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
        "route class(es) [{}] have no `### <class>` section under {PLAYBOOKS_HEADING:?} in \
         {PLAYBOOKS_MD}.\n\nA plan routed to one of those classes is told to cite a playbook that \
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
        "route class(es) [{}] carry MORE THAN ONE `### <class>` section in {PLAYBOOKS_MD}.\n\n\
         A class with two playbooks has none: a plan citing \"the\" playbook by name does not say \
         which steps it followed. FIX: merge them into one section.",
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
        "playbook section(s) [{}] under {PLAYBOOKS_HEADING:?} in {PLAYBOOKS_MD} name no value in \
         `ROUTE_CLASS_VALUES` ({WORKFLOWS_RS}).\n\nNo route can reach those steps, so nobody \
         reads them and nobody notices when they rot.\n\nFIX: rename the section to the class it \
         means, add the class to the constant, or delete the section.",
        orphans.join(" ")
    );
}
