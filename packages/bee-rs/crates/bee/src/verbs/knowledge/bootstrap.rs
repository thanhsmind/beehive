// bee knowledge bootstrap — stand up docs/knowledge/ in a host repo from
// docs/specs/*.md, when no bundle exists yet.
//
// Feature: docs/history/knowledge-usable/CONTEXT.md (U9).
//
// A NEW verb (no `bee.mjs` twin to port): the whole knowledge machine is
// worthless to a host repo while a bundle exists only in THIS repo, so U9
// gives a host a one-shot way to stand one up from what it already has —
// its `docs/specs/*.md` files — rather than starting from an empty
// directory.
//
// Contract (U9):
//   - one `bee.area` concept per `docs/specs/*.md` file (top-level only —
//     v1 does not recurse into subdirectories, matching the glob literally
//     named in the decision);
//   - the spec body is imported under the area verbatim (any leading
//     frontmatter block on the SOURCE file is stripped first — a host
//     spec's own frontmatter dialect is not assumed to be OKF-shaped, and
//     v1 does not attempt to carry it across);
//   - OKF frontmatter is AUTHORED fresh: `type: bee.area`, `title` from the
//     spec's first ATX heading, `description` from the paragraph
//     immediately following it (falls back to the title when none exists),
//     `timestamp`, and `bee.id`/`bee.areas`/`bee.lifecycle`/
//     `bee.authoritative_for`/`bee.sources` (citing the spec path);
//   - `index.md` + subdir indexes are generated via the same
//     `compute_index_files` machinery `bee knowledge index` uses — no
//     second renderer;
//   - `bundle_mode` (the "does this repo have a bundle?" predicate the
//     session preamble and `bee close` both read) flips true the moment
//     the first area concept lands.
//
// Two typed refusals, BOTH zero-write (checked before anything is read off
// disk for writing, let alone written):
//   - `bundle_exists` — docs/knowledge/ already carries a concept.
//     Bootstrap never touches an existing bundle (must_haves prohibition);
//     the refusal names the read-only/regenerate verbs to use instead.
//   - `no_specs` — docs/specs/ is absent or holds no top-level `.md` file.
//     There is nothing to import.
//
// No code scanning in v1 (must_haves prohibition): a spec bootstrap cannot
// classify — no derivable title, an empty or duplicate slug, or an
// unreadable file — is skipped and named as a GAP in the report rather than
// failing the whole run or guessing at content the spec itself does not
// carry.

#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── spec-body heuristics (Agent's Discretion: v1's classify step) ─────────

/// Strips a leading `---`-delimited frontmatter block from a HOST spec file
/// (an arbitrary dialect, not assumed OKF-shaped — v1 never carries it
/// across, it only needs to not leak into the imported body). No closing
/// delimiter found => the whole text is returned unchanged: guessing where a
/// block "should" have ended risks eating real content.
pub(crate) fn strip_leading_frontmatter(text: &str) -> &str {
    let after_open = if let Some(rest) = text.strip_prefix("---\r\n") {
        rest
    } else if let Some(rest) = text.strip_prefix("---\n") {
        rest
    } else {
        return text;
    };
    let mut consumed = 0usize;
    for line in after_open.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        consumed += line.len();
        if trimmed == "---" {
            return &after_open[consumed..];
        }
    }
    text
}

/// `# Heading` / `## Heading` / ... — 1-6 `#` then a required space, the
/// rest trimmed of surrounding whitespace and any closing `#` run. `None`
/// for a line that is not an ATX heading, OR one with no title text at all
/// (a bare `#` line names nothing to classify from).
pub(crate) fn strip_atx_heading(line: &str) -> Option<String> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &line[hashes..];
    if !rest.starts_with(' ') {
        return None; // ATX requires a space after the run — "#no-space" is prose
    }
    let title = rest.trim().trim_end_matches('#').trim();
    if title.is_empty() {
        return None;
    }
    Some(title.to_string())
}

/// The spec's title (Agent's Discretion): the first ATX heading anywhere in
/// the body. `None` when the spec carries no heading at all — v1 has no
/// second heuristic (no code scanning), so that spec becomes a gap.
pub(crate) fn first_heading(body: &str) -> Option<String> {
    body.lines().find_map(|line| strip_atx_heading(line.trim()))
}

