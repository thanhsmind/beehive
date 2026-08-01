// devtools — Rust port of bee's remaining DEV-SURFACE Node scripts (R4 of
// plans/rust-port.md). These are maintenance/CI tools, not porcelain verbs:
// they run in the bee SOURCE checkout, not on a host repo.
//
//   bee dev render-skill-trees   <- scripts/render_plugin_skill_trees.mjs
//   bee dev render-prompt        <- packages/bee/lib/prompt-renderer.mjs (C4)
//   bee dev statusline           <- packages/bee/statusline/statusline-usage.mjs
//   bee dev impact-registry      <- scripts/impact_registry.mjs
//   bee dev release-manifest     <- scripts/release_manifest.mjs
//
// ROUTING (campaign rule 1, conservative argv routing). `try_native` serves
// only the `dev <name> …` argv shapes proven equivalent to their .mjs; every
// other shape returns None BEFORE any output. `bee dev` is outside the Node
// porcelain namespace entirely, so a None falls through to the CLI delegate
// and reports unknown-command exactly as Node's dispatcher does. The .mjs
// scripts stay runnable directly (`node scripts/release_manifest.mjs --check`)
// for the whole two-runtime window.
//
// REPO ROOT. Each .mjs pins REPO_ROOT to its own file location
// (`path.dirname(fileURLToPath(import.meta.url)) + "/.."`). A single binary
// has no such anchor, so `bee_source_root()` walks up from cwd for the two
// markers that identify a bee source checkout. Run from anywhere inside the
// checkout the answer is identical to the .mjs's; run from outside it, the
// probe returns None and delegates rather than guessing a root.
//
// ERROR-PATH DIVERGENCE (documented, deliberate). Three of these scripts end
// in `catch (error) { console.error(`<tool>: ${error.stack}`) }`. A V8 stack
// is unreproducible (campaign rule 2), and unlike a porcelain verb there is
// no Node command to delegate to once execution has begun. Native failure
// paths therefore print the ERROR MESSAGE where Node prints message + `at`
// frames — a strict prefix of Node's first line, same exit code. Every
// deterministic refusal (marker grammar, --check drift, usage) IS reproduced
// byte-for-byte. Failure shapes that can be detected BEFORE any output (a
// symlink in the skill source, a missing input tree) return None instead.

mod impact_registry;
mod jspath;
mod prompts;
mod release_manifest;
mod skill_trees;
mod statusline;

use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn try_native(args: &[OsString]) -> Option<ExitCode> {
    let strs: Vec<&str> = args.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if strs.first().copied() != Some("dev") {
        return None;
    }
    let rest = &strs[1..];
    let (name, flags) = rest.split_first()?;
    match *name {
        "render-skill-trees" => skill_trees::run(flags),
        "render-prompt" => prompts::run(flags),
        "statusline" => statusline::run(flags),
        "impact-registry" => impact_registry::run(flags),
        "release-manifest" => release_manifest::run(flags),
        _ => None,
    }
}

// ─── repo-root discovery ───────────────────────────────────────────────────

