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

    /// expertise-principles D2: the routed principles ride BESIDE the Route
    /// line, through the same shared reader `bee orient` calls. Three silences
    /// are pinned here because each one is a real repo: no recorded route, no
    /// vendored index (every host repo today), and a class no row claims.
    #[test]
    fn routed_principles_ride_beside_the_route_line() {
        let tmp = minimal_repo();
        write(tmp.path(), crate::principles::PRINCIPLE_INDEX, crate::principles::TEST_INDEX);
        // No recorded route: the index is on disk and still nothing renders.
        assert!(!render(tmp.path()).contains("Principles"));

        let route = |class: &str| {
            format!(r#"{{"phase":"planning","route":{{"class":"{class}","lane":"small","flags":[],"product_files":0}}}}"#)
        };
        write(tmp.path(), ".bee/state.json", &route("feature"));
        let text = render(tmp.path());
        assert!(text.contains("- Route: class=feature"), "{text}");
        assert!(
            text.contains(
                "- Principles (class=feature) — name each one you apply and the decision it changed:"
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "  - `principle-red-before-green` — watch it fail for the reported reason before you fix it"
            ),
            "{text}"
        );
        // The Route line is untouched by any of it — the block is added beside
        // it, never in place of it.
        assert!(text.contains("- Route: class=feature | lane=small | flags=0 [] | files=0"));

        // A class no row claims keeps the Route line and drops the block.
        write(tmp.path(), ".bee/state.json", &route("docs"));
        let text = render(tmp.path());
        assert!(text.contains("- Route: class=docs"), "{text}");
        assert!(!text.contains("Principles"), "{text}");

        // No index on disk: a matching class still renders nothing.
        std::fs::remove_file(tmp.path().join(crate::principles::PRINCIPLE_INDEX)).unwrap();
        write(tmp.path(), ".bee/state.json", &route("feature"));
        assert!(!render(tmp.path()).contains("Principles"));
    }

    #[test]
    fn the_standard_commands_block_is_omitted_with_no_recorded_commands() {
        let tmp = minimal_repo();
        assert!(!render(tmp.path()).contains("### Standard commands"));
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"test":"npm test"}}"#);
        let text = render(tmp.path());
        assert!(text.contains("### Standard commands (host project)"), "{text}");
        assert!(text.contains("- test: `npm test`"), "{text}");
        assert!(
            text.contains(
                "- Proof-per-change-type: pick the proof your change needs — related tests for code, parity/pointer checks for docs — and record it in the cap proof line. CI runs the same command on every push and PR."
            ),
            "{text}"
        );
        // The mandatory pre-claim full-suite red check (dropped by D3,
        // 58ec9664) never renders again.
        assert!(!text.contains("- Never build on red:"), "{text}");
        // `commands.verify` is retired: recording one buys no block at all.
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"verify":"npm test"}}"#);
        assert!(!render(tmp.path()).contains("### Standard commands"));
        // The sentinel REPLACES the red paragraph with one loud line — and
        // (test-doctrine D7/D8, td-1) that line still points to a required
        // proof line, never a bare diff-backed cap.
        write(tmp.path(), ".bee/config.json", r#"{"commands":{"test":"none"}}"#);
        let text = render(tmp.path());
        assert!(
            text.contains(
                "- Test gates disabled by repo declaration (commands.test: none) — every cap still records a proof line (command segment `none`, reason naming the parity/docs proof used, e.g. `none — green:static — docs pointer check`); recording a real commands.test re-enables CI's full-run net."
            ),
            "{text}"
        );
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

    // ── csc-1: the command surface ─────────────────────────────────────────

    /// Known fixtures from the embedded registry: the `cells` group carries
    /// its verbs (claim among them), and no flag token appears anywhere —
    /// flags live behind `bee <command> --help` since preamble-surface-slim.
    #[test]
    fn command_surface_lines_render_grouped_names_without_flags() {
        let lines = command_surface_lines();
        let cells = lines
            .iter()
            .find(|l| l.starts_with("cells: "))
            .unwrap_or_else(|| panic!("cells group missing: {lines:?}"));
        assert!(cells.contains("claim"), "{cells}");
        assert!(cells.contains("finish"), "{cells}");
        let state = lines
            .iter()
            .find(|l| l.starts_with("state: "))
            .unwrap_or_else(|| panic!("state group missing: {lines:?}"));
        // A deeper dotted name joins with a space and stays one verb.
        assert!(state.contains("plan-rev bump"), "{state}");
        assert!(!lines.iter().any(|l| l.contains("--")), "no flag tokens allowed: {lines:?}");
    }

    /// A single-segment command renders its bare name — never a dangling
    /// colon.
    #[test]
    fn a_flagless_command_renders_without_a_trailing_colon() {
        let lines = command_surface_lines();
        assert!(lines.contains(&"orient".to_string()), "{lines:?}");
        assert!(!lines.iter().any(|l| l == "orient:"), "{lines:?}");
    }

    /// `--json` appears nowhere — the header note names `json` once and
    /// points at per-command help.
    #[test]
    fn the_json_flag_appears_nowhere_inside_the_command_surface_section() {
        let section = command_surface_section().join("\n");
        assert!(!section.contains("--json"), "{section}");
        assert!(section.contains("### Command surface"), "{section}");
        assert!(section.contains("`bee <verb> --help`"), "{section}");
        assert!(section.contains('`'), "the header note should mention `json` once:\n{section}");
    }

    /// Every registry command appears exactly once across the grouped
    /// lines: each group line carries exactly its members' verbs, and the
    /// group count matches the distinct first segments, path-sorted for
    /// byte-stability across regenerations that reorder the source array.
    #[test]
    fn every_registry_command_appears_once_path_sorted() {
        let entries = crate::catalog::entries();
        let lines = command_surface_lines();
        let mut sorted_names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        sorted_names.sort();
        let total_verbs: usize = lines
            .iter()
            .map(|l| match l.split_once(": ") {
                Some((_, verbs)) => verbs.split(", ").count(),
                None => 1,
            })
            .sum();
        assert_eq!(total_verbs, entries.len(), "{lines:?}");
        // A name that is both bare and a group renders both spellings.
        assert!(lines.contains(&"doctor".to_string()), "{lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("doctor: ")), "{lines:?}");
        let mut prior = "";
        for line in &lines {
            let head = line.split_once(':').map_or(line.as_str(), |(h, _)| h);
            assert!(head >= prior, "not sorted: {prior} then {head}");
            prior = head;
        }
    }

    /// The whole section stays under the character budget, so a future verb
    /// explosion is caught here rather than silently paid for every session.
    #[test]
    fn the_command_surface_section_stays_inside_its_character_budget() {
        let section = command_surface_section().join("\n");
        assert!(
            section.chars().count() <= COMMAND_SURFACE_BUDGET_CHARS,
            "the command surface section is {} characters, over the {COMMAND_SURFACE_BUDGET_CHARS}-character budget:\n{section}",
            section.chars().count()
        );
    }

    /// Placement: after `### Standard commands`, before `### Doc links`, and
    /// the preamble still closes on the same trailer bytes.
    #[test]
    fn the_command_surface_section_sits_between_standard_commands_and_doc_links() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/config.json",
            r#"{"commands":{"test":"npm test"},"doc_viewer":{"base_url":"http://host:7700","project":"p"}}"#,
        );
        let text = render(tmp.path());
        assert!(text.contains("### Command surface"), "{text}");
        let commands_at = text.find("### Standard commands").expect("standard commands present");
        let surface_at = text.find("### Command surface").expect("command surface present");
        let doc_links_at = text.find("### Doc links").expect("doc links present");
        assert!(surface_at > commands_at, "Command surface must follow Standard commands:\n{text}");
        assert!(doc_links_at > surface_at, "Doc links must follow Command surface:\n{text}");
        assert!(
            text.ends_with("Everything above is already read — do not re-fetch it. Run `.bee/bin/bee status --json` yourself when you ROUTE WORK (claim, plan, change phase) or need detail this block does not carry. Never hand bee commands to the user. Route via bee-hive."),
            "closing line drifted:\n{text}"
        );
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
            "### Unapplied promote proposal(s):",
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
        // D6: same two cells, same 2/3, same 67% — the fixtures still carry
        // the LEGACY `tier: "ceiling"` spelling and it is still counted. What
        // changed is the subject: the escalation flag, against the feature's
        // cells rather than against "cells that recorded a tier".
        assert!(
            text.contains("### Ceiling-model scarcity: 67% of this feature's cells escalated"),
            "{text}"
        );
        assert!(text.contains("- 2/3 cells escalated onto the session model (> 40%)"), "{text}");
        assert!(text.contains("### Critical patterns (digest)\n- pattern one"), "{text}");
        assert!(text.contains("### Recent decisions\n- «a» (2026-01-01)"), "{text}");
    }

    /// model-role-split D6 (store 97ce5225), closing the gap mrs-14 named.
    ///
    /// A cell marked the NEW way — `escalate: true`, no `tier` key at all —
    /// was INVISIBLE to this advice line: the enforcing door at
    /// `handlers_close.rs` already read the flag, while the preamble still
    /// matched a tier VALUE. On a fully migrated store every cell looks like
    /// this, so the section would have vanished and read as all-clear.
    ///
    /// The denominator moves with it. It was "cells that recorded a tier",
    /// which is 0 once `role` is the required field, and a zero denominator
    /// is a warning that can never fire again.
    #[test]
    fn ceiling_scarcity_sees_a_cell_marked_with_the_flag_and_no_tier() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(root, ".bee/state.json", r#"{"phase":"swarming","feature":"f1"}"#);
        for id in ["e1", "e2"] {
            write(
                root,
                &format!(".bee/cells/{id}.json"),
                &format!(
                    r#"{{"id":"{id}","feature":"f1","status":"open","lane":"standard","role":"code","escalate":true}}"#
                ),
            );
        }
        write(
            root,
            ".bee/cells/p1.json",
            r#"{"id":"p1","feature":"f1","status":"open","lane":"standard","role":"read"}"#,
        );
        let text = render(root);
        assert!(
            text.contains("### Ceiling-model scarcity: 67% of this feature's cells escalated"),
            "an escalate-flagged cell with no tier must still be counted:\n{text}"
        );
        assert!(text.contains("- 2/3 cells escalated onto the session model (> 40%)"), "{text}");
        assert!(
            !text.contains("re-tier"),
            "the remedy may not name a retired tier value:\n{text}"
        );
    }

    /// The counter-case, and the one that keeps the line honest: three cells
    /// of the feature, one escalated. 33% is under the 40% bar, so nothing is
    /// said. A line that fired here would be noise; a line that never fires
    /// at all is the defect above.
    #[test]
    fn ceiling_scarcity_stays_silent_when_the_escalated_share_is_under_the_bar() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(root, ".bee/state.json", r#"{"phase":"swarming","feature":"f1"}"#);
        write(
            root,
            ".bee/cells/e1.json",
            r#"{"id":"e1","feature":"f1","status":"open","lane":"standard","role":"code","escalate":true}"#,
        );
        for id in ["p1", "p2"] {
            write(
                root,
                &format!(".bee/cells/{id}.json"),
                &format!(
                    r#"{{"id":"{id}","feature":"f1","status":"open","lane":"standard","role":"code"}}"#
                ),
            );
        }
        let text = render(root);
        assert!(!text.contains("### Ceiling-model scarcity"), "{text}");
    }

    /// trun-9 rework (D5), FAIL 1's proof: the first pass wired the deferred
    /// queue into `drivers/close.rs::scribing_debt` only — completing a
    /// `scribe` record cleared close's own door but left THIS preamble line
    /// (and the orphan sweep beside it) still reporting the same debt,
    /// because `hooks::session_preamble::store::scribing_debt` /
    /// `global_scribing_debt` carried a second, unreconciled copy of the
    /// scan. Both now read `state_group::scribe_queue_cells` and decide with
    /// `state_group::deferred_debt_cleared`, the same shared rule close's
    /// door uses — so completing the queue record clears the preamble too.
    #[test]
    fn completing_a_scribe_queue_record_clears_the_preamble_debt_lines() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(root, ".bee/state.json", r#"{"phase":"swarming","feature":"f1"}"#);
        write(
            root,
            ".bee/cells/c1.json",
            r#"{"id":"c1","feature":"f1","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}"#,
        );
        write(
            root,
            ".bee/cells/c2.json",
            r#"{"id":"c2","feature":"f1","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}"#,
        );

        // Before any queue record: both the active-feature line and the
        // orphan sweep report the debt — no scribing run has ever happened
        // (threshold is 0), so the legacy stamp alone leaves both cells open.
        let text = render(root);
        assert!(text.contains("### Scribing debt: 2 behavior_change cell(s) uncaptured"), "{text}");
        assert!(text.contains("### Orphaned scribing debt: 2 cell(s) across 1 feature(s)"), "{text}");

        // A `scribe` deferred-queue record naming BOTH cells, then completed
        // — the exact shape `drivers/close.rs::scribing_debt` materializes
        // and `deferred-queue complete` folds, written directly here so this
        // test proves the READ side, not the write side (covered separately
        // in drivers/close.rs's own tests).
        write(
            root,
            ".bee/deferred-queue.jsonl",
            concat!(
                "{\"ts\":\"2026-01-03T00:00:00.000Z\",\"event\":\"add\",\"id\":\"q1\",\"kind\":\"scribe\",\"feature\":\"f1\",\"cells\":[\"c1\",\"c2\"],\"areas\":[],\"files\":[],\"reason\":\"r\"}\n",
                "{\"ts\":\"2026-01-04T00:00:00.000Z\",\"event\":\"complete\",\"id\":\"q1\"}\n",
            ),
        );
        let text = render(root);
        assert!(!text.contains("### Scribing debt:"), "debt line survived completion:\n{text}");
        assert!(!text.contains("### Orphaned scribing debt:"), "orphan line survived completion:\n{text}");
    }

    /// Same fixture, but the queue record completes only ONE of the two
    /// debtor cells — proves the reconciliation is per-cell, not
    /// per-feature: c1's debt clears while c2's stays reported, on both the
    /// active-feature line and the orphan sweep.
    #[test]
    fn a_partially_completed_scribe_record_still_reports_the_remaining_cell() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(root, ".bee/state.json", r#"{"phase":"swarming","feature":"f1"}"#);
        write(
            root,
            ".bee/cells/c1.json",
            r#"{"id":"c1","feature":"f1","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}"#,
        );
        write(
            root,
            ".bee/cells/c2.json",
            r#"{"id":"c2","feature":"f1","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-01-02T00:00:00.000Z"}}"#,
        );
        write(
            root,
            ".bee/deferred-queue.jsonl",
            concat!(
                "{\"ts\":\"2026-01-03T00:00:00.000Z\",\"event\":\"add\",\"id\":\"q1\",\"kind\":\"scribe\",\"feature\":\"f1\",\"cells\":[\"c1\"],\"areas\":[],\"files\":[],\"reason\":\"r\"}\n",
                "{\"ts\":\"2026-01-04T00:00:00.000Z\",\"event\":\"complete\",\"id\":\"q1\"}\n",
            ),
        );
        let text = render(root);
        assert!(text.contains("### Scribing debt: 1 behavior_change cell(s) uncaptured"), "{text}");
        assert!(text.contains("- c2 capped since the last scribing run"), "{text}");
        assert!(text.contains("### Orphaned scribing debt: 1 cell(s) across 1 feature(s)"), "{text}");
        assert!(text.contains("- Heaviest: f1 (1 cell(s))."), "{text}");
    }

    /// D3 (kf-2): `bee close` writes docs/history/<feature>/promote-proposals.md
    /// on every green close and nothing read it back until this line. A
    /// proposal names its feature, its own count clause, and its path — and
    /// goes silent the moment a compounding run is recorded at or after the
    /// file's own mtime, never before.
    #[test]
    fn an_unapplied_promote_proposal_is_named_and_a_later_compounding_run_silences_it() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(
            root,
            "docs/history/f2/promote-proposals.md",
            "promote proposal for work item \"f2\" (docs/history/f2/CONTEXT.md) — 3 capped cell(s): a-1, a-2, a-3\nanchor: history — docs/history/f2/CONTEXT.md\n",
        );
        let text = render(root);
        // U4: the block is now ONE line — count + newest proposal path,
        // never a per-feature enumeration.
        assert!(
            text.contains(
                "### Unapplied promote proposal(s): 1 — newest: docs/history/f2/promote-proposals.md"
            ),
            "{text}"
        );

        // A compounding run recorded strictly BEFORE the file's own mtime
        // never clears it — the proposal still postdates the last sync.
        write(
            root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"ts\":\"2020-01-01T00:00:00.000Z\",\"feature\":\"f2\",\"areas\":[]}\n",
        );
        assert!(render(root).contains("### Unapplied promote proposal(s): 1"), "{}", render(root));

        // A compounding run AT OR AFTER the file's own mtime silences it.
        let mtime_ms = std::fs::metadata(root.join("docs/history/f2/promote-proposals.md"))
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;
        let ts = crate::verbs::status_full::to_iso(mtime_ms + 1000.0);
        write(
            root,
            ".bee/logs/scribing-runs.jsonl",
            &format!("{{\"ts\":\"{ts}\",\"feature\":\"f2\",\"areas\":[]}}\n"),
        );
        let text = render(root);
        assert!(!text.contains("Unapplied promote proposal"), "{text}");
    }

    /// U4 (docs/history/knowledge-usable/CONTEXT.md): two unapplied
    /// proposals still render as ONE line — count 2, and the NEWEST file by
    /// mtime, never both paths enumerated.
    #[test]
    fn two_unapplied_promote_proposals_still_render_one_line_naming_the_newest() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(
            root,
            "docs/history/f1/promote-proposals.md",
            "promote proposal for work item \"f1\" (docs/history/f1/CONTEXT.md) — 1 capped cell(s): a-1\nanchor: history — docs/history/f1/CONTEXT.md\n",
        );
        let older = filetime::FileTime::from_system_time(
            std::time::SystemTime::now() - std::time::Duration::from_secs(600),
        );
        filetime::set_file_mtime(root.join("docs/history/f1/promote-proposals.md"), older).unwrap();
        write(
            root,
            "docs/history/f2/promote-proposals.md",
            "promote proposal for work item \"f2\" (docs/history/f2/CONTEXT.md) — 3 capped cell(s): a-1, a-2, a-3\nanchor: history — docs/history/f2/CONTEXT.md\n",
        );
        let text = render(root);
        let line = text
            .lines()
            .find(|l| l.contains("Unapplied promote proposal"))
            .unwrap_or_default();
        assert_eq!(
            line,
            "### Unapplied promote proposal(s): 2 — newest: docs/history/f2/promote-proposals.md — review the proposal, then apply what belongs to docs/knowledge/ or record why not."
        );
        // ONE line total — no per-feature enumeration trails it.
        assert!(!text.contains("f1 (1 capped"), "{text}");
    }

    /// D4/D4a: a granted worktree `bee worktree merge` never reached and
    /// nobody pruned announces itself once the count of reclaimable ids is
    /// MORE than one — "one stale worktree is not news" (plan.md). The line
    /// names the count and the command, never the size (D4a), and this test
    /// never touches git or `.git/worktrees/` — only a grants file and two
    /// sibling directories aged past the threshold with `filetime`.
    #[test]
    fn two_or_more_reclaimable_worktrees_name_the_count_and_the_prune_command() {
        let outer = tempfile::tempdir().unwrap();
        let root = outer.path().join("main");
        write(&root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(&root, ".bee/state.json", r#"{"phase":"idle"}"#);
        write(
            &root,
            ".bee/runtime/worktree-grants.json",
            r#"{"repo--wt--a":true,"repo--wt--b":true}"#,
        );
        let old = filetime::FileTime::from_system_time(
            std::time::SystemTime::now() - std::time::Duration::from_secs(8 * 24 * 60 * 60),
        );
        for id in ["repo--wt--a", "repo--wt--b"] {
            let dir = outer.path().join(id);
            std::fs::create_dir_all(&dir).unwrap();
            filetime::set_file_mtime(&dir, old).unwrap();
        }

        let text = render(&root);
        assert!(text.contains("### Reclaimable worktree(s): 2"), "{text}");
        assert!(text.contains("`bee worktree prune --dry-run`"), "{text}");
    }

    /// The one-worktree case stays silent, and so does a worktree still
    /// inside the age threshold — both below the "more than one" and "old
    /// enough" floors this block guards.
    #[test]
    fn a_single_reclaimable_worktree_and_a_too_young_one_stay_silent() {
        let outer = tempfile::tempdir().unwrap();
        let root = outer.path().join("main");
        write(&root, ".bee/onboarding.json", &format!(r#"{{"bee_version":"{BEE_VERSION}"}}"#));
        write(&root, ".bee/state.json", r#"{"phase":"idle"}"#);

        // Only one granted id, aged well past the threshold: below the
        // "more than one" floor.
        write(&root, ".bee/runtime/worktree-grants.json", r#"{"repo--wt--a":true}"#);
        let old = filetime::FileTime::from_system_time(
            std::time::SystemTime::now() - std::time::Duration::from_secs(8 * 24 * 60 * 60),
        );
        let dir_a = outer.path().join("repo--wt--a");
        std::fs::create_dir_all(&dir_a).unwrap();
        filetime::set_file_mtime(&dir_a, old).unwrap();
        assert!(!render(&root).contains("Reclaimable worktree"), "{}", render(&root));

        // Two granted ids, but the second is freshly created — under the age
        // threshold — so the count of RECLAIMABLE ids is still one.
        write(
            &root,
            ".bee/runtime/worktree-grants.json",
            r#"{"repo--wt--a":true,"repo--wt--b":true}"#,
        );
        std::fs::create_dir_all(outer.path().join("repo--wt--b")).unwrap();
        assert!(!render(&root).contains("Reclaimable worktree"), "{}", render(&root));
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
    fn u1_the_preamble_names_the_pull_move_only_when_a_bundle_exists() {
        // No bundle: the spec-only project map never gains the pull line —
        // no-bundle repos render byte-identically to before (U1).
        let tmp = minimal_repo();
        let no_bundle = render(tmp.path());
        assert!(
            !no_bundle.contains("bee knowledge search --text"),
            "{no_bundle}"
        );

        // A real bundle: the map gains exactly one line naming the pull
        // move, and it spells the command in full.
        write(tmp.path(), "docs/knowledge/areas/a/c.md", "---\ntype: bee.area\n---\nx\n");
        let bundled = render(tmp.path());
        assert!(
            bundled.contains(
                "- Hit a symptom mid-flow (error text, odd behavior, an unfamiliar mechanism)? Pull it: `bee knowledge search --text \"<symptom>\"`."
            ),
            "{bundled}"
        );
        // Exactly one occurrence — never a second line or flag-by-flag doc.
        assert_eq!(bundled.matches("bee knowledge search --text").count(), 1, "{bundled}");
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

    /// U5: this session's own repro — a feature bound the moment
    /// `bee state bind` writes its `.bee/lanes/<feature>.json` record, well
    /// before its first scribing run and with no docs/history/<feature>/
    /// file yet, used to fall through resolve_anchor's History arm, its
    /// (pre-widening) Ledger arm, and land on the caller's recency
    /// fallback — the digest printed "recency fallback (no anchor)" for a
    /// feature that WAS bound. The ledger arm now treats a bare lane
    /// record as its own signal, so this resolves and ranks instead.
    #[test]
    fn a_bound_feature_with_only_a_lane_record_resolves_an_anchor_and_ranks() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(
            root,
            "docs/knowledge/patterns/p-auth.md",
            "---\ntype: bee.pattern\ntitle: Auth pattern\n---\nAuthentication login session token flow.\n",
        );
        write(
            root,
            "docs/knowledge/index.md",
            "# Index\n\n## Critical patterns\n- [Auth](patterns/p-auth.md)\n\n## Next\n",
        );
        // A bare lane record: bound, but no last_scribing_run yet, no
        // .bee/logs/scribing-runs.jsonl entry, no docs/history/f3/ file.
        write(
            root,
            ".bee/lanes/f3.json",
            r#"{"schema_version":"1.0","feature":"f3","mode":"small","phase":"shaping","approved_gates":{"context":false,"shape":false,"execution":false,"review":false},"summary":"","created_at":"2026-08-10T00:00:00.000Z"}"#,
        );

        let digest = bundle_critical_patterns_digest(root, 2, Some("f3")).unwrap();
        assert!(
            digest[0].contains("ranked by relevance to \"f3\""),
            "a bound feature with only a lane record must resolve an anchor and rank, not recency-fallback: {digest:?}"
        );
        assert!(
            !digest[0].contains("recency fallback"),
            "a lane record alone must be enough — no scribing-run entry required (U5): {digest:?}"
        );
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
            Some("uat"),
            "uat-gate-before-merge D1: uat becomes the next open gate once execution is approved"
        );
        assert_eq!(
            first_open_gate(&rec(
                "planning",
                json!({"context": true, "shape": true, "execution": true, "uat": true})
            )),
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
        // uat is noise before execution is approved (D1) — invisible here.
        assert_eq!(
            gates_line(&rec("planning", json!({"context": true, "shape": true, "execution": true}))),
            "context: approved | shape: approved | execution: approved | uat: pending"
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
        // The heartbeat is load-bearing, not decoration. With no session-id
        // env var exported — CI's shape, where this test used to be the only
        // red — `resolve_session_id_no_flag` falls back to "exactly one FRESH
        // session record", and a record with no `last_heartbeat` reads as
        // stale. The fixture then resolved to nothing, the knowledge bridge
        // never saw the bound lane, and the assertion below blamed the
        // bridge for the fixture's own gap.
        write(
            root,
            &format!(".bee/sessions/{sid}.json"),
            &format!(
                r#"{{"id":"{sid}","lane":"f-active","last_heartbeat":"{}"}}"#,
                crate::verbs::reservations::now_iso()
            ),
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

    // ── D4 (wayfinding-flow): open discovery maps in the preamble ──────────
    //
    // This hook renders in-process (never through `bee status`), so it
    // reads `scan_discovery` independently — same scan, same guarded shape
    // as `render_status_text` (verbs/status_full/render.rs), asserted here
    // over the preamble's own builder.

    /// An open map with a frontier ticket shows up as its own section,
    /// same "empty means silent" shape as Reclaimable worktree(s) above it.
    #[test]
    fn open_maps_section_renders_with_an_effort_present() {
        let tmp = minimal_repo();
        let root = tmp.path();
        write(
            root,
            "docs/discovery/onboarding-flow/MAP.md",
            "# onboarding-flow\n\n## Destination\n\nA spec for the new flow.\n",
        );
        write(root, "docs/discovery/onboarding-flow/tickets/001-a.md", "status: open\n");
        let text = render(root);
        assert!(
            text.contains("### Open discovery map(s): onboarding-flow — 1 frontier ticket(s)"),
            "{text}"
        );
    }

    /// No `docs/discovery/` directory at all: the section never appears.
    #[test]
    fn open_maps_section_absent_with_no_discovery_dir() {
        let tmp = minimal_repo();
        let text = render(tmp.path());
        assert!(!text.contains("Open discovery map(s)"), "{text}");
    }

    // ── dispatch-door-upfront D2 ──────────────────────────────────────────

    /// RETARGETED, not rewritten (model-role-split D2 + `--role`, store
    /// 8ff6e79e). Both assertions this test always made still stand — the
    /// prepare command is published before any dispatch can happen, and the
    /// herding generation slot renders from the same source the resolver
    /// reads. What moved is the SPELLING of each: the command line now names
    /// `--role <name>`, and the published list is the host's roles rather
    /// than a fixed four-slot tier list, because there is no fixed list to
    /// print once the set is open.
    #[test]
    fn dispatch_door_renders_herding_generation_slot_and_prepare_line() {
        let tmp = minimal_repo();
        let models_obj = json!({"claude":{"generation":{"kind":"herding","agent":"agy-flash"}}});
        write(
            tmp.path(),
            ".bee/config.json",
            &json!({"models": models_obj}).to_string(),
        );
        let text = render(tmp.path());
        assert!(text.contains("### Dispatch door"), "{text}");
        assert!(
            text.contains(
                "- Every subagent/worker dispatch starts with `.bee/bin/bee dispatch prepare --runtime claude --kind cell|gather|reviewer|advisor [--role <name>] --json` — run the exact tool+payload it returns; never hand-pick subagent_type, model, or a [bee-tier] marker."
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "- Roles (claude): generation=herding (agy-flash) | review=opus | extraction=haiku — open set: any name models.claude configures is legal; one nothing configures refuses by name."
            ),
            "{text}"
        );

        // Same-source proof: the rendered generation string matches what the
        // drivers resolver returns for that map — one parser, one answer.
        let map = crate::verbs::drivers::normalize_models(Some(&models_obj));
        let resolved =
            crate::verbs::drivers::resolve_role(&map, &["generation"], "claude", "gather");
        assert_eq!(
            resolved,
            crate::verbs::drivers::Resolved::Herding {
                agent: Some("agy-flash".into()),
                fallback: None,
            }
        );
        let slots = crate::hooks::model_guard::role_slot_display(Some(&models_obj), "claude");
        let gen_str = slots.iter().find(|(k, _)| k == "generation").map(|(_, v)| v.as_str());
        assert_eq!(gen_str, Some("herding (agy-flash)"));
    }

    #[test]
    fn dispatch_door_renders_model_slot_name() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/config.json",
            r#"{"models":{"claude":{"generation":"claude-3-5-sonnet-20241022","extraction":"claude-3-5-haiku-20241022","review":"claude-3-opus-20240229","advisor":"claude-3-7-sonnet-20250219"}}}"#,
        );
        let text = render(tmp.path());
        assert!(
            text.contains(
                "- Roles (claude): generation=claude-3-5-sonnet-20241022 | review=claude-3-opus-20240229 | advisor=claude-3-7-sonnet-20250219 | extraction=claude-3-5-haiku-20241022"
            ),
            "{text}"
        );
    }

    /// The preamble is where a role's own purpose actually reaches a reader,
    /// so the described line is asserted THROUGH `render`, not just through
    /// the helper — an operator who writes `description` on a slot must see
    /// that sentence in the block their session opens with.
    #[test]
    fn dispatch_door_renders_a_role_description_when_the_slot_declares_one() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/config.json",
            r#"{"models":{"claude":{"generation":{"model":"sonnet","description":"build and edit code"},"review":"opus","design":{"model":"opus","description":"shape the thing before it is built"}}}}"#,
        );
        let text = render(tmp.path());
        assert!(
            text.contains(
                "- Roles (claude): generation=sonnet (\"build and edit code\") | review=opus | extraction=haiku | design=opus (\"shape the thing before it is built\")"
            ),
            "{text}"
        );
    }

    /// The same config with the descriptions removed renders the line bee
    /// rendered before this field existed — byte for byte. This is the guard
    /// against the additive change quietly becoming a re-render.
    #[test]
    fn dispatch_door_line_is_unchanged_when_no_slot_declares_a_description() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/config.json",
            r#"{"models":{"claude":{"generation":{"model":"sonnet"},"review":"opus","design":{"model":"opus"}}}}"#,
        );
        let text = render(tmp.path());
        assert!(
            text.contains(
                "- Roles (claude): generation=sonnet | review=opus | extraction=haiku | design=opus — open set: any name models.claude configures is legal; one nothing configures refuses by name."
            ),
            "{text}"
        );
    }

    /// No config at all: the door still publishes something, and what it
    /// publishes is the seeded defaults. `advisor=none` is gone on purpose —
    /// a role that selects no model is dropped rather than printed as a name
    /// with nothing behind it, which is one fewer thing in a block injected
    /// into every session.
    #[test]
    fn dispatch_door_renders_defaults_when_no_models_key_present() {
        let tmp = minimal_repo();
        let text = render(tmp.path());
        assert!(text.contains("### Dispatch door"), "{text}");
        assert!(
            text.contains("- Roles (claude): generation=sonnet | review=opus | extraction=haiku —"),
            "{text}"
        );
        assert!(!text.contains("advisor=none"), "an unconfigured role is dropped:\n{text}");
        assert!(!text.contains("Tier slots"), "the retired tier list is gone:\n{text}");
    }

    /// A role name bee itself never asks for is published exactly like bee's
    /// own — that is what "open set" means at this door (D2, store 06e49368).
    /// bee's own slots still read first, so the name most dispatches land on
    /// has not moved down the line.
    #[test]
    fn dispatch_door_publishes_a_role_name_bee_never_asks_for() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/config.json",
            r#"{"models":{"claude":{"generation":"opus","test":"haiku"}}}"#,
        );
        let text = render(tmp.path());
        assert!(
            text.contains(
                "- Roles (claude): generation=opus | review=opus | extraction=haiku | test=haiku —"
            ),
            "{text}"
        );
    }

    /// The block is injected into EVERY session, so its length is a real,
    /// repeated cost. Past the cap the line counts instead of listing; the
    /// truncation is safe because a name nothing configures refuses BY NAME
    /// with a FIX at both doors rather than resolving silently.
    #[test]
    fn dispatch_door_counts_roles_past_the_cap_instead_of_listing_them() {
        let tmp = minimal_repo();
        write(
            tmp.path(),
            ".bee/config.json",
            r#"{"models":{"claude":{"generation":"opus","test":"haiku","docs":"haiku","design":"opus","migrate":"haiku","triage":"haiku"}}}"#,
        );
        let text = render(tmp.path());
        // 3 seeded + 6 configured, one of which (generation) overlays a
        // seeded slot: 8 roles, 6 shown.
        assert!(text.contains(" +2 more —"), "{text}");
        assert!(!text.contains("triage="), "the 8th role is counted, not listed:\n{text}");
    }

    /// model-role-split records `effort` as a known NON-delivery, so the door
    /// must not print one. It USED to: `render_resolved` spelled a
    /// `{model, effort}` slot as `model:effort` while every `Resolved::Model`
    /// site in `verbs/drivers/prepare.rs` destructures `{ model, .. }` and
    /// drops it — and on codex the `spawn_agent` arm drops it for its OWN
    /// reason (only the `native` arm emits `reasoning_effort`), which the
    /// claude harness explanation does not cover. Publishing a value no
    /// dispatch carries is the exact silent-lie shape this feature removes.
    #[test]
    fn dispatch_door_never_publishes_an_effort_the_dispatch_discards() {
        let tmp = minimal_repo();
        let models_obj = json!({"claude":{"generation":{"model":"opus","effort":"high"}}});
        write(
            tmp.path(),
            ".bee/config.json",
            &json!({"models": models_obj}).to_string(),
        );

        // The effort really is parsed and really does reach the resolver —
        // this is a rendering choice, not a config that failed to load.
        let map = crate::verbs::drivers::normalize_models(Some(&models_obj));
        assert_eq!(
            crate::verbs::drivers::resolve_role(&map, &["generation"], "claude", "gather"),
            crate::verbs::drivers::Resolved::Model {
                model: "opus".into(),
                effort: Some("high".into()),
            }
        );

        let text = render(tmp.path());
        assert!(text.contains("- Roles (claude): generation=opus |"), "{text}");
        assert!(!text.contains("opus:high"), "the door published a dropped effort:\n{text}");
        assert!(!text.contains(":high"), "the door published a dropped effort:\n{text}");
    }
