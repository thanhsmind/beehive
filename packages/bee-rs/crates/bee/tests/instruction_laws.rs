// Instruction-layer laws — the post-cutover home of two Node suites that
// would otherwise go VACUOUSLY GREEN when `packages/bee/**` and `scripts/**`
// are deleted at R6.
//
// Both laws were flagged as hard blockers for deleting the Node runtime
// (plans/rust-port.md § "Hard blockers for deleting the Node runtime";
// plans/r5-test-migration.md § "Deliberately NOT ported" ¶1). Neither is
// NODE-ONLY: they happen to be written in Node, but their subject is the
// shipped instruction layer and the tooling that could damage it. So they are
// RE-POINTED here, at surfaces that survive the deletion, rather than retired.
//
//   ORIGIN                                    RE-POINTED SCAN SET HERE
//   ---------------------------------------   ------------------------------
//   scripts/tests/test_instruction_size_law    packages/bee-rs/crates/**/*.rs
//     .mjs, invariant 1 (no size ceiling on    + scripts/**/*.{mjs,js,cjs}
//     instruction text is ever a standing      while that tree still exists,
//     law here — budget-fence-removal D1,      + JSON data files in both
//     decision 8f63adb4)                       (the shape skill-body-budget
//                                              .json had, under any name)
//
//   scripts/tests/test_scan_set_hygiene.mjs    packages/bee-rs/crates/**/*.rs
//     CHECK 1 (E4): a scan set derived from    — the Rust tree DOES shell out
//     `git ls-files` and read without an       to `git ls-files`
//     existence guard                          (onboard/notices.rs)
//
//   scripts/tests/test_scan_set_hygiene.mjs    skills/*/SKILL.md,
//     CHECK 2 (E8): a retired workflow stage   skills/*/references/**,
//     described as CURRENT                     expertise/**, docs/knowledge/**,
//                                              docs/specs/**, AGENTS.md,
//                                              CLAUDE.md — this surface is
//                                              untouched by the deletion;
//                                              only its TOKEN SOURCE
//                                              (packages/bee/lib/state.mjs)
//                                              dies, so the derivation is
//                                              re-pointed at the Rust tree's
//                                              own coercion record and
//                                              cross-checked against Node's
//                                              while both exist.
//
// NOT re-pointed here, deliberately: invariant 2 of test_instruction_size_law
// .mjs (the "meaning guards in test_agents_budget.mjs still bite" pair). Its
// subject is another Node suite, not the shipped prose — porting it means
// first choosing a home for test_agents_budget.mjs itself, which is R6a
// instruction-layer work. It is recorded as an open item in
// plans/cutover-readiness.md rather than silently dropped, and its Node
// original fails loudly (never vacuously) if its subject disappears.
//
// ── THE RULE THIS FILE ENFORCES ON ITSELF ────────────────────────────────
// A law that scans a set must FAIL when that set is unexpectedly empty,
// naming what it expected to find. A green tick over an empty scan set is
// worse than no check: it reads as coverage while asserting nothing. Every
// scan below goes through `collect_scan_set`, which refuses below a declared
// floor, and `the_vacuity_guard_refuses_an_empty_scan_set` proves the refusal
// fires — permanently, in the suite, not as a probe run once and deleted.
//
// No regex crate: the binary ships zero non-essential dependencies (the Node
// original had zero npm packages for the same reason), so every classifier
// here is a hand-rolled text scanner. Each is proven on synthetic fixtures
// before it is pointed at the tree, the same discipline as
// test_scan_set_hygiene.mjs's `--selftest`.

use std::path::{Path, PathBuf};

// ════════════════════════════════════════════════════════════════════════
// Roots and the vacuity guard
// ════════════════════════════════════════════════════════════════════════

fn repo_root() -> PathBuf {
    // crates/bee -> crates -> bee-rs -> packages -> beehive
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap()
        .to_path_buf()
}

/// The legitimately self-referential sources, excluded from every scan below:
/// each quotes the exact violating text as DATA to prove a classifier bites,
/// and a plain text scan cannot tell "this line is the proof" from "this line
/// is the violation". Exactly two entries — this file, and the Node law it
/// re-points (which excludes itself for the identical reason, via
/// `__filename` / `SELF_PATH`). Every other file in scope is fully scanned;
/// this is not a general carve-out, and
/// `the_self_exclusion_list_carries_only_files_that_quote_violations_as_fixtures`
/// pins it.
const SELF_REFERENTIAL: &[&str] = &[
    // R6 CUTOVER: this list used to carry a second entry,
    // `scripts/tests/test_instruction_size_law.mjs` — the Node law this file
    // re-points, which excluded ITSELF for the identical reason. That file is
    // deleted with the Node tree, so the carve-out is a carve-out of one again.
    "packages/bee-rs/crates/bee/tests/instruction_laws.rs",
];

/// A scan root: where to look, what must be there, and whether its absence is
/// itself a failure. `required: false` marks a tree that the R6 cutover is
/// EXPECTED to delete — its absence is fine, but if it is present it must
/// still be plausibly populated, and the union floor still has to hold.
struct Root {
    rel: &'static str,
    /// Extensions to collect, e.g. `["rs"]`. Empty means "any file".
    exts: &'static [&'static str],
    min_files: usize,
    required: bool,
}

#[derive(Debug)]
struct ScanSet {
    label: &'static str,
    files: Vec<(String, String)>, // (repo-relative path, contents)
}

fn walk(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            // Never scan build output or vendored trees.
            if matches!(name.as_ref(), "target" | "node_modules" | ".git") {
                continue;
            }
            walk(&path, exts, out);
        } else if ft.is_file() {
            let matches_ext = exts.is_empty()
                || path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| exts.contains(&e));
            if matches_ext {
                out.push(path);
            }
        }
    }
}

