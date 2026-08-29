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
        // model-role-split D2 (store 06e49368) RETARGETED: 'advisor' used to
        // coerce to generation and hand back sonnet — the trap resolveAdvisor
        // exists to avoid. A name now resolves its own slot or nothing, so an
        // unconfigured advisor is Budget.
        assert_eq!(resolve_tier(&m, "advisor", "claude", "gather"), Resolved::Budget);

        // review: null falls back to the generation tier BEFORE the cli check.
        let m = models_from(
            r#"{"claude":{"generation":{"kind":"cli","command":"glm run"},"review":null}}"#,
        );
        assert_eq!(
            resolve_tier(&m, "review", "claude", "gather"),
            Resolved::Cli { command: "glm run".into() }
        );
        // cli + cell purpose -> typed refusal naming the RESOLVED slot.
        // model-role-split D2 RETARGETED: the null review yields to
        // generation, and generation is the slot that carries the cli value,
        // so the refusal names the slot actually read rather than the one
        // asked for.
        assert_eq!(
            resolve_tier(&m, "review", "claude", "cell"),
            Resolved::Refused { slot: "generation".into() }
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

    /// model-role-split D2 (store 06e49368): a consumer names an ORDERED LIST
    /// of role names; the first that resolves wins, an unset or unresolvable
    /// name yields to the next, and the last entry always resolves.
    #[test]
    fn resolve_role_walks_the_list_and_refuses_to_guess_an_unknown_name() {
        let m = models_from(r#"{"claude":{"test":"opus","generation":"sonnet"}}"#);
        // A role name bee never shipped is legal the moment the config
        // carries it — there is no membership list left to be added to.
        assert_eq!(
            resolve_role(&m, &["test", "generation"], "claude", "cell"),
            Resolved::Model { model: "opus".into(), effort: None }
        );
        // A configured role is obeyed EXACTLY (decision 72f3d6dd): the walk
        // never looks past a name that resolves.
        assert_eq!(
            resolve_role(&m, &["test"], "claude", "cell"),
            Resolved::Model { model: "opus".into(), effort: None }
        );

        // THE FIX. A misspelling used to be coerced to generation and
        // dispatched sonnet while prepare.rs stamped tier_source:"cell" — a
        // wrong-model dispatch that completed clean. It yields instead: to
        // the next entry when there is one...
        assert_eq!(
            resolve_role(&m, &["tset", "generation"], "claude", "cell"),
            Resolved::Model { model: "sonnet".into(), effort: None }
        );
        // ...and to no model at all when there is not. Nothing in the config
        // names `tset`, so nothing resolves for `tset`.
        assert_eq!(resolve_role(&m, &["tset"], "claude", "cell"), Resolved::Budget);

        // The last entry cannot dead-end: a name the table does not carry
        // falls to bee's own built-in default for that name.
        let bare: Map<String, Value> = Map::new();
        assert_eq!(
            resolve_role(&bare, &["code", "generation"], "claude", "cell"),
            Resolved::Model { model: "sonnet".into(), effort: None }
        );
        // A one-entry list is just a walk of length one, and an empty list
        // asks for nothing rather than panicking.
        assert_eq!(
            resolve_role(&bare, &["generation"], "claude", "cell"),
            Resolved::Model { model: "sonnet".into(), effort: None }
        );
        assert_eq!(resolve_role(&m, &[], "claude", "cell"), Resolved::Budget);

        // An explicitly nulled slot yields like an unset one, and is NOT
        // answered with the built-in default the config just cleared.
        let off = models_from(r#"{"claude":{"extraction":null,"generation":null,"review":null}}"#);
        assert_eq!(resolve_role(&off, &["generation"], "claude", "cell"), Resolved::Budget);
        assert_eq!(
            resolve_role(&off, &["review", "generation"], "claude", "gather"),
            Resolved::Budget
        );
        // RETARGETED by D5 (store `97ce5225`), not weakened. `ceiling` used
        // to end this walk on its own — the one closed word inside the open
        // role set. It is no longer a role at all (escalation is a flag on
        // the cell), so the open walk has no exception left: the name is
        // just a role nothing configures and it yields to the next entry
        // exactly like `tset` above. Decision `0015` is preserved rather
        // than reopened — `ceiling` still is not configurable, because it is
        // not a slot name in the first place.
        assert_eq!(
            resolve_role(&off, &["ceiling", "generation"], "claude", "cell"),
            Resolved::Budget
        );
        // The escalation word survives one layer up, where the tier-shaped
        // marker callers live: `[bee-tier: ceiling]` still means the session
        // model.
        assert_eq!(resolve_tier(&off, "ceiling", "claude", "cell"), Resolved::Inherit);
        // Every leaf shape is reachable through a fall-through, not just a
        // first hit: the cli purpose gate still refuses a cell execution.
        let cli = models_from(r#"{"claude":{"generation":{"kind":"cli","command":"glm run"}}}"#);
        assert_eq!(
            resolve_role(&cli, &["docs", "generation"], "claude", "gather"),
            Resolved::Cli { command: "glm run".into() }
        );
        assert_eq!(
            resolve_role(&cli, &["docs", "generation"], "claude", "cell"),
            Resolved::Refused { slot: "generation".into() }
        );
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

    /// D1 (store `cd72ec97`) — mrs-24 collapsed `resolve_advisor` onto the ONE
    /// leaf parser, and this is what keeps it collapsed. For every documented
    /// value shape the advisor slot and an ordinary role slot must resolve to
    /// the SAME `Resolved`: a second advisor-only parser would have to diverge
    /// on one of these rows to be worth having, and divergence is precisely
    /// what D1 removed — the two copies had already been edited in lockstep
    /// twice (herding-tier D1, then herding-review-slots D1/D2) before this.
    ///
    /// What is genuinely the advisor's own is its WALK, not its parse: one
    /// name, no fall-through, no `Budget` floor (decision `4faf1de9`). That is
    /// the single asymmetry the `None` arm below pins.
    #[test]
    fn the_advisor_slot_parses_a_value_exactly_as_any_other_role_does() {
        for shape in [
            r#""opus""#,
            r#"{"model":"opus"}"#,
            r#"{"kind":"native","model":"opus"}"#,
            r#"{"kind":"native","model":"opus","effort":"low","fork_turns":"none","agent_type":"reviewer"}"#,
            r#"{"kind":"cli","command":"consult me"}"#,
            r#"{"kind":"herding"}"#,
            r#"{"kind":"herding","agent":"named-herd","fallback":"default"}"#,
            r#"{"primary":{"kind":"native","model":"opus"},"fallback_policy":"explicit-only","fallback":{"kind":"cli","command":"consult me"}}"#,
            "null",
        ] {
            let as_advisor = resolve_advisor(
                &models_from(&format!(r#"{{"claude":{{"advisor":{shape}}}}}"#)),
                "claude",
            );
            // `design` is any other name at all — the point is that nothing
            // about the WORD "advisor" changes how the leaf is read.
            let as_role = resolve_role(
                &models_from(&format!(r#"{{"claude":{{"design":{shape}}}}}"#)),
                &["design"],
                "claude",
                "advisor",
            );
            match as_advisor {
                Some(r) => assert_eq!(r, as_role, "shape {shape}: advisor and role disagree"),
                None => assert_eq!(
                    as_role,
                    Resolved::Budget,
                    "shape {shape}: the advisor's None is a walk difference, not a parse one"
                ),
            }
        }
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

        // mrs-29: the JOB decides, not the spelling. Both words for one job
        // reach the one agent that serves it, so the lookup cannot answer
        // differently depending on which vocabulary a host speaks.
        assert_eq!(pinned_agent_type("read"), pinned_agent_type("extraction"));
        assert_eq!(pinned_agent_type("code"), pinned_agent_type("generation"));
        // A name that is nobody's alias is untouched — the open role set has
        // no rendered agent for an operator-invented job, and inventing one
        // would name a file that does not exist.
        assert_eq!(agent_for_role("test"), None);
        assert_eq!(agent_for_role("design"), None);
        assert_eq!(canonical_role("test"), "test");
        // The inverse lookup is deliberately NOT aliased: its answer is
        // resolved against `models.<runtime>`, so it must name the role the
        // agent was rendered from — the one every host still carries.
        assert_eq!(role_for_agent("bee-extract"), Some("extraction"));
        assert_eq!(role_for_agent("bee-gather"), Some("generation"));
    }

    /// mrs-29, the audit's finding, pinned where a caller can feel it: ONE
    /// request gets ONE agent, whatever vocabulary the host's config speaks.
    ///
    /// Measured on the release binary BEFORE the fix, `dispatch prepare
    /// --runtime claude --kind gather --role read` answered
    /// `subagent_type: "general-purpose"` on a host seeded by today's
    /// `default_config`, while `--role extraction` answered `bee-extract` —
    /// the same read job, two agents, decided by config drift rather than by
    /// the request. This asks the DOOR, over the payload a caller actually
    /// receives, on all three host shapes the migration produces.
    #[test]
    fn the_same_job_gets_the_same_rendered_agent_on_every_host_vocabulary() {
        // The three configs a host can be in mid-migration.
        const PRE_ROLES: &str =
            r#"{"models":{"claude":{"extraction":"haiku","generation":"sonnet","review":"opus"}}}"#;
        // What `onboard/templates.rs::default_config` seeds into a fresh host.
        const SEEDED: &str = r#"{"models":{"claude":{"code":"sonnet","read":"haiku","extraction":"haiku","generation":"sonnet"}}}"#;
        // An operator who finished the move and dropped the historical keys.
        const MIGRATED: &str =
            r#"{"models":{"claude":{"code":"sonnet","read":"haiku","review":"opus"}}}"#;

        // Every (host, spelling) pair the door can ANSWER for the read job,
        // and the one agent every one of them must name.
        for (config, role) in [
            (PRE_ROLES, "extraction"),
            (SEEDED, "extraction"),
            (SEEDED, "read"),
            (MIGRATED, "read"),
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let root = repo(&tmp, config);
            let Prepared::Value(v) = prepare_dispatch_with_role(
                &root, "claude", "gather", Some(role), None, None, false, None, None, false, None,
            )
            .unwrap() else {
                panic!("--role {role} did not produce a payload")
            };
            assert_eq!(
                v.get("payload").and_then(|p| p.get("subagent_type")),
                Some(&json!("bee-extract")),
                "--role {role} on {config} must reach the rendered read agent"
            );
        }

        // The same for the generation job, whose FIRST entry (the read-only
        // bee-gather) is the answer a role-only lookup owes on either word.
        for (config, role) in
            [(PRE_ROLES, "generation"), (SEEDED, "generation"), (SEEDED, "code"), (MIGRATED, "code")]
        {
            let tmp = tempfile::tempdir().unwrap();
            let root = repo(&tmp, config);
            let Prepared::Value(v) = prepare_dispatch_with_role(
                &root, "claude", "gather", Some(role), None, None, false, None, None, false, None,
            )
            .unwrap() else {
                panic!("--role {role} did not produce a payload")
            };
            assert_eq!(
                v.get("payload").and_then(|p| p.get("subagent_type")),
                Some(&json!("bee-gather")),
                "--role {role} on {config} must reach the read-only generation agent"
            );
        }

        // The pair this does NOT silently repair, recorded so it is a
        // decision rather than a gap: a spelling the host configures NO model
        // for is still refused by name (D2, store 06e49368) — there is no
        // model to select, and answering an agent while selecting none is the
        // silent-migration defect this feature exists to remove. The refusal
        // is loud and carries its FIX, which is the honest answer; what mrs-29
        // fixes is the case where the door DOES answer.
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, PRE_ROLES);
        let Prepared::Value(v) = prepare_dispatch_with_role(
            &root, "claude", "gather", Some("read"), None, None, false, None, None, false, None,
        )
        .unwrap() else {
            panic!("expected a refusal envelope")
        };
        assert_eq!(v.get("ok"), Some(&json!(false)));
        assert_eq!(v.get("reason"), Some(&json!("role_not_configured")));
    }

    // ── prepareDispatch envelopes ──────────────────────────────────────────

    #[test]
    fn gather_envelope_is_the_claude_agent_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        let out = prepare_dispatch(
            &root, "claude", "gather", None, None, false, None, None, false, None)
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
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
                prepare_dispatch(&root, "claude", "cell", Some(id), Some("w"), false, None, None, false, None)
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
            false, None)
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
            &root, "codex", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
                        "codex":{"generation":"gpt-5","advisor":"gpt-5"},
                        "pi":{"generation":{"kind":"herding","agent":"pi-opus"},
                              "advisor":{"kind":"herding","agent":"pi-fable"}}
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
                    false, None)
                .unwrap() else {
                    panic!("{runtime}/{kind}: expected an envelope, not a refusal")
                };
                let payload = match v.get("payload") {
                    Some(p) => p,
                    None => panic!("{runtime}/{kind}: envelope carries no payload at all"),
                };

                // Only a pair whose payload actually carries a label field
                // owes the assertion below — a transport with none (cli-exec
                // and herding-exec, the recorded limit: an external executor
                // call is `{command, stdin}`, with no field to carry a
                // subject) is exempt by construction, never by a hand-picked
                // skip. pi is the exempt runtime here: D5 makes every pi slot
                // herding, so its payload is that Bash call and `continue`
                // fires for it — the claude and codex rows still owe the
                // label, which is what this device was built to hold.
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
        prepare_dispatch(&root, "claude", "gather", None, None, false, None, None, true, None).unwrap();
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
            prepare_dispatch(&root, "claude", "advisor", None, None, false, None, None, false, None).unwrap()
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
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("reason"), Some(&json!("cli_tier_gather_only")));
        assert_eq!(v.get("slot"), Some(&json!("generation")));
        assert_eq!(v.get("fix"), Some(&json!(CLI_REFUSAL_FIX)));

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "gather", None, None, false, None, None, false, None).unwrap()
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
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
            prepare_dispatch(&root, "codex", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
            prepare_dispatch(&main, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("tool"), Some(&json!("Bash")));
        assert_eq!(
            v.get("payload").unwrap().get("fallback"),
            Some(&json!({"model": "sonnet", "fallback_when": "transport_ready is false"}))
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
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
            prepare_dispatch(&root, "codex", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
            prepare_dispatch(&root, "claude", "advisor", None, None, false, None, None, false, None)
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(v.get("payload").unwrap().get("fallback"), None);
    }

    // ── herding-reach D1: transport reachability probe & payload fields ─────

    #[test]
    fn herding_transport_probe_reports_ready_when_both_vars_set() {
        let env_map = HashMap::from([
            ("HERDR_ENV", "1".to_string()),
            ("HERDR_PANE_ID", "w4:p7".to_string()),
        ]);
        let (ready, reason, pane_id) = herding_transport_probe(&|k| env_map.get(k).cloned());
        assert!(ready);
        assert_eq!(reason, "HERDR_ENV=1 and HERDR_PANE_ID=w4:p7 are set");
        assert_eq!(pane_id, Some("w4:p7".to_string()));
    }

    #[test]
    fn herding_transport_probe_reports_not_ready_when_either_var_missing() {
        // Missing HERDR_ENV
        let env_map = HashMap::from([
            ("HERDR_PANE_ID", "w4:p7".to_string()),
        ]);
        let (ready, reason, pane_id) = herding_transport_probe(&|k| env_map.get(k).cloned());
        assert!(!ready);
        assert_eq!(
            reason,
            "HERDR_ENV is not set — this session is not inside a herdr pane"
        );
        assert_eq!(pane_id, Some("w4:p7".to_string()));

        // HERDR_ENV is not "1"
        let env_map = HashMap::from([
            ("HERDR_ENV", "0".to_string()),
            ("HERDR_PANE_ID", "w4:p7".to_string()),
        ]);
        let (ready, reason, pane_id) = herding_transport_probe(&|k| env_map.get(k).cloned());
        assert!(!ready);
        assert_eq!(
            reason,
            "HERDR_ENV is not 1 — this session is not inside a herdr pane"
        );
        assert_eq!(pane_id, Some("w4:p7".to_string()));

        // Missing HERDR_PANE_ID
        let env_map = HashMap::from([
            ("HERDR_ENV", "1".to_string()),
        ]);
        let (ready, reason, pane_id) = herding_transport_probe(&|k| env_map.get(k).cloned());
        assert!(!ready);
        assert_eq!(
            reason,
            "HERDR_PANE_ID is not set — this session is not inside a herdr pane"
        );
        assert_eq!(pane_id, None);

        // Empty HERDR_PANE_ID
        let env_map = HashMap::from([
            ("HERDR_ENV", "1".to_string()),
            ("HERDR_PANE_ID", "".to_string()),
        ]);
        let (ready, reason, pane_id) = herding_transport_probe(&|k| env_map.get(k).cloned());
        assert!(!ready);
        assert_eq!(
            reason,
            "HERDR_PANE_ID is not set — this session is not inside a herdr pane"
        );
        assert_eq!(pane_id, None);

        // Both missing
        let env_map = HashMap::<&str, String>::new();
        let (ready, reason, pane_id) = herding_transport_probe(&|k| env_map.get(k).cloned());
        assert!(!ready);
        assert_eq!(
            reason,
            "HERDR_ENV is not set — this session is not inside a herdr pane"
        );
        assert_eq!(pane_id, None);
    }

    #[test]
    fn herding_dispatch_payload_carries_transport_ready_and_transport_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":{"kind":"herding"}}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
                .unwrap()
        else {
            panic!()
        };
        let payload = v.get("payload").unwrap();
        assert!(payload.get("transport_ready").is_some());
        assert!(payload.get("transport_ready").unwrap().is_boolean());
        assert!(payload.get("transport_reason").is_some());
        assert!(payload.get("transport_reason").unwrap().is_string());
        // Fallback is omitted when not configured
        assert_eq!(payload.get("fallback"), None);
    }

    // ── tmux-herding-transport D1: the probe's tmux arm and the config key ──

    #[test]
    fn transport_probe_tmux_arm_reports_ready_when_both_vars_set() {
        let env_map = HashMap::from([
            ("TMUX", "/tmp/tmux-1000/default,42,0".to_string()),
            ("TMUX_PANE", "%3".to_string()),
        ]);
        let (ready, reason, pane_id) = herding_transport_probe_for(
            crate::herding::TransportKind::Tmux,
            &|k| env_map.get(k).cloned(),
        );
        assert!(ready);
        assert_eq!(reason, "TMUX and TMUX_PANE=%3 are set");
        assert_eq!(pane_id, Some("%3".to_string()));
    }

    #[test]
    fn transport_probe_tmux_arm_reports_not_ready_when_either_var_missing() {
        // TMUX set, TMUX_PANE missing.
        let env_map = HashMap::from([("TMUX", "/tmp/tmux-1000/default,42,0".to_string())]);
        let (ready, reason, pane_id) = herding_transport_probe_for(
            crate::herding::TransportKind::Tmux,
            &|k| env_map.get(k).cloned(),
        );
        assert!(!ready);
        assert_eq!(
            reason,
            "TMUX_PANE is not set — this session is not inside a tmux pane"
        );
        assert_eq!(pane_id, None);

        // TMUX set, TMUX_PANE empty.
        let env_map = HashMap::from([
            ("TMUX", "/tmp/tmux-1000/default,42,0".to_string()),
            ("TMUX_PANE", "".to_string()),
        ]);
        let (ready, reason, pane_id) = herding_transport_probe_for(
            crate::herding::TransportKind::Tmux,
            &|k| env_map.get(k).cloned(),
        );
        assert!(!ready);
        assert_eq!(
            reason,
            "TMUX_PANE is not set — this session is not inside a tmux pane"
        );
        assert_eq!(pane_id, None);

        // TMUX missing: the pane id still rides along.
        let env_map = HashMap::from([("TMUX_PANE", "%3".to_string())]);
        let (ready, reason, pane_id) = herding_transport_probe_for(
            crate::herding::TransportKind::Tmux,
            &|k| env_map.get(k).cloned(),
        );
        assert!(!ready);
        assert_eq!(
            reason,
            "TMUX is not set — this session is not inside a tmux pane"
        );
        assert_eq!(pane_id, Some("%3".to_string()));
    }

    #[test]
    fn transport_probe_herdr_arm_never_reads_the_tmux_vars() {
        // D1: no auto-detect. A session inside BOTH tools with the key absent
        // stays on herdr and reports the herdr reason.
        let env_map = HashMap::from([
            ("TMUX", "/tmp/tmux-1000/default,42,0".to_string()),
            ("TMUX_PANE", "%3".to_string()),
        ]);
        let (ready, reason, pane_id) = herding_transport_probe(&|k| env_map.get(k).cloned());
        assert!(!ready);
        assert_eq!(
            reason,
            "HERDR_ENV is not set — this session is not inside a herdr pane"
        );
        assert_eq!(pane_id, None);
    }

    #[test]
    fn transport_payload_reason_follows_the_configured_transport() {
        // herding.transport=tmux routes the payload probe onto the tmux arm.
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":{"kind":"herding"}}},"herding":{"transport":"tmux"}}"#,
        );
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
                .unwrap()
        else {
            panic!()
        };
        let reason = v
            .get("payload")
            .and_then(|p| p.get("transport_reason"))
            .and_then(Value::as_str)
            .unwrap()
            .to_string();
        // The real env decides ready/not-ready here; the ARM is what this
        // asserts — every tmux-arm reason names TMUX, no herdr reason does.
        assert!(reason.contains("TMUX"), "got {reason}");
        assert!(!reason.contains("HERDR"), "got {reason}");
    }

    #[test]
    fn transport_payload_reports_not_ready_on_an_unknown_transport_value() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":{"kind":"herding"}}},"herding":{"transport":"nope"}}"#,
        );
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
                .unwrap()
        else {
            panic!()
        };
        let payload = v.get("payload").unwrap();
        assert_eq!(payload.get("transport_ready"), Some(&Value::Bool(false)));
        let reason = payload.get("transport_reason").and_then(Value::as_str).unwrap();
        assert!(reason.contains("herding.transport is \"nope\""), "got {reason}");
        assert!(reason.contains("\"herdr\""), "got {reason}");
        assert!(reason.contains("\"tmux\""), "got {reason}");
    }

    // ── herding-reach D5: recorded cell tier resolution ─────────────────────

    #[test]
    fn recorded_generation_carries_tier_source_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","title":"some work","tier":"generation","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "claude",
            "cell",
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap()
        else {
            panic!("expected an envelope")
        };
        assert_eq!(v.get("tool"), Some(&json!("Agent")));
        let payload = v.get("payload").unwrap();
        assert_eq!(payload.get("model"), Some(&json!("sonnet")));
        assert!(payload.get("prompt").unwrap().as_str().unwrap().starts_with("[bee-tier: generation]\n"));
        let econ = v.get("economics").unwrap();
        assert_eq!(econ.get("logical_tier"), Some(&json!("generation")));
        assert_eq!(econ.get("tier_source"), Some(&json!("cell")));
        assert_eq!(econ.get("channel"), Some(&json!("claude-agent")));
    }

    #[test]
    fn recorded_ceiling_emits_session_model_payload_and_channel() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":{"kind":"herding"}}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","title":"critical ceiling fix","tier":"ceiling","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "claude",
            "cell",
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap()
        else {
            panic!("expected an envelope")
        };
        // Claude runtime: Agent tool, no model param, [bee-tier: ceiling] marker, session-model channel
        assert_eq!(v.get("tool"), Some(&json!("Agent")));
        let payload = v.get("payload").unwrap();
        assert_eq!(payload.get("model"), None, "ceiling dispatch must have no model param");
        assert_eq!(payload.get("subagent_type"), Some(&json!("bee-build")));
        assert_eq!(payload.get("description"), Some(&json!("c-1: critical ceiling fix (ceiling)")));
        assert!(payload.get("prompt").unwrap().as_str().unwrap().starts_with("[bee-tier: ceiling]\n"));
        let econ = v.get("economics").unwrap();
        assert_eq!(econ.get("logical_tier"), Some(&json!("ceiling")));
        assert_eq!(econ.get("tier_source"), Some(&json!("cell")));
        assert_eq!(econ.get("channel"), Some(&json!("session-model")));
        assert_eq!(econ.get("enforcement"), Some(&json!("session-model")));
        assert_eq!(econ.get("requested_model"), Some(&Value::Null));
        assert_eq!(econ.get("effective_model"), Some(&Value::Null));

        // Codex runtime: spawn_agent tool, no model, [bee-tier: ceiling] marker, session-model channel
        let Prepared::Value(v_codex) = prepare_dispatch(
            &root,
            "codex",
            "cell",
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap()
        else {
            panic!("expected an envelope")
        };
        assert_eq!(v_codex.get("tool"), Some(&json!("spawn_agent")));
        let codex_payload = v_codex.get("payload").unwrap();
        assert_eq!(codex_payload.get("model"), None);
        assert_eq!(codex_payload.get("task_name"), Some(&json!("c-1: critical ceiling fix")));
        assert!(codex_payload.get("message").unwrap().as_str().unwrap().starts_with("[bee-tier: ceiling]\n"));
        let codex_econ = v_codex.get("economics").unwrap();
        assert_eq!(codex_econ.get("channel"), Some(&json!("session-model")));
        assert_eq!(codex_econ.get("tier_source"), Some(&json!("cell")));
    }

    /// D5 (store `97ce5225`) — the OTHER half of what `ceiling` meant, now
    /// carried by the flag: run on the session model, with no `model`
    /// parameter and NO herding command.
    ///
    /// The fixture is chosen so the two answers cannot be confused. The cell
    /// carries no `tier` at all (a post-D7 cell never will), declares
    /// `role: "code"`, and the only configured slot is a HERDING
    /// `generation` — so without the flag this dispatch is a Bash
    /// `bee herding run` payload on the `herding-exec` channel. With the
    /// flag it is the session model, and the escalation outranks the job
    /// role the cell declares.
    #[test]
    fn an_escalated_cell_runs_on_the_session_model_with_no_herding_command() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":{"kind":"herding"}}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","title":"rescue ladder","role":"code","escalate":true,"status":"claimed","trace":{"worker":"w"}}"#,
        );
        // The same cell WITHOUT the flag: the herding command this fixture
        // exists to make visible.
        w(
            &root,
            ".bee/cells/c-2.json",
            r#"{"id":"c-2","feature":"f","title":"ordinary work","role":"code","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let prep = |id: &str| {
            let Prepared::Value(v) = prepare_dispatch(
                &root, "claude", "cell", Some(id), Some("w"), false, None, None, false, None,
            )
            .unwrap() else {
                panic!("expected an envelope")
            };
            v
        };

        let unescalated = prep("c-2");
        assert_eq!(unescalated.get("tool"), Some(&json!("Bash")), "the control really does herd");
        assert_eq!(
            unescalated.get("economics").unwrap().get("channel"),
            Some(&json!("herding-exec"))
        );

        let v = prep("c-1");
        assert_eq!(v.get("tool"), Some(&json!("Agent")));
        let payload = v.get("payload").unwrap();
        assert_eq!(payload.get("model"), None, "an escalated dispatch has no model param");
        assert_eq!(payload.get("command"), None, "and no herding command");
        assert_eq!(payload.get("subagent_type"), Some(&json!("bee-build")));
        assert_eq!(payload.get("description"), Some(&json!("c-1: rescue ladder (ceiling)")));
        assert!(payload
            .get("prompt")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("[bee-tier: ceiling]\n"));
        let econ = v.get("economics").unwrap();
        assert_eq!(econ.get("channel"), Some(&json!("session-model")));
        assert_eq!(econ.get("enforcement"), Some(&json!("session-model")));
        assert_eq!(econ.get("requested_model"), Some(&Value::Null));
        assert_eq!(econ.get("effective_model"), Some(&Value::Null));
        // The marker and the audit line name ONE word. A cell declaring
        // `role: "code"` that runs on the session model must not audit as
        // "code" while its prompt says "ceiling".
        assert_eq!(econ.get("logical_tier"), Some(&json!("ceiling")));
        assert_eq!(econ.get("tier_source"), Some(&json!("cell")));

        // Codex takes the same answer through its own transport.
        let Prepared::Value(v_codex) = prepare_dispatch(
            &root, "codex", "cell", Some("c-1"), Some("w"), false, None, None, false, None,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        assert_eq!(v_codex.get("tool"), Some(&json!("spawn_agent")));
        let codex_payload = v_codex.get("payload").unwrap();
        assert_eq!(codex_payload.get("model"), None);
        assert_eq!(codex_payload.get("command"), None);
        assert!(codex_payload
            .get("message")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("[bee-tier: ceiling]\n"));
        assert_eq!(
            v_codex.get("economics").unwrap().get("channel"),
            Some(&json!("session-model"))
        );
    }

    /// An explicit `--role` still names the slot OUTRIGHT — the precedence
    /// D5 did not change. A `--role` naming a real slot beats the cell's own
    /// flag, and `--role ceiling` stays the escalation door for a dispatch
    /// with no cell behind it.
    #[test]
    fn explicit_role_still_outranks_the_cells_escalation_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet","code":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","title":"rescue ladder","role":"code","escalate":true,"status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) = prepare_dispatch_with_role(
            &root,
            "claude",
            "cell",
            Some("code"),
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        assert_eq!(v.get("payload").unwrap().get("model"), Some(&json!("sonnet")));
        let econ = v.get("economics").unwrap();
        assert_eq!(econ.get("channel"), Some(&json!("claude-agent")));
        assert_eq!(econ.get("tier_source"), Some(&json!("flag")));

        // And with no cell at all, `--role ceiling` is still the escalation
        // door for a gather or a reviewer.
        let Prepared::Value(g) = prepare_dispatch_with_role(
            &root,
            "claude",
            "gather",
            Some("ceiling"),
            None,
            None,
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        assert_eq!(
            g.get("economics").unwrap().get("channel"),
            Some(&json!("session-model"))
        );
    }

    /// REVIEW P1-A — the invariant that binds D5's two halves together:
    /// **the set of cells the ration CHARGES is the set of cells that get the
    /// session model.** `prepare.rs` states it in prose beside the flag read
    /// ("the one predicate the 40% ration reads too, so the cell that charges
    /// the budget and the cell that spends it are the same set"); nothing
    /// asserted it, and the two sides had drifted.
    ///
    /// This is the test the review named as missing: it authors a cell, then
    /// asks BOTH layers about the same record. Neither half proves anything
    /// alone — `cell_is_escalated` answering false is only a defect once the
    /// dispatch hands that same record the session model anyway, which is
    /// exactly the shape `role: "ceiling"` had.
    ///
    /// It fails on the pre-fix code at the `c-role-ceiling` row: `tier_token`
    /// on the `from_role` path IS the cell's declared role string, so
    /// `tier_token == ESCALATION_WORD` fired for a record no counter ever
    /// saw — measured at 86% escalated with the 40% refusal silent.
    #[test]
    fn the_ration_and_the_dispatch_agree_on_which_cells_are_escalated() {
        let tmp = tempfile::tempdir().unwrap();
        // ONLY `generation` is configured, so the two answers cannot be
        // confused: a cell that does not escalate resolves a real model on
        // the `claude-agent` channel, and an escalated one has no `model`
        // parameter at all.
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);

        // Every marking a cell can carry, against what D5 says it MEANS.
        // The third row is review P1-A: `verbs::cells` validation accepts the
        // name (`role_validation_checks_shape_and_never_membership` — D2's
        // role set is open and membership-blind), and the dispatch used to
        // read that accepted name as the escalation word.
        let rows: [(&str, &str, bool); 4] = [
            ("c-flag", r#""role":"code","escalate":true"#, true),
            ("c-legacy-tier", r#""tier":"ceiling""#, true),
            ("c-role-ceiling", r#""role":"ceiling""#, false),
            ("c-plain", r#""role":"code""#, false),
        ];

        for (id, marking, escalated) in rows {
            w(
                &root,
                &format!(".bee/cells/{id}.json"),
                &format!(
                    r#"{{"id":"{id}","feature":"f","title":"t",{marking},"status":"claimed","trace":{{"worker":"w"}}}}"#
                ),
            );

            // HALF ONE — what the RATION sees. `cell_is_escalated` is the one
            // predicate `escalation_share_after`, `role_mix` and
            // `ceiling_scarcity_warning` all read, so this is every counter.
            let cell = read_cell(&root, id).unwrap().unwrap();
            assert_eq!(
                crate::verbs::cells::cell_is_escalated(&cell),
                escalated,
                "{id}: the predicate every escalation counter reads"
            );

            // HALF TWO — what the DISPATCH actually hands out for that very
            // same record.
            let Prepared::Value(v) = prepare_dispatch(
                &root, "claude", "cell", Some(id), Some("w"), false, None, None, false, None,
            )
            .unwrap() else {
                panic!("{id}: expected an envelope")
            };
            let econ = v.get("economics").unwrap();
            let payload = v.get("payload").unwrap();
            let prompt = payload.get("prompt").unwrap().as_str().unwrap().to_string();
            let took_session_model = econ.get("channel") == Some(&json!("session-model"));

            // THE BINDING ASSERTION. Charged and spent are ONE set.
            assert_eq!(
                took_session_model, escalated,
                "{id}: the ration says escalated={escalated}, the dispatch says \
                 {took_session_model} — a cell that runs on the session model without \
                 being counted bypasses the 40% ration entirely"
            );

            if escalated {
                assert_eq!(payload.get("model"), None, "{id}: escalated means no model param");
                assert!(
                    prompt.starts_with("[bee-tier: ceiling]\n"),
                    "{id}: an escalated dispatch travels under the escalation word: {prompt}"
                );
            } else {
                assert_eq!(
                    payload.get("model"),
                    Some(&json!("sonnet")),
                    "{id}: an uncounted cell must resolve a real model, never the session one"
                );
                assert!(
                    !prompt.starts_with("[bee-tier: ceiling]\n"),
                    "{id}: an uncounted cell must not travel under the escalation word: {prompt}"
                );
                assert_eq!(econ.get("channel"), Some(&json!("claude-agent")), "{id}");
            }
        }
    }

    /// The other direction of P1-A, stated on its own because it is what
    /// `models.rs` PROMISES in prose: a cell declaring `role: "ceiling"` is
    /// "just a role nothing configures — it warns and falls through like any
    /// other". Escalation left the role axis with D5; it lives on the flag.
    ///
    /// The escalation doors that MUST stay open are asserted in the same
    /// test, so a later narrowing cannot close one of them while closing this
    /// hole: an explicit `--role ceiling` names the slot outright and remains
    /// the escalation door, with a cell behind it or without one.
    #[test]
    fn a_cell_role_of_ceiling_falls_through_the_resolver_but_the_flag_door_stays_open() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","title":"t","role":"ceiling","status":"claimed","trace":{"worker":"w"}}"#,
        );

        let Prepared::Value(v) = prepare_dispatch(
            &root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        let econ = v.get("economics").unwrap();
        // It walked `[ceiling, code, generation]` and the TAIL won, exactly as
        // an operator-invented name would. The marker names the RESOLVED role,
        // never the unresolved head — `hooks/model_guard.rs` reads it back and
        // denies a name nothing configures.
        assert_eq!(v.get("payload").unwrap().get("model"), Some(&json!("sonnet")));
        assert_eq!(econ.get("logical_tier"), Some(&json!("generation")));
        assert_eq!(econ.get("channel"), Some(&json!("claude-agent")));
        // `tier_source` still records WHO chose it: the cell did.
        assert_eq!(econ.get("tier_source"), Some(&json!("cell")));

        // NO OTHER ESCALATION PATH CLOSED. `--role ceiling` with no cell
        // behind it is still the session model.
        let Prepared::Value(g) = prepare_dispatch_with_role(
            &root, "claude", "gather", Some("ceiling"), None, None, false, None, None, false, None,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        assert_eq!(
            g.get("economics").unwrap().get("channel"),
            Some(&json!("session-model")),
            "an explicit --role ceiling is still the escalation door"
        );

        // And `--role ceiling` aimed AT this cell escalates it, because an
        // explicit role names the slot outright — the caller saying so is a
        // different fact from the record saying so.
        let Prepared::Value(f) = prepare_dispatch_with_role(
            &root,
            "claude",
            "cell",
            Some("ceiling"),
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        assert_eq!(
            f.get("economics").unwrap().get("channel"),
            Some(&json!("session-model"))
        );
        assert_eq!(f.get("payload").unwrap().get("model"), None);
    }

    #[test]
    fn cell_with_no_tier_field_has_tier_source_default_and_unchanged_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","title":"some work","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "claude",
            "cell",
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap()
        else {
            panic!("expected an envelope")
        };
        assert_eq!(v.get("tool"), Some(&json!("Agent")));
        let payload = v.get("payload").unwrap();
        assert_eq!(payload.get("model"), Some(&json!("sonnet")));
        assert!(payload.get("prompt").unwrap().as_str().unwrap().starts_with("[bee-tier: generation]\n"));
        let econ = v.get("economics").unwrap();
        assert_eq!(econ.get("logical_tier"), Some(&json!("generation")));
        assert_eq!(econ.get("tier_source"), Some(&json!("default")));
        assert_eq!(econ.get("channel"), Some(&json!("claude-agent")));
    }

    #[test]
    fn unconfigured_recorded_tier_is_a_typed_refusal() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet","extraction":null}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","title":"extract things","tier":"extraction","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "claude",
            "cell",
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap()
        else {
            panic!("expected a value")
        };
        assert_eq!(v.get("ok"), Some(&json!(false)));
        assert_eq!(v.get("type"), Some(&json!("refused")));
        assert_eq!(v.get("reason"), Some(&json!("tier_not_configured")));
        assert_eq!(v.get("tier"), Some(&json!("extraction")));
        assert!(v.get("fix").unwrap().as_str().unwrap().contains("set models.claude.extraction"));

        // Arbitrary unknown tier
        w(
            &root,
            ".bee/cells/c-2.json",
            r#"{"id":"c-2","feature":"f","title":"unknown tier work","tier":"quantum","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v2) = prepare_dispatch(
            &root,
            "claude",
            "cell",
            Some("c-2"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap()
        else {
            panic!("expected a value")
        };
        assert_eq!(v2.get("ok"), Some(&json!(false)));
        assert_eq!(v2.get("type"), Some(&json!("refused")));
        assert_eq!(v2.get("reason"), Some(&json!("tier_not_configured")));
        assert_eq!(v2.get("tier"), Some(&json!("quantum")));
        assert!(v2.get("fix").unwrap().as_str().unwrap().contains("set models.claude.quantum"));
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
            prepare_dispatch(&root, "claude", "reviewer", None, None, false, None, None, false, None)
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
            prepare_dispatch(&root, "claude", "advisor", None, None, false, None, None, false, None)
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
            prepare_dispatch(&root, "claude", "gather", None, None, false, None, None, false, None)
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
                prepare_dispatch(&root, "claude", "cell", None, Some("w"), false, None, None, false, None)
                    .unwrap()
            ),
            "dispatch prepare: --cell is required when --kind cell."
        );
        assert_eq!(
            thrown(
                prepare_dispatch(
                    &root, "claude", "cell", Some("ghost"), Some("w"), false, None, None, false, None)
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
                    &root, "claude", "cell", Some("c-1"), Some("   "), false, None, None, false, None)
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
            &root, "claude", "cell", Some("c-1"), Some("thief"), true, None, None, true, None)
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
            &root, "claude", "cell", Some("c-1"), Some("owner"), true, None, None, false, None)
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
            false, None)
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
            false, None)
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
            prepare_dispatch(&main, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
            prepare_dispatch(&main, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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
            prepare_dispatch(&root, "claude", "cell", Some("c-1"), Some("w"), false, None, None, false, None)
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

    /// `prepare_dispatch` ALONE — the inner build, not the real CLI door —
    /// never registers a worker; dp-r2's claim-less registration branch
    /// lives in `run_dispatch_prepare` (pinned below by
    /// `claim_less_prepare_of_owned_cell_registers_the_worker` and
    /// neighbors), never inside `prepare_dispatch` itself, so calling this
    /// dry-run build directly — the shape a claim-less call's PROBE pass
    /// reads — leaves `workers` untouched even over a cell `--worker`
    /// already owns.
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
            prepare_dispatch(&root, "claude", "cell", Some("c-4"), Some("w"), false, None, None, false, None)
                .unwrap()
        else {
            panic!("expected an envelope")
        };
        assert!(dpr1_workers(&root).is_empty(), "a claim-less prepare registers nothing");
        assert!(!crate::verbs::cells::registered_worker_for_cell(&root, "c-4", Some("w")).unwrap());
    }

    /// A registration failure never unwinds a claim that already stands: the
    /// cell stays claimed ON DISK, the reservations (none, here) stand, and
    /// the outcome names the failure rather than silently dropping it. That
    /// guarantee is the point of this test; the *trigger* is incidental, and
    /// has been retargeted once already.
    ///
    /// **Why a whitespace-only role, and not `"bogus"`.** This test used to
    /// drive the failure with `tier: "bogus"`, back when the worker registry
    /// gated that value against a closed `extraction | generation | ceiling`
    /// enum. D4 (store `97ce5225`) retired the enum: the shared
    /// `worker_role_value` shape check in `verbs/state_group/workers.rs` now
    /// asks only that the name be non-blank, so `bogus` is a perfectly legal
    /// role and the old premise is gone.
    ///
    /// What is still reachable is the blank-shape refusal, and reaching it
    /// takes one specific value, because TWO filters sit on this path and
    /// they are spelled differently:
    ///
    /// * `claim_and_reserve_for_dispatch` itself folds the cell's value away
    ///   with `!t.is_empty()` — UNtrimmed. An `""` role is therefore never
    ///   passed on at all: it becomes `None`, `push_worker_record` writes a
    ///   null, and registration SUCCEEDS. An empty string would make this
    ///   test green through the wrong door while exercising no failure.
    /// * `worker_role_value` refuses on `role.trim().is_empty()` — trimmed.
    ///
    /// A whitespace-only name is the value that passes the first filter and
    /// is refused by the second, so it is the deterministic registration
    /// failure that survives the open role set. It is set on BOTH `role` and
    /// `tier` so the pin survives the key this door reads migrating from one
    /// to the other, rather than quietly ceasing to exercise a failure.
    #[test]
    fn a_registration_failure_never_unwinds_the_standing_claim() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, "{}");
        dpr1_lane_with_route(&root, "f");
        w(
            &root,
            ".bee/cells/c-5.json",
            r#"{"id":"c-5","title":"t","status":"open","lane":"tiny","feature":"f","deps":[],"role":"   ","tier":"   "}"#,
        );

        let (cell, reserved, registered, err) =
            claim_and_reserve_for_dispatch(&root, None, "c-5", "bee-w5", None).unwrap().unwrap();

        // The guarantee, in full: the claim this call took SURVIVES the failed
        // registration — proven off the store the NEXT reader would see, not
        // only off the value handed back.
        assert_eq!(cell["status"], json!("claimed"), "the claim stands despite the registration failure");
        let on_disk = read_cell(&root, "c-5").unwrap().expect("the claimed cell is still on disk");
        assert_eq!(on_disk["status"], json!("claimed"), "the claim was unwound on disk: {on_disk}");
        assert_eq!(on_disk["trace"]["worker"], json!("bee-w5"), "the claim lost its owner: {on_disk}");
        assert!(reserved.is_empty());

        // And the failure travels back by name rather than being dropped.
        assert!(!registered, "a blank role must fail registration, not silently pass");
        let message = err.expect("a failed registration must name why");
        assert!(message.starts_with("worker add: invalid role"), "{message}");
        assert!(message.contains("FIX:"), "a refusal names its remedy: {message}");
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
    /// earlier (an invalid `--cell`/`--worker` never reaches registration; a
    /// blank-shaped cell role is the one deterministic failure left once D4
    /// retired the closed tier enum, and it is
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

    // ── dp-r2: dispatch prepare --kind cell, claim-less, registers an OWNED cell ─

    const CLAIM_LESS_OWNED_CHILD: &str = "verbs::drivers::tests::claim_less_prepare_of_owned_cell_child";

    /// Runs ONLY as a child of the test below — same process-global seam as
    /// `dispatch_prepare_claim_payload_child`, above.
    #[test]
    #[ignore = "spawned by claim_less_prepare_of_owned_cell_registers_the_worker"]
    fn claim_less_prepare_of_owned_cell_child() {
        let (flags, use_json) = parse_flags(&[
            "--runtime", "claude", "--kind", "cell", "--cell", "c-6", "--worker", "bee-w6", "--json",
        ])
        .expect("well-formed fixture argv");
        run_dispatch_prepare(flags, use_json, Instant::now());
    }

    /// dp-r2: a claim-less `dispatch prepare --kind cell` over a cell
    /// `--worker` already owns (status `claimed`, `trace.worker` matches)
    /// registers that worker too — the SAME write dp-r1 makes after a fresh
    /// `--claim` — named on the payload and findable by the SAME B44
    /// close-time door (`registered_worker_for_cell`) the claim path already
    /// pins.
    #[test]
    fn claim_less_prepare_of_owned_cell_registers_the_worker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-6.json",
            r#"{"id":"c-6","feature":"f","status":"claimed","trace":{"worker":"bee-w6"}}"#,
        );

        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = Command::new(&exe);
        cmd.args(["--exact", CLAIM_LESS_OWNED_CHILD, "--ignored", "--test-threads", "1", "--nocapture"]);
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
        assert!(payload.get("claimed").is_none(), "no --claim was passed: payload: {payload}");
        assert_eq!(payload.get("worker_registered"), Some(&json!(true)), "payload: {payload}");
        assert!(payload.get("registration_error").is_none(), "payload: {payload}");
        assert!(
            crate::verbs::cells::registered_worker_for_cell(&root, "c-6", Some("bee-w6")).unwrap(),
            "the claim-less registration must be findable by the same close-time door"
        );
    }

    const CLAIM_LESS_OTHER_OWNER_CHILD: &str =
        "verbs::drivers::tests::claim_less_prepare_of_cell_owned_by_another_child";

    /// Runs ONLY as a child of the test below.
    #[test]
    #[ignore = "spawned by claim_less_prepare_of_cell_owned_by_another_refuses_and_registers_nothing"]
    fn claim_less_prepare_of_cell_owned_by_another_child() {
        let (flags, use_json) = parse_flags(&[
            "--runtime", "claude", "--kind", "cell", "--cell", "c-7", "--worker", "bee-w7", "--json",
        ])
        .expect("well-formed fixture argv");
        run_dispatch_prepare(flags, use_json, Instant::now());
    }

    /// dp-r2: a claim-less prepare of a cell claimed by a DIFFERENT worker
    /// still refuses on `claim_ownership` (`check_cell_claim_ownership`'s
    /// `not_owner` code, unchanged) and registers nothing — the new branch
    /// never runs when ownership itself refuses.
    #[test]
    fn claim_less_prepare_of_cell_owned_by_another_refuses_and_registers_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-7.json",
            r#"{"id":"c-7","feature":"f","status":"claimed","trace":{"worker":"someone-else"}}"#,
        );

        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = Command::new(&exe);
        cmd.args(["--exact", CLAIM_LESS_OTHER_OWNER_CHILD, "--ignored", "--test-threads", "1", "--nocapture"]);
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
        assert_eq!(payload.get("ok"), Some(&json!(false)), "payload: {payload}");
        assert_eq!(payload.get("code"), Some(&json!("not_owner")), "payload: {payload}");
        assert!(payload.get("worker_registered").is_none(), "payload: {payload}");
        assert!(dpr1_workers(&root).is_empty(), "an ownership refusal registers nothing");
        assert!(!crate::verbs::cells::registered_worker_for_cell(&root, "c-7", Some("bee-w7")).unwrap());
    }

    const CLAIM_LESS_UNCLAIMED_CHILD: &str = "verbs::drivers::tests::claim_less_prepare_of_unclaimed_cell_child";

    /// Runs ONLY as a child of the test below.
    #[test]
    #[ignore = "spawned by claim_less_prepare_of_unclaimed_cell_keeps_existing_behaviour"]
    fn claim_less_prepare_of_unclaimed_cell_child() {
        let (flags, use_json) = parse_flags(&[
            "--runtime", "claude", "--kind", "cell", "--cell", "c-8", "--worker", "bee-w8", "--json",
        ])
        .expect("well-formed fixture argv");
        run_dispatch_prepare(flags, use_json, Instant::now());
    }

    /// dp-r2: an UNCLAIMED cell (no prior `bee cells claim`) with no
    /// `--claim` on this call either keeps the pre-dp-r2 behaviour exactly —
    /// `claim_ownership`'s `not_claimed` code — and registers nothing.
    #[test]
    fn claim_less_prepare_of_unclaimed_cell_keeps_existing_behaviour() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(&root, ".bee/cells/c-8.json", r#"{"id":"c-8","feature":"f","status":"open"}"#);

        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = Command::new(&exe);
        cmd.args(["--exact", CLAIM_LESS_UNCLAIMED_CHILD, "--ignored", "--test-threads", "1", "--nocapture"]);
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
        assert_eq!(payload.get("ok"), Some(&json!(false)), "payload: {payload}");
        assert_eq!(payload.get("code"), Some(&json!("not_claimed")), "payload: {payload}");
        assert!(payload.get("worker_registered").is_none(), "payload: {payload}");
        assert!(dpr1_workers(&root).is_empty(), "an unclaimed cell registers nothing");
        assert!(!crate::verbs::cells::registered_worker_for_cell(&root, "c-8", Some("bee-w8")).unwrap());
    }

    // ══ slp-contract S4 — the contract-citation tripwire at the DISPATCH door ══
    //
    // Coverage audit before authoring (`.bee/expertise/tests.md`):
    //   - verbs/cells/tests.rs now pins the CLAIM door over the same rule
    //     (four refuse cases, six allow cases, zero mutation on each
    //     refusal).
    //   - this file already pins prepare's typed refusal grammar for
    //     `claim_ownership` ("claim_less_prepare_of_cell_owned_by_another…")
    //     and for an unmapped kind.
    //   - Gap: the dispatch door over a citation, and — the reason there
    //     are two doors at all — the claim-then-change window the claim
    //     door structurally cannot see.

    fn cit_repo(tmp: &tempfile::TempDir) -> PathBuf {
        let root = repo(tmp, r#"{"models":{"claude":{"generation":"sonnet","code":"sonnet"}}}"#);
        w(
            &root,
            ".bee/lanes/cit.json",
            r#"{"feature":"cit","approved_gates":{"execution":true},"route":true}"#,
        );
        w(
            &root,
            ".bee/test-results.json",
            r#"{"ran_at":"2026-01-01T00:00:00.000Z","green":true,"commands":[{"command":"cargo test","exit":0,"duration_ms":1,"failure_excerpt":null}]}"#,
        );
        root
    }

    const CIT_SETTLED: &str = "11111111-0000-0000-0000-000000000001";
    const CIT_RETIRED: &str = "22222222-0000-0000-0000-000000000002";
    const CIT_WAITING: &str = "33333333-0000-0000-0000-000000000003";
    const CIT_REPLACEMENT: &str = "77777777-0000-0000-0000-000000000007";

    fn cit_decide(id: &str) -> String {
        format!(
            r#"{{"id":"{id}","type":"decide","date":"2026-01-01T00:00:00.000Z","decision":"d","rationale":"r","tags":["contract:cell-store"]}}"#
        )
    }

    fn cit_supersede(id: &str, target: &str) -> String {
        format!(
            r#"{{"id":"{id}","type":"supersede","date":"2026-01-02T00:00:00.000Z","decision":"d2","rationale":"r","supersedes":"{target}","tags":["contract:cell-store"]}}"#
        )
    }

    fn cit_write_decisions(root: &Path, lines: &[String]) {
        w(root, ".bee/decisions.jsonl", &format!("{}\n", lines.join("\n")));
    }

    /// A cell already CLAIMED by `w` — the state every `dispatch prepare
    /// --kind cell` sees.
    fn cit_claimed_cell(root: &Path, id: &str, decisions: &str) {
        w(
            root,
            &format!(".bee/cells/{id}.json"),
            &format!(
                r#"{{"id":"{id}","feature":"cit","title":"t","status":"claimed","decisions":{decisions},"trace":{{"worker":"w"}}}}"#
            ),
        );
    }

    /// The window that justifies a SECOND door: the cell was claimed while
    /// its citation was settled, and the decision was superseded
    /// afterwards. The claim door cannot see that — it already ran — so a
    /// claim-only tripwire would dispatch this worker against a retired
    /// contract. D3's letter names the dispatch, and this is why.
    #[test]
    fn dispatching_a_cell_whose_citation_went_retired_after_the_claim_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = cit_repo(&tmp);
        cit_claimed_cell(&root, "c-cit", &format!(r#"["{CIT_RETIRED}"]"#));
        // The store as it was at claim time: the citation was ACTIVE, so
        // the claim door had nothing to refuse on.
        cit_write_decisions(&root, &[cit_decide(CIT_RETIRED)]);
        assert!(
            crate::verbs::cells::contract_citation_refusal(
                &root,
                "c-cit",
                Some(&read_cell(&root, "c-cit").unwrap().unwrap())
            )
            .is_none(),
            "the fixture must start settled, or this test proves nothing about the window"
        );

        // Now it is superseded — the change the claim door never saw.
        cit_write_decisions(&root, &[cit_decide(CIT_RETIRED), cit_supersede(CIT_REPLACEMENT, CIT_RETIRED)]);
        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-cit"), Some("w"), false, None, None, false, None)
                .unwrap()
        else {
            panic!("expected a refusal value")
        };
        assert_eq!(v.get("ok"), Some(&json!(false)), "payload: {v}");
        assert_eq!(v.get("type"), Some(&json!("refused")), "payload: {v}");
        assert_eq!(v.get("reason"), Some(&json!("contract_retired")), "payload: {v}");
        assert_eq!(v.get("code"), Some(&json!("CONTRACT_RETIRED")), "payload: {v}");
        assert_eq!(v.get("cell"), Some(&json!("c-cit")), "payload: {v}");
        let fix = v.get("fix").and_then(Value::as_str).unwrap_or_default();
        assert!(fix.contains(CIT_RETIRED), "the fix names the decision: {v}");
        assert!(fix.contains("bee decisions active"), "payload: {v}");
        // No envelope leaked out beside the refusal, and nothing was written.
        assert!(v.get("payload").is_none(), "a refusal carries no dispatch payload: {v}");
        assert_eq!(read_cell(&root, "c-cit").unwrap().unwrap()["status"], json!("claimed"));
        assert!(!root.join(".bee/logs/dispatch.jsonl").exists(), "a refusal records no dispatch");
    }

    /// The unsettled arm at the same door: an ACTIVE decision whose trigger
    /// is still waiting.
    #[test]
    fn dispatching_a_cell_citing_an_unsettled_contract_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = cit_repo(&tmp);
        cit_claimed_cell(&root, "c-cit", &format!(r#"["{CIT_WAITING}"]"#));
        cit_write_decisions(&root, &[cit_decide(CIT_WAITING)]);
        w(
            &root,
            ".bee/triggers/upstream__33333333.json",
            r#"{"id":"upstream__33333333","decision":"33333333","condition":"revisit when upstream lands","tier":"manual","predicate":null,"status":"waiting","created_at":"2026-08-16T00:00:00.000Z","updated_at":"2026-08-16T00:00:00.000Z","outcome":null}"#,
        );

        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-cit"), Some("w"), false, None, None, false, None)
                .unwrap()
        else {
            panic!("expected a refusal value")
        };
        assert_eq!(v.get("reason"), Some(&json!("contract_unsettled")), "payload: {v}");
        assert_eq!(v.get("code"), Some(&json!("CONTRACT_UNSETTLED")), "payload: {v}");
        assert!(
            v.get("fix").and_then(Value::as_str).unwrap_or_default().contains("bee triggers list"),
            "payload: {v}"
        );
    }

    /// The two doors are ONE rule, not two that happen to agree today: the
    /// same cell refused at the claim door and at the dispatch door carries
    /// the IDENTICAL sentence, because there is one producer of it
    /// (`cells::contract_citation_refusal`). A fixture that cannot diverge
    /// is the only proof worth having here.
    #[test]
    fn the_claim_door_and_the_dispatch_door_refuse_with_the_same_sentence() {
        let tmp = tempfile::tempdir().unwrap();
        let root = cit_repo(&tmp);
        cit_write_decisions(&root, &[cit_decide(CIT_RETIRED), cit_supersede(CIT_REPLACEMENT, CIT_RETIRED)]);
        // OPEN — the claim door's shape.
        w(
            &root,
            ".bee/cells/c-both.json",
            &format!(
                r#"{{"id":"c-both","feature":"cit","title":"t","status":"open","decisions":["{CIT_RETIRED}"]}}"#
            ),
        );
        let claim_refusal = match crate::verbs::cells::claim_cell_from_flags(
            &root, "c-both", "w", Some("sess-1"), None,
        ) {
            Err(crate::verbs::cells::Fail::Thrown(m)) => m,
            Err(crate::verbs::cells::Fail::Delegate) => panic!("unexpected delegate"),
            Ok(_) => panic!("the claim door must refuse a retired citation"),
        };
        // The claim door refused, so the cell is still open — CLAIMED by
        // hand for the dispatch door, which is the state it always sees.
        assert_eq!(read_cell(&root, "c-both").unwrap().unwrap()["status"], json!("open"));
        cit_claimed_cell(&root, "c-both", &format!(r#"["{CIT_RETIRED}"]"#));
        let Prepared::Value(v) =
            prepare_dispatch(&root, "claude", "cell", Some("c-both"), Some("w"), false, None, None, false, None)
                .unwrap()
        else {
            panic!("expected a refusal value")
        };
        let dispatch_sentence = v.get("fix").and_then(Value::as_str).unwrap_or_default();
        assert_eq!(
            claim_refusal,
            format!("claim: CONTRACT_RETIRED — {dispatch_sentence}"),
            "the two doors must speak one sentence, not two that agree by luck"
        );
    }

    /// ALLOW — a settled citation dispatches normally, and a list of local
    /// D-IDs dispatches normally. The dispatch door repeats the claim
    /// door's pass-over rule exactly: an entry that resolves to no store
    /// decision refuses nothing.
    #[test]
    fn a_settled_or_local_citation_dispatches_normally() {
        let tmp = tempfile::tempdir().unwrap();
        let root = cit_repo(&tmp);
        // A store holding a retired decision, so the allow is proven to be
        // about resolution rather than about an empty store.
        cit_write_decisions(
            &root,
            &[cit_decide(CIT_SETTLED), cit_decide(CIT_RETIRED), cit_supersede(CIT_REPLACEMENT, CIT_RETIRED)],
        );

        for decisions in [format!(r#"["{CIT_SETTLED}"]"#), r#"["D1","D2"]"#.to_string(), "[]".to_string()] {
            cit_claimed_cell(&root, "c-ok", &decisions);
            let Prepared::Value(v) = prepare_dispatch(
                &root, "claude", "cell", Some("c-ok"), Some("w"), false, None, None, false, None,
            )
            .unwrap() else {
                panic!("expected an envelope for {decisions}")
            };
            assert_eq!(v.get("tool"), Some(&json!("Agent")), "{decisions} must dispatch: {v}");
            assert!(v.get("type").is_none(), "{decisions} must not be refused: {v}");
        }
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — pin "off" since this test is about
        // the other doors.
        let root = repo(&tmp, r#"{"commands":{"test":["a","b"]},"uat_stop":"off"}"#);
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
                "door dissent-debt: clear\n",
                // an-3: the advisor-nudge door joined the builder beside the
                // dissent one; this feature carries no nudge, so it is clear.
                "door advisor-nudge-debt: clear\n",
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close —
        // pin "off" since this test is about the promote/capture doors.
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"},"uat_stop":"off"}"#);
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
        // defaults-and-agent-env D1: pin uat_stop off — this test is about
        // the promote door's product_root path, not the uat door.
        let root = repo(
            &tmp,
            r#"{"commands":{"test":"echo suite-green"},"product_root":"elsewhere","uat_stop":"off"}"#,
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — pin "off" since this test is about
        // a different door.
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"},"uat_stop":"off"}"#);
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — pin "off" since this test is about
        // a different door.
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"},"uat_stop":"off"}"#);
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — pin "off" since this test is about
        // a different door.
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"},"uat_stop":"off"}"#);
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
        // defaults-and-agent-env D1: pin uat_stop off — this test is about
        // commands.test never spawning, not the uat door.
        std::fs::write(
            main.join(".bee").join("config.json"),
            r#"{"commands":{"test":"echo should-not-run"},"uat_stop":"off"}"#,
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
        // defaults-and-agent-env D1: pin uat_stop off — this test is about
        // the tests door, not the uat door.
        let root = repo(&tmp, r#"{"uat_stop":"off"}"#);
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
        // defaults-and-agent-env D1: pin uat_stop off — this test is about
        // the capture-reminder doors, not the uat door.
        let root = repo(&tmp, r#"{"uat_stop":"off"}"#);
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
                // sd-4: the dissent-debt door joined the builder; this
                // feature carries no dissent, so it reports clear.
                "door dissent-debt: clear",
                // an-3: and the advisor-nudge door beside it, same posture.
                "door advisor-nudge-debt: clear",
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — this helper's callers are about
        // the capture/scribing-debt doors, not the uat door, so pin "off".
        repo(tmp, r#"{"commands":{"test":"echo suite-green"},"uat_stop":"off"}"#)
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — pin "off" since this test is about
        // a different door.
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"},"uat_stop":"off"}"#);
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — pin "off" since this test is about
        // a different door.
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"},"uat_stop":"off"}"#);
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — pin "off" since this test is about
        // a different door.
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"},"uat_stop":"off"}"#);
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

    /// A pre-seeded baseline (`.bee/doc-deferral-baseline.json`) whose
    /// entries cover none of the lines a test cares about — writing this
    /// first flips `build_doc_deferral_door` from a first-run SEED (which
    /// always passes, D2/D6) into ENFORCE mode, so a still-fresh test can go
    /// on proving the block/clear/deferred behavior the door had before the
    /// baseline existed at all.
    ///
    /// D7: EVERY door test needs this. A door test left in seed mode is
    /// vacuous by construction — the seed arm returns non-blocking no matter
    /// what the scan found, so the fenced-code exemption and the citation
    /// escape can both be deleted outright without the test noticing.
    fn write_empty_doc_deferral_baseline(root: &Path) {
        w(root, ".bee/doc-deferral-baseline.json", "{\"files\":{}}\n");
    }

    /// Deterministic baseline JSON text the same shape `build_doc_deferral_
    /// door` itself writes (sorted files, sorted per-file lines) — used to
    /// pre-seed ENFORCE-mode fixtures whose baseline DOES cover specific
    /// content, and to build the byte-identical expectation independently
    /// of the door's own serialization code.
    fn doc_deferral_baseline_json(entries: &[(&str, &[&str])]) -> String {
        let mut files = Map::new();
        for (rel, lines) in entries {
            let mut sorted: Vec<&str> = lines.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            files.insert((*rel).to_string(), Value::Array(sorted.into_iter().map(|l| Value::String(l.to_string())).collect()));
        }
        let mut root = Map::new();
        root.insert("files".to_string(), Value::Object(files));
        format!("{}\n", serde_json::to_string_pretty(&Value::Object(root)).unwrap())
    }

    fn write_doc_deferral_baseline(root: &Path, entries: &[(&str, &[&str])]) {
        w(root, ".bee/doc-deferral-baseline.json", &doc_deferral_baseline_json(entries));
    }

    fn read_doc_deferral_baseline_bytes(root: &Path) -> Vec<u8> {
        std::fs::read(doc_deferral_baseline_path(root)).unwrap()
    }

    #[test]
    fn doc_deferral_door_blocks_a_deferral_line_with_no_registered_trigger() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_empty_doc_deferral_baseline(root);
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:2"), "{}", door.detail);
        assert!(door.detail.contains("remedy:"), "{}", door.detail);
    }

    #[test]
    fn doc_deferral_door_clears_with_a_registered_trigger_citation() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // D7: ENFORCE mode, covering nothing — without it the seed arm
        // passes this test even with the citation escape deleted.
        write_empty_doc_deferral_baseline(root);
        write_trigger(root, "demo-trigger");
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "line one\nThis work is deferred for now, see `demo-trigger`.\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn doc_deferral_door_exempts_fenced_code() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // D7: ENFORCE mode, covering nothing — without it the seed arm
        // passes this test even with the fenced-code exemption deleted.
        write_empty_doc_deferral_baseline(root);
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "prose\n```\nThis work is deferred for now.\n```\nmore prose\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn doc_deferral_door_exempts_a_reasoned_not_a_deferral_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "prose\n<!-- bee:not-a-deferral: names the queue file, not a promise -->\nThis work is deferred for now.\n<!-- /bee:not-a-deferral -->\nmore prose\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn doc_deferral_door_ignores_a_not_a_deferral_marker_with_an_empty_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_empty_doc_deferral_baseline(root);
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "prose\n<!-- bee:not-a-deferral:  -->\nThis work is deferred for now.\n<!-- /bee:not-a-deferral -->\nmore prose\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:3"), "{}", door.detail);
    }

    #[test]
    fn doc_deferral_door_ignores_a_not_a_deferral_marker_with_a_missing_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_empty_doc_deferral_baseline(root);
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "prose\n<!-- bee:not-a-deferral: -->\nThis work is deferred for now.\n<!-- /bee:not-a-deferral -->\nmore prose\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:3"), "{}", door.detail);
    }

    #[test]
    fn doc_deferral_door_blocks_again_after_the_closing_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_empty_doc_deferral_baseline(root);
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "<!-- bee:not-a-deferral: names the queue file, not a promise -->\nThis work is deferred for now.\n<!-- /bee:not-a-deferral -->\nThis work is deferred for now, too.\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:4"), "{}", door.detail);
        assert!(!door.detail.contains(":2 "), "{}", door.detail);
    }

    #[test]
    fn doc_deferral_door_unclosed_marker_exempts_to_end_of_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "<!-- bee:not-a-deferral: names the queue file, not a promise -->\nThis work is deferred for now.\nThis is deferred later too.\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn doc_deferral_door_fence_inside_a_marked_block_behaves_independently() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_empty_doc_deferral_baseline(root);
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "<!-- bee:not-a-deferral: names the queue file, not a promise -->\n```\nThis is deferred for now inside a fence.\n```\nThis is deferred for now inside the marker.\n<!-- /bee:not-a-deferral -->\nThis is deferred for now outside both.\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:7"), "{}", door.detail);
        assert!(!door.detail.contains(":3 "), "{}", door.detail);
        assert!(!door.detail.contains(":5 "), "{}", door.detail);
    }

    #[test]
    fn doc_deferral_door_marker_inside_a_fence_behaves_independently() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_empty_doc_deferral_baseline(root);
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "```\n<!-- bee:not-a-deferral: names the queue file, not a promise -->\nThis is deferred for now inside the fence and the marker.\n<!-- /bee:not-a-deferral -->\n```\nThis is deferred for now outside both.\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:6"), "{}", door.detail);
        assert!(!door.detail.contains(":3 "), "{}", door.detail);
    }

    /// The exact register.md shapes from the real incident: a heading whose
    /// backtick span is a filename (tried and failing as a trigger id), a
    /// prose line carrying "deferred", and one carrying "later" — all clear
    /// once wrapped in a reasoned not-a-deferral block.
    #[test]
    fn doc_deferral_door_register_md_incident_shapes_clear_once_wrapped() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(
            root,
            "docs/handbook/register.md",
            concat!(
                "<!-- bee:not-a-deferral: this documents the deferred-queue file, it is not a promise to act later -->\n",
                "### `.bee/deferred-queue.jsonl`\n",
                "\n",
                "Entries deferred here are written by the doc-impact-synthesis door.\n",
                "The capture queue is flushed later by bee-capturing.\n",
                "<!-- /bee:not-a-deferral -->\n",
            ),
        );
        write_freshness_capped_cell(root, "demo", "docs/handbook/register.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
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
        write_empty_doc_deferral_baseline(root);
        w(root, "docs/history/demo/CONTEXT.md", "# Demo\n\nThis is deferred for now.\n");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/history/demo/CONTEXT.md:3"), "{}", door.detail);
    }

    #[test]
    fn doc_deferral_door_deferral_decision_demotes_to_non_blocking_with_the_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_empty_doc_deferral_baseline(root);
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        w(
            root,
            ".bee/decisions.jsonl",
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-08-16T00:00:00.000Z\",\"decision\":\"defer registering a trigger for demo until the audit lands\",\"rationale\":\"r\",\"tags\":[\"doc-deferral\"],\"scope\":\"repo\"}\n",
        );
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
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
        // defaults-and-agent-env D1: absent uat_stop now reads as Close,
        // which would grow a blocking uat door for this fixture's
        // unclassified "demo" feature — pin "off" since this test is about
        // a different door.
        let root = repo(&tmp, r#"{"commands":{"test":"echo suite-green"},"uat_stop":"off"}"#);
        write_empty_doc_deferral_baseline(&root);
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

    // ─── doc-deferral-baseline: D1-D7 ────────────────────────────────────────

    /// D2: a repo with no baseline file seeds it on a REAL close run — it
    /// records what it flagged, passes, and writes the tracked file. (Here
    /// the one docs/ file is also the whole repo-wide seed set, so D6's
    /// wider walk and the old scan-set walk agree; the two tests below
    /// separate them.)
    #[test]
    fn doc_deferral_door_seeds_the_baseline_on_a_real_run_and_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert!(!doc_deferral_baseline_path(root).exists());
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        let bytes = std::fs::read(doc_deferral_baseline_path(root)).unwrap();
        let expected = doc_deferral_baseline_json(&[(
            "docs/knowledge/areas/demo/notes.md",
            &["This work is deferred for now."],
        )]);
        assert_eq!(String::from_utf8(bytes).unwrap(), expected);
    }

    /// Two consecutive runs over an unchanged tree — the seed run, then a
    /// second run that only ENFORCES — leave the tracked baseline file
    /// byte-identical: observed on disk, never inferred from the door's own
    /// verdict (a write-once law is invisible to an outcome-only assertion).
    #[test]
    fn doc_deferral_door_second_run_over_an_unchanged_tree_blocks_nothing_and_stays_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let seeded = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!seeded.blocking, "{}", seeded.detail);
        let after_seed = read_doc_deferral_baseline_bytes(root);
        let enforced = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!enforced.blocking, "{}", enforced.detail);
        assert_eq!(enforced.detail, "clear");
        let after_enforce = read_doc_deferral_baseline_bytes(root);
        assert_eq!(after_seed, after_enforce);
    }

    /// D1: a line added AFTER the seed blocks, while every already-baselined
    /// line in the same scan set stays silent.
    #[test]
    fn doc_deferral_door_blocks_a_line_added_after_the_seed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let seeded = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!seeded.blocking, "{}", seeded.detail);
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "line one\nThis work is deferred for now.\nAnother bit is deferred for now too.\n",
        );
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:3"), "{}", door.detail);
        assert!(!door.detail.contains(":2 "), "{}", door.detail);
    }

    /// D1's whole point: identity is the line's normalized CONTENT, never
    /// its line number — a baselined line is still recognized after
    /// unrelated lines are inserted above it in the same file.
    #[test]
    fn doc_deferral_door_recognizes_a_baselined_line_after_unrelated_lines_are_inserted_above_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let seeded = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!seeded.blocking, "{}", seeded.detail);
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "an unrelated new line\nanother unrelated line\nline one\nThis work is deferred for now.\n",
        );
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    /// D5: `--dry-run` with no baseline file writes nothing and reports the
    /// door non-blocking, naming the count it would baseline on a real run.
    #[test]
    fn doc_deferral_door_dry_run_with_no_baseline_writes_nothing_and_reports_the_would_baseline_count() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", true).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.contains("1 pre-existing deferral line(s)"), "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:2"), "{}", door.detail);
        assert!(!doc_deferral_baseline_path(root).exists());
    }

    /// The repo-wide seed set runs to four figures on a real docs tree, and
    /// this detail is both printed and embedded in the JSON doors payload.
    /// Spelling every message out made it 143 KB on the bee repo itself. The
    /// COUNT stays exact — D5 wants an honest prediction — but the sample is
    /// capped and the remainder summarised.
    ///
    /// Every number below is a LITERAL on purpose. Deriving them from
    /// `DOC_DEFERRAL_DRY_RUN_SAMPLE` made this test pass with the cap raised
    /// to 100000 — it asserted only that the code agreed with itself.
    #[test]
    fn doc_deferral_door_dry_run_detail_caps_the_sample_and_still_names_the_exact_count() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for i in 0..27 {
            w(
                root,
                &format!("docs/knowledge/areas/demo/note{i}.md"),
                "intro line\nThis work is deferred for now.\n",
            );
        }
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/note0.md");
        let door = build_doc_deferral_door(root, "demo", true).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(
            door.detail.contains("27 pre-existing deferral line(s)"),
            "the exact count must survive the cap: {}",
            door.detail
        );
        assert!(
            door.detail.contains("and 7 more"),
            "the remainder must be summarised: {}",
            door.detail
        );
        assert_eq!(
            door.detail.matches("deferral-shaped prose").count(),
            20,
            "exactly the sample is spelled out, not every match: {}",
            door.detail
        );
        assert!(
            door.detail.len() < 4096,
            "the detail is printed and embedded in JSON; it stayed {} bytes",
            door.detail.len()
        );
        assert!(!doc_deferral_baseline_path(root).exists());
    }

    /// D6: the SEED is REPO-WIDE — it records deferral lines from every
    /// markdown file under `docs/`, including files the closing feature's
    /// own scan set never sees. Without that, the seed would freeze only the
    /// docs one feature happened to touch, and the NEXT feature to close
    /// over a different long-lived doc would enter enforcement against an
    /// empty entry and block on every pre-existing line in it.
    #[test]
    fn doc_deferral_door_seed_is_repo_wide_and_covers_docs_outside_the_feature_scan_set() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // In "demo"'s scan set.
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        // NOT in "demo"'s scan set — another feature's long-lived doc.
        w(root, "docs/history/other/CONTEXT.md", "# Other\n\nThat migration is deferred for now.\n");
        let scan_set = doc_deferral_scan_files(root, "demo").unwrap();
        assert!(!scan_set.iter().any(|f| f == "docs/history/other/CONTEXT.md"), "{scan_set:?}");

        let seeded = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!seeded.blocking, "{}", seeded.detail);

        // Both files are frozen, not just the one "demo" touched.
        let expected = doc_deferral_baseline_json(&[
            ("docs/history/other/CONTEXT.md", &["That migration is deferred for now."]),
            ("docs/knowledge/areas/demo/notes.md", &["This work is deferred for now."]),
        ]);
        assert_eq!(String::from_utf8(read_doc_deferral_baseline_bytes(root)).unwrap(), expected);

        // D6's whole point: the next feature closing over that other doc is
        // already covered, instead of eating its pre-existing line.
        let other = build_doc_deferral_door(root, "other", false).unwrap();
        assert!(!other.blocking, "{}", other.detail);
        assert_eq!(other.detail, "clear");
    }

    /// D6: a seed run that flags NOTHING still writes the baseline file. An
    /// absent file IS the seed state, so skipping the write would leave the
    /// next close reading `Missing` and ADOPTING the first genuine deferral
    /// line anyone adds — a true positive swallowed by the very mechanism
    /// built to catch it.
    #[test]
    fn doc_deferral_door_seed_with_nothing_to_record_still_writes_the_file_and_a_later_line_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nplain prose with nothing pending.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        assert!(!doc_deferral_baseline_path(root).exists());

        let seeded = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!seeded.blocking, "{}", seeded.detail);
        assert_eq!(seeded.detail, "clear");
        // Written anyway, empty — observed on disk, never inferred from the
        // verdict (both states report the same non-blocking door).
        assert_eq!(
            String::from_utf8(read_doc_deferral_baseline_bytes(root)).unwrap(),
            doc_deferral_baseline_json(&[])
        );

        // The repo is now in ENFORCE, so a genuine deferral line added after
        // the seed BLOCKS instead of being adopted by a second seed run.
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "line one\nplain prose with nothing pending.\nThis work is deferred for now.\n",
        );
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(door.blocking, "{}", door.detail);
        assert!(door.detail.contains("docs/knowledge/areas/demo/notes.md:3"), "{}", door.detail);
    }

    /// D5 + D6: `--dry-run` still writes nothing, and the count it names is
    /// the REPO-WIDE seed a real close would perform — never the closing
    /// feature's narrower scan set, which would under-report it.
    #[test]
    fn doc_deferral_door_dry_run_names_the_repo_wide_seed_count_not_the_scan_set_count() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        w(root, "docs/knowledge/areas/demo/notes.md", "line one\nThis work is deferred for now.\n");
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        w(root, "docs/history/other/CONTEXT.md", "# Other\n\nThat migration is deferred for now.\n");

        let door = build_doc_deferral_door(root, "demo", true).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert!(door.detail.contains("2 pre-existing deferral line(s)"), "{}", door.detail);
        assert!(door.detail.contains("2 markdown file(s) under docs/"), "{}", door.detail);
        assert!(door.detail.contains("docs/history/other/CONTEXT.md:3"), "{}", door.detail);
        assert!(!doc_deferral_baseline_path(root).exists());
    }

    /// D4: the citation escape still clears a NEW line the baseline does not
    /// cover — it resolves before the baseline is ever consulted.
    #[test]
    fn doc_deferral_door_trigger_citation_clears_a_new_line_the_baseline_does_not_cover() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_doc_deferral_baseline(root, &[("docs/knowledge/areas/demo/notes.md", &["unrelated baselined line"])]);
        write_trigger(root, "demo-trigger");
        w(
            root,
            "docs/knowledge/areas/demo/notes.md",
            "unrelated baselined line\nThis NEW work is deferred for now, see `demo-trigger`.\n",
        );
        write_freshness_capped_cell(root, "demo", "docs/knowledge/areas/demo/notes.md");
        let door = build_doc_deferral_door(root, "demo", false).unwrap();
        assert!(!door.blocking, "{}", door.detail);
        assert_eq!(door.detail, "clear");
    }

    #[test]
    fn dispatch_prepare_with_expertise_renders_expertise_section() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let expertise_block = "- skills/bee-swarming/SKILL.md — swarming contract. Read it to follow worker protocol.\n- docs/knowledge/index.md — knowledge index. Read it to understand patterns.";
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "claude",
            "cell",
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            Some(expertise_block),
        )
        .unwrap()
        else {
            panic!("expected an envelope")
        };
        let p = v.get("payload").unwrap();
        let prompt = p.get("prompt").unwrap().as_str().unwrap();
        assert!(prompt.contains("Expertise — dispatcher-picked; read/load before implementing:"));
        assert!(prompt.contains("- skills/bee-swarming/SKILL.md — swarming contract. Read it to follow worker protocol."));
        assert!(prompt.contains("- docs/knowledge/index.md — knowledge index. Read it to understand patterns."));
    }

    #[test]
    fn dispatch_prepare_without_expertise_renders_no_expertise_section() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","trace":{"worker":"w"}}"#,
        );
        let Prepared::Value(v) = prepare_dispatch(
            &root,
            "claude",
            "cell",
            Some("c-1"),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap()
        else {
            panic!("expected an envelope")
        };
        let p = v.get("payload").unwrap();
        let prompt = p.get("prompt").unwrap().as_str().unwrap();
        assert!(!prompt.contains("Expertise — dispatcher-picked"));
    }

    #[test]
    fn run_dispatch_prepare_refuses_malformed_expertise_line() {
        let (flags, use_json) = parse_flags(&[
            "--runtime",
            "claude",
            "--kind",
            "cell",
            "--cell",
            "c-1",
            "--worker",
            "w",
            "--expertise",
            "not a valid three part line",
        ])
        .unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let _root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        // keys_known succeeds
        assert!(crate::verbs::reservations::keys_known(
            &flags,
            &[
                "runtime",
                "kind",
                "cell",
                "worker",
                "force-ownership",
                "claim",
                "session-id",
                "purpose",
                "expertise",
            ],
        ));
        let flag_val = flags.get("expertise").unwrap();
        let raw = match flag_val {
            FlagV::S(s) => s.as_str(),
            _ => "",
        };
        let err = parse_expertise(raw).unwrap_err();
        assert!(err.contains("malformed --expertise line"), "got {err}");
    }

    // ── merge-ready-fact D2: close records WHY it is still standing ─────────
    //
    // `bee close` is the writer of `merge_ready.blocked_by` — the names of
    // the doors still standing, so a board can say what the feature waits on
    // without re-deriving every door itself. The three tests below pin the
    // three places a full doors vector exists: the dry-run listing, the
    // proof-debt refusal arm, and the green path. `merge_ready` is seeded by
    // writing the lane record directly (test setup — production seeds it
    // from the last cap, verbs/workflow_store/merge_ready.rs `set_after_cap`).

    /// A lane record carrying an already-seeded `merge_ready`, with the
    /// blocked_by list pre-dirtied so a rewrite is visible as a rewrite.
    fn seed_merge_ready_lane(root: &Path, feature: &str, extra: &str, blocked_by: &str) {
        w(
            root,
            &format!(".bee/lanes/{feature}.json"),
            &format!(
                r#"{{"feature":"{feature}","phase":"execution","mode":"feature"{extra},
                     "merge_ready":{{"since":"2026-01-01T00:00:00.000Z","branch":"wt/{feature}",
                     "worktree_id":"wt-{feature}","uat":"pending","blocked_by":{blocked_by}}}}}"#
            ),
        );
    }

    fn read_merge_ready(root: &Path, feature: &str) -> Value {
        let raw = std::fs::read_to_string(
            root.join(".bee").join("lanes").join(format!("{feature}.json")),
        )
        .unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        parsed.get("merge_ready").cloned().unwrap_or(Value::Null)
    }

    /// The judge-debt door's name lands on the fact — and ONLY that name.
    /// The "uat" door is blocking in this same fixture and is deliberately
    /// left out: the fact carries the uat answer in its own `uat` field, so
    /// listing it under `blocked_by` too would say it twice. Both the
    /// dry-run vector and the real one write, because they report the same
    /// truth about the same feature.
    #[test]
    fn close_records_the_blocking_doors_on_the_features_merge_ready_fact() {
        let tmp = tempfile::tempdir().unwrap();
        // uat_stop absent reads as Close, which grows the blocking uat door
        // this test needs in order to prove the exclusion.
        let root = repo(&tmp, "{}");
        seed_merge_ready_lane(&root, "demo", r#","route":{"lane":"standard"}"#, r#"["stale-door"]"#);
        // A scribing run recorded after the cap clears the scribing-debt
        // door, so judge-debt is the door that actually surfaces.
        w(
            &root,
            ".bee/logs/scribing-runs.jsonl",
            "{\"feature\":\"demo\",\"ts\":\"2026-08-12T00:00:01.000Z\"}\n",
        );
        // Capped, behavior_change, unjudged, and with no report at all (so
        // the tests door reads it as a legacy cap and never blocks).
        w(
            &root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"behavior_change":true,"capped_at":"2026-08-12T00:00:00.000Z"}}"#,
        );

        // The dry run writes too: it lists the same doors, so it knows the
        // same truth.
        let Out::Emit(dry, _, dry_code) =
            close_handler(&root, "demo", true, None, None, &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(dry_code, 0, "a dry run never refuses");
        assert_eq!(read_merge_ready(&root, "demo")["blocked_by"], json!(["judge-debt"]), "{dry:?}");

        // Re-dirty the list, then take the real close.
        seed_merge_ready_lane(&root, "demo", r#","route":{"lane":"standard"}"#, r#"["stale-door"]"#);
        let Out::Emit(result, _, code) =
            close_handler(&root, "demo", false, None, None, &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1, "judge debt refuses");
        let doors = result.get("doors").unwrap().as_array().unwrap();
        assert_eq!(
            doors.iter().find(|d| d["door"] == "judge-debt").unwrap()["blocking"],
            json!(true)
        );
        assert_eq!(
            doors.iter().find(|d| d["door"] == "uat").unwrap()["blocking"],
            json!(true),
            "the uat door must be blocking here, or the exclusion below proves nothing"
        );

        let fact = read_merge_ready(&root, "demo");
        assert_eq!(fact["blocked_by"], json!(["judge-debt"]), "{fact}");
        assert_eq!(fact["uat"], json!("pending"), "flipping uat is the gate's job, never close's");
        assert_eq!(
            fact["since"],
            json!("2026-01-01T00:00:00.000Z"),
            "the rest of the fact is untouched"
        );
    }

    /// The proof-debt refusal arm assembles its own complete doors vector
    /// and returns before the green path is ever reached — a close stopped
    /// there still records the door it stopped at.
    #[test]
    fn a_close_stopped_at_the_tests_door_still_records_that_door() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"uat_stop":"off"}"#);
        seed_merge_ready_lane(&root, "demo", "", "[]");
        w(
            &root,
            ".bee/cells/demo-1.json",
            r#"{"id":"demo-1","feature":"demo","status":"capped","trace":{"report":{"outcome":"o","commit":"c","files":[],"tests":"","deviations":[]}}}"#,
        );

        let Out::Emit(_, text, code) =
            close_handler(&root, "demo", false, None, None, &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1, "proof debt refuses at the tests door: {text}");
        assert_eq!(read_merge_ready(&root, "demo")["blocked_by"], json!(["tests"]));
    }

    /// A green close writes the empty list back — nothing stands, and the
    /// last refusal's door names must not linger as a stale answer.
    #[test]
    fn a_green_close_writes_the_empty_door_list_back() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"uat_stop":"off"}"#);
        seed_merge_ready_lane(&root, "demo", "", r#"["judge-debt","tests"]"#);

        let Out::Emit(_, text, code) =
            close_handler(&root, "demo", false, None, None, &HashMap::new()).unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 0, "nothing blocks this close: {text}");
        assert_eq!(read_merge_ready(&root, "demo")["blocked_by"], json!([]));
    }

    // ═══ the WORK declares its job role ════════════════════════════════════
    //
    // model-role-split D3 (store 3c9d6262) with its literal lists fixed by
    // 561e1bda: a cell execution asks for [<the cell's own role>, code,
    // generation]; a read-shaped cell asks [read, extraction, generation]; a
    // review dispatch asks [review, generation]; the advisor asks [advisor]
    // ALONE (4faf1de9).

    /// A host that carries no `code` and no `read` key — every host that
    /// onboarded before mrs-10, and the one the fall-through tail exists for.
    ///
    /// NO VALUE HERE IS A BUILT-IN DEFAULT, and the values are deliberately
    /// ROTATED against `default_models("claude")` (`models.rs:79-83`:
    /// extraction `haiku`, generation `sonnet`, review `opus`). This fixture
    /// used to carry those three byte for byte — and `normalize_models` seeds
    /// that same map into every table BEFORE any config overlay, so a config
    /// repeating the defaults is indistinguishable from an EMPTY one. Under
    /// it the fall-through tests below asserted only "an unconfigured role
    /// lands on sonnet", which is true whether the resolver reads this host's
    /// table or ignores it entirely: a tail that regressed to `default_models`
    /// stayed green. The `-custom` values make the two answers different
    /// strings, so the assertions bite; the rotation catches the narrower
    /// regression of reading the right KEY out of the wrong TABLE.
    /// `the_pre_roles_fixture_is_distinguishable_from_an_empty_config` pins
    /// the property so the fixture cannot drift back.
    const HOST_BEFORE_ROLES: &str =
        r#"{"models":{"claude":{"extraction":"haiku-custom","generation":"opus-custom","review":"sonnet-custom"}}}"#;

    /// The same pre-roles host on the CODEX runtime, and the case this block
    /// had no variant for at all. codex is where a tail that stopped reading
    /// the host's table shows LOUDEST: every codex entry in `default_models`
    /// is `null` (`models.rs:81-83`), so the built-ins can answer no role at
    /// all — the dispatch resolves NO model rather than a different one.
    const CODEX_HOST_BEFORE_ROLES: &str =
        r#"{"models":{"codex":{"extraction":"gpt-5-mini-custom","generation":"gpt-5-custom","review":"gpt-5-pro-custom"}}}"#;

    /// THE ANTI-RECURRENCE DEVICE for this block.
    ///
    /// An independent audit found the two fall-through tests below could not
    /// fail, because the fixture they drive off repeated `default_models`
    /// verbatim. This asks the TWO TABLES THEMSELVES — never a hand-written
    /// list — so a fixture edited back toward the built-ins fails HERE, at
    /// the fixture, instead of silently unpinning THE safety property this
    /// whole feature rests on: no existing host's dispatch changes model
    /// until the operator opts in.
    #[test]
    fn the_pre_roles_fixture_is_distinguishable_from_an_empty_config() {
        for (runtime, config) in
            [("claude", HOST_BEFORE_ROLES), ("codex", CODEX_HOST_BEFORE_ROLES)]
        {
            let tmp = tempfile::tempdir().unwrap();
            let root = repo(&tmp, config);
            let models = read_models(&root).unwrap();
            let table = models.get(runtime).unwrap().as_object().unwrap();
            let defaults = default_models(runtime);
            for (slot, value) in table {
                assert_ne!(
                    Some(value),
                    defaults.get(slot),
                    "{runtime}.{slot}: the fixture repeats the built-in default, so a \
                     resolver that ignored this host's table would still answer correctly"
                );
            }
            for slot in ["extraction", "generation", "review"] {
                assert!(
                    table.get(slot).map(Value::is_string).unwrap_or(false),
                    "{runtime}.{slot}: the tail walks this slot, so the host must configure it"
                );
            }
            // …and bee's own tail names stay ABSENT: that absence is what
            // makes this a host from BEFORE the roles existed.
            for name in ASKED_ROLES {
                assert!(
                    table.get(name).is_none(),
                    "{runtime}: {name} must stay unconfigured on a pre-roles host"
                );
            }
        }
    }

    fn cell_with(root: &Path, id: &str, fields: &str) {
        w(
            root,
            &format!(".bee/cells/{id}.json"),
            &format!(
                r#"{{"id":"{id}","feature":"f","title":"some work",{fields},"status":"claimed","trace":{{"worker":"w"}}}}"#
            ),
        );
    }

    fn cell_envelope(root: &Path, id: &str, role: Option<&str>) -> Value {
        cell_envelope_on(root, "claude", id, role)
    }

    /// `cell_envelope` with the runtime spelled out — codex resolves against
    /// its own table, whose built-in defaults are all null.
    fn cell_envelope_on(root: &Path, runtime: &str, id: &str, role: Option<&str>) -> Value {
        let out = prepare_dispatch_with_role(
            root,
            runtime,
            "cell",
            role,
            Some(id),
            Some("w"),
            false,
            None,
            None,
            false,
            None,
        )
        .unwrap();
        let Prepared::Value(v) = out else { panic!("expected an envelope for {id}") };
        v
    }

    /// The whole point of the feature: the cell names its job, the host names
    /// a model for that job, and the dispatch gets it.
    #[test]
    fn a_cell_role_the_host_configures_resolves_that_hosts_model() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":"sonnet","test":"grok-code"}}}"#,
        );
        cell_with(&root, "c-1", r#""role":"test""#);

        let v = cell_envelope(&root, "c-1", None);
        let payload = v.get("payload").unwrap();
        assert_eq!(payload.get("model"), Some(&json!("grok-code")), "the job's own model");
        assert_eq!(payload.get("subagent_type"), Some(&json!("bee-build")));
        assert!(payload
            .get("prompt")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("[bee-tier: test]\n"));
        let econ = v.get("economics").unwrap();
        assert_eq!(econ.get("logical_tier"), Some(&json!("test")));
        assert_eq!(econ.get("tier_source"), Some(&json!("cell")));
    }

    /// …and a host that configured nothing for that job runs EXACTLY where it
    /// ran before this feature. No refusal, no built-in default: the tail
    /// walks to `generation`, and the marker names the role that RESOLVED so
    /// the model-guard — which denies a marker naming an unconfigured role —
    /// still lets through the dispatch bee itself prepared.
    ///
    /// "No built-in default" is the load-bearing half, and it is what the
    /// assertions actually ask: the payload must carry THIS HOST'S generation
    /// model, `opus-custom`, a string `default_models` does not contain
    /// anywhere. If the tail ever stops reading the host's table and answers
    /// out of the built-ins, this reads `sonnet` and goes red — which is the
    /// only shape in which "no existing host silently migrates" is a claim a
    /// test can refute.
    #[test]
    fn a_cell_role_nothing_configures_falls_through_to_the_historical_model() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, HOST_BEFORE_ROLES);
        cell_with(&root, "c-1", r#""role":"code""#);
        cell_with(&root, "c-2", r#""role":"design""#);

        for id in ["c-1", "c-2"] {
            let v = cell_envelope(&root, id, None);
            assert_eq!(v.get("ok"), None, "{id}: a role nothing configures is not a refusal");
            assert_eq!(v.get("tool"), Some(&json!("Agent")), "{id}");
            let payload = v.get("payload").unwrap();
            assert_eq!(
                payload.get("model"),
                Some(&json!("opus-custom")),
                "{id}: the model THIS HOST has run for years, read out of its own \
                 config — `default_models` says `sonnet` here and must not win"
            );
            assert!(
                payload
                    .get("prompt")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .starts_with("[bee-tier: generation]\n"),
                "{id}: the marker names the role that resolved"
            );
            let econ = v.get("economics").unwrap();
            assert_eq!(econ.get("logical_tier"), Some(&json!("generation")), "{id}");
            assert_eq!(econ.get("tier_source"), Some(&json!("cell")), "{id}");
            assert_eq!(
                econ.get("requested_model"),
                Some(&json!("opus-custom")),
                "{id}: the audit line names the same host model the payload pins"
            );
        }
    }

    /// A read-shaped cell takes the READ consumer's list, so it lands on the
    /// historical read model rather than on generation — D9 backfills every
    /// `tier: extraction` cell to `role: read`, and this is where they land.
    ///
    /// Same discipline as the test above: `haiku-custom` is THIS HOST's
    /// extraction model and appears in no built-in table, so a tail that
    /// answered out of `default_models` would read `haiku` and go red.
    #[test]
    fn a_read_cell_falls_through_to_the_historical_read_model() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, HOST_BEFORE_ROLES);
        cell_with(&root, "c-1", r#""role":"read""#);

        let v = cell_envelope(&root, "c-1", None);
        let payload = v.get("payload").unwrap();
        assert_eq!(
            payload.get("model"),
            Some(&json!("haiku-custom")),
            "the host's own extraction model, not `default_models`' `haiku`"
        );
        assert!(payload
            .get("prompt")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("[bee-tier: extraction]\n"));
        assert_eq!(
            v.get("economics").unwrap().get("requested_model"),
            Some(&json!("haiku-custom")),
            "the audit line names the same host model the payload pins"
        );
    }

    /// The same fall-through on CODEX, the runtime this block had no case for
    /// at all — and the one where the regression this pair guards against
    /// would be unmissable. Every codex entry in `default_models` is `null`,
    /// so a tail reading the built-ins instead of this host's table resolves
    /// `Resolved::Budget`: no model requested, nothing pinned, the dispatch
    /// quietly running on whatever the session happens to be.
    ///
    /// codex carries no `model` on the payload (its `spawn_agent` arm takes
    /// the model off the resolved slot, not off a tool parameter), so the
    /// model that resolved is read where codex actually records it — the
    /// economics audit line — beside the `[bee-tier: …]` marker on the
    /// message.
    #[test]
    fn a_codex_cell_role_nothing_configures_falls_through_to_that_hosts_model() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, CODEX_HOST_BEFORE_ROLES);
        cell_with(&root, "c-1", r#""role":"code""#);
        cell_with(&root, "c-2", r#""role":"design""#);
        cell_with(&root, "c-3", r#""role":"read""#);

        for (id, role, model) in [
            ("c-1", "generation", "gpt-5-custom"),
            ("c-2", "generation", "gpt-5-custom"),
            ("c-3", "extraction", "gpt-5-mini-custom"),
        ] {
            let v = cell_envelope_on(&root, "codex", id, None);
            assert_eq!(v.get("ok"), None, "{id}: a role nothing configures is not a refusal");
            assert_eq!(v.get("tool"), Some(&json!("spawn_agent")), "{id}");
            assert!(
                v.get("payload")
                    .unwrap()
                    .get("message")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .starts_with(&format!("[bee-tier: {role}]\n")),
                "{id}: the marker names the role that resolved"
            );
            let econ = v.get("economics").unwrap();
            assert_eq!(econ.get("logical_tier"), Some(&json!(role)), "{id}");
            assert_eq!(econ.get("tier_source"), Some(&json!("cell")), "{id}");
            assert_eq!(
                econ.get("requested_model"),
                Some(&json!(model)),
                "{id}: this host's own model — every codex entry in \
                 `default_models` is null, so the built-ins could not have answered"
            );
        }
    }

    /// Precedence: an explicit `--role` names the slot directly and outranks
    /// the cell's own declared role (store 8ff6e79e over D3).
    #[test]
    fn an_explicit_role_outranks_the_cells_recorded_role() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":"sonnet","review":"opus","test":"grok-code"}}}"#,
        );
        cell_with(&root, "c-1", r#""role":"test""#);

        let v = cell_envelope(&root, "c-1", Some("review"));
        assert_eq!(v.get("payload").unwrap().get("model"), Some(&json!("opus")));
        let econ = v.get("economics").unwrap();
        assert_eq!(econ.get("logical_tier"), Some(&json!("review")));
        assert_eq!(econ.get("tier_source"), Some(&json!("flag")), "the caller chose it");
    }

    /// A pre-mrs-8 record — no `role` at all — keeps the exact path it had:
    /// its recorded `tier` selects, stamps `cell`, and an unconfigured one is
    /// still the typed refusal, never a fall-through.
    #[test]
    fn a_role_less_cell_still_resolves_and_refuses_on_its_recorded_tier() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        cell_with(&root, "c-1", r#""tier":"generation""#);
        cell_with(&root, "c-2", r#""tier":"quantum""#);

        let v = cell_envelope(&root, "c-1", None);
        assert_eq!(v.get("payload").unwrap().get("model"), Some(&json!("sonnet")));
        assert_eq!(
            v.get("economics").unwrap().get("tier_source"),
            Some(&json!("cell")),
            "the recorded tier still selects"
        );

        let refused = cell_envelope(&root, "c-2", None);
        assert_eq!(refused.get("ok"), Some(&json!(false)));
        assert_eq!(refused.get("reason"), Some(&json!("tier_not_configured")));
    }

    /// The advisor list is ONE name with no fall-through (4faf1de9): an
    /// unconfigured advisor refuses, it never walks on to code or generation,
    /// and no cell's role can drag it into one.
    #[test]
    fn the_advisor_list_has_no_fall_through() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(
            &tmp,
            r#"{"models":{"claude":{"generation":"sonnet","code":"sonnet-code"}}}"#,
        );
        let Prepared::Value(v) = prepare_dispatch(
            &root, "claude", "advisor", None, None, false, None, None, false, None,
        )
        .unwrap() else {
            panic!("expected an envelope")
        };
        assert_eq!(v.get("ok"), Some(&json!(false)));
        assert_eq!(v.get("reason"), Some(&json!("advisor_not_configured")));
    }

    /// The warn has to stay BELIEVABLE. On a host that never opted in, every
    /// name a cell dispatch walks is silent — bee's own tail is bee's
    /// plumbing, not the operator's request — while a job role nothing
    /// configures is still loud. Asked of the real ordered lists against a
    /// real config, because the warn itself writes to stderr.
    /// role-edge-hardening D1: a chain key no dispatch can travel under is
    /// named, not silently published. Chains match on the RESOLVED role and
    /// model, so a role-keyed chain for an unconfigured role is dead on
    /// arrival — and until now it died without a word.
    #[test]
    fn a_chain_key_nothing_resolves_is_named_as_dead() {
        let models = normalize_models(Some(&json!({
            "claude": {"code": "sonnet", "test": "opus-custom"},
        })));
        let chains = normalize_fallback_chains(Some(&json!({
            "test": ["sonnet"],          // live: configured role
            "code": ["haiku"],           // live: configured role
            "opus-custom": ["sonnet"],   // live: a model some slot resolves to
            "sonnet": ["haiku"],         // live: a configured model value
            "anthropic/*": ["local/q"],  // live: wildcard, never checked
            "design": ["sonnet"],        // DEAD: no runtime configures it
        })));
        assert_eq!(dead_chain_keys(&chains, &models), vec!["design".to_string()]);

        // Built-in default names stay live even with an empty config — a
        // chain keyed on "generation" fires on every host.
        let empty = normalize_models(None);
        let generic = normalize_fallback_chains(Some(&json!({"generation": ["sonnet"]})));
        assert!(dead_chain_keys(&generic, &empty).is_empty());
    }

    /// role-surface-cleanup D1: the ordered list never repeats a name. A
    /// duplicate makes the fall-through warn fire twice for one dispatch,
    /// and the first copy names the very name that just failed — observed
    /// live for `role: "code"` on a host whose window is shut via `read`.
    #[test]
    fn the_ordered_role_list_never_repeats_a_name() {
        for role in ["code", "read", "generation", "extraction", "review", "test", "design"] {
            let list = cell_role_list(role);
            let mut seen = std::collections::HashSet::new();
            for name in &list {
                assert!(
                    seen.insert(*name),
                    "cell_role_list({role:?}) repeats {name:?}: {list:?}"
                );
            }
            assert_eq!(list.first(), Some(&role), "the cell's own role stays the head");
            assert!(
                list.contains(&"generation"),
                "the historical backstop stays reachable — as the tail, or as the head when \
                 the role IS the backstop: {list:?}"
            );
        }
    }

    #[test]
    fn bees_own_tail_is_silent_while_an_unconfigured_job_role_still_warns() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, HOST_BEFORE_ROLES);
        let models = read_models(&root).unwrap();

        for role in ["code", "read", "generation", "extraction", "review"] {
            for name in cell_role_list(role) {
                assert!(
                    !role_is_unknown(&models, "claude", name),
                    "role {role}: walking {name} would warn on every dispatch"
                );
            }
        }
        for name in tier_role_list("review") {
            assert!(!role_is_unknown(&models, "claude", name), "a review dispatch warns on {name}");
        }

        // …and the names nobody configured stay loud, head or tail.
        for invented in ["test", "design", "migrate"] {
            assert!(
                role_is_unknown(&models, "claude", invented),
                "{invented} names no configured job and must warn"
            );
        }

        // Membership, never a fixed list: configure it and it goes quiet.
        let tmp2 = tempfile::tempdir().unwrap();
        let configured = repo(&tmp2, r#"{"models":{"claude":{"generation":"sonnet","test":"grok"}}}"#);
        let models2 = read_models(&configured).unwrap();
        assert!(!role_is_unknown(&models2, "claude", "test"));
    }

    /// mrs-24 (store `fef79243`) — the silence above is a MIGRATION WINDOW,
    /// and this is the assertion that it SHUTS.
    ///
    /// An audit found the suppression unconditional: `code` and `read` were
    /// exempt on every host forever, so a host that had opted in and then
    /// missed one of the two keys resolved that name silently for good —
    /// exactly the "silently accepted" case D2 (store `06e49368`) forbids,
    /// sheltering behind mrs-9's noise argument long after the noise it
    /// described was gone. The window keeps mrs-9's silence where it was
    /// earned (a host that configured NEITHER name, where falling through to
    /// the historical model is the intended no-op) and nowhere else.
    /// role-edge-hardening D1: the null boundary of the opt-in window,
    /// pinned. The r2 mutation probe narrowed `.is_some()` to non-null and
    /// the WHOLE suite stayed green, because every window fixture used
    /// string values — yet `.bee/config-sample.json` seeds codex as
    /// `"code": null, "read": null`, the exact shape the boundary decides.
    /// An explicit null IS an opt-in: the operator wrote the key, so they
    /// know the vocabulary, and the key resolves Budget rather than falling
    /// through — deliberately unset is not the same as never-heard-of-it.
    #[test]
    fn a_null_valued_asked_role_is_an_opt_in_and_the_window_shuts() {
        // The shape bee itself ships: both asked roles present, both null.
        let tmp = tempfile::tempdir().unwrap();
        let nulls = read_models(&repo(
            &tmp,
            r#"{"models":{"claude":{"generation":"opus-custom","code":null,"read":null}}}"#,
        ))
        .unwrap();
        assert!(
            host_opted_into_roles(&nulls, "claude"),
            "a null-valued asked role is a written key — the window shuts; \
             narrowing the check to non-null values must fail HERE"
        );
        // …and a present-but-null name is KNOWN, never warned: it resolves
        // Budget at its own slot instead of falling through.
        assert!(!role_is_unknown(&nulls, "claude", "code"), "null code is known");
        assert!(!role_is_unknown(&nulls, "claude", "read"), "null read is known");

        // One null sibling behaves exactly like one string sibling: the
        // window is shut either way.
        let tmp2 = tempfile::tempdir().unwrap();
        let one = read_models(&repo(
            &tmp2,
            r#"{"models":{"claude":{"generation":"opus-custom","code":null}}}"#,
        ))
        .unwrap();
        assert!(host_opted_into_roles(&one, "claude"));
    }

    #[test]
    fn opting_into_one_asked_role_makes_the_missing_sibling_warn() {
        // Un-migrated: the window is open, so both of bee's own names are
        // silent — this is the property the test above pins, restated here as
        // the baseline the rest of this test moves away from.
        let tmp = tempfile::tempdir().unwrap();
        let pre = read_models(&repo(&tmp, HOST_BEFORE_ROLES)).unwrap();
        assert!(!host_opted_into_roles(&pre, "claude"), "the pre-roles fixture has not opted in");
        for name in ASKED_ROLES {
            assert!(!role_is_unknown(&pre, "claude", name), "{name}: the window is open");
        }

        // Half-migrated: `code` is configured, so this operator KNOWS about
        // the role names. The key they missed is now something they can act
        // on, so it warns like any other unconfigured name.
        let tmp2 = tempfile::tempdir().unwrap();
        let half = read_models(&repo(
            &tmp2,
            r#"{"models":{"claude":{"generation":"opus-custom","code":"sonnet-custom"}}}"#,
        ))
        .unwrap();
        assert!(host_opted_into_roles(&half, "claude"));
        assert!(!role_is_unknown(&half, "claude", "code"), "a configured name never warns");
        assert!(
            role_is_unknown(&half, "claude", "read"),
            "the sibling key a half-migrated host missed must warn — this is the audit's defect"
        );

        // Fully migrated (what mrs-10 seeds into a fresh config): opted in and
        // complete, so the window is shut and nothing warns anyway.
        let tmp3 = tempfile::tempdir().unwrap();
        let full = read_models(&repo(
            &tmp3,
            r#"{"models":{"claude":{"generation":"opus-custom","code":"sonnet-custom","read":"haiku-custom"}}}"#,
        ))
        .unwrap();
        for name in ASKED_ROLES {
            assert!(!role_is_unknown(&full, "claude", name), "{name} is configured");
        }

        // The window is PER RUNTIME, because the table is. Opting claude in
        // says nothing about codex, whose dispatches would otherwise start
        // warning over a migration their own config never saw.
        let tmp4 = tempfile::tempdir().unwrap();
        let mixed = read_models(&repo(
            &tmp4,
            r#"{"models":{"claude":{"code":"sonnet-custom"},"codex":{"generation":"gpt-5-custom"}}}"#,
        ))
        .unwrap();
        assert!(host_opted_into_roles(&mixed, "claude"));
        assert!(!host_opted_into_roles(&mixed, "codex"));
        assert!(role_is_unknown(&mixed, "claude", "read"), "claude opted in; its gap is loud");
        assert!(!role_is_unknown(&mixed, "codex", "read"), "codex never opted in; its window is open");
    }

    // ── mrs-19: the runtime fallback chain (D10/D11, store 50808d48) ───────
    //
    // Scoped by 51341f84: bee PUBLISHES a chain and its gate, it never walks
    // one. So every test below asks what the payload SAYS — there is no retry
    // loop here to exercise, and adding one would be the decision's rejected
    // alternative.

    const CHAIN_MODELS: &str = r#""models":{"claude":{"code":"sonnet","read":"haiku","generation":"sonnet","review":"opus","advisor":"fable"}}"#;

    const CHAIN_CELL: &str =
        r#"{"id":"c-1","feature":"f","status":"claimed","role":"code","trace":{"worker":"w"}}"#;

    fn chain_repo(tmp: &tempfile::TempDir, retry: &str) -> PathBuf {
        let config = if retry.is_empty() {
            format!("{{{CHAIN_MODELS}}}")
        } else {
            format!("{{{CHAIN_MODELS},{retry}}}")
        };
        let root = repo(tmp, &config);
        w(&root, ".bee/cells/c-1.json", CHAIN_CELL);
        root
    }

    fn chain_payload(root: &Path, kind: &str) -> Value {
        let (cell, worker) = if kind == "cell" { (Some("c-1"), Some("w")) } else { (None, None) };
        let Prepared::Value(v) =
            prepare_dispatch(root, "claude", kind, cell, worker, false, None, None, false, None)
                .unwrap()
        else {
            panic!("{kind} dispatch did not produce a payload")
        };
        v.get("payload").cloned().unwrap()
    }

    /// The whole payload, as the bytes it serializes to. Byte equality is the
    /// question this feature has to answer, so the assertion asks it directly
    /// rather than comparing parsed maps (which compare order-blind).
    fn chain_payload_bytes(root: &Path, kind: &str) -> String {
        serde_json::to_string(&chain_payload(root, kind)).unwrap()
    }

    /// D10, the load-bearing one: with NO `retry.fallbackChains` configured,
    /// every dispatch payload is byte-identical to a bee that had never heard
    /// of chains.
    ///
    /// Two halves, because "identical to BEFORE" is not a thing a test can
    /// re-derive from the code it is testing. The three goldens are the exact
    /// bytes captured from the build one commit before this cell — a whole
    /// payload each, not a key spot-check. The loop then pins that every
    /// shape of chain config that matches NOTHING (absent, empty, unmatched,
    /// junk, and the refused `default` key) leaves all four kinds on those
    /// same bytes.
    #[test]
    fn no_fallback_chain_config_leaves_the_whole_dispatch_payload_byte_identical() {
        const GATHER: &str = r#"{"subagent_type":"bee-gather","prompt":"[bee-tier: generation]\nGather: locate and digest the requested paths/facts. Read-only — never write, never edit, never run a mutating command.\n\nPaths: <caller fills in the exact files/paths to read>\n\nDigest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked.","description":"gather (sonnet)","model":"sonnet"}"#;
        const REVIEWER: &str = r#"{"subagent_type":"bee-review","prompt":"[bee-tier: review]\nReview: check the given claim/diff against the repo. Read-only; may run read-only commands (tests, linters, the configured verify) to check evidence.\n\nPaths: <caller fills in the exact files/paths to read>\n\nDigest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked.","description":"reviewer (opus)","model":"opus"}"#;
        const ADVISOR: &str = r#"{"subagent_type":"general-purpose","prompt":"[bee-tier: advisor]\nAdvisor consult: produce an independent digest/opinion on the given question. Read-only.\n\nPaths: <caller fills in the exact files/paths to read>\n\nDigest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked.","description":"advisor (fable)","model":"fable"}"#;

        let t0 = tempfile::tempdir().unwrap();
        let none = chain_repo(&t0, "");
        assert_eq!(chain_payload_bytes(&none, "gather"), GATHER);
        assert_eq!(chain_payload_bytes(&none, "reviewer"), REVIEWER);
        assert_eq!(chain_payload_bytes(&none, "advisor"), ADVISOR);

        for retry in [
            r#""retry":{}"#,
            r#""retry":{"fallbackChains":{}}"#,
            r#""retry":"nonsense""#,
            // configured, but nothing here keys THIS host's dispatches
            r#""retry":{"fallbackChains":{"nowhere/*":["x"],"design":["y"],"gpt-5.5":["z"]}}"#,
            // every validation refusal at once: the `default` key D10 declined,
            // a non-array chain, an empty chain, a whitespace-only step, and a
            // chain looping to its own head
            r#""retry":{"fallbackChains":{"default":["sonnet"],"code":"opus","review":[],"advisor":["  "],"generation":["generation"]}}"#,
        ] {
            let t = tempfile::tempdir().unwrap();
            let root = chain_repo(&t, retry);
            for kind in ["cell", "gather", "reviewer", "advisor"] {
                assert_eq!(
                    chain_payload_bytes(&root, kind),
                    chain_payload_bytes(&none, kind),
                    "the {kind} payload moved under {retry}"
                );
            }
        }
    }

    /// A chain that DOES match adds exactly one payload key and moves nothing
    /// else — asked of the cell payload, the big one the goldens above leave
    /// out. Remove `fallback_chain` and the bytes are the no-chain bytes again.
    #[test]
    fn a_matching_chain_adds_exactly_one_payload_key_and_moves_nothing_else() {
        let t0 = tempfile::tempdir().unwrap();
        let none = chain_repo(&t0, "");
        let t1 = tempfile::tempdir().unwrap();
        let with = chain_repo(&t1, r#""retry":{"fallbackChains":{"code":["opus","haiku"]}}"#);

        let mut payload = chain_payload(&with, "cell");
        let obj = payload.as_object_mut().unwrap();
        assert!(obj.remove("fallback_chain").is_some(), "the chain was never published");
        assert_eq!(
            serde_json::to_string(&payload).unwrap(),
            chain_payload_bytes(&none, "cell")
        );
    }

    /// D10: a chain key may name a role, a concrete model selector, or a
    /// `provider/*` wildcard, and the most specific key wins — model, then
    /// wildcard, then role.
    #[test]
    fn a_chain_resolves_by_role_by_model_and_by_provider_wildcard_most_specific_first() {
        let chains = normalize_fallback_chains(Some(&json!({
            "code": ["by-role"],
            "anthropic/*": ["by-wildcard"],
            "anthropic/opus": ["by-model"],
            "sonnet": ["by-plain-model"],
        })));

        // each key kind resolves on its own …
        assert_eq!(
            resolve_fallback_chain(&chains, "code", "haiku"),
            Some(("code".into(), vec!["by-role".to_string()]))
        );
        assert_eq!(
            resolve_fallback_chain(&chains, "review", "sonnet"),
            Some(("sonnet".into(), vec!["by-plain-model".to_string()]))
        );
        assert_eq!(
            resolve_fallback_chain(&chains, "review", "anthropic/haiku"),
            Some(("anthropic/*".into(), vec!["by-wildcard".to_string()]))
        );
        // … and where several match, the narrowest key answers.
        assert_eq!(
            resolve_fallback_chain(&chains, "code", "anthropic/opus"),
            Some(("anthropic/opus".into(), vec!["by-model".to_string()]))
        );
        assert_eq!(
            resolve_fallback_chain(&chains, "code", "anthropic/haiku"),
            Some(("anthropic/*".into(), vec!["by-wildcard".to_string()]))
        );
        // A model-keyed chain follows the MODEL, whatever role carries it.
        assert_eq!(
            resolve_fallback_chain(&chains, "read", "sonnet"),
            Some(("sonnet".into(), vec!["by-plain-model".to_string()]))
        );
        // Nothing keyed for this dispatch is no chain, never a borrowed one.
        assert_eq!(resolve_fallback_chain(&chains, "read", "haiku"), None);

        // end to end: the role key on a real cell dispatch (role `code`
        // resolves models.claude.code = sonnet) …
        let t = tempfile::tempdir().unwrap();
        let root = chain_repo(&t, r#""retry":{"fallbackChains":{"code":["opus"]}}"#);
        let payload = chain_payload(&root, "cell");
        assert_eq!(payload.get("model"), Some(&json!("sonnet")));
        assert_eq!(payload.get("fallback_chain").unwrap().get("key"), Some(&json!("code")));
        assert_eq!(payload.get("fallback_chain").unwrap().get("chain"), Some(&json!(["opus"])));

        // … and the model key outranking it on the same dispatch.
        let t2 = tempfile::tempdir().unwrap();
        let root2 =
            chain_repo(&t2, r#""retry":{"fallbackChains":{"code":["opus"],"sonnet":["haiku"]}}"#);
        let by_model = chain_payload(&root2, "cell");
        assert_eq!(by_model.get("fallback_chain").unwrap().get("key"), Some(&json!("sonnet")));
        assert_eq!(by_model.get("fallback_chain").unwrap().get("chain"), Some(&json!(["haiku"])));

        // … and a provider wildcard, on a host whose models carry providers.
        let t3 = tempfile::tempdir().unwrap();
        let root3 = repo(
            &t3,
            r#"{"models":{"claude":{"code":"anthropic/opus"}},"retry":{"fallbackChains":{"anthropic/*":["local/qwen"]}}}"#,
        );
        w(&root3, ".bee/cells/c-1.json", CHAIN_CELL);
        let wild = chain_payload(&root3, "cell");
        assert_eq!(wild.get("model"), Some(&json!("anthropic/opus")));
        assert_eq!(wild.get("fallback_chain").unwrap().get("key"), Some(&json!("anthropic/*")));
    }

    /// D11: the payload publishes the gate in BOTH directions — the classes
    /// that may advance a step and the classes that may never. The executor
    /// must not have to re-derive that list; an unpublished rule is one every
    /// caller invents differently.
    #[test]
    fn the_published_chain_carries_both_halves_of_the_error_gate() {
        let t = tempfile::tempdir().unwrap();
        let root = chain_repo(&t, r#""retry":{"fallbackChains":{"code":["opus"]}}"#);
        let payload = chain_payload(&root, "cell");
        let chain = payload.get("fallback_chain").unwrap();

        assert_eq!(
            chain.get("advance_on"),
            Some(&json!([
                "quota_or_rate_limit",
                "provider_auth_or_policy_rejection",
                "empty_response",
                "malformed_tool_call_replay_safe",
                "stream_stall_or_connection_reset",
                "server_error_5xx"
            ]))
        );
        assert_eq!(
            chain.get("never_advance_on"),
            Some(&json!(["tool_error", "wrong_or_unwanted_result", "failed_proof", "red_test"]))
        );
        assert_eq!(chain.get("fallback_when"), Some(&json!(CHAIN_FALLBACK_WHEN)));
    }

    /// D11's negative half (store `50808d48`), pinned on the PUBLISHED gate.
    ///
    /// mrs-29, from an independent audit. What stood here asserted that
    /// `CHAIN_NEVER_ADVANCE_ON` and `CHAIN_ADVANCE_ON` are disjoint — two
    /// constants in the same crate agreeing with each other, which no caller
    /// can rely on and which stays green through any edit that moves a class
    /// out of BOTH lists. The pin belongs on the payload a caller receives.
    ///
    /// The oracle is D11's own sentence, transcribed here rather than read
    /// back out of the code under test: "A step **never** fires on a semantic
    /// failure — a tool error, a wrong or unwanted result, a failed proof, a
    /// red test." Delete a class from `CHAIN_NEVER_ADVANCE_ON` and this test
    /// goes red; move one into `CHAIN_ADVANCE_ON` and it goes red twice.
    ///
    /// THE CEILING, stated because the plan overstated it: bee PUBLISHES the
    /// chain and never walks one (decision `51341f84`), so nothing on bee's
    /// side can prove an executor OBEYS the gate. What is provable — and what
    /// this asserts — is that every dispatch bee publishes carries the gate
    /// correctly and in both directions. An executor that ignores it is
    /// outside bee's reach, and a test claiming otherwise would be the
    /// self-agreeing kind this one replaces.
    #[test]
    fn every_published_chain_refuses_to_advance_on_a_semantic_failure() {
        // D11's four semantic classes, from the decision text.
        const SEMANTIC: [&str; 4] =
            ["tool_error", "wrong_or_unwanted_result", "failed_proof", "red_test"];

        // Every shape of chain a dispatch can publish: role-keyed, model-keyed,
        // provider-wildcard, and a non-cell kind resolving its own slot. The
        // gate is a property of the published contract, never of one fixture.
        let t1 = tempfile::tempdir().unwrap();
        let by_role = chain_repo(&t1, r#""retry":{"fallbackChains":{"code":["opus"]}}"#);
        let t2 = tempfile::tempdir().unwrap();
        let by_model = chain_repo(&t2, r#""retry":{"fallbackChains":{"sonnet":["haiku"]}}"#);
        let t3 = tempfile::tempdir().unwrap();
        let by_review = chain_repo(&t3, r#""retry":{"fallbackChains":{"review":["sonnet"]}}"#);
        let t4 = tempfile::tempdir().unwrap();
        let wildcard = repo(
            &t4,
            r#"{"models":{"claude":{"code":"anthropic/opus"}},"retry":{"fallbackChains":{"anthropic/*":["local/qwen"]}}}"#,
        );
        w(&wildcard, ".bee/cells/c-1.json", CHAIN_CELL);

        for (root, kind) in [
            (&by_role, "cell"),
            (&by_model, "cell"),
            (&by_review, "reviewer"),
            (&wildcard, "cell"),
        ] {
            let payload = chain_payload(root, kind);
            let chain = payload
                .get("fallback_chain")
                .unwrap_or_else(|| panic!("{kind}: the dispatch published no chain to gate"));

            // Read BOTH halves off the payload — never off the constants that
            // built it. A caller has only these two arrays.
            let advance: Vec<&str> = chain
                .get("advance_on")
                .and_then(Value::as_array)
                .unwrap_or_else(|| panic!("{kind}: the published chain carries no advance_on"))
                .iter()
                .filter_map(Value::as_str)
                .collect();
            let never: Vec<&str> = chain
                .get("never_advance_on")
                .and_then(Value::as_array)
                .unwrap_or_else(|| panic!("{kind}: the published chain carries no never_advance_on"))
                .iter()
                .filter_map(Value::as_str)
                .collect();

            for class in SEMANTIC {
                assert!(
                    never.contains(&class),
                    "{kind}: the published gate omits the semantic failure {class} from \
never_advance_on — an executor reading this payload would have to invent the rule"
                );
                assert!(
                    !advance.contains(&class),
                    "{kind}: the published gate lists the semantic failure {class} in \
advance_on — falling to another model there hides the defect (D11)"
                );
            }

            // And the positive half stays what it is for: only failures that
            // happened BEFORE the model got to be wrong may move a step, so
            // no class is ever published on both sides at once.
            for class in &advance {
                assert!(
                    !never.contains(class),
                    "{kind}: {class} is published as both an advance and a never-advance"
                );
            }

            // The gate travels WITH the chain (the `fallback_when` precedent,
            // af17e217): a caller that reads the chain has read the condition.
            assert_eq!(
                chain.get("fallback_when").and_then(Value::as_str),
                Some(CHAIN_FALLBACK_WHEN),
                "{kind}: the chain published without its condition"
            );
        }
    }

    /// D10 preserving 4faf1de9: the advisor has no fallback by design — the
    /// consult that hit a quota wall was recorded NOT OBTAINED rather than
    /// substituted. A chain configured for other roles must not reach it; only
    /// a chain the operator keys to the advisor itself does.
    #[test]
    fn the_advisor_carries_no_chain_unless_one_is_configured_for_it() {
        let t = tempfile::tempdir().unwrap();
        let others =
            chain_repo(&t, r#""retry":{"fallbackChains":{"code":["opus"],"review":["sonnet"]}}"#);
        assert_eq!(chain_payload(&others, "advisor").get("fallback_chain"), None);

        let t2 = tempfile::tempdir().unwrap();
        let by_role = chain_repo(&t2, r#""retry":{"fallbackChains":{"advisor":["opus"]}}"#);
        assert_eq!(
            chain_payload(&by_role, "advisor").get("fallback_chain").unwrap().get("key"),
            Some(&json!("advisor"))
        );

        let t3 = tempfile::tempdir().unwrap();
        let by_model = chain_repo(&t3, r#""retry":{"fallbackChains":{"fable":["opus"]}}"#);
        assert_eq!(
            chain_payload(&by_model, "advisor").get("fallback_chain").unwrap().get("key"),
            Some(&json!("fable"))
        );
    }

    /// A dispatch with no model of its own has nothing to fall FROM: an
    /// escalated cell runs the session model with no `model` parameter, and a
    /// herding slot runs a pane. Publishing a list of model selectors beside
    /// a payload that names no model would be advice none of them could take.
    #[test]
    fn a_dispatch_that_names_no_model_carries_no_chain() {
        let t = tempfile::tempdir().unwrap();
        let root = chain_repo(&t, r#""retry":{"fallbackChains":{"code":["opus"],"ceiling":["opus"]}}"#);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","status":"claimed","role":"code","escalate":true,"trace":{"worker":"w"}}"#,
        );
        let escalated = chain_payload(&root, "cell");
        assert_eq!(escalated.get("model"), None);
        assert_eq!(escalated.get("fallback_chain"), None);

        let t2 = tempfile::tempdir().unwrap();
        let herding = repo(
            &t2,
            r#"{"models":{"claude":{"code":{"kind":"herding"}}},"retry":{"fallbackChains":{"code":["opus"]}}}"#,
        );
        w(&herding, ".bee/cells/c-1.json", CHAIN_CELL);
        assert_eq!(chain_payload(&herding, "cell").get("fallback_chain"), None);
    }

    /// Parsing and validation, at the door. Junk drops — loudly, on stderr —
    /// rather than reaching a payload half-formed, and `default` is refused
    /// outright: D10 ships no default chain and no role inherits one.
    #[test]
    fn fallback_chain_config_validation_drops_what_it_cannot_use() {
        let chains = normalize_fallback_chains(Some(&json!({
            "  code  ": ["  opus  ", "opus", "", "haiku"],
            "review": "opus",
            "read": [],
            "extraction": [null, 7],
            "sonnet": ["sonnet", "haiku"],
            "default": ["opus"],
            "   ": ["opus"],
        })));

        // trimmed key, trimmed steps, duplicates collapsed, order kept
        assert_eq!(chains.get("code"), Some(&json!(["opus", "haiku"])));
        // a step naming the key's own model is not a step
        assert_eq!(chains.get("sonnet"), Some(&json!(["haiku"])));
        for dropped in ["review", "read", "extraction", "default", "   ", ""] {
            assert_eq!(chains.get(dropped), None, "{dropped} should not have survived");
        }
        assert_eq!(chains.len(), 2);

        // Absent, non-object and empty all read as "no chains configured" —
        // the same answer, so nothing can mistake one for a partial default.
        assert!(normalize_fallback_chains(None).is_empty());
        assert!(normalize_fallback_chains(Some(&json!("chains"))).is_empty());
        assert!(normalize_fallback_chains(Some(&json!({}))).is_empty());
        assert!(resolve_fallback_chain(&Map::new(), "code", "sonnet").is_none());

        // A matched key whose steps clean away to nothing STOPS the walk: the
        // most specific key the operator wrote is the one that answers, and a
        // broader key never overrules it.
        let self_loop = normalize_fallback_chains(Some(&json!({
            "anthropic/*": ["anthropic/opus"],
            "code": ["haiku"],
        })));
        assert_eq!(resolve_fallback_chain(&self_loop, "code", "anthropic/opus"), None);

        // And the whole config path reads the same way `read_models` does.
        let t = tempfile::tempdir().unwrap();
        let root = chain_repo(&t, r#""retry":{"fallbackChains":{"code":["opus"]}}"#);
        assert_eq!(read_fallback_chains(&root).get("code"), Some(&json!(["opus"])));
        let t2 = tempfile::tempdir().unwrap();
        assert!(read_fallback_chains(&chain_repo(&t2, "")).is_empty());
    }

    // ── slp-blind-lanes: the LaneBrief carrier (E1) and its digest (E2) ────

    /// A host that can resolve EVERY runtime's advisor slot, so the brief
    /// probes below can walk `DISPATCH_RUNTIMES` without a config detour.
    ///
    /// pi-support D5/D6: the pi table is herding-shaped throughout, because
    /// that is the only shape the pi door serves — a plain-string `models.pi`
    /// slot is refused by `pi_requires_herding_refusal`, so a fixture copying
    /// the claude table's shape here would prove nothing about the brief and
    /// only re-prove the refusal. Its payload is the herding-exec Bash call,
    /// which carries the rendered prompt on `stdin` (`dispatched_body` already
    /// reads that key), so every body assertion below reads the pi row the
    /// same way it reads the other two.
    const BRIEF_HOST: &str = r#"{"models":{"claude":{"generation":"sonnet","code":"sonnet","review":"opus","advisor":"opus"},"codex":{"generation":"gpt-5","code":"gpt-5","review":"gpt-5","advisor":"gpt-5-pro"},"pi":{"generation":{"kind":"herding","agent":"pi-agy-flash"},"code":{"kind":"herding","agent":"pi-opus"},"review":{"kind":"herding","agent":"pi-opus"},"advisor":{"kind":"herding","agent":"pi-fable"}}}}"#;

    /// The advisor prompt EXACTLY as it read before the `{{#if brief}}` block
    /// was added, pinned as literal bytes rather than re-derived from the
    /// template. Re-deriving it would compare the template against itself and
    /// pass whatever the block does; this fails the moment a falsy block
    /// leaves one byte of residue behind — the newline around `{{/if}}` being
    /// the byte that is easiest to get wrong.
    const ADVISOR_BODY_WITHOUT_A_BRIEF: &str = concat!(
        "Advisor consult: produce an independent digest/opinion on the given question. Read-only.\n",
        "\n",
        "Paths: <caller fills in the exact files/paths to read>\n",
        "\n",
        "Digest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked."
    );

    /// The rendered body a dispatch payload carries, whichever transport key
    /// this runtime used (`prompt` for an Agent, `message` for spawn_agent,
    /// `stdin` for a cli executor), with the `[bee-tier: …]` marker line
    /// stripped so what is compared is the TEMPLATE's bytes.
    fn dispatched_body(v: &Value) -> String {
        let payload = v.get("payload").expect("an envelope carries a payload");
        let raw = ["prompt", "message", "stdin"]
            .iter()
            .find_map(|k| payload.get(*k).and_then(Value::as_str))
            .expect("a payload carries its rendered prompt under prompt/message/stdin");
        match raw.strip_prefix("[bee-tier: ") {
            Some(rest) => rest
                .split_once('\n')
                .map(|(_, body)| body.to_string())
                .unwrap_or_else(|| raw.to_string()),
            None => raw.to_string(),
        }
    }

    fn brief_file(root: &Path, name: &str, body: &str) -> String {
        let file = root.join(name);
        std::fs::write(&file, body).unwrap();
        file.to_string_lossy().into_owned()
    }

    fn advisor_envelope(root: &Path, runtime: &str, brief: Option<&str>) -> Value {
        let Prepared::Value(v) = prepare_dispatch_with_brief(
            root, runtime, "advisor", None, None, None, false, None, None, false, None, brief,
        )
        .unwrap() else {
            panic!("expected an advisor envelope on {runtime}")
        };
        v
    }

    /// E1's integration edge: with no `--brief-file`, EVERY runtime × kind
    /// pair renders what it rendered before the block existed, and no
    /// envelope grows a `brief_sha256`. The non-cell kinds are compared
    /// against `render(template, &[])` — the literal expression the carrier
    /// replaced — so the walk pins the old behaviour, not the new code's
    /// opinion of it.
    #[test]
    fn no_brief_leaves_every_runtime_and_kind_pair_exactly_as_it_was() {
        for runtime in DISPATCH_RUNTIMES {
            for kind in DISPATCH_KINDS {
                let tmp = tempfile::tempdir().unwrap();
                let root = repo(&tmp, BRIEF_HOST);
                w(
                    &root,
                    ".bee/cells/c-1.json",
                    r#"{"id":"c-1","feature":"f","title":"carry a brief","role":"code","status":"claimed","trace":{"worker":"w"}}"#,
                );
                let (cell, worker) =
                    if kind == "cell" { (Some("c-1"), Some("w")) } else { (None, None) };
                let Prepared::Value(v) = prepare_dispatch_with_brief(
                    &root, runtime, kind, None, cell, worker, false, None, None, false, None, None,
                )
                .unwrap() else {
                    panic!("expected an envelope for {runtime}/{kind}")
                };
                assert_eq!(
                    v.get("brief_sha256"),
                    None,
                    "{runtime}/{kind}: a briefless dispatch must not stamp a digest"
                );
                let body = dispatched_body(&v);
                assert!(
                    !body.contains("Brief ("),
                    "{runtime}/{kind}: a briefless payload must carry no brief block"
                );
                if kind != "cell" {
                    let before = render(&load_prompt(kind).unwrap(), &[]).unwrap();
                    assert_eq!(body, before, "{runtime}/{kind}: payload bytes drifted");
                }
            }
        }
        // And the advisor template's own bytes, pinned literally.
        assert_eq!(
            render(&load_prompt("advisor").unwrap(), &[]).unwrap(),
            ADVISOR_BODY_WITHOUT_A_BRIEF
        );
        assert_eq!(
            render(&load_prompt("advisor").unwrap(), &[("brief", "")]).unwrap(),
            ADVISOR_BODY_WITHOUT_A_BRIEF,
            "an empty brief var must drop the block WITH its own leading newline"
        );
    }

    /// A brief that is nothing but whitespace must reach the renderer as NO
    /// brief. `{{#if}}` truthiness is `!v.is_empty()`, so an untrimmed
    /// whitespace brief would splice an EMPTY block in place of today's
    /// bytes — the one way a "no brief" call could stop being byte-identical.
    #[test]
    fn a_whitespace_only_brief_is_no_brief_at_all() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        for body in ["", "   ", "\n\n", " \t \r\n  \n"] {
            let path = brief_file(&root, "brief.md", body);
            assert_eq!(
                resolve_brief_file("advisor", Some(&path)).unwrap(),
                None,
                "a brief of {body:?} must resolve to no brief"
            );
        }
        let path = brief_file(&root, "ws.md", "  \n \t\n");
        let carried = resolve_brief_file("advisor", Some(&path)).unwrap();
        let v = advisor_envelope(&root, "claude", carried.as_deref());
        assert_eq!(dispatched_body(&v), ADVISOR_BODY_WITHOUT_A_BRIEF);
        assert_eq!(v.get("brief_sha256"), None, "no brief travelled, so no digest is stamped");
    }

    /// The carrier itself: the brief's bytes reach the advisor body ONCE,
    /// the block leaves no marker residue, and the untouched prose above it
    /// is still there.
    #[test]
    fn a_carried_brief_renders_into_the_advisor_body_exactly_once() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        // Leading and trailing whitespace on purpose: the carrier trims, so
        // what renders is the trimmed text and the assertion can be exact.
        let question = "## Question\n\nWhich door enforces the read diet?\n\n## Constraints\n\n- no new store";
        let path = brief_file(&root, "brief.md", &format!("\n{question}\n\n"));
        let carried = resolve_brief_file("advisor", Some(&path)).unwrap();
        assert_eq!(carried.as_deref(), Some(question), "the carried brief is the trimmed file");

        for runtime in DISPATCH_RUNTIMES {
            let v = advisor_envelope(&root, runtime, carried.as_deref());
            let body = dispatched_body(&v);
            assert_eq!(
                body.matches(question).count(),
                1,
                "{runtime}: the brief must render exactly once, got {body}"
            );
            assert!(!body.contains("{{"), "{runtime}: unrendered marker left behind: {body}");
            assert!(
                body.starts_with(ADVISOR_BODY_WITHOUT_A_BRIEF),
                "{runtime}: the standing advisor prose must survive above the brief"
            );
            assert!(body.contains("Brief (the question to answer"), "{runtime}: {body}");
        }
    }

    /// The size guard. Over the cap is a TYPED refusal that names the cap, in
    /// the same `{ok:false, type:"refused", reason, …, fix}` shape every
    /// other refusal at this door takes; exactly at the cap still passes, so
    /// the boundary is a decision rather than an off-by-one.
    #[test]
    fn a_brief_over_the_cap_is_refused_by_a_typed_refusal_naming_the_cap() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);

        let at_cap = brief_file(&root, "at-cap.md", &"x".repeat(BRIEF_MAX_BYTES));
        assert_eq!(
            resolve_brief_file("advisor", Some(&at_cap)).unwrap().map(|b| b.len()),
            Some(BRIEF_MAX_BYTES),
            "a brief of exactly {BRIEF_MAX_BYTES} bytes is carried, not refused"
        );

        let over = brief_file(&root, "over.md", &"x".repeat(BRIEF_MAX_BYTES + 1));
        let refusal = resolve_brief_file("advisor", Some(&over)).unwrap_err();
        assert_eq!(refusal.get("ok"), Some(&json!(false)));
        assert_eq!(refusal.get("type"), Some(&json!("refused")));
        assert_eq!(refusal.get("reason"), Some(&json!("brief_too_large")));
        assert_eq!(refusal.get("bytes"), Some(&json!(BRIEF_MAX_BYTES + 1)));
        assert_eq!(refusal.get("max_bytes"), Some(&json!(BRIEF_MAX_BYTES)));
        let fix = refusal.get("fix").and_then(Value::as_str).unwrap_or_default();
        assert!(
            fix.contains(&BRIEF_MAX_BYTES.to_string()),
            "the fix must name the cap in bytes: {fix}"
        );

        // A path that names nothing is refused the same way — never a silent
        // "no brief", which would fire a lane on an empty question.
        let missing = root.join("nope.md").to_string_lossy().into_owned();
        let refusal = resolve_brief_file("advisor", Some(&missing)).unwrap_err();
        assert_eq!(refusal.get("reason"), Some(&json!("brief_file_unreadable")));
        assert_eq!(refusal.get("type"), Some(&json!("refused")));

        // Invalid UTF-8 is refused rather than decoded lossily: a lossy
        // decode would change the very bytes brief_sha256 exists to pin.
        let bad = root.join("bad.md");
        std::fs::write(&bad, [0x51u8, 0xff, 0xfe, 0x51]).unwrap();
        let refusal = resolve_brief_file("advisor", Some(&bad.to_string_lossy())).unwrap_err();
        assert_eq!(refusal.get("reason"), Some(&json!("brief_not_utf8")));
    }

    /// D3 plus the silent-swallow window it leaves open. `--kind cell` is
    /// refused because the worker-cell prompt injects shared learned_context;
    /// `gather` and `reviewer` are refused because their templates carry no
    /// brief block at all, so accepting a brief would swallow it with nothing
    /// to show for it.
    #[test]
    fn a_brief_is_refused_for_every_kind_but_advisor() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        let path = brief_file(&root, "brief.md", "## Question\n\nWhat?");
        for kind in DISPATCH_KINDS {
            let outcome = resolve_brief_file(kind, Some(&path));
            if kind == "advisor" {
                assert!(outcome.is_ok(), "advisor must accept a brief");
                continue;
            }
            let refusal = outcome.unwrap_err();
            assert_eq!(refusal.get("ok"), Some(&json!(false)), "{kind}");
            assert_eq!(refusal.get("type"), Some(&json!("refused")), "{kind}");
            assert_eq!(
                refusal.get("reason"),
                Some(&json!("brief_kind_not_advisor")),
                "{kind}: a brief must never be swallowed silently"
            );
            assert_eq!(refusal.get("kind"), Some(&json!(kind)), "{kind}");
            let fix = refusal.get("fix").and_then(Value::as_str).unwrap_or_default();
            assert!(fix.contains("--kind advisor"), "{kind}: the fix must name the way out: {fix}");
        }
        // The refusal fires before the file is even looked at, so a bad path
        // on a wrong kind still says which mistake to fix first.
        let refusal = resolve_brief_file("cell", Some("no/such/file.md")).unwrap_err();
        assert_eq!(refusal.get("reason"), Some(&json!("brief_kind_not_advisor")));
    }

    /// E2 (D2b): byte-identity across lanes is CHECKABLE. Two prepares over
    /// one unchanged file stamp one digest; an edited file stamps another;
    /// and the digest is the sha256 of the bytes the lane actually received.
    #[test]
    fn one_brief_file_digests_the_same_on_every_lane_and_an_edit_moves_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        let question = "## Question\n\nWhere does the dossier live?";
        let path = brief_file(&root, "brief.md", question);

        let carried = resolve_brief_file("advisor", Some(&path)).unwrap();
        let lane_a = advisor_envelope(&root, "claude", carried.as_deref());
        let carried_again = resolve_brief_file("advisor", Some(&path)).unwrap();
        let lane_b = advisor_envelope(&root, "codex", carried_again.as_deref());

        let digest_a = lane_a.get("brief_sha256").and_then(Value::as_str).unwrap().to_string();
        let digest_b = lane_b.get("brief_sha256").and_then(Value::as_str).unwrap();
        assert_eq!(digest_a, digest_b, "one unchanged brief file is one digest");
        assert_eq!(
            digest_a,
            crate::verbs::reservations::sha256_hex(question),
            "the digest is over the bytes the lane received"
        );
        assert_ne!(
            lane_a.get("dispatch_id"),
            lane_b.get("dispatch_id"),
            "two lanes are two dispatches — only the BRIEF is shared"
        );

        // The same digest reaches the prepare-time record, which is what a
        // later convergence check reads back by dispatch_id.
        assert!(
            std::fs::read_to_string(root.join(".bee/logs/dispatch.jsonl"))
                .unwrap_or_default()
                .is_empty(),
            "record_it=false must leave no log line behind"
        );
        let Prepared::Value(recorded) = prepare_dispatch_with_brief(
            &root,
            "claude",
            "advisor",
            None,
            None,
            None,
            false,
            None,
            None,
            true,
            None,
            carried.as_deref(),
        )
        .unwrap() else {
            panic!("expected an advisor envelope")
        };
        let logged = std::fs::read_to_string(root.join(".bee/logs/dispatch.jsonl")).unwrap();
        let line: Value = serde_json::from_str(logged.trim()).unwrap();
        assert_eq!(line.get("brief_sha256"), Some(&json!(digest_a)));
        assert_eq!(line.get("dispatch_id"), recorded.get("dispatch_id"));

        // An EDIT of the same file moves the digest.
        let edited = brief_file(&root, "brief.md", "## Question\n\nWhere does the DOSSIER live?");
        let carried_edit = resolve_brief_file("advisor", Some(&edited)).unwrap();
        let lane_c = advisor_envelope(&root, "claude", carried_edit.as_deref());
        assert_ne!(
            lane_c.get("brief_sha256").and_then(Value::as_str),
            Some(digest_a.as_str()),
            "an edited brief must report a different digest"
        );
    }

    /// The prompt edit's OWN hazard, from the plan's risk map: a source-only
    /// prompt edit skews the embedded template from disk, `dispatch prepare`
    /// returns None, and the Node delegate it would fall to was deleted at
    /// the R6 cutover — so every dispatch in this checkout would break. This
    /// asserts the vendored copy travelled with the source one.
    #[test]
    fn the_advisor_template_still_matches_disk_after_the_brief_block() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("..");
        assert!(
            prompts_match_disk(&repo_root, "advisor"),
            "packages/bee/prompts/advisor.md or .bee/bin/prompts/advisor.md drifted from the \
             compiled-in template — run `bee dev regen` and rebuild, or every dispatch prepare \
             in this checkout resolves to nothing"
        );
        // The block the carrier depends on is really in the shipped bytes,
        // and really not at byte 0 (the renderer needs a leading newline).
        let template = load_prompt("advisor").unwrap();
        let at = template.find("{{#if brief}}").expect("advisor.md carries a brief block");
        assert!(at > 0, "a block at byte 0 has no leading newline for the renderer to consume");
        assert_eq!(&template[at - 1..at], "\n");
        assert!(template.contains("{{brief}}"), "the block must hold the brief placeholder");
    }

    /// The two ends of the flag, which live in different files and are the
    /// pair that historically drifts: `run_dispatch_prepare`'s own
    /// known-key list (an unlisted key makes the whole command refuse as an
    /// unproven shape) and the embedded registry payload (an undeclared key
    /// is rejected by the argv-shape guard before the handler ever runs). A
    /// flag declared in one and not the other is a flag that is either
    /// swallowed or unreachable — the "unknown flag is a refusal, never a
    /// shrug" pattern, checked here because `dispatch prepare` itself cannot
    /// be invoked from inside a granted worktree.
    ///
    /// Derived from the sources, never hand-copied: the same `include_str!`
    /// + scan discipline `tests/registry_contracts.rs` uses for `state gate`.
    #[test]
    fn brief_file_is_both_accepted_by_the_handler_and_declared_in_the_registry() {
        const PREPARE_SOURCE: &str = include_str!("prepare.rs");
        let anchor = "pub(crate) fn run_dispatch_prepare(";
        let at = PREPARE_SOURCE.find(anchor).expect("run_dispatch_prepare moved — re-anchor this");
        let after = &PREPARE_SOURCE[at..];
        let kk = after.find("keys_known(").expect("run_dispatch_prepare has a keys_known call");
        let rest = &after[at_open(&after[kk..]) + kk..];
        let close = rest.find(']').expect("the keys_known array literal closes");
        let listed: Vec<&str> = rest[..close]
            .split(',')
            .filter_map(|e| e.trim().strip_prefix('"')?.strip_suffix('"'))
            .collect();
        // Not vacuous: the parse must still see the flags that were always
        // there, or a silently-empty scan would pass on anything.
        for known in ["runtime", "kind", "cell", "worker", "purpose", "expertise", "role"] {
            assert!(listed.contains(&known), "the keys_known parse lost {known:?}: {listed:?}");
        }
        assert!(
            listed.contains(&"brief-file"),
            "run_dispatch_prepare does not accept --brief-file, so the whole command would \
             refuse as an unproven shape: {listed:?}"
        );

        let payload: Value =
            serde_json::from_str(include_str!("../../generated/registry_payload.json")).unwrap();
        let entry = payload["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == json!("dispatch.prepare"))
            .expect("the registry declares dispatch.prepare");
        let props = entry["parameters"]["properties"].as_object().unwrap();
        assert!(
            props.contains_key("brief-file"),
            "the registry payload does not declare --brief-file, so the argv-shape guard \
             rejects it before the handler runs"
        );
        assert_eq!(props["brief-file"]["type"], json!("string"));
        let described = props["brief-file"]["description"].as_str().unwrap_or_default();
        assert!(
            described.contains("8192") && described.contains("advisor"),
            "the declaration must name the cap and the one kind that accepts it: {described}"
        );
    }

    /// The offset of the `[` that opens a `keys_known(...)` array literal.
    fn at_open(from_keys_known: &str) -> usize {
        from_keys_known.find('[').expect("keys_known(...) opens an array literal") + 1
    }

    // ── slp-blind-lanes: the LaneBrief leaning guard (E1, D2a) ─────────────

    /// A brief with the four required sections, so a probe can vary ONE thing
    /// (the Question body, or one added line) and know the shape arm is not
    /// what fired.
    fn shaped_brief(question: &str) -> String {
        format!(
            "## Question\n\n{question}\n\n\
             ## Constraints\n\nThe answer stays inside the existing dispatch door.\n\n\
             ## Read diet\n\npackages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs\n\n\
             ## Digest contract\n\nReturn the paths read and the facts with file:line anchors.\n"
        )
    }

    fn refusal_of(brief: &str) -> Value {
        lint_brief(brief).expect_err("this brief must be refused")
    }

    /// ARM 1. Every one of the seventeen FROZEN verdict stems is refused, the
    /// refusal names the phrase that fired, and the fix says LEANING LANGUAGE
    /// was refused — never that neutrality was certified, which a word list
    /// cannot do.
    #[test]
    fn every_frozen_verdict_stem_is_refused_by_name() {
        assert_eq!(LEANING_VERDICT_STEMS.len(), 17, "the frozen list is seventeen stems");
        for stem in LEANING_VERDICT_STEMS {
            // In the CONSTRAINTS section, so only the stem arm can fire.
            let brief = shaped_brief("Where does the dossier live?")
                .replace("The answer stays inside the existing dispatch door.", &format!("Note: {stem} the second option."));
            let refusal = refusal_of(&brief);
            assert_eq!(refusal.get("ok"), Some(&json!(false)), "{stem}: {refusal}");
            assert_eq!(refusal.get("type"), Some(&json!("refused")), "{stem}: {refusal}");
            assert_eq!(
                refusal.get("reason"),
                Some(&json!("brief_leaning_language")),
                "{stem}: {refusal}"
            );
            assert_eq!(refusal.get("phrase"), Some(&json!(stem)), "the refusal names the phrase that fired: {refusal}");
            let fix = refusal.get("fix").and_then(Value::as_str).unwrap_or_default();
            assert!(fix.contains(stem), "the fix quotes the phrase to cut: {fix}");
            assert!(
                fix.to_ascii_lowercase().contains("leaning language"),
                "the refusal says leaning language was refused: {fix}"
            );
            assert!(
                !fix.to_ascii_lowercase().contains("neutral"),
                "no refusal text may use the word at all — a word list cannot certify neutrality, \
                 and even a disclaimer beside it trains the reader to expect the claim: {fix}"
            );
        }

        // Case-folded and word-bounded, both directions.
        assert!(lint_brief(&shaped_brief("What is the plan? WE RECOMMEND nothing.")).is_err());
        assert!(
            lint_brief(&shaped_brief("Which reader wins when the preference set is empty?")).is_ok(),
            "\"preference\" is not \"i prefer\" — the scan is word-bounded"
        );
        // "leaning toward" must not swallow "leaning towards": both are on the
        // list, and the longer one reports ITSELF.
        let towards = refusal_of(&shaped_brief("The draft was leaning towards the cache."));
        assert_eq!(towards.get("phrase"), Some(&json!("leaning towards")), "{towards}");

        // The two stems cut at the re-consult stay cut: neutral interrogative
        // phrasing must survive the door.
        for kept in [
            "What is the right approach for the dossier?",
            "What is the right answer when two lanes disagree?",
        ] {
            assert!(lint_brief(&shaped_brief(kept)).is_ok(), "{kept} must not fire");
        }
    }

    /// ARM 1, honestly: a stem inside an ORDINARY fenced code sample refuses
    /// too. Being fenced is not the exception — only the designated tag below
    /// is, and everything else scans as it always did.
    #[test]
    fn a_verdict_stem_inside_a_fenced_code_sample_still_refuses() {
        let brief = shaped_brief("Where does the dossier live?").replace(
            "The answer stays inside the existing dispatch door.",
            "```text\n// we should use the second reader\n```",
        );
        let refusal = refusal_of(&brief);
        assert_eq!(refusal.get("reason"), Some(&json!("brief_leaning_language")), "{refusal}");
        assert_eq!(refusal.get("phrase"), Some(&json!("we should use")), "{refusal}");
    }

    // ── the tagged fence: the cross-critique round's ONE bounded exception ──
    //
    // D2(c) asks that round two hand each lane the rival proposal VERBATIM,
    // and a real proposal carries verdict stems, its own headings and its own
    // bullet lists. Only a fence whose opening info string is the designated
    // tag is skipped, and it is skipped by ALL THREE scans. Every other fence
    // scans exactly as the test above proves it does today.

    /// The tag spelled as a LITERAL, not read off the constant: a test that
    /// reads the constant it checks follows a silent rename, and this string
    /// is what a brief author types by hand.
    const QUOTED_FENCE_TAG_LITERAL: &str = "lane-proposal";

    /// SCAN 1 — the verdict-stem arm. The same bytes pass inside a tagged
    /// fence, refuse inside an untagged one, and refuse outside any fence.
    #[test]
    fn a_verdict_stem_inside_a_tagged_fence_no_longer_refuses() {
        let quoted = format!(
            "The rival lane returned this:\n\n```{QUOTED_FENCE_TAG_LITERAL}\n\
             we should use the second reader.\n```"
        );
        let ok = shaped_brief("Where does the dossier live?")
            .replace("The answer stays inside the existing dispatch door.", &quoted);
        assert!(lint_brief(&ok).is_ok(), "a tagged fence may carry the rival proposal: {ok}");

        // The IDENTICAL bytes, untagged, still refuse: the exception is the
        // tag, never the fence.
        let untagged = ok.replace(&format!("```{QUOTED_FENCE_TAG_LITERAL}"), "```text");
        let r = refusal_of(&untagged);
        assert_eq!(r.get("reason"), Some(&json!("brief_leaning_language")), "{r}");
        assert_eq!(r.get("phrase"), Some(&json!("we should use")), "{r}");

        // And outside any fence, exactly as before.
        let bare = shaped_brief("Where does the dossier live?").replace(
            "The answer stays inside the existing dispatch door.",
            "we should use the second reader.",
        );
        assert_eq!(refusal_of(&bare).get("phrase"), Some(&json!("we should use")));
    }

    /// SCAN 2 — the four-section shape arm. A quoted proposal carries its own
    /// headings; inside the tag they are prose, outside it they are sections.
    #[test]
    fn a_heading_inside_a_tagged_fence_does_not_become_a_section() {
        let quoted = format!(
            "```{QUOTED_FENCE_TAG_LITERAL}\n## Recommendation\n\nTake the second reader.\n```"
        );
        let ok = shaped_brief("Where does the dossier live?")
            .replace("The answer stays inside the existing dispatch door.", &quoted);
        assert!(lint_brief(&ok).is_ok(), "a quoted heading is not a section: {ok}");

        let untagged = ok.replace(&format!("```{QUOTED_FENCE_TAG_LITERAL}"), "```text");
        let r = refusal_of(&untagged);
        assert_eq!(r.get("reason"), Some(&json!("brief_section_unexpected")), "{r}");
        assert_eq!(r.get("section"), Some(&json!("Recommendation")), "{r}");
    }

    /// SCAN 3 — the Question-enumeration arm. A quoted proposal carries its
    /// own bullet list, and the lane did that enumerating, not the brief.
    #[test]
    fn a_bullet_list_inside_a_tagged_fence_under_question_is_not_answer_enumeration() {
        let quoted = format!(
            "Lane A returned this:\n\n```{QUOTED_FENCE_TAG_LITERAL}\n\
             ## Constraints\n\n- keep it in docs/\n- or beside the plan\n```"
        );
        let ok = shaped_brief(&quoted);
        assert!(lint_brief(&ok).is_ok(), "a quoted list is the lane's, not the brief's: {ok}");

        let untagged = ok.replace(&format!("```{QUOTED_FENCE_TAG_LITERAL}"), "```text");
        let r = refusal_of(&untagged);
        assert_eq!(r.get("reason"), Some(&json!("brief_section_unexpected")), "{r}");

        // The tagged fence's own "## Constraints" must not close the Question
        // section either: a bullet AFTER the fence, still under Question,
        // still refuses. Were the quoted heading counted, the shape arm would
        // have fired first and this reason would read differently.
        let leaked = format!("{quoted}\n\n- and one the brief added itself");
        let r = refusal_of(&shaped_brief(&leaked));
        assert_eq!(r.get("reason"), Some(&json!("brief_question_enumerates_answers")), "{r}");
        assert_eq!(r.get("section"), Some(&json!("Question")), "{r}");
    }

    /// An opener with no closer would hide every following line from every
    /// scan. It refuses BY NAME instead — tagged or not — so the guard never
    /// skips material in silence.
    #[test]
    fn a_fence_left_open_in_a_brief_is_refused_by_name() {
        for opener in [format!("```{QUOTED_FENCE_TAG_LITERAL}"), "```text".to_string()] {
            let brief = shaped_brief("Where does the dossier live?").replace(
                "The answer stays inside the existing dispatch door.",
                &format!("{opener}\nthe rival proposal, with no closing fence"),
            );
            let r = refusal_of(&brief);
            assert_eq!(r.get("reason"), Some(&json!("brief_fence_unclosed")), "{opener}: {r}");
            assert_eq!(r.get("line"), Some(&json!(opener)), "the refusal names the opener: {r}");
            let fix = r.get("fix").and_then(Value::as_str).unwrap_or_default();
            assert!(fix.contains("close"), "the fix says what to do: {fix}");
        }
    }

    /// ONE fence implementation, two callers. A rule that lives in two places
    /// drifts the first time one side is tuned — the same argument the stem
    /// list's own both-doors test makes.
    #[test]
    fn one_fence_implementation_serves_the_guard_and_the_dossier() {
        const GUARD_SOURCE: &str = include_str!("brief_lint.rs");
        const BLIND_SOURCE: &str = include_str!("../blind/mod.rs");

        assert!(
            GUARD_SOURCE.contains("pub(crate) fn scan_fences"),
            "the one fence scanner lives in verbs/drivers/brief_lint.rs"
        );
        assert!(
            BLIND_SOURCE.contains("fence_mask(&lines)"),
            "the dossier parser calls the shared scanner"
        );
        assert!(
            !BLIND_SOURCE.contains("fn fence_mask"),
            "verbs/blind/mod.rs defines a SECOND fence scanner — the rule lives in \
             verbs/drivers/brief_lint.rs alone"
        );
        assert_eq!(
            QUOTED_PROPOSAL_FENCE_TAG, QUOTED_FENCE_TAG_LITERAL,
            "the tag a brief author types by hand is the tag the guard reads"
        );
    }

    /// ARM 2, the arm that carries the real load: leaning is mostly
    /// structural. A missing, extra or misordered section refuses BY NAME of
    /// the offending section.
    #[test]
    fn a_brief_missing_adding_or_misordering_a_section_is_refused_by_that_sections_name() {
        let good = shaped_brief("Where does the dossier live?");
        assert!(lint_brief(&good).is_ok(), "the shaped brief is the control: {good}");

        // MISSING — the name of the section that is not there.
        let missing = good.replace("## Read diet\n\npackages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs\n\n", "");
        let r = refusal_of(&missing);
        assert_eq!(r.get("reason"), Some(&json!("brief_section_missing")), "{r}");
        assert_eq!(r.get("section"), Some(&json!("Read diet")), "{r}");
        assert!(r.get("fix").and_then(Value::as_str).unwrap_or_default().contains("Read diet"));

        // EXTRA — a section the contract does not name.
        let extra = format!("{good}\n## Recommendation\n\nUse the second reader.\n");
        let r = refusal_of(&extra);
        assert_eq!(r.get("reason"), Some(&json!("brief_section_unexpected")), "{r}");
        assert_eq!(r.get("section"), Some(&json!("Recommendation")), "{r}");

        // REPEATED — a second copy of a required section is also extra.
        let repeated = format!("{good}\n## Constraints\n\nAnd one more.\n");
        let r = refusal_of(&repeated);
        assert_eq!(r.get("reason"), Some(&json!("brief_section_unexpected")), "{r}");
        assert_eq!(r.get("section"), Some(&json!("Constraints")), "{r}");

        // MISORDERED — all four present, wrong order, named by the heading
        // found where another was due.
        let misordered = "## Question\n\nWhere does the dossier live?\n\n\
                          ## Read diet\n\nprepare.rs\n\n\
                          ## Constraints\n\nOne door.\n\n\
                          ## Digest contract\n\nPaths read.\n";
        let r = refusal_of(misordered);
        assert_eq!(r.get("reason"), Some(&json!("brief_section_out_of_order")), "{r}");
        assert_eq!(r.get("section"), Some(&json!("Read diet")), "{r}");
        assert!(
            r.get("fix").and_then(Value::as_str).unwrap_or_default().contains("Constraints"),
            "the fix names the section that was due: {r}"
        );

        // A brief with no headings at all is missing the FIRST section.
        let r = refusal_of("Where does the dossier live?");
        assert_eq!(r.get("reason"), Some(&json!("brief_section_missing")), "{r}");
        assert_eq!(r.get("section"), Some(&json!("Question")), "{r}");

        // Headings match case-insensitively on the trimmed text, and a level-3
        // heading inside a section is not a section.
        let folded = "##  QUESTION \n\nWhere?\n\n### A sub-heading\n\nprose\n\n\
                      ## constraints\n\nOne door.\n\n## READ DIET\n\nprepare.rs\n\n\
                      ## Digest Contract\n\nPaths read.\n";
        assert!(lint_brief(folded).is_ok(), "sections fold case and ignore level-3 headings");
    }

    /// ARM 2, the enumeration rule: a brief that LISTS candidate answers has
    /// already led the witness — lanes exist to generate the options.
    #[test]
    fn an_enumerated_list_under_question_is_refused_naming_that_section() {
        for line in ["- keep it in docs/", "* keep it in docs/", "1. keep it in docs/", "  2.  keep it in docs/"] {
            let brief = shaped_brief(&format!("Where does the dossier live?\n\n{line}\n- or beside the plan"));
            let r = refusal_of(&brief);
            assert_eq!(r.get("reason"), Some(&json!("brief_question_enumerates_answers")), "{line}: {r}");
            assert_eq!(r.get("section"), Some(&json!("Question")), "{line}: {r}");
            let fix = r.get("fix").and_then(Value::as_str).unwrap_or_default();
            assert!(fix.contains("Question"), "the fix names the offending section: {fix}");
        }
        // The rule is scoped to Question: the other sections may enumerate.
        let diet = shaped_brief("Where does the dossier live?")
            .replace("packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "- prepare.rs\n- prompt.rs");
        assert!(lint_brief(&diet).is_ok(), "a read diet is a list by nature: {diet}");
    }

    /// THE SCOPE PROPERTY, and the highest-risk property of this feature: the
    /// guard reads the BRIEF and nothing else. A false fire on `--purpose` or
    /// `--expertise` would refuse the advisor consult Gate 3 itself requires
    /// (`high_risk_advisor_refusal`), deadlocking the very workflow that
    /// approves guards. So: every frozen stem, in both of those fields, with
    /// no `--brief-file` at all, must prepare cleanly.
    #[test]
    fn a_dispatch_carrying_every_stem_in_purpose_and_expertise_prepares_cleanly() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);

        let purpose = format!("Judge the shape — {}.", LEANING_VERDICT_STEMS.join(", "));
        // `--expertise` MUST be newline-delimited `<path> :: <purpose> :: <read-to>`
        // or parse_expertise refuses on PARSE before any guard is reached.
        let expertise_raw: String = LEANING_VERDICT_STEMS
            .iter()
            .enumerate()
            .map(|(i, stem)| format!("docs/note-{i}.md :: {stem} the first option :: see why {stem}"))
            .collect::<Vec<_>>()
            .join("\n");
        let entries = parse_expertise(&expertise_raw).expect("the fixture is well-formed expertise");
        assert_eq!(entries.len(), LEANING_VERDICT_STEMS.len());
        let block = entries
            .iter()
            .map(|e| format!("- {} — {}. Read it to {}.", e.path, e.purpose, e.read_to))
            .collect::<Vec<_>>()
            .join("\n");

        // NOT VACUOUS: this text WOULD fire, if the guard were ever handed it.
        assert!(first_leaning_stem(&purpose).is_some(), "the purpose fixture carries real stems");
        assert!(first_leaning_stem(&block).is_some(), "the expertise fixture carries real stems");

        // No `--brief-file`: the brief door resolves to nothing at all.
        assert_eq!(resolve_brief_file("advisor", None).unwrap(), None);

        for runtime in DISPATCH_RUNTIMES {
            let Prepared::Value(v) = prepare_dispatch_with_brief(
                &root,
                runtime,
                "advisor",
                None,
                None,
                None,
                false,
                None,
                Some(&purpose),
                false,
                Some(&block),
                None,
            )
            .unwrap() else {
                panic!("{runtime}: a stem-carrying purpose/expertise must still prepare")
            };
            assert!(v.get("payload").is_some(), "{runtime}: {v}");
            assert_eq!(v.get("brief_sha256"), None, "{runtime}: no brief travelled");
        }
    }

    /// The corpus, scoped to the VERDICT-STEM ARM ONLY. The shape arm is
    /// excluded ON PURPOSE: no prompt file carries the four required sections,
    /// so a whole-guard corpus would fire on every one of them and force a
    /// quiet re-scoping of the guard. That exclusion is what stops the corpus
    /// and the stem list being co-tuned into agreement. A real fire here is
    /// EVIDENCE about the list — the fix is a recorded reason, never a silent
    /// trim.
    #[test]
    fn the_verdict_stem_arm_fires_zero_times_over_every_shipped_prompt() {
        let prompts = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("..")
            .join("packages")
            .join("bee")
            .join("prompts");
        let mut checked = 0usize;
        for entry in std::fs::read_dir(&prompts).expect("packages/bee/prompts exists") {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            assert_eq!(
                first_leaning_stem(&text),
                None,
                "{}: a shipped prompt fires the verdict-stem arm — that is evidence about the \
                 list, and the fix is a recorded reason, never a silent trim",
                path.display()
            );
            checked += 1;
        }
        assert!(checked >= 4, "the corpus walk found only {checked} prompt files — it went vacuous");
    }

    // ── the door calls the guard (one rule, two places, one test) ──────────

    const LEANING_BRIEF_CHILD: &str = "verbs::drivers::tests::dispatch_prepare_leaning_brief_child";
    const SHAPED_BRIEF_CHILD: &str = "verbs::drivers::tests::dispatch_prepare_shaped_brief_child";

    /// Runs ONLY as a child of the test below: `run_dispatch_prepare` reads
    /// the root off `std::env::current_dir()` and prints to the process's own
    /// stdout, both process-global — the same isolation the `--claim` children
    /// above use.
    #[test]
    #[ignore = "spawned by the_dispatch_door_refuses_a_leaning_brief_and_carries_a_shaped_one"]
    fn dispatch_prepare_leaning_brief_child() {
        let (flags, use_json) = parse_flags(&[
            "--runtime", "claude", "--kind", "advisor", "--brief-file", "leaning.md", "--json",
        ])
        .expect("well-formed fixture argv");
        run_dispatch_prepare(flags, use_json, Instant::now());
    }

    /// Runs ONLY as a child of the test below.
    #[test]
    #[ignore = "spawned by the_dispatch_door_refuses_a_leaning_brief_and_carries_a_shaped_one"]
    fn dispatch_prepare_shaped_brief_child() {
        let (flags, use_json) = parse_flags(&[
            "--runtime", "claude", "--kind", "advisor", "--brief-file", "shaped.md", "--json",
        ])
        .expect("well-formed fixture argv");
        run_dispatch_prepare(flags, use_json, Instant::now());
    }

    fn child_payload(root: &Path, name: &str) -> Value {
        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = Command::new(&exe);
        cmd.args(["--exact", name, "--ignored", "--test-threads", "1", "--nocapture"]);
        cmd.current_dir(root);
        let out = cmd.output().expect("spawn the test binary");
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let start = stdout.find('{').unwrap_or_else(|| panic!("no JSON payload in child stdout:\n{stdout}"));
        let end = stdout.rfind('}').map(|i| i + 1).unwrap_or_else(|| panic!("no JSON payload in child stdout:\n{stdout}"));
        serde_json::from_str(&stdout[start..end])
            .unwrap_or_else(|e| panic!("child stdout was not valid JSON ({e}):\n{stdout}"))
    }

    /// The rule lives in TWO places — the guard itself and the dispatch door
    /// that must run it — so ONE test reads both: the REAL CLI entry refuses a
    /// leaning brief with the guard's own typed refusal, and carries a shaped
    /// one through to a payload. Without the second half a door that refused
    /// everything would pass the first half.
    #[test]
    fn the_dispatch_door_refuses_a_leaning_brief_and_carries_a_shaped_one() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        let shaped = shaped_brief("Where does the dossier live?");
        std::fs::write(root.join("shaped.md"), &shaped).unwrap();
        std::fs::write(
            root.join("leaning.md"),
            shaped.replace(
                "The answer stays inside the existing dispatch door.",
                "I recommend the second reader.",
            ),
        )
        .unwrap();

        let refused = child_payload(&root, LEANING_BRIEF_CHILD);
        assert_eq!(refused.get("ok"), Some(&json!(false)), "{refused}");
        assert_eq!(refused.get("type"), Some(&json!("refused")), "{refused}");
        assert_eq!(refused.get("reason"), Some(&json!("brief_leaning_language")), "{refused}");
        assert_eq!(refused.get("phrase"), Some(&json!("i recommend")), "{refused}");
        assert!(refused.get("payload").is_none(), "a refused brief never reaches a payload: {refused}");

        let carried = child_payload(&root, SHAPED_BRIEF_CHILD);
        assert!(carried.get("payload").is_some(), "a shaped brief reaches a payload: {carried}");
        assert!(carried.get("ok").is_none(), "a prepared envelope carries no ok flag: {carried}");
        assert_eq!(
            carried.get("brief_sha256").and_then(Value::as_str),
            Some(crate::verbs::reservations::sha256_hex(shaped.trim_end()).as_str()),
            "the shaped brief travelled and stamped its digest: {carried}"
        );

        // And the door owns no second copy of the list: the stems live in
        // brief_lint.rs alone, so the two places cannot drift apart.
        const PREPARE_SOURCE: &str = include_str!("prepare.rs");
        assert!(
            PREPARE_SOURCE.contains("lint_brief("),
            "the dispatch door must call the shared guard"
        );
        for stem in LEANING_VERDICT_STEMS {
            assert!(
                !PREPARE_SOURCE.to_ascii_lowercase().contains(stem),
                "prepare.rs carries its own copy of {stem:?} — the list lives in brief_lint.rs alone"
            );
        }
    }

    // ── slp-blind-lanes-procedure P1: the reading list reaches the prompt
    //    it was passed for, and never rides beside a brief ─────────────────
    //
    // `--expertise` was declared, accepted, parsed and rendered into a block
    // for every kind, and then DROPPED for every kind but `cell`: the
    // non-cell arm of `prompt_body_for` rendered a vars slice carrying
    // `brief` alone. The cases below are the positive proof PER KIND,
    // because the absent-case byte-identity walk above cannot see that
    // defect — an `{{#if}}` over an absent variable is silently falsy, so a
    // template twin that missed the edit would swallow the list again with
    // every other test in this file green.

    /// The `--expertise` argv value, in the flag's own
    /// `<path> :: <purpose> :: <read-to>` shape.
    const EXPERTISE_RAW: &str = "skills/bee-swarming/SKILL.md :: the worker contract :: follow the loop\ndocs/knowledge/index.md :: the knowledge index :: find the area concept";

    /// What `run_dispatch_prepare` renders `EXPERTISE_RAW` into before it
    /// hands the block to the prompt renderer.
    const EXPERTISE_BLOCK: &str = concat!(
        "- skills/bee-swarming/SKILL.md — the worker contract. Read it to follow the loop.\n",
        "- docs/knowledge/index.md — the knowledge index. Read it to find the area concept."
    );

    /// The exact bytes the block must add to a non-cell body — heading,
    /// blank line and all. Written out literally rather than derived from
    /// the template, so a template edit that moves the newlines fails here.
    const EXPERTISE_SECTION: &str = concat!(
        "\n\nExpertise — dispatcher-picked; read/load before you start:\n",
        "- skills/bee-swarming/SKILL.md — the worker contract. Read it to follow the loop.\n",
        "- docs/knowledge/index.md — the knowledge index. Read it to find the area concept."
    );

    fn non_cell_body(root: &Path, runtime: &str, kind: &str, expertise: Option<&str>) -> String {
        let Prepared::Value(v) =
            prepare_dispatch(root, runtime, kind, None, None, false, None, None, false, expertise)
                .unwrap()
        else {
            panic!("expected a {kind} envelope on {runtime}")
        };
        dispatched_body(&v)
    }

    #[test]
    fn a_gather_dispatch_carrying_expertise_renders_the_reading_list() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        for runtime in DISPATCH_RUNTIMES {
            let plain = non_cell_body(&root, runtime, "gather", None);
            let carried = non_cell_body(&root, runtime, "gather", Some(EXPERTISE_BLOCK));
            assert_eq!(
                carried,
                format!("{plain}{EXPERTISE_SECTION}"),
                "{runtime}: a gather dispatch dropped the reading list it was passed"
            );
            assert_eq!(
                carried.matches("skills/bee-swarming/SKILL.md").count(),
                1,
                "{runtime}: the list must render exactly once"
            );
        }
    }

    #[test]
    fn a_reviewer_dispatch_carrying_expertise_renders_the_reading_list() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        for runtime in DISPATCH_RUNTIMES {
            let plain = non_cell_body(&root, runtime, "reviewer", None);
            let carried = non_cell_body(&root, runtime, "reviewer", Some(EXPERTISE_BLOCK));
            assert_eq!(
                carried,
                format!("{plain}{EXPERTISE_SECTION}"),
                "{runtime}: a reviewer dispatch dropped the reading list it was passed"
            );
            assert_eq!(
                carried.matches("docs/knowledge/index.md").count(),
                1,
                "{runtime}: the list must render exactly once"
            );
        }
    }

    /// The advisor kind carries a brief block too, so this case also pins
    /// that the two blocks stay disjoint: a briefless dispatch with a
    /// reading list carries the list and no brief prose at all.
    #[test]
    fn a_briefless_advisor_dispatch_carrying_expertise_renders_the_reading_list() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        for runtime in DISPATCH_RUNTIMES {
            let carried = non_cell_body(&root, runtime, "advisor", Some(EXPERTISE_BLOCK));
            assert_eq!(
                carried,
                format!("{ADVISOR_BODY_WITHOUT_A_BRIEF}{EXPERTISE_SECTION}"),
                "{runtime}: a briefless advisor dispatch dropped the reading list"
            );
            assert!(
                !carried.contains("Brief ("),
                "{runtime}: no brief travelled, so no brief block may render"
            );
        }
    }

    /// THE EXCEPTION. A reading list may never ride beside a brief. The
    /// leaning guard reads brief bytes and nothing else, by design, so an
    /// `--expertise` list travelling with a brief is an unlinted channel
    /// into a blind lane; and `brief_sha256` would keep proving the BRIEFS
    /// were equal while the payloads diverged. It is a typed refusal, in the
    /// same shape `brief_kind_not_advisor` takes, and it fires BEFORE the
    /// brief file is read.
    #[test]
    fn a_reading_list_beside_a_brief_is_refused_before_the_brief_is_read() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        let path = brief_file(&root, "brief.md", &shaped_brief("Where does the dossier live?"));

        let refusal =
            resolve_brief_with_expertise("advisor", Some(&path), Some(EXPERTISE_BLOCK)).unwrap_err();
        assert_eq!(refusal.get("ok"), Some(&json!(false)), "{refusal}");
        assert_eq!(refusal.get("type"), Some(&json!("refused")), "{refusal}");
        assert_eq!(refusal.get("reason"), Some(&json!("expertise_beside_brief")), "{refusal}");
        let fix = refusal.get("fix").and_then(Value::as_str).unwrap_or_default();
        assert!(
            fix.contains("## Read diet"),
            "the fix must name the brief's own Read diet section as the one carrier: {fix}"
        );

        // Refused before ANY file is read: an unreadable path earns this
        // refusal, never `brief_file_unreadable`.
        let missing =
            resolve_brief_with_expertise("advisor", Some("no/such/brief.md"), Some(EXPERTISE_BLOCK))
                .unwrap_err();
        assert_eq!(
            missing.get("reason"),
            Some(&json!("expertise_beside_brief")),
            "the combination is refused before the read is attempted: {missing}"
        );

        // Neither half alone refuses, and an EMPTY list carries nothing, so
        // there is nothing unlinted to refuse.
        assert!(resolve_brief_with_expertise("advisor", Some(&path), None).is_ok());
        assert!(resolve_brief_with_expertise("advisor", Some(&path), Some("")).is_ok());
        assert!(resolve_brief_with_expertise("gather", None, Some(EXPERTISE_BLOCK)).is_ok());

        // And the kind arm keeps its own refusal — the combination check
        // never speaks for a kind that may not carry a brief at all.
        let kind_refusal =
            resolve_brief_with_expertise("cell", Some(&path), Some(EXPERTISE_BLOCK)).unwrap_err();
        assert_eq!(
            kind_refusal.get("reason"),
            Some(&json!("brief_kind_not_advisor")),
            "{kind_refusal}"
        );
    }

    const BRIEF_WITH_EXPERTISE_CHILD: &str =
        "verbs::drivers::tests::dispatch_prepare_brief_with_expertise_child";
    const EXPERTISE_NO_BRIEF_CHILD: &str =
        "verbs::drivers::tests::dispatch_prepare_expertise_without_brief_child";

    /// Runs ONLY as a child of the door test below.
    #[test]
    #[ignore = "spawned by the_dispatch_door_refuses_a_reading_list_beside_a_brief"]
    fn dispatch_prepare_brief_with_expertise_child() {
        let (flags, use_json) = parse_flags(&[
            "--runtime",
            "claude",
            "--kind",
            "advisor",
            "--brief-file",
            "shaped.md",
            "--expertise",
            EXPERTISE_RAW,
            "--json",
        ])
        .expect("well-formed fixture argv");
        run_dispatch_prepare(flags, use_json, Instant::now());
    }

    /// Runs ONLY as a child of the door test below.
    #[test]
    #[ignore = "spawned by the_dispatch_door_refuses_a_reading_list_beside_a_brief"]
    fn dispatch_prepare_expertise_without_brief_child() {
        let (flags, use_json) = parse_flags(&[
            "--runtime",
            "claude",
            "--kind",
            "gather",
            "--expertise",
            EXPERTISE_RAW,
            "--json",
        ])
        .expect("well-formed fixture argv");
        run_dispatch_prepare(flags, use_json, Instant::now());
    }

    /// The door itself, through the REAL cli entry: the combination refuses
    /// with nothing reaching a payload, and the same list WITHOUT a brief
    /// reaches the rendered body. Without the second half a door that
    /// refused every `--expertise` would pass the first half.
    #[test]
    fn the_dispatch_door_refuses_a_reading_list_beside_a_brief() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        std::fs::write(root.join("shaped.md"), shaped_brief("Where does the dossier live?"))
            .unwrap();

        let refused = child_payload(&root, BRIEF_WITH_EXPERTISE_CHILD);
        assert_eq!(refused.get("ok"), Some(&json!(false)), "{refused}");
        assert_eq!(refused.get("type"), Some(&json!("refused")), "{refused}");
        assert_eq!(refused.get("reason"), Some(&json!("expertise_beside_brief")), "{refused}");
        assert!(
            refused.get("payload").is_none(),
            "a refused combination never reaches a payload: {refused}"
        );
        assert!(
            refused.get("brief_sha256").is_none(),
            "nothing travelled, so nothing is digested: {refused}"
        );

        let carried = child_payload(&root, EXPERTISE_NO_BRIEF_CHILD);
        assert!(carried.get("payload").is_some(), "a briefless list reaches a payload: {carried}");
        assert!(
            dispatched_body(&carried).ends_with(EXPERTISE_SECTION),
            "the door rendered the list it was passed: {carried}"
        );
    }

    /// The vendored half. Each of the three edited templates gets its OWN
    /// disk probe, the same hazard the advisor brief block already carries:
    /// a source-only prompt edit skews the embedded template from disk,
    /// `dispatch prepare` returns None, and the Node delegate it would fall
    /// to was deleted at the R6 cutover. A twin that missed THIS edit is
    /// worse than loud — `{{#if}}` over an absent variable is falsy, so it
    /// would silently swallow the reading list again.
    #[test]
    fn every_non_cell_template_carries_the_expertise_block_and_matches_disk() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("..");
        for name in ["gather", "reviewer", "advisor"] {
            assert!(
                prompts_match_disk(&repo_root, name),
                "packages/bee/prompts/{name}.md or .bee/bin/prompts/{name}.md drifted from the \
                 compiled-in template — run `bee dev regen` and rebuild, or every dispatch \
                 prepare in this checkout resolves to nothing"
            );
            let template = load_prompt(name).unwrap();
            let at = template
                .find("{{#if expertise}}")
                .unwrap_or_else(|| panic!("{name}.md carries no expertise block"));
            assert!(at > 0, "{name}: a block at byte 0 has no leading newline to consume");
            assert_eq!(&template[at - 1..at], "\n", "{name}: the marker needs its leading newline");
            assert!(
                template.contains("{{expertise}}"),
                "{name}: the block must hold the expertise placeholder"
            );
            assert_eq!(
                render(&template, &[]).unwrap(),
                render(&template, &[("expertise", "")]).unwrap(),
                "{name}: an empty list must leave zero residue bytes"
            );
        }
    }

    // ── slp-contract-original-request S1: the user's own words ride every
    //    dispatch (D5, D6) ──────────────────────────────────────────────────
    //
    // Existing coverage judged first. `no_brief_leaves_every_runtime_and_kind_
    // pair_exactly_as_it_was` already walks runtime × kind for byte identity,
    // but it names the BRIEF and never writes an anchor, so it cannot tell an
    // anchor that resolved to nothing from one that was never read. Nothing
    // in this file writes `.bee/intent/*` at all. The cases below are that
    // gap: the anchor resolving, the anchor deliberately NOT resolving, and
    // what the request's own bytes may never do to the template.

    /// The anchor's own framing, imported rather than retyped — a header edit
    /// in `intent_group.rs` must move this assertion with it, not slip past.
    use crate::verbs::intent_group::{
        dispatch_original_request, PRECOMPACT_FOOTER, PRECOMPACT_HEADER,
    };

    /// The user's own words: punctuated, em-dashed and multi-line, so
    /// "verbatim" means more than "a substring survived".
    const ORIGINAL_REQUEST: &str = "Make the /orders endpoint idempotent under retries — the same Idempotency-Key must never\ncreate a second order, and please do NOT change the existing response shape.";

    /// A repo with one claimed cell on feature `f`, ready for a dispatch of
    /// any kind.
    fn anchor_host(tmp: &tempfile::TempDir) -> PathBuf {
        let root = repo(tmp, BRIEF_HOST);
        w(
            &root,
            ".bee/cells/c-1.json",
            r#"{"id":"c-1","feature":"f","title":"carry the request","role":"code","status":"claimed","trace":{"worker":"w"}}"#,
        );
        root
    }

    /// An anchor on disk under `key`, in the shape `intent set` writes.
    fn write_anchor(root: &Path, key: &str, request: &str) {
        let body = serde_json::to_string(&json!({
            "schema_version": "1.0",
            "key": key,
            "written_at": "2026-08-29T00:00:00.000Z",
            "request": request,
            "acceptance": "the user's words arrive unchanged",
        }))
        .unwrap();
        w(root, &format!(".bee/intent/{key}.json"), &body);
    }

    /// Active work on `feature` — `active_feature` reads the phase first, so
    /// an idle phase resolves to no feature no matter what the field says.
    fn active_feature_is(root: &Path, feature: &str) {
        w(root, ".bee/state.json", &format!(r#"{{"phase":"executing","feature":"{feature}"}}"#));
    }

    fn body_of_kind(root: &Path, runtime: &str, kind: &str) -> String {
        let (cell, worker) = if kind == "cell" { (Some("c-1"), Some("w")) } else { (None, None) };
        let Prepared::Value(v) =
            prepare_dispatch(root, runtime, kind, cell, worker, false, None, None, false, None)
                .unwrap()
        else {
            panic!("expected a {kind} envelope on {runtime}")
        };
        dispatched_body(&v)
    }

    /// The bytes the block adds, built from the anchor's own constants.
    fn expected_block(request: &str) -> String {
        format!("{PRECOMPACT_HEADER}\nORIGINAL REQUEST (verbatim):\n{request}\n{PRECOMPACT_FOOTER}")
    }

    /// The absent case, per runtime × kind. The non-cell kinds are compared
    /// against `render(template, &[])` — the literal expression this door
    /// used before any var existed — and the cell kind against a hand-written
    /// literal of the seam the block lands in, so a stray newline on either
    /// side of the marker is a red line rather than a silent reflow.
    #[test]
    fn no_resolvable_anchor_leaves_every_runtime_and_kind_pair_byte_identical() {
        for runtime in DISPATCH_RUNTIMES {
            for kind in DISPATCH_KINDS {
                let tmp = tempfile::tempdir().unwrap();
                let root = anchor_host(&tmp);
                let body = body_of_kind(&root, runtime, kind);
                assert!(
                    !body.contains(PRECOMPACT_HEADER),
                    "{runtime}/{kind}: no anchor resolved, so no block may render: {body}"
                );
                assert!(!body.contains("{{"), "{runtime}/{kind}: unrendered marker left behind");
                if kind == "cell" {
                    assert!(
                        body.starts_with(
                            "Nickname (reservation identity): w\nAssigned cell id: c-1\nFeature: f\n\nCell (authoritative — do not re-fetch):"
                        ),
                        "{runtime}/{kind}: the worker prompt head drifted: {body}"
                    );
                } else {
                    assert_eq!(
                        body,
                        render(&load_prompt(kind).unwrap(), &[]).unwrap(),
                        "{runtime}/{kind}: payload bytes drifted"
                    );
                }
            }
        }
    }

    /// The carrier, per kind. This is the case a missed var slice fails:
    /// `{{#if}}` over an absent variable is silently falsy, so the
    /// byte-identity walk above would stay green with the block swallowed for
    /// a whole kind.
    #[test]
    fn a_resolvable_anchor_rides_every_dispatch_kind_verbatim() {
        for runtime in DISPATCH_RUNTIMES {
            for kind in DISPATCH_KINDS {
                let tmp = tempfile::tempdir().unwrap();
                let root = anchor_host(&tmp);
                active_feature_is(&root, "f");
                write_anchor(&root, "f", ORIGINAL_REQUEST);
                let body = body_of_kind(&root, runtime, kind);
                assert_eq!(
                    body.matches(&expected_block(ORIGINAL_REQUEST)).count(),
                    1,
                    "{runtime}/{kind}: the request must ride exactly once, under the anchor's own header: {body}"
                );
                assert!(!body.contains("{{"), "{runtime}/{kind}: unrendered marker left behind");
            }
        }
    }

    /// THE LOAD-BEARING ONE. `read_anchor_at` applies no TTL and no staleness
    /// check, and the default key is where a session with no feature parks its
    /// anchor — so a featureless dispatch that walked to `default` would print
    /// somebody else's days-old request under a DO-NOT-PARAPHRASE banner. It
    /// renders nothing instead, by every route: no key at all, an active
    /// feature whose own anchor is missing, and a feature slug that
    /// `sanitize_intent_key` collapses onto the default key.
    #[test]
    fn a_featureless_dispatch_reads_no_default_anchor_even_when_one_exists() {
        const STALE: &str = "four days ago, about an unrelated and already-shipped bug";
        for runtime in DISPATCH_RUNTIMES {
            for kind in ["gather", "reviewer", "advisor"] {
                let tmp = tempfile::tempdir().unwrap();
                let root = repo(&tmp, BRIEF_HOST);
                write_anchor(&root, "default", STALE);
                let body = body_of_kind(&root, runtime, kind);
                assert!(!body.contains(STALE), "{runtime}/{kind}: read a stale default anchor");
                assert_eq!(
                    body,
                    render(&load_prompt(kind).unwrap(), &[]).unwrap(),
                    "{runtime}/{kind}: payload bytes drifted"
                );
            }
        }
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, BRIEF_HOST);
        write_anchor(&root, "default", STALE);
        assert_eq!(dispatch_original_request(&root, None), None, "no key resolves to no request");
        assert_eq!(
            dispatch_original_request(&root, Some("default")),
            None,
            "the default key is refused even when it is asked for by name"
        );
        assert_eq!(
            dispatch_original_request(&root, Some("***")),
            None,
            "a slug that sanitizes to \"default\" must not reach the default anchor"
        );
        assert_eq!(dispatch_original_request(&root, Some("f")), None, "no anchor at key f");
        // An idle phase is no active feature, even with the field set.
        w(&root, ".bee/state.json", r#"{"phase":"idle","feature":"f"}"#);
        write_anchor(&root, "f", "the feature's own words");
        assert_eq!(dispatch_original_request(&root, None), None, "idle resolves no feature");
        active_feature_is(&root, "f");
        assert_eq!(
            dispatch_original_request(&root, None).as_deref(),
            Some("the feature's own words"),
            "an ACTIVE feature is the one fallback this door takes"
        );
    }

    /// The cell's own feature is read first; the active feature is the second
    /// candidate, not the first.
    #[test]
    fn a_cell_dispatch_reads_its_own_features_anchor_before_the_active_one() {
        let tmp = tempfile::tempdir().unwrap();
        let root = anchor_host(&tmp);
        active_feature_is(&root, "other");
        write_anchor(&root, "f", "the cell's own feature asked for this");
        write_anchor(&root, "other", "a different feature asked for that");
        let body = body_of_kind(&root, "claude", "cell");
        assert!(body.contains("the cell's own feature asked for this"), "{body}");
        assert!(!body.contains("a different feature asked for that"), "{body}");

        // …and with no anchor under the cell's feature, the active feature is
        // the documented second candidate.
        std::fs::remove_file(root.join(".bee").join("intent").join("f.json")).unwrap();
        let body = body_of_kind(&root, "claude", "cell");
        assert!(body.contains("a different feature asked for that"), "{body}");
    }

    /// A request is DATA. It is substituted in pass 2, which walks the
    /// pre-substitution text, so template syntax inside the user's words is
    /// carried as characters and never interpreted — and the placeholders it
    /// names are not re-expanded.
    #[test]
    fn a_request_carrying_template_syntax_is_carried_as_text() {
        const INJECT: &str =
            "Render {{worker}} and {{#if brief}}this{{/if}} literally — do not expand {{cell_json}}.";
        for kind in DISPATCH_KINDS {
            let tmp = tempfile::tempdir().unwrap();
            let root = anchor_host(&tmp);
            active_feature_is(&root, "f");
            write_anchor(&root, "f", INJECT);
            let body = body_of_kind(&root, "claude", kind);
            assert_eq!(
                body.matches(&expected_block(INJECT)).count(),
                1,
                "{kind}: the request's own braces must survive verbatim: {body}"
            );
        }
    }

    /// Non-ASCII survives byte for byte — the renderer walks chars, and a
    /// truncation or a re-encode would show up here first.
    #[test]
    fn a_non_ascii_request_survives_byte_for_byte() {
        const VI: &str =
            "Hãy giữ nguyên yêu cầu của tôi — đừng tóm tắt, đừng diễn giải lại. 🐝 Ünïcödé ok?";
        let tmp = tempfile::tempdir().unwrap();
        let root = anchor_host(&tmp);
        active_feature_is(&root, "f");
        write_anchor(&root, "f", VI);
        for kind in DISPATCH_KINDS {
            let body = body_of_kind(&root, "claude", kind);
            assert_eq!(body.matches(&expected_block(VI)).count(), 1, "{kind}: {body}");
        }
        assert_eq!(dispatch_original_request(&root, Some("f")).as_deref(), Some(VI));
    }

    /// A blank request is no anchor at all: `{{#if}}` truthiness is
    /// `!is_empty()`, so a whitespace request that reached the var would
    /// splice an EMPTY block where today's bytes are.
    #[test]
    fn a_whitespace_only_request_is_no_request_at_all() {
        for blank in ["   ", "\n\n", " \t \r\n  \n"] {
            let tmp = tempfile::tempdir().unwrap();
            let root = anchor_host(&tmp);
            active_feature_is(&root, "f");
            write_anchor(&root, "f", blank);
            assert_eq!(
                dispatch_original_request(&root, Some("f")),
                None,
                "a request of {blank:?} must read as absent"
            );
            for kind in DISPATCH_KINDS {
                let body = body_of_kind(&root, "claude", kind);
                assert!(!body.contains(PRECOMPACT_HEADER), "{kind}: {body}");
                if kind != "cell" {
                    assert_eq!(body, render(&load_prompt(kind).unwrap(), &[]).unwrap(), "{kind}");
                }
            }
        }
    }

    /// Error cascade: an unusable anchor degrades to "no anchor" and never
    /// fails the dispatch. Corrupt bytes, a non-object payload, and a record
    /// with no `request` field all land in the same place.
    #[test]
    fn an_unusable_anchor_degrades_to_no_block_and_never_fails_the_dispatch() {
        for payload in ["{broken", "[]", r#""just a string""#, r#"{"acceptance":"no request"}"#] {
            let tmp = tempfile::tempdir().unwrap();
            let root = anchor_host(&tmp);
            active_feature_is(&root, "f");
            w(&root, ".bee/intent/f.json", payload);
            assert_eq!(
                dispatch_original_request(&root, Some("f")),
                None,
                "an anchor of {payload:?} must read as absent"
            );
            for kind in DISPATCH_KINDS {
                let body = body_of_kind(&root, "claude", kind);
                assert!(!body.contains(PRECOMPACT_HEADER), "{kind}/{payload:?}: {body}");
                if kind != "cell" {
                    assert_eq!(
                        body,
                        render(&load_prompt(kind).unwrap(), &[]).unwrap(),
                        "{kind}/{payload:?}: payload bytes drifted"
                    );
                }
            }
        }
    }

    /// Why the var had to join BOTH slices: pass 2 refuses a bare `{{NAME}}`
    /// with no supplied value. Inside an `{{#if}}` the same omission is
    /// SILENT, which is exactly why the per-kind carrier case above exists.
    #[test]
    fn a_template_var_missing_from_its_arms_slice_fails_loudly() {
        let err = render("Head: {{original_request}}\n", &[("brief", "")]).unwrap_err();
        assert!(
            err.contains("no value supplied for placeholder {{original_request}}"),
            "the refusal must name the placeholder: {err}"
        );
        assert!(render("Head: {{original_request}}\n", &[("original_request", "")]).is_ok());
        // The silent half, stated so the asymmetry is on the record.
        assert_eq!(
            render("A\n{{#if original_request}}\nB\n{{/if}}\nC", &[]).unwrap(),
            "A\nC",
            "an absent var inside an if-block is dropped, not refused"
        );
    }

    /// Every template that declares the block declares it the same way, and
    /// the shipped copies on disk match the compiled-in bytes — a prompt edit
    /// and a rebuilt vendored twin are one unit of work.
    #[test]
    fn every_template_carries_the_original_request_block_and_matches_disk() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("..");
        for name in ["worker-cell", "gather", "reviewer", "advisor"] {
            assert!(
                prompts_match_disk(&repo_root, name),
                "packages/bee/prompts/{name}.md or .bee/bin/prompts/{name}.md drifted from the \
                 compiled-in template — run `bee dev regen` and rebuild"
            );
            let template = load_prompt(name).unwrap();
            let at = template
                .find("{{#if original_request}}")
                .unwrap_or_else(|| panic!("{name}.md carries no original_request block"));
            assert!(at > 0, "{name}: a block at byte 0 has no leading newline to consume");
            assert_eq!(&template[at - 1..at], "\n", "{name}: the marker needs its leading newline");
            assert!(
                template.contains("{{original_request}}"),
                "{name}: the block must hold the placeholder"
            );
            // The rest of the arm's slice, so the comparison isolates the one
            // var under test. `worker-cell` carries bare placeholders that
            // pass 2 refuses when unsupplied, so they cannot be left out.
            let without: Vec<(&str, &str)> = if name == "worker-cell" {
                vec![
                    ("worker", ""),
                    ("cell_id", ""),
                    ("feature", ""),
                    ("cell_json", ""),
                    ("learned_context", ""),
                    ("expertise", ""),
                    ("prior_rounds", ""),
                    ("worktree_root", ""),
                    ("control_root", ""),
                ]
            } else {
                vec![("brief", ""), ("expertise", "")]
            };
            let mut with = without.clone();
            with.push(("original_request", ""));
            assert_eq!(
                render(&template, &without).unwrap(),
                render(&template, &with).unwrap(),
                "{name}: an empty request must leave zero residue bytes"
            );
        }
    }

    // ── pi-support D5/D6: the herding-only dispatch door ───────────────────
    //
    // Existing coverage judged first. The runtime × kind walks above now
    // include `pi` from `DISPATCH_RUNTIMES` itself, so the HAPPY path (a
    // herding slot renders and carries its prompt) is already held by them.
    // What no walk can hold is the REFUSAL surface: D5 says every resolution
    // that is not `Resolved::Herding` is refused by name, and each of those
    // shapes needs its own config to reach. That is the gap below — one row
    // per resolution arm, plus the D6 table's own values, plus the two gates
    // a runtime name has to pass before the handler is even called.

    /// The D6 table verbatim (settled 2026-08-29): heavy roles on an opus
    /// herding agent, advisor on fable, cheap roles on agy-flash. Every value
    /// is herding-SHAPED because D5 admits nothing else on pi — D6 constrains
    /// the transport, not the model vendor, so the vendor lives inside the
    /// named `herding.agents` entry rather than in this table.
    const PI_D6_HOST: &str = r#"{"models":{"pi":{
        "code":{"kind":"herding","agent":"pi-claude-opus"},
        "test":{"kind":"herding","agent":"pi-claude-opus"},
        "docs":{"kind":"herding","agent":"pi-claude-opus"},
        "review":{"kind":"herding","agent":"pi-claude-opus"},
        "advisor":{"kind":"herding","agent":"pi-claude-fable"},
        "read":{"kind":"herding","agent":"pi-agy-flash"},
        "extraction":{"kind":"herding","agent":"pi-agy-flash"},
        "generation":{"kind":"herding","agent":"pi-agy-flash"},
        "supervisor":{"kind":"herding","agent":"pi-agy-flash"}
    }}}"#;

    /// One `models.pi` table, as a config string.
    fn pi_host(table: &str) -> String {
        format!(r#"{{"models":{{"pi":{table}}}}}"#)
    }

    /// A claimed cell declaring `role`, ready for a `--kind cell` dispatch.
    fn pi_cell(root: &Path, id: &str, role: &str, extra: &str) {
        w(
            root,
            &format!(".bee/cells/{id}.json"),
            &format!(
                r#"{{"id":"{id}","feature":"f","title":"ship the pi belt","role":"{role}",{extra}"status":"claimed","trace":{{"worker":"w"}}}}"#
            ),
        );
    }

    fn pi_envelope(root: &Path, kind: &str, role: Option<&str>, cell: Option<&str>) -> Value {
        let worker = cell.map(|_| "w");
        let Prepared::Value(v) = prepare_dispatch_with_role(
            root, "pi", kind, role, cell, worker, false, None, None, false, None,
        )
        .unwrap() else {
            panic!("expected an envelope or a typed refusal on pi/{kind}")
        };
        v
    }

    /// THE SERVED PATH: a herding slot is the one resolution the pi door
    /// carries, and it carries it as the ordinary herding-exec Bash payload —
    /// the brief on stdin, the slot's own agent named on the command line.
    #[test]
    fn a_herding_slot_on_pi_returns_the_herding_exec_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, PI_D6_HOST);
        pi_cell(&root, "c-1", "code", "");
        let v = pi_envelope(&root, "cell", None, Some("c-1"));

        assert_eq!(v.get("tool"), Some(&json!("Bash")), "a pi dispatch is a herding pane: {v}");
        let payload = v.get("payload").unwrap();
        let command = payload.get("command").unwrap().as_str().unwrap();
        assert!(
            command.starts_with(".bee/bin/bee herding run --task-file - --json"),
            "the herding command drifted: {command}"
        );
        assert!(
            command.contains(r#"--agent "pi-claude-opus""#),
            "the slot's own agent must ride the command: {command}"
        );
        assert!(
            payload.get("stdin").unwrap().as_str().unwrap().contains("c-1"),
            "the worker prompt travels on stdin"
        );
        assert_eq!(
            v.get("economics").unwrap().get("channel"),
            Some(&json!("herding-exec")),
            "{v}"
        );
    }

    /// D6's VALUES, not only its shape: every role in the settled table
    /// resolves to the agent the table names. A refusal-only suite would stay
    /// green with the whole table mis-keyed.
    #[test]
    fn the_d6_table_values_resolve_to_their_named_agents_on_pi() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, PI_D6_HOST);
        let rows = [
            ("code", "pi-claude-opus"),
            ("test", "pi-claude-opus"),
            ("docs", "pi-claude-opus"),
            ("review", "pi-claude-opus"),
            ("advisor", "pi-claude-fable"),
            ("read", "pi-agy-flash"),
            ("extraction", "pi-agy-flash"),
            ("generation", "pi-agy-flash"),
            ("supervisor", "pi-agy-flash"),
        ];
        for (role, agent) in rows {
            // `--role` names the slot outright, so each row reads its OWN
            // entry rather than whatever a walk fell through to.
            let kind = if role == "advisor" { "advisor" } else { "gather" };
            let v = pi_envelope(&root, kind, Some(role), None);
            let command = v
                .get("payload")
                .and_then(|p| p.get("command"))
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("{role}: expected a herding payload, got {v}"));
            assert!(
                command.contains(&format!(r#"--agent "{agent}""#)),
                "{role}: expected agent {agent}, command was {command}"
            );
        }
    }

    /// D5 AT FULL WIDTH. Every non-herding resolution arm — plain-string
    /// `Model`, `Native`, `Cli` (on both a gather purpose, which resolves it,
    /// and a non-gather purpose, which hits the cli purpose gate first) and
    /// the `Budget` floor an empty table lands on — is refused by ONE typed
    /// refusal that names the slot, the resolution and the herding shape to
    /// configure. Pi ships no Agent surface, so any of these would have
    /// dispatched nothing while the envelope read as a success.
    #[test]
    fn every_non_herding_slot_shape_on_pi_is_refused_by_name() {
        let rows: [(&str, &str, &str, &str); 5] = [
            ("plain string", r#"{"generation":"opus"}"#, "gather", "model"),
            (
                "native",
                r#"{"generation":{"kind":"native","model":"gpt-5"}}"#,
                "gather",
                "native",
            ),
            (
                "cli on a gather",
                r#"{"generation":{"kind":"cli","command":"echo hi"}}"#,
                "gather",
                "cli",
            ),
            (
                "cli on a reviewer purpose",
                r#"{"review":{"kind":"cli","command":"echo hi"}}"#,
                "reviewer",
                "cli",
            ),
            ("empty table (budget floor)", r#"{}"#, "gather", "budget"),
        ];
        for (name, table, kind, resolution) in rows {
            let tmp = tempfile::tempdir().unwrap();
            let root = repo(&tmp, &pi_host(table));
            let v = pi_envelope(&root, kind, None, None);
            assert_eq!(v.get("ok"), Some(&json!(false)), "{name}: must refuse — {v}");
            assert_eq!(v.get("type"), Some(&json!("refused")), "{name}: {v}");
            assert_eq!(
                v.get("reason"),
                Some(&json!(PI_HERDING_ONLY_REASON)),
                "{name}: one reason word for the whole door — {v}"
            );
            assert_eq!(v.get("runtime"), Some(&json!("pi")), "{name}: {v}");
            assert_eq!(
                v.get("resolution"),
                Some(&json!(resolution)),
                "{name}: the refusal must name what the slot resolved to — {v}"
            );
            assert_eq!(v.get("payload"), None, "{name}: a refusal carries no payload — {v}");
            assert_eq!(v.get("tool"), None, "{name}: a refusal names no tool — {v}");
            let slot = v.get("slot").and_then(Value::as_str).unwrap_or_default();
            let fix = v.get("fix").and_then(Value::as_str).unwrap_or_default();
            assert!(!slot.is_empty(), "{name}: the refusal must name the slot — {v}");
            assert!(
                fix.contains(&format!("models.pi.{slot}")),
                "{name}: the fix must name the slot to configure: {fix}"
            );
            assert!(
                fix.contains("herding"),
                "{name}: the fix must name the herding requirement: {fix}"
            );
        }
    }

    /// THE ESCALATION ARM, in both of its spellings — a cell marked
    /// `escalate: true`, and an explicit `--role ceiling` with no cell behind
    /// it. On claude this resolves `Resolved::Inherit` and dispatches an
    /// Agent on the session model; on pi there IS no subagent to hand the
    /// session model to, so the refusal carries the one remedy that works:
    /// run the escalated cell inline in the session.
    #[test]
    fn an_escalated_dispatch_on_pi_refuses_with_the_inline_session_remedy() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, PI_D6_HOST);
        pi_cell(&root, "c-1", "code", r#""escalate":true,"#);
        // The control: the SAME table, unescalated, really does dispatch —
        // so the refusal below is the escalation and not a broken fixture.
        pi_cell(&root, "c-2", "code", "");
        assert_eq!(
            pi_envelope(&root, "cell", None, Some("c-2")).get("tool"),
            Some(&json!("Bash")),
            "the control must herd"
        );

        for v in [
            pi_envelope(&root, "cell", None, Some("c-1")),
            pi_envelope(&root, "gather", Some(ESCALATION_WORD), None),
        ] {
            assert_eq!(v.get("ok"), Some(&json!(false)), "{v}");
            assert_eq!(v.get("reason"), Some(&json!(PI_HERDING_ONLY_REASON)), "{v}");
            assert_eq!(v.get("resolution"), Some(&json!("escalation")), "{v}");
            let fix = v.get("fix").and_then(Value::as_str).unwrap_or_default();
            assert!(
                fix.contains(
                    "Pi has no subagent surface, run the escalated cell inline in the session"
                ),
                "the escalation refusal must carry the inline-session remedy: {fix}"
            );
        }
    }

    /// A role nothing configures still earns the standing `role_not_configured`
    /// refusal, which names the role and lists what this runtime CAN resolve —
    /// pi joining the runtime set changes none of that.
    #[test]
    fn a_role_nothing_configures_on_pi_is_refused_by_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = repo(&tmp, PI_D6_HOST);
        let v = pi_envelope(&root, "gather", Some("designer"), None);
        assert_eq!(v.get("ok"), Some(&json!(false)), "{v}");
        assert_eq!(v.get("reason"), Some(&json!("role_not_configured")), "{v}");
        assert_eq!(v.get("role"), Some(&json!("designer")), "{v}");
        let fix = v.get("fix").and_then(Value::as_str).unwrap_or_default();
        assert!(fix.contains("models.pi"), "the fix must name the pi table: {fix}");
    }

    /// GATE TWO of the three a runtime name passes. The argv-shape guard
    /// reads the `runtime` enum out of the embedded registry payload and
    /// refuses BEFORE the handler runs, so a runtime added to
    /// `DISPATCH_RUNTIMES` alone would be refused by the front door and never
    /// reach a line of the code above. Derived from the constant on both
    /// sides — never a hand-typed list — for `dispatch prepare` AND
    /// `dispatch wave`, which declare the flag separately.
    #[test]
    fn the_registry_enum_and_dispatch_runtimes_name_the_same_runtime_set() {
        let payload: Value =
            serde_json::from_str(include_str!("../../generated/registry_payload.json")).unwrap();
        let commands = payload["commands"].as_array().unwrap();
        for name in ["dispatch.prepare", "dispatch.wave"] {
            let entry = commands
                .iter()
                .find(|c| c["name"] == json!(name))
                .unwrap_or_else(|| panic!("the registry declares {name}"));
            let declared: Vec<&str> = entry["parameters"]["properties"]["runtime"]["enum"]
                .as_array()
                .unwrap_or_else(|| panic!("{name} declares a runtime enum"))
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            let mut expected: Vec<&str> = DISPATCH_RUNTIMES.to_vec();
            let mut got = declared.clone();
            expected.sort_unstable();
            got.sort_unstable();
            assert_eq!(
                got, expected,
                "{name}: the shape guard's enum and DISPATCH_RUNTIMES disagree — a runtime in \
                 one and not the other is a door that half-opens"
            );
            assert!(declared.contains(&"pi"), "{name}: --runtime pi must pass the shape guard");
        }
    }

    /// THE WAVE DOOR takes the same two answers: it gates `--runtime` on the
    /// same constant, and it recognises the pi refusal by its reason word so
    /// a refused cell's claim is UNWOUND into `skipped` instead of standing
    /// on a cell nothing can dispatch. Read out of the handler's own source —
    /// the same derivation the `keys_known` probe above uses — because the
    /// wave loop needs a claimed store, a worktree topology and a live
    /// session before it will run one line.
    #[test]
    fn the_wave_door_gates_on_the_same_constant_and_unwinds_a_pi_refusal() {
        const PREPARE_SOURCE: &str = include_str!("prepare.rs");
        let at = PREPARE_SOURCE
            .find("pub(crate) fn run_dispatch_wave(")
            .expect("run_dispatch_wave moved — re-anchor this");
        let body = &PREPARE_SOURCE[at..];
        let end = body.find("\n// ═══").unwrap_or(body.len());
        let body = &body[..end];
        assert!(
            body.contains("DISPATCH_RUNTIMES.contains(&runtime.as_str())"),
            "the wave door must gate --runtime on the one constant"
        );
        assert!(
            body.contains("PI_HERDING_ONLY_REASON"),
            "the wave loop must recognise the pi refusal by its reason word"
        );
        assert!(
            body.contains("unwind_wave_claim"),
            "and unwind the claim it took for a cell it cannot dispatch"
        );
    }