/// The bee SOURCE checkout containing cwd, or None. Markers are two files
/// that exist only in this repo (a vendored `.bee/bin/` install has neither
/// `packages/bee/lib/` nor `scripts/run_verify.mjs`), so a dev tool can never
/// silently operate on a host repo.
pub(crate) fn bee_source_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let cwd = dunce::canonicalize(&cwd).unwrap_or(cwd);
    let mut cur: Option<&Path> = Some(cwd.as_path());
    while let Some(dir) = cur {
        if dir.join("packages").join("bee").join("lib").join("state.mjs").is_file()
            && dir.join("scripts").join("run_verify.mjs").is_file()
        {
            return Some(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    None
}

// ─── shared primitives ─────────────────────────────────────────────────────

/// sha256 hex of raw bytes — `crypto.createHash("sha256").update(x).digest("hex")`.
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// `path.relative(root, abs)` rendered with the PLATFORM separator, exactly
/// as Node prints it in the `WROTE …` lines. (`rel_posix` below is the other
/// flavor — the one that goes INTO manifests.)
pub(crate) fn rel_platform(root: &Path, abs: &Path) -> String {
    let rel = abs.strip_prefix(root).unwrap_or(abs);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(&std::path::MAIN_SEPARATOR.to_string())
}

/// `path.relative(root, abs).split(path.sep).join("/")` — the repo-relative
/// POSIX spelling both manifests key on.
pub(crate) fn rel_posix(root: &Path, abs: &Path) -> String {
    let rel = abs.strip_prefix(root).unwrap_or(abs);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

// ─── String#localeCompare, restricted alphabet ─────────────────────────────
//
// provenance: `a.localeCompare(b)` as called by
//   * release_manifest.mjs  buildRecord sort / compareManifests changed-sort
//   * render_plugin_skill_trees.mjs  walkFiles entry sort
//   * run_verify.mjs  discoverSuites `found.sort(...)`
//
// This is the one comparator in the dev surface that is NOT JS's default
// code-unit sort, and the difference is load-bearing: sorting the real
// release manifest's 326 paths by code unit produces a DIFFERENT order than
// the committed file (measured). So localeCompare has to be reproduced, not
// approximated.
//
// Node's localeCompare is ICU root collation (CLDR default: variable
// weighting non-ignorable, tertiary strength, no numeric ordering). For the
// alphabet these tools actually see — `[A-Za-z0-9._/-]`, repo-relative paths
// — root collation reduces exactly to a two-level comparison:
//
//   primary   `_` < `-` < `.` < `/` < digits < letters (case-folded)
//   tertiary  lowercase before uppercase, compared only after ALL primaries
//
// Note `_` : in ASCII it sits between `Z` and `a`, in ICU it is the FIRST
// punctuation weight. Every other member of the alphabet happens to keep its
// ASCII relative order at the primary level; case folding and the level split
// are the rest of the difference.
//
// Proven, not assumed: a Node harness compared this model against
// `String#localeCompare` over 5,659,641 synthetic pairs (every 1-, 2- and
// 3-char string over `_ - . / 0 1 9 a b z A B Z`) and over all 106,276 pairs
// of the real manifest's paths — zero mismatches (Node v22.14.0, ICU 76.1).
//
// A character OUTSIDE the alphabet is never guessed at: `locale_compare`
// answers None and every caller turns that into a delegate/refusal rather
// than emitting a possibly-misordered file.

fn primary_weight(c: char) -> Option<u32> {
    Some(match c {
        '_' => 1,
        '-' => 2,
        '.' => 3,
        '/' => 4,
        '0'..='9' => 100 + (c as u32 - '0' as u32),
        'a'..='z' => 200 + (c as u32 - 'a' as u32),
        'A'..='Z' => 200 + (c as u32 - 'A' as u32),
        _ => return None,
    })
}

fn tertiary_weight(c: char) -> u32 {
    if c.is_ascii_uppercase() { 1 } else { 0 }
}

/// `a.localeCompare(b)` for the restricted alphabet; None when either string
/// leaves it.
pub(crate) fn locale_compare(a: &str, b: &str) -> Option<Ordering> {
    let pa: Option<Vec<u32>> = a.chars().map(primary_weight).collect();
    let pb: Option<Vec<u32>> = b.chars().map(primary_weight).collect();
    let (pa, pb) = (pa?, pb?);
    match pa.cmp(&pb) {
        Ordering::Equal => {}
        other => return Some(other),
    }
    let ta: Vec<u32> = a.chars().map(tertiary_weight).collect();
    let tb: Vec<u32> = b.chars().map(tertiary_weight).collect();
    Some(ta.cmp(&tb))
}

/// Sort `items` by `key(item).localeCompare(...)`. Returns false (leaving
/// `items` untouched) when any key leaves the proven alphabet.
pub(crate) fn sort_by_locale<T, F: Fn(&T) -> &str>(items: &mut [T], key: F) -> bool {
    if items.iter().any(|i| key(i).chars().any(|c| primary_weight(c).is_none())) {
        return false;
    }
    // Stable, matching V8's Array.prototype.sort. Keys are unique in every
    // call site here, so stability is belt-and-braces.
    items.sort_by(|x, y| locale_compare(key(x), key(y)).unwrap_or(Ordering::Equal));
    true
}

/// JS default `Array.prototype.sort()` on strings: UTF-16 code-unit order.
/// Distinct from `sort_by_locale` above and used where the .mjs calls bare
/// `.sort()` (impact_registry's key/suite sorts, onboard's sidecar skill sort,
/// render_plugin_skill_trees' `listBeeSkillDirs`).
pub(crate) fn js_default_sort(items: &mut [String]) {
    items.sort_by(|a, b| code_unit_cmp(a, b));
}

/// UTF-16 code-unit comparison (V8's `<` on strings).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_compare_puts_underscore_first_unlike_ascii() {
        // ICU: `_` is the first punctuation weight; ASCII puts it after `Z`.
        assert_eq!(locale_compare("_a", "Za"), Some(Ordering::Less));
        assert_eq!(code_unit_cmp("_a", "Za"), Ordering::Greater);
    }

    #[test]
    fn locale_compare_is_case_insensitive_at_primary_level() {
        // ASCII would put every uppercase before every lowercase.
        assert_eq!(locale_compare("apple", "Zebra"), Some(Ordering::Less));
        assert_eq!(code_unit_cmp("apple", "Zebra"), Ordering::Greater);
        // Tertiary only breaks a primary tie, and lowercase wins it.
        assert_eq!(locale_compare("a", "A"), Some(Ordering::Less));
        // …but a primary difference LATER in the string still outranks an
        // earlier case difference (level-by-level, not char-by-char).
        assert_eq!(locale_compare("aB", "Ac"), Some(Ordering::Less));
    }

    #[test]
    fn locale_compare_pins_the_punctuation_order() {
        let mut v = vec![
            "b/x".to_string(),
            "b.x".to_string(),
            "b-x".to_string(),
            "b_x".to_string(),
            "b0x".to_string(),
            "bax".to_string(),
        ];
        assert!(sort_by_locale(&mut v, |s| s.as_str()));
        assert_eq!(v, ["b_x", "b-x", "b.x", "b/x", "b0x", "bax"]);
    }

    #[test]
    fn locale_compare_refuses_characters_it_has_not_proven() {
        assert_eq!(locale_compare("a b", "ab"), None);
        let mut v = vec!["a b".to_string(), "aa".to_string()];
        assert!(!sort_by_locale(&mut v, |s| s.as_str()));
        assert_eq!(v, ["a b", "aa"]); // untouched
    }

    #[test]
    fn rel_posix_and_rel_platform_differ_only_in_separator() {
        let root = Path::new("/repo");
        let abs = Path::new("/repo/a/b/c.md");
        assert_eq!(rel_posix(root, abs), "a/b/c.md");
        assert_eq!(
            rel_platform(root, abs),
            format!("a{0}b{0}c.md", std::path::MAIN_SEPARATOR)
        );
    }

    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
