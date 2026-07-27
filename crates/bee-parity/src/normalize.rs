//! Declared volatile allowlist (CONTEXT.md D7a, validation decision W7 +
//! advisor note 3): the ONLY normalization the parity differ is allowed to
//! apply before comparing two legs' output. Everything else is compared
//! byte-for-byte (well, char-for-char after UTF-8 decode) — this is
//! deliberately NOT a blanket timestamp/whitespace normalizer.
//!
//! Allowlist members:
//! - each leg's own temp-root absolute path, rewritten to the literal
//!   `<ROOT>` (root-path rewriting is part of the declared allowlist, not
//!   blanket normalization — CONTEXT.md D7a diff policy);
//! - ISO-8601 timestamp values (shape-matched, not a generic number
//!   scrubber);
//! - JSON object values under a key literally named `pid`;
//! - JSON object values under a key whose name contains `token`
//!   (case-insensitive) — covers `token`, `authToken`, `claim_token`, etc.

const TS_PLACEHOLDER: &str = "<TS>";
const PID_PLACEHOLDER: &str = "<PID>";
const TOKEN_PLACEHOLDER: &str = "<TOKEN>";
pub const ROOT_PLACEHOLDER: &str = "<ROOT>";

/// Apply the full declared allowlist to `text`, in a fixed order:
/// root-path rewrite first (most specific, avoids the other passes
/// re-matching inside a path fragment), then timestamps, then keyed
/// pid/token values.
pub fn normalize(text: &str, own_root: &str) -> String {
    let rewritten = rewrite_root(text, own_root);
    let rewritten = normalize_timestamps(&rewritten);
    normalize_keyed_values(&rewritten, &["pid"], PID_PLACEHOLDER, KeyMatch::Exact)
        .pipe(|s| normalize_keyed_values(&s, &["token"], TOKEN_PLACEHOLDER, KeyMatch::Contains))
}

// Small local extension so the pipeline above reads left-to-right instead of
// nesting calls inside-out. std-only: no external "pipe"/itertools crate.
trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl Pipe for String {}

/// Replace every occurrence of `own_root` (a leg's own absolute temp-root
/// path) with the literal `<ROOT>`. Matches the root path itself and any
/// longer path that starts with it (e.g. `own_root` plus `/.bee/...`), by
/// virtue of plain substring replacement.
fn rewrite_root(text: &str, own_root: &str) -> String {
    if own_root.is_empty() {
        return text.to_string();
    }
    text.replace(own_root, ROOT_PLACEHOLDER)
}

/// Scan for ISO-8601-shaped timestamps (`YYYY-MM-DDTHH:MM:SS[.fff]Z`) and
/// replace each with `<TS>`. Hand-rolled (no `regex` crate — std only,
/// rust-port-1/2 precedent): a small forward scanner over ASCII digits and
/// literal separators, deliberately narrow so it cannot eat unrelated
/// digit runs (dates need the literal `-`/`T`/`:` skeleton present).
fn normalize_timestamps(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if let Some(len) = match_timestamp(&bytes[i..]) {
            out.push_str(TS_PLACEHOLDER);
            i += len;
        } else {
            // Safe because we only advance by whole UTF-8 char boundaries
            // when not consuming a (pure-ASCII) timestamp match above.
            let ch_len = next_char_len(text, i);
            out.push_str(&text[i..i + ch_len]);
            i += ch_len;
        }
    }
    out
}

fn next_char_len(text: &str, i: usize) -> usize {
    text[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1)
}

fn is_digit(b: u8) -> bool {
    b.is_ascii_digit()
}

fn take_digits(bytes: &[u8], start: usize, count: usize) -> Option<usize> {
    if start + count > bytes.len() {
        return None;
    }
    if bytes[start..start + count].iter().all(|&b| is_digit(b)) {
        Some(count)
    } else {
        None
    }
}

/// Returns the byte length of a matched timestamp at `bytes[0..]`, or None.
/// Shape: `\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,9})?Z?`
fn match_timestamp(bytes: &[u8]) -> Option<usize> {
    let mut pos = 0usize;
    pos += take_digits(bytes, pos, 4)?;
    if bytes.get(pos) != Some(&b'-') {
        return None;
    }
    pos += 1;
    pos += take_digits(bytes, pos, 2)?;
    if bytes.get(pos) != Some(&b'-') {
        return None;
    }
    pos += 1;
    pos += take_digits(bytes, pos, 2)?;
    if bytes.get(pos) != Some(&b'T') {
        return None;
    }
    pos += 1;
    pos += take_digits(bytes, pos, 2)?;
    if bytes.get(pos) != Some(&b':') {
        return None;
    }
    pos += 1;
    pos += take_digits(bytes, pos, 2)?;
    if bytes.get(pos) != Some(&b':') {
        return None;
    }
    pos += 1;
    pos += take_digits(bytes, pos, 2)?;
    // optional fractional seconds
    if bytes.get(pos) == Some(&b'.') {
        let mut frac_len = 1usize;
        let mut j = pos + 1;
        while j < bytes.len() && is_digit(bytes[j]) && frac_len <= 9 {
            j += 1;
            frac_len += 1;
        }
        if j > pos + 1 {
            pos = j;
        }
    }
    // optional trailing Z
    if bytes.get(pos) == Some(&b'Z') {
        pos += 1;
    }
    Some(pos)
}

#[derive(Clone, Copy)]
enum KeyMatch {
    Exact,
    Contains,
}

