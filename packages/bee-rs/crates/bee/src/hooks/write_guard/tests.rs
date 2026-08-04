// Split out of the single 5.9k-line hooks/write_guard.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's `#[cfg(test)] mod tests`,
// indentation and all: the fixtures are raw strings whose leading
// whitespace is content.

// The parent module's own  block travels with the tests: they reach
// for names mod.rs no longer imports now that the code using them lives
// in sibling modules.
#![allow(unused_imports)]

use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
    use super::*;
    use serde_json::json;

    struct Fx {
        _dir: tempfile::TempDir,
        root: PathBuf,
    }

    /// "bee is installed at this root" — the guard's activation probe.
    fn copy_lib(root: &Path) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        if !root.join(".bee").join("onboarding.json").is_file() {
            std::fs::write(root.join(".bee").join("onboarding.json"), "{}
").unwrap();
        }
    }

    fn write_state(root: &Path, state: &Value) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(
            root.join(".bee").join("state.json"),
            format!("{}\n", serde_json::to_string_pretty(state).unwrap()),
        )
        .unwrap();
    }

    fn swarming_state(execution: bool) -> Value {
        json!({
            "phase": "swarming",
            "mode": "standard",
            "feature": "demo",
            "approved_gates": { "context": true, "shape": true, "execution": execution, "review": false }
        })
    }

    fn build_fixture(phase: &str, execution_approved: bool) -> Fx {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&root);
        let mut st = swarming_state(execution_approved);
        st["phase"] = json!(phase);
        write_state(&root, &st);
        Fx { _dir: dir, root }
    }

    fn run_payload(payload: Value, cwd: &Path) -> R<Emit> {
        let mut body = match payload {
            Value::Object(m) => m,
            _ => panic!("payload must be an object"),
        };
        body.insert("cwd".into(), Value::String(cwd.to_string_lossy().into_owned()));
        let stdin = jsjson::stringify(&Value::Object(body));
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        run_native(&ctx)
    }

    fn expect_done(payload: Value, cwd: &Path) -> Emit {
        match run_payload(payload, cwd) {
            Ok(e) => e,
            Err(_) => panic!("expected a native verdict, got Delegate"),
        }
    }

    fn expect_delegate(payload: Value, cwd: &Path) {
        assert!(run_payload(payload, cwd).is_err(), "expected Delegate");
    }

    fn seed_lease(root: &Path, path: &str, agent: &str, cell: &str, session: Option<&str>, kind: &str) {
        let dir = root.join(".bee").join("runtime").join("leases").join("paths");
        std::fs::create_dir_all(&dir).unwrap();
        let now = now_ms();
        let acquired = ms_to_iso(now).unwrap();
        let expires = ms_to_iso(now + 3600.0 * 1000.0).unwrap();
        let record = json!({
            "resource": format!("path:{}", res_normalize_path(path)),
            "mode": "write",
            "workflow_id": cell,
            "session_id": session.unwrap_or(SESSIONLESS_SESSION_ID),
            "workspace_id": format!("agent:{}", agent),
            "epoch": 0,
            "acquired_at": acquired,
            "expires_at": expires,
            "kind": kind
        });
        let n = std::fs::read_dir(&dir).map(|d| d.count()).unwrap_or(0);
        std::fs::write(
            dir.join(format!("lease-{n}.json")),
            format!("{}\n", serde_json::to_string_pretty(&record).unwrap()),
        )
        .unwrap();
    }

    fn add_live_session(root: &Path, id: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        let now = ms_to_iso(now_ms()).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": id, "started_at": now, "last_heartbeat": now
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    fn edit(path: &str) -> Value {
        json!({ "tool_name": "Edit", "tool_input": { "file_path": path } })
    }
    fn bash(cmd: &str) -> Value {
        json!({ "tool_name": "Bash", "tool_input": { "command": cmd } })
    }
    fn patch(input: &str) -> Value {
        json!({ "tool_name": "apply_patch", "tool_input": { "input": input } })
    }

    // ── row1/2/3/3b/3c/3d/4/6: direct-edit deny table ──────────────────────

    #[test]
    fn direct_edit_state_json_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit(".bee/state.json"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee state"));
        assert!(e.stderr.contains("FIX"));
        assert!(e.stderr.contains("direct-edit"));
    }

    #[test]
    fn direct_edit_backlog_write_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"Write","tool_input":{"file_path":".bee/backlog.jsonl","content":"{}\n"}}),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee backlog add"));
    }

    #[test]
    fn bash_redirect_into_backlog_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("cat notes.txt >> .bee/backlog.jsonl"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee backlog add"));
    }

    #[test]
    fn sed_in_place_on_state_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("sed -i \"s/idle/swarming/\" .bee/state.json"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee state"));
    }

    #[test]
    fn docs_backlog_md_denied_with_owning_verbs() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"Write","tool_input":{"file_path":"docs/backlog.md","content":"x\n"}}),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        for needle in [
            "bee backlog pbi add",
            "bee backlog pbi status",
            "bee backlog pbi amend",
            "bee backlog render --write",
            "direct-edit",
        ] {
            assert!(e.stderr.contains(needle), "missing {needle}: {}", e.stderr);
        }
    }

    #[test]
    fn rest_of_docs_unaffected() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit("docs/history/demo/CONTEXT.md"), &fx.root);
        assert_eq!(e.code, 0, "stderr: {}", e.stderr);
    }

    // E2a (guard-hardening): flipped from the pre-E2 allow — cell files are
    // CLI-owned now, like the rest of the direct-edit table.
    #[test]
    fn bee_cells_json_denied_names_owning_verb() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit(".bee/cells/demo-1.json"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee cells add/finish"), "{}", e.stderr);
        assert!(e.stderr.contains("direct-edit"), "{}", e.stderr);
    }

    #[test]
    fn idle_still_denies_direct_edit_and_allows_other_bee_paths() {
        let fx = build_fixture("idle", true);
        let e = expect_done(edit(".bee/state.json"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee state"));
        // E2a: the cells arm flipped to deny (CLI-owned in every phase);
        // .bee/tmp/ stands in as the still-allowed other bee path.
        let e2 = expect_done(edit(".bee/cells/demo-1.json"), &fx.root);
        assert_eq!(e2.code, 2, "{}", e2.stderr);
        assert!(e2.stderr.contains("bee cells add/finish"), "{}", e2.stderr);
        let e3 = expect_done(edit(".bee/tmp/demo/notes.md"), &fx.root);
        assert_eq!(e3.code, 0, "{}", e3.stderr);
    }

    // ── E2 (guard-hardening): cells/lanes/onboarding join the CLI-owned set ─

    #[test]
    fn direct_edit_lanes_json_denied_names_owning_verbs() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit(".bee/lanes/demo.json"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("direct-edit"), "{}", e.stderr);
        assert!(e.stderr.contains("bee state start-feature --as-lane"), "{}", e.stderr);
        assert!(e.stderr.contains("bee state set --lane"), "{}", e.stderr);
    }

    #[test]
    fn direct_edit_onboarding_json_denied_names_owning_verb() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit(".bee/onboarding.json"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("direct-edit"), "{}", e.stderr);
        assert!(e.stderr.contains("bee onboard"), "{}", e.stderr);
    }

    #[test]
    fn bash_mutation_of_lanes_json_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("sed -i \"s/planning/swarming/\" .bee/lanes/demo.json"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee state start-feature --as-lane"), "{}", e.stderr);
        let e2 = expect_done(bash("cat patch.json >> .bee/cells/demo-1.json"), &fx.root);
        assert_eq!(e2.code, 2, "{}", e2.stderr);
        assert!(e2.stderr.contains("bee cells add/finish"), "{}", e2.stderr);
    }

    #[test]
    fn config_json_and_decisions_jsonl_stay_hand_writable() {
        // E2 explicitly preserves the two sanctioned agent surfaces:
        // gate-bypass config edits and decision log merges.
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"Write","tool_input":{"file_path":".bee/config.json","content":"{}\n"}}),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
        let e2 = expect_done(edit(".bee/decisions.jsonl"), &fx.root);
        assert_eq!(e2.code, 0, "{}", e2.stderr);
    }

    // ── check (d): CLI-shape validation, WIRED through the whole hook ──────
    // The pure decision table lives in hooks/cli_shape.rs; these rows prove
    // the wiring — that a denial reaches exit 2 on stderr, that a well-formed
    // call still exits 0, and that check (d) never overwrites an earlier deny.

    #[test]
    fn row5_5b_plain_bee_cli_invocations_still_pass() {
        let fx = build_fixture("swarming", true);
        let a = expect_done(bash("node .bee/bin/bee_state.mjs set --phase swarming"), &fx.root);
        assert_eq!(a.code, 0, "stderr={}", a.stderr);
        let b = expect_done(
            bash("node .bee/bin/bee_backlog.mjs add --type bug --title \"x\" --severity P2"),
            &fx.root,
        );
        assert_eq!(b.code, 0, "stderr={}", b.stderr);
    }

    #[test]
    fn rows5c_5d_a_malformed_bee_cli_call_is_denied_at_exit_two() {
        let fx = build_fixture("swarming", true);
        for command in [
            "node .bee/bin/bee_cells.mjs cap --outcome \"done\"",
            "node .bee/bin/bee.mjs cells cap --outcome \"done\"",
            // R6a spellings — Node saw neither of these.
            ".bee/bin/bee cells cap --outcome \"done\"",
            "bee cells cap --outcome \"done\"",
        ] {
            let e = expect_done(bash(command), &fx.root);
            assert_eq!(e.code, 2, "{command}");
            assert!(e.stdout.is_empty(), "{command}: {}", e.stdout);
            assert!(e.stderr.contains("bee CLI-shape guard"), "{command}: {}", e.stderr);
            assert!(e.stderr.contains("cells.cap"), "{command}: {}", e.stderr);
            assert!(e.stderr.contains("field: id"), "{command}: {}", e.stderr);
        }
    }

    #[test]
    fn a_well_formed_bee_cli_call_reaches_the_ordinary_verdict() {
        let fx = build_fixture("swarming", true);
        for command in [
            "node .bee/bin/bee.mjs cells cap --id demo-1 --outcome done",
            ".bee/bin/bee cells cap --id demo-1 --outcome done",
            "bee status --json",
        ] {
            let e = expect_done(bash(command), &fx.root);
            assert_eq!(e.code, 0, "{command}: {}", e.stderr);
        }
    }

    #[test]
    fn check_d_never_overwrites_a_denial_an_earlier_check_computed() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/reserved.js", "other-agent", "c1", None, "reservation");
        let mut payload = bash("rm src/reserved.js && bee cells cap --outcome done");
        payload["tool_input"]["agent_name"] = json!("me");
        payload["agent_name"] = json!("me");
        let e = expect_done(payload, &fx.root);
        assert_eq!(e.code, 2, "stderr={}", e.stderr);
        assert!(
            e.stderr.contains("bee reservation conflict"),
            "the ORIGINAL deny must survive: {}",
            e.stderr
        );
        assert!(
            !e.stderr.contains("CLI-shape guard"),
            "check (d) must never assign once a denial exists: {}",
            e.stderr
        );
    }

    /// CUTOVER — replaces `a_tampered_registry_still_delegates…` and
    /// `tampered_vendored_lib_delegates`, whose whole subject was the retired
    /// vendored-lib byte gate: a host whose `command-registry.mjs` or
    /// `guards.mjs` differed by a byte used to push the guard back to Node.
    /// There is no vendored lib to tamper with any more, and the property that
    /// mattered — the guard NEVER goes quiet on a host it does not recognise —
    /// is now carried by these two arms instead.
    #[test]
    fn a_stray_vendored_lib_no_longer_changes_any_verdict() {
        let fx = build_fixture("swarming", true);
        // A leftover .bee/bin/lib from a pre-cutover install is inert: the
        // guard reads its own compiled-in semantics and denies exactly as it
        // would with the directory absent.
        let lib = fx.root.join(".bee").join("bin").join("lib");
        std::fs::create_dir_all(&lib).unwrap();
        std::fs::write(lib.join("guards.mjs"), "throw new Error('boom');\n").unwrap();
        std::fs::write(lib.join("command-registry.mjs"), "export const COMMAND_REGISTRY = [];\n")
            .unwrap();
        let e = expect_done(edit(".bee/state.json"), &fx.root);
        assert_eq!(e.code, 2, "the guard still decides: {}", e.stderr);
    }

    #[test]
    fn node_inline_eval_delegates() {
        let fx = build_fixture("swarming", true);
        expect_delegate(
            bash("node -e \"import('./.bee/bin/lib/cells.mjs').then(() => {})\""),
            &fx.root,
        );
        // A file-based node run is native and allowed.
        let e = expect_done(bash("node scripts/test_guards.mjs"), &fx.root);
        assert_eq!(e.code, 0);
    }

    // ── AskUserQuestion (ask-guard-autofix D1/D2) ──────────────────────────

    #[test]
    fn ask_long_header_is_auto_fixed() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"AskUserQuestion","tool_input":{"questions":[
                {"question":"q","header":"Worktree switch","options":[
                    {"label":"A","description":"x"},{"label":"B","description":"y"}]}]}}),
            &fx.root,
        );
        assert_eq!(e.code, 0);
        let parsed: Value = serde_json::from_str(&e.stdout).unwrap();
        // "ask", not "allow" — an allow verdict pre-approves the question
        // prompt away and the human never gets to answer.
        assert_eq!(parsed["hookSpecificOutput"]["permissionDecision"], json!("ask"));
        assert!(parsed["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .is_some_and(|s| !s.is_empty()));
        assert_eq!(
            parsed["hookSpecificOutput"]["updatedInput"]["questions"][0]["header"],
            json!("Worktree sw…")
        );
    }

    #[test]
    fn ask_mixed_fixable_and_unfixable_denies() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"AskUserQuestion","tool_input":{"questions":[
                {"question":"q","header":"Worktree switch","options":[
                    {"label":"only-one","description":"x"}]}]}}),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("option"));
    }

    #[test]
    fn ask_valid_allowed() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            json!({"tool_name":"AskUserQuestion","tool_input":{"questions":[
                {"question":"q","header":"Approach","options":[
                    {"label":"A","description":"x"},{"label":"B","description":"y"}]}]}}),
            &fx.root,
        );
        assert_eq!(e.code, 0);
        assert!(e.stdout.is_empty());
    }

    #[test]
    fn ask_astral_header_counts_utf16_code_units_not_chars() {
        // 7 astral chars is 7 by char_len (≤12, would wrongly pass) but 14
        // UTF-16 code units (>12) — the length AskUserQuestion's own (JS)
        // schema validator actually enforces. Non-ASCII fixes delegate
        // rather than auto-truncate, so the guard must not silently allow.
        let fx = build_fixture("swarming", true);
        let header = "🐝".repeat(7);
        expect_delegate(
            json!({"tool_name":"AskUserQuestion","tool_input":{"questions":[
                {"question":"q","header":header,"options":[
                    {"label":"A","description":"x"},{"label":"B","description":"y"}]}]}}),
            &fx.root,
        );
    }

    // ── apply_patch matrix (rows 8-29) ─────────────────────────────────────

    #[test]
    fn apply_patch_add_safe_passes() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            patch("*** Begin Patch\n*** Add File: src/new-file.txt\n+hello world\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn apply_patch_update_state_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            patch("*** Begin Patch\n*** Update File: .bee/state.json\n@@\n-old\n+new\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee state"));
    }

    #[test]
    fn apply_patch_delete_backlog_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            patch("*** Begin Patch\n*** Delete File: .bee/backlog.jsonl\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee backlog add"));
    }

    #[test]
    fn apply_patch_move_safe_passes_and_denied_destination_denies() {
        let fx = build_fixture("swarming", true);
        let ok = expect_done(
            patch("*** Begin Patch\n*** Update File: src/old-name.txt\n*** Move to: src/new-name.txt\n@@\n-old\n+new\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(ok.code, 0);
        let deny = expect_done(
            patch("*** Begin Patch\n*** Update File: src/old-name.txt\n*** Move to: .bee/state.json\n@@\n-old\n+new\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("bee state"));
    }

    #[test]
    fn apply_patch_multi_target_one_denied_denies_whole() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(
            patch("*** Begin Patch\n*** Add File: src/a.txt\n+content\n*** Update File: src/b.txt\n@@\n-x\n+y\n*** Delete File: .bee/state.json\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee state"));
        let ok = expect_done(
            patch("*** Begin Patch\n*** Add File: src/a.txt\n+content\n*** Update File: src/b.txt\n@@\n-x\n+y\n*** Delete File: src/c.txt\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(ok.code, 0);
    }

    #[test]
    fn apply_patch_unicode_reserved_path_denied_with_holder() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "café/résumé.md", "otto", "other-cell", None, "lease");
        let e = expect_done(
            json!({"tool_name":"apply_patch","tool_input":{"input":"*** Begin Patch\n*** Add File: café/résumé.md\n+hello\n*** End Patch"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("café/résumé.md"));
        assert!(e.stderr.contains("otto"));
    }

    #[test]
    fn apply_patch_spaced_path_reserved_denied() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "my folder/file name.txt", "otto", "other-cell", None, "lease");
        let e = expect_done(
            json!({"tool_name":"apply_patch","tool_input":{"input":"*** Begin Patch\n*** Update File: my folder/file name.txt\n@@\n-a\n+b\n*** End Patch"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("my folder/file name.txt"));
    }

    #[test]
    fn apply_patch_escaped_space_path_resolves_and_denies() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "my\\ folder/escaped.txt", "otto", "other-cell", None, "lease");
        let e = expect_done(
            json!({"tool_name":"apply_patch","tool_input":{"input":"*** Begin Patch\n*** Add File: my\\ folder/escaped.txt\n+hi\n*** End Patch"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("otto"));
    }

    #[test]
    fn apply_patch_unprovable_shapes_deny() {
        let fx = build_fixture("swarming", true);
        for body in [
            "*** Begin Patch\n*** Add File\n+content\n*** End Patch",
            "*** Begin Patch\n*** Rename File: src/a.txt -> src/b.txt\n*** End Patch",
            "*** Begin Patch\n*** Add File:    \n+content\n*** End Patch",
            "*** Begin Patch\n*** Add File: ../../outside-repo.txt\n+x\n*** End Patch",
            // mixed provable+unprovable, both orders (rows 27/28/29)
            "*** Begin Patch\n*** Add File: src/safe-first.txt\n+hello\n*** Update File:    \n@@\n-old\n+new\n*** End Patch",
            "*** Begin Patch\n*** Update File:    \n@@\n-old\n+new\n*** Add File: src/safe-second.txt\n+hello\n*** End Patch",
            "*** Begin Patch\n*** Update File: src/valid.txt\n@@\n-old\n+new\n*** Update File: src/other.txt\n*** Move to: ../../outside-repo.txt\n@@\n-a\n+b\n*** End Patch",
        ] {
            let e = expect_done(patch(body), &fx.root);
            assert_eq!(e.code, 2, "should deny: {body}");
            assert!(e.stderr.contains("FIX"), "{body}");
            assert!(!e.gaps.is_empty(), "coverage gap logged: {body}");
        }
    }

    #[test]
    fn apply_patch_absolute_outside_denies() {
        let fx = build_fixture("swarming", true);
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("outside.txt");
        let body = format!(
            "*** Begin Patch\n*** Add File: {}\n+x\n*** End Patch",
            target.to_string_lossy()
        );
        let e = expect_done(patch(&body), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("FIX"));
    }

    #[test]
    fn apply_patch_no_envelope_fails_open_with_gap() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(patch("not a patch at all"), &fx.root);
        assert_eq!(e.code, 0);
        assert_eq!(e.gaps.len(), 1);
        assert_eq!(e.gaps[0].1, "applypatch-unparsed");
        assert!(e.gaps[0].2.contains("no canonical patch envelope"));
    }

    #[test]
    fn apply_patch_gate_policy_denies_source_allows_docs() {
        let fx = build_fixture("planning", false);
        let deny = expect_done(
            patch("*** Begin Patch\n*** Add File: src/feature.txt\n+new code\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("bee gate"));
        let ok = expect_done(
            patch("*** Begin Patch\n*** Add File: docs/notes.md\n+notes\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(ok.code, 0);
    }

    #[test]
    fn apply_patch_self_reservation_passes() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/mine.txt", "mel", "other-cell", None, "lease");
        let e = expect_done(
            json!({"tool_name":"apply_patch","tool_input":{"input":"*** Begin Patch\n*** Update File: src/mine.txt\n@@\n-a\n+b\n*** End Patch"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── reservations via Bash + intent advisory ────────────────────────────

    #[test]
    fn bash_write_to_reserved_file_denied() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/held.txt", "otto", "cell-1", None, "lease");
        let e = expect_done(
            json!({"tool_name":"Bash","tool_input":{"command":"printf x > \"src/held.txt\""},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee reservation conflict"));
        assert!(e.stderr.contains("otto holds \"src/held.txt\" (cell cell-1)"));
    }

    #[test]
    fn intent_reservation_allows_with_warning() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/*", "otto", "plan-cell", None, "intent");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
        let parsed: Value = serde_json::from_str(&e.stdout).unwrap();
        // Advisory only — no permission verdict rides along, so the write still
        // takes the host's ordinary permission flow.
        assert!(parsed["hookSpecificOutput"]["permissionDecision"].is_null());
        assert!(parsed["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .is_some_and(|s| s.contains("bee reservation intent")));
        let msg = parsed["systemMessage"].as_str().unwrap();
        assert!(msg.contains("bee reservation intent"));
        assert!(msg.contains("advisory only (kind: intent)"));
    }

    #[test]
    fn cross_session_hold_denies_and_corrupt_store_fails_closed() {
        let fx = build_fixture("swarming", true);
        add_live_session(&fx.root, "other");
        seed_lease(&fx.root, "src/held.txt", "otto", "cell-9", Some("other"), "lease");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/held.txt"},"session_id":"mine"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee cross-session hold"));
        assert!(e.stderr.contains("\"other\""));

        // corrupt projection store fails closed for a session-aware write
        std::fs::write(fx.root.join(".bee").join("reservations.json"), "{ not json").unwrap();
        let e2 = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/anything.txt"},"session_id":"mine"}),
            &fx.root,
        );
        assert_eq!(e2.code, 2);
        assert!(e2.stderr.contains("bee hold guard"));
        assert!(e2.stderr.contains("unreadable/corrupt"));
    }

    // ── linked-worktree matrix (rows 30-34) ────────────────────────────────

    struct Linked {
        _main_dir: tempfile::TempDir,
        _work_dir: tempfile::TempDir,
        main_root: PathBuf,
        work_root: PathBuf,
    }

    fn build_linked(valid: bool) -> Linked {
        let main_dir = tempfile::tempdir().unwrap();
        let work_dir = tempfile::tempdir().unwrap();
        let main_root = dunce::canonicalize(main_dir.path()).unwrap();
        let work_root = dunce::canonicalize(work_dir.path()).unwrap();
        let gitdir = main_root.join(".git").join("worktrees").join("fixture");
        std::fs::create_dir_all(&gitdir).unwrap();
        std::fs::write(
            work_root.join(".git"),
            format!("gitdir: {}\n", gitdir.to_string_lossy()),
        )
        .unwrap();
        if valid {
            std::fs::write(
                gitdir.join("gitdir"),
                format!("{}\n", work_root.join(".git").to_string_lossy()),
            )
            .unwrap();
        }
        std::fs::create_dir_all(main_root.join(".bee")).unwrap();
        std::fs::write(main_root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&main_root);
        write_state(
            &main_root,
            &json!({
                "phase": "swarming", "mode": "high-risk", "feature": "worktree-isolation",
                "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
            }),
        );
        std::fs::create_dir_all(work_root.join("src")).unwrap();
        Linked { _main_dir: main_dir, _work_dir: work_dir, main_root, work_root }
    }

    #[test]
    fn linked_worktree_reads_foreign_reservation_from_main_store() {
        let lx = build_linked(true);
        seed_lease(&lx.main_root, "src/held.txt", "otto", "other", None, "lease");
        let deny = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/held.txt"},"agent_name":"mel"}),
            &lx.work_root,
        );
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        assert!(deny.stderr.contains("otto"));
        let allow = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/held.txt"},"agent_name":"otto"}),
            &lx.work_root,
        );
        assert_eq!(allow.code, 0, "{}", allow.stderr);
    }

    #[test]
    fn linked_invalid_denies_before_mutation() {
        let lx = build_linked(false);
        for payload in [
            edit("src/new.txt"),
            bash("printf x > \"src/new.txt\""),
            patch("*** Begin Patch\n*** Add File: src/new.txt\n+x\n*** End Patch"),
        ] {
            let e = expect_done(payload, &lx.work_root);
            assert_eq!(e.code, 2);
            assert!(e.stderr.contains("WORKTREE_LINK_INVALID"));
        }
    }

    #[test]
    fn escape_rows_deny_and_contained_backslashes_pass() {
        let lx = build_linked(true);
        let traversal = expect_done(edit("../outside.txt"), &lx.work_root);
        assert_eq!(traversal.code, 2);
        assert_eq!(traversal.stderr, GENERIC_CONTAINMENT_MESSAGE);
        let win_traversal = expect_done(edit("..\\outside-win.txt"), &lx.work_root);
        assert_eq!(win_traversal.code, 2);
        let absolute_main = expect_done(
            edit(&lx.main_root.join("src").join("main-only.txt").to_string_lossy()),
            &lx.work_root,
        );
        assert_eq!(absolute_main.code, 2);
        // Contained Windows separators normalize into the reservation namespace.
        let contained = expect_done(edit("src\\nested\\new.txt"), &lx.work_root);
        assert_eq!(contained.code, 0, "{}", contained.stderr);
    }

    #[cfg(windows)]
    #[test]
    fn home_spellings_get_identical_deny() {
        let lx = build_linked(true);
        let home = std::env::var("USERPROFILE").unwrap();
        let absolute = format!("{}\\.claude\\bee-gmr1-probe.md", home);
        let spellings = [
            absolute.clone(),
            "~/.claude/bee-gmr1-probe.md".to_string(),
            "$HOME/.claude/bee-gmr1-probe.md".to_string(),
            "${HOME}/.claude/bee-gmr1-probe.md".to_string(),
        ];
        let baseline = expect_done(edit(&spellings[0]), &lx.work_root);
        assert_eq!(baseline.code, 2);
        for s in &spellings[1..] {
            let e = expect_done(edit(s), &lx.work_root);
            assert_eq!(e.code, 2, "{s}");
            assert_eq!(e.stderr, baseline.stderr, "{s}");
        }
    }

    #[test]
    fn bare_tilde_is_not_containment_denied() {
        let lx = build_linked(true);
        let e = expect_done(bash("rm -rf ~"), &lx.work_root);
        assert!(!e.stderr.contains("could not be canonically contained"), "{}", e.stderr);
    }

    // ── worktree-first (docs/specs/worktree-first.md §2) ───────────────────

    struct Wtf {
        _root_dir: tempfile::TempDir,
        _wt_dir: tempfile::TempDir,
        root: PathBuf,
        wt_root: PathBuf,
        id: String,
    }

    fn build_worktree_first(lane: &str, config_off: bool) -> Wtf {
        let fx_dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(fx_dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&root);
        let route_state = json!({
            "phase": "swarming", "mode": "standard", "feature": "demo",
            "route": { "class": "feature", "lane": lane, "flags": [], "product_files": 2, "rationale": null, "updated_at": ms_to_iso(now_ms()).unwrap() },
            "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
        });
        write_state(&root, &route_state);
        if config_off {
            std::fs::write(
                root.join(".bee").join("config.json"),
                "{\"worktree_first\":\"off\"}\n",
            )
            .unwrap();
        }
        let id = "wtf-demo-wt".to_string();
        let wt_dir = tempfile::tempdir().unwrap();
        let wt_root = dunce::canonicalize(wt_dir.path()).unwrap();
        let git_worktree_dir = root.join(".git").join("worktrees").join(&id);
        std::fs::create_dir_all(&git_worktree_dir).unwrap();
        std::fs::write(
            git_worktree_dir.join("gitdir"),
            format!("{}\n", wt_root.join(".git").to_string_lossy()),
        )
        .unwrap();
        std::fs::write(
            wt_root.join(".git"),
            format!("gitdir: {}\n", git_worktree_dir.to_string_lossy()),
        )
        .unwrap();
        std::fs::create_dir_all(root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            root.join(".bee").join("runtime").join("worktree-grants.json"),
            format!("{}\n", json!({ &id: true })),
        )
        .unwrap();
        std::fs::create_dir_all(wt_root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            wt_root.join(".bee").join("runtime").join("worktree-identity.json"),
            format!("{}\n", json!({ "feature": "demo", "created_at": ms_to_iso(now_ms()).unwrap() })),
        )
        .unwrap();
        std::fs::write(wt_root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&wt_root);
        write_state(&wt_root, &route_state);
        Wtf { _root_dir: fx_dir, _wt_dir: wt_dir, root, wt_root, id }
    }

    #[test]
    fn worktree_first_denies_main_source_write() {
        let wtf = build_worktree_first("standard", false);
        let e = expect_done(edit("src/app.js"), &wtf.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("worktree-first"));
        assert!(e.stderr.contains(&*wtf.wt_root.file_name().unwrap().to_string_lossy()));
        assert!(e.stderr.contains(&format!("bee worktree merge --id {}", wtf.id)));
        assert!(e.stderr.contains("worktree_first: \"off\""));
        assert!(e.stderr.contains("\"demo\"") && e.stderr.contains("\"standard\""));
        // Bash-extracted target too.
        let eb = expect_done(bash("printf x > src/app.js"), &wtf.root);
        assert_eq!(eb.code, 2);
    }

    #[test]
    fn worktree_first_exemptions_hold() {
        let wtf = build_worktree_first("standard", false);
        assert_eq!(expect_done(edit("docs/notes/plan.md"), &wtf.root).code, 0);
        assert_eq!(expect_done(edit("README.md"), &wtf.root).code, 0);
        // Inside the granted worktree the guard never fires.
        let inside = expect_done(edit("src/app.js"), &wtf.wt_root);
        assert_eq!(inside.code, 0, "{}", inside.stderr);
        // docs-lane route is exempt.
        let docs_lane = build_worktree_first("docs", false);
        assert_eq!(expect_done(edit("src/app.js"), &docs_lane.root).code, 0);
        // recorded off-switch disables the refusal.
        let off = build_worktree_first("standard", true);
        assert_eq!(expect_done(edit("src/app.js"), &off.root).code, 0);
        // corrupt grants registry fails OPEN.
        let corrupt = build_worktree_first("standard", false);
        std::fs::write(
            corrupt.root.join(".bee").join("runtime").join("worktree-grants.json"),
            "{ not json",
        )
        .unwrap();
        assert_eq!(expect_done(edit("src/app.js"), &corrupt.root).code, 0);
    }

    // ── scratch-shape guard (rows 35-45) ───────────────────────────────────

    #[test]
    fn scratch_shape_matrix() {
        let fx = build_fixture("swarming", true);
        let deny = |p: &str| {
            let e = expect_done(
                json!({"tool_name":"Write","tool_input":{"file_path":p,"content":"x\n"}}),
                &fx.root,
            );
            assert_eq!(e.code, 2, "expected deny for {p}: {}", e.stderr);
            assert!(e.stderr.contains(".bee/tmp/"), "{p}");
        };
        let allow = |p: &str| {
            let e = expect_done(
                json!({"tool_name":"Write","tool_input":{"file_path":p,"content":"x\n"}}),
                &fx.root,
            );
            assert_eq!(e.code, 0, "expected allow for {p}: {}", e.stderr);
        };
        deny(".bee/bin/.foo_stress_debug.sh");
        allow(".bee/tmp/th6/.foo_stress_debug.sh");
        allow("docs/history/tree-hygiene/reports/verdict-th6.md");
        // E2a: `.bee/cells/*.json` is CLI-owned now — the direct-edit guard
        // (ordered before scratch-shape in check_write) denies this row, so
        // it moved out of the scratch allow set. Kept as a deny expectation
        // (not dropped) to pin the direct-edit-before-scratch ordering.
        let cells = expect_done(
            json!({"tool_name":"Write","tool_input":{"file_path":".bee/cells/probe-th-6.json","content":"x\n"}}),
            &fx.root,
        );
        assert_eq!(cells.code, 2, "{}", cells.stderr);
        assert!(cells.stderr.contains("bee cells add/finish"), "{}", cells.stderr);
        allow(".claude-plugin/skills/bee-swarming/probe-render.json");
        allow("test/fixtures/sample.log");
        deny("results.log");
        deny(".rel9999_stress_debug.sh");
        deny("scripts/scratch-notes.tmp");
        deny("scripts/probe-foo.mjs");
        // decisions ledger append stays allowed
        let e = expect_done(bash("printf \"x\" >> .bee/decisions.jsonl"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── intake-gate git exemption (rows 47-56) ─────────────────────────────

    fn run_git(cwd: &Path, args: &[&str]) {
        let st = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(st.success(), "git {:?}", args);
    }

    fn build_git_fixture(phase: &str) -> Fx {
        let fx = build_fixture(phase, false);
        run_git(&fx.root, &["init", "-q"]);
        run_git(&fx.root, &["config", "user.email", "ige2@example.com"]);
        run_git(&fx.root, &["config", "user.name", "ige2 fixture"]);
        fx
    }

    fn stage_file(root: &Path, rel: &str) {
        let abs = root.join(rel.replace('/', &SEP.to_string()));
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, "x\n").unwrap();
        run_git(root, &["add", rel]);
    }

    #[test]
    fn git_readonly_allowed_at_terminal_phase() {
        let fx = build_git_fixture("idle");
        assert_eq!(expect_done(bash("git status"), &fx.root).code, 0);
        assert_eq!(expect_done(bash("git log --oneline -5"), &fx.root).code, 0);
    }

    #[test]
    fn git_commit_bookkeeping_exemption_and_source_refusal() {
        let bk = build_git_fixture("idle");
        stage_file(&bk.root, ".bee/cells/demo-1.json");
        stage_file(&bk.root, "docs/notes.md");
        let ok = expect_done(bash("git commit -m \"bookkeeping only\""), &bk.root);
        assert_eq!(ok.code, 0, "{}", ok.stderr);

        let src = build_git_fixture("idle");
        stage_file(&src.root, ".bee/cells/demo-1.json");
        stage_file(&src.root, "src/feature.js");
        let deny = expect_done(bash("git commit -m \"mixed change\""), &src.root);
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        assert!(deny.stderr.contains("intake gate"));
        assert!(deny.stderr.contains("src/feature.js"));
        // D3: the bookkeeping route is named before guards.idle_gate.
        let bk_idx = deny.stderr.find("commit or write bookkeeping").unwrap();
        let gate_idx = deny.stderr.find("guards.idle_gate").unwrap();
        assert!(bk_idx < gate_idx);
    }

    #[test]
    fn git_push_and_unknown_subcommands_refused() {
        let fx = build_git_fixture("idle");
        let push = expect_done(bash("git push origin main"), &fx.root);
        assert_eq!(push.code, 2);
        assert!(push.stderr.contains("never exempted"));
        let unk = expect_done(bash("git bisect start"), &fx.root);
        assert_eq!(unk.code, 2);
    }

    #[test]
    fn git_add_pathspec_exemption() {
        let fx = build_git_fixture("idle");
        std::fs::create_dir_all(fx.root.join("src")).unwrap();
        std::fs::write(fx.root.join("src").join("new.js"), "x\n").unwrap();
        let deny = expect_done(bash("git add src/new.js"), &fx.root);
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        std::fs::create_dir_all(fx.root.join("docs")).unwrap();
        std::fs::write(fx.root.join("docs").join("new.md"), "# x\n").unwrap();
        let ok = expect_done(bash("git add docs/new.md"), &fx.root);
        assert_eq!(ok.code, 0, "{}", ok.stderr);
    }

    #[test]
    fn git_commit_outside_terminal_phase_unaffected() {
        let fx = build_git_fixture("swarming");
        write_state(&fx.root, &swarming_state(true));
        stage_file(&fx.root, "src/feature.js");
        let e = expect_done(bash("git commit -m \"normal work\""), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── large-read guard (rows 57-64) ──────────────────────────────────────

    #[test]
    fn read_size_guard_matrix() {
        let fx = build_fixture("swarming", true);
        let big: String = (0..900).map(|i| format!("line {i}\n")).collect();
        std::fs::write(fx.root.join("big.md"), &big).unwrap();
        std::fs::write(fx.root.join("small.md"), "line 1\nline 2\nline 3\n").unwrap();
        std::fs::create_dir_all(fx.root.join("a-directory")).unwrap();

        let read = |ti: Value| json!({ "tool_name": "Read", "tool_input": ti });
        let deny = expect_done(read(json!({"file_path":"big.md"})), &fx.root);
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("big.md"));
        assert!(deny.stderr.contains("900"));
        assert!(deny.stderr.contains("800"));
        assert!(deny.stderr.contains("limit"));
        assert!(deny.stderr.contains("bee-extract"));

        assert_eq!(expect_done(read(json!({"file_path":"big.md","limit":50})), &fx.root).code, 0);
        assert_eq!(expect_done(read(json!({"file_path":"big.md","offset":100})), &fx.root).code, 0);
        assert_eq!(expect_done(read(json!({"file_path":"small.md"})), &fx.root).code, 0);
        assert_eq!(expect_done(read(json!({"file_path":"a-directory"})), &fx.root).code, 0);
        assert_eq!(expect_done(read(json!({"file_path":"does-not-exist.md"})), &fx.root).code, 0);

        // hooks.write-guard=false disables the whole guard.
        let disabled = build_fixture("swarming", true);
        std::fs::write(disabled.root.join("big.md"), &big).unwrap();
        std::fs::write(
            disabled.root.join(".bee").join("config.json"),
            format!("{}\n", serde_json::to_string_pretty(&json!({"hooks":{"write-guard":false}})).unwrap()),
        )
        .unwrap();
        assert_eq!(expect_done(read(json!({"file_path":"big.md"})), &disabled.root).code, 0);

        // custom threshold trips on a 3-line file.
        let custom = build_fixture("swarming", true);
        std::fs::write(custom.root.join("small.md"), "line 1\nline 2\nline 3\n").unwrap();
        std::fs::write(
            custom.root.join(".bee").join("config.json"),
            format!("{}\n", serde_json::to_string_pretty(&json!({"guards":{"max_read_lines":2}})).unwrap()),
        )
        .unwrap();
        let e = expect_done(read(json!({"file_path":"small.md"})), &custom.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("(threshold: 2)"));
    }

    #[test]
    fn secret_and_scout_reads_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(json!({"tool_name":"Read","tool_input":{"file_path":".env"}}), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee privacy guard"));
        assert!(e.stderr.contains("@@BEE_PRIVACY@@"));
        assert!(e.stderr.contains("@@END@@"));
        let s = expect_done(
            json!({"tool_name":"Read","tool_input":{"file_path":"node_modules/x/index.js"}}),
            &fx.root,
        );
        assert_eq!(s.code, 2);
        assert!(s.stderr.contains("bee scout guard"));
    }

    // ── shared nested-checkout guard (wcg rows 71/72/78/80) ────────────────

    #[test]
    fn nested_checkout_concurrent_denies_solo_allows() {
        let fx = build_fixture("swarming", true);
        let nested = fx.root.join("repo");
        std::fs::create_dir_all(nested.join(".git")).unwrap();
        std::fs::write(nested.join("foo.js"), "// nested plain\n").unwrap();

        // Solo (no live session): allowed.
        let solo = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"repo/foo.js"},"session_id":"me"}),
            &fx.root,
        );
        assert_eq!(solo.code, 0, "{}", solo.stderr);

        // Another live session: denied with the paved-road refusal.
        add_live_session(&fx.root, "other-live");
        let deny = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"repo/foo.js"},"session_id":"me"}),
            &fx.root,
        );
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        assert!(deny.stderr.contains("bee shared-checkout guard"));
        assert!(deny.stderr.contains("bee worktree new --with-companion"));

        // Bash branch is wired too.
        let bash_deny = expect_done(
            json!({"tool_name":"Bash","tool_input":{"command":"cp new.js repo/foo.js"},"session_id":"me"}),
            &fx.root,
        );
        assert_eq!(bash_deny.code, 2);
    }

    #[test]
    fn own_live_session_is_excluded() {
        let fx = build_fixture("swarming", true);
        let nested = fx.root.join("repo");
        std::fs::create_dir_all(nested.join(".git")).unwrap();
        std::fs::write(nested.join("foo.js"), "// nested plain\n").unwrap();
        add_live_session(&fx.root, "me");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"repo/foo.js"},"session_id":"me"}),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn companion_marker_present_delegates_on_containment_failure() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("companion-session.json"),
            "{\"sessionId\":\"s1\",\"worktreePath\":\"/x\",\"mountPath\":\"repo\"}\n",
        )
        .unwrap();
        expect_delegate(edit("../outside.txt"), &fx.root);
        // A contained target never consults the marker — stays native.
        let e = expect_done(edit("src/inside.txt"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn declared_memory_root_delegates_on_containment_failure() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("config.json"),
            "{\"guards\":{\"memory_root\":\"~/.claude/projects/x/memory\"}}\n",
        )
        .unwrap();
        expect_delegate(edit("../outside.txt"), &fx.root);
        let e = expect_done(edit("src/inside.txt"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── unknown phase / gate phases ────────────────────────────────────────

    #[test]
    fn unknown_phase_fails_closed() {
        let fx = build_fixture("bogus-phase", true);
        let e = expect_done(edit("src/app.js"), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee phase guard"));
        assert!(e.stderr.contains("bogus-phase"));
    }

    #[test]
    fn gated_phase_denies_source_until_execution_approved() {
        let fx = build_fixture("planning", false);
        let deny = expect_done(edit("src/app.js"), &fx.root);
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("bee gate"));
        assert!(deny.stderr.contains("execution"));
        let docs = expect_done(edit("docs/plan.md"), &fx.root);
        assert_eq!(docs.code, 0);
        let approved = build_fixture("planning", true);
        assert_eq!(expect_done(edit("src/app.js"), &approved.root).code, 0);
    }

    #[test]
    fn idle_intake_gate_denies_source_and_respects_opt_out() {
        let fx = build_fixture("idle", false);
        let deny = expect_done(edit("src/app.js"), &fx.root);
        assert_eq!(deny.code, 2);
        assert!(deny.stderr.contains("bee intake gate"));
        assert!(deny.stderr.contains("phase: idle"));
        std::fs::write(
            fx.root.join(".bee").join("config.json"),
            "{\"guards\":{\"idle_gate\":false}}\n",
        )
        .unwrap();
        assert_eq!(expect_done(edit("src/app.js"), &fx.root).code, 0);
    }

    // ── misc plumbing rows ─────────────────────────────────────────────────

    #[test]
    fn a_root_where_bee_is_not_installed_decides_nothing() {
        // The activation probe, re-keyed at cutover: `.bee/onboarding.json`
        // absent means "bee is not installed here" and the guard returns no
        // verdict — the same posture the `.mjs` took for a missing vendored
        // `.bee/bin/lib/state.mjs`. A `.bee/` directory alone is not an
        // install: a bare state.json (a fixture, a copy, a scratch dir) must
        // NOT switch the guard on.
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        write_state(&root, &swarming_state(true));
        let e = expect_done(edit(".bee/state.json"), &root);
        assert_eq!(e.code, 0);
        // …and the SAME root with the marker present denies, so the arm above
        // is proving the gate rather than a broken fixture.
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        let e = expect_done(edit(".bee/state.json"), &root);
        assert_eq!(e.code, 2, "installed root must decide: {}", e.stderr);
    }

    #[test]
    fn disabled_hook_exits_zero() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("config.json"),
            "{\"hooks\":{\"write-guard\":false}}\n",
        )
        .unwrap();
        let e = expect_done(edit(".bee/state.json"), &fx.root);
        assert_eq!(e.code, 0);
    }

    #[test]
    fn corrupt_state_json_warns_and_guards_from_defaults() {
        let fx = build_fixture("swarming", true);
        std::fs::write(fx.root.join(".bee").join("state.json"), "{broken").unwrap();
        // readJson's null fallback → defaultState(): phase idle, execution
        // gate false. The guard decides NATIVELY and, because it no longer
        // sees an approved execution gate, refuses the source edit.
        let e = expect_done(edit("src/app.js"), &fx.root);
        assert_eq!(e.code, 2, "stderr={}", e.stderr);
        // One warning per bad file, queued for flush() (never printed before
        // the native/delegate decision is final).
        let queued = take_corrupt_json_warnings();
        assert_eq!(queued.matches("could not parse JSON at").count(), 1);
        assert!(queued.ends_with("Using fallback; fix the file.\n"));
    }

    #[test]
    fn corrupt_state_json_does_not_leak_output_on_a_delegating_run() {
        // A run that still delegates (a `node -e` inline eval) must emit
        // nothing even though it read the corrupt state.json on the way.
        let fx = build_fixture("swarming", true);
        std::fs::write(fx.root.join(".bee").join("state.json"), "{broken").unwrap();
        expect_delegate(bash("node -e \"require('fs')\""), &fx.root);
        // Whatever was queued is dropped with the run — flush() never ran.
        take_corrupt_json_warnings();
    }

    #[test]
    fn plain_bash_is_native_and_allowed() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("echo hi"), &fx.root);
        assert_eq!(e.code, 0);
        assert!(e.stdout.is_empty() && e.stderr.is_empty());
    }

    #[test]
    fn bee_agent_name_env_prefix_is_parsed_from_command() {
        assert_eq!(agent_name_from_command("BEE_AGENT_NAME=mel printf x > f"), Some("mel".into()));
        assert_eq!(agent_name_from_command("BEE_AGENT_NAME=\"mel\" x"), Some("mel".into()));
        assert_eq!(agent_name_from_command("BEE_AGENT_NAME='mel' x"), Some("mel".into()));
        assert_eq!(agent_name_from_command("XBEE_AGENT_NAME=mel"), None);
        assert_eq!(agent_name_from_command("BEE_AGENT_NAME=\"mel x"), None);
    }

    // ── tokenizer decision-table cases ─────────────────────────────────────

    #[test]
    fn tokenizer_matches_mjs_semantics() {
        assert_eq!(tokenize("a b"), vec!["a", "b"]);
        // separators split even glued to text
        assert_eq!(tokenize("x 2>/dev/null; y"), vec!["x", "2>/dev/null", ";", "y"]);
        assert_eq!(tokenize("a&&b"), vec!["a", "&&", "b"]);
        // adjacent quoted/unquoted segments merge (bash word-splitting)
        assert_eq!(tokenize("'.bee/state'\".json\""), vec![".bee/state.json"]);
        // backslash escapes the next char literally
        assert_eq!(tokenize("a\\;b.txt"), vec!["a;b.txt"]);
        // unterminated quote runs to end
        assert_eq!(tokenize("\"a b"), vec!["a b"]);
    }

    #[test]
    fn quote_concat_cannot_evade_direct_edit_deny() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("printf x > '.bee/state'\".json\""), &fx.root);
        assert_eq!(e.code, 2);
        assert!(e.stderr.contains("bee state"));
    }

    // ── Node path-port vectors (generated from node:path win32) ────────────

    #[cfg(windows)]
    #[test]
    fn node_win32_path_vectors() {
        assert_eq!(np_relative("D:\\a\\b", "D:\\a\\b\\c.txt").unwrap(), "c.txt");
        assert_eq!(np_relative("D:\\a\\b", "D:\\a\\x").unwrap(), "..\\x");
        assert_eq!(np_relative("D:\\a", "C:\\b").unwrap(), "C:\\b");
        assert_eq!(np_relative("D:\\a\\b", "D:\\a").unwrap(), "..");
        assert_eq!(np_resolve2("D:\\a", "..\\..\\x").unwrap(), "D:\\x");
        assert_eq!(np_resolve2("D:\\a\\b", "src/new file.txt").unwrap(), "D:\\a\\b\\src\\new file.txt");
        assert_eq!(np_resolve2("D:\\a", "\\foo").unwrap(), "D:\\foo");
        assert_eq!(np_relative("d:\\A\\B", "D:\\a\\b\\C.txt").unwrap(), "C.txt");
        assert_eq!(np_relative("D:\\", "D:\\x").unwrap(), "x");
        assert_eq!(np_dirname("D:\\"), "D:\\");
        assert_eq!(np_resolve2("D:\\a", "café/résumé.md").unwrap(), "D:\\a\\café\\résumé.md");
        assert_eq!(np_relative("D:\\a\\b", "D:\\a\\bc").unwrap(), "..\\bc");
        assert_eq!(np_relative("D:\\a\\b", "D:\\a\\b").unwrap(), "");
        assert_eq!(np_relative("D:\\a\\b\\c", "D:\\a").unwrap(), "..\\..");
        assert_eq!(np_resolve2("D:\\a", "..").unwrap(), "D:\\");
        assert_eq!(np_resolve2("D:\\a", ".").unwrap(), "D:\\a");
        assert_eq!(np_basename("D:\\a\\b.txt"), "b.txt");
        assert_eq!(np_dirname("D:\\a\\b.txt"), "D:\\a");
        assert_eq!(np_resolve2("D:\\a", "my\\ folder/escaped.txt").unwrap(), "D:\\a\\my\\ folder\\escaped.txt");
        assert_eq!(np_relative("D:\\a", "D:\\a\\..b\\x").unwrap(), "..b\\x");
        assert_eq!(np_resolve2("D:\\a\\b", "..\\outside.txt").unwrap(), "D:\\a\\outside.txt");
        assert_eq!(np_resolve2("D:\\x", "").unwrap(), "D:\\x");
        assert_eq!(np_relative("D:\\a\\b\\", "D:\\a\\b\\c").unwrap(), "c");
        assert_eq!(np_relative("D:\\a\\b", "D:\\A\\B\\c").unwrap(), "c");
        assert_eq!(np_relative("D:\\x", "E:\\y").unwrap(), "E:\\y");
        assert!(np_resolve2("D:\\a", "C:foo").is_err()); // drive-relative → Nd
        assert!(np_resolve1("\\\\srv\\share\\x").is_err()); // UNC → Nd
    }

    #[test]
    fn paths_overlap_vectors() {
        assert!(paths_overlap("src/api", "src/api/router.ts"));
        assert!(paths_overlap("src/api/*", "src/api/router.ts"));
        assert!(paths_overlap("a", "a"));
        assert!(!paths_overlap("src/a", "src/b"));
        assert!(paths_overlap("*", "anything"));
        assert!(paths_overlap("my/ folder/escaped.txt", "my\\ folder/escaped.txt"));
    }

    #[test]
    fn glob_matcher_vectors() {
        let m = |g: &str, p: &str| glob_match(&glob_tokens(g), &p.chars().collect::<Vec<_>>());
        assert!(m("**/migrations/**", "db/migrations/001.sql"));
        assert!(m("**/migrations/**", "migrations/001.sql"));
        assert!(!m("**/migrations/**", "migrations"));
        assert!(m("package-lock.json", "package-lock.json"));
        assert!(m("**/package-lock.json", "pkg/a/package-lock.json"));
        assert!(!m("package-lock.json", "pkg/package-lock.json"));
        assert!(m("**/generated/**", "src/generated/client.ts"));
    }

    #[test]
    fn extract_bash_targets_vectors() {
        let t = extract_bash_targets("cat notes.txt >> .bee/backlog.jsonl");
        assert_eq!(t.paths, vec![".bee/backlog.jsonl"]);
        let t = extract_bash_targets("printf x 2>&1");
        assert!(t.paths.is_empty());
        let t = extract_bash_targets("rm -rf");
        assert!(t.paths.is_empty());
        assert!(t.broad_write);
        let t = extract_bash_targets("git add --all");
        assert!(t.broad_write);
        let t = extract_bash_targets("git add .bee/state.json");
        assert!(t.paths.is_empty()); // D8: staging a CLI-owned file is not a direct edit
        let t = extract_bash_targets("git mv a.txt b.txt");
        assert_eq!(t.paths, vec!["a.txt", "b.txt"]);
        let t = extract_bash_targets("sed -i \"s/a/b/\" f.txt");
        assert_eq!(t.paths, vec!["f.txt"]);
        let t = extract_bash_targets("node x.mjs > out.log && echo done");
        assert_eq!(t.paths, vec!["out.log"]);
    }

    // ── D9 glued/spaced separator matrix ──────────────────────────────────
    // R5 port of packages/bee/tests/test_guards_tokenizer.mjs. The existing
    // `tokenizer_matches_mjs_semantics` asserts only two of the five
    // separator forms; the whole point of that suite is that EVERY form in
    // the SEPARATORS set splits identically whether glued or spaced — a
    // glued `&` that failed to split used to garble the adjacent path and
    // leak command-verb tokens into the target list.

    #[test]
    fn d9_every_separator_form_splits_glued_and_spaced_alike() {
        for sep in [";", "&&", "&", "|", "||"] {
            let glued = extract_bash_targets(&format!("git add a.txt{sep}git add b.txt"));
            assert_eq!(
                glued.paths,
                vec!["a.txt", "b.txt"],
                "glued {sep:?} must not glue onto the adjacent token"
            );
            let spaced = extract_bash_targets(&format!("git add a.txt {sep} git add b.txt"));
            assert_eq!(spaced.paths, vec!["a.txt", "b.txt"], "spaced {sep:?}");
        }
    }

    #[test]
    fn d9_separator_lookalikes_are_not_boundaries() {
        // fd duplication is not a file write
        assert!(extract_bash_targets("echo hi 2>&1").paths.is_empty());
        assert!(extract_bash_targets("echo hi 1>&2").paths.is_empty());
        // a separator character inside quotes is data
        assert_eq!(extract_bash_targets("rm 'a&b.txt'").paths, vec!["a&b.txt"]);
        // a backslash-escaped separator stays part of the filename
        assert_eq!(extract_bash_targets("rm a\\;b.txt").paths, vec!["a;b.txt"]);
    }

    #[test]
    fn d8_staging_a_cli_owned_file_is_not_a_direct_edit_target() {
        // Chained form — the case the mixed command used to break.
        assert!(extract_bash_targets("git add .bee/backlog.jsonl && git commit -m \"stage\"")
            .paths
            .is_empty());
        assert!(extract_bash_targets("git add .bee/backlog.jsonl").paths.is_empty());
        // Control: an actual content mutation of the same file IS a target,
        // so the two assertions above are the D8 exemption firing rather
        // than the extractor going blind on `.bee/` paths.
        assert_eq!(
            extract_bash_targets("sed -i s/a/b/ .bee/backlog.jsonl").paths,
            vec![".bee/backlog.jsonl"]
        );
    }

    #[test]
    fn d12_companion_marker_is_direct_edit_denied() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(edit(".bee/companion-session.json"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee worktree new --with-companion"), "{}", e.stderr);
    }

    // ── isSharedNestedCheckoutTarget primitive, rows 72–77 ─────────────────
    // R5 port of test_write_guard.mjs rows 72–77, which call the primitive
    // DIRECTLY (the .mjs imports isSharedNestedCheckoutTarget from the
    // vendored guards.mjs). Rows 71/78/80/81 already run through the hook in
    // `nested_checkout_concurrent_denies_solo_allows`; these five are the
    // exclusion arms that the wired rows never reach.

    /// Probe (never a platform guess): attempt a real directory symlink in a
    /// scratch dir. win32 denies this without Developer Mode / elevation.
    fn symlink_capable() -> bool {
        use std::sync::OnceLock;
        static CAP: OnceLock<bool> = OnceLock::new();
        *CAP.get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("t");
            std::fs::create_dir(&target).unwrap();
            symlink_dir(&target, &dir.path().join("l")).is_ok()
        })
    }

    fn symlink_dir(target: &Path, link: &Path) -> std::io::Result<()> {
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(target, link)
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link)
        }
    }

    /// A bare root with `.bee/` — the primitive needs no state.json, only the
    /// sessions dir it reads through `is_concurrent_mode`.
    fn wcg_root() -> Fx {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        Fx { _dir: dir, root }
    }

    fn flagged(root: &Path, target: &Path) -> bool {
        is_shared_nested_checkout_target(
            &root.to_string_lossy(),
            &target.to_string_lossy(),
            None,
            None,
        )
        .expect("primitive must decide natively")
    }

    #[test]
    fn row72_71_plain_nested_checkout_flags_only_when_concurrent() {
        let fx = wcg_root();
        let nested = fx.root.join("repo");
        std::fs::create_dir_all(nested.join(".git")).unwrap();
        std::fs::write(nested.join("foo.js"), "// nested plain\n").unwrap();
        let target = nested.join("foo.js");

        // row72: solo — the D6 backward-compat no-op.
        assert!(!flagged(&fx.root, &target), "row72: solo must not flag");
        // row71: another live session — STR65's unguarded incident shape.
        add_live_session(&fx.root, "other-live");
        assert!(flagged(&fx.root, &target), "row71: concurrent must flag");
    }

    #[test]
    fn row73_registered_submodule_is_never_flagged() {
        // The exclusion keys off `.gitmodules` registration, not the `.git`
        // shape, so the fixture only needs the registration the primitive
        // actually reads (Node's row73 spends a real `git submodule add` to
        // produce exactly these two artifacts).
        let fx = wcg_root();
        let nested = fx.root.join("repo");
        std::fs::create_dir_all(nested.join(".git")).unwrap();
        std::fs::write(nested.join("foo.js"), "// submodule file\n").unwrap();
        std::fs::write(
            fx.root.join(".gitmodules"),
            "[submodule \"repo\"]\n\tpath = repo\n\turl = ../remote.git\n",
        )
        .unwrap();
        add_live_session(&fx.root, "other-live");
        assert!(
            !flagged(&fx.root, &nested.join("foo.js")),
            "row73: a registered submodule is excluded even when concurrent"
        );

        // Control: the SAME tree without the registration IS flagged — proves
        // the assertion above is the exclusion firing, not a vacuous pass.
        std::fs::remove_file(fx.root.join(".gitmodules")).unwrap();
        assert!(flagged(&fx.root, &nested.join("foo.js")));

        // A .gitmodules registering some OTHER path does not excuse this one.
        std::fs::write(
            fx.root.join(".gitmodules"),
            "[submodule \"vendor\"]\n\tpath = vendor/lib\n",
        )
        .unwrap();
        assert!(flagged(&fx.root, &nested.join("foo.js")));
    }

    #[test]
    fn rows74_77_verified_companion_mount_exclusions() {
        const CAP: &str =
            "symlink creation denied — needs Developer Mode or an elevated shell";
        if !symlink_capable() {
            for row in [
                "row75: verified companion mount, solo, is NOT flagged",
                "row74: verified companion mount + concurrent session IS flagged",
                "row76: a marker whose worktreePath mismatches the live symlink is NOT flagged",
                "row77: a symlink mount with NO marker is NOT flagged by the primitive",
            ] {
                eprintln!("SKIP (env-limited: {CAP}) — {row}");
            }
            return;
        }

        let mount_dir = tempfile::tempdir().unwrap();
        let mount_target = dunce::canonicalize(mount_dir.path()).unwrap();
        std::fs::create_dir_all(mount_target.join(".git")).unwrap();
        std::fs::write(mount_target.join("foo.js"), "// companion file\n").unwrap();

        let write_marker = |root: &Path, worktree: &Path| {
            std::fs::write(
                root.join(".bee").join("companion-session.json"),
                format!(
                    "{}\n",
                    serde_json::to_string_pretty(&json!({
                        "sessionId": "s1",
                        "worktreePath": worktree.to_string_lossy(),
                        "mountPath": "repo"
                    }))
                    .unwrap()
                ),
            )
            .unwrap();
        };

        // rows 75 / 74 — verified marker, solo then concurrent.
        let verified = wcg_root();
        symlink_dir(&mount_target, &verified.root.join("repo")).unwrap();
        write_marker(&verified.root, &mount_target);
        let target = verified.root.join("repo").join("foo.js");
        assert!(!flagged(&verified.root, &target), "row75: solo is a no-op");
        add_live_session(&verified.root, "other-live");
        assert!(
            flagged(&verified.root, &target),
            "row74: a verified mount reachable by another live session IS flagged"
        );

        // row76 — the marker's declared worktreePath does not resolve to the
        // live symlink, so verification fails and the primitive stays quiet.
        let other_dir = tempfile::tempdir().unwrap();
        let other_real = dunce::canonicalize(other_dir.path()).unwrap();
        let mismatch = wcg_root();
        symlink_dir(&mount_target, &mismatch.root.join("repo")).unwrap();
        write_marker(&mismatch.root, &other_real);
        add_live_session(&mismatch.root, "other-live");
        assert!(
            !flagged(&mismatch.root, &mismatch.root.join("repo").join("foo.js")),
            "row76: verification failure is not a flag"
        );

        // row77 — a symlink mount with no marker at all: containment's job,
        // not this primitive's.
        let no_marker = wcg_root();
        symlink_dir(&mount_target, &no_marker.root.join("repo")).unwrap();
        add_live_session(&no_marker.root, "other-live");
        assert!(
            !flagged(&no_marker.root, &no_marker.root.join("repo").join("foo.js")),
            "row77: an unmarked symlink mount is not flagged by the primitive"
        );
    }

    // ── gh-1: harness-owned surface containment allowlist (guard-hardening
    // E1) — writes whose RESOLVED target sits under <home>/.claude/projects/
    // or <system-temp>/claude/ are exempt from the outside-root containment
    // deny, and only that deny. Bases are injected (never env-mutated); the
    // fixture's fake home/temp are plain tempdirs.

    fn expect_done_with_roots(payload: Value, cwd: &Path, harness: &HarnessRoots) -> Emit {
        let mut body = match payload {
            Value::Object(m) => m,
            _ => panic!("payload must be an object"),
        };
        body.insert("cwd".into(), Value::String(cwd.to_string_lossy().into_owned()));
        let stdin = jsjson::stringify(&Value::Object(body));
        let ctx = read_hook_context(HOOK_NAME, &[], &stdin);
        match run_native_with_roots(&ctx, harness) {
            Ok(e) => e,
            Err(_) => panic!("expected a native verdict, got Delegate"),
        }
    }

    struct HarnessFx {
        fx: Fx,
        _home_dir: tempfile::TempDir,
        _temp_dir: tempfile::TempDir,
        home: PathBuf,
        temp: PathBuf,
        roots: HarnessRoots,
    }

    fn build_harness_fixture() -> HarnessFx {
        let fx = build_fixture("swarming", true);
        let home_dir = tempfile::tempdir().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let home = dunce::canonicalize(home_dir.path()).unwrap();
        let temp = dunce::canonicalize(temp_dir.path()).unwrap();
        let roots = HarnessRoots::from_bases(Some(home.clone()), Some(temp.clone()));
        assert_eq!(roots.roots.len(), 2, "both injected bases must resolve");
        HarnessFx { fx, _home_dir: home_dir, _temp_dir: temp_dir, home, temp, roots }
    }

    #[test]
    fn gh1_harness_memory_write_is_exempt_from_containment() {
        let hx = build_harness_fixture();
        let target =
            hx.home.join(".claude").join("projects").join("x").join("memory").join("f.md");
        let e = expect_done_with_roots(edit(&target.to_string_lossy()), &hx.fx.root, &hx.roots);
        assert_eq!(e.code, 0, "{}", e.stderr);

        // The linked-worktree shape threads the same exemption.
        let lx = build_linked(true);
        let wt = expect_done_with_roots(edit(&target.to_string_lossy()), &lx.work_root, &hx.roots);
        assert_eq!(wt.code, 0, "{}", wt.stderr);
    }

    #[test]
    fn gh1_harness_scratchpad_write_is_exempt_for_write_and_bash() {
        let hx = build_harness_fixture();
        let target = hx.temp.join("claude").join("sess").join("scratchpad").join("f.txt");
        let w = expect_done_with_roots(
            json!({"tool_name":"Write","tool_input":{"file_path":target.to_string_lossy(),"content":"x\n"}}),
            &hx.fx.root,
            &hx.roots,
        );
        assert_eq!(w.code, 0, "{}", w.stderr);

        // Bash-extracted target (forward slashes: the tokenizer treats a bare
        // backslash as an escape, exactly like the shell it models).
        let bash_target = target.to_string_lossy().replace('\\', "/");
        let b = expect_done_with_roots(
            bash(&format!("printf x > \"{bash_target}\"")),
            &hx.fx.root,
            &hx.roots,
        );
        assert_eq!(b.code, 0, "{}", b.stderr);
    }

    #[test]
    fn gh1_sibling_worktree_target_stays_denied_despite_allowlist() {
        let lx = build_linked(true);
        let home_dir = tempfile::tempdir().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let roots = HarnessRoots::from_bases(
            Some(dunce::canonicalize(home_dir.path()).unwrap()),
            Some(dunce::canonicalize(temp_dir.path()).unwrap()),
        );
        let e = expect_done_with_roots(
            edit(&lx.main_root.join("src").join("main-only.txt").to_string_lossy()),
            &lx.work_root,
            &roots,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("could not be canonically contained"), "{}", e.stderr);
    }

    #[test]
    fn gh1_traversal_spelling_resolving_into_allowlist_root_is_exempt() {
        let hx = build_harness_fixture();

        // Absolute spelling with a `..` hop that still RESOLVES under the
        // memory root — judged by its resolved location (E1).
        let abs_spelling = format!(
            "{sep}projects{sep}..{sep}projects{sep}x{sep}esc.md",
            sep = SEP
        );
        let abs_spelling =
            format!("{}{}", hx.home.join(".claude").to_string_lossy(), abs_spelling);
        let a = expect_done_with_roots(edit(&abs_spelling), &hx.fx.root, &hx.roots);
        assert_eq!(a.code, 0, "{}", a.stderr);

        // Relative traversal out of the repo that resolves into the fake home
        // (both tempdirs share a parent, asserted so a layout change fails
        // loudly instead of testing nothing).
        assert_eq!(hx.fx.root.parent(), hx.home.parent(), "fixture layout precondition");
        let rel_spelling = format!(
            "..{sep}{home_base}{sep}.claude{sep}projects{sep}x{sep}esc.md",
            sep = SEP,
            home_base = hx.home.file_name().unwrap().to_string_lossy()
        );
        let r = expect_done_with_roots(edit(&rel_spelling), &hx.fx.root, &hx.roots);
        assert_eq!(r.code, 0, "{}", r.stderr);
    }

    #[test]
    fn gh1_unrelated_out_of_root_target_stays_denied() {
        let hx = build_harness_fixture();
        let outside = tempfile::tempdir().unwrap();
        let target = dunce::canonicalize(outside.path()).unwrap().join("elsewhere.txt");
        let e = expect_done_with_roots(edit(&target.to_string_lossy()), &hx.fx.root, &hx.roots);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert_eq!(e.stderr, GENERIC_CONTAINMENT_MESSAGE);

        // Bash shape keeps its deny too.
        let bash_target = target.to_string_lossy().replace('\\', "/");
        let b = expect_done_with_roots(
            bash(&format!("printf x > \"{bash_target}\"")),
            &hx.fx.root,
            &hx.roots,
        );
        assert_eq!(b.code, 2, "{}", b.stderr);
        assert_eq!(b.stderr, GENERIC_BASH_CONTAINMENT_MESSAGE);
    }

    #[test]
    fn gh1_unresolvable_harness_roots_leave_the_deny_intact() {
        let hx = build_harness_fixture();
        let target = hx.home.join(".claude").join("projects").join("x").join("f.md");

        // No bases at all → the allowlist contributes nothing.
        let none = HarnessRoots::from_bases(None, None);
        assert!(none.roots.is_empty());
        let e = expect_done_with_roots(edit(&target.to_string_lossy()), &hx.fx.root, &none);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert_eq!(e.stderr, GENERIC_CONTAINMENT_MESSAGE);

        // Empty-path bases are unresolvable, never a wildcard.
        let empty = HarnessRoots::from_bases(Some(PathBuf::new()), Some(PathBuf::new()));
        assert!(empty.roots.is_empty());
        let e2 = expect_done_with_roots(edit(&target.to_string_lossy()), &hx.fx.root, &empty);
        assert_eq!(e2.code, 2, "{}", e2.stderr);

        // Relative (non-absolute) bases fail closed too.
        let relative =
            HarnessRoots::from_bases(Some(PathBuf::from("not-abs")), Some(PathBuf::from("x")));
        assert!(relative.roots.is_empty());
    }
