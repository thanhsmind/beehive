// onboard::merge — the two marker-splice merges (AGENTS.md and .gitignore).
//
// Provenance: onboard_bee.mjs renderAgentsBlock (l. 2077),
// renderGitignoreBlock (l. 2082), agentsBlockPresent (l. 2088),
// extractAgentsBlock (l. 2092), mergeAgentsContent (l. 2101),
// gitignoreBlockPresent (l. 2137), findGitignoreMarkers (l. 2141),
// extractGitignoreBlock (l. 2150), normalizeGitignoreForCompare (l. 2162)
// and mergeGitignoreContent (l. 2166).
//
// The gitignore marker match is WHOLE-LINE anchored (review P2/P3): a user
// comment like `# BEE:START custom notes` must never be adopted as the
// managed block, and everything outside the two marker lines is copied
// through byte-for-byte, never re-normalized.

use super::templates::{
    GITIGNORE_BLOCK_PATTERNS, GITIGNORE_MARKER_END, GITIGNORE_MARKER_START, MARKER_END,
    MARKER_START,
};
use super::util::{read_text_if_exists, trim_trailing_ws};
use std::path::Path;

/// renderAgentsBlock (l. 2077).
pub fn render_agents_block(agents_block_template: &Path) -> String {
    let body = read_text_if_exists(agents_block_template);
    format!("{MARKER_START}\n{}\n{MARKER_END}\n", trim_trailing_ws(&body))
}

/// renderGitignoreBlock (l. 2082).
pub fn render_gitignore_block() -> String {
    format!(
        "{GITIGNORE_MARKER_START}\n{}\n{GITIGNORE_MARKER_END}\n",
        GITIGNORE_BLOCK_PATTERNS.join("\n")
    )
}

// ── AGENTS.md ──────────────────────────────────────────────────────────────

pub fn agents_block_present(text: &str) -> bool {
    text.contains(MARKER_START) && text.contains(MARKER_END)
}

/// extractAgentsBlock (l. 2092).
pub fn extract_agents_block(text: &str) -> Option<String> {
    let start = text.find(MARKER_START)?;
    let end = text.find(MARKER_END)?;
    if end < start {
        return None;
    }
    Some(format!("{}\n", &text[start..end + MARKER_END.len()]))
}

pub struct MergeResult {
    pub text: String,
    /// "created" | "appended" | "updated" — the .mjs returns it and the
    /// suite pins it, but the apply path only ever writes `text` (the plan
    /// item already names which of the three happened).
    #[allow(dead_code)]
    pub status: &'static str,
}

/// mergeAgentsContent (l. 2101).
pub fn merge_agents_content(existing: &str, rendered_block: &str) -> MergeResult {
    if existing.trim().is_empty() {
        return MergeResult { text: rendered_block.to_string(), status: "created" };
    }
    if agents_block_present(existing) {
        let start = existing.find(MARKER_START).unwrap();
        let mut end = existing.find(MARKER_END).unwrap() + MARKER_END.len();
        if existing.as_bytes().get(end) == Some(&b'\n') {
            end += 1;
        }
        let updated = format!("{}{rendered_block}{}", &existing[..start], &existing[end..]);
        return MergeResult {
            text: format!("{}\n", trim_trailing_ws(&updated)),
            status: "updated",
        };
    }
    MergeResult {
        text: format!("{}\n\n{rendered_block}", trim_trailing_ws(existing)),
        status: "appended",
    }
}

// ── .gitignore ─────────────────────────────────────────────────────────────

/// One whole-line-anchored marker match: `^# BEE:START[ \t]*\r?$`. Returns
/// (byte offset of the line start, byte length of the match — the line
/// content INCLUDING a CRLF's `\r`, excluding the `\n`).
fn find_marker_line(text: &str, marker: &str) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut line_start = 0usize;
    loop {
        let line_end = bytes[line_start..]
            .iter()
            .position(|b| *b == b'\n')
            .map(|i| line_start + i)
            .unwrap_or(bytes.len());
        let line = &text[line_start..line_end];
        if let Some(tail) = line.strip_prefix(marker) {
            let tail = tail.strip_suffix('\r').unwrap_or(tail);
            if tail.chars().all(|c| c == ' ' || c == '\t') {
                return Some((line_start, line_end - line_start));
            }
        }
        if line_end >= bytes.len() {
            return None;
        }
        line_start = line_end + 1;
    }
}

pub fn gitignore_block_present(text: &str) -> bool {
    find_marker_line(text, GITIGNORE_MARKER_START).is_some()
        && find_marker_line(text, GITIGNORE_MARKER_END).is_some()
}

/// findGitignoreMarkers (l. 2141): (start offset, end offset-after-marker).
fn find_gitignore_markers(text: &str) -> Option<(usize, usize)> {
    let (start_idx, _) = find_marker_line(text, GITIGNORE_MARKER_START)?;
    let (end_idx, end_len) = find_marker_line(text, GITIGNORE_MARKER_END)?;
    if end_idx < start_idx {
        return None;
    }
    Some((start_idx, end_idx + end_len))
}