/// The spec's description (Agent's Discretion): the paragraph immediately
/// following the title heading — the run of non-blank, non-heading lines up
/// to the next blank line or heading, space-joined. `None` when nothing
/// follows (the caller falls back to the title, never invents prose).
pub(crate) fn first_paragraph(body: &str) -> Option<String> {
    let mut past_heading = false;
    let mut collected: Vec<&str> = Vec::new();
    for line in body.lines() {
        let t = line.trim();
        if !past_heading {
            if strip_atx_heading(t).is_some() {
                past_heading = true;
            }
            continue;
        }
        if t.is_empty() || strip_atx_heading(t).is_some() {
            if collected.is_empty() {
                if t.is_empty() {
                    continue; // blank lines before the paragraph starts
                }
                break; // a heading with no paragraph text before it
            }
            break;
        }
        collected.push(t);
    }
    if collected.is_empty() {
        None
    } else {
        Some(collected.join(" "))
    }
}

/// A filename stem folded to the same kebab-slug shape existing bundle
/// areas use (`docs/knowledge/areas/<slug>/`): lowercase, runs of
/// non-alphanumeric characters collapse to one `-`, no leading/trailing
/// dash. Empty when the stem carries no letters or digits at all.
pub(crate) fn slug_from_stem(stem: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for c in stem.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.extend(c.to_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

// ─── bootstrap ──────────────────────────────────────────────────────────────

pub(crate) struct BootstrapArea {
    pub(crate) slug: String,
    /// `docs/knowledge/areas/<slug>/overview.md` — directly readable.
    pub(crate) path: String,
    pub(crate) title: String,
}

pub(crate) enum BootstrapOutcome {
    /// docs/knowledge/ already carries a concept — zero writes.
    BundleExists,
    /// docs/specs/ is absent or holds no top-level `.md` file — zero writes.
    NoSpecs,
    Ok {
        created: Vec<BootstrapArea>,
        /// (spec filename under docs/specs/, why it could not be classified)
        gaps: Vec<(String, String)>,
    },
}

/// One `bee.area` concept, built and canonically emitted (so `knowledge
/// check`'s parse→re-emit `not_canonical` warning never fires on a
/// bootstrapped file). `None` on the `Err(())` `emit_frontmatter` guards
/// against (a value the emitted subset cannot carry) — unreachable for the
/// plain strings this function builds, but never unwrapped past it.
fn build_area_concept(slug: &str, title: &str, description: &str, spec_name: &str, today: &str) -> Option<String> {
    let mut bee = Map::new();
    bee.insert("id".to_string(), Value::String(format!("{slug}-overview")));
    bee.insert("lifecycle".to_string(), Value::String("active".to_string()));
    bee.insert("areas".to_string(), Value::Array(vec![Value::String(slug.to_string())]));
    bee.insert(
        "sources".to_string(),
        Value::Array(vec![Value::String(format!("docs/specs/{spec_name}"))]),
    );
    bee.insert(
        "authoritative_for".to_string(),
        Value::String(format!("{slug}: {title}")),
    );
    bee.insert(
        "owns.code".to_string(),
        Value::Array(vec![Value::String(format!("docs/specs/{spec_name}"))]),
    );

    let mut data = Map::new();
    data.insert("type".to_string(), Value::String("bee.area".to_string()));
    data.insert("title".to_string(), Value::String(title.to_string()));
    data.insert("description".to_string(), Value::String(description.to_string()));
    data.insert("timestamp".to_string(), Value::String(today.to_string()));
    data.insert("bee".to_string(), Value::Object(bee));

    emit_frontmatter(&data).ok()
}

/// bootstrapBundle: the whole U9 flow over an already-resolved `root` +
/// bundle `dir`. `today` is threaded in (rather than read from the clock
/// here) so callers — the CLI wrapper and this file's own tests — agree on
/// exactly what timestamp a run produced.
pub(crate) fn bootstrap_bundle(root: &Path, dir: &Path, today: &str) -> BootstrapOutcome {
    // Zero-write refusal #1: an existing bundle is never touched. Reuses the
    // ONE "does this repo have a bundle?" predicate every other caller
    // (session preamble, `bee close`) reads, rather than a second one that
    // could drift from it.
    if crate::hooks::session_preamble::bundle_mode(root) {
        return BootstrapOutcome::BundleExists;
    }

    let specs_dir = root.join("docs").join("specs");
    let mut spec_files: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&specs_dir) {
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else { continue };
            if !file_type.is_file() {
                continue; // v1: top-level only — docs/specs/*.md, not a recursive walk
            }
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
            spec_files.push((name.to_string(), path));
        }
    }
    // Deterministic run order (byte order — the same tiebreak knowledge
    // search/index already use) so two bootstraps over the same specs
    // produce identical CREATED/GAP reports.
    spec_files.sort_by(|a, b| a.0.cmp(&b.0));

    // Zero-write refusal #2: nothing to import.
    if spec_files.is_empty() {
        return BootstrapOutcome::NoSpecs;
    }

    let mut created: Vec<BootstrapArea> = Vec::new();
    let mut gaps: Vec<(String, String)> = Vec::new();
    let mut used_slugs: HashSet<String> = HashSet::new();

    for (name, abs_spec) in &spec_files {
        let stem = name.strip_suffix(".md").unwrap_or(name.as_str());
        let slug = slug_from_stem(stem);
        if slug.is_empty() {
            gaps.push((name.clone(), "filename carries no letters or digits to slug an area from".to_string()));
            continue;
        }
        if !used_slugs.insert(slug.clone()) {
            gaps.push((name.clone(), format!("area slug \"{slug}\" collides with an earlier spec in this run")));
            continue;
        }

        let text = match read_file_lossy(abs_spec) {
            Ok(t) => t,
            Err(e) => {
                gaps.push((name.clone(), format!("could not read the spec: {e}")));
                continue;
            }
        };
        let body = strip_leading_frontmatter(&text).trim_start_matches(['\n', '\r']).to_string();

        let Some(title) = first_heading(&body) else {
            gaps.push((
                name.clone(),
                "no ATX heading (\"# ...\") found to classify a title from — no code scanning in v1".to_string(),
            ));
            continue;
        };
        let description = first_paragraph(&body).unwrap_or_else(|| title.clone());

        let Some(frontmatter) = build_area_concept(&slug, &title, &description, name, today) else {
            gaps.push((name.clone(), "frontmatter could not be canonically emitted".to_string()));
            continue;
        };
        let content = if body.is_empty() { frontmatter } else { format!("{frontmatter}\n{body}") };

        let rel = format!("areas/{slug}/overview.md");
        let abs = join_rel(dir, &rel);
        let Some(parent) = abs.parent() else {
            gaps.push((name.clone(), "area path resolved with no parent directory".to_string()));
            continue;
        };
        if let Err(e) = std::fs::create_dir_all(parent) {
            gaps.push((name.clone(), format!("could not create the area directory: {e}")));
            continue;
        }
        if let Err(e) = std::fs::write(&abs, &content) {
            gaps.push((name.clone(), format!("could not write the area concept: {e}")));
            continue;
        }

        created.push(BootstrapArea { slug, path: format!("docs/knowledge/{rel}"), title });
    }

    // Render the generated indexes over what was just created — the exact
    // machinery `bee knowledge index` uses, never a second renderer. Only
    // when at least one area landed: an all-gaps run leaves docs/knowledge/
    // untouched rather than standing up an index over nothing.
    if !created.is_empty() {
        if let Some(files) = compute_index_files(dir) {
            for (rel, index_content) in &files {
                let abs = join_rel(dir, rel);
                if let Some(parent) = abs.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&abs, index_content);
            }
        }
    }

    BootstrapOutcome::Ok { created, gaps }
}