fn rel_of(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// THE VACUITY GUARD. Collects a scan set and refuses — with a message that
/// names the label, every root it walked, what it expected to find there, the
/// count it got and the floor it needed — whenever the set is empty or
/// implausibly small. Returns `Err` rather than panicking so that
/// `the_vacuity_guard_refuses_an_empty_scan_set` can assert the refusal text
/// directly, without panic-hook games in a parallel test binary.
fn collect_scan_set(
    root: &Path,
    label: &'static str,
    expectation: &'static str,
    roots: &[Root],
    union_min: usize,
) -> Result<ScanSet, String> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut walked: Vec<String> = Vec::new();

    for r in roots {
        let abs = root.join(r.rel);
        walked.push(r.rel.to_string());
        if !abs.exists() {
            if r.required {
                return Err(format!(
                    "{label}: SCAN ROOT MISSING — `{}` does not exist. Expected {expectation}. \
                     A law whose scan root is gone must be re-pointed at the surface that replaced \
                     it, or retired with a recorded reason — never left to pass over nothing. \
                     See plans/cutover-readiness.md.",
                    r.rel
                ));
            }
            continue;
        }
        let mut found = Vec::new();
        walk(&abs, r.exts, &mut found);
        let found: Vec<PathBuf> = found
            .into_iter()
            .filter(|p| !SELF_REFERENTIAL.contains(&rel_of(root, p).as_str()))
            .collect();
        if found.len() < r.min_files {
            return Err(format!(
                "{label}: SCAN ROOT `{}` IS IMPLAUSIBLY SMALL — {} file(s) matched {:?}, expected at \
                 least {}. Expected {expectation}. This law would be asserting almost nothing over \
                 that root. See plans/cutover-readiness.md.",
                r.rel,
                found.len(),
                r.exts,
                r.min_files
            ));
        }
        for p in found {
            let Ok(text) = std::fs::read_to_string(&p) else {
                continue; // binary or unreadable — not this law's subject
            };
            files.push((rel_of(root, &p), text));
        }
    }

    if files.len() < union_min {
        return Err(format!(
            "{label}: SCAN SET IS EMPTY OR IMPLAUSIBLY SMALL — {} file(s) across roots [{}], \
             expected at least {}. Expected {expectation}. This check asserts nothing over that \
             set, so it refuses rather than reporting a vacuous PASS. If the subject tree was \
             deliberately removed (e.g. the R6 Node-runtime cutover), re-point this law at the \
             surface that replaced it or retire it explicitly — see plans/cutover-readiness.md.",
            files.len(),
            walked.join(", "),
            union_min
        ));
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ScanSet { label, files })
}

fn require(set: Result<ScanSet, String>) -> ScanSet {
    match set {
        Ok(s) => s,
        Err(msg) => panic!("{msg}"),
    }
}

// ── The roots, declared once ─────────────────────────────────────────────

/// The Rust source tree — the tooling that survives the cutover, and
/// therefore the tree that could reintroduce either defect class after Node
/// is gone.
const RUST_TREE: Root = Root {
    rel: "packages/bee-rs/crates",
    exts: &["rs"],
    min_files: 40,
    required: true,
};

/// RETIRED AT THE R6 CUTOVER: `SCRIPT_TREE` — `scripts/**/*.{mjs,js,cjs}`,
/// `required: false`, `min_files: 10`.
///
/// The Node tree it scanned is deleted, which is the whole point of this file
/// existing. `required: false` was written for exactly this moment, but it was
/// not enough on its own: `scripts/` SURVIVES (it still holds install.sh and
/// install.ps1), so the root does not vanish — it goes to ZERO matching files,
/// which trips `IMPLAUSIBLY SMALL` rather than the tolerated-absence arm. Two
/// laws would have gone red for the wrong reason and read as regressions.
///
/// Both laws now scan `RUST_TREE` alone. That is not a widening or a
/// weakening: every tool the deleted scripts implemented lives in the Rust
/// tree, so the DEFECT CLASS each law hunts (a size ceiling on instruction
/// text; an unguarded scan set) has exactly one place left to reappear, and
/// the vacuity guard's union floor still holds over it.
///
/// A fixture-only root lives in `the_vacuity_guard_refuses_an_empty_scan_set`,
/// which needs an OPTIONAL root to exercise its two tolerated-absence arms
/// against a tempdir. It is deliberately local to that test: it must never
/// again be a root that a real law scans.

// ════════════════════════════════════════════════════════════════════════
// Proof: the vacuity guard actually bites
// ════════════════════════════════════════════════════════════════════════

#[test]
fn the_vacuity_guard_refuses_an_empty_scan_set() {
    let tmp = tempfile::tempdir().unwrap();

    // A fixture-only optional root. It exists so this test can exercise the
    // tolerated-absence arms; no real law scans it (see the SCRIPT_TREE
    // retirement note above).
    const FIXTURE_TREE: Root =
        Root { rel: "scripts", exts: &["mjs", "js", "cjs"], min_files: 10, required: false };

    // Arm 1 — a PRESENT but EMPTY root. This is what emptying a tree
    // produces, and what a mis-typed extension filter produces.
    std::fs::create_dir_all(tmp.path().join("scripts")).unwrap();
    let err = collect_scan_set(
        tmp.path(),
        "fixture law",
        "the repo's tooling trees",
        &[FIXTURE_TREE],
        10,
    )
    .expect_err("an empty scan set must REFUSE, never return a green empty set");
    assert!(
        err.contains("IMPLAUSIBLY SMALL") && err.contains("0 file(s)") && err.contains("at least 10"),
        "the refusal must name the count it got and the floor it needed: {err}"
    );
    assert!(
        err.contains("cutover-readiness"),
        "the refusal must say where to go: {err}"
    );

    // Arm 2 — an ABSENT required root. This is exactly what deleting
    // `packages/bee/**` or `scripts/**` produces for the Node originals.
    let err = collect_scan_set(
        tmp.path(),
        "fixture law",
        "the Rust source tree (packages/bee-rs/crates/**/*.rs)",
        &[RUST_TREE],
        1,
    )
    .expect_err("an absent REQUIRED scan root must REFUSE");
    assert!(
        err.contains("SCAN ROOT MISSING")
            && err.contains("packages/bee-rs/crates")
            && err.contains("Rust source tree"),
        "the refusal must name the missing root AND what it expected to find there: {err}"
    );

    // Arm 3 — an absent OPTIONAL root is tolerated on its own, but the union
    // floor still bites, so "everything vanished" can never read as green.
    let err = collect_scan_set(
        tempfile::tempdir().unwrap().path(),
        "fixture law",
        "the repo's tooling trees",
        &[FIXTURE_TREE],
        1,
    )
    .expect_err("an absent optional root must still trip the union floor");
    assert!(
        err.contains("SCAN SET IS EMPTY"),
        "the union floor must catch a scan set that vanished entirely: {err}"
    );

    // Control — the LIVE tree passes the same guard, so this is a floor, not
    // a blanket refusal.
    let live = collect_scan_set(
        &repo_root(),
        "fixture law",
        "the repo's tooling trees",
        &[RUST_TREE],
        40,
    );
    assert!(
        live.is_ok(),
        "the vacuity guard must not fire on the live tree: {:?}",
        live.err()
    );
}

