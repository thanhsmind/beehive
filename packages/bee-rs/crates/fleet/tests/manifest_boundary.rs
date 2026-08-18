//! D5 states the crate edge is the only mechanism that enforces D2: a
//! generic worker-coordination core must never depend on `bee`. Without a
//! test, that boundary is review-by-eye — one `bee` import inside `fleet`
//! would compile green and every behavioural test would still pass, quietly
//! turning the core into a second copy of the `bee` binary's own concerns.
//!
//! This test reads `fleet`'s own `Cargo.toml` from disk and fails if it
//! declares a dependency named `bee` in any table — normal, dev, build, or
//! a platform-conditional `target.'cfg(...)'.dependencies` table.
//!
//! It also fails on the narrower alias gap: a dependency in one of those
//! tables that uses a DIFFERENT key but whose `path` still resolves to the
//! `bee` crate's own directory (`notbee = { path = "../bee" }` or
//! `notbee.path = "../bee"`). Cargo's own cyclic-package-dependency check
//! already refuses a NORMAL dependency that points at `bee` regardless of
//! what the key is named — that check runs on package identity, not on the
//! manifest text — but it does not fire for `dev-dependencies`,
//! `build-dependencies`, or a `target.'cfg(...)'.dependencies` table, so
//! this test is the only thing that catches an alias planted there.
//!
//! See `docs/history/herding-orchestration/CONTEXT.md`, decisions D2 and D5.

use std::fs;
use std::path::Path;

/// True if `header` (the text between `[` and `]` on a TOML table line, for
/// example `dependencies` or `target.'cfg(windows)'.dependencies`) names a
/// dependency table.
fn is_dependency_table(header: &str) -> bool {
    header == "dependencies"
        || header == "dev-dependencies"
        || header == "build-dependencies"
        || header.ends_with(".dependencies")
}

/// True if `line`, once any trailing comment is stripped, declares a
/// dependency literally named `bee` — either a bare key (`bee = "1"`) or a
/// dotted key (`bee.path = "../bee"`). A comment mentioning `bee` does not
/// count: the comment text is stripped before the check runs.
fn declares_bee_dependency(line: &str) -> bool {
    let without_comment = line.split('#').next().unwrap_or("");
    let trimmed = without_comment.trim();
    let Some(rest) = trimmed.strip_prefix("bee") else {
        return false;
    };
    match rest.chars().next() {
        None => true,
        Some(c) => c == '=' || c == '.' || c.is_whitespace(),
    }
}

/// True if `line`, once any trailing comment is stripped, declares a
/// dependency (under any key/alias) whose `path` value's last path
/// segment is literally `bee` — for example `notbee = { path = "../bee" }`
/// (inline-table form) or `notbee.path = "../bee"` (dotted form). This is
/// what closes the alias gap: the key naming the dependency can be
/// anything, but the directory it points at cannot secretly be `bee`'s.
fn path_value_targets_bee_crate(line: &str) -> bool {
    let without_comment = line.split('#').next().unwrap_or("");
    let mut rest = without_comment;
    while let Some(idx) = rest.find("path") {
        let after_path = rest[idx + "path".len()..].trim_start();
        if let Some(after_eq) = after_path.strip_prefix('=') {
            let after_eq = after_eq.trim_start();
            if let Some(after_quote) = after_eq.strip_prefix('"') {
                if let Some(end) = after_quote.find('"') {
                    let path_value = &after_quote[..end];
                    let normalized = path_value.trim_end_matches(['/', '\\']);
                    let last_segment = normalized.rsplit(['/', '\\']).next().unwrap_or(normalized);
                    if last_segment == "bee" {
                        return true;
                    }
                }
            }
        }
        rest = &rest[idx + "path".len()..];
    }
    false
}

#[test]
fn fleet_manifest_never_depends_on_bee() {
    // env!("CARGO_MANIFEST_DIR") resolves to this crate's own directory at
    // compile time, on every platform including Windows, so this works
    // whether cargo test runs from the workspace root or the crate itself.
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        panic!(
            "could not read fleet's own manifest at {}: {e}",
            manifest_path.display()
        )
    });

    let mut current_table: Option<String> = None;
    for raw_line in manifest.lines() {
        let line = raw_line.trim();
        if let Some(inner) = line.strip_prefix('[') {
            if let Some(header) = inner.strip_suffix(']') {
                current_table = Some(header.trim().to_string());
            }
            continue;
        }
        let Some(table) = current_table.as_deref() else {
            continue;
        };
        if !is_dependency_table(table) {
            continue;
        }
        assert!(
            !declares_bee_dependency(line),
            "fleet's Cargo.toml declares a dependency on bee in [{table}]: `{line}`.\n\
             D5 makes the crate boundary the only mechanism that enforces D2: fleet is a \
             generic worker-coordination core and must never depend on bee, even to reuse \
             a helper — that dependency is what would let bee-shaped concepts (cells, \
             lanes, worktrees, gates, proof) leak into the core silently."
        );
        assert!(
            !path_value_targets_bee_crate(line),
            "fleet's Cargo.toml declares a dependency in [{table}] whose `path` value points \
             at the bee crate under a different name: `{line}`.\n\
             Aliasing the dependency key does not change what gets linked in — cargo's own \
             cyclic-package-dependency check only fires for a NORMAL dependency, so dev-, \
             build-, and target-conditional tables need this check instead."
        );
    }
}

#[cfg(test)]
mod helper_tests {
    use super::path_value_targets_bee_crate;

    #[test]
    fn dotted_alias_pointing_at_bee_crate_is_detected() {
        assert!(path_value_targets_bee_crate("notbee.path = \"../bee\""));
    }

    #[test]
    fn inline_table_alias_pointing_at_bee_crate_is_detected() {
        assert!(path_value_targets_bee_crate(
            "notbee = { path = \"../../crates/bee\" }"
        ));
    }

    #[test]
    fn inline_table_alias_with_trailing_slash_is_still_detected() {
        assert!(path_value_targets_bee_crate(
            "core-cli = { path = \"../bee/\" }"
        ));
    }

    #[test]
    fn a_path_dependency_that_does_not_target_bee_is_not_flagged() {
        assert!(!path_value_targets_bee_crate(
            "logging.path = \"../logging\""
        ));
        assert!(!path_value_targets_bee_crate("fleet = { path = \".\" }"));
    }

    #[test]
    fn a_line_with_no_path_key_is_not_flagged() {
        assert!(!path_value_targets_bee_crate("anyhow = \"1\""));
    }
}
