// jspath — Node's `path` module, win32 flavor on Windows and posix
// elsewhere, for the operations the dev-surface scripts perform on paths.
//
// provenance: node lib/path.js (`win32.normalize/join/dirname/basename`
// and their posix twins).
//
// WHY NOT std::path. `statusline-usage.mjs` slices and rejoins paths with
// Node's LEXICAL rules — trailing-separator preservation, `..` popping and the
// win32 drive prefix — and its output is byte-compared against the original's.
// std::path differs on several of those, so Node's `path` is reproduced rather
// than approximated. The operations are purely lexical here, exactly as they
// are in Node.
//
// SCOPE. Trimmed at the R6 cutover to the operations `statusline.rs` still
// calls (SEP / normalize / join / dirname / basename). `resolve`, `relative`,
// `is_absolute` and `extname` existed only for the retired impact registry —
// which had to rebuild a committed JSON graph keyed on paths it assembled out
// of scanned source fragments — and went with it.

pub const SEP: char = if cfg!(windows) { '\\' } else { '/' };

fn is_sep(c: char) -> bool {
    c == '/' || (cfg!(windows) && c == '\\')
}

/// Splits a path into (root prefix, remainder, is-absolute). The prefix is a
/// SLICE of the input, so the caller's separator spelling survives — Node's
/// `dirname`/`basename` slice the original string and never normalize, and
/// `path.win32.dirname("D:/a/b")` really is `"D:/a"`.
fn split_root(p: &str) -> (&str, &str, bool) {
    if cfg!(windows) {
        let b = p.as_bytes();
        if b.len() >= 2 && b[1] == b':' && (b[0] as char).is_ascii_alphabetic() {
            let rest = &p[2..];
            if rest.starts_with(is_sep) {
                return (&p[..3], &p[3..], true);
            }
            return (&p[..2], rest, false);
        }
    }
    if p.starts_with(is_sep) {
        return (&p[..1], &p[1..], true);
    }
    ("", p, false)
}

/// provenance: `path.normalize` — separators unified, `.` dropped, `..`
/// popped (kept when it cannot be popped in a relative path), and a trailing
/// separator PRESERVED.
pub fn normalize(p: &str) -> String {
    if p.is_empty() {
        return ".".to_string();
    }
    let unified: String = p.chars().map(|c| if is_sep(c) { SEP } else { c }).collect();
    let trailing = unified.len() > 1 && unified.ends_with(SEP);
    let (prefix, rest, absolute) = split_root(&unified);
    let mut out = prefix.to_string();
    let mut parts: Vec<&str> = Vec::new();
    for seg in rest.split(SEP) {
        match seg {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|l| *l != "..") {
                    parts.pop();
                } else if !absolute {
                    parts.push("..");
                }
            }
            other => parts.push(other),
        }
    }
    let body = parts.join(&SEP.to_string());
    out.push_str(&body);
    if out.is_empty() {
        return if absolute { SEP.to_string() } else { ".".to_string() };
    }
    if trailing && !out.ends_with(SEP) {
        out.push(SEP);
    }
    out
}

/// provenance: `path.join` — non-empty args concatenated with the platform
/// separator, then normalized; all-empty is ".".
pub fn join(parts: &[&str]) -> String {
    let joined: Vec<&str> = parts.iter().copied().filter(|p| !p.is_empty()).collect();
    if joined.is_empty() {
        return ".".to_string();
    }
    normalize(&joined.join(&SEP.to_string()))
}

/// provenance: `path.dirname`.
pub fn dirname(p: &str) -> String {
    if p.is_empty() {
        return ".".to_string();
    }
    let (prefix, rest, absolute) = split_root(p);
    let rest = rest.trim_end_matches(is_sep);
    match rest.rfind(is_sep) {
        Some(i) => {
            let head = rest[..i].trim_end_matches(is_sep);
            if head.is_empty() {
                if absolute { prefix.to_string() } else { ".".to_string() }
            } else {
                format!("{prefix}{head}")
            }
        }
        None => {
            if absolute || !prefix.is_empty() {
                prefix.to_string()
            } else {
                ".".to_string()
            }
        }
    }
}

/// provenance: `path.basename(p[, ext])` — the suffix is dropped only when
/// the basename ends with it AND is not equal to it.
pub fn basename(p: &str, ext: &str) -> String {
    let (_, rest, _) = split_root(p);
    let rest = rest.trim_end_matches(is_sep);
    let base = match rest.rfind(is_sep) {
        Some(i) => &rest[i + 1..],
        None => rest,
    };
    if !ext.is_empty() && base.len() > ext.len() && base.ends_with(ext) {
        return base[..base.len() - ext.len()].to_string();
    }
    base.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_matches_node() {
        assert_eq!(normalize("a/b/../c"), format!("a{SEP}c"));
        assert_eq!(normalize("./a/./b"), format!("a{SEP}b"));
        assert_eq!(normalize("a/b/"), format!("a{SEP}b{SEP}"));
        assert_eq!(normalize("../../x"), format!("..{SEP}..{SEP}x"));
        assert_eq!(normalize(""), ".");
    }

    #[test]
    fn join_matches_node() {
        assert_eq!(join(&["a", "b", "c.mjs"]), format!("a{SEP}b{SEP}c.mjs"));
        assert_eq!(join(&["a", "", "b"]), format!("a{SEP}b"));
        assert_eq!(join(&["a/b", ".."]), "a");
        assert_eq!(join(&[]), ".");
        assert_eq!(join(&["", ""]), ".");
    }

    #[test]
    fn dirname_and_basename_match_node() {
        // Node slices the ORIGINAL string here — separators are not unified.
        assert_eq!(dirname("a/b/c.md"), "a/b");
        assert_eq!(dirname(&format!("a{SEP}b{SEP}c.md")), format!("a{SEP}b"));
        assert_eq!(dirname("c.md"), ".");
        assert_eq!(basename("a/b/c.md", ""), "c.md");
        assert_eq!(basename("a/b/c.md", ".md"), "c");
        assert_eq!(basename("a/.md", ".md"), ".md"); // base == ext
    }
}
