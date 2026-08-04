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
use std::collections::HashSet;
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
        match kctx::parse_scalar_token(&raw, 7) {
            Err(kctx::Fm::Failed { code, line, .. }) => {
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
            resolve_tier(&m, "generation", "claude", false),
            Resolved::Model { model: "sonnet".into(), effort: None }
        );
        assert_eq!(
            resolve_tier(&m, "review", "claude", true),
            Resolved::Model { model: "opus".into(), effort: None }
        );
        // codex defaults are null -> budget; review falls back to generation
        // (also null) -> budget.
        assert_eq!(resolve_tier(&m, "generation", "codex", false), Resolved::Budget);
        assert_eq!(resolve_tier(&m, "review", "codex", true), Resolved::Budget);
        // ceiling is never configured.
        assert_eq!(resolve_tier(&m, "ceiling", "claude", false), Resolved::Inherit);
        // An unknown slot ('advisor') coerces to generation — the trap
        // resolveAdvisor exists to avoid.
        assert_eq!(
            resolve_tier(&m, "advisor", "claude", true),
            Resolved::Model { model: "sonnet".into(), effort: None }
        );

        // review: null falls back to the generation tier BEFORE the cli check.
        let m = models_from(
            r#"{"claude":{"generation":{"kind":"cli","command":"glm run"},"review":null}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "review", "claude", true),
            Resolved::Cli { command: "glm run".into() }
        );
        // cli + cell purpose -> typed refusal naming the RESOLVED slot.
        assert_eq!(
            resolve_tier(&m, "review", "claude", false),
            Resolved::Refused { slot: "review".into() }
        );

        // {model, effort}
        let m = models_from(r#"{"claude":{"generation":{"model":"opus","effort":"high"}}}"#);
        assert_eq!(
            resolve_tier(&m, "generation", "claude", false),
            Resolved::Model { model: "opus".into(), effort: Some("high".into()) }
        );
        // An invalid effort is dropped by normalize.
        let m = models_from(r#"{"claude":{"generation":{"model":"opus","effort":"turbo"}}}"#);
        assert_eq!(
            resolve_tier(&m, "generation", "claude", false),
            Resolved::Model { model: "opus".into(), effort: None }
        );

        // native leaf + explicit-only cli fallback composite
        let m = models_from(
            r#"{"codex":{"generation":{"primary":{"kind":"native","model":"gpt-5","effort":"high"},"fallback_policy":"explicit-only","fallback":{"kind":"cli","command":"codex exec"}}}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "generation", "codex", false),
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
        match resolve_tier(&m, "generation", "codex", false) {
            Resolved::Native { fallback, .. } => assert_eq!(fallback, None),
            other => panic!("expected native, got {other:?}"),
        }
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
    }

    #[test]
    fn pinned_types_match_the_rendered_bee_agents() {
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
        let out =
            prepare_dispatch(&root, "claude", "gather", None, None, false, None, false).unwrap();
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
    fn recording_pass_appends_exactly_one_prepare_line() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        prepare_dispatch(&root, "claude", "gather", None, None, false, None, true).unwrap();
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
            prepare_dispatch(&root, "claude", "advisor", None, None, false, None, false).unwrap()
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
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, false)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("reason"), Some(&json!("cli_tier_gather_only")));
        assert_eq!(v.get("slot"), Some(&json!("generation")));
        assert_eq!(v.get("fix"), Some(&json!(CLI_REFUSAL_FIX)));

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "gather", None, None, false, None, false).unwrap()
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
                prepare_dispatch(&root, "claude", "cell", None, Some("w"), false, None, false)
                    .unwrap()
            ),
            "dispatch prepare: --cell is required when --kind cell."
        );
        assert_eq!(
            thrown(
                prepare_dispatch(
                    &root, "claude", "cell", Some("ghost"), Some("w"), false, None, false
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
                    &root, "claude", "cell", Some("c-1"), Some("   "), false, None, false
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
            &root, "claude", "cell", Some("c-1"), Some("thief"), true, None, true,
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
            &root, "claude", "cell", Some("c-1"), Some("owner"), true, None, false,
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

    /// THE CROSS-CHECK the port rules require: the `kctx` lift must produce
    /// the SAME manifest as the shipped `bee knowledge context` verb (whose own
    /// copy lives in verbs/knowledge.rs) and as Node's lib/knowledge.mjs
    /// buildContextManifest. The golden below was captured from BOTH runtimes
    /// on this exact fixture — `node bee.mjs knowledge context --work demo
    /// --budget 20000 --json` and `bee.exe knowledge context --work demo
    /// --budget 20000 --json` printed it byte-for-byte — so a drift in either
    /// Rust copy, or from the .mjs, fails here.
    #[test]
    fn learned_context_agrees_with_the_knowledge_verb_port() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        bundle_fixture(&root);
        let dir = kctx::bundle_dir(&root).unwrap();
        let kctx::ManifestOut::Built(manifest) =
            kctx::build_context_manifest(&dir, "demo", 20000.0, &kctx::num(20000.0))
        else {
            panic!("expected a built manifest")
        };
        const GOLDEN: &str = concat!(
            r#"{"work":"demo","decisions":["d-1"],"budget":20000,"estimator":"bytes/4","#,
            r#""total_est":240,"entries":["#,
            r#"{"path":"docs/knowledge/work/demo/work-item.md","bytes":232,"est_tokens":58,"reason":"work item"},"#,
            r#"{"path":"docs/knowledge/work/demo/plan.md","bytes":124,"est_tokens":31,"reason":"plan sibling in work/demo/"},"#,
            r#"{"path":"docs/knowledge/patterns/dispatch-prompt.md","bytes":252,"est_tokens":63,"reason":"critical pattern (relevance 0.508333, rank 1 of 2, floor)"},"#,
            r#"{"path":"docs/knowledge/patterns/unrelated.md","bytes":195,"est_tokens":49,"reason":"critical pattern (relevance 0, rank 2 of 2, floor)"},"#,
            r#"{"path":"docs/knowledge/areas/dispatch.md","bytes":156,"est_tokens":39,"reason":"decision for area dispatch"}"#,
            r#"],"truncated":[],"excluded":[],"#,
            r#""floor":["docs/knowledge/patterns/dispatch-prompt.md","docs/knowledge/patterns/unrelated.md"],"#,
            r#""critical_total":2,"zero_signal_count":1}"#,
        );
        assert_eq!(jsjson::stringify(&manifest), GOLDEN);
    }

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

    #[test]
    fn close_dry_run_reports_the_doors_and_runs_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":["a","b"]}}"#);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", true, declared, None).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        assert_eq!(
            text,
            concat!(
                "door tests: open — commands.test declared (2 command(s)) — close runs the full declared suite fresh; a stale test-results record is never trusted | settle: bee test\n",
                "door scribing-debt: clear\n",
                "door capture-queue: clear\n",
                "next: bee close --feature demo — runs the declared tests and reports"
            )
        );
        assert_eq!(result.get("feature"), Some(&json!("demo")));
        // Nothing ran: no record file.
        assert!(!root.join(".bee/logs/test-results.json").exists());

        // Undeclared repo: the teaching detail + a different next line.
        w(&root, ".bee/config.json", "{}");
        let Out::Emit(_, text, _) = close_handler(&root, "demo", true, None, None).unwrap() else {
            panic!()
        };
        assert!(text.starts_with(&format!("door tests: open — {CLOSE_TESTS_UNDECLARED_DETAIL}\n")));
        assert!(text.ends_with(
            "next: feature \"demo\" has no test door — close proceeds; capture stays pending for bee-capturing"
        ));
    }

    #[test]
    fn close_green_reports_the_capture_checklist() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"}}"#);
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell)).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0);
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(
            lines[0],
            "Tests GREEN for \"demo\" — 1 command(s) passed (record: .bee/logs/test-results.json)."
        );
        assert!(lines[1].starts_with("✓ echo suite-green ("));
        assert_eq!(
            lines[2],
            "Capture (deferred, decision c8e25271): scribing clear; capture queue clear."
        );
        assert_eq!(
            lines[3],
            "next: done — capture is recorded as pending (run bee-capturing whenever; orient keeps the reminder)."
        );
        assert_eq!(result.get("ran_tests"), Some(&json!(true)));
        assert_eq!(
            result.get("tests").unwrap().get("results"),
            Some(&json!(".bee/logs/test-results.json"))
        );
        // The run is FRESH: the record exists and is green.
        let record: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".bee/logs/test-results.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(record.get("green"), Some(&json!(true)));
    }

    #[test]
    fn close_red_stops_at_the_tests_door_and_exits_one() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"commands":{"test":["echo boom-line; echo more 1>&2; exit 3","echo second-ok"]}}"#,
        );
        let declared = declared_test_commands(&root).unwrap();
        let Out::Emit(result, text, code) =
            close_handler(&root, "demo", false, declared, Some(shell)).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1);
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(
            lines[0],
            "Tests RED for \"demo\" — close stops at the tests door (record: .bee/logs/test-results.json):"
        );
        assert!(lines[1].starts_with("✗ echo boom-line; echo more 1>&2; exit 3 ("));
        assert!(lines[1].ends_with(", exit 3)"));
        assert!(lines[2].starts_with("✓ echo second-ok ("));
        assert_eq!(lines[3], "--- echo boom-line; echo more 1>&2; exit 3 (exit 3) ---");
        assert_eq!(lines[4], "boom-line");
        assert_eq!(lines[5], "more");
        assert_eq!(
            lines[6],
            "next: the red is the work — fix it (boom-line), then re-run bee close --feature demo"
        );
        // The record is STILL written on a red (a red is a normal result).
        assert!(root.join(".bee/logs/test-results.json").exists());
        let doors = result.get("doors").unwrap().as_array().unwrap();
        assert_eq!(doors[0].get("blocking"), Some(&json!(true)));
        assert_eq!(
            doors[0].get("detail"),
            Some(&json!("the declared test run is RED (1 of 2 command(s) failed; record: .bee/logs/test-results.json)"))
        );
        // The report-only doors are never blocking, even beside a red.
        assert_eq!(doors[1].get("blocking"), Some(&json!(false)));
        assert_eq!(doors[2].get("blocking"), Some(&json!(false)));
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
            close_handler(&root, "demo", false, declared, Some(shell)).unwrap()
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
            close_handler(&root, "demo", false, declared, Some(shell)).unwrap()
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
            close_handler(&root, "demo", false, declared, Some(shell)).unwrap()
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
            close_handler(&root, "demo", false, declared, Some(shell)).unwrap()
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
            close_handler(&root, "demo", false, declared, Some(shell)).unwrap()
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
            close_handler(&root, "demo", true, declared, None).unwrap()
        else {
            panic!("a corrupt lane record must not stop close")
        };
        assert_eq!(code, 0);
        assert!(text.contains("door scribing-debt: clear"));
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
            close_handler(&root, "demo", true, declared.clone(), None).unwrap()
        else {
            panic!("close_handler must run the driver, not refuse")
        };
        // Now "demo" gets its own lane record — the exact shape that used to
        // delegate the whole command before `close_handler` was reached.
        w(&root, ".bee/lanes/demo.json", r#"{"feature":"demo","phase":"executing"}"#);
        let Out::Emit(_, with_lane, with_code) =
            close_handler(&root, "demo", true, declared, None).unwrap()
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
