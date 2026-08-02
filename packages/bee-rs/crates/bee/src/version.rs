// version — THE bee release version, single-sourced.
//
// R6 CUTOVER DECISION (owner, 2026-08-01). `BEE_VERSION` used to live in
// `packages/bee/lib/state.mjs` as
//
//     export const BEE_VERSION = '1.20.3';
//
// and every Rust consumer either re-parsed that line at runtime
// (`onboard::source::read_version_strict`) or carried a hand-maintained copy
// of the literal (`hooks/compaction.rs`, `hooks/session_preamble.rs`,
// `verbs/status_full.rs` — three copies, with NOTHING enforcing that they
// agreed with each other or with the .mjs). With the Node tree deleted the
// declaration has no file to live in, so it moves to
// `.claude-plugin/plugin.json`, which the installer already reads and which
// `.codex-plugin/plugin.json` and the release manifest already mirror.
//
// Two properties are deliberate:
//
//   1. COMPILE TIME, not runtime. The manifest is `include_str!`d and the
//      version extracted by a `const fn`, so the binary cannot disagree with
//      the checkout it was built from, and a manifest that loses its
//      `"version"` key is a BUILD ERROR (`panic!` in const context), never a
//      silently empty string at runtime.
//   2. ONE copy. The three former literals now `use crate::version::BEE_VERSION`.
//      Bumping a release means editing the two plugin manifests and nothing
//      else; `onboard::source::read_source_release_identity` still proves the
//      two agree with each other at runtime.

/// The authoritative manifest, embedded at build time. The path is relative
/// to this source file: crates/bee/src → crates/bee → crates → bee-rs →
/// packages → <repo root>.
const PLUGIN_MANIFEST: &str = include_str!("../../../../../.claude-plugin/plugin.json");

/// THE version string, e.g. `"1.20.3"`.
pub const BEE_VERSION: &str = parse_version(PLUGIN_MANIFEST);

/// Extract `"version": "<value>"` from the embedded manifest at compile time.
///
/// This is a deliberately narrow scanner, not a JSON parser: it finds the
/// first `"version"` key, skips whitespace and the colon, and takes the
/// double-quoted string that follows. The manifest is a fixed file in this
/// repo, so the grammar it has to cope with is exactly the grammar it has.
/// Anything else is a compile-time panic naming the file to fix.
const fn parse_version(src: &str) -> &str {
    let bytes = src.as_bytes();
    let key = b"\"version\"";
    let mut i = 0;
    while i + key.len() <= bytes.len() {
        // match the key
        let mut k = 0;
        while k < key.len() && bytes[i + k] == key[k] {
            k += 1;
        }
        if k == key.len() {
            // skip whitespace, then the colon, then whitespace
            let mut j = i + key.len();
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b':' {
                j += 1;
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'"' {
                    let start = j + 1;
                    let mut end = start;
                    while end < bytes.len() && bytes[end] != b'"' {
                        end += 1;
                    }
                    if end < bytes.len() {
                        // `bytes[start..end]`, const-safely.
                        let (_, rest) = bytes.split_at(start);
                        let (value, _) = rest.split_at(end - start);
                        return match core::str::from_utf8(value) {
                            Ok(s) => s,
                            Err(_) => panic!(
                                "\
.claude-plugin/plugin.json: the \"version\" value is not valid UTF-8"
                            ),
                        };
                    }
                }
            }
        }
        i += 1;
    }
    panic!(
        "\
.claude-plugin/plugin.json carries no `\"version\": \"...\"` key. That file is THE source of \
truth for BEE_VERSION since the R6 Node cutover (see crates/bee/src/version.rs) — restore the \
key rather than reintroducing a hand-maintained copy of the literal in Rust."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The version is only useful if it is a real numeric release triple —
    /// `onboard::source::is_numeric_release` refuses anything else at
    /// runtime, and a manifest carrying `"0.0.0-dev"` would make every
    /// onboarding block with an unhelpful message. Catch it here instead.
    #[test]
    fn the_embedded_version_is_a_numeric_release_triple() {
        let parts: Vec<&str> = BEE_VERSION.split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "BEE_VERSION `{BEE_VERSION}` (from .claude-plugin/plugin.json) is not a three-part release"
        );
        for p in parts {
            assert!(
                !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()),
                "BEE_VERSION `{BEE_VERSION}` (from .claude-plugin/plugin.json) has a non-numeric part `{p}`"
            );
        }
    }

    /// The compile-time scanner and a real JSON parse must agree. If they
    /// ever diverge (a reordered manifest, a `"version"` appearing first
    /// inside a nested object), this is the test that says so.
    #[test]
    fn the_const_scanner_agrees_with_a_real_json_parse() {
        let parsed: serde_json::Value = serde_json::from_str(PLUGIN_MANIFEST)
            .expect(".claude-plugin/plugin.json must be valid JSON");
        assert_eq!(
            parsed.get("version").and_then(serde_json::Value::as_str),
            Some(BEE_VERSION),
            "the const scanner in version.rs disagrees with a real JSON parse of \
             .claude-plugin/plugin.json — the manifest's shape changed under it"
        );
    }

    /// The two plugin manifests are one release tuple (the runtime check in
    /// `onboard::source::read_source_release_identity` proves it for a live
    /// checkout; this proves it for the bytes this binary was BUILT from).
    #[test]
    fn both_plugin_manifests_carry_the_same_version() {
        const CODEX_MANIFEST: &str = include_str!("../../../../../.codex-plugin/plugin.json");
        let codex: serde_json::Value =
            serde_json::from_str(CODEX_MANIFEST).expect(".codex-plugin/plugin.json must be valid JSON");
        assert_eq!(
            codex.get("version").and_then(serde_json::Value::as_str),
            Some(BEE_VERSION),
            ".codex-plugin/plugin.json disagrees with .claude-plugin/plugin.json — bump both"
        );
    }
}