/// A carve-out nobody polices is a hole. Every entry in `SELF_REFERENTIAL`
/// must still BE self-referential — i.e. still carry fixture text that a
/// classifier in this file would otherwise flag. An entry that stopped being
/// so is a silent exemption and must be deleted from the list.
#[test]
fn the_self_exclusion_list_carries_only_files_that_quote_violations_as_fixtures() {
    let root = repo_root();
    assert_eq!(
        SELF_REFERENTIAL.len(),
        1,
        "the self-exclusion list may not be widened without a recorded reason — since the R6 \
         cutover it is exactly this file (the Node law it re-pointed is deleted with its tree)"
    );
    assert!(
        root.join(SELF_REFERENTIAL[0]).exists(),
        "this file must be in its own exclusion list at its real path"
    );
    for rel in SELF_REFERENTIAL {
        let path = root.join(rel);
        if !path.exists() {
            continue; // scripts/** is expected to be deleted at R6 cutover
        }
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            !size_law_shape_hits(rel, &text).is_empty()
                || !find_unguarded_scan_sets(&text).is_empty(),
            "`{rel}` is excluded from every scan in this file but no longer contains any fixture a \
             classifier here would flag. The exclusion has lost its reason — remove the entry, or \
             the file is an unpoliced blind spot."
        );
    }
}

// ════════════════════════════════════════════════════════════════════════
// LAW A — no size ceiling on instruction text is ever a standing law here
// ════════════════════════════════════════════════════════════════════════
//
// budget-fence-removal D1/D5 (docs/history/budget-fence-removal/CONTEXT.md,
// decision 8f63adb4). A removal is verified by its INVARIANTS, not by the
// names it deleted (critical pattern 20260711): an `rg` for
// "skill_budget_fence" would pass forever while someone reintroduced the same
// law under a new name. So this asserts the DEFECT CLASS, in three shapes.

const UNIT_WORDS: &[&str] = &["byte", "line"];
const LIMIT_WORDS: &[&str] = &[
    "hardfail",
    "hard_fail",
    "hard-fail",
    "warn",
    "max",
    "limit",
    "ceil",
    "budget",
    "threshold",
];

fn is_ceiling_shaped_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    UNIT_WORDS.iter().any(|u| lower.contains(u)) && LIMIT_WORDS.iter().any(|l| lower.contains(l))
}

/// The identifier immediately to the left of an assignment operator, with a
/// Rust type annotation (`const FOO: usize = 1`) stripped. Returns None when
/// there is no plain identifier there.
fn assignment_target(lhs: &str) -> Option<String> {
    // Strip a trailing type annotation: the last `:` that is not part of `::`.
    let bytes = lhs.as_bytes();
    let mut cut = lhs.len();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':' {
            if i + 1 < bytes.len() && bytes[i + 1] == b':' {
                i += 2;
                continue;
            }
            if i > 0 && bytes[i - 1] == b':' {
                i += 1;
                continue;
            }
            cut = i;
        }
        i += 1;
    }
    let head = lhs[..cut].trim_end();
    let ident: String = head
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if ident.is_empty() || ident.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(ident)
}

/// Shape A — an identifier naming BOTH a size unit and a limiting concept,
/// assigned a numeric literal. Order-independent on purpose: it catches
/// `HARD_FAIL_BYTES = 15000` exactly as it would catch a future
/// `skill_text_ceiling_bytes = 9001` or `LINE_LIMIT: usize = 200`. The two
/// constants budget-fence-removal deleted are two INSTANCES of this shape,
/// not the shape itself.
fn ceiling_shaped_identifiers(line: &str) -> Vec<String> {
    let b = line.as_bytes();
    let mut hits = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let is_op = match b[i] {
            b'=' => {
                // Not `==`, `<=`, `>=`, `!=`, `+=`, `-=`, `*=`, `/=`.
                let next_is_eq = i + 1 < b.len() && b[i + 1] == b'=';
                let prev_is_cmp = i > 0
                    && matches!(
                        b[i - 1],
                        b'=' | b'<' | b'>' | b'!' | b'+' | b'-' | b'*' | b'/' | b'%'
                    );
                !next_is_eq && !prev_is_cmp
            }
            b':' => {
                // Not `::`, and not a type annotation (RHS must be a number).
                let next_is_colon = i + 1 < b.len() && b[i + 1] == b':';
                let prev_is_colon = i > 0 && b[i - 1] == b':';
                !next_is_colon && !prev_is_colon
            }
            _ => false,
        };
        if is_op {
            let rhs = line[i + 1..].trim_start();
            if rhs.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                if let Some(name) = assignment_target(&line[..i]) {
                    if is_ceiling_shaped_name(&name) {
                        hits.push(name);
                    }
                }
            }
        }
        i += 1;
    }
    hits
}

/// Shape B — a runtime comparison of a computed size against a numeric
/// literal, on the same line as a reference to instruction text. The shape of
/// "is this SKILL.md too big", inline, with no named constant at all.
fn inline_size_comparison_against_instruction_text(line: &str) -> bool {
    let names_instruction_text =
        line.contains("skills/") || line.contains("skills\\") || line.contains("SKILL.md");
    if !names_instruction_text {
        return false;
    }
    for accessor in [".len()", ".size", ".chars().count()", "byte_len()"] {
        let mut from = 0;
        while let Some(idx) = line[from..].find(accessor) {
            let after = &line[from + idx + accessor.len()..];
            let after = after.trim_start();
            let mut rest = after;
            let mut saw_cmp = false;
            if let Some(stripped) = rest.strip_prefix(">=").or_else(|| rest.strip_prefix("<=")) {
                rest = stripped;
                saw_cmp = true;
            } else if let Some(stripped) = rest.strip_prefix('>').or_else(|| rest.strip_prefix('<')) {
                rest = stripped;
                saw_cmp = true;
            }
            if saw_cmp {
                let digits: String = rest
                    .trim_start()
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '_')
                    .filter(|c| *c != '_')
                    .collect();
                if digits.len() >= 2 {
                    return true;
                }
            }
            from += idx + accessor.len();
        }
    }
    false
}

