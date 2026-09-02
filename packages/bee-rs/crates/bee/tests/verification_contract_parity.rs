// The verification-contract fence.
//
// `verification-in-the-flow` shipped two doctrine facts that nothing reads
// back. This file closes those two rows of its test matrix (decision
// 87f9409b); it proves the contracts, it does not change them.
//
// Two things are pinned.
//
//   1. **One name, stated identically everywhere.** Per D1 (`d0e3c3a0`) a
//      generated verification skill is called `VERIFY_APP_SKILL_NAME` in every
//      repo — a fixed literal, never a per-project name. That literal is
//      declared once in `onboard/templates.rs` and then RESTATED in the
//      doctrine block and in several skill bodies as plain prose. Renaming the
//      Rust constant leaves every one of those prose copies stale, in silence.
//      This is the repo's own recorded pattern: a rule living in N places needs
//      one test that reads all N
//      (`docs/knowledge/patterns/20260826-a-rule-living-in-n-places-needs-one-test-that-reads-all-n.md`).
//      The expected value is PARSED out of `templates.rs` as text, and the list
//      of surfaces is DERIVED by walking the doctrine roots — a sixth skill
//      joining the set is covered without editing this file. A fence that
//      hardcoded both sides would agree with itself forever.
//
//   2. **The cap-proof case is present.** Per D5 (`036e8a79`) the
//      `agents-proof-at-cap` bullet carries a user-facing-surface case naming
//      `green:live`, inside that one existing bullet rather than in a new one.
//
// Why this is NOT redundant with `agents_block_render_parity.rs`. That fence
// pins the `AGENTS.md` block byte-for-byte to `packages/bee/AGENTS.block.md`,
// which means it cannot catch a DELETION: remove the proof case from the
// SOURCE, run `bee dev regen`, and the two files agree perfectly while the
// doctrine is gone. Equality between a generated file and its source says
// nothing about what the source still says.
//
// What is deliberately NOT pinned, and why.
//
//   - **D4's read-first feature-map mention** (`c93a6948`). D4 carries its own
//     named falsifier, trigger `two-features-have-been-planned-with-a-ma__c93a6948`:
//     if two planned features do not cite gotchas from a mapped feature file,
//     D4's shaping tier is REVERTED. A test asserting D4's line is present
//     would turn that agreed revert into a test failure — the fence would
//     defend a rule the repo has already agreed to drop on evidence. Settled
//     contracts get pinned; provisional ones do not (decision `29b853d8`).
//   - **The prose itself.** Test 2 asserts a few load-bearing tokens, never a
//     sentence. A fence pinning wording goes red on every legitimate reword and
//     gets deleted by the next person who hits it.
//   - **Anything outside the doctrine roots.** The walk covers `skills/` and
//     `packages/bee/` — the surfaces that INSTRUCT an agent. `docs/knowledge/`
//     and the Rust sources are out: both legitimately write the generic
//     `.bee/verify/<name>` placeholder and quote the retired `verify-<app>`
//     form when they document the onboarding contract, so walking them would
//     red on correct text. The generated skill trees (`.claude/`, `.agents/`,
//     `.opencode/`, `.claude-plugin/`, `.codex-plugin/`) are rendered copies of
//     `skills/`, and `docs/history/` is an immutable record that correctly
//     preserves the old form; neither is a surface anyone would edit to fix a
//     rename. None of the three sits under a walked root.
//
// Shape, deliberately: pure filesystem, std only, NOTHING imported from the bee
// crate — the model is `rule_index_parity.rs` and `agents_block_render_parity.rs`
// beside it. A fence importing the constant would rename along with it and
// catch nothing.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where the one name is declared, and the constant that declares it.
const TEMPLATES: &str = "packages/bee-rs/crates/bee/src/onboard/templates.rs";
const NAME_CONST: &str = "VERIFY_APP_SKILL_NAME";

/// The source of the operating block. `AGENTS.md` is its render and is pinned
/// to it by `agents_block_render_parity`, so asserting here would double-count.
const AGENTS_BLOCK: &str = "packages/bee/AGENTS.block.md";

/// The doctrine roots: bee's own instruction surfaces.
const ROOTS: [&str; 2] = ["skills", "packages/bee"];

/// The fixed source path of a generated verification skill (D8, `9f4f90f0`).
/// The segment after it is the skill's name, which is why this prefix is the
/// unambiguous anchor for "this text states the skill's name".
const VERIFY_PREFIX: &str = ".bee/verify/";

