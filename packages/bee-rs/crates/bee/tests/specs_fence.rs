// The docs/specs read-only fence.
//
// `docs/knowledge/areas/okf-profile/specs-read-only-fence.md` says the verify
// chain classifies every file under the compatibility surface on every green
// run, in two forms — a self-test proving the classifier bites, and a check of
// this repo's own surface — and fails by name on new content.
//
// It said that while nothing ran. `scripts/okf_specs_fence.mjs` was deleted with
// the Node runtime and never ported, so for weeks four files sat in docs/specs
// as new area truth with no red anywhere. That is the failure this fence exists
// to prevent, happening to the fence itself; the concept's own words for it are
// "a rotted allowlist stops fencing SILENTLY".
//
// This is the port. It is a test rather than a verb on purpose: the surface it
// guards is bee's own repository tree, which never ships to a host.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// R3 — named exceptions are closed by decision, and each carries its reason
/// HERE, in the classifier, so the report can print why a file was let through
/// without anyone reading the implementation.
const NAMED_EXCEPTIONS: &[(&str, &str)] = &[(
    "reading-map.md",
    "the hand-written navigation surface: a \"where does X live\" map that points AT the bundle \
     and is never area truth itself (G4)",
)];

/// R4 — a placeholder passes only while it is provably unwritten. Each is named
/// and reasoned, exactly like an exception.
const PLACEHOLDERS: &[(&str, &str)] = &[(
    "system-overview.md",
    "an unwritten placeholder: it holds no content to migrate, so no stub exists to point \
     anywhere, and it is pinned to that state",
)];

