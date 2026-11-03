// The route-class vocabulary parity fence.
//
// `ROUTE_CLASS_VALUES` is named verbatim in FOUR source documents, and until
// this test existed only one of them sat anywhere near a check: no test read
// `docs/product-description/` at all, so three of the four sites could drift
// from the constant for as long as nobody happened to read them side by side.
// A vocabulary spelled out in four places and pinned in none is a rotted
// allowlist wearing prose: it stops matching SILENTLY.
//
// Two things are pinned here.
//
//   1. Every one of the four documents lists exactly the class values the
//      constant holds — no more, no fewer, in the constant's own order.
//   2. No class value is also a LANE value beyond the pair the source itself
//      grandfathers. That is the `mode`-carries-a-class leak: a lane record's
//      `mode` field usually holds a workflow CLASS, and two readers
//      (verbs/drivers/close.rs, uat.rs) fall back to `mode` as a LANE only
//      when the value it holds is itself a lane value. A newly added class
//      that collides with a lane name would turn that fallback into a silent
//      misread, so the collision set is frozen at what is already there.
//
// Shape, deliberately: pure filesystem, std only, and NOTHING imported from
// the bee crate — the model is `specs_fence.rs`. `ROUTE_CLASS_VALUES` is
// `pub(crate)`, so an integration test cannot see it as a symbol; this file
// reads `workflows.rs` as TEXT instead.
//
// The one thing this file must never do is re-declare either list. That
// duplication has already happened once in this codebase
// (`FEATURE_ROUTE_LANE_CLASSES` in verbs/drivers/close.rs, copied because the
// const is module-private), and removing that class of drift is the reason
// this fence exists. A fence holding its own private copy of the truth it
// guards agrees with itself forever and catches nothing.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// The single home of both vocabularies. Everything below is derived from this
/// file's text; nothing below states a value of its own.
const WORKFLOWS_RS: &str = "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs";

/// The prose in `workflows.rs` that names the already-present class/lane
/// collisions and freezes them. Quoting the source's own sentence keeps the
/// grandfather clause single-homed with the safety argument it belongs to.
const GRANDFATHER_PHRASE: &str = "already sit in both vocabularies";

