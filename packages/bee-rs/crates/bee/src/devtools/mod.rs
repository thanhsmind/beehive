// devtools — Rust port of bee's remaining DEV-SURFACE Node scripts (R4 of
// plans/rust-port.md). These are maintenance/CI tools, not porcelain verbs:
// they run in the bee SOURCE checkout, not on a host repo.
//
//   bee dev render-skill-trees   <- scripts/render_plugin_skill_trees.mjs
//   bee dev render-prompt        <- packages/bee/lib/prompt-renderer.mjs (C4)
//   bee dev statusline           <- packages/bee/statusline/statusline-usage.mjs
//   bee dev release-manifest     <- scripts/release_manifest.mjs
//   bee dev plugin-distribution <- packages/bee/scripts/plugin_distribution.mjs
//
// RETIRED at the R6 cutover: `bee dev impact-registry`. Its whole subject was
// the Node suite graph — it parsed `scripts/run_verify.mjs`'s SUITES and
// walked `.mjs` import edges to decide which suites a changed file could
// affect. With the Node tree gone there is no graph to walk, and the cargo
// suite it would have been re-pointed at runs in ~20s end to end, so
// impact-based test filtering buys nothing. The tool,
// `scripts/impact-registry.json`, `scripts/verify-cache-inputs.json` and the
// cap-time E1 cross-check in verbs/cells.rs went with it.
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

mod hook_manifests;
mod install_support;
mod jspath;
mod plugin_distribution;
mod prompts;
mod release_manifest;
mod skill_trees;
mod statusline;

use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The `dev` verbs whose implementation resolves `bee_source_root()` and
/// returns None without it. `statusline` is deliberately absent: it renders a
/// HOST repo's status line and works anywhere, so guarding it would refuse a
/// call that succeeds today.
const SOURCE_CHECKOUT_DEV_VERBS: [&str; 4] = [
    "render-skill-trees",
    "render-prompt",
    "release-manifest",
    "render-hook-manifests",
];

pub fn try_native(args: &[OsString]) -> Option<ExitCode> {
    let strs: Vec<&str> = args.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if strs.first().copied() != Some("dev") {
        return None;
    }
    let rest = &strs[1..];
    let (name, flags) = rest.split_first()?;
    // A REAL dev verb outside a bee source checkout used to return None here.
    // While the delegate existed that meant "let Node answer"; since the
    // cutover it means the argv reaches `emit_unsupported_shape`, which tells
    // the caller `bee dev render-skill-trees` is an unknown command and
    // suggests `bee test, bee gate, bee close`. The command is not unknown —
    // its precondition is unmet, and that is what it must say. (Inside a
    // checkout nothing changes: a bad flag still falls through to the
    // unknown-shape path, which is the right answer for a bad flag.)
    if SOURCE_CHECKOUT_DEV_VERBS.contains(name) && bee_source_root().is_none() {
        eprintln!(
            "bee dev {name}: the dev surface runs in a bee SOURCE checkout \
             (one containing packages/bee/ and packages/bee-rs/), and this is not one. \
             FIX: cd into the bee checkout, or run `bee onboard` if you meant to install bee here."
        );
        return Some(ExitCode::from(1));
    }
    match *name {
        "render-skill-trees" => skill_trees::run(flags),
        "render-prompt" => prompts::run(flags),
        "statusline" => statusline::run(flags),
        "release-manifest" => release_manifest::run(flags),
        "plugin-distribution" => plugin_distribution::run(flags),
        "install-support" => install_support::run(flags),
        "render-hook-manifests" => hook_manifests::run(flags),
        _ => None,
    }
}

/// The release manifest's inventory roots, for `verbs/cells.rs`'s regen
/// obligation. Re-exported here so the obligation depends on the devtools
/// MODULE contract rather than reaching into a private submodule.
pub(crate) fn release_manifest_roots() -> &'static [&'static str] {
    release_manifest::INVENTORY_ROOTS
}

/// The manifest file's own repo-relative path (the regen obligation's required
/// file), from the same module that writes it.
pub(crate) fn release_manifest_rel() -> &'static str {
    release_manifest::MANIFEST_REL
}

// ─── repo-root discovery ───────────────────────────────────────────────────

