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
//! See `docs/history/herding-orchestration/CONTEXT.md`, decisions D2 and D5.

use std::fs;
use std::path::Path;

/// True if `header` (the text between `[` and `]` on a TOML table line, for
/// example `dependencies` or `target.'cfg(windows)'.dependencies`) names a
/// dependency table.
fn is_dependency_table(header: &str) -> bool {
    header == "dependencies" || header == "dev-dependencies" || header == "build-dependencies" || header.ends_with(".dependencies")
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

#[test]
fn fleet_manifest_never_depends_on_bee() {
    // env!("CARGO_MANIFEST_DIR") resolves to this crate's own directory at
    // compile time, on every platform including Windows, so this works
    // whether cargo test runs from the workspace root or the crate itself.
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("could not read fleet's own manifest at {}: {e}", manifest_path.display()));

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
    }
}