#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// A migrated source path carrying the marker the migration writes.
    Stub,
    /// A stub whose `migrated_to` target does not exist. Its own class, never
    /// lumped in with new content: "the migration target moved" is a different
    /// problem from "someone wrote truth here".
    DanglingStub,
    Navigation,
    Placeholder,
    NewContent,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn frontmatter(text: &str) -> Option<BTreeMap<String, String>> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    let mut out = BTreeMap::new();
    for line in rest[..end].lines() {
        if let Some((k, v)) = line.split_once(':') {
            if !k.starts_with(char::is_whitespace) {
                out.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    Some(out)
}

/// R4's "provably unwritten": a placeholder may explain what it is and where its
/// content belongs, and may hold nothing else. A `##` section is prose taking
/// shape, which is exactly the moment this must start failing.
fn is_unwritten(text: &str) -> bool {
    text.to_lowercase().contains("not written yet")
        && !text.lines().any(|l| l.starts_with("## "))
}

/// R2 — structural, never by filename. A filename list rots the first time an
/// area is added or renamed, and a rotted list stops fencing silently.
fn classify(rel_name: &str, text: &str, bundle_root: &Path) -> Verdict {
    if let Some(fm) = frontmatter(text) {
        if let Some(target) = fm.get("migrated_to") {
            let target = target.trim_matches(['"', '\'']);
            let stripped = target.strip_prefix("docs/knowledge/").unwrap_or(target);
            return if bundle_root.join(stripped).exists() {
                Verdict::Stub
            } else {
                Verdict::DanglingStub
            };
        }
    }
    if NAMED_EXCEPTIONS.iter().any(|(n, _)| *n == rel_name) {
        return Verdict::Navigation;
    }
    if PLACEHOLDERS.iter().any(|(n, _)| *n == rel_name) && is_unwritten(text) {
        return Verdict::Placeholder;
    }
    Verdict::NewContent
}

// ── form 1: the self-test — prove the classifier actually bites ────────────

#[test]
fn the_classifier_bites_in_every_direction() {
    let tmp = tempfile::tempdir().unwrap();
    let bundle = tmp.path().join("knowledge");
    std::fs::create_dir_all(bundle.join("areas/x")).unwrap();
    std::fs::write(bundle.join("areas/x/overview.md"), "x").unwrap();

    let stub = "---\narea: x\nmigrated_to: docs/knowledge/areas/x/overview.md\n---\n\n# x (stub)\n";
    assert_eq!(classify("x.md", stub, &bundle), Verdict::Stub);

    // The marker is what makes a stub. Drop it and the same file is new
    // content — the intended behaviour, not a false positive: an unmarked file
    // in a read-only tree is indistinguishable from freshly authored truth.
    let unmarked = "---\narea: x\n---\n\n# x\n\n## Rules\n\n- R1 — something\n";
    assert_eq!(classify("x.md", unmarked, &bundle), Verdict::NewContent);

    // A marker pointing nowhere is its own verdict, never silence.
    let dangling = "---\nmigrated_to: docs/knowledge/areas/gone/overview.md\n---\n\n# x\n";
    assert_eq!(classify("x.md", dangling, &bundle), Verdict::DanglingStub);

    // R2, stated as a test: recognition is structural. A file NAMED like a
    // migrated spec but carrying no marker gets no credit for its name.
    assert_eq!(classify("workflow-state.md", unmarked, &bundle), Verdict::NewContent);

    // R4 both ways: a placeholder passes while unwritten and fails the moment
    // real prose lands, naming where the content belongs.
    let empty = "# System Overview\n\n(not written yet — it belongs in the bundle.)\n";
    assert_eq!(classify("system-overview.md", empty, &bundle), Verdict::Placeholder);
    let written = "# System Overview\n\n(not written yet)\n\n## Areas\n\nReal prose.\n";
    assert_eq!(classify("system-overview.md", written, &bundle), Verdict::NewContent);

    // A named exception is exempt by NAME, and only the closed set is.
    assert_eq!(classify("reading-map.md", unmarked, &bundle), Verdict::Navigation);
    assert_eq!(classify("reading-map-2.md", unmarked, &bundle), Verdict::NewContent);
}

// ── form 2: this repo's own compatibility surface ──────────────────────────

#[test]
fn the_compatibility_surface_carries_no_new_content() {
    let root = repo_root();
    let specs = root.join("docs/specs");
    let bundle = root.join("docs/knowledge");

    // R5 — with no bundle the fence is inert and scans nothing.
    if !bundle.join("index.md").exists() {
        return;
    }
    assert!(specs.is_dir(), "docs/specs must exist to be fenced");

    let mut offenders: Vec<String> = Vec::new();
    let mut dangling: Vec<String> = Vec::new();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();

    for entry in std::fs::read_dir(&specs).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&path).unwrap();
        match classify(&name, &text, &bundle) {
            Verdict::Stub => *counts.entry("stub").or_default() += 1,
            Verdict::Navigation => *counts.entry("navigation").or_default() += 1,
            Verdict::Placeholder => *counts.entry("placeholder").or_default() += 1,
            Verdict::DanglingStub => dangling.push(name),
            Verdict::NewContent => offenders.push(name),
        }
    }

    // R3 — the reasons are printed, so a reader can see WHY a file was let
    // through without opening this file.
    let reasons: Vec<String> = NAMED_EXCEPTIONS
        .iter()
        .chain(PLACEHOLDERS.iter())
        .map(|(n, why)| format!("  {n} — {why}"))
        .collect();

    assert!(
        dangling.is_empty(),
        "pointer stub(s) whose migrated_to target does not exist — the migration target moved, \
         and the citations these stubs exist to resolve now dead-end:\n  {}",
        dangling.join("\n  ")
    );
    assert!(
        offenders.is_empty(),
        "docs/specs is a READ-ONLY compatibility surface once a bundle exists, and these files \
         are new area truth written outside the bundle's own gates:\n  {}\n\nFIX: author the \
         content as a concept under docs/knowledge/areas/<area>/ and leave a pointer stub here \
         (frontmatter `migrated_to:` plus an anchor map), so existing citations keep resolving.\n\n\
         Files that pass by name, and why:\n{}",
        offenders.join("\n  "),
        reasons.join("\n")
    );
    assert!(
        counts.get("stub").copied().unwrap_or(0) > 0,
        "the fence classified zero stubs, which means it is not reaching the surface it guards — \
         a check that cannot fail is the thing this whole guard exists to prevent"
    );
}