/// The law's SUBJECT is instruction text, not sizes in general. The Node
/// original could leave shape A unqualified because `scripts/**` happened to
/// contain no size constants at all; the Rust runtime legitimately does
/// (`CONTENTION_TAIL_MAX_BYTES` is a log-tail read window,
/// `LEARNED_CONTEXT_MAX_LINES` a prompt-payload cap — neither is a standing
/// law on authored instruction text). So a shape-A hit must also carry
/// evidence that its subject IS the instruction layer: in the file path, in
/// the identifier, or in the declaration's own doc comment.
///
/// This still catches what budget-fence-removal deleted — the fence lived in
/// `scripts/skill_budget_fence.mjs` (path evidence) and its baseline in
/// `scripts/skill-body-budget.json` — and it still catches a reintroduction
/// under a brand-new name, because a fence on skill bodies cannot be written
/// without naming skills somewhere in those three places.
const INSTRUCTION_SUBJECT_TOKENS: &[&str] = &[
    "skill",
    "instruction",
    "doctrine",
    "expertise",
    "agents.md",
    "claude.md",
];
const SUBJECT_COMMENT_LOOKBACK: usize = 6;

fn has_instruction_subject_evidence(rel: &str, lines: &[&str], idx: usize, name: &str) -> bool {
    let mut haystack = rel.to_ascii_lowercase();
    haystack.push('\n');
    haystack.push_str(&name.to_ascii_lowercase());
    haystack.push('\n');
    let start = idx.saturating_sub(SUBJECT_COMMENT_LOOKBACK);
    for line in &lines[start..=idx] {
        haystack.push_str(&line.to_ascii_lowercase());
        haystack.push('\n');
    }
    INSTRUCTION_SUBJECT_TOKENS
        .iter()
        .any(|t| haystack.contains(t))
}

fn size_law_shape_hits(rel: &str, text: &str) -> Vec<(usize, String, String)> {
    let mut hits = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        for name in ceiling_shaped_identifiers(line) {
            if !has_instruction_subject_evidence(rel, &lines, i, &name) {
                continue;
            }
            hits.push((
                i + 1,
                format!("ceiling-shaped identifier \"{name}\""),
                line.trim().to_string(),
            ));
        }
        if inline_size_comparison_against_instruction_text(line) {
            hits.push((
                i + 1,
                "inline size comparison against instruction text".to_string(),
                line.trim().to_string(),
            ));
        }
    }
    hits
}

#[test]
fn no_size_ceiling_on_instruction_text_survives_in_any_shipped_tooling_tree() {
    let root = repo_root();
    let set = require(collect_scan_set(
        &root,
        "LAW A invariant 1 — ceiling-shaped constructs",
        "the shipped tooling tree: the Rust source tree (packages/bee-rs/crates/**/*.rs). The Node \
         script tree that used to join it here was deleted at the R6 cutover; every tool it held \
         now lives in the Rust tree, so that is the one place the defect class can reappear",
        &[RUST_TREE],
        40,
    ));

    let mut offenders = Vec::new();
    for (rel, text) in &set.files {
        for (line, rule, snippet) in size_law_shape_hits(rel, text) {
            offenders.push(format!("{rel}:{line} ({rule}): {snippet}"));
        }
    }
    assert!(
        offenders.is_empty(),
        "{}: a size ceiling on instruction text is never a standing law here \
         (budget-fence-removal D1, decision 8f63adb4) — found {} offender(s) across {} scanned \
         file(s):\n{}",
        set.label,
        offenders.len(),
        set.files.len(),
        offenders.join("\n")
    );
}

/// Shape C — a JSON table whose values are all-but-metadata numbers, keyed at
/// least in part by something skill-shaped: the shape
/// `scripts/skill-body-budget.json` had (a per-skill byte baseline), under
/// any filename, in any tree.
fn baseline_shape_hits(value: &serde_json::Value, key_path: &str) -> Vec<String> {
    let mut hits = Vec::new();
    let serde_json::Value::Object(map) = value else {
        return hits;
    };
    let entries: Vec<_> = map.iter().collect();
    let numeric: Vec<_> = entries.iter().filter(|(_, v)| v.is_number()).collect();
    let skill_keyed_numeric = entries.iter().any(|(k, v)| {
        v.is_number() && (k.to_ascii_lowercase().contains("skill") || k.ends_with("SKILL.md"))
    });
    if numeric.len() >= 2
        && !entries.is_empty()
        && (numeric.len() as f64) / (entries.len() as f64) >= 0.5
        && skill_keyed_numeric
    {
        hits.push(if key_path.is_empty() {
            "<root>".to_string()
        } else {
            key_path.to_string()
        });
    }
    for (k, v) in entries {
        let next = if key_path.is_empty() {
            k.clone()
        } else {
            format!("{key_path}.{k}")
        };
        hits.extend(baseline_shape_hits(v, &next));
    }
    hits
}

#[test]
fn no_per_skill_byte_baseline_table_survives_in_any_shipped_tooling_tree() {
    let root = repo_root();
    let set = require(collect_scan_set(
        &root,
        "LAW A invariant 1 — per-skill byte-baseline tables",
        "JSON data files in the shipped tooling trees — the genre \
         scripts/skill-body-budget.json belonged to, wherever it might reappear",
        &[
            Root {
                rel: "packages/bee-rs/crates",
                exts: &["json"],
                min_files: 1,
                required: true,
            },
            // R6 cutover: this root used to be `scripts` (optional), where the
            // deleted scripts/skill-body-budget.json lived. `scripts/` is now
            // the two installers and nothing else, so a JSON root there would
            // be permanently empty — a vacuous pass wearing a green tick. The
            // law re-points at the shipped TEMPLATE tree, which is both a
            // "shipped tooling tree" and where a data table would land today.
            Root {
                rel: "packages/bee",
                exts: &["json"],
                min_files: 1,
                required: true,
            },
            Root {
                rel: "skills",
                exts: &["json"],
                min_files: 0,
                required: true,
            },
        ],
        1,
    ));

    let mut offenders = Vec::new();
    for (rel, text) in &set.files {
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) else {
            continue; // other suites own JSON validity
        };
        for hit in baseline_shape_hits(&parsed, "") {
            offenders.push(format!("{rel} @ {hit}"));
        }
    }
    assert!(
        offenders.is_empty(),
        "{}: found {} per-skill byte-baseline-shaped table(s) across {} scanned file(s):\n{}",
        set.label,
        offenders.len(),
        set.files.len(),
        offenders.join("\n")
    );
}