/// The rule this fence reads the proof case out of.
const PROOF_RULE: &str = "<!-- rule: agents-proof-at-cap -->";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// The VALUE of `VERIFY_APP_SKILL_NAME`, read out of `templates.rs` as text.
/// An integration test cannot see the symbol, and reading it is the whole
/// point: this is the one copy every other surface must agree with.
fn declared_name() -> String {
    let src = read(TEMPLATES);
    let needle = format!("pub const {NAME_CONST}: &str = \"");
    let start = src.find(&needle).unwrap_or_else(|| {
        panic!(
            "{TEMPLATES} no longer declares `pub const {NAME_CONST}: &str = \"…\"`.\n\nThat \
             constant is the single home of the verification skill's name; this fence parses it \
             as text so it cannot rename along with it. FIX: restore the constant, or point this \
             fence at its new home."
        )
    }) + needle.len();
    let rest = &src[start..];
    let end = rest
        .find('"')
        .unwrap_or_else(|| panic!("{NAME_CONST} in {TEMPLATES} has no closing quote"));
    rest[..end].to_string()
}

/// Every file under the doctrine roots, as repo-relative paths, sorted.
fn doctrine_files() -> Vec<String> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let root = repo_root();
    let mut out = Vec::new();
    for r in ROOTS {
        walk(&root.join(r), &root, &mut out);
    }
    out.sort();
    out
}

/// A path segment is the run of name characters after `VERIFY_PREFIX`. It stops
/// at `/` and at any punctuation, so `.bee/verify/verify-app/features/` yields
/// `verify-app` and a bare `.bee/verify/` or a `<name>` placeholder yields
/// nothing to compare.
fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
}

/// `(line number, stated name)` for every skill name stated as a source path.
fn path_form_names(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let mut cursor = 0usize;
        while let Some(rel) = line[cursor..].find(VERIFY_PREFIX) {
            let start = cursor + rel + VERIFY_PREFIX.len();
            let seg: String = line[start..].chars().take_while(|c| is_name_char(*c)).collect();
            if !seg.is_empty() {
                out.push((n + 1, seg));
            }
            cursor = start;
        }
    }
    out
}

/// `(line number, span content)` for every markdown code span, skipping fenced
/// code blocks so a ```` ```bash ```` fence line cannot be read as a span.
fn code_spans(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut fenced = false;
    for (n, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        for (i, chunk) in line.split('`').enumerate() {
            if i % 2 == 1 {
                out.push((n + 1, chunk.to_string()));
            }
        }
    }
    out
}

/// The retired per-project form: `verify-` followed straight by a placeholder
/// opener. The `verify-` must start at a token boundary — `notes-verify-$RUN_ID`
/// is an environment variable, not a skill name — and a path-form
/// `.bee/verify/verify-<app>` is left to the segment comparison, which names the
/// mismatch better than a placeholder ban could.
fn retired_form(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let mut cursor = 0usize;
        while let Some(rel) = line[cursor..].find("verify-") {
            let at = cursor + rel;
            let before = line[..at].chars().next_back();
            let after = line[at + "verify-".len()..].chars().next();
            let boundary =
                !matches!(before, Some(c) if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '/');
            if boundary && matches!(after, Some('<') | Some('{') | Some('$')) {
                out.push((n + 1, line.trim().to_string()));
            }
            cursor = at + "verify-".len();
        }
    }
    out
}

// ── one name, stated identically on every surface that states it ───────────

