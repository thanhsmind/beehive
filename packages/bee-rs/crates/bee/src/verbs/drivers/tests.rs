// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's inline module, indentation
// and all: a moved inline module is the same child of the same parent,
// so no path changes, and the fixtures inside are raw strings whose
// leading whitespace is content.

// The parent module's own `use` block travels with the tests: they reach
// for names mod.rs no longer imports now that the code using them lives
// in sibling modules.
#![allow(unused_imports)]

use crate::fsutil::{ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::reservations::{
    finish, js_is_ws, parse_flags, prelude, pseudo_uuid_v4, truthy, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{
    release_reservations_for_agent, reserve_path_atomic, Err2, ReserveOutcome,
};
use serde_json::{Map, Number, Value};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;
    use super::*;
    use serde_json::json;

    // ── fixtures ───────────────────────────────────────────────────────────

    fn w(root: &Path, rel: &str, body: &str) {
        let file = rel.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, body).unwrap();
    }

    fn repo(tmp: &tempfile::TempDir, config: &str) -> PathBuf {
        let root = tmp.path().to_path_buf();
        w(&root, ".bee/onboarding.json", "{\"version\":1}");
        w(&root, ".bee/config.json", config);
        root
    }

    // ── CUTOVER: corrupt JSON on a read path ───────────────────────────────

    /// `rj` (readJson(file, null)) used to answer Delegate on a corrupt file,
    /// which sent the whole command to Node. It now warns and hands back the
    /// `null` fallback, so a corrupt cell reads exactly like an absent one —
    /// which is what Node's own `!cell` guard did with that same fallback.
    #[test]
    fn a_corrupt_cell_reads_as_absent_instead_of_delegating() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(&root, ".bee/cells/c-1.json", r#"{"id":"c-1","status":"open"}"#);
        assert_eq!(read_cell(&root, "c-1").unwrap().unwrap()["status"], json!("open"));

        w(&root, ".bee/cells/c-2.json", "{broken");
        assert_eq!(read_cell(&root, "c-2").unwrap(), None, "corrupt reads as absent");
        // A truly absent cell answers the same thing — that is the point.
        assert_eq!(read_cell(&root, "c-3").unwrap(), None);
        // …and the readable sibling is untouched by its neighbour.
        assert_eq!(read_cell(&root, "c-1").unwrap().unwrap()["status"], json!("open"));
    }

    /// The lone-surrogate class in a quoted FRONTMATTER scalar: V8's
    /// JSON.parse decoded it, serde cannot, and no Rust String can hold it.
    /// It used to send the whole command to Node; it is now the same typed
    /// `bad_quoted_string` failure any other undecodable scalar produces.
    #[test]
    fn a_lone_surrogate_frontmatter_scalar_is_a_typed_failure_not_a_delegation() {
        let raw = format!("{}{}{}", "\"\\", "uD800", "\"");
        match crate::verbs::knowledge::parse_scalar_token(&raw, 7) {
            Err(crate::verbs::knowledge::Fm::Failed { code, line, .. }) => {
                assert_eq!(code, "bad_quoted_string");
                assert_eq!(line, 7);
            }
            _ => panic!("expected the typed bad_quoted_string failure"),
        }
    }

    // ── C4: the prompt byte-identity pin ───────────────────────────────────

    /// The compiled-in templates MUST be the checked-out prompts/*.md bytes.
    /// This is contract C4 restated for the Rust runtime: edit a prompt file
    /// and the binary must be rebuilt; break the include_str! paths and this
    /// fails.
    #[test]
    fn embedded_prompts_are_the_checked_out_files_byte_for_byte() {
        let prompts = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("bee")
            .join("prompts");
        for (name, embedded) in [
            ("worker-cell", PROMPT_WORKER_CELL),
            ("gather", PROMPT_GATHER),
            ("reviewer", PROMPT_REVIEWER),
            ("advisor", PROMPT_ADVISOR),
        ] {
            let disk = std::fs::read(prompts.join(format!("{name}.md")))
                .unwrap_or_else(|e| panic!("prompts/{name}.md unreadable: {e}"));
            assert_eq!(
                embedded.as_bytes(),
                disk.as_slice(),
                "prompts/{name}.md drifted from the compiled-in copy"
            );
        }
    }

    /// loadPrompt's normalization: CRLF -> LF, exactly ONE trailing newline
    /// stripped (a template ending in two newlines keeps one).
    #[test]
    fn load_prompt_normalizes_line_endings_like_node() {
        assert_eq!(normalize_template("a\r\nb\r\n"), "a\nb");
        assert_eq!(normalize_template("a\nb\n\n"), "a\nb\n");
        assert_eq!(normalize_template("a\nb"), "a\nb");
        // The real worker-cell template ends with `{{/if}}\n` -> `{{/if}}`.
        assert!(load_prompt("worker-cell").unwrap().ends_with("{{/if}}"));
        assert!(!load_prompt("gather").unwrap().ends_with('\n'));
    }

    /// The `{{#if}}` block grammar: a dropped block leaves ZERO residue bytes
    /// (the preceding newline goes with it), a kept block splices its inner
    /// lines in exactly.
    #[test]
    fn render_conditional_blocks_leave_no_residue() {
        let t = "head\n{{#if x}}\nkept {{v}}\n{{/if}}\ntail";
        assert_eq!(render(t, &[("x", ""), ("v", "V")]).unwrap(), "head\ntail");
        assert_eq!(
            render(t, &[("x", "yes"), ("v", "V")]).unwrap(),
            "head\nkept V\ntail"
        );
    }

    #[test]
    fn render_refuses_nesting_and_missing_placeholders() {
        let nested = "a\n{{#if x}}\n{{#if y}}\nz\n{{/if}}\n{{/if}}\nb";
        let err = render(nested, &[("x", "1"), ("y", "1")]).unwrap_err();
        assert!(err.contains("nested"), "{err}");
        let err = render("a {{who}} b", &[]).unwrap_err();
        assert_eq!(
            err,
            "prompt-renderer: no value supplied for placeholder {{who}}."
        );
    }

    /// A first-dispatch cell with neither machine block renders byte-identically
    /// to the unconditional template — the invariant the C4 pin protects.
    #[test]
    fn worker_cell_without_machine_blocks_matches_the_plain_template() {
        let template = load_prompt("worker-cell").unwrap();
        let rendered = render(
            &template,
            &[
                ("worker", "w"),
                ("cell_id", "c-1"),
                ("feature", "f"),
                ("cell_json", "{}"),
                ("learned_context", ""),
                ("prior_rounds", ""),
            ],
        )
        .unwrap();
        assert!(!rendered.contains("Learned context"));
        assert!(!rendered.contains("Prior rounds"));
        assert!(rendered.starts_with("Nickname (reservation identity): w\n"));
        // Zero residue: both block markers and their newlines are gone.
        assert!(!rendered.contains("{{"));
        assert!(rendered.contains("- docs/history/f/plan.md (when present)\n\nContract:\n"));
    }

    // ── oneLine ────────────────────────────────────────────────────────────

    #[test]
    fn one_line_collapses_whitespace_and_ellipsises() {
        assert_eq!(one_line(Some(&json!("  a \n b  ")), 140), "a b");
        assert_eq!(one_line(None, 140), "");
        assert_eq!(one_line(Some(&Value::Null), 140), "");
        assert_eq!(one_line(Some(&json!(42)), 140), "42");
        let long = "x".repeat(60);
        assert_eq!(
            one_line(Some(&json!(long)), 40),
            format!("{}...", "x".repeat(37))
        );
    }

    // ── prior rounds ───────────────────────────────────────────────────────

    #[test]
    fn prior_rounds_orders_chronologically_and_skips_passes() {
        let cell = json!({
            "id": "c-1",
            "trace": {
                "capped_at": "2026-01-05T00:00:00.000Z",
                "attempts": [
                    {"at": "2026-01-03T00:00:00.000Z", "worker": "w2", "verdict": "tests-red", "note": "AssertionError: 1 != 2"},
                    {"at": "2026-01-01T00:00:00.000Z", "worker": "w0", "verdict": "fail", "failure_signature": "sig"},
                    {"at": "2026-01-04T00:00:00.000Z", "worker": "w3", "verdict": "pass"},
                    {"at": "2026-01-02T00:00:00.000Z", "verdict": "blocked"}
                ],
                "deviations": ["renamed the helper", "  "],
                "semantic_judge": [{"recorded_at": "2026-01-06T00:00:00.000Z", "verdict": "NEEDS_REVISION"}]
            }
        });
        assert_eq!(
            prior_round_event_lines(&cell),
            vec![
                "- w0 failed verify: failure signature sig",
                "- (unknown worker) blocked: failure signature (none recorded)",
                "- w2 tests red: AssertionError: 1 != 2",
                "- (prior worker) deviation: renamed the helper",
                "- (judge) consult: NEEDS_REVISION",
            ]
        );
        // A cell with no history produces NO lines (first-dispatch parity).
        assert!(prior_round_event_lines(&json!({"id": "c-2"})).is_empty());
    }

    #[test]
    fn prior_rounds_elides_the_oldest_past_twelve() {
        let attempts: Vec<Value> = (1..=15)
            .map(|i| json!({"at": format!("2026-01-{i:02}T00:00:00.000Z"), "worker": format!("w{i}"), "verdict": "blocked", "note": "n"}))
            .collect();
        let lines = prior_round_event_lines(&json!({"trace": {"attempts": attempts}}));
        assert_eq!(lines.len(), PRIOR_ROUNDS_MAX_EVENT_LINES);
        assert_eq!(
            lines[0],
            "- (4 earlier event(s) elided — the cell record holds the rest)"
        );
        assert_eq!(lines[1], "- w5 blocked: n");
        assert_eq!(lines[11], "- w15 blocked: n");
    }

    #[test]
    fn timeless_events_sink_to_the_end_in_insertion_order() {
        let cell = json!({"trace": {"attempts": [
            {"worker": "no-ts", "verdict": "blocked", "note": "a"},
            {"at": "2026-01-01T00:00:00.000Z", "worker": "dated", "verdict": "blocked", "note": "b"}
        ]}});
        assert_eq!(
            prior_round_event_lines(&cell),
            vec!["- dated blocked: b", "- no-ts blocked: a"]
        );
    }

    // ── claim-ownership guard ──────────────────────────────────────────────

    #[test]
    fn claim_ownership_refusals_are_byte_faithful() {
        let open = json!({"id": "c-1", "status": "open"});
        let o = check_cell_claim_ownership(&open, "w");
        assert!(!o.ok);
        assert_eq!(o.code, Some("not_claimed"));
        assert_eq!(o.reason, "cell \"c-1\" is \"open\", not \"claimed\" — dispatch prepare requires a claimed cell (run bee cells claim or bee cells claim-next first). Pass --force-ownership to override (audited).");

        let foreign = json!({"id": "c-1", "status": "claimed", "trace": {"worker": "other"}});
        let o = check_cell_claim_ownership(&foreign, "w");
        assert_eq!(o.code, Some("not_owner"));
        assert_eq!(o.owner, json!("other"));
        assert_eq!(o.reason, "cell \"c-1\" is claimed by worker \"other\" — \"w\" does not own this claim. Pass --force-ownership to override (audited).");

        // A claimed cell with no trace.worker reads "(unknown)".
        let orphan = json!({"id": "c-1", "status": "claimed"});
        assert!(check_cell_claim_ownership(&orphan, "w")
            .reason
            .contains("worker \"(unknown)\""));

        let mine = json!({"id": "c-1", "status": "claimed", "trace": {"worker": "w"}});
        assert!(check_cell_claim_ownership(&mine, "w").ok);
    }

    // ── tier resolution ────────────────────────────────────────────────────

    fn models_from(raw: &str) -> Map<String, Value> {
        normalize_models(Some(&serde_json::from_str::<Value>(raw).unwrap()))
    }

    #[test]
    fn resolve_tier_covers_every_documented_slot_shape() {
        let m = models_from("{}");
        // Defaults (DEFAULT_MODELS).
        assert_eq!(
            resolve_tier(&m, "generation", "claude", "cell"),
            Resolved::Model { model: "sonnet".into(), effort: None }
        );
        assert_eq!(
            resolve_tier(&m, "review", "claude", "gather"),
            Resolved::Model { model: "opus".into(), effort: None }
        );
        // codex defaults are null -> budget; review falls back to generation
        // (also null) -> budget.
        assert_eq!(resolve_tier(&m, "generation", "codex", "cell"), Resolved::Budget);
        assert_eq!(resolve_tier(&m, "review", "codex", "gather"), Resolved::Budget);
        // ceiling is never configured.
        assert_eq!(resolve_tier(&m, "ceiling", "claude", "cell"), Resolved::Inherit);
        // An unknown slot ('advisor') coerces to generation — the trap
        // resolveAdvisor exists to avoid.
        assert_eq!(
            resolve_tier(&m, "advisor", "claude", "gather"),
            Resolved::Model { model: "sonnet".into(), effort: None }
        );

        // review: null falls back to the generation tier BEFORE the cli check.
        let m = models_from(
            r#"{"claude":{"generation":{"kind":"cli","command":"glm run"},"review":null}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "review", "claude", "gather"),
            Resolved::Cli { command: "glm run".into() }
        );
        // cli + cell purpose -> typed refusal naming the RESOLVED slot.
        assert_eq!(
            resolve_tier(&m, "review", "claude", "cell"),
            Resolved::Refused { slot: "review".into() }
        );

        // {model, effort}
        let m = models_from(r#"{"claude":{"generation":{"model":"opus","effort":"high"}}}"#);
        assert_eq!(
            resolve_tier(&m, "generation", "claude", "cell"),
            Resolved::Model { model: "opus".into(), effort: Some("high".into()) }
        );
        // An invalid effort is dropped by normalize.
        let m = models_from(r#"{"claude":{"generation":{"model":"opus","effort":"turbo"}}}"#);
        assert_eq!(
            resolve_tier(&m, "generation", "claude", "cell"),
            Resolved::Model { model: "opus".into(), effort: None }
        );

        // native leaf + explicit-only cli fallback composite
        let m = models_from(
            r#"{"codex":{"generation":{"primary":{"kind":"native","model":"gpt-5","effort":"high"},"fallback_policy":"explicit-only","fallback":{"kind":"cli","command":"codex exec"}}}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "generation", "codex", "cell"),
            Resolved::Native {
                model: "gpt-5".into(),
                effort: Some("high".into()),
                fork_turns: "none".into(),
                agent_type: "worker".into(),
                fallback: Some("codex exec".into()),
            }
        );
        // Without the policy string the fallback is stripped (no silent
        // native->cli switching).
        let m = models_from(
            r#"{"codex":{"generation":{"primary":{"kind":"native","model":"gpt-5"},"fallback":{"kind":"cli","command":"codex exec"}}}}"#,
        );
        match resolve_tier(&m, "generation", "codex", "cell") {
            Resolved::Native { fallback, .. } => assert_eq!(fallback, None),
            other => panic!("expected native, got {other:?}"),
        }
    }

    // herding-review-slots D1 (widened to the full mapping): `{kind:
    // "herding"}` is a router value — EVERY purpose (cell, gather,
    // reviewer, advisor, extraction) resolves to Resolved::Herding
    // (ht-3/hrv-1/hrv-3 turn that into the herding-exec Bash payload);
    // there is no longer a gather-default split.
    #[test]
    fn normalize_tier_value_accepts_and_round_trips_the_herding_shape() {
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding"}))),
            Some(json!({"kind": "herding"}))
        );
        // No other fields are required, and unknown extras are dropped —
        // same posture as the cli/native shapes above.
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding", "command": "ignored", "extra": 1}))),
            Some(json!({"kind": "herding"}))
        );
        // A near-miss kind value is not herding — it falls through the
        // existing rules unchanged (no `model` field either, so None).
        assert_eq!(normalize_tier_value(Some(&json!({"kind": "hording"}))), None);
    }

    #[test]
    fn resolve_tier_routes_every_purpose_on_a_herding_slot_to_herding() {
        // cell purpose -> Resolved::Herding, never a refusal.
        let m = models_from(r#"{"claude":{"generation":{"kind":"herding"}}}"#);
        assert_eq!(
            resolve_tier(&m, "generation", "claude", "cell"),
            Resolved::Herding { agent: None, fallback: None }
        );

        // gather purpose on the SAME slot -> Herding too (D1 widened): the
        // gather-default-model split hrv-1 still carried is gone.
        assert_eq!(
            resolve_tier(&m, "generation", "claude", "gather"),
            Resolved::Herding { agent: None, fallback: None }
        );

        // A runtime with no configured default for the slot (codex) still
        // reads Herding on both purposes — the herding shape never
        // consults the runtime's default-model table to resolve itself.
        let m = models_from(r#"{"codex":{"generation":{"kind":"herding"}}}"#);
        assert_eq!(
            resolve_tier(&m, "generation", "codex", "gather"),
            Resolved::Herding { agent: None, fallback: None }
        );
        assert_eq!(
            resolve_tier(&m, "generation", "codex", "cell"),
            Resolved::Herding { agent: None, fallback: None }
        );

        // The extraction slot resolves exactly the same way — D1's "full
        // mapping" covers every CONFIGURABLE_SLOTS member, not just
        // generation/review.
        let m = models_from(r#"{"claude":{"extraction":{"kind":"herding"}}}"#);
        assert_eq!(
            resolve_tier(&m, "extraction", "claude", "cell"),
            Resolved::Herding { agent: None, fallback: None }
        );
        assert_eq!(
            resolve_tier(&m, "extraction", "claude", "gather"),
            Resolved::Herding { agent: None, fallback: None }
        );
    }

    // herding-review-slots D1: the widened split — reviewer AND advisor
    // purposes resolve a herding-shaped slot to Herding exactly like a cell
    // purpose does — every purpose reads the shape the same way now (D1
    // widened to the full mapping, no purpose left carrying its own rule).
    #[test]
    fn resolve_tier_routes_reviewer_and_advisor_purposes_to_herding_too() {
        let m = models_from(r#"{"claude":{"review":{"kind":"herding","agent":"agy-flash"}}}"#);
        assert_eq!(
            resolve_tier(&m, "review", "claude", "reviewer"),
            Resolved::Herding { agent: Some("agy-flash".into()), fallback: None }
        );
        // resolve_tier itself is purpose-agnostic beyond the cli-only
        // gate it already knew — an "advisor" purpose on the same shape
        // also resolves to Herding (resolve_advisor is the production path
        // for a real --kind advisor dispatch; this pins resolve_tier's own
        // symmetry so the two doors never drift apart).
        assert_eq!(
            resolve_tier(&m, "review", "claude", "advisor"),
            Resolved::Herding { agent: Some("agy-flash".into()), fallback: None }
        );
    }

    // herd-registry D2: `agent` on a kind:herding slot round-trips through
    // normalize and resolve, and prepare's herding-exec arm appends
    // `--agent "<name>"` when the slot names one.
    #[test]
    fn normalize_tier_value_round_trips_the_herding_agent_field() {
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding", "agent": "codex-cli"}))),
            Some(json!({"kind": "herding", "agent": "codex-cli"}))
        );
        // Trimmed, same as every other string field on this leaf.
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding", "agent": "  codex-cli  "}))),
            Some(json!({"kind": "herding", "agent": "codex-cli"}))
        );
        // Empty/whitespace-only agent is dropped, not carried as "".
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding", "agent": "   "}))),
            Some(json!({"kind": "herding"}))
        );
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding"}))),
            Some(json!({"kind": "herding"}))
        );
    }

    #[test]
    fn resolve_tier_carries_the_herding_agent_name_through() {
        let m = models_from(
            r#"{"claude":{"generation":{"kind":"herding","agent":"codex-cli"}}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "generation", "claude", "cell"),
            Resolved::Herding { agent: Some("codex-cli".into()), fallback: None }
        );
    }

    // herding-review-slots D3: `"fallback": "default"` on the herding shape
    // round-trips through normalize (only the literal string "default" is
    // recognized) and resolve_tier carries it through to Resolved::Herding.
    #[test]
    fn normalize_tier_value_round_trips_the_herding_fallback_field() {
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding", "fallback": "default"}))),
            Some(json!({"kind": "herding", "fallback": "default"}))
        );
        // Any other value is dropped, not carried through — no near-miss
        // spellings, same exact-match posture as `fork_turns`.
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding", "fallback": "budget"}))),
            Some(json!({"kind": "herding"}))
        );
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding", "fallback": ""}))),
            Some(json!({"kind": "herding"}))
        );
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding", "fallback": true}))),
            Some(json!({"kind": "herding"}))
        );
        // Absent stays absent.
        assert_eq!(
            normalize_tier_value(Some(&json!({"kind": "herding"}))),
            Some(json!({"kind": "herding"}))
        );
    }

    #[test]
    fn resolve_tier_carries_the_herding_fallback_field_through() {
        let m = models_from(
            r#"{"claude":{"generation":{"kind":"herding","agent":"codex-cli","fallback":"default"}}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "generation", "claude", "cell"),
            Resolved::Herding {
                agent: Some("codex-cli".into()),
                fallback: Some("default".into())
            }
        );

        // Absent -> None, unchanged from before D3.
        let m = models_from(r#"{"claude":{"generation":{"kind":"herding"}}}"#);
        assert_eq!(
            resolve_tier(&m, "generation", "claude", "cell"),
            Resolved::Herding { agent: None, fallback: None }
        );
    }

    // herding-review-slots D1: a herding-shaped advisor slot now resolves
    // to Resolved::Herding (widening herding-tier D1's cell-only scope),
    // never "no advisor" — an advisor consult is one task in, one result
    // out, the same shape as the herding-exec pane's own read-only job.
    #[test]
    fn resolve_advisor_treats_a_herding_shaped_slot_as_herding() {
        assert_eq!(
            resolve_advisor(&models_from(r#"{"claude":{"advisor":{"kind":"herding"}}}"#), "claude"),
            Some(Resolved::Herding { agent: None, fallback: None })
        );
        // herd-registry D2: `agent` carries through, same as every other
        // herding-shaped slot.
        assert_eq!(
            resolve_advisor(
                &models_from(r#"{"claude":{"advisor":{"kind":"herding","agent":"named-herd"}}}"#),
                "claude"
            ),
            Some(Resolved::Herding { agent: Some("named-herd".into()), fallback: None })
        );
    }

    #[test]
    fn resolve_advisor_never_falls_back() {
        // Unset -> None (not budget, not generation).
        assert_eq!(resolve_advisor(&models_from("{}"), "claude"), None);
        assert_eq!(
            resolve_advisor(&models_from(r#"{"claude":{"advisor":"opus"}}"#), "claude"),
            Some(Resolved::Model { model: "opus".into(), effort: None })
        );
        // An explicit null is still "no advisor".
        assert_eq!(
            resolve_advisor(&models_from(r#"{"claude":{"advisor":null}}"#), "claude"),
            None
        );
        // An unknown runtime coerces to claude.
        assert_eq!(
            resolve_advisor(&models_from(r#"{"claude":{"advisor":"opus"}}"#), "banana"),
            Some(Resolved::Model { model: "opus".into(), effort: None })
        );
    }

    // opencode-support E4/S4: `models.opencode` used to be silently dropped
    // (RUNTIMES only listed claude/codex) — docs/config-reference.md called
    // this "dead config that never resolves". It is now a real third key.
    #[test]
    fn opencode_is_a_real_runtime_key_not_silently_ignored() {
        // Unconfigured opencode defaults to Budget on every slot (same
        // no-baked-in-default treatment codex gets — no established model
        // naming convention to assume).
        let m = models_from("{}");
        assert_eq!(resolve_tier(&m, "generation", "opencode", "cell"), Resolved::Budget);
        assert_eq!(resolve_tier(&m, "extraction", "opencode", "cell"), Resolved::Budget);
        assert_eq!(resolve_tier(&m, "review", "opencode", "gather"), Resolved::Budget);

        // A configured models.opencode block resolves exactly like claude/codex do.
        let m = models_from(
            r#"{"opencode":{"extraction":"opencode/ling-3.0-tiny-free","generation":"opencode/big-pickle","review":"opencode/nemotron-3-ultra-free"}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "generation", "opencode", "cell"),
            Resolved::Model { model: "opencode/big-pickle".into(), effort: None }
        );
        assert_eq!(
            resolve_tier(&m, "extraction", "opencode", "cell"),
            Resolved::Model { model: "opencode/ling-3.0-tiny-free".into(), effort: None }
        );
        assert_eq!(
            resolve_tier(&m, "review", "opencode", "gather"),
            Resolved::Model { model: "opencode/nemotron-3-ultra-free".into(), effort: None }
        );
        // A sibling claude/codex block in the same config is untouched by
        // opencode's presence (no cross-runtime leakage).
        assert_eq!(resolve_tier(&m, "generation", "claude", "cell"), Resolved::Model { model: "sonnet".into(), effort: None });
        assert_eq!(resolve_tier(&m, "generation", "codex", "cell"), Resolved::Budget);
    }

    // ── economics ──────────────────────────────────────────────────────────

    #[test]
    fn derive_economics_matches_the_honest_mapping() {
        let model = Resolved::Model { model: "sonnet".into(), effort: None };
        let e = derive_economics("claude-agent", "generation", Some("sonnet"), &model, false);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":"sonnet","effective_model":"sonnet","effective_model_status":"pinned","channel":"claude-agent","enforcement":"model-param"}"#
        );
        // codex-native without a confirmed override is ALWAYS
        // inherited-or-unknown, whatever the tier resolves to.
        let e = derive_economics("codex-native", "generation", None, &Resolved::Budget, false);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":null,"effective_model":null,"effective_model_status":"inherited-or-unknown","channel":"codex-native","enforcement":"prompt-budget"}"#
        );
        // A confirmed native override: native-requested, effective_model still
        // null (catalog-accepted is not runtime-confirmed, D7).
        let native = Resolved::Native {
            model: "gpt-5".into(),
            effort: None,
            fork_turns: "none".into(),
            agent_type: "worker".into(),
            fallback: None,
        };
        let e = derive_economics("codex-native", "generation", None, &native, true);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":"gpt-5","effective_model":null,"effective_model_status":"native-requested","channel":"codex-native","enforcement":"native-model-param"}"#
        );
        // cli-exec never reports a requested_model.
        let cli = Resolved::Cli { command: "glm run".into() };
        let e = derive_economics("cli-exec", "generation", None, &cli, false);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":null,"effective_model":null,"effective_model_status":"unverified","channel":"cli-exec","enforcement":"cli-command"}"#
        );
        // herding-exec mirrors cli-exec, never prompt-budget.
        let e = derive_economics("herding-exec", "generation", None, &Resolved::Budget, false);
        assert_eq!(
            jsjson::stringify(&Value::Object(e)),
            r#"{"logical_tier":"generation","requested_model":null,"effective_model":null,"effective_model_status":"unverified","channel":"herding-exec","enforcement":"herding-command"}"#
        );
    }

    #[test]
    fn pinned_types_match_the_rendered_bee_agents() {
        // guard.rs's tier-only pinned_agent_type stays a tier lookup — the
        // generation/cell override lives at the prepare.rs call site
        // (cell_envelope_names_the_execution_agent_not_the_read_only_gather
        // below), since prepare.rs is this cell's only declared file.
        assert_eq!(pinned_agent_type("generation"), "bee-gather");
        assert_eq!(pinned_agent_type("extraction"), "bee-extract");
        assert_eq!(pinned_agent_type("review"), "bee-review");
        // 'advisor' has no rendered agent — `|| 'general-purpose'`.
        assert_eq!(pinned_agent_type("advisor"), "general-purpose");
    }

    // ── prepareDispatch envelopes ──────────────────────────────────────────

    #[test]
    fn gather_envelope_is_the_claude_agent_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        let out = prepare_dispatch(
            &root, "claude", "gather", None, None, false, None, None, false,
        )
        .unwrap();
        let Prepared::Value(v) = out else { panic!("expected an envelope") };
        assert_eq!(v.get("tool"), Some(&json!("Agent")));
        let p = v.get("payload").unwrap();
        assert_eq!(p.get("subagent_type"), Some(&json!("bee-gather")));
        assert_eq!(p.get("model"), Some(&json!("sonnet")));
        assert_eq!(p.get("description"), Some(&json!("gather (sonnet)")));
        // The marker anchors at the very start of the prompt.
        let prompt = p.get("prompt").unwrap().as_str().unwrap();
        assert!(prompt.starts_with("[bee-tier: generation]\nGather: locate and digest"));
        // The prepare-time record is NOT written on a non-recording pass.
        assert!(!root.join(".bee/logs/dispatch.jsonl").exists());
    }

    #[test]
    fn cell_envelope_names_the_execution_agent_not_the_read_only_gather() {
        // The generation tier carries two rendered agents; a --kind cell
        // dispatch is the one whose whole job is executing the cell (reserve,
        // write, commit, cap), so it must name bee-build, never bee-gather
        // (the read-only agent from bee-gather.md — "Never writes, never
        // edits"). dp-2.
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!("expected an envelope")
        };
        assert_eq!(v.get("tool"), Some(&json!("Agent")));
        let p = v.get("payload").unwrap();
        assert_eq!(p.get("subagent_type"), Some(&json!("bee-build")));
        assert_eq!(p.get("model"), Some(&json!("sonnet")));
        // The economics record still names the tier, not the agent, as the
        // generation-tier model authority (must-have 4).
        assert_eq!(v.get("economics").and_then(|e| e.get("logical_tier")), Some(&json!("generation")));
    }

    /// Measured from a live agent list: every row read `bee-build  Execute
    /// bbp-6` — an id and nothing else. prepare's own description was
    /// `cell (sonnet)`, which work-visibility D2 names a red flag, so nobody
    /// used it. The row now says what the work IS.
    #[test]
    fn a_cell_dispatch_description_says_what_the_work_is() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        let mut cell = |id: &str, title: &str| {
            w(
                &root,
                &format!(".bee/cells/{id}.json"),
                &format!(
                    r#"{{"id":"{id}","feature":"f","title":{},"status":"claimed","trace":{{"worker":"w"}}}}"#,
                    json!(title)
                ),
            );
        };

        cell("c-1", "cap the test scrubber");
        let d = |id: &str| {
            let Prepared::Value(v) =
                prepare_dispatch(&root, "claude", "cell", Some(id), Some("w"), false, None, None, false)
                    .unwrap()
            else {
                panic!("expected an envelope")
            };
            v.get("payload").unwrap().get("description").unwrap().as_str().unwrap().to_string()
        };
        assert_eq!(d("c-1"), "c-1: cap the test scrubber (sonnet)");

        // A whitespace title is no title: today's bytes, never `c-2:  (…)`.
        cell("c-2", "   ");
        assert_eq!(d("c-2"), "cell (sonnet)");
        w(
            &root,
            ".bee/cells/c-3.json",
            r#"{"id":"c-3","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );
        assert_eq!(d("c-3"), "cell (sonnet)");

        // Long titles are cut, so the row still reads at a glance.
        cell("c-4", &"word ".repeat(40));
        let long = d("c-4");
        assert!(long.starts_with("c-4: word word"), "{long}");
        assert!(long.ends_with(" (sonnet)"), "{long}");
        assert!(long.len() < 90, "{long}");

        // A newline in the title never breaks the one-line row.
        cell("c-5", "first line\nsecond line");
        assert_eq!(d("c-5"), "c-5: first line second line (sonnet)");
    }

    /// Gap 2 of the audit (dispatch-label-chokepoint plan.md): a non-cell
    /// kind (`gather`/`reviewer`/`advisor`) had no way to say what it was FOR
    /// — `--purpose` is that way. Given, it renders; omitted, today's exact
    /// bytes (covered by `gather_envelope_is_the_claude_agent_shape` above).
    #[test]
    fn a_non_cell_kind_with_purpose_renders_it_in_the_description() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "claude",
            "gather",
            None,
            None,
            false,
            None,
            Some("scout the auth middleware before the shape gate"),
            false,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        assert_eq!(
            v.get("payload").unwrap().get("description"),
            Some(&json!("gather: scout the auth middleware before the shape gate (sonnet)"))
        );
    }

    /// Gap 1 of the audit: codex's `task_name` carried the bare cell id
    /// (`prepare.rs:687` pre-fix), the exact case a live agent list still
    /// read as `Execute cell etom-nid-mapping-6` a month after work-
    /// visibility D2 required a real label. `task_name` now carries the same
    /// subject as the claude Agent's `description`.
    #[test]
    fn codex_task_name_carries_the_cell_title_not_the_bare_id() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"codex":{"generation":"gpt-5"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","title":"cap the test scrubber","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) = prepare_dispatch(
            &root, "codex", "cell", Some("c-1"), Some("w"), false, None, None, false,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        assert_eq!(v.get("tool"), Some(&json!("spawn_agent")));
        assert_eq!(
            v.get("payload").unwrap().get("task_name"),
            Some(&json!("c-1: cap the test scrubber"))
        );
    }

    /// THE ANTI-RECURRENCE DEVICE (dispatch-label-chokepoint plan.md — the
    /// fourth attempt at "a dispatch label says what the work is"). All four
    /// audited gaps shared one shape: a subject computed for ONE transport
    /// arm, silently absent everywhere else — codex's `task_name`, claude's
    /// `gather`/`reviewer`/`advisor` description, cli-exec's bare payload.
    /// This walks DISPATCH_RUNTIMES × DISPATCH_KINDS FROM THE CONSTANTS
    /// THEMSELVES — never a hand-written list — so a runtime or kind added
    /// later fails HERE until its payload actually carries a subject,
    /// instead of shipping unlabelled for a month like the last one did.
    #[test]
    fn every_supported_runtime_and_kind_pair_labels_its_dispatch_with_a_subject() {
        for &runtime in DISPATCH_RUNTIMES.iter() {
            for &kind in DISPATCH_KINDS.iter() {
                let tmp = tempfile::tempdir().unwrap();
                let root = repo(
                    &tmp,
                    r#"{"models":{
                        "claude":{"generation":"sonnet","advisor":"sonnet"},
                        "codex":{"generation":"gpt-5","advisor":"gpt-5"}
                    }}"#,
                );
                let model = if runtime == "claude" { "sonnet" } else { "gpt-5" };

                // kind=="cell" carries its subject through the cell record;
                // every other kind carries it through --purpose. Both are
                // exercised with REAL content — the byte-identical
                // no-cell-title / no-purpose fallback is covered separately
                // (a_cell_dispatch_description_says_what_the_work_is,
                // gather_envelope_is_the_claude_agent_shape) so it can never
                // paper over a branch that silently drops what it was given.
                let (cell_id, worker, purpose, marker) = if kind == "cell" {
                    w(
                        &root,
                        ".bee/cells/mx-1.json",
                        r#"{"id":"mx-1","feature":"f","title":"matrix subject text","status":"claimed","trace":{"worker":"w"}}"#,
                    );
                    (Some("mx-1"), Some("w"), None, "mx-1: matrix subject text".to_string())
                } else {
                    (None, None, Some("matrix subject text"), format!("{kind}: matrix subject text"))
                };

                let Prepared::Value(v) = prepare_dispatch(
                    &root,
                    runtime,
                    kind,
                    cell_id,
                    worker,
                    false,
                    None,
                    purpose,
                    false,
                )
                .unwrap() else {
                    panic!("{runtime}/{kind}: expected an envelope, not a refusal")
                };
                let payload = match v.get("payload") {
                    Some(p) => p,
                    None => panic!("{runtime}/{kind}: envelope carries no payload at all"),
                };

                // Only a pair whose payload actually carries a label field
                // owes the assertion below — a transport with none (cli-exec,
                // the recorded limit) is exempt by construction, never by a
                // hand-picked skip; under this fixture (plain string models,
                // no cli/native kind) every (runtime, kind) pair here resolves
                // to a labelled transport, so `continue` is unreached today
                // and only guards a future resolution shape.
                let label = match runtime {
                    "claude" => payload.get("description").and_then(Value::as_str),
                    "codex" => payload.get("task_name").and_then(Value::as_str),
                    _ => None,
                };
                let Some(label) = label else { continue };

                assert!(
                    label.contains(&marker),
                    "{runtime}/{kind}: label {label:?} carries no subject — expected it to contain {marker:?}"
                );
                assert_ne!(
                    label, kind,
                    "{runtime}/{kind}: label {label:?} is the bare kind, not a subject"
                );
                assert_ne!(
                    label,
                    format!("({model})"),
                    "{runtime}/{kind}: label {label:?} is only a model name"
                );
                assert_ne!(
                    label,
                    format!("{kind} ({model})"),
                    "{runtime}/{kind}: label {label:?} dropped the given subject and fell back to the bare kind"
                );
            }
        }
    }

    #[test]
    fn recording_pass_appends_exactly_one_prepare_line() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        prepare_dispatch(&root, "claude", "gather", None, None, false, None, None, true).unwrap();
        let log = std::fs::read_to_string(root.join(".bee/logs/dispatch.jsonl")).unwrap();
        assert_eq!(log.lines().count(), 1);
        let line: Value = serde_json::from_str(log.lines().next().unwrap()).unwrap();
        assert_eq!(line.get("source"), Some(&json!("prepare")));
        assert_eq!(line.get("kind"), Some(&json!("gather")));
        assert_eq!(line.get("cell"), Some(&Value::Null));
        assert_eq!(line.get("channel"), Some(&json!("claude-agent")));
        assert_eq!(line.get("enforcement"), Some(&json!("model-param")));
    }

    #[test]
    fn advisor_not_configured_is_a_typed_refusal_not_a_throw() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "advisor", None, None, false, None, None, false).unwrap()
        else {
            panic!("expected a refusal value")
        };
        assert_eq!(
            jsjson::stringify(&v),
            r#"{"ok":false,"reason":"advisor_not_configured","fix":"set models.claude.advisor in .bee/config.json to enable an advisor consult (resolveAdvisor never falls back to another tier)."}"#
        );
    }

    #[test]
    fn cli_shaped_generation_refuses_for_cell_and_serves_gather() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":{"kind":"cli","command":"glm run"}}}}"#,
        );
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("reason"), Some(&json!("cli_tier_gather_only")));
        assert_eq!(v.get("slot"), Some(&json!("generation")));
        assert_eq!(v.get("fix"), Some(&json!(CLI_REFUSAL_FIX)));

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "gather", None, None, false, None, None, false).unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("tool"), Some(&json!("Bash")));
        assert_eq!(v.get("payload").unwrap().get("command"), Some(&json!("glm run")));
        assert!(v
            .get("payload")
            .unwrap()
            .get("stdin")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("Gather:"));
    }

    // ── ht-3: herding-tier D4 — the herding-exec Bash payload ──────────────

    #[test]
    fn herding_shaped_generation_emits_the_herding_exec_bash_payload_for_a_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":{"kind":"herding"}}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("tool"), Some(&json!("Bash")));
        let payload = v.get("payload").unwrap();
        assert_eq!(
            payload.get("command"),
            Some(&json!(".bee/bin/bee herding run --task-file - --json"))
        );
        let stdin = payload.get("stdin").unwrap().as_str().unwrap();
        assert!(!stdin.is_empty());
        assert!(stdin.contains("c-1"), "{stdin}");
        assert_eq!(
            v.get("economics").unwrap().get("channel"),
            Some(&json!("herding-exec"))
        );
    }

    /// D4: the same herding slot on a codex runtime dispatch takes the SAME
    /// Bash arm — a herding pane is a subprocess call, never a native
    /// codex spawn_agent (the `_ if runtime == "codex"` catch-all must
    /// never see a `Resolved::Herding`).
    #[test]
    fn herding_shaped_generation_takes_the_bash_arm_on_codex_too() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"codex":{"generation":{"kind":"herding"}}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "codex", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("tool"), Some(&json!("Bash")));
        assert_eq!(
            v.get("payload").unwrap().get("command"),
            Some(&json!(".bee/bin/bee herding run --task-file - --json"))
        );
    }

    /// D4: a granted worktree adds `--cwd "<worktree_root>"` to the
    /// command — the same single Location resolution the envelope and
    /// prompt already read (dp1_worktree_fixture, defined below).
    #[test]
    fn herding_shaped_generation_adds_cwd_for_a_granted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted) = dp1_worktree_fixture(tmp.path());
        std::fs::write(
            main.join(".bee").join("config.json"),
            r#"{"models":{"claude":{"generation":{"kind":"herding"}}}}"#,
        )
        .unwrap();
        w(
            &main,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"demo","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&main, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        let norm = |p: &str| match dunce::canonicalize(p) {
            Ok(c) => c.to_string_lossy().into_owned(),
            Err(_) => p.to_string(),
        };
        let granted_s = v.get("worktree_root").unwrap().as_str().unwrap().to_string();
        assert_eq!(norm(&granted_s), norm(granted.to_str().unwrap()));
        assert_eq!(
            v.get("payload").unwrap().get("command"),
            Some(&json!(format!(
                ".bee/bin/bee herding run --task-file - --json --cwd \"{granted_s}\""
            )))
        );
    }

    /// herd-registry D2: a slot naming `agent:"<name>"` appends
    /// `--agent "<name>"` after `--task-file - --json` (and after --cwd,
    /// when a granted worktree also applies).
    #[test]
    fn herding_shaped_generation_appends_agent_when_the_slot_names_one() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":{"kind":"herding","agent":"codex-cli"}}}}"#,
        );
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(
            v.get("payload").unwrap().get("command"),
            Some(&json!(
                ".bee/bin/bee herding run --task-file - --json --agent \"codex-cli\""
            ))
        );
    }

    // ── hrv-3: herding-review-slots D3 — the fallback:default field ────────

    /// D3: `"fallback": "default"` on a herding-shaped generation slot adds
    /// a `fallback: {model}` payload field naming the runtime's own default
    /// model for that slot (claude generation -> sonnet) — the same value
    /// a gather purpose used to resolve to silently, pre-D1-widening, now
    /// surfaced for the orchestrator's re-dispatch instead.
    #[test]
    fn herding_shaped_generation_with_fallback_default_adds_the_payload_fallback_field() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":{"kind":"herding","fallback":"default"}}}}"#,
        );
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("tool"), Some(&json!("Bash")));
        assert_eq!(
            v.get("payload").unwrap().get("fallback"),
            Some(&json!({"model": "sonnet"}))
        );
    }

    /// D3: absent `fallback` leaves the payload byte-identical — no
    /// `fallback` key at all.
    #[test]
    fn herding_shaped_generation_without_fallback_omits_the_payload_field() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":{"kind":"herding"}}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("payload").unwrap().get("fallback"), None);
    }

    /// D3: a runtime whose default for the slot is null (codex generation)
    /// still recognizes `fallback:"default"` on the shape, but has no
    /// default model to name — the payload stays byte-identical to a slot
    /// with no `fallback` field at all, never a `{model: null}`.
    #[test]
    fn herding_shaped_generation_with_fallback_default_and_no_runtime_default_omits_the_field() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"codex":{"generation":{"kind":"herding","fallback":"default"}}}}"#,
        );
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "codex", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("payload").unwrap().get("fallback"), None);
    }

    /// D3: an advisor purpose recognizes `fallback:"default"` on the shape
    /// (it round-trips through Resolved::Herding the same as every other
    /// purpose) but the advisor slot has no default-model table entry at
    /// all (resolveAdvisor "NEVER a tier fallback") — the payload stays
    /// byte-identical to no `fallback` field.
    #[test]
    fn herding_shaped_advisor_with_fallback_default_has_no_default_model_to_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"advisor":{"kind":"herding","fallback":"default"}}}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "advisor", None, None, false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("payload").unwrap().get("fallback"), None);
    }

    // ── hrv-1: herding-review-slots D1/D2 — reviewer/advisor purposes ──────

    /// D1: a reviewer purpose on a herding-shaped review slot takes the
    /// SAME herding-exec Bash arm as a cell purpose — the exact widening
    /// this cell exists to make.
    #[test]
    fn reviewer_purpose_on_a_herding_shaped_review_slot_emits_the_herding_exec_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"review":{"kind":"herding","agent":"agy-flash"}}}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "reviewer", None, None, false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("tool"), Some(&json!("Bash")));
        assert_eq!(
            v.get("payload").unwrap().get("command"),
            Some(&json!(
                ".bee/bin/bee herding run --task-file - --json --agent \"agy-flash\""
            ))
        );
        assert_eq!(
            v.get("economics").unwrap().get("channel"),
            Some(&json!("herding-exec"))
        );
    }

    /// D1: an advisor purpose on a herding-shaped advisor slot takes the
    /// same herding-exec Bash arm — resolve_advisor no longer reads a
    /// herding-shaped slot as "no advisor".
    #[test]
    fn advisor_purpose_on_a_herding_shaped_advisor_slot_emits_the_herding_exec_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"advisor":{"kind":"herding","agent":"named-herd"}}}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "advisor", None, None, false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("tool"), Some(&json!("Bash")));
        assert_eq!(
            v.get("payload").unwrap().get("command"),
            Some(&json!(
                ".bee/bin/bee herding run --task-file - --json --agent \"named-herd\""
            ))
        );
        assert_eq!(
            v.get("economics").unwrap().get("channel"),
            Some(&json!("herding-exec"))
        );
    }

    /// hrv-3 (D1 widened to the full mapping): a gather purpose on the SAME
    /// herding-shaped generation slot now ALSO takes the herding-exec Bash
    /// arm — the gather-default-model split the three tests above used to
    /// pin against is gone; every purpose reads the shape identically.
    #[test]
    fn gather_purpose_on_a_herding_shaped_generation_slot_also_emits_the_herding_exec_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":{"kind":"herding"}}}}"#);

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "gather", None, None, false, None, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("tool"), Some(&json!("Bash")));
        assert_eq!(
            v.get("payload").unwrap().get("command"),
            Some(&json!(".bee/bin/bee herding run --task-file - --json"))
        );
        assert_eq!(
            v.get("economics").unwrap().get("channel"),
            Some(&json!("herding-exec"))
        );
    }

    #[test]
    fn malformed_calls_throw_with_node_wording() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        let thrown = |o: Prepared| match o {
            Prepared::Thrown(m) => m,
            _ => panic!("expected a throw"),
        };
        assert_eq!(
            thrown(
                prepare_dispatch(&root, "claude", "cell", None, Some("w"), false, None, None, false)
                    .unwrap()
            ),
            "dispatch prepare: --cell is required when --kind cell."
        );
        assert_eq!(
            thrown(
                prepare_dispatch(
                    &root, "claude", "cell", Some("ghost"), Some("w"), false, None, None, false
                )
                .unwrap()
            ),
            "dispatch prepare: cell \"ghost\" not found."
        );
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","status":"claimed","trace":{"worker":"w"}}"#,
        );
        assert_eq!(
            thrown(
                prepare_dispatch(
                    &root, "claude", "cell", Some("c-1"), Some("   "), false, None, None, false
                )
                .unwrap()
            ),
            "dispatch prepare: --worker is required when --kind cell."
        );
    }

    #[test]
    fn force_ownership_always_leaves_an_audit_line() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"owner"}}"#,
        );
        // Forced past a real conflict.
        let Prepared::Value(v) = prepare_dispatch(
            &root, "claude", "cell", Some("c-1"), Some("thief"), true, None, None, true,
        )
        .unwrap() else {
            panic!()
        };
        let ov = v.get("ownership_override").unwrap();
        assert_eq!(ov.get("bypassed"), Some(&json!(true)));
        assert_eq!(ov.get("code"), Some(&json!("not_owner")));
        assert_eq!(ov.get("owner_bypassed"), Some(&json!("owner")));
        assert_eq!(ov.get("transferred"), Some(&json!(false)));
        // Forced with NO conflict still audits (msh-4 mirror).
        let Prepared::Value(v) = prepare_dispatch(
            &root, "claude", "cell", Some("c-1"), Some("owner"), true, None, None, false,
        )
        .unwrap() else {
            panic!()
        };
        let ov = v.get("ownership_override").unwrap();
        assert_eq!(ov.get("bypassed"), Some(&json!(false)));
        assert_eq!(ov.get("code"), Some(&Value::Null));
        // The audited override rides the dispatch record too.
        let log = std::fs::read_to_string(root.join(".bee/logs/dispatch.jsonl")).unwrap();
        assert!(log.contains("\"ownership_override\""));
    }

    #[test]
    fn native_unavailable_refuses_rather_than_downgrading() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"codex":{"generation":{"kind":"native","model":"gpt-5"}}}}"#,
        );
        // No confirmed override, no configured fallback -> typed refusal.
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "codex",
            "gather",
            None,
            None,
            false,
            Some(NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY),
            None,
            false,
        )
        .unwrap() else {
            panic!()
        };
        assert_eq!(
            jsjson::stringify(&v),
            r#"{"ok":false,"type":"refused","reason":"native_unavailable","detail":"native_budget_only"}"#
        );
        // A confirmed override emits the spawn_agent model-override payload.
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "codex",
            "gather",
            None,
            None,
            false,
            Some(NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE),
            None,
            false,
        )
        .unwrap() else {
            panic!()
        };
        assert_eq!(v.get("transport"), Some(&json!("native-override")));
        let p = v.get("payload").unwrap();
        assert_eq!(p.get("agent_type"), Some(&json!("worker")));
        assert_eq!(p.get("model"), Some(&json!("gpt-5")));
        assert_eq!(p.get("fork_turns"), Some(&json!("none")));
    }

    // ── dp-1: the granted-worktree Location (envelope + prompt) ────────────

    fn dp1_git_ok(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the worktree fixture");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// main + a REGISTERED `wt-granted` worktree carrying `feature: "demo"`
    /// — the same real `git worktree add` shape cells/tests.rs's
    /// `wf_worktree_fixture` uses, kept local rather than imported (that
    /// module's fixture is private to its own `#[cfg(test)] mod tests`).
    fn dp1_worktree_fixture(tmp: &Path) -> (PathBuf, PathBuf) {
        let main = tmp.join("main");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::write(
            main.join(".bee").join("config.json"),
            r#"{"models":{"claude":{"generation":"sonnet"}}}"#,
        )
        .unwrap();
        std::fs::write(main.join("f.txt"), "x").unwrap();
        dp1_git_ok(&main, &["init", "-q", "-b", "main", "."]);
        dp1_git_ok(&main, &["config", "user.email", "a@b.c"]);
        dp1_git_ok(&main, &["config", "user.name", "t"]);
        dp1_git_ok(&main, &["add", "-A"]);
        dp1_git_ok(&main, &["commit", "-qm", "init"]);
        let granted = tmp.join("wt-granted");
        dp1_git_ok(&main, &["worktree", "add", "-q", granted.to_str().unwrap(), "-b", "wt/g"]);
        std::fs::create_dir_all(main.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            main.join(".bee").join("runtime").join("worktree-grants.json"),
            "{\"wt-granted\": true}\n",
        )
        .unwrap();
        std::fs::create_dir_all(granted.join(".bee").join("runtime")).unwrap();
        std::fs::write(granted.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        std::fs::write(
            granted.join(".bee").join("runtime").join("worktree-identity.json"),
            "{\"feature\":\"demo\"}\n",
        )
        .unwrap();
        (main, granted)
    }

    /// must-have 1 + 2: when `find_granted_worktree_for_feature` resolves a
    /// granted worktree for the cell's feature, the envelope names both
    /// roots and the rendered prompt tells the worker where to work and
    /// that the store is elsewhere.
    #[test]
    fn envelope_and_prompt_name_the_granted_worktree_for_the_cells_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted) = dp1_worktree_fixture(tmp.path());
        w(
            &main,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"demo","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&main, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!("expected an envelope")
        };
        // Compare paths by IDENTITY, not by spelling. The envelope's roots come
        // out of the gitdir chain — git's own writing of the path — while the
        // fixture holds whatever `tempdir()` returned. On a Windows runner those
        // are the long and 8.3-short forms of one directory
        // (`runneradmin` vs `RUNNER~1`), so a lexical compare failed for a reason
        // that has nothing to do with what is being asserted. Same rule, and same
        // reason, as `roots.rs`'s test-local `norm`.
        let norm = |p: &str| match dunce::canonicalize(p) {
            Ok(c) => c.to_string_lossy().into_owned(),
            Err(_) => p.to_string(),
        };
        let envelope_str = |key: &str| v.get(key).and_then(|x| x.as_str()).map(norm);
        assert_eq!(
            envelope_str("worktree_root"),
            Some(norm(granted.to_str().unwrap()))
        );
        assert_eq!(
            envelope_str("control_root"),
            Some(norm(main.to_str().unwrap()))
        );

        // The prompt must name the SAME roots the envelope named, in the
        // envelope's own spelling — asserting it against the fixture's spelling
        // is what made this test platform-dependent in the first place.
        let granted_s = v.get("worktree_root").unwrap().as_str().unwrap();
        let main_s = v.get("control_root").unwrap().as_str().unwrap();

        let prompt = v
            .get("payload")
            .unwrap()
            .get("prompt")
            .unwrap()
            .as_str()
            .unwrap();
        assert!(
            prompt.contains("Location — work here, the store is in the other checkout:"),
            "{prompt}"
        );
        assert!(prompt.contains(&format!("- Work in: {granted_s}")), "{prompt}");
        assert!(
            prompt.contains(&format!(
                "- The bee store (cells, claims, reservations) lives in: {main_s}"
            )),
            "{prompt}"
        );
    }

    /// must-have 3: a feature with no granted worktree renders
    /// byte-identically to before this Location block existed — neither
    /// envelope key appears, and the prompt carries no Location text.
    #[test]
    fn envelope_and_prompt_stay_byte_identical_without_a_granted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!("expected an envelope")
        };
        assert_eq!(v.get("worktree_root"), None);
        assert_eq!(v.get("control_root"), None);
        let prompt = v
            .get("payload")
            .unwrap()
            .get("prompt")
            .unwrap()
            .as_str()
            .unwrap();
        assert!(!prompt.contains("Location —"), "{prompt}");
        assert!(!prompt.contains("{{"), "no unrendered marker residue: {prompt}");
    }

    // ── wmn-1: the knowledge bundle is native to the WORKING TREE, so a
    //    granted-worktree feature's learned-context block reads the
    //    worktree's own docs/knowledge/, never the control root's ─────────

    /// debt row 621: `docs/knowledge/` lives in the repo working tree. When
    /// `dispatch prepare` runs from the MAIN checkout for a cell whose
    /// feature has a granted worktree, the bundle that answers `--work
    /// <feature>` is the one checked out in THAT worktree, not main's own
    /// (possibly bundle-less) checkout — exactly like a native verb resolves
    /// its store root from inside the worktree (roots.rs's WIDE door).
    #[test]
    fn learned_context_reads_the_bundle_from_the_cells_granted_worktree_not_main() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted) = dp1_worktree_fixture(tmp.path());
        // The bundle lives ONLY in the granted worktree's own checkout —
        // main carries no docs/knowledge/ at all.
        bundle_fixture(&granted);
        w(
            &main,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"demo","status":"claimed","trace":{"worker":"w"},"lane":"small"}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&main, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!("expected an envelope")
        };
        let prompt = v.get("payload").unwrap().get("prompt").unwrap().as_str().unwrap();
        assert!(
            prompt.contains("Learned context (machine-assembled"),
            "the worktree's own bundle must produce a real learned-context block: {prompt}"
        );
        assert!(
            prompt.contains("- docs/knowledge/work/demo/work-item.md — Demo work item"),
            "{prompt}"
        );
        assert!(
            prompt.contains("- docs/knowledge/patterns/dispatch-prompt.md — Dispatch prompt assembly"),
            "{prompt}"
        );
    }

    /// The unworktreed / main-checkout path stays exactly as it was:
    /// `bundle_root` falls back to the control root itself, so a bundle that
    /// lives in main's own checkout renders unchanged.
    #[test]
    fn learned_context_still_reads_mains_own_bundle_without_a_granted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        bundle_fixture(&root);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"demo","status":"claimed","trace":{"worker":"w"},"lane":"small"}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!("expected an envelope")
        };
        let prompt = v.get("payload").unwrap().get("prompt").unwrap().as_str().unwrap();
        assert_eq!(
            learned_context_lines(&root, &json!({"id": "c-1", "feature": "demo", "lane": "small"})).unwrap(),
            vec![
                "- docs/knowledge/work/demo/work-item.md — Demo work item",
                "- docs/knowledge/work/demo/plan.md — Demo plan",
                "- docs/knowledge/patterns/dispatch-prompt.md — Dispatch prompt assembly",
                "- docs/knowledge/patterns/unrelated.md — Unrelated pattern",
                "- docs/knowledge/areas/dispatch.md — Dispatch decision",
            ]
        );
        assert!(
            prompt.contains("- docs/knowledge/work/demo/work-item.md — Demo work item"),
            "{prompt}"
        );
    }

    // ── dp-r1: dispatch prepare --claim registers the claiming worker ───────

    /// A lane record whose `approved_gates.execution` is true and whose
    /// `route` key is present — `claim_cell_from_flags`'s execution-gate
    /// door and D4's route check both clear the same way `cells/tests.rs`'s
    /// own `lane_with_route` fixture does (kept local since that helper is
    /// private to its own `#[cfg(test)] mod tests`).
    fn dpr1_lane_with_route(root: &Path, feature: &str) {
        w(
            root,
            &format!(".bee/lanes/{feature}.json"),
            &format!(r#"{{"feature":"{feature}","approved_gates":{{"execution":true}},"route":true}}"#),
        );
    }

    fn dpr1_workers(root: &Path) -> Vec<Value> {
        match crate::fsutil::read_json(&crate::state::state_path(root)) {
            ReadJson::Parsed(Value::Object(m)) => match m.get("workers") {
                Some(Value::Array(a)) => a.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    /// must-have: a successful `--claim` registers the claiming worker
    /// against the cell it just claimed, in the exact shape `bee state
    /// worker add --nickname <w> --cell <id> --tier <t> --status running`
    /// writes — findable by the SAME read the B44 close-time door
    /// (`registered_worker_for_cell`) uses, never a raw file peek.
    #[test]
    fn claim_registers_the_worker_findable_by_the_close_doors_own_read() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        dpr1_lane_with_route(&root, "f");
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","title":"t","status":"open","lane":"tiny","feature":"f","deps":[],"tier":"generation"}"#,
        );

        assert!(
            !crate::verbs::cells::registered_worker_for_cell(&root, "c-1", Some("bee-w1")).unwrap(),
            "an unclaimed cell has no registered worker yet"
        );

        let (cell, reserved, registered, err) =
            claim_and_reserve_for_dispatch(&root, None, "c-1", "bee-w1", None).unwrap().unwrap();
        assert_eq!(cell["status"], json!("claimed"), "the claim itself still lands");
        assert!(reserved.is_empty(), "no files declared, nothing to reserve");
        assert!(registered, "registration must succeed: {err:?}");
        assert_eq!(err, None);

        // The exact read the B44 cap-time door (`registered_worker_for_cell`,
        // verbs/cells/handlers_close.rs) uses to decide whether a small+ cell
        // may cap — never a raw file read standing in for it.
        assert!(
            crate::verbs::cells::registered_worker_for_cell(&root, "c-1", Some("bee-w1")).unwrap(),
            "the freshly registered worker must be findable through the close door's own check"
        );

        // The record shape itself: nickname/cell/tier/status, tier lifted
        // off the cell's own `tier` field, status hard-coded "running".
        let workers = dpr1_workers(&root);
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0].get("nickname"), Some(&json!("bee-w1")));
        assert_eq!(workers[0].get("cell"), Some(&json!("c-1")));
        assert_eq!(workers[0].get("tier"), Some(&json!("generation")));
        assert_eq!(workers[0].get("status"), Some(&json!("running")));
    }

    /// A cell with no `tier` field of its own registers with `tier: null` —
    /// `state worker add`'s own default when `--tier` is not passed, never a
    /// guessed value.
    #[test]
    fn claim_registers_with_a_null_tier_when_the_cell_names_none() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        dpr1_lane_with_route(&root, "f");
        w(
            &root,
            ".bee/cells/c-2.json",
            r#"{"id":"c-2","title":"t","status":"open","lane":"tiny","feature":"f","deps":[]}"#,
        );

        let (_cell, _reserved, registered, err) =
            claim_and_reserve_for_dispatch(&root, None, "c-2", "bee-w2", None).unwrap().unwrap();
        assert!(registered, "registration must succeed: {err:?}");

        let workers = dpr1_workers(&root);
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0].get("nickname"), Some(&json!("bee-w2")));
        assert_eq!(workers[0].get("tier"), Some(&Value::Null));
        assert_eq!(workers[0].get("status"), Some(&json!("running")));
    }

    /// A claim that never happens (already-claimed cell, refused before any
    /// registration attempt) registers nothing.
    #[test]
    fn a_failed_claim_registers_no_worker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        dpr1_lane_with_route(&root, "f");
        w(
            &root,
            ".bee/cells/c-3.json",
            r#"{"id":"c-3","title":"t","status":"claimed","lane":"tiny","feature":"f","deps":[],"trace":{"worker":"someone-else"}}"#,
        );

        let refusal = match claim_and_reserve_for_dispatch(&root, None, "c-3", "bee-w3", None).unwrap() {
            Err(message) => message,
            Ok(_) => panic!("an already-claimed cell must refuse, not succeed"),
        };
        assert!(refusal.contains("not \"open\""), "{refusal}");
        assert!(dpr1_workers(&root).is_empty(), "a refused claim registers nothing");
        assert!(!crate::verbs::cells::registered_worker_for_cell(&root, "c-3", Some("bee-w3")).unwrap());
    }

    /// `dispatch prepare` WITHOUT `--claim` never reaches the claim-and-
    /// register door at all — `prepare_dispatch`'s own dry-run build over an
    /// already-claimed cell (the shape a claim-less call reads) leaves
    /// `workers` untouched.
    #[test]
    fn claim_less_prepare_registers_no_worker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-4.json",
            r#"{"id":"c-4","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(_) =
            prepare_dispatch(&root, "claude", "cell", Some("c-4"), Some("w"), false, None, None, false)
                .unwrap()
        else {
            panic!("expected an envelope")
        };
        assert!(dpr1_workers(&root).is_empty(), "a claim-less prepare registers nothing");
        assert!(!crate::verbs::cells::registered_worker_for_cell(&root, "c-4", Some("w")).unwrap());
    }

    /// A registration failure (an invalid tier on the cell — the one
    /// deterministic failure `register_worker_for_cell` can hit) never
    /// unwinds a claim that already stands: the cell is claimed, the
    /// reservations (none, here) stand, and the outcome names the failure
    /// rather than silently dropping it.
    #[test]
    fn a_registration_failure_never_unwinds_the_standing_claim() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        dpr1_lane_with_route(&root, "f");
        w(
            &root,
            ".bee/cells/c-5.json",
            r#"{"id":"c-5","title":"t","status":"open","lane":"tiny","feature":"f","deps":[],"tier":"bogus"}"#,
        );

        let (cell, reserved, registered, err) =
            claim_and_reserve_for_dispatch(&root, None, "c-5", "bee-w5", None).unwrap().unwrap();
        assert_eq!(cell["status"], json!("claimed"), "the claim stands despite the registration failure");
        assert!(reserved.is_empty());
        assert!(!registered, "an invalid tier must fail registration, not silently pass");
        let message = err.expect("a failed registration must name why");
        assert!(message.contains("invalid tier"), "{message}");
        assert!(dpr1_workers(&root).is_empty(), "the bad record was never written");
    }

    // ── B-P2-6: worker_registered pinned through the REAL entry ────────────

    const DISPATCH_CLAIM_CHILD: &str = "verbs::drivers::tests::dispatch_prepare_claim_payload_child";

    /// Runs ONLY as a child of the test below. `run_dispatch_prepare` (the
    /// REAL CLI entry, not `claim_and_reserve_for_dispatch` alone) resolves
    /// its root off `std::env::current_dir()` and prints its payload to the
    /// process's own stdout — both process-global, so this is exercised
    /// out-of-process instead of mutating this binary's shared cwd/stdout
    /// under every other test, the same isolation `cells/tests.rs`'s
    /// `session_id_env_chain_child` uses for its own process-global seam.
    #[test]
    #[ignore = "spawned by dispatch_prepare_claim_payload_pins_worker_registered_true"]
    fn dispatch_prepare_claim_payload_child() {
        let (flags, use_json) = parse_flags(&[
            "--runtime", "claude", "--kind", "cell", "--cell", "c-1", "--worker", "bee-w1",
            "--claim", "--json",
        ])
        .expect("well-formed fixture argv");
        run_dispatch_prepare(flags, use_json, Instant::now());
    }

    /// dp-r1 / B-P2-6: a successful `--claim` names `worker_registered:true`
    /// on the SAME JSON payload `dispatch prepare` prints — pinned through
    /// the real CLI entry (`run_dispatch_prepare`), not just the inner
    /// claim/register function the tests above already cover.
    ///
    /// The registration-FAILURE half stays on `claim_and_reserve_for_dispatch`
    /// directly (`a_registration_failure_never_unwinds_the_standing_claim`,
    /// above): `run_dispatch_prepare` has no clean, corrupt-able seam that
    /// fails registration alone without also refusing the claim itself
    /// earlier (an invalid `--cell`/`--worker` never reaches registration; an
    /// invalid cell `tier` is the one deterministic failure, and it is
    /// already exercised, byte-for-byte, on the inner function both doors
    /// share) — so this test pins the success shape through the real entry
    /// and the existing inner-fn test keeps the failure shape, per rph-4.
    #[test]
    fn dispatch_prepare_claim_payload_pins_worker_registered_true() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        dpr1_lane_with_route(&root, "f");
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","title":"t","status":"open","lane":"tiny","feature":"f","deps":[],"tier":"generation"}"#,
        );

        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = Command::new(&exe);
        cmd.args(["--exact", DISPATCH_CLAIM_CHILD, "--ignored", "--test-threads", "1", "--nocapture"]);
        cmd.current_dir(&root);
        let out = cmd.output().expect("spawn the test binary");
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let start = stdout.find('{').unwrap_or_else(|| panic!("no JSON payload in child stdout:\n{stdout}"));
        let end = stdout.rfind('}').map(|i| i + 1).unwrap_or_else(|| panic!("no JSON payload in child stdout:\n{stdout}"));
        let payload: Value = serde_json::from_str(&stdout[start..end])
            .unwrap_or_else(|e| panic!("child stdout was not valid JSON ({e}):\n{stdout}"));
        assert_eq!(payload.get("claimed"), Some(&json!(true)), "payload: {payload}");
        assert_eq!(payload.get("worker_registered"), Some(&json!(true)), "payload: {payload}");
        assert!(payload.get("registration_error").is_none(), "payload: {payload}");
    }

    #[test]
    fn absent_probe_record_classifies_budget_only_without_a_subprocess() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        assert_eq!(
            native_transport_classification(&root).unwrap(),
            NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY
        );
        // Corrupt / schema-mismatched records short-circuit the same way.
        w(&root, ".bee/native-transport-probe.json", "{not json");
        assert_eq!(
            native_transport_classification(&root).unwrap(),
            NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY
        );
        w(&root, ".bee/native-transport-probe.json", r#"{"schema":"other/9"}"#);
        assert_eq!(
            native_transport_classification(&root).unwrap(),
            NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY
        );
        // A LIVE record needs codex-cli probes — delegate.
        w(
            &root,
            ".bee/native-transport-probe.json",
            r#"{"schema":"native-transport-probe/1"}"#,
        );
        assert!(native_transport_classification(&root).is_err());
    }

    // ── learned context ────────────────────────────────────────────────────

    /// The knowledge-bundle fixture, shared by the manifest cross-check and
    /// the learned-context tests. Byte-stable on purpose: the golden manifest
    /// below quotes its exact byte sizes.
    fn bundle_fixture(root: &Path) {
        w(root, "docs/knowledge/index.md", "# Knowledge\n\n## Critical patterns\n\n- none yet\n");
        w(root, "docs/knowledge/work/demo/work-item.md",
          "---\ntype: bee.work-item\ntitle: Demo work item\ndescription: port the dispatch driver to rust\nbee:\n  id: demo\n  lifecycle: active\n  areas: [dispatch]\n  decisions: [d-1]\n---\n\nThe demo work item body mentions dispatch prompts and rust.\n");
        w(root, "docs/knowledge/work/demo/plan.md",
          "---\ntype: bee.plan\ntitle: Demo plan\ndescription: the plan for demo\nbee:\n  id: demo-plan\n  lifecycle: active\n---\n\nPlan body.\n");
        w(root, "docs/knowledge/patterns/dispatch-prompt.md",
          "---\ntype: bee.pattern\ntitle: Dispatch prompt assembly\ndescription: how dispatch prompts are assembled\nbee:\n  id: p-dispatch\n  lifecycle: active\n  critical: true\n  areas: [dispatch]\n---\n\nDispatch prompts are assembled from templates and machine blocks.\n");
        w(root, "docs/knowledge/patterns/unrelated.md",
          "---\ntype: bee.pattern\ntitle: Unrelated pattern\ndescription: about billing invoices\nbee:\n  id: p-billing\n  lifecycle: active\n  critical: true\n  areas: [billing]\n---\n\nBilling invoices and refunds.\n");
        w(root, "docs/knowledge/areas/dispatch.md",
          "---\ntype: bee.decision\ntitle: Dispatch decision\ndescription: a decision about dispatch\nbee:\n  id: d-1\n  lifecycle: active\n  areas: [dispatch]\n---\n\nDecided.\n");
    }

    // R6 (CLOSED): `learned_context_agrees_with_the_knowledge_verb_port` and
    // `learned_context_history_anchor_agrees_across_both_ports` — the two
    // byte-parity tests that pinned the drivers/kctx.rs lift against
    // verbs/knowledge's own build_context_manifest — are retired with the
    // copy they compared. There is exactly one build_context_manifest now
    // (crate::verbs::knowledge::build_context_manifest, called directly by
    // bundle_learned_lines/bundle_mode below), so a cross-copy-agreement
    // assertion has nothing left to say; its behavior (work-item anchor,
    // history anchor from CONTEXT.md/plan.md, zero_signal reporting under a
    // history anchor) stays proven by knowledge/tests.rs's own suite
    // (history_anchor_resolves_from_whichever_of_context_and_plan_exist,
    // history_anchor_reports_zero_signal_instead_of_throwing, and
    // neighbors), and by the learned-context tests below that already
    // exercise this file's own bundle_learned_lines/learned_context_lines
    // call sites end to end.

    #[test]
    fn learned_context_uses_the_manifest_and_honours_read_first() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        bundle_fixture(&root);
        assert!(bundle_mode(&root).unwrap());
        let cell = json!({"id": "c-1", "feature": "demo", "lane": "small"});
        assert_eq!(
            learned_context_lines(&root, &cell).unwrap(),
            vec![
                "- docs/knowledge/work/demo/work-item.md — Demo work item",
                "- docs/knowledge/work/demo/plan.md — Demo plan",
                "- docs/knowledge/patterns/dispatch-prompt.md — Dispatch prompt assembly",
                "- docs/knowledge/patterns/unrelated.md — Unrelated pattern",
                "- docs/knowledge/areas/dispatch.md — Dispatch decision",
            ]
        );
        // read_first stays authoritative: its entries are never duplicated,
        // and backslashes / "./" prefixes normalize first.
        let cell = json!({
            "id": "c-1", "feature": "demo",
            "read_first": ["docs\\knowledge\\work\\demo\\work-item.md", "./docs/knowledge/areas/dispatch.md"]
        });
        let lines = learned_context_lines(&root, &cell).unwrap();
        assert!(!lines.iter().any(|l| l.contains("work-item.md")));
        assert!(!lines.iter().any(|l| l.contains("areas/dispatch.md")));
        assert_eq!(lines.len(), 3);
    }

    /// review B-P3-3: a manifest entry's `path` is a raw filesystem-derived
    /// string — only the title went through `one_line`. A bundle filename
    /// carrying a newline used to forge extra bullet lines inside the
    /// worker prompt's "Learned context" block; the path now gets the same
    /// whitespace-collapsing treatment.
    ///
    /// Unix-only, and not for convenience: the attack this pins needs a real
    /// file whose NAME contains a newline, which Windows refuses outright
    /// (`InvalidFilename`, OS error 123 — control characters are illegal in
    /// NTFS names). The vector is unrepresentable on that platform, so the
    /// end-to-end wiring is proved where it can exist; the collapsing rule
    /// itself is pinned platform-independently by
    /// `one_line_collapses_whitespace_and_ellipsises`. Before this gate the
    /// Windows lane was red on every commit, which hid every other Windows
    /// regression behind a failure no Windows user could ever hit.
    #[cfg(unix)]
    #[test]
    fn learned_context_collapses_a_newline_smuggled_in_a_bundle_filename() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        bundle_fixture(&root);
        w(&root, "docs/knowledge/patterns/evil\ninjected.md",
          "---\ntype: bee.pattern\ntitle: Evil pattern\ndescription: forged bullet via a newline in the filename\nbee:\n  id: p-evil\n  lifecycle: active\n  critical: true\n  areas: [dispatch]\n---\n\nDispatch prompts rust body.\n");
        let cell = json!({"id": "c-1", "feature": "demo", "lane": "high-risk"});
        let lines = learned_context_lines(&root, &cell).unwrap();
        // No line carries a raw newline — the smuggled break collapses to a
        // single space, so it can never masquerade as a second bullet.
        assert!(lines.iter().all(|l| !l.contains('\n')));
        assert!(
            lines.contains(&"- docs/knowledge/patterns/evil injected.md — Evil pattern".to_string()),
            "expected the collapsed path in {lines:?}"
        );
        // A normal path is byte-identical to before the fix — one_line is a
        // no-op on text with no whitespace to collapse.
        assert!(lines.contains(&"- docs/knowledge/work/demo/work-item.md — Demo work item".to_string()));
    }

    #[test]
    fn learned_context_falls_back_to_the_index_pointer_then_to_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        bundle_fixture(&root);
        // A cell whose feature names no work item: the manifest throws
        // unknown_work (caught) and the index pointer answers instead.
        let cell = json!({"id": "c-1", "feature": "not-a-work-item"});
        assert_eq!(
            learned_context_lines(&root, &cell).unwrap(),
            vec!["- docs/knowledge/index.md — Knowledge bundle index (see \"Critical patterns\")"]
        );
        // …unless read_first already names it.
        let cell =
            json!({"id": "c-1", "feature": "nope", "read_first": ["docs/knowledge/index.md"]});
        assert!(learned_context_lines(&root, &cell).unwrap().is_empty());
    }

    /// D8: `bundle_learned_lines` already passes a cell's `feature` slug as
    /// `work` and maps `ManifestOut::Thrown(_)` to `None` — before this
    /// feature, a dispatched cell for a slug with no bee.work-item concept
    /// always hit that `Thrown` arm and fell back to the bare index pointer.
    /// It now inherits kl-1's resolver through crate::verbs::knowledge, so a slug whose only
    /// anchor is a docs/history/<slug>/CONTEXT.md carries that manifest's
    /// own entries instead of falling back to `None`.
    #[test]
    fn dispatch_prepare_carries_a_history_anchor_manifest_instead_of_none() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(&root, "docs/knowledge/index.md", "# Knowledge\n\n## Critical patterns\n\n- none yet\n");
        w(&root, "docs/knowledge/patterns/x.md",
          "---\ntype: bee.pattern\ntitle: X\ndescription: x\nbee:\n  id: p-x\n  lifecycle: active\n---\n\nBody.\n");
        w(&root, "docs/history/hist-only/CONTEXT.md", "# Hist Only Context\n\nSome learnings.\n");
        let cell = json!({"id": "c-1", "feature": "hist-only", "lane": "small"});
        assert_eq!(
            learned_context_lines(&root, &cell).unwrap(),
            vec!["- docs/history/hist-only/CONTEXT.md — CONTEXT.md"]
        );
    }

    #[test]
    fn no_bundle_falls_back_to_the_legacy_critical_patterns_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        let cell = json!({"id": "c-1", "feature": "demo"});
        // Nothing at all -> the block is omitted (byte-identical to a repo
        // with no knowledge layer).
        assert!(learned_context_lines(&root, &cell).unwrap().is_empty());
        w(&root, "docs/history/learnings/critical-patterns.md", "# Critical patterns\n");
        assert_eq!(
            learned_context_lines(&root, &cell).unwrap(),
            vec!["- docs/history/learnings/critical-patterns.md — Critical patterns (hard-won learnings)"]
        );
        // A directory full of markdown that parses as NO concept is not a
        // bundle (advisor-digest-f3 finding 1).
        w(&root, "docs/knowledge/stray.md", "just prose, no frontmatter\n");
        assert!(!bundle_mode(&root).unwrap());
    }

    #[test]
    fn learned_context_is_capped_at_eight_pointer_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        bundle_fixture(&root);
        for i in 0..12 {
            w(
                &root,
                &format!("docs/knowledge/patterns/extra-{i}.md"),
                &format!("---\ntype: bee.pattern\ntitle: Extra {i}\ndescription: dispatch prompts rust {i}\nbee:\n  id: p-extra-{i}\n  lifecycle: active\n  critical: true\n  areas: [dispatch]\n---\n\nDispatch prompts extra {i}.\n"),
            );
        }
        let cell = json!({"id": "c-1", "feature": "demo", "lane": "high-risk"});
        let lines = learned_context_lines(&root, &cell).unwrap();
        assert_eq!(lines.len(), LEARNED_CONTEXT_MAX_LINES);
    }

    #[test]
    fn lane_budget_scales_and_defaults() {
        assert_eq!(lane_budget(Some(&json!("tiny"))), 8000.0);
        assert_eq!(lane_budget(Some(&json!("small"))), 12000.0);
        assert_eq!(lane_budget(Some(&json!("standard"))), 20000.0);
        assert_eq!(lane_budget(Some(&json!("high-risk"))), 30000.0);
        assert_eq!(lane_budget(None), 20000.0);
        assert_eq!(lane_budget(Some(&json!("banana"))), 20000.0);
    }

    // ── close ──────────────────────────────────────────────────────────────

    #[test]
    fn declaration_normalizes_strings_arrays_and_the_none_sentinel() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{}}"#);
        assert!(declared_test_commands(&root).unwrap().is_none());
        w(&root, ".bee/config.json", r#"{"commands":{"test":"  npm test  "}}"#);
        assert_eq!(
            declared_test_commands(&root).unwrap(),
            Some(vec!["npm test".to_string()])
        );
        w(&root, ".bee/config.json", r#"{"commands":{"test":[" a ",1,""," b "]}}"#);
        assert_eq!(
            declared_test_commands(&root).unwrap(),
            Some(vec!["a".to_string(), "b".to_string()])
        );
        w(&root, ".bee/config.json", r#"{"commands":{"test":["none"," none "]}}"#);
        assert!(declared_test_commands(&root).unwrap().is_none());
        // A dogfood_repos list makes readConfig warn per dead repo -> delegate.
        w(
            &root,
            ".bee/config.json",
            r#"{"commands":{"test":"x"},"dogfood_repos":["Z:/gone"]}"#,
        );
        assert!(declared_test_commands(&root).is_err());
        // CUTOVER: a corrupt config used to bail to Node (readJson's warning
        // carried a V8 parse message). It now warns and reads as "no config",
        // which is readJson's own `{}` fallback — so the declaration is
        // simply absent and the close report says tests are undeclared.
        w(&root, ".bee/config.json", "{broken");
        assert_eq!(declared_test_commands(&root).unwrap(), None);
    }

    /// D7 (docs/history/test-doctrine/CONTEXT.md): close never spawns
    /// `commands.test` — the tests door reads recorded proof from capped
    /// cells instead (verbs/cells/proof.rs `feature_proof_check`). A
    /// feature with no capped cells at all reads as "nothing to prove yet",
    /// never blocking.
    #[test]
    fn close_dry_run_reports_the_doors_and_runs_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":["a","b"]}}"#);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", true, declared, None, &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        assert_eq!(
            text,
            concat!(
                "door tests: open — no capped cells yet — nothing to prove\n",
                "door scribing-debt: clear\n",
                "door capture-queue: clear\n",
                "door pattern-check: clear\n",
                "door knowledge-freshness: clear\n",
                "door impact: clear\n",
                "door routing: open — NOTICE — no docs/history/demo/CONTEXT.md found to route (legacy-form gap); the routing door never blocks on it — route it manually or fold it into the D4 historical-routing-sweep campaign backlog row\n",
                "door doc-deferral: clear\n",
                "next: bee close --feature demo — checks every capped cell's proof line and reports"
            )
        );
        assert_eq!(result.get("feature"), Some(&json!("demo")));
        // Nothing ran: no test process spawns, so no record file appears.
        assert!(!root.join(".bee/logs/test-results.json").exists());

        // A capped cell whose report carries an empty proof string blocks
        // the dry-run listing too, naming the cell and the remedy.
        w(
            &root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"","deviations":[]}}}"#,
        );
        let Out::Emit(_, text, _) = close_handler(&root, "demo", true, None, None, &HashMap::new()).unwrap() else {
            panic!()
        };
        assert!(
            text.starts_with(
                "door tests: BLOCKING — 1 capped cell(s) carry a report with no valid proof line (demo-1) — re-cap with a real proof line: \"<command> — <result> — <scope reason>\". | settle: bee cells finish\n"
            ),
            "{text}"
        );
        assert!(text.ends_with(
            "next: re-cap the cell(s) above with a real proof line (\"<command> — <result> — <scope reason>\"), then re-run bee close --feature demo"
        ));
    }

    #[test]
    fn close_green_reports_the_capture_checklist() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        let lines: Vec<&str> = text.split('\n').collect();
        // D7: no capped cells here, so the tests door reads "nothing to
        // prove" — no test process spawns, so there is no per-command line
        // between the headline and the capture checklist.
        assert_eq!(lines[0], "Tests GREEN for \"demo\" — no capped cells yet — nothing to prove");
        assert_eq!(
            lines[1],
            "Capture (deferred, decision c8e25271): scribing clear; capture queue clear."
        );
        // D2 soft promote door: "demo" carries neither a bee.work-item
        // concept nor a docs/history/demo/ anchor here, so build_promotion
        // throws unknown_work — which degrades to ONE warning line, not a
        // refusal, and close still proceeds to its own next: line.
        assert_eq!(
            lines[2],
            "Promote skipped for \"demo\": knowledge promote: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id \"demo\" (D38)."
        );
        assert_eq!(
            lines[3],
            "next: done — capture is recorded as pending (run bee-capturing whenever; orient keeps the reminder)."
        );
        assert_eq!(result.get("ran_tests"), Some(&json!(false)));
        assert_eq!(result.get("tests"), Some(&Value::Null));
        // No test process ever spawns any more — no record file appears.
        assert!(!root.join(".bee/logs/test-results.json").exists());
        // A promote Thrown never writes the proposals file (D38 stays intact).
        assert!(!root.join("docs/history/demo/promote-proposals.md").exists());
    }

    /// D2: build_promotion's `None` arm ("delegate to Node" — no Node is
    /// left) degrades through the SAME one warning line a Thrown does, and
    /// close's exit code is unchanged either way. A configured non-empty
    /// `product_root` is the one live path left that makes `bundle_dir`
    /// return `None` (see `bundle_dir`, frame.rs).
    #[test]
    fn close_green_promote_none_warns_once_and_writes_nothing() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"commands":{"test":"echo suite-green"},"product_root":"elsewhere"}"#,
        );
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(_, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        let lines: Vec<&str> = text.split('\n').collect();
        // D7: no per-command line between the headline and the capture
        // checklist any more (no test process ever spawns).
        assert_eq!(
            lines[2],
            "Promote skipped for \"demo\": no docs/knowledge/ bundle to mine here"
        );
        assert_eq!(
            lines[3],
            "next: done — capture is recorded as pending (run bee-capturing whenever; orient keeps the reminder)."
        );
        assert!(!root.join("docs/history/demo/promote-proposals.md").exists());
    }

    /// D2 happy path: a green close with a resolvable history anchor
    /// (docs/history/<feature>/CONTEXT.md, D6) and a capped behavior_change
    /// cell proposes off it, writes ONE proposals file naming the delivery
    /// draft, the area updates and the pattern candidates, and appends ONE
    /// headline line naming the counts and the file.
    ///
    /// `cells_archive_on_close` is left at its DEFAULT (true) here on
    /// purpose: `build_promotion` now runs before retirement moves the
    /// feature's capped cells into `.bee/cells/archive/`, so it still finds
    /// `.bee/cells/demo-1.json` and the proposal counts below stay
    /// non-zero even though the same close call retires that cell a few
    /// lines later.
    #[test]
    fn close_green_promote_ok_writes_the_proposals_file_and_a_headline() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        std::fs::create_dir_all(root.join("docs/knowledge")).unwrap();
        w(
            &root,
            "docs/history/demo/CONTEXT.md",
            "# Demo Context\n\nBody.\n",
        );
        w(
            &root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","title":"first","verify":"cargo test","trace":{"behavior_change":true,"outcome":"did the thing","files_changed":["src/a.rs"],"capped_at":"2024-06-02T10:00:00.000Z"}}"#,
        );
        // Capture already recorded (a scribing run after the cap) — the
        // scribing-debt door stays clear, so this close reaches the promote
        // door at all; a debt-blocked close is proven separately.
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"demo\",\"ts\":\"2024-06-03T00:00:00.000Z\"}\n",
        );
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(_, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        let lines: Vec<&str> = text.split('\n').collect();
        // D7: demo-1 carries no trace.report at all — a legacy cap, so the
        // tests door passes it ungated with a note; no per-command line
        // exists any more either way (no test process spawns). Default
        // archiving retires demo-1 in the same close, so its line lands
        // ahead of the promote line — build_promotion already ran (and saw
        // the cell) before this retirement happened.
        assert_eq!(
            lines[2],
            "Retired \"demo\": 1 cell(s) moved out of the active scan (bee cells unarchive --feature demo to reverse)."
        );
        assert_eq!(
            lines[3],
            "Promote proposed for \"demo\": 1 capped cell(s) mined, 0 area bullet(s), 0 pattern candidate(s) — see docs/history/demo/promote-proposals.md."
        );
        assert_eq!(
            lines[4],
            "next: done — capture is recorded as pending (run bee-capturing whenever; orient keeps the reminder)."
        );
        let proposals = std::fs::read_to_string(root.join("docs/history/demo/promote-proposals.md")).unwrap();
        assert!(proposals.contains("(a) DELIVERY DRAFT"));
        assert!(proposals.contains("(b) AREA UPDATES"));
        assert!(proposals.contains("(c) PATTERN CANDIDATES"));
        // D38/D5 still hold: promote proposes, close never writes under
        // docs/knowledge/.
        assert!(!root.join("docs/knowledge/work").exists());
        // Retirement did happen (D2 must see the cells BEFORE this moves
        // them, not skip the move).
        assert!(!root.join(".bee/cells/demo-1.json").exists());
        assert!(root.join(".bee/cells/archive/demo/demo-1.json").exists());
    }

    /// U4 (docs/history/knowledge-usable/CONTEXT.md): a close that writes a
    /// promote proposal ALSO enqueues exactly one capture-queue stub
    /// pointing at it — the queue is the living channel a proposal reaches
    /// flush through; the proposals file keeps being written unchanged
    /// (asserted above, in the sibling test this one shares its fixture
    /// with).
    #[test]
    fn close_green_promote_ok_enqueues_one_capture_stub_pointing_at_it() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        std::fs::create_dir_all(root.join("docs/knowledge")).unwrap();
        w(
            &root,
            "docs/history/demo/CONTEXT.md",
            "# Demo Context\n\nBody.\n",
        );
        w(
            &root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","title":"first","verify":"cargo test","trace":{"behavior_change":true,"outcome":"did the thing","files_changed":["src/a.rs"],"capped_at":"2024-06-02T10:00:00.000Z"}}"#,
        );
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"demo\",\"ts\":\"2024-06-03T00:00:00.000Z\"}\n",
        );
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(_, _text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        let queue = std::fs::read_to_string(root.join(".bee/capture-queue.jsonl")).unwrap();
        let rows: Vec<Value> = queue.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        assert_eq!(rows.len(), 1, "expected exactly one enqueued stub: {queue}");
        let stub = &rows[0];
        assert_eq!(stub.get("kind"), Some(&json!("stub")));
        assert_eq!(
            stub.get("outcome"),
            Some(&json!("Promote proposal for \"demo\" — docs/history/demo/promote-proposals.md"))
        );
        assert_eq!(
            stub.get("files"),
            Some(&json!(["docs/history/demo/promote-proposals.md"]))
        );
        assert_eq!(stub.get("source"), Some(&json!("promote")));
    }

    /// D2: a promote door that skips (no proposal was written) never
    /// enqueues a stub — nothing to point at.
    #[test]
    fn close_green_promote_skipped_enqueues_nothing() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(_, _text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        assert!(!root.join(".bee/capture-queue.jsonl").exists());
    }

    /// D7 (docs/history/test-doctrine/CONTEXT.md): a capped cell whose
    /// `trace.report` is present but carries no valid D8 proof string stops
    /// close at the tests door, naming the cell — the refusal is red output
    /// (exit 1), never silent, and close never reaches the promote door.
    #[test]
    fn close_refuses_at_the_tests_door_naming_the_bad_proof_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(
            &root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"not a proof string","deviations":[]}}}"#,
        );
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, None, None, &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1);
        let lines: Vec<&str> = text.split('\n').collect();
        assert!(
            lines[0].starts_with(CLOSE_PROOF_DEBT_PREFIX),
            "refusal headline must start with the pinned prefix: {}",
            lines[0]
        );
        assert_eq!(
            lines[0],
            "Proof debt for \"demo\" — close stops at the tests door: 1 capped cell(s) carry a report with no valid proof line (demo-1)."
        );
        assert!(lines[1].contains("re-cap"), "{}", lines[1]);
        assert!(lines[1].contains("bee cells finish"), "{}", lines[1]);
        assert!(lines[2].starts_with("next:"));
        // No test process ever spawns — close never wrote a results record.
        assert!(!root.join(".bee/logs/test-results.json").exists());
        let doors = result.get("doors").unwrap().as_array().unwrap();
        assert_eq!(doors[0].get("blocking"), Some(&json!(true)));
        // The report-only doors are never blocking, even beside a proof refusal.
        assert_eq!(doors[1].get("blocking"), Some(&json!(false)));
        assert_eq!(doors[2].get("blocking"), Some(&json!(false)));
        // D2: the promote door sits past the tests door — a refused close
        // never reaches it, so nothing is proposed and nothing is written.
        assert!(!lines.iter().any(|l| l.starts_with("Promote")));
        assert!(!root.join("docs/history/demo/promote-proposals.md").exists());
    }

    /// truths #1: `bee close` never spawns `commands.test` — not even when
    /// the feature has a granted worktree and the config declares a
    /// command. The `dp1_worktree_fixture` config below names a command
    /// that would leave a tell-tale side effect (a results record) if it
    /// ever ran; it never does, GREEN or not, no matter the worktree grant.
    #[test]
    fn close_never_spawns_commands_test_even_with_a_granted_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted) = dp1_worktree_fixture(tmp.path());
        std::fs::write(
            main.join(".bee").join("config.json"),
            r#"{"commands":{"test":"echo should-not-run"}}"#,
        )
        .unwrap();
        let declared = declared_test_commands(&main).unwrap();
        assert!(declared.is_some(), "fixture must declare commands.test");
        let Out::Emit(result, text, code) =
            close_handler(&main, "demo", false, declared, None, &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        // No test process ever ran: the results record a spawn would leave
        // behind is never created.
        assert!(!main.join(".bee/logs/test-results.json").exists());
        let doors = result.get("doors").unwrap().as_array().unwrap();
        assert_eq!(doors[0].get("door"), Some(&json!("tests")));
        assert_eq!(doors[0].get("blocking"), Some(&json!(false)));
        assert_eq!(result.get("ran_tests"), Some(&json!(false)));
        assert_eq!(result.get("tests"), Some(&Value::Null));
        assert!(text.starts_with("Tests GREEN for \"demo\""), "text: {text}");
    }

    /// must-have: "close on a feature whose caps all carry proof strings
    /// proceeds" — every capped cell here carries a well-formed D8 proof
    /// line, so the tests door reports it and never blocks.
    #[test]
    fn close_proceeds_when_every_capped_cell_carries_a_valid_proof_line() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(
            &root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"cargo test -p bee — green — touched a.rs","deviations":[]}}}"#,
        );
        w(
            &root,
            ".bee/cells/demo-2.json",
            r#"{"id":"demo-2","feature":"demo","status":"capped","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"cargo test -p bee — green — touched b.rs","deviations":[]}}}"#,
        );
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, None, None, &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        assert!(
            text.starts_with("Tests GREEN for \"demo\" — 2 capped cell(s) all carry a proof line"),
            "text: {text}"
        );
        let doors = result.get("doors").unwrap().as_array().unwrap();
        assert_eq!(doors[0].get("door"), Some(&json!("tests")));
        assert_eq!(doors[0].get("blocking"), Some(&json!(false)));
    }

    #[test]
    fn close_surfaces_pending_capture_reminders() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(&root, ".bee/state.json", r#"{"feature":"demo"}"#);
        w(&root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(&root, ".bee/cells/demo-5.json", r#"{"id":"demo-5","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-02T00:00:00.000Z"}}"#);
        // A capped cell of ANOTHER feature never counts.
        w(&root, ".bee/cells/other.json", r#"{"id":"other","feature":"x","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-02T00:00:00.000Z"}}"#);
        w(
            &root,
            ".bee/capture-queue.jsonl",
            "{\"kind\":\"stub\",\"id\":\"s1\"}\n{\"kind\":\"stub\",\"id\":\"s2\"}\n{\"kind\":\"flush\",\"id\":\"s1\"}\n",
        );

        // D1: uncaptured behavior_change cells with no capture-deferral
        // decision on file — the door BLOCKS.
        let doors = build_close_report_doors(&root, "demo").unwrap();
        assert!(doors[0].blocking);
        assert_eq!(
            doors[0].detail,
            "pending — 2 behavior_change cell(s) uncaptured (demo-4, demo-5); run bee-capturing to record the capture, or log a decision tagged capture-deferral naming \"demo\" to defer it"
        );
        assert_eq!(doors[0].command, Some("bee-capturing"));
        assert!(!doors[1].blocking);
        assert_eq!(
            doors[1].detail,
            "pending — 1 capture stub(s) awaiting flush; settle later via bee-capturing"
        );
        assert_eq!(
            render_close_door_lines(&doors),
            vec![
                "door scribing-debt: BLOCKING — pending — 2 behavior_change cell(s) uncaptured (demo-4, demo-5); run bee-capturing to record the capture, or log a decision tagged capture-deferral naming \"demo\" to defer it | settle: bee-capturing",
                "door capture-queue: open — pending — 1 capture stub(s) awaiting flush; settle later via bee-capturing",
            ]
        );

        // A logged capture-deferral decision naming the feature LIFTS the
        // block without touching the count — same cells, softer door.
        w(
            &root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-07-02T12:00:00.000Z\",\"decision\":\"defer capture for demo until next sprint\",\"rationale\":\"r\",\"tags\":[\"capture-deferral\"],\"scope\":\"repo\"}\n",
        );
        let deferred_doors = build_close_report_doors(&root, "demo").unwrap();
        assert!(!deferred_doors[0].blocking);
        assert_eq!(deferred_doors[0].command, None);
        assert_eq!(
            deferred_doors[0].detail,
            "deferred — 2 behavior_change cell(s) uncaptured (demo-4, demo-5); a logged capture-deferral decision names \"demo\""
        );
        // A capture-deferral decision naming a DIFFERENT feature never lifts
        // THIS feature's block.
        assert!(!has_capture_deferral_decision(&root, "elsewhere").unwrap());
        std::fs::remove_file(root.join(".bee/decisions.jsonl")).unwrap();

        // A scribing run after the caps clears the debt (threshold is >, not >=).
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"demo\",\"ts\":\"2026-07-03T00:00:00.000Z\"}\n",
        );
        let cleared = build_close_report_doors(&root, "demo").unwrap();
        assert_eq!(cleared[0].detail, "clear");
        assert!(!cleared[0].blocking);
        // A ledger row for ANOTHER feature never moves this feature's threshold.
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"elsewhere\",\"ts\":\"2026-07-03T00:00:00.000Z\"}\n",
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 2);
    }

    // ── D1: the uncaptured-set computation (red-first, before the flip) ─────

    /// The counter close's refusal reads (scribing_debt) is the SAME counter
    /// the scribing-debt door already reported pre-D1 — one membership, one
    /// owner. Pins the three fixture shapes D6 calls out: a behavior_change
    /// capped cell with no capture counts; one capped AFTER a scribing run
    /// does not; a non-behavior_change capped cell never counts regardless
    /// of capture state.
    #[test]
    fn uncaptured_behavior_change_set_matches_the_scribing_debt_definition() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(&root, ".bee/cells/demo-1.json", r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        // Non-behavior_change: never counts, capture state notwithstanding.
        w(&root, ".bee/cells/demo-2.json", r#"{"id":"demo-2","feature":"demo","status":"capped","trace":{"behavior_change":false,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        // Not even capped yet: an open cell never counts either way.
        w(&root, ".bee/cells/demo-3.json", r#"{"id":"demo-3","feature":"demo","status":"open","trace":{"behavior_change":true}}"#);
        let debt = scribing_debt(&root, "demo").unwrap();
        assert_eq!(debt.count, 1);
        assert_eq!(debt.ids, vec![json!("demo-1")]);

        // A scribing run recorded AFTER the cap is "capture recorded" — the
        // same cell drops out of the uncaptured set.
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"demo\",\"ts\":\"2026-07-02T00:00:00.000Z\"}\n",
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 0);
    }

    // ── debt-door-archive dda-1: the archive walk ────────────────────────────

    /// `bee close` archives a feature's cells on a green close
    /// (`.bee/cells/archive/<feature>/*.json`). A behavior_change cell that
    /// lives ONLY there — never in the live `.bee/cells/` dir — must still
    /// count as debt when its `capped_at` is newer than the feature's best
    /// scribing stamp: this is the exact live scenario measured against
    /// `doc-viewer-links` (closed with "door scribing-debt: clear" while both
    /// of its behavior_change cells were uncaptured).
    #[test]
    fn scribing_debt_counts_a_cell_that_lives_only_in_the_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(
            &root,
            ".bee/cells/archive/demo/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let debt = scribing_debt(&root, "demo").unwrap();
        assert_eq!(debt.count, 1, "an archived-only cell must still count as debt");
        assert_eq!(debt.ids, vec![json!("demo-1")]);

        // A scribing run recorded AFTER the archived cell's cap still clears
        // it — an archived cell is counted by the same threshold rule as a
        // hot one, not a separate one.
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"demo\",\"ts\":\"2026-07-02T00:00:00.000Z\"}\n",
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 0);
    }

    /// An archived cell OLDER than the feature's best scribing stamp stays
    /// uncounted — the archive walk adds visibility, it never lowers the bar.
    #[test]
    fn scribing_debt_leaves_an_archived_cell_older_than_the_threshold_uncounted() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(
            &root,
            ".bee/cells/archive/demo/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"demo\",\"ts\":\"2026-07-02T00:00:00.000Z\"}\n",
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 0);
    }

    /// A cell id present in BOTH the hot dir and the archive counts ONCE, and
    /// the LIVE copy's own trace decides — not the archived copy's. Mirrors
    /// `verbs/knowledge/promote.rs:353-376`'s live-copy-wins dedup, pinned
    /// there by `verbs/knowledge/tests.rs:682`.
    #[test]
    fn scribing_debt_dedupes_by_id_with_the_live_copy_winning() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        // The archived copy is stale (old capped_at, would clear on its own).
        w(
            &root,
            ".bee/cells/archive/demo/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-01-01T00:00:00.000Z"}}"#,
        );
        // The live copy shares the id but carries a fresh trace: it must be
        // the one scribing_debt actually reads, so it counts.
        w(
            &root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let debt = scribing_debt(&root, "demo").unwrap();
        assert_eq!(debt.count, 1, "a duplicate id must be counted once, not twice");
        assert_eq!(debt.ids, vec![json!("demo-1")]);
    }

    /// `bee close`'s own door, `scribing_debt_close_door` and
    /// `scribing_debt_swap_door` (state_group/set_gate.rs) all call
    /// `drivers::scribing_debt` — there is one counter, so proving the door
    /// text here proves all three see the archived cell.
    #[test]
    fn close_door_reports_debt_for_a_behavior_change_cell_that_is_already_archived() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(
            &root,
            ".bee/cells/archive/demo/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#,
        );
        let doors = build_close_report_doors(&root, "demo").unwrap();
        let scribing = doors.iter().find(|d| d.door == "scribing-debt").unwrap();
        assert!(scribing.blocking, "an archived-only debt cell must still block close");
        assert_eq!(
            scribing.detail,
            "pending — 1 behavior_change cell(s) uncaptured (demo-1); run bee-capturing to record the capture, or log a decision tagged capture-deferral naming \"demo\" to defer it"
        );
    }

    // ── D1: the refusal itself (red, green-after-capture, green-with-deferral) ─

    fn declare_echo_test(tmp: &tempfile::TempDir) -> PathBuf {
        repo(tmp, r#"{"commands":{"test":"echo suite-green"}}"#)
    }

    /// RED: a behavior_change cell with no capture recorded and no logged
    /// capture-deferral decision refuses close, even with tests GREEN.
    #[test]
    fn close_refuses_uncaptured_behavior_change_cells() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = declare_echo_test(&tmp);
        w(&root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(&root, ".bee/cells/demo-5.json", r#"{"id":"demo-5","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-02T00:00:00.000Z"}}"#);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1, "capture debt refuses even though tests are green");
        let lines: Vec<&str> = text.split('\n').collect();
        // The refusal's pinned prefix — the message-contract test.
        assert!(
            lines[0].starts_with(CLOSE_CAPTURE_DEBT_PREFIX),
            "refusal headline must start with the pinned prefix: {}",
            lines[0]
        );
        assert_eq!(
            lines[0],
            "Capture debt for \"demo\" — close stops at the scribing-debt door: 2 behavior_change cell(s) uncaptured (demo-4, demo-5)."
        );
        // Both remedies are named.
        assert!(lines[1].contains("bee-capturing"), "{}", lines[1]);
        assert!(lines[1].contains("capture-deferral"), "{}", lines[1]);
        assert!(lines[2].starts_with("next:"));
        // Cells are NEVER archived on a refused close.
        assert!(root.join(".bee/cells/demo-4.json").exists());
        let doors = result.get("doors").unwrap().as_array().unwrap();
        assert_eq!(doors.iter().find(|d| d["door"] == "scribing-debt").unwrap()["blocking"], json!(true));
    }

    /// GREEN after capture: a scribing run recorded after the cap clears the
    /// debt and close proceeds to its normal green report.
    #[test]
    fn close_proceeds_green_once_capture_is_recorded() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = declare_echo_test(&tmp);
        w(&root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"demo\",\"ts\":\"2026-07-02T00:00:00.000Z\"}\n",
        );
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(_, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        assert!(text.starts_with("Tests GREEN for \"demo\""));
        assert!(!text.starts_with(CLOSE_CAPTURE_DEBT_PREFIX));
    }

    /// GREEN with a capture-deferral decision: the debt is unchanged, but a
    /// logged decision tagged capture-deferral naming the feature lifts the
    /// refusal and close proceeds.
    #[test]
    fn close_proceeds_green_with_a_capture_deferral_decision() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = declare_echo_test(&tmp);
        w(&root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(
            &root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-07-02T00:00:00.000Z\",\"decision\":\"defer capture for demo until next sprint\",\"rationale\":\"r\",\"tags\":[\"capture-deferral\"],\"scope\":\"repo\"}\n",
        );
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        assert!(text.starts_with("Tests GREEN for \"demo\""));
        assert!(!text.starts_with(CLOSE_CAPTURE_DEBT_PREFIX));
        let doors = result.get("doors").unwrap().as_array().unwrap();
        let scribing = doors.iter().find(|d| d["door"] == "scribing-debt").unwrap();
        assert_eq!(scribing["blocking"], json!(false));
        assert!(scribing["detail"].as_str().unwrap().starts_with("deferred —"));
        // The deferral only lifts the refusal — it does not touch the debt
        // count reported in the door text a moment earlier.
        assert!(scribing["detail"].as_str().unwrap().contains("1 behavior_change cell(s)"));
        // A green close still retires the feature's (now terminal) cells.
        assert_eq!(result.get("retired").unwrap().get("archived"), Some(&json!(true)));
    }

    /// A capture-deferral decision naming a DIFFERENT feature never lifts
    /// this feature's refusal.
    #[test]
    fn close_refuses_when_the_deferral_decision_names_another_feature() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = declare_echo_test(&tmp);
        w(&root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(
            &root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-07-02T00:00:00.000Z\",\"decision\":\"defer capture for elsewhere until next sprint\",\"rationale\":\"r\",\"tags\":[\"capture-deferral\"],\"scope\":\"repo\"}\n",
        );
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(_, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1);
        assert!(text.starts_with(CLOSE_CAPTURE_DEBT_PREFIX));
    }

    /// A decision naming the feature but WITHOUT the capture-deferral tag
    /// never lifts the refusal either.
    #[test]
    fn close_refuses_when_the_decision_lacks_the_capture_deferral_tag() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = declare_echo_test(&tmp);
        w(&root, ".bee/cells/demo-4.json", r#"{"id":"demo-4","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(
            &root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-07-02T00:00:00.000Z\",\"decision\":\"note about demo\",\"rationale\":\"r\",\"tags\":[\"other\"],\"scope\":\"repo\"}\n",
        );
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(_, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1);
        assert!(text.starts_with(CLOSE_CAPTURE_DEBT_PREFIX));
    }

    /// Regression: the scribing-debt door JOINS the cell ids, so listCells'
    /// numeric-aware localeCompare order reaches an emitted byte. A plain byte
    /// sort put "rust-port-5" after "rust-port-23" — caught by a live diff
    /// against the beehive repo itself.
    #[test]
    fn scribing_debt_ids_keep_numeric_aware_collation_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        for n in ["5", "7", "11", "23"] {
            w(&root, &format!(".bee/cells/f-{n}.json"), &format!(
                r#"{{"id":"f-{n}","feature":"demo","status":"capped","trace":{{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}}}"#
            ));
        }
        let debt = scribing_debt(&root, "demo").unwrap();
        assert_eq!(
            js_join(&debt.ids, ", "),
            "f-5, f-7, f-11, f-23",
            "numeric runs compare by value, not byte order"
        );
        // "01" and "1" are fully equal at every ICU level.
        assert_eq!(locale_cmp("a01", "a1", true), std::cmp::Ordering::Equal);
        assert_eq!(locale_cmp("_a", "-a", true), std::cmp::Ordering::Less);
    }

    #[test]
    fn state_last_scribing_run_raises_the_threshold_only_for_its_own_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(&root, ".bee/cells/c.json", r#"{"id":"c","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        w(
            &root,
            ".bee/state.json",
            r#"{"last_scribing_run":{"feature":"other","at":"2026-07-09T00:00:00.000Z"}}"#,
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 1);
        w(
            &root,
            ".bee/state.json",
            r#"{"last_scribing_run":{"feature":"demo","at":"2026-07-09T00:00:00.000Z"}}"#,
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 0);
    }

    /// The lane record's own `last_scribing_run` (`.bee/lanes/<feature>.json`)
    /// now joins the threshold beside the jsonl ledger and `state.json`'s —
    /// the freshest of the three wins, matching the ledger/lane/state order
    /// `status`'s own `best_scribing_stamp_ms` (verbs/status_full/cells.rs)
    /// already uses.
    #[test]
    fn a_lane_records_last_scribing_run_raises_the_threshold() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        w(&root, ".bee/cells/c.json", r#"{"id":"c","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-07-01T00:00:00.000Z"}}"#);
        // A lane record naming ANOTHER feature never raises this threshold.
        w(
            &root,
            ".bee/lanes/other.json",
            r#"{"feature":"other","last_scribing_run":{"feature":"other","at":"2026-07-09T00:00:00.000Z"}}"#,
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 1);
        // The feature's OWN lane record, stamped BEFORE the cell was capped:
        // still debt — the lane hasn't caught up to this capture yet.
        w(
            &root,
            ".bee/lanes/demo.json",
            r#"{"feature":"demo","last_scribing_run":{"feature":"demo","at":"2026-06-01T00:00:00.000Z"}}"#,
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 1);
        // Stamped AFTER: the cell was capped before the lane's last scribing
        // run, so it is no longer debt.
        w(
            &root,
            ".bee/lanes/demo.json",
            r#"{"feature":"demo","last_scribing_run":{"feature":"demo","at":"2026-07-09T00:00:00.000Z"}}"#,
        );
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 0);
    }

    /// A corrupt/unparseable lane record used to be exactly the shape the
    /// (now-removed) delegation guard existed for. It must not throw and
    /// must not stop close: `read_lane_display` warns (naming the path) and
    /// reads as "no lane", so the feature's threshold simply gets no lane
    /// contribution and the driver proceeds to completion.
    #[test]
    fn a_corrupt_lane_record_warns_and_close_still_runs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":["a"]}}"#);
        w(&root, ".bee/lanes/demo.json", "{broken");
        // The scribing-debt read itself must not error.
        assert_eq!(scribing_debt(&root, "demo").unwrap().count, 0);
        // The driver still runs to completion (dry-run door report), never a
        // thrown error and never a refusal.
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(_, text, code) =
            close_handler(&root, "demo", true, declared, None, &HashMap::new()).unwrap()
        else {
            panic!("a corrupt lane record must not stop close")
        };
        assert_eq!(code, 0);
        assert!(text.contains("door scribing-debt: clear"));
    }

    // ─── D1: knowledge-freshness close door ─────────────────────────────────

    /// Writes one area concept tagged `areas: [<area>]` whose `bee.sources`
    /// names `src/a.rs` — the same touched-file match `touched_bundle_areas`
    /// applies elsewhere — so a capped cell touching `src/a.rs` puts `<area>`
    /// in the door's scope, with no dangling pointer of its own.
    fn write_freshness_touch_anchor(root: &Path, area: &str) {
        // `src/a.rs` must be real on disk too — this concept's OWN sources
        // entry is otherwise itself a dangling_source finding the door
        // would (correctly) also see.
        w(root, "src/a.rs", "// placeholder\n");
        w(
            root,
            &format!("docs/knowledge/areas/{area}/overview.md"),
            &format!(
                "---\ntype: bee.area\ntitle: {area} area\ndescription: d\nbee:\n  id: {area}-area\n  lifecycle: active\n  areas: [{area}]\n  sources: [src/a.rs]\n---\nbody\n"
            ),
        );
    }

    /// A second concept living under `areas/<area>/` whose `bee.sources`
    /// names a path that does not exist on disk — the dangling_source the
    /// door's own scoping is meant to catch (or miss, when `<area>` is never
    /// touched).
    fn write_freshness_dangling(root: &Path, area: &str) {
        w(
            root,
            &format!("docs/knowledge/areas/{area}/dangling.md"),
            &format!(
                "---\ntype: bee.area\ntitle: {area} dangling\ndescription: d\nbee:\n  id: {area}-dangling\n  lifecycle: active\n  areas: [{area}]\n  sources: [does/not/exist.md]\n---\nbody\n"
            ),
        );
    }

    fn write_freshness_capped_cell(root: &Path, feature: &str, file: &str) {
        w(
            root,
            &format!(".bee/cells/{feature}-1.json"),
            &format!(
                r#"{{"id":"{feature}-1","feature":"{feature}","status":"capped","trace":{{"behavior_change":false,"outcome":"did the thing","files_changed":["{file}"],"capped_at":"2026-08-16T00:00:00.000Z"}}}}"#
            ),
        );
    }

    #[test]
    fn knowledge_freshness_door_blocks_on_a_dangling_source_in_a_touched_area() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_freshness_touch_anchor(root, "demo");
        write_freshness_dangling(root, "demo");
        write_freshness_capped_cell(root, "demo", "src/a.rs");
        let door = build_knowledge_freshness_door(root, "demo").unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("areas/demo/dangling.md"), "{}", door.detail);
        assert!(door.detail.contains("does/not/exist.md"), "{}", door.detail);
        assert!(door.detail.contains("remedy:"), "{}", door.detail);
    }

    #[test]
    fn knowledge_freshness_door_is_clear_when_the_dangling_source_is_outside_touched_areas() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_freshness_touch_anchor(root, "demo");
        // The dangling pointer lives under a DIFFERENT area — never touched.
        write_freshness_dangling(root, "other");
        write_freshness_capped_cell(root, "demo", "src/a.rs");
        let door = build_knowledge_freshness_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn knowledge_freshness_door_deferral_decision_demotes_to_non_blocking_with_the_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_freshness_touch_anchor(root, "demo");
        write_freshness_dangling(root, "demo");
        write_freshness_capped_cell(root, "demo", "src/a.rs");
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:00.000Z\",\"decision\":\"defer the demo stale pointer until the retired-path migration lands\",\"rationale\":\"r\",\"tags\":[\"knowledge-freshness-deferral\"],\"scope\":\"repo\"}\n",
        );
        let door = build_knowledge_freshness_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.starts_with("deferred —"), "{}", door.detail);
        assert!(
            door.detail.contains("defer the demo stale pointer until the retired-path migration lands"),
            "{}",
            door.detail
        );
    }

    #[test]
    fn knowledge_freshness_door_is_clear_with_no_bundle() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_freshness_capped_cell(root, "demo", "src/a.rs");
        let door = build_knowledge_freshness_door(root, "demo").unwrap();
        assert!(!door.blocking);
        assert_eq!(door.detail, "clear");
    }

    /// End-to-end: `close_handler` itself stops at the knowledge-freshness
    /// door — tests GREEN, past every earlier door — naming the file and the
    /// remedy in the refusal headline, exactly like the tests/scribing-debt/
    /// judge-debt/pattern-check refusals above it.
    #[test]
    fn close_refuses_at_the_knowledge_freshness_door() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        write_freshness_touch_anchor(&root, "demo");
        write_freshness_dangling(&root, "demo");
        write_freshness_capped_cell(&root, "demo", "src/a.rs");
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1);
        assert!(
            text.starts_with(&format!(
                "{CLOSE_KNOWLEDGE_FRESHNESS_PREFIX} \"demo\" — close stops at the knowledge-freshness door:"
            )),
            "{text}"
        );
        assert!(text.contains("areas/demo/dangling.md"), "{text}");
        assert!(text.contains("next: settle the stale pointer(s) above, then re-run bee close --feature demo"), "{text}");
        let doors = result.get("doors").unwrap().as_array().unwrap();
        let freshness = doors.iter().find(|d| d.get("door") == Some(&json!("knowledge-freshness"))).unwrap();
        assert_eq!(freshness.get("blocking"), Some(&json!(true)));
    }

    /// C1's generator/check fix (check.rs): a promoted delivery concept's
    /// `bee.required_context` names `docs/history/<feature>/...` paths —
    /// repo-root-relative, per `promote.rs`'s history-anchor arm — which
    /// used to be born dangling because `check_bundle` only ever tried the
    /// bundle-relative resolution. Bundle-first-then-repo-root now resolves
    /// it clean.
    #[test]
    fn check_bundle_resolves_a_history_anchor_required_context_at_the_repo_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/history/demo/CONTEXT.md", "# Demo context\n");
        w(root, "docs/history/demo/plan.md", "# Demo plan\n");
        w(
            root,
            "docs/knowledge/work/demo/delivery.md",
            "---\ntype: bee.delivery\ntitle: Demo delivery\ndescription: d\nbee:\n  id: demo-delivery\n  lifecycle: active\n  required_context: [docs/history/demo/CONTEXT.md, docs/history/demo/plan.md]\n---\nbody\n",
        );
        let dir = crate::verbs::knowledge::bundle_dir(root).unwrap();
        let report = crate::verbs::knowledge::check_bundle(&dir, false).unwrap();
        assert!(
            !report.warnings.iter().any(|w| w.get("code") == Some(&json!("dangling_required_context"))),
            "{:?}",
            report.warnings
        );
    }

    // ── routing (every delegating shape returns None before any output) ─────

    #[test]
    fn routing_serves_only_the_proven_shapes() {
        let os = |v: &[&str]| -> Vec<OsString> { v.iter().map(OsString::from).collect() };
        let t0 = Instant::now();
        // Not our verbs.
        assert!(try_native(&os(&["status"]), t0).is_none());
        assert!(try_native(&os(&["dispatch"]), t0).is_none());
        assert!(try_native(&os(&["dispatch", "guard"]), t0).is_none());
        // --help anywhere -> Node renders command-scoped help.
        assert!(try_native(&os(&["close", "--help"]), t0).is_none());
        assert!(try_native(&os(&["dispatch", "prepare", "--help"]), t0).is_none());
        // Stray positionals, unknown flags, missing/empty required flags.
        assert!(try_native(&os(&["close", "extra", "--feature", "f"]), t0).is_none());
        assert!(try_native(&os(&["close", "--feature", "f", "--wat", "x"]), t0).is_none());
        assert!(try_native(&os(&["close"]), t0).is_none());
        assert!(try_native(&os(&["close", "--feature="]), t0).is_none());
        assert!(try_native(&os(&["close", "--feature", "f", "--dry-run=maybe"]), t0).is_none());
        // dispatch: `--claim` is NATIVE at R6, so the shapes that still go
        // back to Node are the UNPROVEN spellings only — a `--claim` value
        // that is not `true`/`false` (validate()'s own message) and a bare
        // `--session-id` under --claim (`String(true)` as a session id).
        assert!(try_native(
            &os(&["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--cell", "c", "--worker", "w", "--claim=maybe"]),
            t0
        )
        .is_none());
        // --session-id WITHOUT --claim is documented as ignored — unproven.
        assert!(try_native(
            &os(&["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--session-id", "s"]),
            t0
        )
        .is_none());
        assert!(try_native(
            &os(&["dispatch", "prepare", "--runtime", "banana", "--kind", "gather"]),
            t0
        )
        .is_none());
        assert!(try_native(
            &os(&["dispatch", "prepare", "--runtime", "claude", "--kind", "banana"]),
            t0
        )
        .is_none());
        assert!(try_native(&os(&["dispatch", "prepare", "--kind", "gather"]), t0).is_none());
        assert!(try_native(&os(&["dispatch", "prepare", "--runtime", "claude"]), t0).is_none());
    }

    /// CUTOVER: `feature_has_lane_record` used to gate `run_close` — any
    /// feature carrying `.bee/lanes/<feature>.json` delegated the WHOLE
    /// command before `close_handler` ever ran. That guard is gone: the
    /// close driver now runs natively for a lane feature exactly like any
    /// other, whether the record is present, absent, or belongs to a
    /// DIFFERENT feature. `close_handler` — the driver itself — is the
    /// right level to prove this at, since the removed guard sat one layer
    /// above it (in `run_close`, which resolves the store root off the real
    /// process cwd and so is not unit-testable without it).
    #[test]
    fn a_features_own_lane_record_no_longer_delegates_close() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":["a","b"]}}"#);
        // A DIFFERENT feature's lane record was always irrelevant.
        w(&root, ".bee/lanes/other.json", r#"{"feature":"other"}"#);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(_, without_lane, without_code) =
            close_handler(&root, "demo", true, declared.clone(), None, &HashMap::new()).unwrap()
        else {
            panic!("close_handler must run the driver, not refuse")
        };
        // Now "demo" gets its own lane record — the exact shape that used to
        // delegate the whole command before `close_handler` was reached.
        w(&root, ".bee/lanes/demo.json", r#"{"feature":"demo","phase":"executing"}"#);
        let Out::Emit(_, with_lane, with_code) =
            close_handler(&root, "demo", true, declared, None, &HashMap::new()).unwrap()
        else {
            panic!("close still runs — a lane record is not a refusal")
        };
        // The dry-run door report is unaffected by the lane record's mere
        // presence (no scribing debt either way here) — truths #4.
        assert_eq!(without_lane, with_lane);
        assert_eq!(without_code, with_code);
        assert_eq!(with_code, 0);
    }

    #[test]
    fn prompt_skew_delegates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        // No on-disk prompts: the embedded copy is the only one.
        assert!(prompts_match_disk(&root, "gather"));
        // A vendored copy that MATCHES is fine (CRLF normalization included).
        w(&root, ".bee/bin/prompts/gather.md", &PROMPT_GATHER.replace('\n', "\r\n"));
        assert!(prompts_match_disk(&root, "gather"));
        // A skewed vendored copy delegates.
        w(&root, ".bee/bin/prompts/gather.md", "Gather: something else\n");
        assert!(!prompts_match_disk(&root, "gather"));
    }

    #[test]
    fn char_slicing_follows_the_char_contract() {
        use crate::textutil::{truncate_chars_head, truncate_chars_tail};
        assert_eq!(truncate_chars_head("abcdef", 3), "abc");
        assert_eq!(truncate_chars_head("abc", 10), "abc");
        assert_eq!(truncate_chars_tail(&"x".repeat(650), 500).chars().count(), 500);
        // Decision D3: the cap counts CHARS, not UTF-16 units — an astral
        // char is one char, so a 5-char head of 4 astral chars is the whole
        // string untouched (it would have been cut mid-pair under the old
        // UTF-16-unit contract).
        let astral = "🐝".repeat(4);
        assert_eq!(truncate_chars_head(&astral, 5), astral);
        assert_eq!(truncate_chars_head(&astral, 5).chars().count(), 4);
    }

    // ─── doc-impact-synthesis D1b: impact door at close ─────────────────────

    /// One feature-stamped `decide` event, tagged onto the closing feature by
    /// kds-1's structured `feature` field — never by text scan. `id`'s first
    /// 8 chars (the short8 `sweep_decision_citations` also keys on) are
    /// exactly `aaaaaaaa`.
    fn write_impact_decision(root: &Path, feature: &str) {
        w(
            root,
            ".bee/decisions.jsonl",
            &format!(
                "{{\"id\":\"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:00.000Z\",\"decision\":\"d\",\"rationale\":\"r\",\"scope\":\"repo\",\"feature\":\"{feature}\"}}\n"
            ),
        );
    }

    #[test]
    fn impact_door_refuses_when_an_outside_doc_cites_a_feature_stamped_decision() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_impact_decision(root, "demo");
        w(root, "docs/knowledge/areas/other/notes.md", "cites decision aaaaaaaa here\n");
        let door = build_impact_door(root, "demo").unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/other/notes.md:1"), "{}", door.detail);
        assert!(door.detail.contains("remedy:"), "{}", door.detail);
        assert!(door.detail.contains("re-run bee close --feature demo"), "{}", door.detail);
    }

    /// v1's rejected flush-coverage design tied the door to a persisted
    /// stub queue — a hit written after log time never had one. This door
    /// re-derives its findings fresh every call, so fixing the citing doc
    /// clears it on the very next check with no stub to reconcile.
    #[test]
    fn impact_door_clears_after_fixing_the_citing_doc() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_impact_decision(root, "demo");
        w(root, "docs/knowledge/areas/other/notes.md", "cites decision aaaaaaaa here\n");
        assert!(build_impact_door(root, "demo").unwrap().blocking);
        w(root, "docs/knowledge/areas/other/notes.md", "no citation left\n");
        let door = build_impact_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn impact_door_excludes_the_generated_index_and_the_feature_own_history() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_impact_decision(root, "demo");
        w(root, "docs/decisions/index.md", "cites decision aaaaaaaa here\n");
        w(root, "docs/history/demo/CONTEXT.md", "cites decision aaaaaaaa here\n");
        let door = build_impact_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    /// The write-guard's own generated/vendored tree list (`SCOUT_DIRS`,
    /// hooks/write_guard/guards.rs) is reused verbatim rather than
    /// hand-copied — a hit under `docs/vendor/` never blocks.
    #[test]
    fn impact_door_excludes_the_write_guards_generated_tree_list() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_impact_decision(root, "demo");
        w(root, "docs/vendor/upstream.md", "cites decision aaaaaaaa here\n");
        let door = build_impact_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn impact_door_deferral_decision_demotes_to_non_blocking_with_the_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_impact_decision(root, "demo");
        w(root, "docs/knowledge/areas/other/notes.md", "cites decision aaaaaaaa here\n");
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:00.000Z\",\"decision\":\"d\",\"rationale\":\"r\",\"scope\":\"repo\",\"feature\":\"demo\"}\n{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:01.000Z\",\"decision\":\"defer the demo citation cleanup until the doc rewrite lands\",\"rationale\":\"r\",\"tags\":[\"impact-deferral\"],\"scope\":\"repo\"}\n",
        );
        let door = build_impact_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.starts_with("deferred —"), "{}", door.detail);
        assert!(
            door.detail.contains("defer the demo citation cleanup until the doc rewrite lands"),
            "{}",
            door.detail
        );
    }

    #[test]
    fn impact_door_is_clear_with_no_feature_stamped_decisions() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let door = build_impact_door(root, "demo").unwrap();
        assert!(!door.blocking);
        assert_eq!(door.detail, "clear");
    }

    /// D1b's named deviation: NO time-window fallback exists — a decision
    /// naming "demo" only in its prose text, with no structured `feature`
    /// field, is never collected. Structured field ONLY.
    #[test]
    fn impact_door_never_walks_a_decision_with_no_structured_feature_field() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:00.000Z\",\"decision\":\"something about demo\",\"rationale\":\"r\",\"scope\":\"repo\"}\n",
        );
        w(root, "docs/knowledge/areas/other/notes.md", "cites decision aaaaaaaa here\n");
        let door = build_impact_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    /// End-to-end: `close_handler` stops at the impact door — tests GREEN,
    /// past every earlier door — naming the file and the remedy in the
    /// refusal headline, exactly like the knowledge-freshness refusal above.
    #[test]
    fn close_refuses_at_the_impact_door() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        write_impact_decision(&root, "demo");
        w(&root, "docs/knowledge/areas/other/notes.md", "cites decision aaaaaaaa here\n");
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1);
        assert!(
            text.starts_with(&format!("{CLOSE_IMPACT_PREFIX} \"demo\" — close stops at the impact door:")),
            "{text}"
        );
        assert!(text.contains("docs/knowledge/areas/other/notes.md:1"), "{text}");
        assert!(text.contains("next: settle the citation(s) above, then re-run bee close --feature demo"), "{text}");
        let doors = result.get("doors").unwrap().as_array().unwrap();
        let impact = doors.iter().find(|d| d.get("door") == Some(&json!("impact"))).unwrap();
        assert_eq!(impact.get("blocking"), Some(&json!(true)));
    }

    // ─── doc-impact-synthesis D2/D3: context-table parser, routing door,
    //     doc-deferral door ────────────────────────────────────────────────

    /// The canonical template's own `## Locked Decisions` table
    /// (`.claude/skills/bee-shaping/references/context-template.md:26`)
    /// must parse — the routing door's grammar IS the template's grammar.
    const CONTEXT_TEMPLATE: &str = include_str!(
        "../../../../../../../.claude/skills/bee-shaping/references/context-template.md"
    );

    /// This feature's own `docs/history/doc-impact-synthesis/CONTEXT.md`
    /// must parse too — the door that would refuse every OTHER feature's
    /// legacy CONTEXT is bound by its own grammar first.
    const THIS_FEATURE_CONTEXT: &str = include_str!(
        "../../../../../../../docs/history/doc-impact-synthesis/CONTEXT.md"
    );

    #[test]
    fn context_table_parses_the_canonical_template_header() {
        assert_eq!(
            parse_locked_decision_ids(CONTEXT_TEMPLATE),
            Some(vec!["D1".to_string(), "D2".to_string()])
        );
    }

    #[test]
    fn context_table_parses_this_features_own_context_md() {
        assert_eq!(
            parse_locked_decision_ids(THIS_FEATURE_CONTEXT),
            Some(vec!["D1".to_string(), "D2".to_string(), "D3".to_string(), "D4".to_string()])
        );
    }

    #[test]
    fn context_table_is_none_for_a_legacy_bullet_context() {
        let text = "# Old Feature — Context\n\n## Locked Decisions\n\n- D1: bullet form decision.\n- D2: another bullet decision.\n";
        assert_eq!(parse_locked_decision_ids(text), None);
    }

    #[test]
    fn context_table_covers_d_id_matches_the_plain_form() {
        let text = "prose then demo D2 lives here\n";
        assert!(context_table_covers_d_id(text, "demo", 2));
        assert!(!context_table_covers_d_id(text, "demo", 3));
    }

    #[test]
    fn context_table_covers_d_id_matches_the_range_form() {
        let text = "prose then demo D1-D3 lives here\n";
        assert!(context_table_covers_d_id(text, "demo", 1));
        assert!(context_table_covers_d_id(text, "demo", 2));
        assert!(context_table_covers_d_id(text, "demo", 3));
        assert!(!context_table_covers_d_id(text, "demo", 4));
    }

    #[test]
    fn context_table_covers_d_id_matches_the_slash_form() {
        let text = "prose then demo D1/D3 lives here\n";
        assert!(context_table_covers_d_id(text, "demo", 1));
        assert!(!context_table_covers_d_id(text, "demo", 2));
        assert!(context_table_covers_d_id(text, "demo", 3));
    }

    fn write_locked_decisions_table(root: &Path, feature: &str, ids: &[&str]) {
        let mut body = "# Demo — Context\n\n## Locked Decisions\n\n| ID | Decision | Rationale (only if it changes implementation) |\n|----|----------|-----------------------------------------------|\n".to_string();
        for id in ids {
            body.push_str(&format!("| {id} | placeholder decision text | - |\n"));
        }
        w(root, &format!("docs/history/{feature}/CONTEXT.md"), &body);
    }

    #[test]
    fn routing_door_routes_the_plain_range_and_slash_citation_forms() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_locked_decisions_table(root, "demo", &["D1", "D2", "D3", "D4", "D5"]);
        w(root, "docs/knowledge/areas/a/plain.md", "demo D1 lives here\n");
        w(root, "docs/knowledge/areas/a/range.md", "demo D2-D3 lives here\n");
        w(root, "docs/knowledge/areas/a/slash.md", "demo D4/D5 lives here\n");
        let door = build_routing_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear — every locked D-ID routed");
    }

    #[test]
    fn routing_door_routes_via_the_decisions_logged_short8() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_locked_decisions_table(root, "demo", &["D1"]);
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"aaaa1111-bbbb-cccc-dddd-eeeeeeeeeeee\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:00.000Z\",\"decision\":\"demo D1: something locked\",\"rationale\":\"r\",\"scope\":\"repo\"}\n",
        );
        w(root, "docs/knowledge/areas/a/short8.md", "cites decision aaaa1111 here\n");
        let door = build_routing_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear — every locked D-ID routed");
    }

    #[test]
    fn routing_door_blocks_on_an_unrouted_d_id() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_locked_decisions_table(root, "demo", &["D1"]);
        let door = build_routing_door(root, "demo").unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("D1"), "{}", door.detail);
        assert!(door.detail.contains("cite"), "{}", door.detail);
        assert!(door.detail.contains("feature-local"), "{}", door.detail);
    }

    #[test]
    fn routing_door_clears_via_a_feature_local_tagged_decision() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_locked_decisions_table(root, "demo", &["D1"]);
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"b1\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:00.000Z\",\"decision\":\"demo D1: kept as feature-local, no separate area yet\",\"rationale\":\"r\",\"tags\":[\"feature-local\"],\"scope\":\"repo\"}\n",
        );
        let door = build_routing_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear — every locked D-ID routed");
    }

    #[test]
    fn routing_door_multi_area_citation_is_a_report_only_warning() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_locked_decisions_table(root, "demo", &["D1"]);
        w(root, "docs/knowledge/areas/a/one.md", "demo D1 lives here\n");
        w(root, "docs/knowledge/areas/b/two.md", "demo D1 also lives here\n");
        let door = build_routing_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.contains("clear"), "{}", door.detail);
        assert!(door.detail.contains("duplication"), "{}", door.detail);
        assert!(door.detail.contains("report-only"), "{}", door.detail);
    }

    #[test]
    fn routing_door_legacy_context_yields_a_loud_notice_never_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            "docs/history/demo/CONTEXT.md",
            "# Demo — Context\n\n## Locked Decisions\n\n- D1: bullet form decision.\n",
        );
        let door = build_routing_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.starts_with("NOTICE"), "{}", door.detail);
        assert!(door.detail.contains("legacy"), "{}", door.detail);
        assert!(door.detail.contains("D4"), "{}", door.detail);
    }

    #[test]
    fn routing_door_deferral_decision_demotes_to_non_blocking_with_the_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_locked_decisions_table(root, "demo", &["D1"]);
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"c1\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:00.000Z\",\"decision\":\"defer routing cleanup for demo until the next pass\",\"rationale\":\"r\",\"tags\":[\"routing-deferral\"],\"scope\":\"repo\"}\n",
        );
        let door = build_routing_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.starts_with("deferred —"), "{}", door.detail);
        assert!(
            door.detail.contains("defer routing cleanup for demo until the next pass"),
            "{}",
            door.detail
        );
    }

    #[test]
    fn close_refuses_at_the_routing_door() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        write_locked_decisions_table(&root, "demo", &["D1"]);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1);
        assert!(
            text.starts_with(&format!("{CLOSE_ROUTING_PREFIX} \"demo\" — close stops at the routing door:")),
            "{text}"
        );
        assert!(text.contains("next: settle the unrouted D-ID(s) above, then re-run bee close --feature demo"), "{text}");
        let doors = result.get("doors").unwrap().as_array().unwrap();
        let routing = doors.iter().find(|d| d.get("door") == Some(&json!("routing"))).unwrap();
        assert_eq!(routing.get("blocking"), Some(&json!(true)));
    }

    // ─── doc-impact-synthesis D3: doc-deferral door ─────────────────────────

    fn write_trigger(root: &Path, id: &str) {
        std::fs::create_dir_all(root.join(".git")).unwrap();
        let dir = root.join(".bee").join("triggers");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                r#"{{"id":"{id}","decision":"deadbeef","condition":"upstream lands","tier":"manual","predicate":null,"status":"waiting","created_at":"2026-08-16T00:00:00.000Z","updated_at":"2026-08-16T00:00:00.000Z","outcome":null}}"#
            ),
        )
        .unwrap();
    }

    #[test]
    fn doc_deferral_door_blocks_a_deferral_line_with_no_registered_trigger() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo").unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:2"), "{}", door.detail);
        assert!(door.detail.contains("remedy:"), "{}", door.detail);
    }

    #[test]
    fn doc_deferral_door_clears_with_a_registered_trigger_citation() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_trigger(root, "demo-trigger");
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "line one\nThis work is deferred for now, see `demo-trigger`.\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn doc_deferral_door_exempts_fenced_code() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "prose\n```\nThis work is deferred for now.\n```\nmore prose\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    /// CONTEXT.md is written at shaping, before any cell exists — the scan
    /// set is the UNION of capped-cell `files_changed` and every file on
    /// disk under `docs/history/<feature>/`, so a feature with ZERO capped
    /// cells still has its own CONTEXT.md scanned.
    #[test]
    fn doc_deferral_door_scan_set_includes_the_on_disk_context_md_with_no_capped_cells() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/history/demo/CONTEXT.md", "# Demo\n\nThis is deferred for now.\n");
        let door = build_doc_deferral_door(root, "demo").unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/history/demo/CONTEXT.md:3"), "{}", door.detail);
    }

    #[test]
    fn doc_deferral_door_deferral_decision_demotes_to_non_blocking_with_the_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:00.000Z\",\"decision\":\"defer registering a trigger for demo until the audit lands\",\"rationale\":\"r\",\"tags\":[\"doc-deferral\"],\"scope\":\"repo\"}\n",
        );
        let door = build_doc_deferral_door(root, "demo").unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.starts_with("deferred —"), "{}", door.detail);
        assert!(
            door.detail.contains("defer registering a trigger for demo until the audit lands"),
            "{}",
            door.detail
        );
    }

    #[test]
    fn close_refuses_at_the_doc_deferral_door() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        w(&root, "docs/history/demo/CONTEXT.md", "# Demo\n\nThis is deferred for now.\n");
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell), &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1);
        assert!(
            text.starts_with(&format!("{CLOSE_DOC_DEFERRAL_PREFIX} \"demo\" — close stops at the doc-deferral door:")),
            "{text}"
        );
        assert!(text.contains("next: settle the deferral line(s) above, then re-run bee close --feature demo"), "{text}");
        let doors = result.get("doors").unwrap().as_array().unwrap();
        let doc_deferral = doors.iter().find(|d| d.get("door") == Some(&json!("doc-deferral"))).unwrap();
        assert_eq!(doc_deferral.get("blocking"), Some(&json!(true)));
    }
