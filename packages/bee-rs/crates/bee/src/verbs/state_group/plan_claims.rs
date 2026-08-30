// The load-bearing claims precondition (feature existence-is-not-evidence,
// CONTEXT.md D1/D2).
//
// WHAT IT IS. plan.md carries a `## Load-bearing claims` table: one row per
// factual assertion the plan's shape depends on, each row naming the claim, an
// evidence LABEL (`read` / `ran` / `guessed`), the ANCHOR it was taken from,
// and the verbatim bytes seen there. D2 makes that table a door rather than a
// suggestion: the shape/merged gate refuses while the table is missing,
// malformed, or still carrying a `guessed` row. There is no waiver flag — the
// remedy is to go touch reality (upgrade the label with a real read or run) or
// to admit the uncertainty (move the claim to `## Open Questions`).
//
// WHY THE RULES ARE ONLY MECHANICAL. The binary cannot judge prose, and it
// deliberately does not try: it never decides whether a claim IS load-bearing
// (every row in the table is, by definition) and it never matches the quote
// against the anchored bytes. Quote matching is the `hat-facts-gaps` seat's
// job at the plan step (D4) — a second reader, not a parser. What is left here
// is exactly what a parser can prove: the table exists, it has the four
// columns, it has rows, every row is filled in, every label is in the
// vocabulary, no label is `guessed`, and a `read` row's `path:line` anchor
// points at a path that really exists. The residual (a fabricated quote at a
// real path) is named and accepted in plan.md's Approach.
//
// FAIL-CLOSED, like its two neighbours in `set_gate.rs`
// (`high_risk_advisor_refusal`, `conflict_review_refusal`). The one deliberate
// opening is `ErrorKind::NotFound`: tiny and small lanes legitimately have no
// plan.md, so a missing file makes the precondition INAPPLICABLE. Every other
// read failure — a directory in plan.md's place, a permission denial, invalid
// UTF-8 — REFUSES. "I could not read the evidence" is never "the evidence is
// fine".

use std::io::ErrorKind;
use std::path::Path;

use super::advisor_plan_path;
use super::io_read_reason;

/// The section heading the table must live under, quoted verbatim in every
/// refusal so the message teaches the shape it wants.
pub(crate) const CLAIMS_HEADING: &str = "## Load-bearing claims";

/// The whole label vocabulary. Matching is EXACT and lowercase: `Read` is a
/// refusal, not a near-miss, because a label the parser has to guess at is the
/// same ambiguity this feature exists to delete.
const LABELS: [&str; 3] = ["read", "ran", "guessed"];

/// At most this many problems are listed before the message stops counting
/// them out. A plan with thirty broken rows needs the shape and the remedy,
/// not thirty sentences.
const MAX_LISTED: usize = 8;

/// The refusal text for `docs/history/<feature>/plan.md`, or `None` when the
/// approval may proceed. `None` covers both "the table is fine" and "there is
/// no plan.md at all" — see the module header for why those share an answer.
pub(crate) fn claims_refusal(root: &Path, feature: &str) -> Option<String> {
    let path = advisor_plan_path(root, feature);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        // The one opening: tiny/small lanes have no plan.md to check.
        Err(e) if e.kind() == ErrorKind::NotFound => return None,
        Err(e) => {
            return Some(refusal(
                feature,
                &[format!(
                    "plan.md is present but could not be read ({})",
                    io_read_reason(&path, &e)
                )],
            ));
        }
    };
    let problems = claims_problems(&text, root);
    if problems.is_empty() { None } else { Some(refusal(feature, &problems)) }
}

// ─── rules ─────────────────────────────────────────────────────────────────

