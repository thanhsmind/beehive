pub(crate) fn section_after_heading<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
    let mut offset = 0;
    let mut start = None;
    for line in text.split_inclusive('\n') {
        let next = offset + line.len();
        match start {
            None => {
                if line.trim_end_matches(['\n', '\r', ' ']) == heading {
                    start = Some(next);
                }
            }
            Some(from) => {
                if line.starts_with("## ") {
                    return Some(&text[from..offset]);
                }
            }
        }
        offset = next;
    }
    start.map(|from| &text[from..])
}

#[cfg(test)]
mod tests {
    use super::section_after_heading;

    #[test]
    fn a_paragraph_quoting_the_heading_does_not_move_the_section() {
        let text = "intro quotes ## Target inline\n\n## Target\nreal\n## Next\nafter\n";
        assert_eq!(section_after_heading(text, "## Target"), Some("real\n"));
    }

    #[test]
    fn a_longer_line_starting_with_the_heading_does_not_match() {
        let text = "## Target homes\nwrong\n## Target\nright\n";
        assert_eq!(section_after_heading(text, "## Target"), Some("right\n"));
    }

    #[test]
    fn a_crlf_heading_line_matches() {
        let text = "## Target\r\nbody\r\n## Next\r\n";
        assert_eq!(section_after_heading(text, "## Target"), Some("body\r\n"));
    }

    #[test]
    fn no_heading_line_gives_none() {
        let text = "## Other\nsee ## Target inline\n";
        assert_eq!(section_after_heading(text, "## Target"), None);
    }

    #[test]
    fn the_section_ends_at_the_next_level_two_heading() {
        let text = "## Target\na\n### sub\nb\n## Next\nc\n";
        assert_eq!(
            section_after_heading(text, "## Target"),
            Some("a\n### sub\nb\n")
        );
    }
}