// ─── routing ────────────────────────────────────────────────────────────

pub(crate) fn run_bootstrap(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &[]) {
        return None;
    }
    let ctx = match g_prelude("knowledge bootstrap", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let dir = bundle_dir(&ctx.root)?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    match bootstrap_bundle(&ctx.root, &dir, &today) {
        BootstrapOutcome::BundleExists => Some(ctx.fail(
            "knowledge bootstrap: bundle_exists — docs/knowledge/ already carries a concept; bootstrap \
             stands up a bundle only where none exists and never touches an existing one. Use `bee \
             knowledge check` or `bee knowledge index` on the existing bundle instead.",
        )),
        BootstrapOutcome::NoSpecs => Some(ctx.fail(
            "knowledge bootstrap: no_specs — docs/specs/ is absent or holds no top-level .md file, so \
             there is nothing to import. Author docs/specs/*.md first, or start the bundle by hand under \
             docs/knowledge/.",
        )),
        BootstrapOutcome::Ok { created, gaps } => {
            let mut lines: Vec<String> = Vec::new();
            for area in &created {
                lines.push(format!("CREATED {} — {}", area.path, area.title));
            }
            for (spec, reason) in &gaps {
                lines.push(format!("GAP docs/specs/{spec} — {reason}"));
            }
            let failing = created.is_empty();
            lines.push(format!(
                "knowledge bootstrap: {} area(s) created, {} gap(s) — {}",
                created.len(),
                gaps.len(),
                if failing { "FAIL (no spec could be classified)" } else { "OK" }
            ));

            let created_rows: Vec<Value> = created
                .iter()
                .map(|a| {
                    let mut m = Map::new();
                    m.insert("path".into(), Value::String(a.path.clone()));
                    m.insert("area".into(), Value::String(a.slug.clone()));
                    m.insert("title".into(), Value::String(a.title.clone()));
                    Value::Object(m)
                })
                .collect();
            let gap_rows: Vec<Value> = gaps
                .iter()
                .map(|(spec, reason)| {
                    let mut m = Map::new();
                    m.insert("spec".into(), Value::String(format!("docs/specs/{spec}")));
                    m.insert("reason".into(), Value::String(reason.clone()));
                    Value::Object(m)
                })
                .collect();
            let mut result = Map::new();
            result.insert("created".into(), Value::Array(created_rows));
            result.insert("gaps".into(), Value::Array(gap_rows));

            Some(ctx.emit(&Value::Object(result), &lines.join("\n"), u8::from(failing)))
        }
    }
}