/// extractGitignoreBlock (l. 2150).
pub fn extract_gitignore_block(text: &str) -> Option<String> {
    let (start, end) = find_gitignore_markers(text)?;
    Some(format!("{}\n", &text[start..end]))
}

/// normalizeGitignoreForCompare (l. 2162): CRLF→LF for the equality check
/// ONLY — writes always stay LF.
pub fn normalize_gitignore_for_compare(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// mergeGitignoreContent (l. 2166).
pub fn merge_gitignore_content(existing: &str, rendered_block: &str) -> MergeResult {
    if existing.trim().is_empty() {
        return MergeResult { text: rendered_block.to_string(), status: "created" };
    }
    if let Some((start, markers_end)) = find_gitignore_markers(existing) {
        let mut end = markers_end;
        if existing.as_bytes().get(end) == Some(&b'\n') {
            end += 1;
        }
        return MergeResult {
            text: format!("{}{rendered_block}{}", &existing[..start], &existing[end..]),
            status: "updated",
        };
    }
    MergeResult {
        text: format!("{}\n\n{rendered_block}", trim_trailing_ws(existing)),
        status: "appended",
    }
}

/// The CLAUDE.md `@AGENTS.md` import probe: `/^@AGENTS\.md\s*$/m`. With the
/// `m` flag `$` matches before any `\n`, so the test reduces to "a line that
/// starts with @AGENTS.md and whose remainder is whitespace".
pub fn claude_md_imports_agents(text: &str) -> bool {
    super::util::split_lines(text)
        .into_iter()
        .any(|line| line.strip_prefix("@AGENTS.md").is_some_and(|t| t.trim().is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK: &str = "<!-- BEE:START -->\nbody\n<!-- BEE:END -->\n";

    #[test]
    fn agents_merge_creates_appends_and_updates() {
        let created = merge_agents_content("", BLOCK);
        assert_eq!(created.status, "created");
        assert_eq!(created.text, BLOCK);

        let appended = merge_agents_content("# My project\n\nprose", BLOCK);
        assert_eq!(appended.status, "appended");
        assert_eq!(appended.text, format!("# My project\n\nprose\n\n{BLOCK}"));

        let existing = format!("# head\n\n<!-- BEE:START -->\nOLD\n<!-- BEE:END -->\n\nfooter\n");
        let updated = merge_agents_content(&existing, BLOCK);
        assert_eq!(updated.status, "updated");
        assert_eq!(updated.text, format!("# head\n\n{BLOCK}\nfooter\n"));
    }

    #[test]
    fn agents_append_inserts_a_blank_line_even_without_a_trailing_newline() {
        let m = merge_agents_content("no newline", BLOCK);
        assert!(m.text.starts_with("no newline\n\n<!-- BEE:START -->"));
    }

    #[test]
    fn extract_round_trips_the_rendered_block() {
        let text = format!("head\n{BLOCK}tail\n");
        assert_eq!(extract_agents_block(&text).as_deref(), Some(BLOCK));
    }

    #[test]
    fn gitignore_markers_are_whole_line_anchored() {
        let decoy = "# BEE:START custom notes\nfoo\n# BEE:END extra\n";
        assert!(!gitignore_block_present(decoy));
        let real = "# BEE:START\nfoo\n# BEE:END\n";
        assert!(gitignore_block_present(real));
        assert!(gitignore_block_present("# BEE:START \t\r\nfoo\n# BEE:END\r\n"));
    }

    #[test]
    fn gitignore_merge_preserves_content_outside_the_markers() {
        let block = render_gitignore_block();
        let existing = "node_modules/\n\n# BEE:START\nold\n# BEE:END\n\n# my own footer\n";
        let m = merge_gitignore_content(existing, &block);
        assert_eq!(m.status, "updated");
        assert_eq!(m.text, format!("node_modules/\n\n{block}\n# my own footer\n"));
        // Idempotent.
        assert_eq!(merge_gitignore_content(&m.text, &block).text, m.text);
    }

    #[test]
    fn gitignore_append_never_merges_two_patterns_onto_one_line() {
        let block = render_gitignore_block();
        let m = merge_gitignore_content("dist", &block);
        assert_eq!(m.status, "appended");
        assert_eq!(m.text, format!("dist\n\n{block}"));
    }

    #[test]
    fn crlf_only_affects_the_comparison() {
        let block = render_gitignore_block();
        let crlf = block.replace('\n', "\r\n");
        assert_eq!(normalize_gitignore_for_compare(&crlf), block);
    }

    #[test]
    fn claude_md_import_probe() {
        assert!(claude_md_imports_agents("## bee\n\n@AGENTS.md\n"));
        assert!(claude_md_imports_agents("@AGENTS.md  \r\n"));
        assert!(claude_md_imports_agents("@AGENTS.md"));
        assert!(!claude_md_imports_agents("`@AGENTS.md`\n"));
        assert!(!claude_md_imports_agents("@AGENTS.md and more\n"));
        assert!(!claude_md_imports_agents("# Project Rules\n"));
    }
}
