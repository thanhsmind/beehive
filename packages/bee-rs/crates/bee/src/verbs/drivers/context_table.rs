// doc-impact-synthesis D2: the canonical `## Locked Decisions` pipe-table
// parser CONTEXT.md's own grammar defines. Pure text parsing, no I/O — the
// caller (`build_routing_door`, close.rs) reads the file and hands the text
// in, exactly like the rest of this module's small parsers.
//
// Grammar (CONTEXT.md template, `.claude/skills/bee-shaping/references/
// context-template.md:26`): a `## Locked Decisions` heading, followed
// (anywhere before the next `## ` heading) by the first pipe table whose
// header row STARTS WITH `| ID | Decision` — prefix-tolerant, so the
// template's 3-column header and any feature's own extended header both
// parse. Rows shaped `| D<n> | ... |` name the locked decision ids, in
// table order. A section with no such table (bullet/split legacy CONTEXT
// forms — 63 of 112 files at the 2026-08-16 audit) has no canonical
// grammar to parse: `None`, never a panic, never a guess.

/// `None` = no canonical table found under `## Locked Decisions` (legacy
/// form, or no such heading at all) — the caller degrades to a report-only
/// notice, never a block, for this file (plan v2 kds-3, named deviation
/// from D2's letter bounded by D4's no-archaeology rationale).
/// `Some(ids)` = the table parsed; `ids` is every `D<n>` row, in table
/// order (possibly empty — a canonical table with zero rows is not itself
/// a grammar failure).
pub(crate) fn parse_locked_decision_ids(text: &str) -> Option<Vec<String>> {
    let lines: Vec<&str> = text.lines().collect();
    let heading_idx = lines.iter().position(|l| l.trim() == "## Locked Decisions")?;
    let mut i = heading_idx + 1;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("## ") {
            break; // next section reached — no matching table in this one
        }
        if trimmed.starts_with('|') && !is_separator_row(trimmed) {
            if table_header_matches(trimmed) {
                return Some(parse_table_rows(&lines, i));
            }
            // A pipe table exists here but its header does not match — skip
            // the whole block and keep looking for a later, matching table
            // before the next heading (still "the first following pipe
            // table whose header row STARTS WITH...").
            i = skip_table(&lines, i);
            continue;
        }
        i += 1;
    }
    None
}

fn table_header_matches(line: &str) -> bool {
    line.trim_start().starts_with("| ID | Decision")
}

fn is_separator_row(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('|') && t.contains('-') && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

fn skip_table(lines: &[&str], mut i: usize) -> usize {
    while i < lines.len() && lines[i].trim_start().starts_with('|') {
        i += 1;
    }
    i
}

/// Rows after the header row (`header_idx`): an optional separator row
/// (`|---|---|`) is skipped, then every contiguous `|`-led line is read as
/// a data row until the table ends. Only rows whose first cell is a plain
/// `D<digits>` id count — a row with any other first cell (a note, a
/// continuation) is silently skipped rather than guessed at.
fn parse_table_rows(lines: &[&str], header_idx: usize) -> Vec<String> {
    let mut ids = Vec::new();
    let mut j = header_idx + 1;
    if j < lines.len() && is_separator_row(lines[j]) {
        j += 1;
    }
    while j < lines.len() {
        let row = lines[j].trim();
        if !row.starts_with('|') {
            break;
        }
        if let Some(id) = row_d_id(row) {
            ids.push(id);
        }
        j += 1;
    }
    ids
}

/// `| D<n> | ... |` -> `Some("D<n>")`. The first cell after the leading
/// `|`, trimmed; a later `|` inside the decision/rationale prose never
/// confuses this since only the FIRST split segment is read.
fn row_d_id(row: &str) -> Option<String> {
    let rest = row.strip_prefix('|')?;
    let first_cell = rest.split('|').next()?.trim();
    if first_cell.len() > 1
        && first_cell.starts_with('D')
        && first_cell[1..].chars().all(|c| c.is_ascii_digit())
    {
        Some(first_cell.to_string())
    } else {
        None
    }
}

/// doc-impact-synthesis D2: does `text` (a docs/knowledge/ bundle file's
/// raw content — body and frontmatter alike, scanned as one string so
/// either surface counts) carry a citation of `feature D<n>` — plain
/// (`feature D2`), range (`feature D1-D3` covers every id in the closed
/// numeric interval), or slash-list (`feature D1/D3`, exactly the ids
/// named)? `d_num` is the bare numeric suffix of the D-id being checked
/// (`D2` -> `2`). Citation-detection regex left to planning's discretion
/// (CONTEXT.md "Agent's Discretion"); hand-scanned, matching this crate's
/// no-regex-dependency convention (`matches_deferral_prose` and friends).
pub(crate) fn context_table_covers_d_id(text: &str, feature: &str, d_num: u32) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let slug: Vec<char> = feature.chars().collect();
    if slug.is_empty() || chars.len() < slug.len() {
        return false;
    }
    let is_word_or_dash = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
    let mut i = 0usize;
    while i + slug.len() <= chars.len() {
        if chars[i..i + slug.len()] != slug[..] {
            i += 1;
            continue;
        }
        let before_ok = i == 0 || !is_word_or_dash(chars[i - 1]);
        if !before_ok {
            i += 1;
            continue;
        }
        let mut j = i + slug.len();
        let ws_start = j;
        while j < chars.len() && chars[j].is_whitespace() && chars[j] != '\n' {
            j += 1;
        }
        if j == ws_start || j >= chars.len() || chars[j] != 'D' {
            i += 1;
            continue;
        }
        if let Some((nums, seps, end)) = parse_d_sequence(&chars, j) {
            let end_ok = end >= chars.len() || !is_word_or_dash(chars[end]);
            if end_ok && d_sequence_covers(&nums, &seps, d_num) {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// `D<digits>` at `start`, then zero or more `[-/]D<digits>` continuations.
/// Returns the numbers in order, the separators between them (same length
/// as `nums.len() - 1`), and the index just past the last digit.
fn parse_d_sequence(chars: &[char], start: usize) -> Option<(Vec<u32>, Vec<char>, usize)> {
    let (first, mut end) = parse_one_d(chars, start)?;
    let mut nums = vec![first];
    let mut seps = Vec::new();
    loop {
        if end < chars.len() && matches!(chars[end], '-' | '/') && end + 1 < chars.len() && chars[end + 1] == 'D' {
            let sep = chars[end];
            if let Some((n, e)) = parse_one_d(chars, end + 1) {
                seps.push(sep);
                nums.push(n);
                end = e;
                continue;
            }
        }
        break;
    }
    Some((nums, seps, end))
}

/// `D<digits>` at `start` (start is the index of `D`) -> the number and the
/// index just past its last digit.
fn parse_one_d(chars: &[char], start: usize) -> Option<(u32, usize)> {
    if chars.get(start) != Some(&'D') {
        return None;
    }
    let mut end = start + 1;
    while end < chars.len() && chars[end].is_ascii_digit() {
        end += 1;
    }
    if end == start + 1 {
        return None; // "D" with no digits
    }
    let n: u32 = chars[start + 1..end].iter().collect::<String>().parse().ok()?;
    Some((n, end))
}

fn d_sequence_covers(nums: &[u32], seps: &[char], d_num: u32) -> bool {
    if nums.len() == 1 {
        return nums[0] == d_num;
    }
    if seps.iter().any(|&s| s == '-') {
        let lo = *nums.iter().min().unwrap();
        let hi = *nums.iter().max().unwrap();
        return d_num >= lo && d_num <= hi;
    }
    nums.contains(&d_num)
}