// ─── tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        (tmp, PathBuf::new())
    }

    fn write_spec(root: &Path, name: &str, text: &str) {
        let dir = root.join("docs").join("specs");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(name), text).unwrap();
    }

    fn bundle_dir_of(root: &Path) -> PathBuf {
        root.join("docs").join("knowledge")
    }

    #[test]
    fn fresh_bootstrap_creates_a_bundle_check_accepts_and_bundle_mode_reads_true() {
        let (tmp, _) = root();
        let root = tmp.path();
        write_spec(
            root,
            "advisor-protocol.md",
            "# Advisor Protocol\n\nThe advisor protocol governs how outside advice enters a session.\n\nMore prose.\n",
        );
        write_spec(
            root,
            "doctrine layer.md", // a space in the filename — slug must fold it
            "# Doctrine Layer\n\nThe doctrine layer holds standing rules.\n",
        );

        let dir = bundle_dir_of(root);
        let outcome = bootstrap_bundle(root, &dir, "2026-08-10");
        let BootstrapOutcome::Ok { created, gaps } = outcome else {
            panic!("expected Ok, got a refusal");
        };
        assert!(gaps.is_empty(), "unexpected gaps: {gaps:?}");
        assert_eq!(created.len(), 2);

        let paths: Vec<&str> = created.iter().map(|a| a.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "docs/knowledge/areas/advisor-protocol/overview.md",
                "docs/knowledge/areas/doctrine-layer/overview.md",
            ]
        );

        // knowledge check accepts the fresh bundle (zero errors/profile errors).
        let report = check_bundle(&dir, false).expect("bundle walk");
        assert!(report.okf_errors.is_empty(), "{:?}", report.okf_errors);
        assert!(report.profile_errors.is_empty(), "{:?}", report.profile_errors);
        assert!(report.ok, "check_bundle should accept a fresh bootstrap");

        // bundle_mode — the ONE "does this repo have a bundle?" predicate
        // the session preamble and `bee close` both read — now reads true.
        assert!(crate::hooks::session_preamble::bundle_mode(root));

        // the generated indexes landed too, via the shared index machinery.
        assert!(dir.join("index.md").exists());
        assert!(dir.join("areas").join("index.md").exists());
    }

    #[test]
    fn existing_bundle_is_a_typed_refusal_with_zero_writes() {
        let (tmp, _) = root();
        let root = tmp.path();
        // an existing bundle: one concept file already present.
        let dir = bundle_dir_of(root);
        std::fs::create_dir_all(dir.join("areas").join("existing")).unwrap();
        std::fs::write(
            dir.join("areas").join("existing").join("overview.md"),
            "---\ntype: bee.area\ntitle: Existing\ndescription: Existing\ntimestamp: 2026-01-01\nbee:\n  id: existing-overview\n  lifecycle: active\n---\nAlready here.\n",
        )
        .unwrap();
        write_spec(root, "unused.md", "# Unused\n\nnever imported.\n");

        let before: Vec<PathBuf> = walk_all(&dir);
        let outcome = bootstrap_bundle(root, &dir, "2026-08-10");
        assert!(matches!(outcome, BootstrapOutcome::BundleExists));
        let after: Vec<PathBuf> = walk_all(&dir);
        assert_eq!(before, after, "an existing bundle must never be touched (U9 prohibition)");
        // the unused spec's area was never written.
        assert!(!dir.join("areas").join("unused").exists());
    }

    #[test]
    fn no_specs_directory_is_a_typed_refusal_with_zero_writes() {
        let (tmp, _) = root();
        let root = tmp.path();
        let dir = bundle_dir_of(root);
        let outcome = bootstrap_bundle(root, &dir, "2026-08-10");
        assert!(matches!(outcome, BootstrapOutcome::NoSpecs));
        assert!(!dir.exists(), "docs/knowledge/ must not be created on the no_specs refusal");
    }

    #[test]
    fn empty_specs_directory_is_also_no_specs() {
        let (tmp, _) = root();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("docs").join("specs")).unwrap();
        std::fs::write(root.join("docs").join("specs").join("readme.txt"), "not markdown").unwrap();
        let dir = bundle_dir_of(root);
        let outcome = bootstrap_bundle(root, &dir, "2026-08-10");
        assert!(matches!(outcome, BootstrapOutcome::NoSpecs));
    }

    #[test]
    fn a_spec_with_no_heading_is_named_as_a_gap_not_a_crash() {
        let (tmp, _) = root();
        let root = tmp.path();
        write_spec(root, "no-heading.md", "just prose, no heading anywhere.\n");
        write_spec(root, "has-heading.md", "# Has Heading\n\nreal body.\n");

        let dir = bundle_dir_of(root);
        let BootstrapOutcome::Ok { created, gaps } = bootstrap_bundle(root, &dir, "2026-08-10") else {
            panic!("expected Ok");
        };
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].slug, "has-heading");
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].0, "no-heading.md");
        assert!(gaps[0].1.contains("no code scanning"), "{}", gaps[0].1);
    }

    #[test]
    fn duplicate_slugs_keep_the_first_and_gap_the_rest() {
        let (tmp, _) = root();
        let root = tmp.path();
        write_spec(root, "cache-warm.md", "# Cache Warm\n\nfirst.\n");
        write_spec(root, "cache_warm.md", "# Cache Warm Again\n\nsecond, same slug.\n");

        let dir = bundle_dir_of(root);
        let BootstrapOutcome::Ok { created, gaps } = bootstrap_bundle(root, &dir, "2026-08-10") else {
            panic!("expected Ok");
        };
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].slug, "cache-warm");
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].0, "cache_warm.md");
        assert!(gaps[0].1.contains("collides"), "{}", gaps[0].1);
    }

    #[test]
    fn strip_leading_frontmatter_removes_a_host_dialect_block() {
        let text = "---\narea: porcelain\nupdated: 2026-08-03\n---\n\n# Title\n\nbody\n";
        assert_eq!(strip_leading_frontmatter(text), "\n# Title\n\nbody\n");
    }

    #[test]
    fn strip_leading_frontmatter_leaves_text_without_one_unchanged() {
        let text = "# Title\n\nbody\n";
        assert_eq!(strip_leading_frontmatter(text), text);
    }

    #[test]
    fn slug_from_stem_folds_spaces_and_underscores_and_case() {
        assert_eq!(slug_from_stem("Doctrine Layer"), "doctrine-layer");
        assert_eq!(slug_from_stem("cache_warm"), "cache-warm");
        assert_eq!(slug_from_stem("already-kebab"), "already-kebab");
        assert_eq!(slug_from_stem("...."), "");
    }

    #[test]
    fn first_paragraph_falls_back_to_none_when_the_heading_is_immediately_followed_by_another_heading() {
        let body = "# Title\n## Subheading\nprose under the subheading.\n";
        assert_eq!(first_paragraph(body), None);
    }

    fn walk_all(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        fn rec(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else { return };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    rec(&path, out);
                } else {
                    out.push(path);
                }
            }
        }
        if dir.exists() {
            rec(dir, &mut out);
        }
        out.sort();
        out
    }
}