/// Every reason this plan's claims table does not stand behind an approval, in
/// document order. Empty means the table is sound.
fn claims_problems(text: &str, root: &Path) -> Vec<String> {
    let Some(section) = claims_section(text) else {
        return vec![format!("plan.md has no \"{CLAIMS_HEADING}\" section")];
    };
    let lines: Vec<&str> = section
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('|'))
        .collect();
    if lines.len() < 2 {
        return vec![format!(
            "the \"{CLAIMS_HEADING}\" section carries no markdown table"
        )];
    }
    let header = split_row(lines[0]);
    if !is_separator_row(lines[1]) {
        return vec!["the claims table has no header separator row".to_string()];
    }
    let mut problems: Vec<String> = Vec::new();
    let mut found: Vec<usize> = Vec::new();
    for (keyword, shown) in
        [("claim", "Claim"), ("label", "Label"), ("anchor", "Anchor"), ("evidence", "Verbatim evidence")]
    {
        match header.iter().position(|c| c.to_ascii_lowercase().contains(keyword)) {
            Some(i) => found.push(i),
            None => problems.push(format!("the claims table has no \"{shown}\" column")),
        }
    }
    if !problems.is_empty() {
        return problems;
    }
    // Indexing is safe: `problems` is empty, so all four columns resolved.
    let (claim, label, anchor, evidence) = (found[0], found[1], found[2], found[3]);

    let rows: Vec<Vec<String>> = lines[2..]
        .iter()
        .map(|l| split_row(l))
        .filter(|cells| cells.iter().any(|c| !c.is_empty()))
        .collect();
    if rows.is_empty() {
        return vec![format!(
            "the \"{CLAIMS_HEADING}\" table has zero rows \u{2014} a plan with no load-bearing claim to declare says so in prose, never with an empty table"
        )];
    }
    for (i, cells) in rows.iter().enumerate() {
        // Row numbers are the reader's numbers: the first DATA row is row 1,
        // which is also what a well-formed `#` column counts.
        let n = i + 1;
        for (idx, shown) in
            [(claim, "Claim"), (label, "Label"), (anchor, "Anchor"), (evidence, "Verbatim evidence")]
        {
            if cell(cells, idx).is_empty() {
                problems.push(format!("row {n} has an empty \"{shown}\" cell"));
            }
        }
        let label = cell(cells, label);
        if label.is_empty() {
            continue; // already reported as an empty cell; no vocabulary noise
        }
        if !LABELS.contains(&label) {
            problems.push(format!(
                "row {n}'s label \"{label}\" is not one of read / ran / guessed"
            ));
            continue;
        }
        if label == "guessed" {
            problems.push(format!(
                "row {n} is still labeled `guessed` \u{2014} the plan's shape rests on something nobody looked at"
            ));
            continue;
        }
        // A `read` row promises somebody opened that file at that line. The
        // cheapest half of that promise is checkable: the file exists.
        if label == "read" {
            if let Some(p) = anchor_path(cell(cells, anchor)) {
                if !root.join(p).exists() {
                    problems.push(format!(
                        "row {n} is labeled `read` but its anchor path \"{p}\" does not exist under the repo root"
                    ));
                }
            }
        }
    }
    problems
}

/// One cell of a parsed row, trimmed, with an out-of-range index reading as
/// empty — a short row is a hole in the evidence, never a panic.
fn cell(cells: &[String], idx: usize) -> &str {
    cells.get(idx).map(String::as_str).unwrap_or("").trim()
}

/// The body of the `## Load-bearing claims` section: everything after the
/// heading up to the next heading of any level. Heading matching ignores the
/// hash count and letter case so a level slip is not a refusal; the words must
/// match.
fn claims_section(text: &str) -> Option<&str> {
    let mut start: Option<usize> = None;
    let mut end = text.len();
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let here = offset;
        offset += line.len();
        let Some(title) = heading_title(line) else { continue };
        match start {
            None => {
                if title.eq_ignore_ascii_case("load-bearing claims") {
                    start = Some(offset);
                }
            }
            Some(_) => {
                end = here;
                break;
            }
        }
    }
    let start = start?;
    Some(&text[start..end.max(start)])
}

/// `## Some title` → `Some title`, for 1-6 hashes followed by whitespace.
/// Anything else is not a heading line.
fn heading_title(line: &str) -> Option<&str> {
    let t = line.trim();
    let hashes = t.len() - t.trim_start_matches('#').len();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &t[hashes..];
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some(rest.trim())
}

/// A markdown separator row: every cell is dashes with optional alignment
/// colons.
fn is_separator_row(line: &str) -> bool {
    let cells = split_row(line);
    !cells.is_empty()
        && cells.iter().all(|c| {
            !c.is_empty()
                && c.chars().all(|ch| ch == '-' || ch == ':')
                && c.contains('-')
        })
}