#[test]
fn law_a_detectors_bite_on_reintroductions_under_brand_new_names() {
    // Shape A, three spellings a reintroduction could take. Each is checked
    // end to end through `size_law_shape_hits`, so the SUBJECT qualification
    // (path / identifier / doc comment) is exercised too, not just the
    // arithmetic shape.
    assert!(
        !size_law_shape_hits(
            "src/devtools/whatever.rs",
            "const SKILL_TEXT_CEILING_BYTES: usize = 9001;"
        )
        .is_empty(),
        "must flag a renamed byte-ceiling constant (subject named in the identifier), not just the \
         two deleted names"
    );
    assert!(
        !size_law_shape_hits("src/devtools/skill_trees.rs", "    line_budget: 200,").is_empty(),
        "must flag a ceiling declared as a struct field (subject named by the file path)"
    );
    assert!(
        !size_law_shape_hits(
            "src/devtools/whatever.rs",
            "/// how big a SKILL.md body may get\nlet max_body_bytes = 15000;"
        )
        .is_empty(),
        "must flag a ceiling declared as a plain let binding (subject named in the doc comment)"
    );

    // Controls on the arithmetic shape — a size-unit name with no limiting
    // concept, and a limiting concept with no size unit, are both legitimate.
    assert!(
        ceiling_shaped_identifiers("const BYTES_PER_CHUNK: usize = 4096;").is_empty(),
        "a size-unit name with no limiting concept must not be flagged"
    );
    assert!(
        ceiling_shaped_identifiers("const MAX_RETRIES: usize = 5;").is_empty(),
        "a limit with no size unit must not be flagged"
    );
    assert!(
        ceiling_shaped_identifiers("if byte_limit == 3 { }").is_empty(),
        "a comparison is not an assignment"
    );

    // Control on the SUBJECT qualification — the exact shape the live Rust
    // tree carries today (a log-tail read window, a prompt-payload cap).
    // These are ceiling-SHAPED but their subject is not instruction text, and
    // flagging them would make this law noise that gets suppressed.
    assert!(
        size_law_shape_hits(
            "src/verbs/status_full.rs",
            "// bee.mjs ~819-821\nconst CONTENTION_TAIL_MAX_BYTES: u64 = 65536;"
        )
        .is_empty(),
        "a size cap whose subject is NOT instruction text is out of this law's scope"
    );
    assert!(
        !ceiling_shaped_identifiers("const CONTENTION_TAIL_MAX_BYTES: u64 = 65536;").is_empty(),
        "…but the arithmetic detector must still see it — the sparing is the subject test doing \
         its job, not the shape detector failing"
    );

    // Shape B — an inline ceiling with no named constant at all.
    assert!(
        inline_size_comparison_against_instruction_text(
            "if std::fs::read_to_string(\"skills/renamed/SKILL.md\").unwrap().len() > 9001 { panic!() }"
        ),
        "the detector must flag an inline size-vs-instruction-text comparison"
    );
    assert!(
        !inline_size_comparison_against_instruction_text("if payload.len() > 9001 { bail!() }"),
        "a size comparison NOT against instruction text is out of this law's scope"
    );
    assert!(
        !inline_size_comparison_against_instruction_text("let s = read(\"skills/x/SKILL.md\");"),
        "merely naming instruction text is not a ceiling"
    );

    // Shape C — a renamed per-skill byte table.
    let table: serde_json::Value = serde_json::json!({
        "note": "a renamed baseline, not scripts/skill-body-budget.json",
        "skills/renamed-skill/SKILL.md": 4096,
        "skills/other-skill/SKILL.md": 2048,
    });
    assert!(
        !baseline_shape_hits(&table, "").is_empty(),
        "the detector must flag a renamed per-skill byte-baseline table"
    );
    // Control — a numeric table not keyed by anything skill-shaped is fine.
    let benign: serde_json::Value = serde_json::json!({ "a": 1, "b": 2 });
    assert!(
        baseline_shape_hits(&benign, "").is_empty(),
        "a numeric table with no skill-shaped key is not this law's subject"
    );
}

// ════════════════════════════════════════════════════════════════════════
// LAW B, CHECK 1 (E4) — an unguarded git-index scan set
// ════════════════════════════════════════════════════════════════════════
//
// `git ls-files` reflects the INDEX, not the working tree: a file deleted on
// disk but not yet staged still comes back. Reading such a path crashes
// whatever gate derived its scan set that way — what happened to
// test_doctrine_parity.mjs (vd-13) and sat live in test_portable_paths.mjs
// until dch-4. Source pattern:
// docs/knowledge/patterns/20260728-a-scan-set-from-the-git-index-crashes-the-gate-that-guards-it.md
//
// TRANSLATED, not transliterated, into Rust semantics. In Node the crash
// shape is a bare `readFileSync` on the derived path. In Rust `read_to_string`
// returns a `Result`, so the equivalent crash shape is a read of a
// scan-set-derived path that is `.unwrap()`ed or `.expect()`ed. The guard the
// law demands is the same: an existence filter between deriving the list and
// reading from it.
//
// The subject is REAL in the Rust tree, not hypothetical:
// `src/onboard/notices.rs::tracked_gitignore_paths` genuinely invokes
// `git ls-files`. It is the known-good "derives but never reads" shape, and
// this check must spare it — which is exactly what makes a PASS here mean
// something.

const READ_CALLS: &[&str] = &[
    "read_to_string(",
    "fs::read(",
    "File::open(",
    "fs::copy(",
    "read_dir(",
    "metadata(",
];
const PANIC_MARKERS: &[&str] = &[".unwrap()", ".expect("];
const EXISTENCE_GUARDS: &[&str] = &[
    "exists()",
    "try_exists",
    "is_file()",
    "symlink_metadata",
    "if let Ok(",
    "ok()",
];

/// A real `git ls-files` INVOCATION, not a bare co-occurrence of the two
/// words: `hooks/write_guard.rs` lists "ls-files" in a table of read-only git
/// subcommands, nowhere near an invocation, and must not trip this.
fn git_ls_files_invocation_lines(text: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(idx) = text[from..].find("Command::new(\"git\")") {
        let abs = from + idx;
        let end = (abs + 500).min(text.len());
        let mut end = end;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        if text[abs..end].contains("ls-files") {
            out.push(text[..abs].lines().count());
        }
        from = abs + 1;
    }
    out
}

