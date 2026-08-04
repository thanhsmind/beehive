// Text length/truncation/order primitives — consolidated (decision D3,
// docs/history/js-parity-cleanup/CONTEXT.md).
//
// Before this module, "how long is this string" and "give me the front/back
// of this string" were each reimplemented independently across the crate —
// seven copies, several byte-identical — all counting UTF-16 code units to
// match a V8 runtime that no longer exists on the other side of any of these
// calls. Every caller here is a DISPLAY or LOG truncation cap (a failure
// excerpt, a decision-line preview, a backlog field limit, an audit-log
// description), never something that round-trips through a JS engine, so D3
// retires the UTF-16 counting for those and standardizes on Rust-native
// `char` counting instead: astral-plane input (emoji and the like) now counts
// as ONE unit instead of two, so a cap's cut point can land one char earlier
// for such input. Every ASCII/BMP caller — the overwhelming majority of real
// traffic — is byte-for-byte unchanged.
//
// EXCEPTION — `code_unit_cmp` / `js_default_sort` below still order strings
// by UTF-16 code unit. That is not a truncation cap; it is the exact
// ordering already baked into release manifests stamped before this
// decision existed. Re-sorting those same paths in char/byte order (which,
// for the ASCII repo-relative paths these tools see, usually — but not
// always — coincides with code-unit order) produces a DIFFERENT sequence
// than the committed file; `release_manifest.rs`'s
// `code_unit_sort_would_not_reproduce_the_manifest` test proves the
// divergence is real, not theoretical. Converting this comparator to a
// char-based order is not authorized: it would silently desynchronize the
// binary from history it has already written.

use std::cmp::Ordering;

/// Character count — the crate's one "how long is this string" primitive for
/// display/log caps (replaces the historical `utf16_len` — three identical
/// copies).
pub(crate) fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// The first `n` CHARACTERS of `s` (the whole string when it is already
/// `n` chars or shorter) — the crate's one "front of this string" truncation
/// primitive (replaces the historical `utf16_head` / `js_slice_head` /
/// `js_slice_utf16` / `utf16_slice`).
pub(crate) fn truncate_chars_head(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// The last `n` CHARACTERS of `s` (the whole string when it is already `n`
/// chars or shorter) — the crate's one "tail of this string" truncation
/// primitive (replaces the historical `utf16_tail` / `js_slice_tail_utf16`).
pub(crate) fn truncate_chars_tail(s: &str, n: usize) -> String {
    let len = char_len(s);
    if len <= n {
        return s.to_string();
    }
    s.chars().skip(len - n).collect()
}

// ─── the code-unit-order EXCEPTION (release-manifest reproduction) ────────

/// String ordering by UTF-16 code unit. Kept deliberately distinct from this
/// module's char-based primitives above — see the module-level EXCEPTION
/// note: it is the ordering already baked into release manifests stamped
/// before this decision, and no other order reproduces them.
pub(crate) fn code_unit_cmp(a: &str, b: &str) -> Ordering {
    let mut x = a.encode_utf16();
    let mut y = b.encode_utf16();
    loop {
        match (x.next(), y.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(p), Some(q)) if p == q => continue,
            (Some(p), Some(q)) => return p.cmp(&q),
        }
    }
}

/// Sort `items` by [`code_unit_cmp`] — the ordering `devtools`'s skill-tree
/// and release-manifest renderers must reproduce to keep committed output
/// byte-for-byte unchanged.
pub(crate) fn js_default_sort(items: &mut [String]) {
    items.sort_by(|a, b| code_unit_cmp(a, b));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_len_counts_chars_not_code_units() {
        assert_eq!(char_len("abc"), 3);
        // An astral char is one CHAR though it is two UTF-16 code units —
        // the whole point of the D3 cutover.
        assert_eq!(char_len("🐝"), 1);
        assert_eq!(char_len("ab🐝"), 3);
    }

    #[test]
    fn truncate_chars_head_takes_the_front() {
        assert_eq!(truncate_chars_head("abcdef", 3), "abc");
        assert_eq!(truncate_chars_head("abc", 10), "abc");
        assert_eq!(truncate_chars_head("🐝🐝🐝", 2), "🐝🐝");
    }

    #[test]
    fn truncate_chars_tail_takes_the_back() {
        assert_eq!(truncate_chars_tail(&"x".repeat(650), 500).chars().count(), 500);
        assert_eq!(truncate_chars_tail("short", 500), "short");
        assert_eq!(truncate_chars_tail("abcdef", 3), "def");
    }

    #[test]
    fn code_unit_cmp_orders_by_utf16_unit() {
        assert_eq!(code_unit_cmp("a", "b"), Ordering::Less);
        assert_eq!(code_unit_cmp("b", "a"), Ordering::Greater);
        assert_eq!(code_unit_cmp("a", "a"), Ordering::Equal);
    }

    #[test]
    fn js_default_sort_is_stable_code_unit_order() {
        let mut v = vec!["b".to_string(), "_".to_string(), "a".to_string()];
        js_default_sort(&mut v);
        assert_eq!(v, ["_", "a", "b"]); // '_' (0x5F) < 'a' (0x61) < 'b' (0x62)
    }
}
