// Split out of the single 3.1k-line hooks/session_preamble.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's inline module, indentation
// and all: a moved inline module is the same child of the same parent,
// so no path changes, and the fixtures inside are raw strings whose
// leading whitespace is content.

// The parent module's own `use` block travels with the tests: they reach
// for names mod.rs no longer imports now that the code using them lives
// in sibling modules.
#![allow(unused_imports)]

use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::jsjson;
use crate::state::{bypass_level, ship_visibility};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::version::BEE_VERSION;
    use super::*;

    fn write(root: &Path, rel: &str, content: &str) {
        let file = root.join(rel);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }

    /// The smallest repo the preamble can render against: an onboarding
    /// marker and an idle state record, nothing else.
    fn minimal_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(tmp.path(), ".bee/state.json", r#"{"phase":"idle"}"#);
        tmp
    }

    fn render(root: &Path) -> String {
        build_session_preamble(root, None, None)
    }

    // ── (1) the cutover contract ──────────────────────────────────────────

    #[test]
    fn a_minimal_repo_renders_and_closes_on_the_binary_spelling() {
        let tmp = minimal_repo();
        let text = render(tmp.path());
        assert!(text.starts_with(&format!("## bee v{BEE_VERSION}\n")), "{text}");
        assert!(
            text.ends_with("Everything above is already read — do not re-fetch it. Run `.bee/bin/bee status --json` yourself when you ROUTE WORK (claim, plan, change phase) or need detail this block does not carry. Never hand bee commands to the user. Route via bee-hive."),
            "closing line drifted:\n{text}"
        );
    }

    #[test]
    fn no_mjs_spelling_survives_anywhere_in_the_preamble() {
        // Every section that could carry a command is turned ON at once, so
        // the sweep covers the knowledge-context line too — the other .mjs
        // spelling inject.mjs carried.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(
            root,
            ".bee/state.json",
            r#"{"phase":"swarming","feature":"f1","mode":"standard","route":{"class":"c","lane":"standard","flags":["a"],"product_files":3}}"#,
        );
        write(
            root,
            ".bee/config.json",
            r#"{"gate_bypass":"total","ship_visibility":"draft-pr","commands":{"setup":"s","start":"r","test":"t","verify":"v"}}"#,
        );
        write(root, ".bee/HANDOFF.json", r#"{"kind":"pause","phase":"swarming"}"#);
        write(
            root,
            "docs/knowledge/areas/okf-profile/a.md",
            "---\ntype: bee.area\ntitle: A\n---\nbody\n",
        );
        write(
            root,
            "docs/knowledge/work/f1/work-item.md",
            "---\ntype: bee.work-item\nbee:\n  id: f1\n---\nbody\n",
        );
        write(
            root,
            "docs/knowledge/index.md",
            "## Critical patterns\n- [p1](areas/x/p1.md)\n\n## Other\n",
        );
        write(root, ".bee/decisions.jsonl", "{\"type\":\"decide\",\"id\":\"d1\",\"decision\":\"keep it\",\"date\":\"2026-01-01\"}\n");
        write(root, ".bee/capture-queue.jsonl", "{\"kind\":\"stub\",\"id\":\"s1\",\"at\":\"2026-01-01T00:00:00.000Z\"}\n");
        let text = render(root);
        assert!(!text.contains(".mjs"), "an .mjs spelling survived:\n{text}");
        assert!(
            text.contains("- `.bee/bin/bee knowledge context --work f1 --budget 20000` (anchor: work-item)"),
            "knowledge-context command missing or misspelled:\n{text}"
        );
    }

    // ── (2) every optional section: present when it should be, gone when not ─

    #[test]
    fn the_bypass_banner_is_omitted_when_off_and_two_lines_at_full() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("GATE BYPASS"));
        write(tmp.path(), ".bee/config.json", r#"{"gate_bypass":"full"}"#);
        let text = render(tmp.path());
        assert!(text.contains("⚡⚡ GATE BYPASS: FULL AUTOPILOT"), "{text}");
        assert!(text.contains("Only reading a secret-shaped file"), "{text}");
        assert_eq!(bypass_banner_lines("off").len(), 0);
        assert_eq!(bypass_banner_lines("").len(), 0);
        assert_eq!(bypass_banner_lines("normal").len(), 1);
        assert_eq!(bypass_banner_lines("full").len(), 2);
        assert_eq!(bypass_banner_lines("total").len(), 2);
    }

    #[test]
    fn ship_visibility_costs_nothing_until_draft_pr() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("Ship visibility"));
        write(tmp.path(), ".bee/config.json", r#"{"ship_visibility":"draft-pr"}"#);
        assert!(render(tmp.path()).contains("- Ship visibility: draft-pr — first cap opens a draft PR"));
    }

    #[test]
    fn the_route_line_appears_only_for_a_recorded_route() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("- Route:"));
        write(
            tmp.path(),
            ".bee/state.json",
            r#"{"phase":"planning","route":{"class":"feature","lane":"small","flags":["x","y"],"product_files":4}}"#,
        );
        assert!(render(tmp.path())
            .contains("- Route: class=feature | lane=small | flags=2 [x,y] | files=4"));
    }

    #[test]
    fn the_standard_commands_block_is_omitted_with_no_recorded_commands() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("### Standard commands"));
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"test":"npm test"}}"#);
        let text = render(tmp.path());
        assert!(text.contains("### Standard commands (host project)"), "{text}");
        assert!(text.contains("- test: `npm test`"), "{text}");
        assert!(text.contains("- Never build on red:"), "{text}");
        // The line points at the LOCAL command, not at a nightly CI run —
        // the instruction it replaced told agents to trust evidence that
        // could predate their change by a day.
        assert!(!text.contains("check CI instead of running anything locally"), "{text}");
        // `commands.verify` is retired: recording one buys no block at all.
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"verify":"npm test"}}"#);
        assert!(!render(tmp.path()).contains("### Standard commands"));
        // The sentinel REPLACES the red paragraph with one loud line.
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"test":"none"}}"#);
        let text = render(tmp.path());
        assert!(text.contains("- Test gates disabled by repo declaration (commands.test: none)"));
        assert!(!text.contains("- Never build on red:"));
    }

    /// doc-viewer-links (decision 4205835b): the section renders only when
    /// the key resolves, right after Standard commands, and costs nothing
    /// otherwise.
    #[test]
    fn doc_links_section_renders_only_when_the_prefix_resolves() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("### Doc links"));
        write(
            tmp.path(),
            ".bee/config.json",
            r#"{"commands":{"test":"npm test"},"doc_viewer":{"base_url":"http://10.255.255.254:7700","project":"beedashboard"}}"#,
        );
        let text = render(tmp.path());
        assert!(text.contains("### Doc links"), "{text}");
        assert!(
            text.contains("- Doc viewer: http://10.255.255.254:7700/p/beedashboard"),
            "{text}"
        );
        // Right after Standard commands, never appended at the end — the
        // closing trailer's bytes are pinned elsewhere (`ends_with`).
        let commands_at = text.find("### Standard commands").expect("commands block present");
        let doc_links_at = text.find("### Doc links").expect("doc links present");
        assert!(doc_links_at > commands_at, "Doc links must follow Standard commands:\n{text}");
        // A half-set key stays silent in the preamble too (the warning is
        // stderr-only, covered on the reader itself in state.rs).
        write(tmp.path(), ".bee/config.json", r#"{"doc_viewer":{"base_url":"http://host:7700"}}"#);
        assert!(!render(tmp.path()).contains("### Doc links"));
    }

    /// D1 (kf-1): resolve_anchor is the ONE gate, so a feature anchored by
    /// docs/history or its scribing stamp gets the exact same invitation a
    /// work item already got — the anchor kind is named so the reader knows
    /// what the manifest was ranked against — and a feature with no anchor
    /// at all gets silence, never the retired "author a work-item file"
    /// advice (D5 made that file optional; the line contradicted it).
    #[test]
    fn the_knowledge_context_bridge_invites_every_anchor_kind_and_stays_silent_with_none() {
        let tmp = minimal_repo();
        write(tmp.path(), "docs/knowledge/areas/a/c.md", "---\ntype: bee.area\n---\nx\n");
        // idle: nothing at all, even with a bundle present.
        assert!(!render(tmp.path()).contains("### Knowledge context"));

        // An active feature with NO anchor at all: silence, no advice line.
        write(tmp.path(), ".bee/state.json", r#"{"phase":"swarming","feature":"f9"}"#);
        let text = render(tmp.path());
        assert!(!text.contains("### Knowledge context"), "{text}");
        assert!(!text.contains("work-item.md"), "{text}");
        assert!(!text.contains("No knowledge work item"), "{text}");

        // Anchored ONLY by docs/history/<slug>/CONTEXT.md -> invited, named "history".
        write(tmp.path(), "docs/history/f9/CONTEXT.md", "# f9\nsome context\n");
        let text = render(tmp.path());
        assert!(text.contains("### Knowledge context — load it before code"), "{text}");
        assert!(
            text.contains("- `.bee/bin/bee knowledge context --work f9 --budget 20000` (anchor: history)"),
            "{text}"
        );

        // Anchored ONLY by the scribing ledger stamp -> invited, named "ledger".
        std::fs::remove_file(tmp.path().join("docs/history/f9/CONTEXT.md")).unwrap();
        write(
            tmp.path(),
            ".bee/logs/scribing-runs.jsonl",
            "{\"ts\":\"2026-08-05T07:00:30.067Z\",\"feature\":\"f9\",\"areas\":[\"a\"]}\n",
        );
        let text = render(tmp.path());
        assert!(
            text.contains("- `.bee/bin/bee knowledge context --work f9 --budget 20000` (anchor: ledger)"),
            "{text}"
        );

        // With a work item it keeps the invitation, named "work-item", budget by mode.
        write(
            tmp.path(),
            "docs/knowledge/work/f9/work-item.md",
            "---\ntype: bee.work-item\nbee:\n  id: f9\n---\nx\n",
        );
        write(tmp.path(), ".bee/state.json", r#"{"phase":"swarming","feature":"f9","mode":"tiny"}"#);
        let text = render(tmp.path());
        assert!(
            text.contains("- `.bee/bin/bee knowledge context --work f9 --budget 8000` (anchor: work-item)"),
            "{text}"
        );
    }

    #[test]
    fn scribing_debt_capture_queue_and_scarcity_each_omit_when_empty() {
        let tmp = minimal_repo();
        let text = render(tmp.path());
        for absent in [
            "### Scribing debt:",
            "### Orphaned scribing debt:",
            "### Capture queue:",
            "### Ceiling-model scarcity:",
            "### Critical patterns (digest)",
            "### Recent decisions",
        ] {
            assert!(!text.contains(absent), "{absent} leaked into an empty repo:\n{text}");
        }

        let root = tmp.path();
        write(root, ".bee/state.json", r#"{"phase":"swarming","feature":"f1"}"#);
        write(
            root,
            ".bee/cells/c1.json",
            r#"{"id":"c1","feature":"f1","status":"capped","tier":"ceiling","trace":{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}"#,
        );
        write(
            root,
            ".bee/cells/c2.json",
            r#"{"id":"c2","feature":"f1","status":"capped","tier":"ceiling","trace":{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}"#,
        );
        write(
            root,
            ".bee/cells/c3.json",
            r#"{"id":"c3","feature":"f1","status":"open","tier":"extraction"}"#,
        );
        write(root, ".bee/capture-queue.jsonl", "{\"kind\":\"stub\",\"id\":\"s1\",\"at\":\"2026-01-01T00:00:00.000Z\"}\n");
        write(
            root,
            ".bee/decisions.jsonl",
            "{\"type\":\"decide\",\"id\":\"d1\",\"decision\":\"a\",\"date\":\"2026-01-01\"}\n",
        );
        write(root, "docs/history/learnings/critical-patterns.md", "<!-- note -->\n- pattern one\n");
        let text = render(root);
        assert!(text.contains("### Scribing debt: 2 behavior_change cell(s) uncaptured"), "{text}");
        assert!(text.contains("- c1, c2 capped since the last scribing run"), "{text}");
        assert!(text.contains("settled behavior belongs in docs/specs/."), "{text}");
        assert!(
            text.contains("### Orphaned scribing debt: 2 cell(s) across 1 feature(s)"),
            "{text}"
        );
        assert!(text.contains("- Heaviest: f1 (2 cell(s))."), "{text}");
        assert!(text.contains("Capped with no scribing sync"), "{text}");
        assert!(text.contains("### Capture queue: 1 stub(s) pending flush"), "{text}");
        assert!(text.contains("### Ceiling-model scarcity: 67% of tiered cells on ceiling"), "{text}");
        assert!(text.contains("- 2/3 cells tiered ceiling (> 40%)"), "{text}");
        assert!(text.contains("### Critical patterns (digest)\n- pattern one"), "{text}");
        assert!(text.contains("### Recent decisions\n- «a» (2026-01-01)"), "{text}");
    }

    /// THE BUDGET LAW. The preamble is injected into every session, so a
    /// section that grows with the store spends the reader's context on data
    /// it cannot act on. Measured before the caps, on this repo: 11,390 bytes,
    /// 3,169 of them (28%) one line naming 176 uncaptured cell ids.
    ///
    /// This builds a store far worse than the real one — 400 orphaned cells
    /// across 200 features, 60 capped cells owed capture, 40 patterns, three
    /// paragraph-length decisions, a 2 KB handoff — and asserts the rendered
    /// block still fits. A cap that only holds for today's data is not a cap.
    #[test]
    fn the_preamble_stays_inside_its_budget_however_big_the_store_gets() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(root, ".bee/state.json", r#"{"phase":"swarming","feature":"f0"}"#);
        // Every optional section ON at once, with the longest real shapes:
        // the two-paragraph bypass banner and this repo's own PATH-prefixed
        // command strings. A budget proved only on a quiet repo is not one.
        write(
            root,
            ".bee/config.json",
            r#"{"gate_bypass":"total","ship_visibility":"draft-pr","commands":{"test":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml"},"doc_viewer":{"base_url":"http://10.255.255.254:7700","project":"beedashboard"}}"#,
        );
        write(
            root,
            ".bee/HANDOFF.json",
            &format!(
                r#"{{"kind":"pause","phase":"swarming","feature":"f0","mode":"standard","next_action":"{}"}}"#,
                "REDRAFT the context document and re-run the gate ".repeat(40)
            ),
        );
        for f in 0..200 {
            for c in 0..2 {
                write(
                    root,
                    &format!(".bee/cells/orph-{f}-{c}.json"),
                    &format!(
                        r#"{{"id":"orphan-cell-{f}-{c}","feature":"feature-with-a-long-name-{f}","status":"capped","tier":"generation","trace":{{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}}}"#
                    ),
                );
            }
        }
        for c in 0..60 {
            write(
                root,
                &format!(".bee/cells/f0-{c}.json"),
                &format!(
                    r#"{{"id":"f0-cell-{c}","feature":"f0","status":"capped","tier":"ceiling","trace":{{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}}}"#
                ),
            );
        }
        let patterns: String = (0..40)
            .map(|i| {
                format!(
                    "- [A long pattern title number {i} that states the whole lesson in one line](docs/knowledge/patterns/2026-pattern-{i}.md) — and then a gloss that restates the same lesson at length, again\n"
                )
            })
            .collect();
        write(
            root,
            "docs/knowledge/index.md",
            &format!("{CRITICAL_PATTERNS_HEADING}\n{patterns}\n## Next\n"),
        );
        write(root, "docs/knowledge/areas/a/c.md", "---\ntype: bee.area\n---\nx\n");
        let decisions: String = (0..3)
            .map(|i| {
                format!(
                    "{{\"type\":\"decide\",\"id\":\"d{i}\",\"decision\":\"{}\",\"date\":\"2026-01-0{}\"}}\n",
                    "a decision recorded at paragraph length ".repeat(12),
                    i + 1
                )
            })
            .collect();
        write(root, ".bee/decisions.jsonl", &decisions);
        write(
            root,
            ".bee/capture-queue.jsonl",
            "{\"kind\":\"stub\",\"id\":\"s1\",\"at\":\"2026-01-01T00:00:00.000Z\"}\n",
        );

        let text = render(root);
        assert!(
            text.len() <= PREAMBLE_BUDGET_BYTES,
            "the session preamble is {} bytes, over the {PREAMBLE_BUDGET_BYTES}-byte budget. \
             A section is growing with the store instead of with what the reader can act on — \
             cap the list and keep the count.\n\n{text}",
            text.len()
        );
        // …and it is still SAYING the things that matter, not just short.
        // (460 / 201: f0's own 60 capped cells are orphaned too.)
        assert!(text.contains("### Orphaned scribing debt: 460 cell(s) across 201 feature(s)"), "{text}");
        assert!(text.contains("+198 more feature(s)"), "{text}");
        assert!(text.contains("### Scribing debt: 60 behavior_change cell(s) uncaptured"), "{text}");
        assert!(text.contains("+52 more capped since"), "{text}");
        assert!(text.contains("### HANDOFF present"), "{text}");
        assert!(text.contains("[…]"), "a clamped field must show that it was clamped:\n{text}");
        // doc-viewer-links: measured inside the ceiling, not beside it.
        assert!(
            text.contains("### Doc links\n- Doc viewer: http://10.255.255.254:7700/p/beedashboard"),
            "{text}"
        );
    }

    #[test]
    fn a_bundle_pattern_row_keeps_its_title_and_drops_the_restating_gloss() {
        let row = "- [A test selector that matches nothing reports green](docs/knowledge/patterns/x.md) — A test selector that matches nothing reports green";
        assert_eq!(
            strip_row_gloss(row),
            "- [A test selector that matches nothing reports green](docs/knowledge/patterns/x.md)"
        );
        // An em dash INSIDE the title is not a gloss separator.
        let titled = "- [Fix the law — not the line](docs/knowledge/patterns/y.md)";
        assert_eq!(strip_row_gloss(titled), titled);
    }

    #[test]
    fn clamping_cuts_on_a_character_boundary_and_marks_the_cut() {
        assert_eq!(clamp_chars("short", 10), "short");
        assert_eq!(clamp_chars("abcdefghij", 10), "abcdefghij");
        assert_eq!(clamp_chars("abcdefghijk", 10), "abcdefghij […]");
        // Multi-byte input must not panic or split a code point.
        let vi = "một quyết định đã được ghi lại rất dài dòng";
        let cut = clamp_chars(vi, 10);
        assert!(cut.starts_with("một quyết"), "{cut}");
        assert!(cut.ends_with("[…]"));
    }

    #[test]
    fn the_project_map_switches_on_the_one_bundle_predicate() {
        let tmp = minimal_repo();
        // No maps at all -> the missing-map warning.
        assert!(render(tmp.path()).contains("- Project map missing (Q1/Q2 unanswerable from repo)"));
        write(tmp.path(), "docs/specs/system-overview.md", "x\n");
        write(tmp.path(), "docs/specs/area-a.md", "x\n");
        let text = render(tmp.path());
        assert!(text.contains("- System overview: docs/specs/system-overview.md"), "{text}");
        assert!(text.contains("- Specced areas: 1 (docs/specs/ — read the spec before the code)"), "{text}");
        // A real bundle flips both the map and the scribing target.
        write(tmp.path(), "docs/knowledge/areas/a/c.md", "---\ntype: bee.area\n---\nx\n");
        let text = render(tmp.path());
        assert!(text.contains("- Knowledge bundle: docs/knowledge/"), "{text}");
        assert!(text.contains("- Bundle holds: 1 area(s), 1 concept(s)"), "{text}");
        assert!(!text.contains("- Specced areas:"), "{text}");
        // A directory with no parsing concept is NOT a bundle (G8).
        let bare = minimal_repo();
        std::fs::create_dir_all(bare.path().join("docs/knowledge")).unwrap();
        write(bare.path(), "docs/knowledge/.gitkeep", "");
        assert!(!bundle_mode(bare.path()), "an empty directory is not a bundle");
    }

    #[test]
    fn the_bundle_digest_counts_reverses_and_rewrites_its_links() {
        let tmp = minimal_repo();
        write(tmp.path(), "docs/knowledge/areas/a/c.md", "---\ntype: bee.pattern\n---\nx\n");
        let mut index = String::from("# Index\n\n## Critical patterns\n");
        for n in 1..=12 {
            index.push_str(&format!("- [p{n}](areas/a/p{n}.md)\n"));
        }
        index.push_str("- [ext](https://example.com/x)\n\n## Other\n- ignored\n");
        write(tmp.path(), "docs/knowledge/index.md", &index);
        let text = render(tmp.path());
        // 3, not 9: the digest is a REMINDER that the bundle has patterns and
        // where they live, and it is re-injected every session. The full list
        // is one file read away.
        assert!(
            text.contains(
                "- 13 critical pattern(s) in the bundle — recency fallback (no feature bound), the 3 most recent below"
            ),
            "{text}"
        );
        // Newest-first, and bundle-relative links rewritten; absolute/http untouched.
        assert!(text.contains("- [ext](https://example.com/x)"), "{text}");
        assert!(text.contains("- [p12](docs/knowledge/areas/a/p12.md)"), "{text}");
        assert!(!text.contains("- [p4]"), "only the 3 most recent rows ride the digest:\n{text}");
        assert!(!text.contains("- [p10]"), "only the 3 most recent rows ride the digest:\n{text}");
    }

    /// D3: the digest ranks by relevance to the bound feature's anchor
    /// instead of recency — and the header always names which mode produced
    /// the rows, so a stale or unbound repo degrades visibly, never silently.
    #[test]
    fn the_bundle_digest_ranks_by_relevance_and_the_rows_change_with_the_bound_feature() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(
            root,
            "docs/knowledge/patterns/p-auth.md",
            "---\ntype: bee.pattern\ntitle: Auth pattern\n---\nAuthentication login session token flow.\n",
        );
        write(
            root,
            "docs/knowledge/patterns/p-billing.md",
            "---\ntype: bee.pattern\ntitle: Billing pattern\n---\nBilling invoice payment subscription charge.\n",
        );
        write(
            root,
            "docs/knowledge/index.md",
            "# Index\n\n## Critical patterns\n- [Auth](patterns/p-auth.md)\n- [Billing](patterns/p-billing.md)\n\n## Next\n",
        );
        write(root, "docs/history/f1/CONTEXT.md", "# f1\nAuthentication login session flow work.\n");
        write(root, "docs/history/f2/CONTEXT.md", "# f2\nBilling invoice payment subscription work.\n");

        let auth = bundle_critical_patterns_digest(root, 2, Some("f1")).unwrap();
        assert!(auth[0].contains("ranked by relevance to \"f1\""), "{auth:?}");
        assert!(auth[1].contains("p-auth.md"), "{auth:?}");

        let billing = bundle_critical_patterns_digest(root, 2, Some("f2")).unwrap();
        assert!(billing[0].contains("ranked by relevance to \"f2\""), "{billing:?}");
        assert!(billing[1].contains("p-billing.md"), "{billing:?}");
        assert_ne!(auth[1], billing[1], "the ranked row must change when the bound feature changes");

        // No feature bound -> recency fallback, and the header says so.
        let none_bound = bundle_critical_patterns_digest(root, 2, None).unwrap();
        assert!(none_bound[0].contains("recency fallback (no feature bound)"), "{none_bound:?}");

        // A feature with no docs/history/<slug>/ anchor at all -> falls back too.
        let no_anchor = bundle_critical_patterns_digest(root, 2, Some("ghost")).unwrap();
        assert!(no_anchor[0].contains("recency fallback (no anchor for \"ghost\")"), "{no_anchor:?}");
    }

    #[test]
    fn a_critical_row_missing_from_disk_is_dropped_and_counted_in_the_ranked_header() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(
            root,
            "docs/knowledge/patterns/p-real.md",
            "---\ntype: bee.pattern\n---\nAuthentication login flow.\n",
        );
        write(
            root,
            "docs/knowledge/index.md",
            "# Index\n\n## Critical patterns\n- [Real](patterns/p-real.md)\n- [Gone](patterns/p-gone.md)\n\n## Next\n",
        );
        write(root, "docs/history/f1/CONTEXT.md", "# f1\nAuthentication login flow work.\n");

        let digest = bundle_critical_patterns_digest(root, 4, Some("f1")).unwrap();
        assert!(digest[0].contains("1 row(s) dropped: target file missing"), "{digest:?}");
        assert!(digest.iter().any(|l| l.contains("p-real.md")), "{digest:?}");
        assert!(!digest.iter().any(|l| l.contains("p-gone.md")), "{digest:?}");
    }

    /// The cost evidence the cell demands: with 3 rows to rank and 200 OTHER
    /// bundle concepts sitting right next to them (simulating the rest of a
    /// real bundle collect_concepts would walk), the ranker opens exactly its
    /// 3 candidates — never the 203.
    #[test]
    fn the_ranker_opens_only_the_critical_rows_it_scores_never_the_whole_bundle() {
        let tmp = minimal_repo();
        let root = tmp.path();
        for i in 0..3 {
            write(
                root,
                &format!("docs/knowledge/patterns/p{i}.md"),
                &format!("---\ntype: bee.pattern\n---\nauthentication login body {i}\n"),
            );
        }
        for i in 0..200 {
            write(root, &format!("docs/knowledge/other/o{i}.md"), "---\ntype: bee.pattern\n---\nnoise\n");
        }
        write(root, "docs/history/f1/CONTEXT.md", "# f1\nauthentication login body\n");
        let rows: Vec<String> = (0..3).map(|i| format!("- [p{i}](patterns/p{i}.md)")).collect();

        let (top, dropped, opened) = rank_critical_rows(root, &rows, "f1", 3).unwrap();
        assert_eq!(dropped, 0, "{top:?}");
        assert_eq!(top.len(), 3, "{top:?}");
        assert_eq!(
            opened, 3,
            "the ranker must open exactly its 3 candidates, never the 203-file bundle"
        );
    }

    // ── (3) fail-open on a corrupt store ──────────────────────────────────

    #[test]
    fn a_corrupt_state_file_still_renders_a_preamble() {
        let tmp = minimal_repo();
        write(tmp.path(), ".bee/state.json", "{ this is not json");
        let text = render(tmp.path());
        // defaultState() shows through, and the preamble is whole.
        assert!(text.contains("- Phase: idle | Mode: none | Feature: none"), "{text}");
        assert!(text.contains("- Gates: none pending (no active work)"), "{text}");
        assert!(text.ends_with("Route via bee-hive."), "{text}");
    }

    #[test]
    fn a_corrupt_config_handoff_and_cell_still_render_a_preamble() {
        let tmp = minimal_repo();
        write(tmp.path(), ".bee/config.json", "{nope");
        write(tmp.path(), ".bee/HANDOFF.json", "[[[");
        write(tmp.path(), ".bee/cells/c1.json", "}{");
        write(tmp.path(), ".bee/decisions.jsonl", "not json at all\n");
        let text = render(tmp.path());
        assert!(text.contains("## bee v"), "{text}");
        assert!(!text.contains("### HANDOFF present"), "a corrupt handoff reads as absent:\n{text}");
        assert!(text.ends_with("Route via bee-hive."), "{text}");
    }

    // ── (4) the handoff block's three arms ────────────────────────────────

    #[test]
    fn a_pause_handoff_renders_the_wait_block_after_a_blank_separator() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/HANDOFF.json",
            r#"{"kind":"pause","phase":"swarming","feature":"f1","mode":"small","cells_in_flight":["c1","c2"],"next_action":"resume c1"}"#,
        );
        let text = render(tmp.path());
        assert!(
            text.contains("\n\n### HANDOFF present — present it and WAIT — never auto-resume\n"),
            "the caller owns the blank separator:\n{text}"
        );
        assert!(text.contains("- Phase: swarming | Feature: f1 | Mode: small"), "{text}");
        assert!(text.contains("- Cells in flight: c1, c2"), "{text}");
        assert!(text.contains("- Saved next action: resume c1"), "{text}");
        assert!(!text.contains("- Adoption not applied:"), "{text}");
        // A kindless record normalizes to pause, byte-identically.
        write(tmp.path(), ".bee/HANDOFF.json", r#"{"phase":"swarming"}"#);
        assert!(render(tmp.path()).contains("### HANDOFF present"));
        // And the block itself carries no leading blank of its own.
        let handoff = read_handoff(tmp.path()).unwrap();
        assert_eq!(
            handoff_block_lines(&handoff, None)[0],
            "### HANDOFF present — present it and WAIT — never auto-resume"
        );
    }

    #[test]
    fn an_adopted_planned_next_replaces_the_wait_block_with_start_now() {
        let tmp = minimal_repo();
        // adoptHandoff cleared HANDOFF.json already — the outcome is the only record.
        write(
            tmp.path(),
            ".bee/cells/c7.json",
            r#"{"id":"c7","title":"Wire the thing","lane":"small","verify":"npm test -- c7"}"#,
        );
        let outcome = HandoffOutcome {
            ok: true,
            next_cell: Some("c7".to_string()),
            ..Default::default()
        };
        let text = build_session_preamble(tmp.path(), None, Some(&outcome));
        assert!(
            text.contains(
                "\n\n### PLANNED-NEXT ADOPTED — starting now, no confirmation needed (D1)\n"
            ),
            "{text}"
        );
        assert!(text.contains("- Cell: c7 — Wire the thing"), "{text}");
        assert!(text.contains("- Lane: small"), "{text}");
        assert!(text.contains("- Verify: `npm test -- c7`"), "{text}");
        assert!(!text.contains("### HANDOFF present"), "{text}");

        // An unknown cell degrades to the unknown arms, never a failure.
        let outcome = HandoffOutcome { ok: true, ..Default::default() };
        let text = build_session_preamble(tmp.path(), None, Some(&outcome));
        assert!(text.contains("- Cell: unknown"), "{text}");
        assert!(text.contains("- Lane: unknown"), "{text}");
        assert!(!text.contains("- Verify:"), "{text}");
    }

    #[test]
    fn a_refused_adoption_waits_and_names_the_reason() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/HANDOFF.json",
            r#"{"kind":"planned-next","phase":"swarming","feature":"f1","mode":"small"}"#,
        );
        let outcome = HandoffOutcome {
            ok: false,
            code: Some("WRONG_SOURCE".to_string()),
            reason: Some("resumed sessions never adopt".to_string()),
            next_cell: None,
        };
        let text = build_session_preamble(tmp.path(), None, Some(&outcome));
        assert!(text.contains("### HANDOFF present — present it and WAIT"), "{text}");
        assert!(text.contains("- Adoption not applied: resumed sessions never adopt"), "{text}");
        assert!(!text.contains("PLANNED-NEXT ADOPTED"), "{text}");

        // reason ?? code ?? 'unknown reason'.
        let handoff = read_handoff(tmp.path()).unwrap();
        let code_only = HandoffOutcome {
            ok: false,
            code: Some("WRONG_SOURCE".to_string()),
            ..Default::default()
        };
        assert!(handoff_block_lines(&handoff, Some(&code_only))
            .iter()
            .any(|l| l == "- Adoption not applied: WRONG_SOURCE"));
        let bare = HandoffOutcome { ok: false, ..Default::default() };
        assert!(handoff_block_lines(&handoff, Some(&bare))
            .iter()
            .any(|l| l == "- Adoption not applied: unknown reason"));
        // A PAUSE handoff never carries a refusal line, outcome or not.
        write(tmp.path(), ".bee/HANDOFF.json", r#"{"kind":"pause"}"#);
        let handoff = read_handoff(tmp.path()).unwrap();
        assert!(!handoff_block_lines(&handoff, Some(&code_only))
            .iter()
            .any(|l| l.starts_with("- Adoption not applied")));
    }

    // ── the shared renderers, on their own ────────────────────────────────

    #[test]
    fn onboarding_line_covers_all_three_arms() {
        assert_eq!(
            onboarding_line(None),
            "- Onboarding: MISSING — run bee-hive onboarding before anything else."
        );
        assert_eq!(
            onboarding_line(Some(&json!({"bee_version": "0.9.0"}))),
            format!("- Onboarding: installed at bee 0.9.0 but plugin is {BEE_VERSION} — re-run onboarding to refresh vendored helpers.")
        );
        assert_eq!(
            onboarding_line(Some(&json!({"bee_version": BEE_VERSION}))),
            format!("- Onboarding: ok (bee {BEE_VERSION})")
        );
        // A record with no version at all reads as ok at the plugin version.
        assert_eq!(
            onboarding_line(Some(&json!({}))),
            format!("- Onboarding: ok (bee {BEE_VERSION})")
        );
    }

    #[test]
    fn first_open_gate_skips_review_outside_a_review_session_and_terminal_records() {
        let rec = |phase: &str, gates: Value| -> JMap {
            let mut m = JMap::new();
            m.insert("phase".into(), json!(phase));
            m.insert("approved_gates".into(), gates);
            m
        };
        assert_eq!(first_open_gate(&rec("idle", json!({}))), None);
        assert_eq!(first_open_gate(&rec("compounding-complete", json!({}))), None);
        assert_eq!(first_open_gate(&rec("planning", json!({}))), Some("context"));
        assert_eq!(
            first_open_gate(&rec("planning", json!({"context": true, "shape": true, "execution": true}))),
            None,
            "review is on-demand — never pending outside a review session"
        );
        assert_eq!(
            first_open_gate(&rec("reviewing", json!({"context": true, "shape": true, "execution": true}))),
            Some("review")
        );
        // gatesLine follows the same rule.
        assert_eq!(gates_line(&rec("idle", json!({}))), "none pending (no active work)");
        assert_eq!(
            gates_line(&rec("planning", json!({"context": true}))),
            "context: approved | shape: pending | execution: pending"
        );
    }

    #[test]
    fn a_lane_bound_session_reports_the_other_active_lanes() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(root, ".bee/sessions/s1.json", r#"{"id":"s1","lane":"f1"}"#);
        write(root, ".bee/lanes/f1.json", r#"{"feature":"f1","phase":"swarming","mode":"small"}"#);
        write(root, ".bee/lanes/f2.json", r#"{"feature":"f2","phase":"planning"}"#);
        write(root, ".bee/lanes/f3.json", r#"{"feature":"f3","phase":"idle"}"#);
        let text = build_session_preamble(root, Some("s1"), None);
        assert!(text.contains("- Phase: swarming | Mode: small | Feature: f1"), "{text}");
        assert!(text.contains("- 1 other active lane(s): f2"), "{text}");
        // An unresolvable binding falls back to the DEFAULT record, silently.
        write(root, ".bee/sessions/s2.json", r#"{"id":"s2","lane":"nope"}"#);
        let text = build_session_preamble(root, Some("s2"), None);
        assert!(text.contains("- Phase: idle | Mode: none | Feature: none"), "{text}");
        assert!(!text.contains("other active lane(s)"), "{text}");
    }

    /// The session id `session_binding`'s own resolver will actually look
    /// for: its env chain (BEE_SESSION_ID / CLAUDE_CODE_SESSION_ID) outranks
    /// single-live-session adoption, and a real test runner may already
    /// export one of those — so a hard-coded fixture id would be invisible
    /// to the code under test. Ask the resolver instead (same pattern
    /// `verbs/state_group/tests.rs::fixture_session_id` already uses).
    fn fixture_session_id(root: &Path) -> String {
        crate::verbs::state_group::resolve_session_id_no_flag(root)
            .ok()
            .flatten()
            .unwrap_or_else(|| "sess-1".to_string())
    }

    /// D2 (kf-1): the knowledge bridge and the critical-pattern digest must
    /// read the SESSION's active feature — the bound lane when the session
    /// has one — even when the SessionStart hook's own `session_id` (here:
    /// None) never named the session record that carries the binding. The
    /// fallback resolves through the exact chain `state gate`/`state set`/
    /// `state route` already read through (`session_binding`), the same
    /// shape the measured bug had: the default record still named a feature
    /// closed hours earlier while the calling session was actively lane-bound.
    #[test]
    fn an_unbound_hook_session_id_still_finds_the_process_bound_lane_over_the_default_record() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(root, ".bee/state.json", r#"{"phase":"compounding-complete","feature":"stale-closed"}"#);
        let sid = fixture_session_id(root);
        write(
            root,
            &format!(".bee/sessions/{sid}.json"),
            &format!(r#"{{"id":"{sid}","lane":"f-active"}}"#),
        );
        write(
            root,
            ".bee/lanes/f-active.json",
            r#"{"feature":"f-active","phase":"swarming","mode":"standard"}"#,
        );
        write(root, "docs/history/f-active/CONTEXT.md", "# f-active\nactive work\n");
        write(
            root,
            "docs/knowledge/index.md",
            "## Critical patterns\n- [p1](areas/x/p1.md)\n\n## Next\n",
        );
        write(root, "docs/knowledge/areas/x/p1.md", "---\ntype: bee.pattern\n---\nsome pattern text\n");

        // No session_id given to the hook (the common cold-start shape) —
        // resolution must still land on the process-bound lane, not the
        // default record's stale feature.
        // The header line still names the DEFAULT record's own phase/feature
        // (untouched by D2 — only the knowledge bridge and the digest below
        // read the session's active feature).
        let text = build_session_preamble(root, None, None);
        assert!(text.contains("Feature: stale-closed"), "{text}");
        let knowledge_at = text.find("### Knowledge context").expect("knowledge block present:\n{text}");
        assert!(
            text[knowledge_at..].contains(
                "- `.bee/bin/bee knowledge context --work f-active --budget 20000` (anchor: history)"
            ),
            "{text}"
        );
        assert!(!text[knowledge_at..].contains("stale-closed"), "{text}");
        let digest_at = text.find("### Critical patterns (digest)").expect("digest present:\n{text}");
        assert!(text[digest_at..].contains("\"f-active\""), "{text}");
        assert!(!text[digest_at..].contains("stale-closed"), "{text}");
    }