/// The enclosing `fn` body (inclusive line range), located by scanning
/// backward for a function-opening line and forward until braces balance.
/// A heuristic — it does not understand strings or comments — but every real
/// candidate in this tree is plainly formatted enough that it holds, and the
/// selftest below proves both the biting and the sparing arms.
fn enclosing_fn_range(lines: &[&str], match_line_idx: usize) -> Option<(usize, usize)> {
    let is_fn_start = |l: &str| {
        let t = l.trim_start();
        t.starts_with("fn ")
            || t.starts_with("pub fn ")
            || t.starts_with("pub(crate) fn ")
            || t.starts_with("pub(super) fn ")
            || t.starts_with("async fn ")
            || t.starts_with("pub async fn ")
            || t.starts_with("unsafe fn ")
    };
    let mut start = None;
    for i in (0..=match_line_idx.min(lines.len().saturating_sub(1))).rev() {
        if is_fn_start(lines[i]) {
            start = Some(i);
            break;
        }
    }
    let start = start?;
    let mut depth: i32 = 0;
    let mut seen_open = false;
    for (i, line) in lines.iter().enumerate().skip(start) {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                seen_open = true;
            } else if ch == '}' {
                depth -= 1;
            }
        }
        if seen_open && depth <= 0 {
            return Some((start, i));
        }
    }
    Some((start, lines.len() - 1))
}

fn find_unguarded_scan_sets(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut violations = Vec::new();
    for match_line in git_ls_files_invocation_lines(text) {
        let idx = match_line.saturating_sub(1);
        let (start, end) = enclosing_fn_range(&lines, idx)
            .unwrap_or((idx, (idx + 80).min(lines.len().saturating_sub(1))));
        let body: Vec<&str> = lines[start..=end.max(start)].to_vec();
        // A read that would panic on a stale index entry.
        let panicking_read = body.iter().any(|l| {
            READ_CALLS.iter().any(|r| l.contains(r)) && PANIC_MARKERS.iter().any(|p| l.contains(p))
        });
        if !panicking_read {
            continue; // derived but never read fatally here — E4 does not apply
        }
        let joined = body.join("\n");
        if !EXISTENCE_GUARDS.iter().any(|g| joined.contains(g)) {
            violations.push((
                match_line,
                "derives a path list from a `git ls-files` invocation and reads from it with a \
                 panicking read (.unwrap()/.expect()) with no existence filter in the enclosing \
                 function — `git ls-files` reflects the INDEX, so a file deleted on disk but not \
                 yet staged still comes back and the read aborts the process"
                    .to_string(),
            ));
        }
    }
    violations
}