/// The bee SOURCE checkout containing cwd, or None. TWO markers must both be
/// present, so a dev tool can never silently operate on a host repo — nor on a
/// vendored install, nor on an unpacked plugin package.
///
/// R6 CUTOVER. The markers were `packages/bee/lib/state.mjs` +
/// `scripts/run_verify.mjs`; both are deleted. Their replacements are chosen
/// to keep the SAME two exclusions, which is the whole point of there being
/// two:
///
///   * `packages/bee/AGENTS.block.md` — the template payload. Excludes a HOST
///     repo and a vendored `.bee/bin/` install: a host receives `.bee/`, never
///     `packages/bee/`. (This is also `onboard::source::Engine::locate`'s
///     marker, so "where is the engine" has ONE answer across the binary.)
///   * `packages/bee-rs/crates/bee/Cargo.toml` — the Rust sources. Excludes an
///     unpacked PLUGIN PACKAGE, which ships `packages/bee/` but not the crate
///     it was built from (the release manifest's `package_payload` root is
///     `packages/bee`; `packages/bee-rs` is deliberately not in the shipped
///     frame). A dev verb that regenerates committed artifacts must never run
///     against a package where those artifacts are the product, not the source.
///
/// A single marker would have collapsed one of those two exclusions silently,
/// so both are kept.
pub(crate) fn bee_source_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let cwd = dunce::canonicalize(&cwd).unwrap_or(cwd);
    let mut cur: Option<&Path> = Some(cwd.as_path());
    while let Some(dir) = cur {
        if dir.join("packages").join("bee").join("AGENTS.block.md").is_file()
            && dir
                .join("packages")
                .join("bee-rs")
                .join("crates")
                .join("bee")
                .join("Cargo.toml")
                .is_file()
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

/// Default string order for the skill-tree sidecar digests and the
/// stored-vs-current diff report: kept as UTF-16 code-unit order
/// (`crate::textutil::code_unit_cmp`/`js_default_sort` — see that module's
/// EXCEPTION note for the reproduction rationale: `skill_trees.rs`'s
/// `manifest_fingerprint`/sha256 sidecar digests and `release_manifest.rs`'s
/// `diff.missing`/`diff.added`). Distinct from `sort_by_locale` above, which
/// reproduces `localeCompare` — and which is what orders the release
/// manifest's OWN stored path list, not this comparator.
pub(crate) use crate::textutil::{code_unit_cmp, js_default_sort};

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

/// The hooks manifest this binary renders for a runtime's REPO target, by
/// name. `doctor` compares a host's wiring against it byte for byte, and it
/// has no business knowing hook_manifests' enums to ask that question.
///
/// `opencode` is a recognized runtime label here — accepted, not refused —
/// but always answers None: hook_manifests::Runtime carries a NAMED
/// exclusion for it (see that enum's doc comment), because OpenCode's hook
/// belt is the checked-in `.opencode/plugins/bee-guard.ts` plugin, not a
/// rendered JSON manifest this function could produce. A caller (e.g.
/// `doctor`) reads None the same way it already does for any runtime with
/// nothing to byte-compare.
pub fn render_projection_text_for(runtime: &str) -> Option<String> {
    let r = match runtime {
        "claude" => hook_manifests::Runtime::Claude,
        "codex" => hook_manifests::Runtime::Codex,
        // "opencode" is recognized, not merely unmatched — see the doc
        // comment above — but it shares None with every other unknown
        // label because there is nothing to render either way.
        "opencode" => return None,
        _ => return None,
    };
    Some(hook_manifests::render_projection_text(r, hook_manifests::Target::Repo))
}

#[cfg(test)]
mod render_projection_text_for_tests {
    use super::render_projection_text_for;

    #[test]
    fn claude_and_codex_render_something_opencode_recognized_but_none() {
        assert!(render_projection_text_for("claude").is_some());
        assert!(render_projection_text_for("codex").is_some());
        // Recognized runtime label, but hook_manifests::Runtime names an
        // exclusion for it (opencode's belt is a TS plugin, not a rendered
        // manifest) — same None an unknown label would get, for a different,
        // documented reason.
        assert_eq!(render_projection_text_for("opencode"), None);
        assert_eq!(render_projection_text_for("emacs"), None);
    }
}
