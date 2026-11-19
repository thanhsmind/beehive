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
