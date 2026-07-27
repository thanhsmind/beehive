//! feedback — the shared KIND vocabulary, ported from
//! `packages/bee/lib/feedback.mjs:67` (`KIND_ALIASES`) and `:98`
//! (`NORMALIZED_KINDS`), frozen under D1.
//!
//! Landed by `rpl-3` rather than by the feedback cell (`rpl-8`) on purpose,
//! and this is the whole reason the module exists this early:
//! `bee.mjs:3694` `backlogAllowedTypes()` is
//! `[...new Set([...Object.keys(KIND_ALIASES), ...NORMALIZED_KINDS])].sort()`
//! and IS the `backlog add --type` refusal text — a byte-parity surface the
//! backlog cell (`rpl-6`) has to reproduce, and `rpl-6` runs BEFORE the
//! feedback port. Without this module `rpl-6` would hardcode a second copy
//! of the list, which is exactly the duplicated-enumeration break D7
//! forbids. There is ONE enumeration authority, and it is here.
//!
//! `KIND_ALIASES` is a JS object literal, so `Object.keys` yields INSERTION
//! order and `new Set(Object.values(...))` yields first-occurrence order.
//! Both are preserved as ordered slices rather than maps: an alphabetical
//! re-emission would be a different `--help`/refusal surface even though the
//! same strings are present.

/// `feedback.mjs:67` `KIND_ALIASES`, in object-literal insertion order:
/// `(raw type, normalized kind)`.
pub const KIND_ALIASES: &[(&str, &str)] = &[
    ("friction", "friction"),
    ("finding", "finding"),
    ("review-finding", "finding"),
    ("proposal", "proposal"),
    ("kill-proposal", "proposal"),
    ("outcome", "outcome"),
    ("kill-outcome", "outcome"),
    ("kill-approval", "approval"),
    ("backlog-closed", "closed"),
    ("entropy-audit", "audit"),
    ("harness-issue", "harness-issue"),
    ("debt", "debt"),
    ("migrate-on-touch", "debt"),
    ("scope-correction", "correction"),
    // derived kinds (built directly from cells / learnings) normalize to
    // themselves
    ("blocked", "blocked"),
    ("deviation", "deviation"),
    ("learning", "learning"),
];

/// `feedback.mjs:98` `NORMALIZED_KINDS` — `new Set(Object.values(KIND_ALIASES))`,
/// so: the alias VALUES, deduplicated, in first-occurrence order.
pub fn normalized_kinds() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for (_, kind) in KIND_ALIASES {
        if !out.contains(kind) {
            out.push(kind);
        }
    }
    out
}

/// `normalizeKind`'s lookup: a raw type maps to its normalized kind, and a
/// type absent from the map is NOT silently dropped by callers — it is
/// `unknown_type`.
pub fn normalize_kind(raw: &str) -> Option<&'static str> {
    KIND_ALIASES.iter().find(|(alias, _)| *alias == raw).map(|(_, kind)| *kind)
}

/// `bee.mjs:3694` `backlogAllowedTypes()`. `Array.prototype.sort()` with no
/// comparator sorts by UTF-16 code unit; every member here is ASCII, so
/// Rust's byte ordering on `&str` is the same ordering.
pub fn backlog_allowed_types() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for (alias, _) in KIND_ALIASES {
        if !out.contains(alias) {
            out.push(alias);
        }
    }
    for kind in normalized_kinds() {
        if !out.contains(&kind) {
            out.push(kind);
        }
    }
    out.sort_unstable();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The list is diffed against the REAL mjs in
    /// `tests/datamark_oracle.rs`; this only pins the shape so a structural
    /// regression fails here first, next to the table.
    #[test]
    fn allowed_types_are_sorted_and_deduplicated() {
        let list = backlog_allowed_types();
        let mut sorted = list.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(list, sorted);
        assert_eq!(list.len(), 21);
        assert_eq!(list[0], "approval");
    }

    #[test]
    fn normalized_kinds_keep_first_occurrence_order() {
        assert_eq!(normalized_kinds()[0], "friction");
        assert_eq!(normalize_kind("review-finding"), Some("finding"));
        assert_eq!(normalize_kind("no-such-type"), None);
    }
}
