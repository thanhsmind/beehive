// Split out of the single 4.4k-line verbs/knowledge.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's inline module, indentation
// and all: a moved inline module is the same child of the same parent,
// so no path changes, and the fixtures inside are raw strings whose
// leading whitespace is content.

// The parent module's own `use` block travels with the tests: they reach
// for names mod.rs no longer imports now that the code using them lives
// in sibling modules.
#![allow(unused_imports)]

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
    use super::*;

    fn parse_ok(text: &str) -> Map<String, Value> {
        match parse_frontmatter(text) {
            Fm::Parsed { data, .. } => data,
            _ => panic!("expected parse"),
        }
    }

    #[test]
    fn frontmatter_round_trips_canonical_form() {
        let text = "---\ntype: bee.pattern\ntitle: \"A: colon title\"\ntags: [one, two]\nbee:\n  id: p-1\n  lifecycle: active\n  critical: true\n---\nbody\n";
        let (data, block, body) = match parse_frontmatter(text) {
            Fm::Parsed { data, block, body } => (data, block, body),
            _ => panic!("parse failed"),
        };
        assert_eq!(body, "body\n");
        assert_eq!(emit_frontmatter(&data).unwrap(), block);
        assert_eq!(data["title"], Value::String("A: colon title".into()));
        assert_eq!(data["bee"]["critical"], Value::Bool(true));
    }

    #[test]
    fn frontmatter_failures_match_node_codes() {
        match parse_frontmatter("---\ntitle 'x'\n---\n") {
            Fm::Failed { code, .. } => assert_eq!(code, "unrecognized_line"),
            _ => panic!("expected failure"),
        }
        match parse_frontmatter("---\ntitle: 'x'\n---\n") {
            Fm::Failed { code, .. } => assert_eq!(code, "single_quoted_string"),
            _ => panic!("expected failure"),
        }
        match parse_frontmatter("---\n\ntitle: x\n---\n") {
            Fm::Failed { code, line, .. } => {
                assert_eq!(code, "blank_line");
                assert_eq!(line, 2);
            }
            _ => panic!("expected failure"),
        }
        match parse_frontmatter("---\ntitle: x") {
            Fm::Failed { code, .. } => assert_eq!(code, "unclosed_frontmatter"),
            _ => panic!("expected failure"),
        }
        // CUTOVER: a lone-surrogate escape used to be NeedsNode (delegate).
        // It is now the ordinary undecodable-quoted-scalar finding.
        match parse_frontmatter("---\ntitle: \"\\ud800\"\n---\n") {
            Fm::Failed { code, .. } => assert_eq!(code, "bad_quoted_string"),
            _ => panic!("a lone surrogate must be a finding, not a delegation"),
        }
    }

    #[test]
    fn crlf_parses_but_block_keeps_bytes() {
        let text = "---\r\ntype: bee.pattern\r\n---\r\nbody";
        match parse_frontmatter(text) {
            Fm::Parsed { data, block, .. } => {
                assert_eq!(data["type"], Value::String("bee.pattern".into()));
                assert!(block.contains('\r'));
                assert_ne!(emit_frontmatter(&data).unwrap(), block); // not_canonical trigger
            }
            _ => panic!("expected parse"),
        }
    }

    #[test]
    fn iso_heading_calendar_check_matches_date_utc() {
        assert!(is_iso_date_heading("2026-02-28"));
        assert!(is_iso_date_heading("2024-02-29"));
        assert!(!is_iso_date_heading("2026-02-29"));
        assert!(!is_iso_date_heading("2026-13-01"));
        assert!(!is_iso_date_heading("0099-01-01")); // Date.UTC maps 0-99 to 1900+y
        assert!(is_iso_date_heading("2026-07-01T10:30"));
        assert!(is_iso_date_heading("2026-07-01 10:30:05.123Z"));
        assert!(is_iso_date_heading("2026-07-01T10:30:05+07:00"));
        assert!(!is_iso_date_heading("yesterday"));
    }

    #[test]
    fn bundle_check_and_index_render() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("knowledge");
        std::fs::create_dir_all(dir.join("areas/billing")).unwrap();
        std::fs::write(
            dir.join("areas/billing/refunds.md"),
            "---\ntype: bee.pattern\ntitle: Refund flow\ndescription: How refunds settle\nbee:\n  id: pat-1\n  lifecycle: active\n  critical: true\n---\nBody here.\n",
        )
        .unwrap();
        std::fs::write(dir.join("notes.md"), "no frontmatter\n").unwrap();

        let report = check_bundle(&dir, false).unwrap();
        assert_eq!(report.files, 2);
        assert_eq!(report.concepts, 2);
        assert_eq!(report.okf_errors.len(), 1); // notes.md missing_frontmatter
        assert_eq!(report.okf_errors[0]["code"], "missing_frontmatter");
        assert!(report.warnings.is_empty());
        assert!(!report.ok);

        let files = compute_index_files(&dir).unwrap();
        // Dir-sort order: '' sorts first, so the root index leads the set.
        let rels: Vec<&str> = files.iter().map(|(r, _)| r.as_str()).collect();
        assert_eq!(rels, vec!["index.md", "areas/index.md", "areas/billing/index.md"]);
        let root = &files[0].1;
        assert!(root.starts_with("---\nokf_version: 0.1\n---\n<!--\n"));
        assert!(root.contains("## Critical patterns"));
        assert!(root.contains("- [Refund flow](areas/billing/refunds.md) — How refunds settle"));
        assert!(root.contains("- [areas/](areas/index.md) — 1 concept(s)"));
        // Non-root indexes carry no frontmatter.
        assert!(files[1].1.starts_with("<!--\n"));
    }

    /// knowledge.mjs foldEncoding + normalizeSubject, unit-level. Every row is
    /// an ENCODING difference that must NOT be able to buy a second authority
    /// for one subject, paired with the genuine-difference control.
    #[test]
    fn normalize_subject_is_a_skeleton_not_a_string() {
        // Case, punctuation and whitespace are not identity.
        assert_eq!(normalize_subject("Billing: Refunds!"), "billing refunds");
        assert_eq!(normalize_subject("  BILLING---refunds.  "), "billing refunds");
        // No letters or digits at all -> '' (the signal layer 2 refuses on).
        for empty in ["", "   ", "...", "-- //"] {
            assert_eq!(normalize_subject(empty), "", "{empty:?}");
        }

        // NFKC: fullwidth, ligature and math-alphanumeric forms all fold.
        assert_eq!(normalize_subject("\u{ff47}\u{ff41}\u{ff54}\u{ff45}\u{ff53}"), "gates");
        assert_eq!(normalize_subject("\u{fb01}le"), "file"); // ﬁ ligature
        assert_eq!(normalize_subject("\u{1d420}ates"), "gates"); // 𝐠 math bold
        assert_eq!(normalize_subject("\u{2460}"), "1"); // ① circled digit
        // NFD + \p{M} strip: diacritics are not identity, precomposed or not.
        assert_eq!(normalize_subject("caf\u{e9}"), "cafe");
        assert_eq!(normalize_subject("cafe\u{301}"), "cafe");
        assert_eq!(normalize_subject("N\u{c3}\u{a9}"), normalize_subject("N\u{c3}\u{a9}"));
        // Confusable fold: NFKC alone leaves these distinct forever.
        assert_eq!(normalize_subject("g\u{430}tes"), "gates"); // Cyrillic 'а'
        assert_eq!(normalize_subject("gat\u{435}s"), "gates"); // Cyrillic 'е'
        assert_eq!(normalize_subject("g\u{3b1}tes"), "gates"); // Greek 'α'
        assert_eq!(normalize_subject("\u{41a}ey"), "key"); // uppercase Cyrillic 'К'
        assert_eq!(normalize_subject("\u{451}poch"), "epoch"); // Cyrillic 'ё'
        // The fold is bounded: a letter with no ASCII look-alike survives, so
        // two genuinely different scripts are still two subjects.
        assert_ne!(normalize_subject("gates"), normalize_subject("шлюзы"));
        // …and a word-order paraphrase is a DIFFERENT subject, never folded
        // (the residual layer-1 cannot close, by design).
        assert_ne!(
            normalize_subject("refunds and reversals"),
            normalize_subject("reversals and refunds")
        );
    }

    /// The confusable table is transcribed by hand from the .mjs map, so it is
    /// pinned as a set: exactly the 25 Cyrillic + 17 Greek entries, no more.
    #[test]
    fn the_confusable_table_is_exactly_the_mjs_map() {
        assert_eq!(CONFUSABLE_FOLD.len(), 42);
        let mut seen: Vec<char> = CONFUSABLE_FOLD.iter().map(|(f, _)| *f).collect();
        let before = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), before, "a duplicated key would silently shadow");
        for (from, to) in CONFUSABLE_FOLD {
            assert!(to.is_ascii(), "{from:?} folds to a non-ASCII target {to:?}");
            assert_eq!(from.to_lowercase().next(), Some(from), "{from:?} must be a lowercase key");
        }
    }

    #[test]
    fn bundle_target_normalization_matches_path_resolve_containment() {
        // An absolute dir whose components this platform can actually see:
        // on Linux "D:\\repo\\docs\\knowledge" is ONE opaque component, so
        // `../knowledge/y.md` escapes the base and the containment cases
        // below stop testing containment.
        #[cfg(windows)]
        let dir = Path::new("D:\\repo\\docs\\knowledge");
        #[cfg(not(windows))]
        let dir = Path::new("/repo/docs/knowledge");
        let norm = |t: &str| normalize_bundle_target(dir, t);
        assert_eq!(norm("areas/x.md").unwrap().unwrap(), "areas/x.md");
        assert_eq!(norm("a/./b/../c.md").unwrap().unwrap(), "a/c.md");
        assert_eq!(norm("../escape.md").unwrap(), None);
        // Climb out and re-enter — path.resolve calls this contained.
        assert_eq!(norm("../knowledge/y.md").unwrap().unwrap(), "y.md");
        assert_eq!(norm("../KNOWLEDGE/y.md").unwrap(), None); // case-sensitive prefix
        assert!(norm("/abs.md").is_err()); // rooted → delegate
        assert!(norm("C:/x.md").is_err()); // drive shape → delegate
    }

    #[test]
    fn relevance_tokens_stop_and_singularize() {
        let stops = stopwords();
        // "rows".length is 4, NOT > 4 — Node keeps it plural; "refunds" drops the s.
        assert_eq!(
            relevance_tokens("The refunds and Reversals of class rows!", &stops),
            vec!["refund", "reversal", "class", "rows"]
        );
        // <=2 chars and stopwords drop; 'ss' endings keep.
        assert_eq!(relevance_tokens("is at process", &stops), vec!["process"]);
    }

    #[test]
    fn js_number_conv_subset() {
        assert_eq!(js_number_conv("20000"), Some(20000.0));
        assert_eq!(js_number_conv("  1.5e3 "), Some(1500.0));
        assert_eq!(js_number_conv(".5"), Some(0.5));
        assert_eq!(js_number_conv(""), Some(0.0));
        assert_eq!(js_number_conv("0x10"), None); // JS-valid but delegated
        assert_eq!(js_number_conv("Infinity"), None);
        assert_eq!(js_number_conv("12px"), None);
    }

    #[test]
    fn context_manifest_orders_and_cuts() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("knowledge");
        std::fs::create_dir_all(dir.join("work/w1")).unwrap();
        std::fs::write(
            dir.join("work/w1/item.md"),
            // required_context targets resolve against the BUNDLE root (D19).
            "---\ntype: bee.work-item\ntitle: Widget work\ndescription: widgets and gears\nbee:\n  id: w1\n  lifecycle: active\n  areas: [billing]\n  decisions: [\"0001\"]\n  required_context: [ctx.md]\n---\nwidgets gears assembly\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("work/w1/plan.md"),
            "---\ntype: bee.plan\ntitle: Plan\nbee:\n  id: w1-plan\n  lifecycle: active\n---\nplan body\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("ctx.md"),
            "---\ntype: bee.pattern\ntitle: Context doc\nbee:\n  id: ctx\n  lifecycle: active\n---\nctx body\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("crit.md"),
            "---\ntype: bee.pattern\ntitle: Widget gear lesson\ndescription: widgets gears\nbee:\n  id: crit\n  lifecycle: active\n  critical: true\n---\nwidgets gears everywhere\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("dec.md"),
            "---\ntype: bee.decision\ntitle: Billing decision\nbee:\n  id: dec\n  lifecycle: active\n  areas: [billing]\n---\ndecision body\n",
        )
        .unwrap();

        let manifest = match build_context_manifest(&dir, "w1", 20000.0, &json_raw("20000")) {
            ManifestOut::Built(m) => m,
            _ => panic!("expected manifest"),
        };
        let entries: Vec<String> = manifest["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["path"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            entries,
            vec![
                "docs/knowledge/work/w1/item.md",
                "docs/knowledge/work/w1/plan.md",
                "docs/knowledge/ctx.md",
                "docs/knowledge/crit.md",
                "docs/knowledge/dec.md",
            ]
        );
        assert_eq!(manifest["decisions"], serde_json::json!(["0001"]));
        assert_eq!(manifest["critical_total"], serde_json::json!(1));
        assert_eq!(manifest["floor"], serde_json::json!(["docs/knowledge/crit.md"]));
        let reasons: Vec<&str> = manifest["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["reason"].as_str().unwrap())
            .collect();
        assert_eq!(reasons[0], "work item");
        assert_eq!(reasons[1], "plan sibling in work/w1/");
        assert_eq!(reasons[2], "required_context depth 1 via work/w1/item.md");
        assert!(reasons[3].starts_with("critical pattern (relevance "));
        assert!(reasons[3].ends_with(", rank 1 of 1, floor)"));
        assert_eq!(reasons[4], "decision for area billing");

        // Zero budget still includes nothing — hard ceiling.
        let manifest0 = match build_context_manifest(&dir, "w1", 0.0, &json_raw("0")) {
            ManifestOut::Built(m) => m,
            _ => panic!("expected manifest"),
        };
        assert!(manifest0["entries"].as_array().unwrap().is_empty());
        assert_eq!(manifest0["truncated"].as_array().unwrap().len(), 5);

        // Unknown work id throws the typed error.
        match build_context_manifest(&dir, "nope", 100.0, &json_raw("100")) {
            ManifestOut::Thrown(msg) => assert!(msg.contains("unknown_work")),
            _ => panic!("expected thrown"),
        }

        // bad_budget quotes the RAW CLI string, JSON.stringify-style.
        match build_context_manifest(&dir, "w1", -5.0, &json_raw("-5")) {
            ManifestOut::Thrown(msg) => assert!(msg.contains("got \"-5\" (D27)")),
            _ => panic!("expected thrown"),
        }
    }

    fn json_raw(s: &str) -> Value {
        Value::String(s.to_string())
    }

    #[test]
    fn to_fixed6_matches_number_tofixed_shape() {
        assert_eq!(to_fixed6(0.05), 0.05);
        assert_eq!(to_fixed6(0.1234567), 0.123457);
        assert_eq!(to_fixed6(0.0), 0.0);
    }

    // ── knowledge promote ──────────────────────────────────────────────────

    #[test]
    fn compare_cell_ids_is_natural_order() {
        use std::cmp::Ordering::*;
        let mut ids = vec!["okf-10", "okf-9", "okf-1", "okf-2b", "okf-2a", "zz", "okf"];
        ids.sort_by(|a, b| compare_cell_ids(a, b));
        assert_eq!(ids, vec!["okf", "okf-1", "okf-2a", "okf-2b", "okf-9", "okf-10", "zz"]);
        assert_eq!(compare_cell_ids("a1", "a1"), Equal);
        assert_eq!(compare_cell_ids("a01", "a1"), Equal); // Number('01') === 1
        assert_eq!(compare_cell_ids("a", "a1"), Less);    // shorter split runs out
        assert_eq!(compare_cell_ids("a1", "a"), Greater);
    }

    #[test]
    fn one_line_collapses_whitespace_and_caps() {
        assert_eq!(one_line("  a\n\tb   c  ", 0), "a b c");
        assert_eq!(one_line("", 0), "");
        assert_eq!(one_line("abcdef", 4), "abc\u{2026}");
        assert_eq!(one_line("abcd", 4), "abcd"); // exactly at the limit
        assert_eq!(strip_one_trailing_newline("x\n"), "x");
        assert_eq!(strip_one_trailing_newline("x\n\n"), "x\n");
    }

    #[test]
    fn deviation_text_handles_both_recorded_shapes() {
        assert_eq!(deviation_text(&json!("plain")), "plain");
        assert_eq!(
            deviation_text(&json!({"type": "scope", "description": "why"})),
            "scope: why"
        );
        assert_eq!(deviation_text(&json!({"description": "why"})), "why");
        assert_eq!(deviation_text(&json!({"note": "x"})), r#"{"note":"x"}"#);
        assert_eq!(deviation_text(&json!(["a"])), r#"["a"]"#);
        assert_eq!(deviation_text(&json!(7)), "7");
    }

    #[test]
    fn verify_summary_prefers_the_recorded_keys() {
        let ev = |raw: &str| verify_summary(&json!({"verification_evidence": raw}));
        assert_eq!(ev("").unwrap(), "");
        assert_eq!(ev("   ").unwrap(), "");
        assert_eq!(ev(r#"{"summary":"s","verify_tail":"t"}"#).unwrap(), "t"); // key order fixed
        assert_eq!(ev(r#"{"evidence":"e"}"#).unwrap(), "e");
        assert_eq!(ev(r#"{"other":"x"}"#).unwrap(), r#"{"other":"x"}"#);
        assert_eq!(ev("just text  here").unwrap(), "just text here");
        assert_eq!(verify_summary(&json!({})).unwrap(), "");
        // CUTOVER: JSON-looking text this CLI cannot parse used to delegate
        // ("only V8 knows which branch ran"). With one parser left, the catch
        // branch IS the answer — the raw text, one-lined.
        assert_eq!(ev(r#"{"a":"\ud800"}"#).unwrap(), r#"{"a":"\ud800"}"#);
        assert_eq!(ev("{not json").unwrap(), "{not json");
    }

    #[test]
    fn iso_date_and_touches_subject() {
        assert_eq!(iso_date(Some(&json!("2024-06-02T10:00:00Z"))).as_deref(), Some("2024-06-02"));
        assert_eq!(iso_date(Some(&json!("2024-06-02"))).as_deref(), Some("2024-06-02"));
        assert_eq!(iso_date(Some(&json!("2024-6-2"))), None);
        assert_eq!(iso_date(Some(&json!(20240602))), None);
        assert_eq!(iso_date(None), None);
        assert!(touches_subject("src/cli/main.rs", "src/cli"));
        assert!(touches_subject("src/cli", "src/cli/main.rs"));
        assert!(touches_subject("a", "a"));
        assert!(!touches_subject("src/clix/main.rs", "src/cli"));
    }

    #[test]
    fn promotion_mines_capped_traces_and_proposes_without_writing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let kn = root.join("docs").join("knowledge");
        std::fs::create_dir_all(kn.join("work")).unwrap();
        std::fs::create_dir_all(kn.join("areas")).unwrap();
        std::fs::write(
            kn.join("work").join("w1.md"),
            "---\ntype: bee.work-item\ntitle: Widget work\ndescription: does widgets\ntags: [alpha]\nbee:\n  id: w1\n  lifecycle: active\n  areas: [cli]\n  lane: small\n---\n\n# Widget work\n",
        )
        .unwrap();
        std::fs::write(
            kn.join("areas").join("cli.md"),
            "---\ntype: bee.area\ntitle: CLI\ndescription: the cli\nbee:\n  id: a-cli\n  lifecycle: active\n  areas: [cli]\n  sources: [\"src/cli\"]\n---\n\n# CLI\n",
        )
        .unwrap();
        let cells = root.join(".bee").join("cells");
        std::fs::create_dir_all(cells.join("archive")).unwrap();
        std::fs::write(
            cells.join("w1-10.json"),
            r#"{"id":"w1-10","feature":"w1","status":"capped","title":"tenth","verify":"cargo test","trace":{"behavior_change":true,"outcome":"did   the  thing","files_changed":["src/cli/main.rs","other.rs"],"deviations":["dev one",{"type":"scope","description":"dev two"},"  "],"attempts":[{"failure_signature":"boom"}],"capped_at":"2024-06-02T10:00:00.000Z","verification_evidence":"{\"verify_tail\":\"green\"}"}}"#,
        )
        .unwrap();
        std::fs::write(
            cells.join("w1-9.json"),
            r#"{"id":"w1-9","feature":"w1","status":"capped","title":"ninth","verify":"npm test","behavior_change":true,"trace":{"files_changed":["src/store/x.rs"],"capped_at":"2024-06-01T00:00:00.000Z"}}"#,
        )
        .unwrap();
        std::fs::write(
            cells.join("w1-3.json"),
            r#"{"id":"w1-3","feature":"w1","status":"open","title":"open cell"}"#,
        )
        .unwrap();
        std::fs::write(
            cells.join("archive").join("w1-99.json"),
            r#"{"id":"w1-99","feature":"w1","status":"capped","title":"archived"}"#,
        )
        .unwrap();

        let Some(Promo::Ok(p)) = build_promotion(root, &kn, "w1") else {
            panic!("expected a proposal")
        };
        // Natural id order; the archive subdir and the open cell never appear.
        let ids: Vec<&str> = p["cells"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["w1-9", "w1-10"]);
        assert_eq!(p["writes"], json!([]));
        assert_eq!(p["work_item"], "work/w1.md");
        assert_eq!(p["delivery"]["path"], "work/delivery.md");
        assert_eq!(p["delivery"]["repo_path"], "docs/knowledge/work/delivery.md");
        // trace.outcome wins over the title; the fallback is the title.
        assert_eq!(p["cells"][1]["outcome"], "did   the  thing");
        assert_eq!(p["cells"][0]["outcome"], "ninth");
        assert_eq!(p["cells"][1]["verify_summary"], "green");
        // behavior_change: trace.true, and the cell-level fallback.
        assert_eq!(p["cells"][0]["behavior_change"], true);
        // Timestamp = the LATEST capped date.
        let content = p["delivery"]["content"].as_str().unwrap();
        assert!(content.contains("timestamp: 2024-06-02"));
        assert!(content.starts_with("---\ntype: bee.delivery\n"));
        assert!(content.contains("bee:\n  id: w1-delivery\n  lifecycle: active\n  areas: [cli]\n"));
        assert!(content.contains("lane: small"));
        assert!(content.contains("- **w1-10** — did the thing (2 file(s) changed)"));
        assert!(content.contains("- **w1-10** — `cargo test` — green"));
        assert!(content.contains("- **w1-10** — scope: dev two"));

        // Area bullets: only the behavior_change cell whose files touch the
        // area subjects (src/cli via the area concept's sources).
        let areas = p["area_updates"].as_array().unwrap();
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0]["area"], "cli");
        assert_eq!(
            areas[0]["subjects"],
            json!(["docs/knowledge/areas/cli.md", "docs/knowledge/work/w1.md", "src/cli"])
        );
        assert_eq!(areas[0]["bullets"].as_array().unwrap().len(), 1);
        assert_eq!(areas[0]["bullets"][0]["files"], json!(["src/cli/main.rs"]));

        // Pattern candidates: only cells carrying a deviation or a signature.
        let cands = p["pattern_candidates"].as_array().unwrap();
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0]["cell"], "w1-10");
        assert_eq!(cands[0]["repo_path"], "docs/knowledge/patterns/w1-w1-10-pitfall.md");
        assert_eq!(
            cands[0]["evidence"],
            json!([
                {"kind": "deviation", "text": "dev one"},
                {"kind": "deviation", "text": "scope: dev two"},
                {"kind": "failure_signature", "text": "boom"},
            ])
        );
        assert!(cands[0]["content"].as_str().unwrap().contains("polarity: pitfall"));

        // Nothing was written anywhere under docs/knowledge/.
        assert!(!kn.join("work").join("delivery.md").exists());
        assert!(!kn.join("patterns").exists());

        // Typed refusals.
        assert!(matches!(
            build_promotion(root, &kn, "   "),
            Some(Promo::Thrown(m)) if m == "knowledge promote: missing_work — --work <id> is required (D38)."
        ));
        assert!(matches!(
            build_promotion(root, &kn, "nope"),
            Some(Promo::Thrown(m)) if m.starts_with("knowledge promote: unknown_work — no bee.work-item concept")
        ));
    }

    /// D1/D38: neither a bee.work-item concept nor a docs/history/<work>/
    /// file exists — the refusal text and its (D38) tag survive resolve_anchor
    /// byte for byte, unchanged from before this feature.
    #[test]
    fn promotion_unknown_work_message_is_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let kn = root.join("docs").join("knowledge");
        std::fs::create_dir_all(&kn).unwrap();
        assert!(matches!(
            build_promotion(root, &kn, "ghost"),
            Some(Promo::Thrown(m)) if m == "knowledge promote: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id \"ghost\" (D38)."
        ));
    }

    /// D1: a feature with no bee.work-item concept but a docs/history/<slug>/
    /// CONTEXT.md still proposes off its capped cell traces — the anchor
    /// stands in for the work-item concept the rest of build_promotion reads.
    #[test]
    fn promotion_resolves_a_history_anchor_when_no_work_item_concept_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let kn = root.join("docs").join("knowledge");
        std::fs::create_dir_all(&kn).unwrap();
        std::fs::create_dir_all(root.join("docs").join("history").join("hist-1")).unwrap();
        std::fs::write(
            root.join("docs").join("history").join("hist-1").join("CONTEXT.md"),
            "# Hist One Context\n\nBody.\n",
        )
        .unwrap();
        let cells = root.join(".bee").join("cells");
        std::fs::create_dir_all(&cells).unwrap();
        std::fs::write(
            cells.join("hist-1-1.json"),
            r#"{"id":"hist-1-1","feature":"hist-1","status":"capped","title":"first","verify":"cargo test","trace":{"behavior_change":true,"outcome":"did the thing","files_changed":["src/a.rs"],"capped_at":"2024-06-02T10:00:00.000Z"}}"#,
        )
        .unwrap();

        let Some(Promo::Ok(p)) = build_promotion(root, &kn, "hist-1") else {
            panic!("expected a proposal off the history anchor")
        };
        assert_eq!(p["writes"], json!([]));
        assert_eq!(p["anchor"]["kind"], "history");
        assert_eq!(p["anchor"]["paths"], json!(["docs/history/hist-1/CONTEXT.md"]));
        assert_eq!(p["work_item"], "docs/history/hist-1/CONTEXT.md");
        // The canonical proposed save path — a PROPOSAL only; D5 holds, nothing
        // under docs/knowledge/ is ever created, moved, or deleted here.
        assert_eq!(p["delivery"]["path"], "work/hist-1/delivery.md");
        assert_eq!(p["delivery"]["repo_path"], "docs/knowledge/work/hist-1/delivery.md");
        let ids: Vec<&str> =
            p["cells"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["hist-1-1"]);
        assert!(!kn.join("work").exists());

        // Empty bee.areas keeps its existing D19 render, unchanged, and the
        // text names the anchor on its own line.
        let text = promote_text(&p);
        assert!(text.contains(
            "None: the work item declares no bee.areas, so there is no area to sync (D19)."
        ));
        assert!(text.contains("anchor: history — docs/history/hist-1/CONTEXT.md"));
    }

    // ═══ R5: fixture builders ══════════════════════════════════════════════
    //
    // Node oracle: tests/test_knowledge.mjs makeRepo / writeBundleFile /
    // conceptText (l.60–108). Fixtures are authored THROUGH `emit_frontmatter`
    // — D12 makes the emitter the subset's source of truth — so every fixture
    // is canonical by construction and `not_canonical` can only fire where a
    // test bends the bytes on purpose.

    fn bundle() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("docs").join("knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        (tmp, dir)
    }

    fn write_bundle_file(dir: &Path, rel: &str, text: &str) {
        let abs = join_rel(dir, rel);
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(abs, text).unwrap();
    }

    struct Cx {
        ty: &'static str,
        title: String,
        description: Option<String>,
        id: String,
        lifecycle: &'static str,
        tags: Vec<String>,
        areas: Vec<String>,
        bee_extra: Vec<(&'static str, Value)>,
        body: String,
    }

    impl Cx {
        fn new(id: &str) -> Self {
            Cx {
                ty: "bee.pattern",
                title: "A demo pattern".into(),
                description: Some("A canonical fixture concept".into()),
                id: id.into(),
                lifecycle: "active",
                tags: vec!["demo".into()],
                areas: vec!["demo-area".into()],
                bee_extra: Vec::new(),
                body: "Body.".into(),
            }
        }
        fn ty(mut self, t: &'static str) -> Self {
            self.ty = t;
            self
        }
        fn title(mut self, t: &str) -> Self {
            self.title = t.into();
            self
        }
        fn description(mut self, d: &str) -> Self {
            self.description = Some(d.into());
            self
        }
        fn no_description(mut self) -> Self {
            self.description = None;
            self
        }
        fn lifecycle(mut self, l: &'static str) -> Self {
            self.lifecycle = l;
            self
        }
        fn tags(mut self, t: &[&str]) -> Self {
            self.tags = t.iter().map(|s| (*s).to_string()).collect();
            self
        }
        fn areas(mut self, a: &[&str]) -> Self {
            self.areas = a.iter().map(|s| (*s).to_string()).collect();
            self
        }
        fn body(mut self, b: &str) -> Self {
            self.body = b.into();
            self
        }
        fn bee(mut self, key: &'static str, value: Value) -> Self {
            self.bee_extra.push((key, value));
            self
        }
        fn critical(self) -> Self {
            self.bee("critical", json!(true))
        }

        fn text(&self) -> String {
            let mut data = Map::new();
            data.insert("type".into(), json!(self.ty));
            data.insert("title".into(), json!(self.title));
            if let Some(d) = &self.description {
                data.insert("description".into(), json!(d));
            }
            data.insert("tags".into(), json!(self.tags));
            data.insert("timestamp".into(), json!("2026-07-22"));
            let mut bee = Map::new();
            bee.insert("id".into(), json!(self.id));
            bee.insert("lifecycle".into(), json!(self.lifecycle));
            bee.insert("areas".into(), json!(self.areas));
            bee.insert("required_context".into(), json!([]));
            bee.insert("decisions".into(), json!([]));
            bee.insert("sources".into(), json!([]));
            for (k, v) in &self.bee_extra {
                bee.insert((*k).to_string(), v.clone());
            }
            data.insert("bee".into(), Value::Object(bee));
            format!("{}\n# {}\n\n{}\n", emit_frontmatter(&data).unwrap(), self.title, self.body)
        }
    }

    fn put(dir: &Path, rel: &str, c: Cx) {
        write_bundle_file(dir, rel, &c.text());
    }

    fn codes(list: &[Value]) -> Vec<&str> {
        list.iter().map(|f| f["code"].as_str().unwrap()).collect()
    }

    fn of_code<'a>(list: &'a [Value], code: &str) -> Vec<&'a Value> {
        list.iter().filter(|f| f["code"] == code).collect()
    }

    fn msg(f: &Value) -> &str {
        f["message"].as_str().unwrap()
    }

    // ═══ profile WARNINGS (D4) ═════════════════════════════════════════════

    /// Node: 'profile warning: type outside the D18 nine warns, does not
    /// error, exits ok un-strict' (test_knowledge.mjs l.271).
    #[test]
    fn unknown_type_warns_without_erroring_and_a_profile_type_stays_silent() {
        let (_tmp, dir) = bundle();
        put(&dir, "patterns/guide.md", Cx::new("guide-1").ty("bee.guide"));
        put(&dir, "patterns/known.md", Cx::new("known-1")); // control: bee.pattern is in the nine
        let report = check_bundle(&dir, false).unwrap();
        assert!(
            report.okf_errors.is_empty(),
            "an unknown type is a SHOULD, never an OKF error: {:?}",
            codes(&report.okf_errors)
        );
        let warns = of_code(&report.warnings, "unknown_type");
        assert_eq!(warns.len(), 1, "only the off-vocabulary file may warn: {:?}", report.warnings);
        assert_eq!(warns[0]["file"], "patterns/guide.md");
        assert!(msg(warns[0]).contains("bee.guide"), "the offending type must flow into the message: {}", msg(warns[0]));
        assert!(report.ok, "warnings alone must not fail un-strict");
    }

    /// Node: 'profile warning: missing profile-required field (D10: never
    /// invented, warned by name)' (l.281). The nested `bee.id`/`bee.lifecycle`
    /// paths exercise readPath's object walk, which the flat cases cannot.
    #[test]
    fn missing_profile_field_warns_by_name_and_a_complete_concept_stays_silent() {
        let (_tmp, dir) = bundle();
        put(&dir, "patterns/complete.md", Cx::new("complete")); // control: all four present
        put(&dir, "patterns/undescribed.md", Cx::new("undescribed").no_description());
        // No `bee:` map at all — readPath stops mid-walk on both nested keys.
        write_bundle_file(
            &dir,
            "patterns/nobee.md",
            "---\ntype: bee.pattern\ntitle: No bee map\ndescription: Has no bee map\n---\nBody.\n",
        );
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.okf_errors.is_empty(), "a missing profile field is never an OKF error: {:?}", codes(&report.okf_errors));
        let warns = of_code(&report.warnings, "missing_profile_field");
        // Walk order: files path-sorted, keys in PROFILE_REQUIRED order.
        let got: Vec<(&str, &str)> = warns.iter().map(|w| (w["file"].as_str().unwrap(), msg(w))).collect();
        assert_eq!(got.len(), 3, "exactly the three absent fields may warn: {:?}", report.warnings);
        assert_eq!(got[0].0, "patterns/nobee.md");
        assert!(got[0].1.contains("\"bee.id\""), "{}", got[0].1);
        assert_eq!(got[1].0, "patterns/nobee.md");
        assert!(got[1].1.contains("\"bee.lifecycle\""), "{}", got[1].1);
        assert_eq!(got[2].0, "patterns/undescribed.md");
        assert!(got[2].1.contains("\"description\""), "{}", got[2].1);
        assert!(report.ok, "a missing profile field is a warning, green un-strict");
    }

    /// Node: 'profile warning: dangling required_context path; a resolving
    /// path stays silent' (l.290).
    #[test]
    fn dangling_required_context_warns_only_for_the_unresolvable_target() {
        let (_tmp, dir) = bundle();
        put(&dir, "areas/demo/overview.md", Cx::new("demo-overview").ty("bee.area"));
        put(
            &dir,
            "patterns/linked.md",
            Cx::new("linked").bee(
                "required_context",
                // one resolving target (the control) + one ghost
                json!(["areas/demo/overview.md", "areas/ghost/nothing.md"]),
            ),
        );
        let report = check_bundle(&dir, false).unwrap();
        let dangling = of_code(&report.warnings, "dangling_required_context");
        assert_eq!(dangling.len(), 1, "only the ghost path may warn: {:?}", report.warnings);
        assert_eq!(dangling[0]["file"], "patterns/linked.md");
        assert!(
            msg(dangling[0]).contains("areas/ghost/nothing.md"),
            "the unresolved target must be named: {}",
            msg(dangling[0])
        );
        assert!(report.ok);
    }

    /// Node: 'profile warning: dangling supersedes id; a resolving id stays
    /// silent' (l.300).
    #[test]
    fn dangling_supersedes_warns_only_for_the_id_no_concept_claims() {
        let (_tmp, dir) = bundle();
        put(&dir, "patterns/old.md", Cx::new("old-pattern").lifecycle("superseded"));
        put(&dir, "patterns/new.md", Cx::new("new-pattern").bee("supersedes", json!("old-pattern")));
        put(&dir, "patterns/orphan.md", Cx::new("orphan").bee("supersedes", json!("never-existed")));
        let report = check_bundle(&dir, false).unwrap();
        let dangling = of_code(&report.warnings, "dangling_supersedes");
        assert_eq!(dangling.len(), 1, "the resolving supersedes must stay silent: {:?}", report.warnings);
        assert_eq!(dangling[0]["file"], "patterns/orphan.md");
        assert!(msg(dangling[0]).contains("never-existed"), "{}", msg(dangling[0]));
        assert!(report.ok);
    }

    /// Node: 'profile warning: duplicate bee.id (D31: id is globally unique)'
    /// (l.310). A duplicate id is a WARNING — the pair to the authority ERROR
    /// below, which fails the chain on its own.
    #[test]
    fn duplicate_id_warns_and_names_every_claimant() {
        let (_tmp, dir) = bundle();
        put(&dir, "patterns/a.md", Cx::new("same-id"));
        put(&dir, "patterns/b.md", Cx::new("same-id"));
        put(&dir, "patterns/c.md", Cx::new("unique-id")); // control: never named
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.profile_errors.is_empty(), "a duplicate id is not a profile error: {:?}", codes(&report.profile_errors));
        let dup = of_code(&report.warnings, "duplicate_id");
        assert_eq!(dup.len(), 1, "one finding per duplicated id: {:?}", report.warnings);
        assert_eq!(dup[0]["file"], "patterns/a.md", "the finding is filed against the first claimant");
        let m = msg(dup[0]);
        assert!(m.contains("same-id"), "{m}");
        assert!(m.contains("patterns/a.md") && m.contains("patterns/b.md"), "both claimants must be traceable: {m}");
        assert!(!m.contains("patterns/c.md"), "the unique id must not be dragged in: {m}");
        assert!(report.ok, "a duplicate id alone stays green un-strict");
    }

    // ═══ profile ERRORS (G14 layer 3 / cell f3-3) ══════════════════════════

    /// Node: 'profile ERROR: duplicate bee.authoritative_for FAILS the chain'
    /// (l.337) + 'grouped by the HARDENED subject' (l.352). The chain runs
    /// `knowledge check` WITHOUT --strict, so this must be an error, not a
    /// warning promoted only under strict.
    #[test]
    fn duplicate_authoritative_for_is_a_chain_failing_error_over_the_hardened_subject() {
        // Control: two DIFFERENT subjects stay green — the grouping is not a
        // blanket "two claims anywhere" rule.
        {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!("gates")));
            put(&dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!("locks")));
            let report = check_bundle(&dir, false).unwrap();
            assert!(report.profile_errors.is_empty(), "distinct subjects: {:?}", report.profile_errors);
            assert!(report.ok);
        }
        // Every ASCII spelling that normalizeSubject folds onto "gates".
        for second in ["gates", "gates.", "  GATES  ", "Gates!", "GATES---"] {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!("gates")));
            put(&dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!(second)));
            let report = check_bundle(&dir, false).unwrap();
            let dup = of_code(&report.profile_errors, "duplicate_authoritative_for");
            assert_eq!(dup.len(), 1, "{second:?}: exact-string grouping misses this, hardened grouping must not: {:?}", report.profile_errors);
            assert_eq!(dup[0]["file"], "areas/x/one.md");
            let m = msg(dup[0]);
            assert!(m.contains("areas/x/one.md") && m.contains("areas/x/two.md"), "{second:?}: both claimants must be named: {m}");
            assert!(m.contains("\"gates\""), "{second:?}: the RAW claim is quoted, not the normalized key: {m}");
            assert!(
                !report.ok,
                "{second:?}: a forked subject must fail the chain with no --strict"
            );
            assert!(
                of_code(&report.warnings, "duplicate_authoritative_for").is_empty(),
                "{second:?}: promoted to profile.errors, not duplicated across buckets"
            );
        }
    }

    /// Node's hardened grouping folds NFKC + confusables (l.352: Cyrillic 'а',
    /// fullwidth). This port used to model only the ASCII-identity slice and
    /// DELEGATE a non-ASCII claim; it now answers natively, so a homoglyph can
    /// no longer buy a second authority for an already-owned subject.
    #[test]
    fn a_homoglyph_authority_claim_is_caught_natively_as_a_duplicate() {
        // Every non-ASCII spelling that normalizeSubject folds onto "gates".
        for second in [
            "g\u{430}tes",                                   // Cyrillic 'а'
            "\u{ff47}\u{ff41}\u{ff54}\u{ff45}\u{ff53}",      // fullwidth
            "\u{1d420}ates",                                 // math bold 𝐠
            "G\u{430}TES.",                                  // homoglyph + case + punctuation
            "g\u{3b1}t\u{435}s",                             // Greek 'α' + Cyrillic 'е'
        ] {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!("gates")));
            put(&dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!(second)));
            let report = check_bundle(&dir, false)
                .unwrap_or_else(|| panic!("{second:?}: a non-ASCII claim must be ANSWERED, not delegated"));
            let dup = of_code(&report.profile_errors, "duplicate_authoritative_for");
            assert_eq!(dup.len(), 1, "{second:?}: {:?}", report.profile_errors);
            assert_eq!(dup[0]["file"], "areas/x/one.md");
            let m = msg(dup[0]);
            assert!(m.contains("areas/x/one.md") && m.contains("areas/x/two.md"), "{second:?}: {m}");
            assert!(m.contains(&format!("\"{second}\"")), "{second:?}: the RAW claim is quoted: {m}");
            assert!(!report.ok, "{second:?}: a forked subject must fail the chain");
        }

        // A diacritic is likewise not identity — and the control beside it: a
        // genuinely different subject in another script is NOT a duplicate.
        for (a, b, is_dup) in [
            ("caf\u{e9}", "cafe", true),
            ("caf\u{e9}", "cafe\u{301}", true),
            ("gates", "\u{448}\u{43b}\u{44e}\u{437}\u{44b}", false),
        ] {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!(a)));
            put(&dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!(b)));
            let report = check_bundle(&dir, false).expect("answered natively");
            assert_eq!(
                of_code(&report.profile_errors, "duplicate_authoritative_for").len(),
                usize::from(is_dup),
                "{a:?} vs {b:?}"
            );
        }
    }

    /// Node: 'profile ERROR: a MALFORMED bee.authoritative_for is a
    /// chain-failing error naming the file, never a silent skip' (l.369). The
    /// reachable set is measured against the D12 parser: `42`/`null` parse as
    /// STRINGS, and a mapping is already an unparseable_frontmatter OKF error.
    #[test]
    fn malformed_authoritative_for_is_a_chain_failing_error_naming_the_got_type() {
        for (literal, got) in [
            (json!(["gates", "locks"]), "array"),
            (json!(true), "boolean"),
            (json!(""), "string"),
            (json!("   "), "string"),
        ] {
            let (_tmp, dir) = bundle();
            put(&dir, "areas/x/bad.md", Cx::new("x-bad").ty("bee.area").bee("authoritative_for", literal.clone()));
            let report = check_bundle(&dir, false).unwrap();
            assert!(
                report.okf_errors.is_empty(),
                "{literal}: the frontmatter itself parses — this is a profile fault, not an OKF one: {:?}",
                report.okf_errors
            );
            let bad = of_code(&report.profile_errors, "malformed_authoritative_for");
            assert_eq!(bad.len(), 1, "{literal}: {:?}", report.profile_errors);
            assert_eq!(bad[0]["file"], "areas/x/bad.md");
            assert!(msg(bad[0]).contains(&format!("(got {got})")), "{literal}: {}", msg(bad[0]));
            assert!(!report.ok, "{literal}: a claim bee cannot read must fail the chain");
        }
        // Control: a well-formed claim produces neither finding.
        let (_tmp, dir) = bundle();
        put(&dir, "areas/x/good.md", Cx::new("x-good").ty("bee.area").bee("authoritative_for", json!("gates")));
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.profile_errors.is_empty(), "{:?}", report.profile_errors);
        assert!(report.ok);
    }

    // ═══ --strict (D4/D13) ═════════════════════════════════════════════════

    /// Node: 'strict flip: a warnings-only bundle is ok un-strict and not ok
    /// under strict' (l.473) + 'CLI (f3-3): a duplicated authority exits
    /// NON-ZERO with no --strict' (l.511). `run_check` turns `!report.ok`
    /// straight into the exit code (l.1815/1862), so `ok` IS the exit-code
    /// contract at this level.
    #[test]
    fn strict_flips_a_warnings_only_bundle_but_an_authority_error_fails_without_it() {
        let (_tmp, warn_dir) = bundle();
        put(&warn_dir, "patterns/guide.md", Cx::new("guide-1").ty("bee.guide"));
        let loose = check_bundle(&warn_dir, false).unwrap();
        assert!(loose.okf_errors.is_empty() && loose.profile_errors.is_empty());
        assert!(!loose.warnings.is_empty(), "the fixture must actually warn or the flip proves nothing");
        assert!(loose.ok, "un-strict passes on warnings only");
        assert!(!check_bundle(&warn_dir, true).unwrap().ok, "--strict fails on any finding");

        // Control: strict must not invent a failure on a clean bundle.
        let (_tmp2, clean_dir) = bundle();
        put(&clean_dir, "patterns/clean.md", Cx::new("clean"));
        assert!(check_bundle(&clean_dir, false).unwrap().ok);
        assert!(check_bundle(&clean_dir, true).unwrap().ok, "--strict is a warning promoter, not a new check");

        // A forked authority is non-zero WITHOUT --strict.
        let (_tmp3, dup_dir) = bundle();
        put(&dup_dir, "areas/x/one.md", Cx::new("x-one").ty("bee.area").bee("authoritative_for", json!("gates")));
        put(&dup_dir, "areas/x/two.md", Cx::new("x-two").ty("bee.area").bee("authoritative_for", json!("gates")));
        let dup = check_bundle(&dup_dir, false).unwrap();
        assert!(dup.warnings.is_empty(), "nothing in this bundle is a mere warning: {:?}", dup.warnings);
        assert_eq!(codes(&dup.profile_errors), vec!["duplicate_authoritative_for"]);
        assert!(!dup.ok, "the fork fails the chain with the flag absent");
    }

    // ═══ round-trip guard: not_canonical (D12) ═════════════════════════════

    /// Node: round-trip guard for an unquoted colon (l.416), a mid-value '#'
    /// (l.429) and CRLF (l.441), plus 'a fully canonical bundle yields zero
    /// not_canonical warnings' (l.452). Each bend must keep the DATA intact
    /// and warn — a silent misparse is the failure this guard exists to stop.
    #[test]
    fn not_canonical_warns_on_bent_bytes_and_a_canonical_bundle_warns_zero_times() {
        let canonical = Cx::new("bent").text();
        for (rel, bent, expected_title) in [
            (
                "patterns/colon.md",
                canonical.replace("title: A demo pattern", "title: Routing: the golden rule"),
                "Routing: the golden rule",
            ),
            (
                "patterns/hash.md",
                canonical.replace("title: A demo pattern", "title: value # not a comment"),
                "value # not a comment",
            ),
            ("patterns/crlf.md", canonical.replace('\n', "\r\n"), "A demo pattern"),
        ] {
            let (_tmp, dir) = bundle();
            write_bundle_file(&dir, rel, &bent);
            let report = check_bundle(&dir, false).unwrap();
            assert!(report.okf_errors.is_empty(), "{rel}: bent bytes are a profile warning, never an OKF error: {:?}", report.okf_errors);
            let warns = of_code(&report.warnings, "not_canonical");
            assert_eq!(warns.len(), 1, "{rel}: {:?}", report.warnings);
            assert_eq!(warns[0]["file"], rel);
            // The value survived the bend intact — never comment-stripped,
            // never truncated at the colon.
            let data = parse_ok(&bent);
            assert_eq!(data["title"], json!(expected_title), "{rel}");
            assert_eq!(data["bee"]["id"], json!("bent"), "{rel}");
        }
        // Control: the same concept, unbent, warns zero times.
        let (_tmp, dir) = bundle();
        write_bundle_file(&dir, "patterns/clean.md", &canonical);
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.warnings.is_empty(), "a canonical file must not warn: {:?}", report.warnings);
        assert!(report.ok);
    }

    // ═══ knowledge index (D21) ═════════════════════════════════════════════

    /// makeIndexFixture (test_knowledge.mjs l.573): nested dirs, one critical,
    /// one plain, and a log.md with an ISO heading.
    fn index_fixture(dir: &Path) {
        put(
            dir,
            "areas/demo/overview.md",
            Cx::new("demo-overview")
                .ty("bee.area")
                .title("Demo overview")
                .description("Overview of the demo area")
                .areas(&["routing"])
                .bee("authoritative_for", json!("demo-overview")),
        );
        put(
            dir,
            "areas/demo/rules.md",
            Cx::new("demo-rules")
                .ty("bee.area")
                .title("Demo rules")
                .description("Rules of the demo area")
                .lifecycle("draft")
                .areas(&["routing"])
                .bee("authoritative_for", json!("demo-rules")),
        );
        put(
            dir,
            "patterns/critical-one.md",
            Cx::new("critical-one").title("A critical pattern").description("Always in context").critical(),
        );
        put(dir, "patterns/plain.md", Cx::new("plain-one").title("A plain pattern").description("Not critical"));
        write_bundle_file(dir, "log.md", "# Log\n\n## 2026-07-22\n\n- Fixture bundle created.\n");
    }

    /// Node: 'index generates an index at every level ... two consecutive runs
    /// are byte-identical, LF-only' (l.594) + 'index --check exits non-zero
    /// naming a doctored stale index; regeneration heals it' (l.614).
    ///
    /// `run_index --check` (l.1877-1884) is `compute_index_files` plus a
    /// read-and-compare per file; it cannot be entered without a process cwd,
    /// so both production halves are driven directly and only the three-line
    /// join is written here.
    #[test]
    fn index_check_flags_exactly_the_doctored_index_and_regeneration_heals_it() {
        let (_tmp, dir) = bundle();
        index_fixture(&dir);
        let expected = compute_index_files(&dir).unwrap();
        let rels: Vec<&str> = expected.iter().map(|(r, _)| r.as_str()).collect();
        assert_eq!(rels, vec!["index.md", "areas/index.md", "areas/demo/index.md", "patterns/index.md"]);
        for (rel, content) in &expected {
            assert!(!content.contains('\r'), "{rel}: a generated index is LF-only");
            let has_clock = content.as_bytes().windows(8).any(|w| {
                w[0].is_ascii_digit()
                    && w[1].is_ascii_digit()
                    && w[2] == b':'
                    && w[3].is_ascii_digit()
                    && w[4].is_ascii_digit()
                    && w[5] == b':'
                    && w[6].is_ascii_digit()
                    && w[7].is_ascii_digit()
            });
            assert!(!has_clock, "{rel}: a generated index carries no wall-clock value");
        }
        assert_eq!(compute_index_files(&dir).unwrap(), expected, "recomputation must be byte-identical");

        // Render, exactly as run_index's write loop does.
        for (rel, content) in &expected {
            write_bundle_file(&dir, rel, content);
        }
        let stale = |exp: &Vec<(String, String)>| -> Vec<String> {
            exp.iter()
                .filter(|(rel, content)| read_file_lossy(&join_rel(&dir, rel)).ok().as_deref() != Some(content.as_str()))
                .map(|(rel, _)| format!("docs/knowledge/{rel}"))
                .collect()
        };
        assert!(stale(&expected).is_empty(), "a fresh render is not stale: {:?}", stale(&expected));

        // Doctor exactly one index. index.md is a reserved basename, so the
        // expected SET must not move — only that one file goes stale.
        let doctored = join_rel(&dir, "areas/demo/index.md");
        let bent = format!("{}\nHand-edited drift.\n", read_file_lossy(&doctored).unwrap());
        std::fs::write(&doctored, &bent).unwrap();
        let after = compute_index_files(&dir).unwrap();
        assert_eq!(after, expected, "a hand-edited index is not a concept and cannot change the expected set");
        assert_eq!(stale(&after), vec!["docs/knowledge/areas/demo/index.md"]);

        // Regeneration heals it.
        for (rel, content) in &after {
            write_bundle_file(&dir, rel, content);
        }
        assert!(stale(&after).is_empty(), "regeneration must clear the drift: {:?}", stale(&after));
    }

    /// Node: 'generated non-root indexes carry NO frontmatter — only the HTML
    /// provenance comment' (l.631) + 'generated root index keeps
    /// okf_version-only frontmatter ... and the generated bundle passes
    /// knowledge check' (l.644).
    #[test]
    fn generated_indexes_obey_the_okf_frontmatter_rules_and_pass_check() {
        let (_tmp, dir) = bundle();
        index_fixture(&dir);
        let expected = compute_index_files(&dir).unwrap();
        for (rel, content) in &expected {
            if rel == "index.md" {
                let Fm::Parsed { data, .. } = parse_frontmatter(content) else {
                    panic!("the root index must carry frontmatter");
                };
                assert_eq!(
                    data.keys().map(String::as_str).collect::<Vec<_>>(),
                    vec!["okf_version"],
                    "root index frontmatter carries ONLY okf_version"
                );
                assert_eq!(data["okf_version"], json!("0.1"));
            } else {
                assert!(matches!(parse_frontmatter(content), Fm::Absent), "{rel}: a non-root index must carry no frontmatter");
                assert!(content.starts_with("<!--"), "{rel}: must open with the provenance comment");
            }
            // PINNED PROSE: D21 makes the provenance header part of the
            // artifact's contract (the Node oracle asserts it at l.639-640),
            // so these two strings are asserted deliberately.
            assert!(content.contains("GENERATED FILE — do not hand-edit"), "{rel}");
            assert!(content.contains("bee knowledge index"), "{rel}: the regenerate command must be named");
        }

        // Rendered into the bundle, the generated indexes keep check green —
        // the control proving check_index_file agrees with what index emits.
        for (rel, content) in &expected {
            write_bundle_file(&dir, rel, content);
        }
        let report = check_bundle(&dir, false).unwrap();
        assert!(report.okf_errors.is_empty(), "generated indexes must produce zero OKF errors: {:?}", report.okf_errors);
        assert!(report.ok);

        // And the control proving check_index_file BITES: hand-added
        // frontmatter on a NON-root index is an OKF error.
        let patterns = expected.iter().find(|(r, _)| r == "patterns/index.md").unwrap();
        write_bundle_file(&dir, "patterns/index.md", &format!("---\nokf_version: 0.1\n---\n{}", patterns.1));
        let bent = check_bundle(&dir, false).unwrap();
        assert_eq!(codes(&bent.okf_errors), vec!["index_frontmatter"]);
        assert!(!bent.ok);
    }

    // ═══ knowledge context: the relevance-cut invariants (G5/G11) ══════════

    fn built(dir: &Path, work: &str, budget: f64) -> Value {
        match build_context_manifest(dir, work, budget, &json_raw(&format!("{budget}"))) {
            ManifestOut::Built(m) => m,
            ManifestOut::Thrown(m) => panic!("unexpected throw: {m}"),
            ManifestOut::NeedsNode => panic!("unexpected delegation"),
        }
    }

    fn entry_paths(manifest: &Value) -> Vec<String> {
        manifest["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["path"].as_str().unwrap().to_string())
            .collect()
    }

    fn str_list(v: &Value) -> Vec<String> {
        v.as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_string()).collect()
    }

    /// Node: 'knowledge context CONSERVES the critical set: entries +
    /// truncated + excluded == every bee.critical concept, no duplicates'
    /// (l.1360). buildContextManifest carries its own conservation guard; this
    /// asserts the PAYLOAD accounts for the set independently of it.
    #[test]
    fn context_conserves_the_critical_set_at_every_budget() {
        let (_tmp, dir) = bundle();
        put(
            &dir,
            "work/billing/work-item.md",
            Cx::new("billing-migration")
                .ty("bee.work-item")
                .title("Migrate the billing ledger onto the invoice schema")
                .description("Move every ledger row into the new billing schema behind a coverage gate.")
                .tags(&["billing", "ledger", "migration"])
                .areas(&["billing"])
                .body("Every ledger row is migrated into the invoice schema, one migration cell per ledger table."),
        );
        let critical_names = ["ledger-rows", "coverage-gate", "schema-rollback", "kiln-firing", "estuary-silt"];
        for name in critical_names {
            put(
                &dir,
                &format!("patterns/{name}.md"),
                Cx::new(name)
                    .title(&format!("{name} guidance"))
                    .description(&format!("{name} guidance for the ledger schema migration"))
                    .tags(&["pattern"])
                    .areas(&["billing"])
                    .critical()
                    .body(&format!("{name} guidance notes, technique and maintenance for a ledger row.")),
            );
        }
        let all: Vec<String> = critical_names.iter().map(|n| format!("docs/knowledge/patterns/{n}.md")).collect();

        for budget in [100000.0, 900.0, 0.0] {
            let manifest = built(&dir, "billing-migration", budget);
            let mut accounted: Vec<String> = entry_paths(&manifest);
            accounted.extend(str_list(&manifest["truncated"]));
            accounted.extend(
                manifest["excluded"].as_array().unwrap().iter().map(|e| e["path"].as_str().unwrap().to_string()),
            );
            accounted.retain(|p| all.contains(p));
            let unique: HashSet<&String> = accounted.iter().collect();
            assert_eq!(unique.len(), accounted.len(), "budget {budget}: a critical is accounted for exactly ONCE: {accounted:?}");
            assert_eq!(
                unique.len(),
                all.len(),
                "budget {budget}: CONSERVATION FAILED — {} criticals exist, {} accounted for",
                all.len(),
                unique.len()
            );
            assert_eq!(manifest["critical_total"], json!(all.len()), "budget {budget}: critical_total states the full population");
        }
    }

    /// Node: 'knowledge context FLOOR: the highest-scoring critical survives a
    /// budget that the plain prefix cut would have evicted it under' (l.1376).
    #[test]
    fn context_floor_keeps_the_top_criticals_a_plain_prefix_cut_would_evict() {
        let (_tmp, dir) = bundle();
        put(
            &dir,
            "work/billing/work-item.md",
            Cx::new("billing-migration")
                .ty("bee.work-item")
                .title("Migrate the billing ledger onto the invoice schema")
                .description("Move every ledger row into the new billing schema behind a coverage gate.")
                .tags(&["billing", "ledger", "migration"])
                .areas(&["billing"])
                .bee(
                    "required_context",
                    json!([
                        "areas/billing/ledger-schema.md",
                        "areas/billing/invoice-rows.md",
                        "areas/billing/rollback-runbook.md"
                    ]),
                )
                .body("Every ledger row is migrated into the invoice schema, one migration cell per ledger table."),
        );
        // The required_context chain is deliberately far larger than the floor:
        // under a plain prefix cut it eats the whole budget and every critical
        // is evicted, which is the failure the floor exists to stop.
        for name in ["ledger-schema", "invoice-rows", "rollback-runbook"] {
            put(
                &dir,
                &format!("areas/billing/{name}.md"),
                Cx::new(name)
                    .ty("bee.area")
                    .title(&format!("The {name}"))
                    .description(&format!("{name} reference"))
                    .tags(&["billing"])
                    .areas(&["billing"])
                    .body(&format!("{name} reference material. ").repeat(60)),
            );
        }
        for name in ["rel-ledger-rows", "rel-coverage-gate", "rel-schema-rollback", "irr-kiln-firing"] {
            put(
                &dir,
                &format!("patterns/{name}.md"),
                Cx::new(name)
                    .title(&format!("{name} guidance"))
                    .description(&format!("{name} guidance for the ledger schema migration"))
                    .tags(&["pattern"])
                    .areas(&["billing"])
                    .critical()
                    .body(&format!("{name} guidance notes and technique for a migrated ledger row.")),
            );
        }

        let full = built(&dir, "billing-migration", 100_000.0);
        let work_path = "docs/knowledge/work/billing/work-item.md";
        assert_eq!(full["entries"][0]["path"], work_path, "rank 1 is the work item");
        let floor = str_list(&full["floor"]);
        assert_eq!(floor.len(), FLOOR, "the floor is the pinned FLOOR: {floor:?}");
        let entries = full["entries"].as_array().unwrap();
        let top_critical = entries
            .iter()
            .find(|e| e["reason"].as_str().unwrap().starts_with("critical pattern"))
            .expect("a critical must be in entries at a large budget");
        assert!(floor.contains(&top_critical["path"].as_str().unwrap().to_string()), "the highest-scoring critical is in the floor");

        let est = |path: &str| -> f64 {
            entries.iter().find(|e| e["path"] == path).unwrap()["est_tokens"].as_f64().unwrap()
        };
        let work_cost = est(work_path);
        let floor_cost: f64 = floor.iter().map(|p| est(p)).sum();
        let req_cost: f64 = entries
            .iter()
            .filter(|e| e["reason"].as_str().unwrap().contains("required_context"))
            .map(|e| e["est_tokens"].as_f64().unwrap())
            .sum();
        assert!(
            req_cost > floor_cost,
            "the fixture must make the required_context chain the thing that would evict the floor ({req_cost} vs {floor_cost})"
        );

        // Exactly the work item plus the floor.
        let tight = work_cost + floor_cost;
        let cut = built(&dir, "billing-migration", tight);
        assert!(cut["total_est"].as_f64().unwrap() <= tight, "the budget stays a hard ceiling even with a floor");
        let cut_paths = entry_paths(&cut);
        for p in &floor {
            assert!(cut_paths.contains(p), "every floor critical must survive a tight budget; {p} was evicted from {cut_paths:?}");
        }
        assert_eq!(cut_paths[0], work_path, "the work item is never displaced by its own floor");
        assert_eq!(cut_paths.len(), 1 + floor.len(), "under this budget exactly the work item and the floor survive: {cut_paths:?}");
        let truncated = str_list(&cut["truncated"]);
        assert!(
            truncated.iter().any(|p| p.contains("areas/billing/")),
            "the floor must beat the higher-ranked required_context chain: {truncated:?}"
        );
    }

    /// Node: 'knowledge context FAILS when zero_signal_count exceeds the
    /// pinned threshold' (l.1423) + 'the zero-signal guard is inert below the
    /// pinned population floor' (l.1450).
    #[test]
    fn context_zero_signal_fails_above_the_population_floor_and_is_inert_below() {
        let work = || {
            Cx::new("signalless")
                .ty("bee.work-item")
                .title("Reconcile quarterly payroll withholding")
                .description("Withholding reconciliation across payroll periods.")
                .tags(&["payroll"])
                .areas(&["payroll"])
                .body("Payroll withholding reconciliation across quarterly periods, employer contributions included.")
        };
        let void = |topic: &str, i: usize| {
            Cx::new(&format!("void-{i}"))
                .title(&format!("{topic} guidance"))
                .description(&format!("{topic} guidance notes"))
                .tags(&["unrelated"])
                .areas(&["unrelated"])
                .critical()
                .body(&format!("{topic} guidance notes, {topic} technique, {topic} maintenance."))
        };
        let topics = [
            "kubernetes ingress",
            "sourdough hydration",
            "telescope collimation",
            "bicycle derailleur",
            "harpsichord tuning",
            "glacier moraine",
            "origami tessellation",
            "submarine ballast",
            "volcanic tephra",
            "lighthouse fresnel",
            "saffron cultivation",
            "permafrost drilling",
        ];

        let (_tmp, dir) = bundle();
        put(&dir, "work/lonely/work-item.md", work());
        for (i, topic) in topics.iter().enumerate() {
            put(&dir, &format!("patterns/void-{i:02}.md"), void(topic, i));
        }
        match build_context_manifest(&dir, "signalless", 100_000.0, &json_raw("100000")) {
            ManifestOut::Thrown(m) => {
                assert!(m.contains("zero_signal"), "the typed code must lead: {m}");
                assert!(m.contains("12 of 12"), "the measured counts must flow into the failure: {m}");
                assert!(m.contains("0.5"), "the pinned ratio must be named: {m}");
            }
            ManifestOut::Built(_) => panic!("an all-zero ranking must FAIL the run"),
            ManifestOut::NeedsNode => panic!("unexpected delegation"),
        }

        // Control: the SAME zero-signal vocabulary, one critical — below
        // ZERO_SIGNAL_MIN_POPULATION the guard is inert, and the count is
        // still reported.
        let (_tmp2, small) = bundle();
        put(&small, "work/lonely/work-item.md", work());
        put(&small, "patterns/void-00.md", void(topics[0], 0));
        let manifest = built(&small, "signalless", 100_000.0);
        assert_eq!(manifest["zero_signal_count"], json!(1), "the count is still REPORTED below the floor");
        assert_eq!(manifest["critical_total"], json!(1));
    }

    /// Node: 'knowledge context: relevance ties break DETERMINISTICALLY by
    /// path, and repeat runs are byte-identical' (l.1401).
    #[test]
    fn context_relevance_ties_break_deterministically_by_path() {
        let (_tmp, dir) = bundle();
        put(
            &dir,
            "work/twins/work-item.md",
            Cx::new("twins")
                .ty("bee.work-item")
                .title("Twin ranking")
                .description("Two criticals with identical vocabulary must not flap")
                .tags(&["twin"])
                .areas(&["twin"])
                .body("Identical vocabulary twin ranking flap determinism."),
        );
        for name in ["zulu-twin", "alpha-twin"] {
            put(
                &dir,
                &format!("patterns/{name}.md"),
                Cx::new(name)
                    .title("Twin pattern")
                    .description("Identical vocabulary twin")
                    .tags(&["twin"])
                    .areas(&["twin"])
                    .critical()
                    .body("Identical vocabulary twin ranking flap determinism, word for word."),
            );
        }
        let first = built(&dir, "twins", 100_000.0);
        let second = built(&dir, "twins", 100_000.0);
        assert_eq!(
            jsjson::stringify(&first),
            jsjson::stringify(&second),
            "two runs over the same bundle must serialize identically"
        );
        let criticals: Vec<String> = first["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["reason"].as_str().unwrap().starts_with("critical pattern"))
            .map(|e| e["path"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            criticals,
            vec!["docs/knowledge/patterns/alpha-twin.md", "docs/knowledge/patterns/zulu-twin.md"],
            "tied scores must order by path"
        );
        // Control: the tie is real — both criticals carry the same score.
        let scores: Vec<&str> = first["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|e| e["reason"].as_str())
            .filter(|r| r.starts_with("critical pattern"))
            .map(|r| r.split("relevance ").nth(1).unwrap().split(',').next().unwrap())
            .collect();
        assert_eq!(scores[0], scores[1], "the fixture must actually tie or the path tie-break proves nothing");
    }

    // ═══ knowledge context: the docs/history/ fallback anchor (D1/D5/D6/D7) ═

    fn write_history(root: &Path, work: &str, name: &str, text: &str) {
        let file = root.join("docs").join("history").join(work).join(name);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }

    /// Node oracle equivalent: D6 — CONTEXT.md alone, plan.md alone, and both
    /// together all resolve; the anchor is entry rank 1 with its real,
    /// non-zero byte size, and no bee.work-item concept is required.
    #[test]
    fn history_anchor_resolves_from_whichever_of_context_and_plan_exist() {
        let (tmp, dir) = bundle();
        let root = tmp.path();
        put(
            &dir,
            "patterns/dispatch.md",
            Cx::new("p-dispatch")
                .title("Dispatch prompt assembly")
                .description("how dispatch prompts are assembled")
                .tags(&["dispatch"])
                .areas(&["dispatch"])
                .critical()
                .body("Dispatch prompts are assembled from templates and rust code."),
        );

        // CONTEXT.md only.
        write_history(root, "ctx-only", "CONTEXT.md", "# Ctx Only Context\n\nDispatch prompts assembled from templates in rust.\n");
        let m = built(&dir, "ctx-only", 20_000.0);
        assert_eq!(m["anchor"]["kind"], json!("history"));
        assert_eq!(m["anchor"]["paths"], json!(["docs/history/ctx-only/CONTEXT.md"]));
        let entries = m["entries"].as_array().unwrap();
        assert_eq!(entries[0]["path"], "docs/history/ctx-only/CONTEXT.md");
        assert_eq!(entries[0]["reason"], "history anchor");
        assert!(entries[0]["bytes"].as_u64().unwrap() > 0, "the anchor must carry its real byte size, never zero");

        // plan.md only.
        write_history(root, "plan-only", "plan.md", "# Plan Only Context\n\nDispatch prompts assembled from templates in rust.\n");
        let m = built(&dir, "plan-only", 20_000.0);
        assert_eq!(m["anchor"]["kind"], json!("history"));
        assert_eq!(m["anchor"]["paths"], json!(["docs/history/plan-only/plan.md"]));
        assert!(m["entries"][0]["bytes"].as_u64().unwrap() > 0);

        // Both present — both paths in the anchor, bytes summed over both.
        write_history(root, "both", "CONTEXT.md", "# Both Context\n\nDispatch prompts assembled from templates in rust.\n");
        write_history(root, "both", "plan.md", "# Both Plan\n\nMore dispatch prompt template detail.\n");
        let m = built(&dir, "both", 20_000.0);
        assert_eq!(m["anchor"]["kind"], json!("history"));
        assert_eq!(
            m["anchor"]["paths"],
            json!(["docs/history/both/CONTEXT.md", "docs/history/both/plan.md"])
        );
        let ctx_bytes = std::fs::metadata(root.join("docs/history/both/CONTEXT.md")).unwrap().len();
        let plan_bytes = std::fs::metadata(root.join("docs/history/both/plan.md")).unwrap().len();
        assert_eq!(m["entries"][0]["bytes"], json!(ctx_bytes + plan_bytes));
    }

    /// D5: an existing bee.work-item concept always wins over a present
    /// docs/history/ fallback, and its manifest keeps today's shape apart
    /// from the added `anchor` field.
    #[test]
    fn work_item_wins_over_a_present_history_anchor() {
        let (tmp, dir) = bundle();
        let root = tmp.path();
        put(
            &dir,
            "work/w1/item.md",
            Cx::new("w1").ty("bee.work-item").title("Widget work").description("widgets and gears"),
        );
        write_history(root, "w1", "CONTEXT.md", "# Should Not Win\n\nThis must never be the anchor.\n");

        let m = built(&dir, "w1", 20_000.0);
        assert_eq!(m["anchor"]["kind"], json!("work-item"));
        assert_eq!(m["anchor"]["paths"], json!(["docs/knowledge/work/w1/item.md"]));
        assert_eq!(m["entries"][0]["path"], "docs/knowledge/work/w1/item.md");
        assert_eq!(m["entries"][0]["reason"], "work item");
    }

    /// D7: a history anchor has no tags/areas, so a sparse-vocabulary bundle
    /// that would THROW zero_signal under a work-item anchor instead REPORTS
    /// it under a history anchor and still builds.
    #[test]
    fn history_anchor_reports_zero_signal_instead_of_throwing() {
        let void = |topic: &str, i: usize| {
            Cx::new(&format!("void-{i}"))
                .title(&format!("{topic} guidance"))
                .description(&format!("{topic} guidance notes"))
                .tags(&["unrelated"])
                .areas(&["unrelated"])
                .critical()
                .body(&format!("{topic} guidance notes, {topic} technique, {topic} maintenance."))
        };
        let topics = [
            "kubernetes ingress",
            "sourdough hydration",
            "telescope collimation",
            "bicycle derailleur",
            "harpsichord tuning",
            "glacier moraine",
            "origami tessellation",
            "submarine ballast",
            "volcanic tephra",
            "lighthouse fresnel",
            "saffron cultivation",
            "permafrost drilling",
        ];
        let (tmp, dir) = bundle();
        let root = tmp.path();
        for (i, topic) in topics.iter().enumerate() {
            put(&dir, &format!("patterns/void-{i:02}.md"), void(topic, i));
        }
        write_history(root, "quiet-close", "CONTEXT.md", "# Quiet Close\n\nQuarterly payroll withholding reconciliation.\n");

        match build_context_manifest(&dir, "quiet-close", 100_000.0, &json_raw("100000")) {
            ManifestOut::Built(m) => {
                assert_eq!(m["anchor"]["kind"], json!("history"));
                assert_eq!(m["zero_signal_count"], json!(12), "the count is still REPORTED under a history anchor");
            }
            ManifestOut::Thrown(msg) => panic!("a history anchor must report zero_signal, never throw it: {msg}"),
            ManifestOut::NeedsNode => panic!("unexpected delegation"),
        }
    }