/// Scan JSON-shaped text for `"<key>":<value>` pairs whose key matches one
/// of `keys` (per `mode`, case-insensitive), and replace the value with
/// `"<placeholder>"`. The value may be a quoted string or a bare
/// number/null/true/false — both are replaced uniformly with a quoted
/// placeholder string, since only the fact that a diff-irrelevant volatile
/// value sat there matters, not its original JSON type.
fn normalize_keyed_values(text: &str, keys: &[&str], placeholder: &str, mode: KeyMatch) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            if let Some((key, key_end)) = read_json_key(text, i) {
                let matches = keys.iter().any(|k| match mode {
                    KeyMatch::Exact => key.eq_ignore_ascii_case(k),
                    KeyMatch::Contains => key.to_ascii_lowercase().contains(&k.to_ascii_lowercase()),
                });
                // Look past the key for `:` then a value.
                let mut j = key_end;
                while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                    j += 1;
                }
                if matches && bytes.get(j) == Some(&b':') {
                    j += 1;
                    while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                        j += 1;
                    }
                    if let Some(value_end) = skip_json_value(text, j) {
                        out.push_str(&text[i..key_end]);
                        out.push(':');
                        out.push('"');
                        out.push_str(placeholder);
                        out.push('"');
                        i = value_end;
                        continue;
                    }
                }
            }
        }
        let ch_len = next_char_len(text, i);
        out.push_str(&text[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Given `text[start] == '"'`, read a JSON string key. Returns
/// `(key_content, byte_index_just_after_closing_quote)`.
fn read_json_key(text: &str, start: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    if bytes.get(start) != Some(&b'"') {
        return None;
    }
    let mut i = start + 1;
    let mut key = String::new();
    while i < bytes.len() {
        match bytes[i] {
            b'"' => return Some((key, i + 1)),
            b'\\' if i + 1 < bytes.len() => {
                key.push(bytes[i] as char);
                key.push(bytes[i + 1] as char);
                i += 2;
            }
            b => {
                let ch_len = next_char_len(text, i);
                key.push_str(&text[i..i + ch_len]);
                i += ch_len;
                let _ = b;
            }
        }
    }
    None
}

/// Skip a single JSON value (string, number, true/false/null) starting at
/// `text[start]`. Deliberately does NOT descend into objects/arrays —
/// volatile pid/token values in this codebase's stores are always scalars,
/// and refusing to touch a nested structure is the safer failure mode (no
/// match found -> value left untouched -> a real diff there still shows).
fn skip_json_value(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    match bytes.get(start)? {
        b'"' => {
            let mut i = start + 1;
            while i < bytes.len() {
                match bytes[i] {
                    b'"' => return Some(i + 1),
                    b'\\' if i + 1 < bytes.len() => i += 2,
                    _ => i += next_char_len(text, i),
                }
            }
            None
        }
        b't' if text[start..].starts_with("true") => Some(start + 4),
        b'f' if text[start..].starts_with("false") => Some(start + 5),
        b'n' if text[start..].starts_with("null") => Some(start + 4),
        b'-' | b'0'..=b'9' => {
            let mut i = start;
            if bytes[i] == b'-' {
                i += 1;
            }
            while i < bytes.len() && (is_digit(bytes[i]) || bytes[i] == b'.' || bytes[i] == b'e' || bytes[i] == b'E' || bytes[i] == b'+' || bytes[i] == b'-') {
                i += 1;
            }
            Some(i)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_own_root_only() {
        let out = normalize("path is /tmp/leg-a/.bee/state.json", "/tmp/leg-a");
        assert_eq!(out, "path is <ROOT>/.bee/state.json");
    }

    #[test]
    fn does_not_rewrite_other_legs_root() {
        // Only THIS leg's own root is a declared allowlist member; the
        // other leg's differing path is a real diff signal, not noise.
        let out = normalize("path is /tmp/leg-b/.bee/state.json", "/tmp/leg-a");
        assert_eq!(out, "path is /tmp/leg-b/.bee/state.json");
    }

    #[test]
    fn normalizes_iso8601_timestamp() {
        let out = normalize("\"ts\":\"2026-07-26T05:23:13.467Z\"", "");
        assert_eq!(out, "\"ts\":\"<TS>\"");
    }

    #[test]
    fn does_not_touch_plain_dates_without_time_skeleton() {
        let out = normalize("\"note\":\"2026-07-26 is a date-shaped string\"", "");
        assert!(out.contains("2026-07-26 is a date-shaped string"));
    }

    #[test]
    fn normalizes_pid_key_exact_match_only() {
        let out = normalize("{\"pid\":12345,\"pidgeon\":12345}", "");
        assert!(out.contains("\"pid\":\"<PID>\""));
        // "pidgeon" is not the exact key "pid" -> left untouched.
        assert!(out.contains("\"pidgeon\":12345"));
    }

    #[test]
    fn normalizes_any_key_containing_token() {
        let out = normalize("{\"token\":\"abc\",\"claim_token\":\"xyz\"}", "");
        assert!(out.contains("\"token\":\"<TOKEN>\""));
        assert!(out.contains("\"claim_token\":\"<TOKEN>\""));
    }

    #[test]
    fn leaves_unrelated_numbers_untouched() {
        let out = normalize("{\"cells_count\":250,\"decisions_bytes\":700000}", "");
        assert_eq!(out, "{\"cells_count\":250,\"decisions_bytes\":700000}");
    }
}