/// A table row's cells, outer pipes dropped, each cell trimmed. A `\|` is a
/// literal pipe inside a cell, not a separator — the escape survives into the
/// cell text, which only ever feeds messages and comparisons.
fn split_row(line: &str) -> Vec<String> {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    let mut cells: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut escaped = false;
    for ch in t.chars() {
        if escaped {
            cur.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => {
                cur.push(ch);
                escaped = true;
            }
            '|' => cells.push(std::mem::take(&mut cur).trim().to_string()),
            _ => cur.push(ch),
        }
    }
    cells.push(cur.trim().to_string());
    cells
}

/// The repo-relative path an anchor points at, when the anchor really is a
/// `path:line` (or `path:12-15`, or `path:12,30-32`) reference. Returns `None`
/// for anything else — a command, a prose anchor, a bare URL — because this
/// rule only ever fires on what it can read unambiguously. A backticked span
/// wins over the rest of the cell, so trailing commentary after the anchor is
/// harmless.
fn anchor_path(anchor: &str) -> Option<&str> {
    let cand = first_backticked(anchor).unwrap_or_else(|| anchor.trim());
    let (path, spec) = cand.rsplit_once(':')?;
    let (path, spec) = (path.trim(), spec.trim());
    if path.is_empty() || spec.is_empty() {
        return None;
    }
    if !spec.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '-') {
        return None;
    }
    if !spec.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(path)
}

/// The contents of the first `` `…` `` span in a cell.
fn first_backticked(cell: &str) -> Option<&str> {
    let open = cell.find('`')?;
    let rest = &cell[open + 1..];
    let close = rest.find('`')?;
    Some(rest[..close].trim())
}

// ─── the message ───────────────────────────────────────────────────────────