/// The four SOURCE documents that spell the class vocabulary out, each with the
/// anchor text that opens its list.
///
/// SOURCE only. The rendered plugin copies (`.claude-plugin/` and friends) are
/// `bee dev regen` OUTPUT, not a second truth: fencing them would report one
/// stale document as five and would go red for a missing regen run rather than
/// for the drift this test is about.
const CLASS_SITES: &[(&str, &str)] = &[
    ("skills/bee-hive/references/scout-and-ticks.md", "`class` ∈"),
    ("docs/product-description/goal.md", "Route vocabularies: class"),
    ("docs/product-description/lifecycle/planning.md", "`class` from"),
    ("docs/product-description/verification/lifecycle.md", "`class` is a closed enum:"),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn workflows_source() -> String {
    let path = repo_root().join(WORKFLOWS_RS);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read the vocabulary's home {WORKFLOWS_RS}: {e}"))
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

/// Every double-quoted literal in a slice of text, in order.
fn quoted(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut parts = text.split('"');
    parts.next();
    while let Some(value) = parts.next() {
        out.push(value.to_string());
        if parts.next().is_none() {
            break;
        }
    }
    out
}

/// The class/lane collisions `workflows.rs` itself declares pre-existing and
/// frozen, read out of its own safety-argument comment.
fn grandfathered_collisions(src: &str) -> BTreeSet<String> {
    let at = src.find(GRANDFATHER_PHRASE).unwrap_or_else(|| {
        panic!(
            "the sentence containing {GRANDFATHER_PHRASE:?} is gone from {WORKFLOWS_RS}.\n\nThat \
             sentence is where the already-present class/lane collisions are named and frozen, \
             and this fence reads them there rather than keeping a copy. FIX: restore the \
             sentence, or point this fence at wherever the grandfather clause now lives."
        )
    });
    let head = &src[..at];
    let start = head.rfind(". ").map(|i| i + 2).unwrap_or(0);
    let found: BTreeSet<String> = quoted(&head[start..]).into_iter().collect();
    assert!(
        !found.is_empty(),
        "the grandfather sentence in {WORKFLOWS_RS} names no quoted value, so this fence cannot \
         tell an old collision from a new one"
    );
    found
}

/// Pull one document's class list out of the line carrying `anchor`.
///
/// The four sites do not agree on punctuation — one backticks each value and
/// separates with commas, two use a single pipe-joined span, one a single
/// space-joined span. So: after the anchor, take backtick spans for as long as
/// the text between them is nothing but list separators. The first gap that
/// holds real prose ends the list, which is what keeps the neighbouring LANE
/// vocabulary on the same line from being swallowed into the class list.
fn class_values_at(line: &str, anchor: &str) -> Vec<String> {
    let is_separator = |c: char| c.is_whitespace() || c == ',' || c == '|';
    let mut cursor = line.find(anchor).expect("caller already matched the anchor") + anchor.len();
    let mut values: Vec<String> = Vec::new();

    loop {
        let rest = &line[cursor..];
        let Some(open) = rest.find('`') else { break };
        if !rest[..open].chars().all(is_separator) {
            break;
        }
        let Some(close) = rest[open + 1..].find('`') else { break };
        let span = &rest[open + 1..open + 1 + close];
        values.extend(
            span.split(is_separator).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string),
        );
        cursor += open + 1 + close + 1;
    }
    values
}

// ── the four documents name exactly the constant's values ──────────────────

#[test]
fn every_document_naming_the_class_enum_lists_exactly_its_values() {
    let root = repo_root();
    let expected = const_str_array(&workflows_source(), "ROUTE_CLASS_VALUES");

    let mut stale: Vec<String> = Vec::new();

    for (rel, anchor) in CLASS_SITES {
        let path = root.join(rel);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read the documented site {rel}: {e}"));

        let mut anchored = 0usize;
        for (i, line) in text.lines().enumerate() {
            if !line.contains(anchor) {
                continue;
            }
            anchored += 1;
            let found = class_values_at(line, anchor);
            if found == expected {
                continue;
            }
            let expected_set: BTreeSet<&String> = expected.iter().collect();
            let found_set: BTreeSet<&String> = found.iter().collect();
            let missing: Vec<&str> =
                expected.iter().filter(|v| !found_set.contains(v)).map(|s| s.as_str()).collect();
            let extra: Vec<&str> =
                found.iter().filter(|v| !expected_set.contains(v)).map(|s| s.as_str()).collect();
            stale.push(format!(
                "  {rel}:{}\n      lists : {}\n      should: {}\n      missing here: [{}]   \
                 not in the enum: [{}]",
                i + 1,
                found.join(" "),
                expected.join(" "),
                missing.join(" "),
                extra.join(" "),
            ));
        }

        assert!(
            anchored > 0,
            "{rel} no longer carries the anchor {anchor:?}, so this fence stopped reading it.\n\n\
             A site that silently drops out of the fence is the drift this test exists to catch. \
             FIX: update the anchor here, or remove the site if that document genuinely stopped \
             naming the vocabulary."
        );
    }

    assert!(
        stale.is_empty(),
        "document(s) naming the route-class enum disagree with `ROUTE_CLASS_VALUES` \
         ({WORKFLOWS_RS}), which is its single home:\n\n{}\n\nFIX: edit the document(s) above to \
         match the constant. Every place that spells the vocabulary out has to be updated in the \
         same change as the constant — that is the whole point of listing it four times.",
        stale.join("\n")
    );
}

// ── no class value is a lane value, beyond the frozen pair ─────────────────

#[test]
fn no_class_value_is_also_a_lane_value() {
    let src = workflows_source();
    let classes = const_str_array(&src, "ROUTE_CLASS_VALUES");
    let lanes: BTreeSet<String> = const_str_array(&src, "ROUTE_LANE_VALUES").into_iter().collect();
    let grandfathered = grandfathered_collisions(&src);

    let collisions: BTreeSet<String> =
        classes.iter().filter(|c| lanes.contains(*c)).cloned().collect();

    let fresh: Vec<&str> = collisions.difference(&grandfathered).map(|s| s.as_str()).collect();
    assert!(
        fresh.is_empty(),
        "class value(s) [{}] are also LANE values, and the grandfather clause in {WORKFLOWS_RS} \
         does not cover them.\n\nA lane record's `mode` field usually carries a workflow CLASS, \
         and the readers in verbs/drivers/close.rs and uat.rs fall back to reading `mode` as a \
         LANE whenever its value happens to be a lane name. A new class that collides with a \
         lane name makes that fallback fire on a record that never meant a lane, silently.\n\n\
         FIX: rename the class so it cannot be read as a lane. Widening the grandfather clause \
         instead only makes the misread legal.",
        fresh.join(" ")
    );

    let healed: Vec<&str> = grandfathered.difference(&collisions).map(|s| s.as_str()).collect();
    assert!(
        healed.is_empty(),
        "the grandfather clause in {WORKFLOWS_RS} still excuses [{}], but those value(s) are no \
         longer in both vocabularies.\n\nFIX: drop them from that sentence. A clause excusing a \
         collision that no longer exists is spare permission waiting to be reused.",
        healed.join(" ")
    );
}