#[test]
fn every_doctrine_surface_states_the_one_declared_verify_skill_name() {
    let name = declared_name();

    // The one sanity check on the parse itself: a fence comparing every surface
    // against an empty or malformed literal would pass on anything.
    assert!(
        !name.is_empty() && name.chars().all(is_name_char),
        "{NAME_CONST} in {TEMPLATES} parsed as {name:?}, which is not a usable skill name. A \
         fence comparing every surface against that literal cannot fail — the exact silence it \
         exists to break."
    );

    let mut wrong: Vec<String> = Vec::new();
    let mut retired: Vec<String> = Vec::new();
    let mut stating_files: BTreeSet<String> = BTreeSet::new();

    for rel in doctrine_files() {
        let Ok(text) = std::fs::read_to_string(repo_root().join(&rel)) else { continue };

        let paths = path_form_names(&text);
        if !paths.is_empty() {
            stating_files.insert(rel.clone());
        }
        for (line, stated) in &paths {
            if *stated != name {
                wrong.push(format!("{rel}:{line} states {stated:?} as the skill name"));
            }
        }

        // A bare `verify-…` code span is a name statement only in a file that
        // also states the skill's source path. Elsewhere in these roots a
        // `verify-`-prefixed token is a different noun — `verify-red` is the CI
        // issue label — and reading those as names would red on correct text.
        if !paths.is_empty() {
            for (line, span) in code_spans(&text) {
                if span.starts_with("verify-") && span.chars().all(is_name_char) && span != name {
                    wrong.push(format!("{rel}:{line} names the skill {span:?} in prose"));
                }
            }
        }

        for (line, source) in retired_form(&text) {
            retired.push(format!("{rel}:{line}: {source}"));
        }
    }

    assert!(
        stating_files.len() > 1,
        "only {} file(s) under {ROOTS:?} state a `{VERIFY_PREFIX}<name>` path, so this fence has \
         no drift to detect. It exists because that name is restated on many surfaces; a walk \
         finding one means the walk broke, not that the duplication ended.",
        stating_files.len(),
    );

    assert!(
        wrong.is_empty(),
        "surface(s) state a verification skill name other than {name:?}, the value of \
         {NAME_CONST} in {TEMPLATES}:\n\n  {}\n\nThat name is ONE fixed literal in every repo \
         (verification-in-the-flow D1): agents find the skill at a fixed path and bee surfaces \
         name it in literal text, so a surface that disagrees points people at a skill that is \
         not there. FIX: make the surfaces above match the constant — or, if the rename is \
         intended, change the constant and every surface in the SAME commit.",
        wrong.join("\n  "),
    );

    assert!(
        retired.is_empty(),
        "surface(s) still carry the retired per-project form `verify-<app>`:\n\n  {}\n\nThe \
         per-project name was dropped (verification-in-the-flow D1); the name is the fixed \
         literal {name:?} everywhere. FIX: replace the placeholder with that literal.",
        retired.join("\n  "),
    );
}

// ── the cap-proof rule still carries its user-facing-surface case ──────────

#[test]
fn the_proof_at_cap_rule_carries_the_user_facing_surface_case() {
    let block = read(AGENTS_BLOCK);

    let start = block.find(PROOF_RULE).unwrap_or_else(|| {
        panic!(
            "{AGENTS_BLOCK} no longer carries the marker {PROOF_RULE}.\n\nThat marker is how the \
             rule is addressed by name and how this fence finds the bullet. FIX: restore it, or \
             point this fence at the rule's new id."
        )
    }) + PROOF_RULE.len();
    let rest = &block[start..];
    let end = rest.find("\n<!--").unwrap_or(rest.len());
    let rule = &rest[..end];

    // Two tokens, not the sentence. `user-facing` says WHICH class of change
    // the case governs and `green:live` is the literal a cap must record — the
    // two pieces a worker acts on. Deleting the case removes both; rewording it
    // keeps both. The middle of the sentence ("its mapped feature driven,
    // evidence attached") is left unpinned on purpose: it is the part most
    // likely to be legitimately rephrased.
    for token in ["user-facing", "green:live"] {
        assert!(
            rule.contains(token),
            "the `agents-proof-at-cap` rule in {AGENTS_BLOCK} no longer says {token:?}.\n\nThe \
             rule must carry the user-facing-surface proof case — drive the change's mapped \
             feature and record `green:live` (verification-in-the-flow D5). Without it a \
             user-facing change caps on a unit test that never ran the product.\n\nNote that \
             `agents_block_render_parity` CANNOT catch this: it pins AGENTS.md to this file \
             byte-for-byte, so deleting the case here and regenerating leaves both files in \
             perfect agreement. FIX: restore the case inside this rule's existing bullet, then \
             run `.bee/bin/bee dev regen`.\n\nrule as found:\n{rule}"
        );
    }

    let bullets = rule.lines().filter(|l| l.starts_with("- ")).count();
    assert!(
        bullets == 1,
        "the `agents-proof-at-cap` rule in {AGENTS_BLOCK} spans {bullets} top-level bullets; it \
         must be exactly one.\n\nThe user-facing-surface case belongs INSIDE the existing \
         proof-line bullet, not beside it (verification-in-the-flow D5): a separate bullet reads \
         as a separate obligation and gets skipped by anyone who thinks their change is not \
         user-facing. FIX: fold the case back into the one bullet, then run `.bee/bin/bee dev \
         regen`.\n\nrule as found:\n{rule}"
    );
}