#[test]
fn no_unguarded_git_index_scan_set_survives_in_the_rust_tree() {
    let root = repo_root();
    let set = require(collect_scan_set(
        &root,
        "LAW B check 1 — unguarded git-index scan set",
        "the Rust source tree (packages/bee-rs/crates/**/*.rs) — the tree that shells out to \
         `git ls-files` after the Node runtime is deleted (src/onboard/notices.rs does so today)",
        &[RUST_TREE],
        40,
    ));

    // The subject must actually be present, or this law is scanning a tree
    // where its defect class cannot occur — vacuity of a second kind.
    let invocation_sites: Vec<&String> = set
        .files
        .iter()
        .filter(|(_, text)| !git_ls_files_invocation_lines(text).is_empty())
        .map(|(rel, _)| rel)
        .collect();
    assert!(
        !invocation_sites.is_empty(),
        "{}: scanned {} file(s) and found NO `git ls-files` invocation at all. Either the Rust \
         tree stopped deriving scan sets from the git index (in which case retire this check with \
         a recorded reason in plans/cutover-readiness.md) or the invocation detector broke. \
         Passing over a subject that is not there is not a result.",
        set.label,
        set.files.len()
    );

    let mut failures = Vec::new();
    for (rel, text) in &set.files {
        for (line, reason) in find_unguarded_scan_sets(text) {
            failures.push(format!("{rel}:{line} — {reason}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{}: {} violation(s) across {} scanned file(s) ({} file(s) invoke `git ls-files`):\n{}\n\
         Filter the derived list through an existence check (chained on the list, or a per-item \
         guard before each read) in the same function that derives it — see \
         src/onboard/notices.rs::tracked_gitignore_paths.",
        set.label,
        failures.len(),
        set.files.len(),
        invocation_sites.len(),
        failures.join("\n")
    );
}

#[test]
fn law_b_check1_bites_on_an_unguarded_derivation_and_spares_the_known_good_shapes() {
    // Violation: derives from `git ls-files`, reads with a panicking read, no
    // existence filter anywhere in the enclosing function.
    let violation = r#"
fn load_all(root: &Path) -> Vec<String> {
    let out = Command::new("git").arg("ls-files").arg("-z").current_dir(root).output().unwrap();
    let files: Vec<String> = String::from_utf8_lossy(&out.stdout).split('\0').map(str::to_string).collect();
    let mut acc = Vec::new();
    for rel in files {
        acc.push(std::fs::read_to_string(root.join(rel)).unwrap());
    }
    acc
}
"#;
    assert_eq!(
        find_unguarded_scan_sets(violation).len(),
        1,
        "check 1 did not flag the unguarded git-ls-files-then-panicking-read fixture"
    );

    // Known-good A: a chained existence filter on the derived list.
    let good_chained = r#"
fn derive_scan_set(root: &Path) -> Vec<String> {
    let out = Command::new("git").arg("ls-files").arg("-z").current_dir(root).output().unwrap();
    let mut v: Vec<String> = String::from_utf8_lossy(&out.stdout).split('\0').map(str::to_string).collect();
    v.retain(|p| root.join(p).exists());
    let mut acc = Vec::new();
    for rel in &v {
        acc.push(std::fs::read_to_string(root.join(rel)).unwrap());
    }
    acc
}
"#;
    assert!(
        find_unguarded_scan_sets(good_chained).is_empty(),
        "check 1 false-flagged the chained-existence-filter known-good shape"
    );

    // Known-good B: derives a list and never reads from disk with it at all
    // — this is `src/onboard/notices.rs::tracked_gitignore_paths`'s real
    // shape. No guard is needed because there is nothing to guard against.
    let good_no_read = r#"
fn tracked_gitignore_paths(repo_root: &Path) -> Vec<String> {
    let out = Command::new("git").arg("ls-files").arg("-z").arg("--").current_dir(repo_root).output();
    let Ok(out) = out else { return Vec::new() };
    String::from_utf8_lossy(&out.stdout).split('\0').filter(|s| !s.is_empty()).map(str::to_string).collect()
}
"#;
    assert!(
        find_unguarded_scan_sets(good_no_read).is_empty(),
        "check 1 false-flagged the derives-but-never-reads known-good shape"
    );

    // Known-good C: a table that merely NAMES "ls-files" (the write guard's
    // read-only git subcommand roster) is not an invocation.
    let good_roster = r#"
const READ_ONLY_GIT: &[&str] = &["status", "log", "diff", "show", "ls-files", "check-ignore"];
"#;
    assert!(
        git_ls_files_invocation_lines(good_roster).is_empty(),
        "a roster naming `ls-files` must never be read as an invocation"
    );
}

// ════════════════════════════════════════════════════════════════════════
// LAW B, CHECK 2 (E8) — a retired workflow stage described as CURRENT
// ════════════════════════════════════════════════════════════════════════
//
// derived-check-hardening E8. D11's acceptance criterion said "no live file
// may describe a retired stage as current", but it was a manual sweep run
// once, and two residuals slipped through it (cleaned up later in dch-7).
//
// The SCAN SET here survives the cutover untouched — shipped prose is shipped
// prose. What dies is the TOKEN SOURCE: the Node original derives the retired
// stage name(s) from `LEGACY_PHASE_COERCIONS` in packages/bee/lib/state.mjs.
// So the derivation is re-pointed at the Rust tree's own coercion record, and
// cross-checked against Node's while both exist — meaning the Rust derivation
// is continuously proven correct BEFORE the day it becomes the only one.

const RETIREMENT_MARKERS: &[&str] = &[
    "retir", "delet", "legacy", "supersed", "deprecat", "coerc",
];
const MARKER_WINDOW_CHARS: usize = 250;

/// Retired-phase tokens as the RUST tree records them: each coercion site
/// carries the mapping in its own doc comment (`coerceLegacyPhase: 'X' ->
/// 'Y'`), which is the Rust port's equivalent of Node's
/// LEGACY_PHASE_COERCIONS map. Deriving from the record — never from a name
/// written into this law — is what lets the check survive the next phase
/// retirement unmodified.
fn rust_retired_phase_tokens(files: &[(String, String)]) -> Vec<String> {
    let mut tokens = Vec::new();
    for (_, text) in files {
        for line in text.lines() {
            if !line.contains("coerceLegacyPhase") {
                continue;
            }
            for quote in ['\'', '"'] {
                let parts: Vec<&str> = line.split(quote).collect();
                // Odd indices are quoted spans.
                let mut i = 1;
                while i < parts.len() {
                    let after = parts.get(i + 1).map(|s| s.trim_start()).unwrap_or("");
                    if after.starts_with("->") || after.starts_with('\u{2192}') {
                        let tok = parts[i].trim();
                        if !tok.is_empty() && !tokens.contains(&tok.to_string()) {
                            tokens.push(tok.to_string());
                        }
                    }
                    i += 2;
                }
            }
        }
    }
    tokens.sort();
    tokens
}

/// The same tokens as the NODE tree records them, while `state.mjs` exists.
/// `None` once it is gone.
fn node_retired_phase_tokens(root: &Path) -> Option<Vec<String>> {
    let src = std::fs::read_to_string(root.join("packages/bee/lib/state.mjs")).ok()?;
    let start = src.find("LEGACY_PHASE_COERCIONS")?;
    let open = src[start..].find('{')? + start;
    let close = src[open..].find('}')? + open;
    let body = &src[open + 1..close];
    let mut tokens = Vec::new();
    for entry in body.split(',') {
        let Some((key, _)) = entry.split_once(':') else {
            continue;
        };
        let key = key.trim().trim_matches(['\'', '"', '`']).trim();
        if !key.is_empty() && !tokens.contains(&key.to_string()) {
            tokens.push(key.to_string());
        }
    }
    tokens.sort();
    Some(tokens)
}

fn floor_boundary(s: &str, mut i: usize) -> usize {
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn ceil_boundary(s: &str, mut i: usize) -> usize {
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i.min(s.len())
}

fn is_word_boundary(text: &str, start: usize, end: usize) -> bool {
    let before_ok = text[..start]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '-');
    let after_ok = text[end..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '-');
    before_ok && after_ok
}

/// Occurrences of a retired stage token described as CURRENT: either the
/// `bee-<token>` skill-name form (routing to a skill that no longer exists)
/// or a bare quoted phase literal, with no retirement acknowledgment within
/// MARKER_WINDOW_CHARS on either side. A bare name match cannot tell
/// "bee-validating is deleted (validation-diet D1)" — every live mention in
/// this tree today — from the real E8 shape "route proof obligations to
/// bee-validating"; the marker window is what distinguishes them.
fn find_current_behavior_mentions(text: &str, token: &str) -> Vec<(usize, String)> {
    let needles = [
        format!("bee-{token}"),
        format!("'{token}'"),
        format!("\"{token}\""),
    ];
    let mut hits: Vec<(usize, String)> = Vec::new();
    let mut seen: Vec<usize> = Vec::new();
    for needle in &needles {
        let mut from = 0;
        while let Some(idx) = text[from..].find(needle.as_str()) {
            let abs = from + idx;
            from = abs + 1;
            let end = abs + needle.len();
            if needle.starts_with("bee-") && !is_word_boundary(text, abs, end) {
                continue;
            }
            if seen.contains(&abs) {
                continue;
            }
            let ws = floor_boundary(text, abs.saturating_sub(MARKER_WINDOW_CHARS));
            let we = ceil_boundary(text, (end + MARKER_WINDOW_CHARS).min(text.len()));
            let window = text[ws..we].to_ascii_lowercase();
            if RETIREMENT_MARKERS.iter().any(|m| window.contains(m)) {
                continue;
            }
            seen.push(abs);
            let line = text[..abs].lines().count();
            hits.push((line, needle.clone()));
        }
    }
    hits.sort();
    hits
}

/// The CURRENT-behavior instructional surface: the files whose job is to
/// state what the system does TODAY, as opposed to narrating how it got
/// there. This is the R6a instruction layer — it survives the Node deletion
/// intact, and `expertise/**` is included here where the Node original
/// covered only `skills/**` (rust-port.md R6a names both halves).
///
/// Deliberately excluded, as in the Node original: narrative genres this repo
/// uses constantly and legitimately to discuss retired things by name while
/// explaining the retirement — docs/history/**, docs/decisions/**,
/// CREATION-LOG.md, README.md — plus test corpora that exercise the
/// legacy-coercion path itself.
fn current_behavior_roots() -> Vec<Root> {
    vec![
        Root {
            rel: "skills",
            exts: &["md"],
            min_files: 9,
            required: true,
        },
        Root {
            rel: "expertise",
            exts: &["md"],
            min_files: 5,
            required: true,
        },
        Root {
            rel: "docs/knowledge",
            exts: &["md"],
            min_files: 50,
            required: true,
        },
        Root {
            rel: "docs/specs",
            exts: &["md"],
            min_files: 5,
            required: true,
        },
    ]
}

fn is_excluded_narrative(rel: &str) -> bool {
    rel.ends_with("CREATION-LOG.md") || rel.ends_with("README.md")
}

#[test]
fn no_current_behavior_file_describes_a_retired_stage_as_current() {
    let root = repo_root();
    let mut set = require(collect_scan_set(
        &root,
        "LAW B check 2 — retired workflow stage described as current",
        "the shipped current-behavior instruction layer: skills/**, expertise/**, \
         docs/knowledge/**, docs/specs/**, plus AGENTS.md and CLAUDE.md — the surface an agent \
         actually reads to act, which SURVIVES the Node-runtime deletion",
        &current_behavior_roots(),
        200,
    ));
    set.files.retain(|(rel, _)| !is_excluded_narrative(rel));

    // The two always-loaded files, added by name (they are not under a root).
    for name in ["AGENTS.md", "CLAUDE.md"] {
        let text = std::fs::read_to_string(root.join(name)).unwrap_or_else(|e| {
            panic!(
                "{}: {name} must exist — it is loaded every session and is the single highest-\
                 traffic current-behavior file in the repo ({e})",
                set.label
            )
        });
        set.files.push((name.to_string(), text));
    }

    // ── Token derivation, cross-checked across both trees ────────────────
    let rust_tree = require(collect_scan_set(
        &root,
        "LAW B check 2 — retired-phase token source",
        "the Rust tree's own coercion record (`coerceLegacyPhase: 'X' -> 'Y'` at each site)",
        &[RUST_TREE],
        40,
    ));
    let rust_tokens = rust_retired_phase_tokens(&rust_tree.files);
    let node_tokens = node_retired_phase_tokens(&root);

    let tokens = match (&rust_tokens[..], &node_tokens) {
        ([], None) => panic!(
            "{}: DERIVED ZERO RETIRED-PHASE TOKENS. Looked in (a) the Rust tree for \
             `coerceLegacyPhase: 'X' -> 'Y'` coercion records and (b) \
             packages/bee/lib/state.mjs for LEGACY_PHASE_COERCIONS. Neither yielded a token, so \
             this check would scan {} file(s) for nothing and report a vacuous PASS. Either the \
             coercion record moved (re-point this derivation) or the phase enum stopped carrying \
             legacy coercions at all (retire this check with a recorded reason in \
             plans/cutover-readiness.md).",
            set.label,
            set.files.len()
        ),
        ([], Some(n)) => panic!(
            "{}: the Rust tree yielded NO retired-phase tokens while Node still records {n:?}. \
             The Rust coercion sites must carry their mapping in a `coerceLegacyPhase: 'X' -> 'Y'` \
             record, or this check goes vacuous the moment packages/bee/ is deleted.",
            set.label
        ),
        (r, Some(n)) => {
            assert_eq!(
                r,
                &n[..],
                "{}: the Rust and Node retired-phase records DISAGREE (rust={r:?}, node={n:?}). \
                 They are the same contract; a divergence means one tree coerces a phase the \
                 other does not, and whichever survives the cutover would carry the wrong law.",
                set.label
            );
            r.to_vec()
        }
        (r, None) => r.to_vec(),
    };
    assert!(!tokens.is_empty(), "{}: empty token set", set.label);

    let mut failures = Vec::new();
    for (rel, text) in &set.files {
        for token in &tokens {
            for (line, phrase) in find_current_behavior_mentions(text, token) {
                failures.push(format!(
                    "{rel}:{line} — \"{phrase}\" (retired token \"{token}\") with no retirement \
                     acknowledgment nearby"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{}: {} mention(s) across {} current-behavior file(s), tokens [{}]:\n{}\n\
         Either the reference is stale (route it to whatever now carries that role) or the file \
         needs a retirement note nearby (\"retired\"/\"deleted\"/\"legacy\").",
        set.label,
        failures.len(),
        set.files.len(),
        tokens.join(", "),
        failures.join("\n")
    );
}

#[test]
fn law_b_check2_derivation_and_classifier_are_proven_on_fixtures() {
    // Derivation: TWO tokens, not one — proves it is not silently pinned to a
    // single hardcoded name.
    let fixture = vec![(
        "fixture.rs".to_string(),
        "    // coerceLegacyPhase: 'qualifying' -> 'exploring' (D13).\n\
         /// state.mjs coerceLegacyPhase (D13): 'staging' \u{2192} 'swarming'.\n"
            .to_string(),
    )];
    let tokens = rust_retired_phase_tokens(&fixture);
    assert_eq!(
        tokens,
        vec!["qualifying".to_string(), "staging".to_string()],
        "the derivation must read every coercion record, in both the ASCII and the arrow spelling"
    );
    // A line that merely mentions the phases without the record marker is not
    // a source of tokens.
    assert!(
        rust_retired_phase_tokens(&[(
            "x.rs".to_string(),
            "if phase == json!(\"validating\") { }".to_string()
        )])
        .is_empty(),
        "a bare comparison is not a coercion record"
    );

    // Classifier: the real E8 shape — a live reference routing to a retired
    // skill with no acknowledgment anywhere near it.
    let violation = "Open questions (for the user, or as proof obligations for bee-qualifying):\n";
    let hits = find_current_behavior_mentions(violation, "qualifying");
    assert_eq!(
        hits.len(),
        1,
        "check 2 did not flag the unacknowledged bee-<retired> reference: {hits:?}"
    );

    // Known-good: the same reference, retirement acknowledged in the same
    // breath (the shape every live mention in this repo actually uses).
    let good = "`bee-qualifying` is deleted (some-feature D1); route proof obligations to \
                `bee-planning` instead.\n";
    assert!(
        find_current_behavior_mentions(good, "qualifying").is_empty(),
        "check 2 false-flagged an acknowledged-as-deleted mention"
    );

    // Known-good: a bare phase literal framed as legacy.
    let coercion = "assert phase == 'planning' — the legacy phase 'staging' must coerce to it\n";
    assert!(
        find_current_behavior_mentions(coercion, "staging").is_empty(),
        "check 2 false-flagged a legacy-acknowledged phase literal"
    );

    // Known-good: `bee-qualifying-extra` is a different word.
    assert!(
        find_current_behavior_mentions("see bee-qualifyingx for details\n", "qualifying").is_empty(),
        "check 2 must respect word boundaries on the bee-<token> form"
    );

    // Multi-byte safety: the window slicer must not split a UTF-8 char.
    let unicode = format!("{}bee-qualifying{}", "\u{2014} ".repeat(200), " \u{2192}".repeat(200));
    let _ = find_current_behavior_mentions(&unicode, "qualifying");
}