/// Self-serve by construction (plan.md U3): the offending rows, the shape that
/// was expected, and the two ways out. A reader who has never seen this door
/// must be able to fix the plan from this one paragraph.
fn refusal(feature: &str, problems: &[String]) -> String {
    let shown: Vec<&str> = problems.iter().take(MAX_LISTED).map(String::as_str).collect();
    let more = problems.len().saturating_sub(shown.len());
    let tail = if more > 0 { format!(" (and {more} more)") } else { String::new() };
    format!(
        "gate: approval refused \u{2014} the load-bearing claims table in docs/history/{feature}/plan.md \
         does not stand behind this plan (D1/D2). Problem(s): {}{tail}. \
         EXPECTED: a \"{CLAIMS_HEADING}\" section holding one markdown table with the columns \
         | # | Claim | Label | Anchor | Verbatim evidence |, at least one row, every cell filled, \
         and every Label exactly one of read / ran / guessed (lowercase) \u{2014} `read` = you opened \
         that file at that line, `ran` = you executed it and hold the output, `guessed` = inferred. \
         FIX: go touch reality and upgrade each offending row (open the file at its anchor, or run \
         the command and keep the output), or move the claim to \"## Open Questions\". There is no \
         waiver flag.",
        shown.join("; "),
    )
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn root_with_plan(body: &str) -> (tempfile::TempDir, String) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("history").join("demo");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("plan.md"), body).unwrap();
        (tmp, "demo".to_string())
    }

    /// A table body wrapped in the surrounding plan prose a real plan.md has.
    fn plan(table: &str) -> String {
        format!("# Plan: demo\n\n## Approach\n\nsomething\n\n{CLAIMS_HEADING}\n\nlead-in prose\n\n{table}\n\n## Shape\n\none cell\n")
    }

    const HEADER: &str = "| # | Claim | Label | Anchor | Verbatim evidence |\n|---|-------|-------|--------|-------------------|";

    fn check(body: &str) -> Option<String> {
        let (tmp, feature) = root_with_plan(body);
        claims_refusal(tmp.path(), &feature)
    }

    // ── the opening: no plan.md at all ─────────────────────────────────────

    #[test]
    fn a_missing_plan_md_makes_the_precondition_inapplicable() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(claims_refusal(tmp.path(), "tiny-lane"), None);
    }

    #[test]
    fn a_feature_whose_history_dir_exists_but_holds_no_plan_is_still_inapplicable() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("docs").join("history").join("demo")).unwrap();
        assert_eq!(claims_refusal(tmp.path(), "demo"), None);
    }

    // ── fail-closed: present but unreadable ────────────────────────────────

    #[test]
    fn an_unreadable_plan_md_refuses_rather_than_passing() {
        let tmp = tempfile::tempdir().unwrap();
        // Portable non-NotFound read error: a directory where plan.md goes.
        std::fs::create_dir_all(tmp.path().join("docs").join("history").join("demo").join("plan.md"))
            .unwrap();
        let msg = claims_refusal(tmp.path(), "demo").expect("an unreadable plan.md must refuse");
        assert!(msg.contains("could not be read"), "{msg}");
        assert!(msg.contains("directory"), "{msg}");
    }

    // ── structure rules ───────────────────────────────────────────────────

    #[test]
    fn a_plan_without_the_claims_heading_refuses() {
        let msg = check("# Plan: demo\n\n## Approach\n\nprose only\n").expect("no heading, no gate");
        assert!(msg.contains("no \"## Load-bearing claims\" section"), "{msg}");
    }

    #[test]
    fn the_heading_may_slip_a_level_but_not_a_word() {
        // Level slip: still found, and the table under it is judged.
        let body = format!(
            "# Plan\n\n### Load-bearing claims\n\n{HEADER}\n| 1 | c | guessed | a | e |\n"
        );
        let msg = check(&body).expect("a level-3 heading is still the claims section");
        assert!(msg.contains("row 1 is still labeled `guessed`"), "{msg}");
        // Wrong words: not the section at all.
        let body = format!("# Plan\n\n## Load bearing claims\n\n{HEADER}\n| 1 | c | read | a | e |\n");
        let msg = check(&body).expect("a differently-named section is not this one");
        assert!(msg.contains("no \"## Load-bearing claims\" section"), "{msg}");
    }

    #[test]
    fn a_claims_section_with_no_table_refuses() {
        let msg = check(&plan("we read everything, honest")).expect("prose is not a table");
        assert!(msg.contains("carries no markdown table"), "{msg}");
    }

    #[test]
    fn a_table_with_no_separator_row_refuses() {
        let body = plan("| # | Claim | Label | Anchor | Verbatim evidence |\n| 1 | c | read | a | e |");
        let msg = check(&body).expect("a header plus data with no separator is malformed");
        assert!(msg.contains("no header separator row"), "{msg}");
    }

    #[test]
    fn a_table_with_zero_rows_refuses() {
        let msg = check(&plan(HEADER)).expect("an empty table is not a filled one");
        assert!(msg.contains("zero rows"), "{msg}");
    }

    #[test]
    fn a_missing_column_refuses_and_names_the_column() {
        let body = plan("| # | Claim | Label | Anchor |\n|---|---|---|---|\n| 1 | c | read | a |");
        let msg = check(&body).expect("four columns are the contract");
        assert!(msg.contains("no \"Verbatim evidence\" column"), "{msg}");
    }

    #[test]
    fn the_claims_section_ends_at_the_next_heading() {
        // The table lives under a LATER section; the claims section is empty.
        let body = format!(
            "# Plan\n\n{CLAIMS_HEADING}\n\nnothing here\n\n## Shape\n\n{HEADER}\n| 1 | c | read | a | e |\n"
        );
        let msg = check(&body).expect("a table under a different heading is not this table");
        assert!(msg.contains("carries no markdown table"), "{msg}");
    }

    // ── row rules ─────────────────────────────────────────────────────────

    #[test]
    fn a_row_with_an_empty_cell_refuses_and_names_the_cell() {
        let body = plan(&format!("{HEADER}\n| 1 | c | read | | e |\n| 2 | | ran | cmd | out |"));
        let msg = check(&body).expect("an empty required cell is a hole in the evidence");
        assert!(msg.contains("row 1 has an empty \"Anchor\" cell"), "{msg}");
        assert!(msg.contains("row 2 has an empty \"Claim\" cell"), "{msg}");
    }

    #[test]
    fn an_empty_label_is_reported_once_not_twice() {
        let body = plan(&format!("{HEADER}\n| 1 | c | | a | e |"));
        let msg = check(&body).expect("an empty label refuses");
        assert!(msg.contains("row 1 has an empty \"Label\" cell"), "{msg}");
        assert!(!msg.contains("is not one of read"), "no vocabulary noise on top: {msg}");
    }

    #[test]
    fn a_label_outside_the_vocabulary_refuses_including_a_case_variant() {
        let body = plan(&format!("{HEADER}\n| 1 | c | Read | a | e |\n| 2 | c | skimmed | a | e |"));
        let msg = check(&body).expect("the vocabulary is exact and lowercase");
        assert!(msg.contains("row 1's label \"Read\" is not one of read / ran / guessed"), "{msg}");
        assert!(msg.contains("row 2's label \"skimmed\" is not one of read / ran / guessed"), "{msg}");
    }

    #[test]
    fn a_guessed_row_refuses_and_is_named_by_its_row_number() {
        let body = plan(&format!(
            "{HEADER}\n| 1 | c | ran | cmd | out |\n| 2 | c | ran | cmd | out |\n| 3 | c | guessed | a | e |"
        ));
        let msg = check(&body).expect("a guessed load-bearing claim never reaches a gate");
        assert!(msg.contains("row 3 is still labeled `guessed`"), "{msg}");
        assert!(!msg.contains("row 1 is still"), "the sound rows are not accused: {msg}");
    }

    #[test]
    fn a_read_row_whose_anchor_path_is_absent_refuses() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("history").join("demo");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("plan.md"),
            plan(&format!("{HEADER}\n| 1 | c | read | `src/nope.rs:12` | fn nope |")),
        )
        .unwrap();
        let msg = claims_refusal(tmp.path(), "demo").expect("an anchor into nothing refuses");
        assert!(msg.contains("row 1 is labeled `read` but its anchor path \"src/nope.rs\""), "{msg}");
    }

    #[test]
    fn a_read_row_whose_anchor_path_exists_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("history").join("demo");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src").join("real.rs"), "fn real() {}\n").unwrap();
        std::fs::write(
            dir.join("plan.md"),
            plan(&format!(
                "{HEADER}\n| 1 | c | read | `src/real.rs:1` | fn real |\n| 2 | c | read | `src/real.rs:1-2` | fn real |\n| 3 | c | read | `src/real.rs:1,2-3` | fn real |"
            )),
        )
        .unwrap();
        assert_eq!(claims_refusal(tmp.path(), "demo"), None);
    }

    #[test]
    fn a_non_path_anchor_never_fires_the_existence_rule() {
        // A `ran` command, a prose anchor, and a backticked path followed by
        // commentary: none of these is a `path:line` the rule may judge.
        let body = plan(&format!(
            "{HEADER}\n| 1 | c | ran | `fd -t f plan.md \\| wc -l` | 193 |\n| 2 | c | read | the session preamble | commands.test |\n| 3 | c | read | somewhere:else | bytes |"
        ));
        assert_eq!(check(&body), None);
    }

    #[test]
    fn a_complete_table_with_no_guessed_row_passes() {
        let body = plan(&format!(
            "{HEADER}\n| 1 | the verb refuses | ran | `cargo test -p bee` | test result: ok |\n| 2 | the field exists | read | not-a-path-anchor | `feature` |"
        ));
        assert_eq!(check(&body), None);
    }

    #[test]
    fn an_escaped_pipe_stays_inside_its_cell() {
        // Without escape handling this row parses as SIX cells and the
        // evidence column reads empty — a false refusal.
        let body = plan(&format!("{HEADER}\n| 1 | c | ran | `a \\| b` | out \\| put |"));
        assert_eq!(check(&body), None);
    }

    // ── the message ───────────────────────────────────────────────────────

    #[test]
    fn the_refusal_teaches_the_shape_and_both_remedies() {
        let body = plan(&format!("{HEADER}\n| 1 | c | guessed | a | e |"));
        let msg = check(&body).expect("a guessed row refuses");
        assert!(msg.contains("docs/history/demo/plan.md"), "names the file: {msg}");
        assert!(msg.contains("| # | Claim | Label | Anchor | Verbatim evidence |"), "columns: {msg}");
        assert!(msg.contains("read / ran / guessed"), "vocabulary: {msg}");
        assert!(msg.contains("## Open Questions"), "second remedy: {msg}");
        assert!(msg.contains("no waiver flag"), "no escape hatch: {msg}");
    }

    #[test]
    fn a_flood_of_broken_rows_is_capped_and_counted() {
        let rows: String = (1..=12)
            .map(|i| format!("\n| {i} | c | guessed | a | e |"))
            .collect();
        let body = plan(&format!("{HEADER}{rows}"));
        let msg = check(&body).expect("twelve guessed rows refuse");
        assert!(msg.contains("row 8 is still labeled"), "{msg}");
        assert!(!msg.contains("row 9 is still labeled"), "capped at eight: {msg}");
        assert!(msg.contains("(and 4 more)"), "the rest are counted: {msg}");
    }
}
