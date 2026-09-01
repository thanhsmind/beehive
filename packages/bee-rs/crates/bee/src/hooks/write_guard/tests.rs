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
        // bah-2: `--layer` joined the declared required set when
        // `backlog.add`'s registry entry was corrected to match its handler,
        // so this call needs it to stay a well-shaped one. The paired
        // malformed case is rows5c_5d below.
        let b = expect_done(
            bash("node .bee/bin/bee_backlog.mjs add --type bug --title \"x\" --severity P2 --layer cli"),
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
            "node .bee/bin/bee.mjs cells cap --id demo-1 --outcome done --report \"cargo test -p bee — green:unit — touched close.rs\"",
            ".bee/bin/bee cells cap --id demo-1 --outcome done --report \"cargo test -p bee — green:unit — touched close.rs\"",
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

    #[test]
    fn wrapper_hidden_node_inline_eval_still_delegates() {
        // detectors.rs routed through tokenize_deep too: a wrapper must not
        // hide an inline-eval node call from this detector either.
        let fx = build_fixture("swarming", true);
        expect_delegate(bash("sh -c \"node -e 'x'\""), &fx.root);
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
        // trun-5: retargeted from "docs/notes.md" (now refused, same gated-
        // phase split as `gated_phase_docs_outside_history_now_refuses`) to
        // "docs/history/", which is the gated-phase list's allowed docs path.
        let ok = expect_done(
            patch("*** Begin Patch\n*** Add File: docs/history/some-feature/notes.md\n+notes\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(ok.code, 0, "{}", ok.stderr);
        let still_denies = expect_done(
            patch("*** Begin Patch\n*** Add File: docs/notes.md\n+notes\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(still_denies.code, 2, "{}", still_denies.stdout);
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

    // D1: cp checks only the destination operand — a source operand must
    // never surface in a refusal.
    #[test]
    fn cp_under_a_gated_phase_refuses_naming_the_destination_only() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/dst.txt", "otto", "cell-1", None, "lease");
        let e = expect_done(
            json!({"tool_name":"Bash","tool_input":{"command":"cp src/held_source.txt src/dst.txt"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee reservation conflict"));
        assert!(e.stderr.contains("otto holds \"src/dst.txt\" (cell cell-1)"));
        assert!(!e.stderr.contains("held_source.txt"), "{}", e.stderr);
    }

    // P1-1: mv unlinks its source, so mv checks EVERY operand — a held
    // destination still refuses even though the (unheld) source is also now
    // extracted, and it names only the destination because that is the one
    // path actually held.
    #[test]
    fn mv_under_a_gated_phase_refuses_naming_the_destination_only() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/dst.txt", "otto", "cell-1", None, "lease");
        let e = expect_done(
            json!({"tool_name":"Bash","tool_input":{"command":"mv src/held_source.txt src/dst.txt"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee reservation conflict"));
        assert!(e.stderr.contains("otto holds \"src/dst.txt\" (cell cell-1)"));
        assert!(!e.stderr.contains("held_source.txt"), "{}", e.stderr);
    }

    // P1-1: mv of a RESERVATION-HELD SOURCE now raises the conflict too —
    // before this cell the source was dropped from extraction entirely, so
    // this shape used to allow silently even though mv unlinks the source.
    #[test]
    fn mv_of_a_reservation_held_source_raises_the_conflict() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/held_source.txt", "otto", "cell-1", None, "lease");
        let e = expect_done(
            json!({"tool_name":"Bash","tool_input":{"command":"mv src/held_source.txt safe_dst.txt"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee reservation conflict"));
        assert!(e.stderr.contains("otto holds \"src/held_source.txt\" (cell cell-1)"), "{}", e.stderr);
    }

    // P1-1: mv of a CLI-owned file is a direct-edit-guard hit again — before
    // this cell, mv's source was never extracted so the guard never saw it.
    // (An in-worktree destination is used rather than an absolute /tmp path
    // so the containment check — a separate, pre-existing denial path for a
    // target outside the worktree — never masks the direct-edit denial this
    // test pins.)
    #[test]
    fn mv_of_a_cli_owned_state_file_denies_again() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("mv .bee/state.json elsewhere.txt"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee direct-edit guard"), "{}", e.stderr);
        assert!(e.stderr.contains(".bee/state.json"), "{}", e.stderr);
    }

    #[test]
    fn cp_with_target_directory_flag_refuses_naming_the_directory_only() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "held_dir", "otto", "cell-1", None, "lease");
        let e = expect_done(
            json!({"tool_name":"Bash","tool_input":{"command":"cp a.txt b.txt -t held_dir"},"agent_name":"mel"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("otto holds \"held_dir\" (cell cell-1)"));
        assert!(!e.stderr.contains("a.txt") && !e.stderr.contains("b.txt"), "{}", e.stderr);
    }

    // A compound command ending in a null redirect must never be refused
    // because of that redirect token — the extractor must not turn it into
    // a bogus path operand.
    #[test]
    fn null_redirect_tail_never_becomes_a_refusable_target() {
        let fx = build_fixture("swarming", true);
        seed_lease(&fx.root, "src/held.txt", "otto", "cell-1", None, "lease");
        for cmd in [
            "cp src/held.txt safe.txt 2>/dev/null",
            "rm safe.txt 2>>/dev/null",
            "mv src/other.txt safe.txt &>/dev/null",
        ] {
            let e = expect_done(bash(cmd), &fx.root);
            // None of these name the reserved file as a destination, so the
            // guard must not deny on the redirect tail alone.
            assert!(!e.stderr.contains("bee reservation conflict"), "{cmd}: {}", e.stderr);
        }
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

    // ── gc-2: concurrent-worker git guard, Unresolved arm ───────────────────

    #[test]
    fn concurrent_tree_guard_unresolved_arm_names_the_reservation_store() {
        let fx = build_fixture("swarming", true);
        std::fs::write(fx.root.join(".bee").join("reservations.json"), "{ not json").unwrap();
        let e = expect_done(bash("git reset --hard"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee concurrent-worker git guard"));
        // Names the actual file and the direct remedy, not just the
        // temp-index recipe (which stays reserved for the `count > 1` arm).
        assert!(e.stderr.contains(".bee/reservations.json"));
        assert!(e.stderr.contains("inspect/restore the reservation store"));
        // Also names the path-scoped-commit escape the `count > 1` arm
        // already carries, so a solo session is not only told to repair a
        // file — it is also told it can land its work directly.
        assert!(e.stderr.contains("A genuinely path-scoped `git commit -- <your paths>` is allowed too."));
    }

    // ── gc-2 / wgg-2: the guard inside a GRANTED worktree ──────────────────
    //
    // The wave the guard was built for runs INSIDE a granted worktree, and
    // that is the one place it never fired: the worktree's own reservation
    // store is empty (the orchestrator writes reservations at the control
    // root) and the wave's sessions are stamped "main", so both halves of the
    // old count resolved to zero and `count > 1` was unreachable
    // (docs/history/wave-guard-gaps/CONTEXT.md, "Gap 2"). The mirrored-holds
    // ledger at the control root is the record that crosses both checkouts.

    /// One mirrored hold row, appended to the control root's ledger — the same
    /// shape `bee reservations reserve` writes (verbs/reservations/reserve.rs).
    fn seed_mirrored_hold(main_root: &Path, holder: &str, cell: &str, path: &str, session: Option<&str>) {
        let dir = main_root.join(".bee").join("runtime");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("cross-worktree-holds.json");
        let mut store: Value = std::fs::read_to_string(&file)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .unwrap_or_else(|| json!({ "holds": [] }));
        store["holds"].as_array_mut().unwrap().push(json!({
            "path": path,
            "holder": holder,
            "feature": "demo",
            "session": session.map(Value::from).unwrap_or(Value::Null),
            "cell": cell,
            "ttl_seconds": 3600,
            "mirrored_at": ms_to_iso(now_ms()).unwrap(),
            "released_at": Value::Null,
        }));
        std::fs::write(&file, format!("{}\n", serde_json::to_string_pretty(&store).unwrap())).unwrap();
    }

    /// A live session record stamped with the workspace it runs in — the field
    /// `session_workspace_id` reads. `add_live_session` leaves it absent, which
    /// reads as "main".
    fn add_live_session_in_workspace(root: &Path, id: &str, workspace_id: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        let now = ms_to_iso(now_ms()).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": id, "started_at": now, "last_heartbeat": now,
                    "workspace_id": workspace_id
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    /// The reproduction from CONTEXT.md, as a test: sibling workers live in a
    /// granted worktree must deny a bare `git commit`, and a genuinely
    /// path-scoped one must still pass.
    #[test]
    fn concurrent_tree_guard_counts_siblings_inside_a_granted_worktree() {
        let wtf = build_worktree_first("swarming", "standard", false);
        for cell in ["c-1", "c-2", "c-3"] {
            seed_mirrored_hold(&wtf.root, &wtf.id, cell, &format!("src/{cell}.txt"), Some("s-orch"));
        }
        let deny = expect_done(bash("git commit -m wip"), &wtf.wt_root);
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        assert!(deny.stderr.contains("bee concurrent-worker git guard"), "{}", deny.stderr);
        assert!(deny.stderr.contains("3 workers are live in this checkout"), "{}", deny.stderr);

        // Constraint: the escape the refusal names must actually work.
        let scoped = expect_done(bash("git commit -m wip -- src/mine.txt"), &wtf.wt_root);
        assert_eq!(scoped.code, 0, "{}", scoped.stderr);
    }

    /// Deny-more only. The very same ledger read from the MAIN checkout is
    /// worth nothing to it — those holds belong to the worktree's index, not
    /// main's — so main's verdict is unchanged.
    #[test]
    fn concurrent_tree_guard_leaves_the_main_checkout_verdict_unchanged() {
        let wtf = build_worktree_first("swarming", "standard", false);
        for cell in ["c-1", "c-2", "c-3"] {
            seed_mirrored_hold(&wtf.root, &wtf.id, cell, &format!("src/{cell}.txt"), Some("s-orch"));
        }
        let e = expect_done(bash("git commit -m wip"), &wtf.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
        // A ledger main cannot even parse must not start denying in main
        // either — the new fail-safe arm belongs to the worktree branch alone.
        std::fs::write(
            wtf.root.join(".bee").join("runtime").join("cross-worktree-holds.json"),
            "{ not json",
        )
        .unwrap();
        let corrupt = expect_done(bash("git commit -m wip"), &wtf.root);
        assert_eq!(corrupt.code, 0, "{}", corrupt.stderr);
    }

    /// The constraint that outranks the fix: ONE worker alone in its own
    /// granted worktree keeps committing. Its single cell is visible three
    /// times over — a lease in the worktree store, a live session stamped to
    /// the worktree, and the mirrored hold — and all three are the SAME
    /// worker, so the count stays 1.
    #[test]
    fn concurrent_tree_guard_never_blocks_a_solo_worker_in_its_own_worktree() {
        let wtf = build_worktree_first("swarming", "standard", false);
        seed_mirrored_hold(&wtf.root, &wtf.id, "c-1", "src/solo.txt", Some("s-solo"));
        add_live_session_in_workspace(&wtf.root, "s-solo", &wtf.id);
        seed_lease(&wtf.wt_root, "src/solo.txt", "wk-solo", "c-1", Some("s-solo"), "lease");
        let e = expect_done(bash("git commit -m wip"), &wtf.wt_root);
        assert_eq!(e.code, 0, "{}", e.stderr);
        // Same for the other whole-tree verbs the classifier catches.
        for cmd in ["git stash", "git reset --hard", "git revert HEAD"] {
            let solo = expect_done(bash(cmd), &wtf.wt_root);
            assert_eq!(solo.code, 0, "{cmd}: {}", solo.stderr);
        }
    }

    /// Fail-safe, same shape the reservation store already takes: an
    /// unreadable ledger is "more than one worker", never a silent zero.
    #[test]
    fn concurrent_tree_guard_fails_safe_on_an_unparseable_holds_ledger() {
        let wtf = build_worktree_first("swarming", "standard", false);
        std::fs::create_dir_all(wtf.root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            wtf.root.join(".bee").join("runtime").join("cross-worktree-holds.json"),
            "{ not json",
        )
        .unwrap();
        let e = expect_done(bash("git commit -m wip"), &wtf.wt_root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee concurrent-worker git guard"), "{}", e.stderr);
        assert!(e.stderr.contains("cross-worktree-holds.json"), "{}", e.stderr);
        assert!(
            e.stderr.contains("treated as more than one worker"),
            "{}",
            e.stderr
        );
    }

    /// A hold that names no cell names no worker: it is skipped rather than
    /// counted, because counting it could push a lone worker to two.
    #[test]
    fn concurrent_tree_guard_ignores_cell_less_and_foreign_holds() {
        let wtf = build_worktree_first("swarming", "standard", false);
        // Same worktree, no cell.
        seed_mirrored_hold(&wtf.root, &wtf.id, "", "src/a.txt", Some("s-1"));
        seed_mirrored_hold(&wtf.root, &wtf.id, "   ", "src/b.txt", Some("s-2"));
        // Another checkout's holds — a different index entirely.
        seed_mirrored_hold(&wtf.root, "main", "c-9", "src/c.txt", Some("s-3"));
        seed_mirrored_hold(&wtf.root, "some-other-wt", "c-8", "src/d.txt", Some("s-4"));
        // One real sibling — still one worker, so no denial.
        seed_mirrored_hold(&wtf.root, &wtf.id, "c-1", "src/e.txt", Some("s-5"));
        let e = expect_done(bash("git commit -m wip"), &wtf.wt_root);
        assert_eq!(e.code, 0, "{}", e.stderr);
        // Add a second real sibling and the guard fires.
        seed_mirrored_hold(&wtf.root, &wtf.id, "c-2", "src/f.txt", Some("s-6"));
        let deny = expect_done(bash("git commit -m wip"), &wtf.wt_root);
        assert_eq!(deny.code, 2, "{}", deny.stderr);
        assert!(deny.stderr.contains("2 workers are live in this checkout"), "{}", deny.stderr);
    }

    /// Two holds, one cell: one worker reserving two paths is still one
    /// worker, and a released or expired hold is nobody.
    #[test]
    fn concurrent_tree_guard_counts_workers_not_hold_rows() {
        let wtf = build_worktree_first("swarming", "standard", false);
        seed_mirrored_hold(&wtf.root, &wtf.id, "c-1", "src/one.txt", Some("s-1"));
        seed_mirrored_hold(&wtf.root, &wtf.id, "c-1", "src/two.txt", Some("s-1"));
        assert_eq!(expect_done(bash("git commit -m wip"), &wtf.wt_root).code, 0);

        // A released row and an expired row are both inactive.
        let file = wtf.root.join(".bee").join("runtime").join("cross-worktree-holds.json");
        seed_mirrored_hold(&wtf.root, &wtf.id, "c-2", "src/gone.txt", Some("s-2"));
        seed_mirrored_hold(&wtf.root, &wtf.id, "c-3", "src/old.txt", Some("s-3"));
        let mut store: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        store["holds"][2]["released_at"] = json!(ms_to_iso(now_ms()).unwrap());
        store["holds"][3]["mirrored_at"] = json!(ms_to_iso(now_ms() - 7200.0 * 1000.0).unwrap());
        std::fs::write(&file, format!("{}\n", serde_json::to_string_pretty(&store).unwrap())).unwrap();
        let e = expect_done(bash("git commit -m wip"), &wtf.wt_root);
        assert_eq!(e.code, 0, "{}", e.stderr);
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

    fn build_worktree_first(phase: &str, lane: &str, config_off: bool) -> Wtf {
        let fx_dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(fx_dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&root);
        let route_state = json!({
            "phase": phase, "mode": "standard", "feature": "demo",
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
        let wtf = build_worktree_first("swarming", "standard", false);
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
        let wtf = build_worktree_first("swarming", "standard", false);
        assert_eq!(expect_done(edit("docs/notes/plan.md"), &wtf.root).code, 0);
        assert_eq!(expect_done(edit("README.md"), &wtf.root).code, 0);
        // Inside the granted worktree the guard never fires.
        let inside = expect_done(edit("src/app.js"), &wtf.wt_root);
        assert_eq!(inside.code, 0, "{}", inside.stderr);
        // docs-lane route is exempt.
        let docs_lane = build_worktree_first("swarming", "docs", false);
        assert_eq!(expect_done(edit("src/app.js"), &docs_lane.root).code, 0);
        // recorded off-switch disables the refusal.
        let off = build_worktree_first("swarming", "standard", true);
        assert_eq!(expect_done(edit("src/app.js"), &off.root).code, 0);
        // corrupt grants registry fails OPEN.
        let corrupt = build_worktree_first("swarming", "standard", false);
        std::fs::write(
            corrupt.root.join(".bee").join("runtime").join("worktree-grants.json"),
            "{ not json",
        )
        .unwrap();
        assert_eq!(expect_done(edit("src/app.js"), &corrupt.root).code, 0);
    }

    // wtf-4: a second independent read of aac9984f found the tiny-lane
    // exemption sitting above BOTH arms — a feature that ALREADY holds a
    // granted worktree could take a lane "tiny" source edit in main. That
    // is precisely the drift worktree-first exists to stop: a tiny edit
    // landing in main while its worktree is live is a merge conflict
    // waiting at `bee worktree merge`. The exemption belongs to the
    // no-grant arm only (tested in worktree_first_no_grant_arm_carve_outs_allow
    // above); the granted arm must deny lane "tiny" exactly as it denies
    // lane "standard", matching 96db1a33^ behavior.
    #[test]
    fn worktree_first_granted_arm_denies_lane_tiny_too() {
        let wtf = build_worktree_first("swarming", "tiny", false);
        let e = expect_done(edit("src/app.js"), &wtf.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("worktree-first"), "{}", e.stderr);
        assert!(e.stderr.contains(&*wtf.wt_root.file_name().unwrap().to_string_lossy()), "{}", e.stderr);
        assert!(e.stderr.contains(&format!("bee worktree merge --id {}", wtf.id)), "{}", e.stderr);
        assert!(e.stderr.contains("\"demo\"") && e.stderr.contains("\"tiny\""), "{}", e.stderr);
    }

    // wtf-3 (1): the pre-existing granted-worktree refusal predates the
    // "swarming" phase gate 96db1a33 hoisted above BOTH arms — it must stay
    // phase-independent, denying at every phase, not only "swarming".
    #[test]
    fn worktree_first_granted_arm_is_phase_independent() {
        for phase in ["reviewing", "planning", "scribing"] {
            let wtf = build_worktree_first(phase, "standard", false);
            let e = expect_done(edit("src/app.js"), &wtf.root);
            assert_eq!(e.code, 2, "phase {phase}: {}", e.stderr);
            assert!(e.stderr.contains("worktree-first"), "phase {phase}: {}", e.stderr);
            assert!(
                e.stderr.contains(&format!("bee worktree merge --id {}", wtf.id)),
                "phase {phase}: {}",
                e.stderr
            );
        }
    }

    // wtf-3 (2): a grant IS recorded for the feature, but its worktree
    // directory or identity file can't be read — never guessed at as "no
    // grant"; the write must fail OPEN, never deny with a factually false
    // "holds no granted worktree" message and a `bee worktree new` remedy
    // that would itself refuse (WORKTREE_TARGET_EXISTS / WORKTREE_GRANT_EXISTS).

    fn build_worktree_first_broken_grant(phase: &str, kind: &str) -> Wtf {
        let fx_dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(fx_dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&root);
        run_git(&root, &["init", "-q"]);
        let route_state = json!({
            "phase": phase, "mode": "standard", "feature": "demo",
            "route": { "class": "feature", "lane": "standard", "flags": [], "product_files": 2, "rationale": null, "updated_at": ms_to_iso(now_ms()).unwrap() },
            "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
        });
        write_state(&root, &route_state);
        let id = "wtf-demo-wt-broken".to_string();
        let wt_dir = tempfile::tempdir().unwrap();
        let wt_root = dunce::canonicalize(wt_dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            root.join(".bee").join("runtime").join("worktree-grants.json"),
            format!("{}\n", json!({ &id: true })),
        )
        .unwrap();
        if kind == "identity_unreadable" {
            // The worktree link resolves cleanly, but nothing in it names a
            // feature — no worktree-identity.json, no .bee/state.json.
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
        }
        // kind == "dir_gone": the registry still lists the id as granted,
        // but .git/worktrees/<id> was never (re)created — exactly what a
        // `git worktree remove` leaves behind when the grant entry survives
        // it.
        Wtf { _root_dir: fx_dir, _wt_dir: wt_dir, root, wt_root, id }
    }

    #[test]
    fn worktree_first_unresolvable_worktree_dir_fails_open() {
        let wtf = build_worktree_first_broken_grant("swarming", "dir_gone");
        let e = expect_done(edit("src/app.js"), &wtf.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn worktree_first_unreadable_identity_fails_open() {
        let wtf = build_worktree_first_broken_grant("swarming", "identity_unreadable");
        let e = expect_done(edit("src/app.js"), &wtf.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── no-grant arm (wtf-1): the DOMINANT shape — a feature that never
    // held a granted worktree at all — must now deny, not silently allow.

    struct WtfNoGrant {
        _dir: tempfile::TempDir,
        root: PathBuf,
    }

    fn build_worktree_first_no_grant(phase: &str, lane_route: Option<&str>) -> WtfNoGrant {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&root);
        // wtf-3 (5): a REAL git checkout, not just a `.bee`-onboarded
        // tempdir — the no-grant arm must be proven where `bee worktree
        // new` can actually run.
        run_git(&root, &["init", "-q"]);
        let mut state = json!({
            "phase": phase, "mode": "standard", "feature": "demo",
            "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
        });
        if let Some(lane) = lane_route {
            state["route"] = json!({
                "class": "feature", "lane": lane, "flags": [], "product_files": 2,
                "rationale": null, "updated_at": ms_to_iso(now_ms()).unwrap()
            });
        }
        write_state(&root, &state);
        // No .bee/runtime/worktree-grants.json at all — the dominant shape:
        // this feature (indeed this repo) never granted a worktree.
        WtfNoGrant { _dir: dir, root }
    }

    #[test]
    fn worktree_first_denies_main_source_write_with_no_grant_at_all() {
        let wtf = build_worktree_first_no_grant("swarming", Some("standard"));
        let e = expect_done(edit("src/app.js"), &wtf.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("worktree-first"), "{}", e.stderr);
        assert!(e.stderr.contains("bee worktree new --feature demo"), "{}", e.stderr);
        assert!(e.stderr.contains("MAIN checkout"), "{}", e.stderr);
        assert!(e.stderr.contains("\"demo\"") && e.stderr.contains("\"standard\""), "{}", e.stderr);
        assert!(e.stderr.contains("worktree_first: \"off\""), "{}", e.stderr);
        // Bash-extracted target too.
        let eb = expect_done(bash("printf x > src/app.js"), &wtf.root);
        assert_eq!(eb.code, 2, "{}", eb.stderr);
    }

    #[test]
    fn worktree_first_no_grant_arm_carve_outs_allow() {
        // lane "tiny" never fires — but only here, on the no-grant arm; the
        // granted arm still denies lane "tiny" (see
        // worktree_first_granted_arm_denies_lane_tiny_too below).
        let tiny = build_worktree_first_no_grant("swarming", Some("tiny"));
        assert_eq!(expect_done(edit("src/app.js"), &tiny.root).code, 0);
        // lane "docs" never fires.
        let docs = build_worktree_first_no_grant("swarming", Some("docs"));
        assert_eq!(expect_done(edit("src/app.js"), &docs.root).code, 0);
        // a phase other than "swarming" never fires ("idle" is skipped here:
        // it trips the unrelated intake gate before reaching this check at
        // all, which would prove nothing about worktree-first itself).
        let planning = build_worktree_first_no_grant("planning", Some("standard"));
        assert_eq!(expect_done(edit("src/app.js"), &planning.root).code, 0);
        // a missing/empty route on the acting record is "no opinion".
        let no_route = build_worktree_first_no_grant("swarming", None);
        assert_eq!(expect_done(edit("src/app.js"), &no_route.root).code, 0);
    }

    // dmc-3: the no-grant arm's "tiny" carve-out is gated on whether another
    // LIVE session exists — closing the gap the comment above it named at
    // wtf-3. A session file whose `last_heartbeat` will not parse counts as
    // corrupt data, not a live session — the same shape `add_live_session`
    // uses, minus a readable heartbeat.
    fn add_session_with_unparseable_heartbeat(root: &Path, id: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": id, "started_at": "not-a-date", "last_heartbeat": "not-a-date"
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    #[test]
    fn worktree_first_no_grant_tiny_denied_when_sibling_session_live() {
        let wtf = build_worktree_first_no_grant("swarming", Some("tiny"));
        add_live_session(&wtf.root, "other-live");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"session_id":"mine"}),
            &wtf.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("worktree-first"), "{}", e.stderr);
    }

    #[test]
    fn worktree_first_no_grant_tiny_self_exclusion_still_allows_solo_write() {
        // Only session record on disk is the ACTING session's own — it must
        // never count as "another live session" and take the carve-out away.
        let wtf = build_worktree_first_no_grant("swarming", Some("tiny"));
        add_live_session(&wtf.root, "mine");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"session_id":"mine"}),
            &wtf.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // A direct call to `check_worktree_first` itself, not through the full
    // `expect_done`/`run_native` pipeline: an EARLIER, unrelated guard
    // (wcg-2's `is_shared_nested_checkout_target`, main.rs) also scans
    // `.bee/sessions` in STRICT mode ahead of this one, on every
    // write-capable call — a session record with an unparseable heartbeat
    // trips THAT guard's own (intentional, differently-scoped) fail-closed
    // read-error handling before a write ever reaches `check_worktree_first`,
    // which would make an end-to-end test prove that guard's behavior, not
    // this one's. Calling the function under test directly isolates the
    // claim this cell actually makes: a corrupt/unreadable session record
    // never turns `check_worktree_first`'s own tiny carve-out into a refusal.
    #[test]
    fn worktree_first_no_grant_tiny_fails_open_on_corrupt_session_store() {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        add_session_with_unparseable_heartbeat(&root, "other-live");
        let record = json!({
            "phase": "swarming",
            "feature": "demo",
            "route": { "lane": "tiny" }
        });
        let record = record.as_object().unwrap().clone();
        let root_s = root.to_string_lossy().into_owned();
        let result = check_worktree_first(
            "ordinary",
            &root_s,
            &root,
            &record,
            &["src/app.js".to_string()],
            Some("mine"),
        )
        .unwrap();
        assert!(result.is_none(), "corrupt session store must fail OPEN, got {:?}", result);
    }

    // dll-1: docs-lane main privilege is gated on the same liveness fact as
    // tiny's, computed once and reused (D1/D3, docs/history/docs-lane-liveness/
    // CONTEXT.md) — solo, docs stays exactly as fast as today; with a live
    // peer, it routes into a worktree like any other feature. Modelled on
    // the tiny liveness tests above (dmc-3).

    fn add_stale_session(root: &Path, id: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        // Well past HEARTBEAT_STALE_SECONDS (900s) — reads as no session at
        // all, the same shape a genuinely dead peer leaves behind.
        let stale = ms_to_iso(now_ms() - (HEARTBEAT_STALE_SECONDS + 60.0) * 1000.0).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": id, "started_at": stale, "last_heartbeat": stale
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    #[test]
    fn worktree_first_docs_lane_solo_allowed_byte_identical() {
        // No other live session at all: unchanged from today.
        let wtf = build_worktree_first_no_grant("swarming", Some("docs"));
        assert_eq!(expect_done(edit("src/app.js"), &wtf.root).code, 0);
    }

    #[test]
    fn worktree_first_docs_lane_denied_when_sibling_session_live() {
        let wtf = build_worktree_first_no_grant("swarming", Some("docs"));
        add_live_session(&wtf.root, "other-live");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"session_id":"mine"}),
            &wtf.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("worktree-first"), "{}", e.stderr);
        assert!(e.stderr.contains("bee worktree new --feature demo"), "{}", e.stderr);
    }

    #[test]
    fn worktree_first_docs_lane_self_exclusion_still_allows_solo_write() {
        // Only session record on disk is the ACTING session's own — it must
        // never count as "another live session" and take the carve-out away.
        let wtf = build_worktree_first_no_grant("swarming", Some("docs"));
        add_live_session(&wtf.root, "mine");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"session_id":"mine"}),
            &wtf.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn worktree_first_docs_lane_stale_peer_reads_as_no_peer() {
        let wtf = build_worktree_first_no_grant("swarming", Some("docs"));
        add_stale_session(&wtf.root, "other-stale");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"session_id":"mine"}),
            &wtf.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ser-2: a session record marked `status: "closed"` (SessionEnd's clean
    // exit) reads as no-peer here too, even with a heartbeat that is still
    // WITHIN the freshness window — the closed mark itself is what releases
    // it, independent of timing.
    fn add_closed_session(root: &Path, id: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        let now = ms_to_iso(now_ms()).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": id, "started_at": now, "last_heartbeat": now,
                    "status": "closed", "closed_at": now
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    #[test]
    fn worktree_first_docs_lane_closed_peer_reads_as_no_peer() {
        let wtf = build_worktree_first_no_grant("swarming", Some("docs"));
        add_closed_session(&wtf.root, "other-closed");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"session_id":"mine"}),
            &wtf.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ser-3: an explicit `state session release` writes the SAME
    // `status: "closed"` mark `add_closed_session` above pins, plus
    // `released: true`. Neither `is_concurrent_mode` nor
    // `active_worker_session_ids` inspects `released` — the closed mark
    // alone already reads as not-live — so this only proves that reading
    // holds true for a released record too, exactly as the "closed" test
    // above proves it for a plain SessionEnd close.
    fn add_released_session(root: &Path, id: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        let now = ms_to_iso(now_ms()).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": id, "started_at": now, "last_heartbeat": now,
                    "status": "closed", "closed_at": now, "released": true
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    #[test]
    fn worktree_first_docs_lane_released_peer_reads_as_no_peer() {
        let wtf = build_worktree_first_no_grant("swarming", Some("docs"));
        add_released_session(&wtf.root, "other-released");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"session_id":"mine"}),
            &wtf.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn is_concurrent_mode_and_active_worker_session_ids_treat_a_released_session_as_not_live() {
        let fx = wcg_root();
        add_released_session(&fx.root, "other-released");
        let root_s = fx.root.to_string_lossy().into_owned();
        assert!(
            !is_concurrent_mode(&root_s, None),
            "a released session must not count toward concurrent mode"
        );
        assert!(
            active_worker_session_ids(&root_s, None).unwrap().is_empty(),
            "a released session must not read as an active worker"
        );
    }

    #[test]
    fn worktree_first_docs_lane_fails_open_on_corrupt_session_store() {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        add_session_with_unparseable_heartbeat(&root, "other-live");
        let record = json!({
            "phase": "swarming",
            "feature": "demo",
            "route": { "lane": "docs" }
        });
        let record = record.as_object().unwrap().clone();
        let root_s = root.to_string_lossy().into_owned();
        let result = check_worktree_first(
            "ordinary",
            &root_s,
            &root,
            &record,
            &["src/app.js".to_string()],
            Some("mine"),
        )
        .unwrap();
        assert!(result.is_none(), "corrupt session store must fail OPEN, got {:?}", result);
    }

    // dll-1: the `.md` blanket exemption (`worktree_first_exempt_rel`) is
    // gated on the same fact — a bare `.md` path outside the prefix list
    // (README.md is not under `.bee/`, `docs/`, `plans/`, or `AGENTS.md`)
    // loses its exemption only while a peer is live. Proven on lane
    // "standard" so the .md gate stands on its own, independent of the
    // docs/tiny lane gates above.
    #[test]
    fn worktree_first_md_write_denied_with_live_peer_allowed_solo() {
        let wtf = build_worktree_first_no_grant("swarming", Some("standard"));
        add_live_session(&wtf.root, "other-live");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"README.md"},"session_id":"mine"}),
            &wtf.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("worktree-first"), "{}", e.stderr);

        let solo = build_worktree_first_no_grant("swarming", Some("standard"));
        assert_eq!(expect_done(edit("README.md"), &solo.root).code, 0);
    }

    // dll-1: the prefix exemptions stay unconditional even with a live
    // peer — bee's own bookkeeping and the merge auto-commit already cover
    // those roots (D3, docs/history/docs-lane-liveness/CONTEXT.md).
    #[test]
    fn worktree_first_prefix_exemptions_stay_unconditional_with_live_peer() {
        let wtf = build_worktree_first_no_grant("swarming", Some("standard"));
        add_live_session(&wtf.root, "other-live");
        for path in [".bee/notes.txt", "docs/specs/plan.txt", "plans/roadmap.txt", "AGENTS.md"] {
            let e = expect_done(
                json!({"tool_name":"Edit","tool_input":{"file_path":path},"session_id":"mine"}),
                &wtf.root,
            );
            assert_eq!(e.code, 0, "{path}: {}", e.stderr);
        }
    }

    // wtf-3 (5) / decision 0cd7bc46: the live beedashboard shape is a
    // PRESENT, EMPTY `{}` grants registry — a different code branch from a
    // missing file — and it must still deny, not be misread as corrupt.
    #[test]
    fn worktree_first_denies_with_present_empty_grants_registry() {
        let wtf = build_worktree_first_no_grant("swarming", Some("standard"));
        std::fs::create_dir_all(wtf.root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            wtf.root.join(".bee").join("runtime").join("worktree-grants.json"),
            "{}\n",
        )
        .unwrap();
        let e = expect_done(edit("src/app.js"), &wtf.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("worktree-first"), "{}", e.stderr);
        assert!(e.stderr.contains("bee worktree new --feature demo"), "{}", e.stderr);
    }

    // wtf-3 (3): `bee worktree new` refuses with WORKTREE_CALLER_NOT_ORDINARY
    // outside a git checkout (adapter.rs supports a .bee/onboarding.json
    // root with no .git at all) — the no-grant arm must never name that
    // remedy where it cannot run.
    #[test]
    fn worktree_first_no_grant_arm_skips_non_git_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&root);
        // No .git anywhere under root.
        assert!(!root.join(".git").exists());
        write_state(
            &root,
            &json!({
                "phase": "swarming", "mode": "standard", "feature": "demo",
                "route": {
                    "class": "feature", "lane": "standard", "flags": [], "product_files": 2,
                    "rationale": null, "updated_at": ms_to_iso(now_ms()).unwrap()
                },
                "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
            }),
        );
        let e = expect_done(edit("src/app.js"), &root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    fn write_session_lane(root: &Path, id: &str, lane: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        let now = ms_to_iso(now_ms()).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": id, "lane": lane, "started_at": now, "last_heartbeat": now
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    fn write_lane_record(root: &Path, feature: &str, phase: &str, lane_route: &str) {
        let dir = root.join(".bee").join("lanes");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{feature}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "schema_version": "1.0",
                    "feature": feature,
                    "phase": phase,
                    "route": {
                        "class": "feature", "lane": lane_route, "flags": [], "product_files": 2,
                        "rationale": null, "updated_at": ms_to_iso(now_ms()).unwrap()
                    },
                    "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    /// A session bound to a lane with no record is in the one state every
    /// lane-resolving seam refuses on — and the refusals all name
    /// `bee state session unbind` as the FIX. The lane guard therefore must
    /// not stand in front of that command: it judges git invocations, so a
    /// Bash command carrying none of them is not its business. What it does
    /// judge keeps its teeth — a git verb still denies, and so does a file
    /// write through check_write.
    #[test]
    fn the_lane_guard_holds_git_and_writes_but_never_the_unbind_that_escapes_it() {
        let fx = build_fixture("swarming", true);
        run_git(&fx.root, &["init", "-q"]);
        write_session_lane(&fx.root, "sess-1", "ghost");
        // No .bee/lanes/ghost.json — the binding resolves to nothing.

        let allowed = |command: &str| {
            let e = expect_done(
                json!({"tool_name":"Bash","tool_input":{"command":command},"session_id":"sess-1"}),
                &fx.root,
            );
            assert_eq!(e.code, 0, "expected {command} to be allowed: {}", e.stderr);
            assert!(!e.stderr.contains("bee lane guard"), "{command}: {}", e.stderr);
        };
        // The escape hatch itself, and an ordinary read.
        allowed("bee state session unbind --session-id sess-1");
        allowed("ls -la");

        // A git invocation under the same broken binding still denies.
        let denied = expect_done(
            json!({
                "tool_name":"Bash",
                "tool_input":{"command":"git commit -m wip"},
                "session_id":"sess-1"
            }),
            &fx.root,
        );
        assert_eq!(denied.code, 2, "{}", denied.stderr);
        assert!(denied.stderr.contains("bee lane guard"), "{}", denied.stderr);
        assert!(denied.stderr.contains("\"ghost\""), "{}", denied.stderr);

        // And so does a file write — check_write resolves the same record.
        let write_denied = expect_done(
            json!({
                "tool_name":"Edit",
                "tool_input":{"file_path":"src/app.js"},
                "session_id":"sess-1"
            }),
            &fx.root,
        );
        assert_eq!(write_denied.code, 2, "{}", write_denied.stderr);
        assert!(write_denied.stderr.contains("bee lane guard"), "{}", write_denied.stderr);
    }

    #[test]
    fn worktree_first_judges_the_lane_bound_acting_record_not_state_json() {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        copy_lib(&root);
        run_git(&root, &["init", "-q"]);
        // state.json (the default pipeline record) names a DIFFERENT
        // feature on an exempt lane ("docs") — if the guard mistakenly
        // judged this file for a lane-bound session, it would silently
        // allow, exactly the live 2026-08-12 incident.
        write_state(
            &root,
            &json!({
                "phase": "swarming", "mode": "standard", "feature": "other-feature",
                "route": {
                    "class": "feature", "lane": "docs", "flags": [], "product_files": 2,
                    "rationale": null, "updated_at": ms_to_iso(now_ms()).unwrap()
                },
                "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
            }),
        );
        // The session is bound to lane "hub-finished-compact" — its OWN
        // lane record names a different feature and a code-touching lane,
        // mirroring the live hub-finished-compact.json shape from that
        // incident (phase swarming, route.lane small).
        write_session_lane(&root, "sess-1", "hub-finished-compact");
        write_lane_record(&root, "hub-finished-compact", "swarming", "small");
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/app.js"},"session_id":"sess-1"}),
            &root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("worktree-first"), "{}", e.stderr);
        assert!(e.stderr.contains("\"hub-finished-compact\""), "{}", e.stderr);
        assert!(e.stderr.contains("\"small\""), "{}", e.stderr);
        assert!(
            e.stderr.contains("bee worktree new --feature hub-finished-compact"),
            "{}",
            e.stderr
        );
        assert!(!e.stderr.contains("other-feature"), "{}", e.stderr);
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
        // A probe-/verdict-/digest- prefixed name is a real source file, not
        // a scratch payload, once its extension marks it code.
        allow("scripts/probe-foo.mjs");
        allow("src/probe-runner.rs");
        deny("probe-results.json");
        // D8: .bee/mailbox/ is a tmp-then-rename staging area (D3's
        // gesture) — the exemption must cover it, not just the other
        // .bee/ scratch dirs.
        allow(".bee/mailbox/job-1/result-1.json.tmp");
        deny(".bee/other/x.tmp");
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
    fn git_idle_gate_safe_form_table() {
        let fx = build_git_fixture("idle");
        let allow = |cmd: &str| {
            let e = expect_done(bash(cmd), &fx.root);
            assert_eq!(e.code, 0, "expected allow for {cmd:?}: {}", e.stderr);
        };
        let deny = |cmd: &str| {
            let e = expect_done(bash(cmd), &fx.root);
            assert_eq!(e.code, 2, "expected deny for {cmd:?}: {}", e.stderr);
        };

        // Safe (non-mutating) spellings — newly allowed.
        allow("git branch");
        allow("git remote");
        allow("git stash list");
        allow("git stash show");
        allow("git worktree list");
        allow("git reflog");
        allow("git grep pattern");
        allow("git tag --list");
        allow("git branch --list");

        // Mutating spellings of the SAME verbs — still denied at idle.
        deny("git stash pop");
        deny("git stash -- list"); // routes to `stash push` with pathspec "list"
        deny("git worktree add x");
        deny("git reflog expire");
        deny("git branch -D x");
        deny("git branch --set-upstream-to=origin/main");
        deny("git branch --unset-upstream");
        deny("git branch -uorigin/main");
        deny("git remote add o u");
        deny("git grep --open-files-in-pager=cmd x");
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

    // ── sfg-1 (slp-followup-gaps D1/D2): an unbound session's OWN live claim
    // resolves the acting record before the control-root default record ────

    fn write_claim(root: &Path, cell: &str, session: &str) {
        write_claim_ttl(root, cell, session, 3600.0, now_ms());
    }

    fn write_claim_ttl(root: &Path, cell: &str, session: &str, ttl: f64, claimed_ms: f64) {
        let dir = root.join(".bee").join("claims");
        std::fs::create_dir_all(&dir).unwrap();
        let stamp = ms_to_iso(claimed_ms).unwrap();
        std::fs::write(
            dir.join(format!("{cell}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "cell": cell,
                    "session": session,
                    "workspace_id": "main",
                    "ttl_seconds": ttl,
                    "claimed_at": stamp,
                    "acquired_at": stamp,
                    "fence_epoch": 1
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    fn write_cell_record(root: &Path, cell: &str, feature: &str) {
        let dir = root.join(".bee").join("cells");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{cell}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": cell, "feature": feature, "role": "code", "status": "claimed"
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    fn session_bash(command: &str, session: &str) -> Value {
        json!({
            "tool_name": "Bash",
            "tool_input": { "command": command },
            "session_id": session
        })
    }

    /// The live incident (pattern 20260828). `.bee/state.json` says idle; the
    /// dispatched worker was never bound to a lane; its own claim names the
    /// cell, the cell names the feature, and that feature's lane record is
    /// swarming. The mid-cell commit is legitimate and must not be refused.
    #[test]
    fn sfg1_an_unbound_session_with_one_live_claim_commits_against_its_claimed_lane() {
        let fx = build_git_fixture("idle");
        add_live_session(&fx.root, "sess-1");
        write_claim(&fx.root, "cell-1", "sess-1");
        write_cell_record(&fx.root, "cell-1", "demo-feat");
        write_lane_record(&fx.root, "demo-feat", "swarming", "docs");
        stage_file(&fx.root, "src/feature.js");
        let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    /// The same resolution at the write check, not only at the git gate.
    #[test]
    fn sfg1_the_write_check_judges_the_claimed_lane_record_too() {
        let fx = build_fixture("idle", false);
        add_live_session(&fx.root, "sess-1");
        write_claim(&fx.root, "cell-1", "sess-1");
        write_cell_record(&fx.root, "cell-1", "demo-feat");
        write_lane_record(&fx.root, "demo-feat", "swarming", "docs");
        let e = expect_done(
            json!({
                "tool_name": "Edit",
                "tool_input": { "file_path": "src/app.js" },
                "session_id": "sess-1"
            }),
            &fx.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    /// D2: no claim to derive from, so the default record answers — and the
    /// refusal names binding the session as the remedy, beside the FIX line.
    #[test]
    fn sfg1_an_unbound_session_with_no_claim_is_refused_and_told_to_bind() {
        let fx = build_git_fixture("idle");
        add_live_session(&fx.root, "sess-1");
        stage_file(&fx.root, "src/feature.js");
        let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("intake gate"), "{}", e.stderr);
        assert!(e.stderr.contains("bee state session bind --lane"), "{}", e.stderr);
        // The existing FIX line still leads; the binding remedy sits beside it.
        let fix = e.stderr.find("FIX: commit or write bookkeeping").unwrap();
        let bind = e.stderr.find("bee state session bind --lane").unwrap();
        assert!(fix < bind, "{}", e.stderr);
    }

    /// The remedy is scoped: with no session at all there is nothing to bind,
    /// so the refusal is byte-identical to today's.
    #[test]
    fn sfg1_the_bind_remedy_is_absent_when_there_is_no_session_to_bind() {
        let fx = build_git_fixture("idle");
        stage_file(&fx.root, "src/feature.js");
        let e = expect_done(bash("git commit -m \"mid-cell\""), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(!e.stderr.contains("bee state session bind"), "{}", e.stderr);
    }

    /// ...and a BOUND session never sees it either: its record came from its
    /// own lane, not from the default.
    #[test]
    fn sfg1_the_bind_remedy_is_absent_for_a_lane_bound_session() {
        let fx = build_git_fixture("idle");
        write_session_lane(&fx.root, "sess-1", "demo-feat");
        write_lane_record(&fx.root, "demo-feat", "idle", "docs");
        stage_file(&fx.root, "src/feature.js");
        let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("intake gate"), "{}", e.stderr);
        assert!(!e.stderr.contains("bee state session bind"), "{}", e.stderr);
    }

    /// Every narrowing condition of D1, each one falling back to the default
    /// record without inventing the bound path's typed lane refusals.
    #[test]
    fn sfg1_ambiguous_or_unusable_claims_fall_back_to_the_default_record() {
        let refused = |fx: &Fx| {
            let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &fx.root);
            assert_eq!(e.code, 2, "{}", e.stderr);
            assert!(e.stderr.contains("intake gate"), "{}", e.stderr);
            assert!(!e.stderr.contains("bee lane guard"), "{}", e.stderr);
        };

        // Two claims naming two DIFFERENT features — ambiguous.
        let two = build_git_fixture("idle");
        add_live_session(&two.root, "sess-1");
        write_claim(&two.root, "cell-1", "sess-1");
        write_cell_record(&two.root, "cell-1", "feat-a");
        write_claim(&two.root, "cell-2", "sess-1");
        write_cell_record(&two.root, "cell-2", "feat-b");
        write_lane_record(&two.root, "feat-a", "swarming", "docs");
        write_lane_record(&two.root, "feat-b", "swarming", "docs");
        stage_file(&two.root, "src/feature.js");
        refused(&two);

        // One claim, but the feature has no lane record at all.
        let missing = build_git_fixture("idle");
        add_live_session(&missing.root, "sess-1");
        write_claim(&missing.root, "cell-1", "sess-1");
        write_cell_record(&missing.root, "cell-1", "ghost-feat");
        stage_file(&missing.root, "src/feature.js");
        refused(&missing);

        // One claim, lane record present but corrupt.
        let corrupt = build_git_fixture("idle");
        add_live_session(&corrupt.root, "sess-1");
        write_claim(&corrupt.root, "cell-1", "sess-1");
        write_cell_record(&corrupt.root, "cell-1", "feat-a");
        std::fs::create_dir_all(corrupt.root.join(".bee").join("lanes")).unwrap();
        std::fs::write(
            corrupt.root.join(".bee").join("lanes").join("feat-a.json"),
            "{ not json\n",
        )
        .unwrap();
        stage_file(&corrupt.root, "src/feature.js");
        refused(&corrupt);

        // One claim, lane record present but naming a DIFFERENT feature.
        let mismatched = build_git_fixture("idle");
        add_live_session(&mismatched.root, "sess-1");
        write_claim(&mismatched.root, "cell-1", "sess-1");
        write_cell_record(&mismatched.root, "cell-1", "feat-a");
        std::fs::create_dir_all(mismatched.root.join(".bee").join("lanes")).unwrap();
        std::fs::write(
            mismatched.root.join(".bee").join("lanes").join("feat-a.json"),
            "{\"schema_version\":\"1.0\",\"feature\":\"someone-else\",\"phase\":\"swarming\"}\n",
        )
        .unwrap();
        stage_file(&mismatched.root, "src/feature.js");
        refused(&mismatched);

        // One claim whose cell record does not exist — nothing names a feature.
        let cellless = build_git_fixture("idle");
        add_live_session(&cellless.root, "sess-1");
        write_claim(&cellless.root, "cell-1", "sess-1");
        write_lane_record(&cellless.root, "demo-feat", "swarming", "docs");
        stage_file(&cellless.root, "src/feature.js");
        refused(&cellless);
    }

    /// A claim owned by a different session is never read — and neither is an
    /// expired one of our own.
    #[test]
    fn sfg1_a_foreign_or_expired_claim_is_never_read() {
        let foreign = build_git_fixture("idle");
        add_live_session(&foreign.root, "sess-1");
        write_claim(&foreign.root, "cell-1", "sess-other");
        write_cell_record(&foreign.root, "cell-1", "demo-feat");
        write_lane_record(&foreign.root, "demo-feat", "swarming", "docs");
        stage_file(&foreign.root, "src/feature.js");
        let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &foreign.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("intake gate"), "{}", e.stderr);

        let expired = build_git_fixture("idle");
        add_live_session(&expired.root, "sess-1");
        write_claim_ttl(&expired.root, "cell-1", "sess-1", 1.0, now_ms() - 3_600_000.0);
        write_cell_record(&expired.root, "cell-1", "demo-feat");
        write_lane_record(&expired.root, "demo-feat", "swarming", "docs");
        stage_file(&expired.root, "src/feature.js");
        let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &expired.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
    }

    /// A BOUND session is untouched by the new arm: its broken binding still
    /// earns the typed lane refusal, claim or no claim.
    #[test]
    fn sfg1_a_bound_session_holding_a_claim_keeps_its_typed_lane_refusal() {
        let fx = build_git_fixture("idle");
        write_session_lane(&fx.root, "sess-1", "ghost");
        write_claim(&fx.root, "cell-1", "sess-1");
        write_cell_record(&fx.root, "cell-1", "demo-feat");
        write_lane_record(&fx.root, "demo-feat", "swarming", "docs");
        stage_file(&fx.root, "src/feature.js");
        let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee lane guard"), "{}", e.stderr);
        assert!(e.stderr.contains("\"ghost\""), "{}", e.stderr);
    }

    // ── sfg-3 (slp-followup-gaps): the claim reader never decides the
    // guard's fate, and the ownership guard's trigger set is pinned ────────

    /// A claim carrying an arbitrary `claimed_at` value — the shapes
    /// `date_parse_ms` cannot turn into milliseconds.
    fn write_claim_stamp(root: &Path, cell: &str, session: &str, claimed_at: Value) {
        let dir = root.join(".bee").join("claims");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{cell}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "cell": cell,
                    "session": session,
                    "workspace_id": "main",
                    "ttl_seconds": 3600.0,
                    "claimed_at": claimed_at,
                    "acquired_at": claimed_at,
                    "fence_epoch": 1
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    /// Every `claimed_at` shape `date_parse_ms` refuses: a non-RFC3339
    /// string, a numeric epoch, an object, a bool — `Some(_) => Err(Nd)`.
    fn unparseable_stamps() -> Vec<Value> {
        vec![
            json!("yesterday"),
            json!("2026-08-29 10:56:37"),
            json!(1_756_400_000_000_i64),
            json!({ "iso": "2026-08-29T10:56:37.000Z" }),
            json!(true),
        ]
    }

    /// The fail-OPEN hole. `claim_active` promises in its own comment that a
    /// claim with no parseable timestamp reads as ACTIVE, but it read the
    /// stamp through `date_parse_ms(..)?`, and that `Err(Nd)` escaped
    /// `resolve_write_record` all the way to `emit_undecidable` — "the guard
    /// did NOT run on it", every path, `.bee` mutations included. ONE
    /// malformed byte in this session's OWN claim switched the whole write
    /// guard off. The reader must swallow it and read the claim as active.
    #[test]
    fn sfg3_an_unparseable_claim_stamp_reads_as_active_and_never_stops_the_guard() {
        for stamp in unparseable_stamps() {
            let fx = build_git_fixture("idle");
            add_live_session(&fx.root, "sess-1");
            write_claim_stamp(&fx.root, "cell-1", "sess-1", stamp.clone());
            write_cell_record(&fx.root, "cell-1", "demo-feat");
            write_lane_record(&fx.root, "demo-feat", "swarming", "docs");
            stage_file(&fx.root, "src/feature.js");
            // expect_done panics on Delegate — the undecidable fail-open.
            let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &fx.root);
            assert_eq!(e.code, 0, "stamp {stamp}: {}", e.stderr);
        }
    }

    /// The same claim, and the guard still REFUSES what it should: reading a
    /// claim as active is not reading it as permission. Its lane is
    /// terminal, so the intake gate holds — proof the guard decided rather
    /// than fell open.
    #[test]
    fn sfg3_an_unparseable_claim_stamp_still_lets_the_guard_refuse() {
        for stamp in unparseable_stamps() {
            let fx = build_git_fixture("swarming");
            add_live_session(&fx.root, "sess-1");
            write_claim_stamp(&fx.root, "cell-1", "sess-1", stamp.clone());
            write_cell_record(&fx.root, "cell-1", "demo-feat");
            write_lane_record(&fx.root, "demo-feat", "idle", "docs");
            stage_file(&fx.root, "src/feature.js");
            let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &fx.root);
            assert_eq!(e.code, 2, "stamp {stamp}: {}", e.stderr);
            assert!(e.stderr.contains("intake gate"), "stamp {stamp}: {}", e.stderr);
        }
    }

    /// The write check reaches the same reader, so it fell open the same
    /// way. A source write judged against a claim-derived SWARMING lane is
    /// allowed, and the guard decided it.
    #[test]
    fn sfg3_an_unparseable_claim_stamp_never_stops_the_write_check_either() {
        let fx = build_fixture("idle", false);
        add_live_session(&fx.root, "sess-1");
        write_claim_stamp(&fx.root, "cell-1", "sess-1", json!("yesterday"));
        write_cell_record(&fx.root, "cell-1", "demo-feat");
        write_lane_record(&fx.root, "demo-feat", "swarming", "docs");
        let allowed = expect_done(
            json!({
                "tool_name": "Edit",
                "tool_input": { "file_path": "src/app.js" },
                "session_id": "sess-1"
            }),
            &fx.root,
        );
        assert_eq!(allowed.code, 0, "{}", allowed.stderr);
    }

    /// A claim the reader cannot understand contributes NOTHING — it never
    /// takes the guard down with it. A garbage `ttl_seconds` beside a
    /// garbage stamp still reads as active; a claim file that is not an
    /// object is skipped and the default record answers. Neither is ever an
    /// undecidable guard.
    #[test]
    fn sfg3_a_claim_the_reader_cannot_understand_never_decides_the_guards_fate() {
        let fx = build_git_fixture("idle");
        add_live_session(&fx.root, "sess-1");
        std::fs::create_dir_all(fx.root.join(".bee").join("claims")).unwrap();
        std::fs::write(
            fx.root.join(".bee").join("claims").join("cell-1.json"),
            "{\"cell\":\"cell-1\",\"session\":\"sess-1\",\"ttl_seconds\":\"soon\",\"claimed_at\":42}\n",
        )
        .unwrap();
        write_cell_record(&fx.root, "cell-1", "demo-feat");
        write_lane_record(&fx.root, "demo-feat", "swarming", "docs");
        stage_file(&fx.root, "src/feature.js");
        let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);

        let arr = build_git_fixture("idle");
        add_live_session(&arr.root, "sess-1");
        std::fs::create_dir_all(arr.root.join(".bee").join("claims")).unwrap();
        std::fs::write(arr.root.join(".bee").join("claims").join("cell-1.json"), "[1,2,3]\n")
            .unwrap();
        write_cell_record(&arr.root, "cell-1", "demo-feat");
        write_lane_record(&arr.root, "demo-feat", "swarming", "docs");
        stage_file(&arr.root, "src/feature.js");
        let e = expect_done(session_bash("git commit -m \"mid-cell\"", "sess-1"), &arr.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("intake gate"), "{}", e.stderr);
    }

    /// A workspace whose write ownership is held by another LIVE session —
    /// the state msn-21's ownership deny fires on.
    fn write_owned_workspace(root: &Path, workspace_id: &str, owner: &str) {
        let dir = root.join(".bee").join("runtime").join("workspaces");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{workspace_id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": workspace_id,
                    "write_owner_session": owner,
                    "fence_epoch": 1,
                    "attached_sessions": [owner]
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    /// msn-21's workspace-ownership guard at write_policy `isolated` (the
    /// default), against a CLAIM-DERIVED record. The claim arm DID move this
    /// guard's trigger set, and this pins which way: the derived lane phase
    /// is the acting phase, read as such in both directions.
    #[test]
    fn sfg3_the_ownership_guard_reads_the_claim_derived_phase_not_the_default_one() {
        // `.bee/state.json` says idle, so before the claim arm this session
        // read `idle` and hit the deny. Its claimed lane is swarming — the
        // phase it is really working in — and a swarming session is never
        // told to isolate.
        let swarming = build_fixture("idle", false);
        add_live_session(&swarming.root, "sess-1");
        add_live_session(&swarming.root, "sess-owner");
        write_owned_workspace(&swarming.root, "main", "sess-owner");
        write_claim(&swarming.root, "cell-1", "sess-1");
        write_cell_record(&swarming.root, "cell-1", "demo-feat");
        write_lane_record(&swarming.root, "demo-feat", "swarming", "docs");
        let e = expect_done(
            json!({
                "tool_name": "Edit",
                "tool_input": { "file_path": "src/app.js" },
                "session_id": "sess-1"
            }),
            &swarming.root,
        );
        assert_eq!(e.code, 0, "{}", e.stderr);
        assert!(!e.stderr.contains("bee write-policy"), "{}", e.stderr);

        // The other direction, same guard: `.bee/state.json` says swarming,
        // the claimed lane says executing. The DERIVED phase wins, so the
        // ownership deny fires where the default record would have skipped
        // it — the claim arm resolves a lane, never an ownership exemption.
        let executing = build_fixture("swarming", true);
        add_live_session(&executing.root, "sess-1");
        add_live_session(&executing.root, "sess-owner");
        write_owned_workspace(&executing.root, "main", "sess-owner");
        write_claim(&executing.root, "cell-1", "sess-1");
        write_cell_record(&executing.root, "cell-1", "demo-feat");
        write_lane_record(&executing.root, "demo-feat", "executing", "docs");
        let e = expect_done(
            json!({
                "tool_name": "Edit",
                "tool_input": { "file_path": "src/app.js" },
                "session_id": "sess-1"
            }),
            &executing.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee write-policy"), "{}", e.stderr);
        assert!(e.stderr.contains("sess-owner"), "{}", e.stderr);
    }

    // ── sfg-4 (slp-followup-gaps): no store or config READ can switch the
    // whole write guard off ────────────────────────────────────────────────

    /// A session record carrying an arbitrary `last_heartbeat` value.
    fn add_session_with_heartbeat(root: &Path, id: &str, last_heartbeat: Value) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "id": id,
                    "started_at": ms_to_iso(now_ms()).unwrap(),
                    "last_heartbeat": last_heartbeat
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    /// Every `last_heartbeat` shape `date_parse_ms` cannot turn into
    /// milliseconds — the `Err(Nd)` arms, not the absent/blank ones.
    fn unparseable_heartbeats() -> Vec<Value> {
        vec![
            json!("just now"),
            json!("2026-08-29 10:56:37"),
            json!(1_756_400_000_000_i64),
            json!({ "iso": "2026-08-29T10:56:37.000Z" }),
            json!(true),
        ]
    }

    /// The residual fail-OPEN sfg-3 left one call frame away. `heartbeat_stale`
    /// read the stamp through `date_parse_ms(..)?`, and
    /// `check_workspace_ownership` `?`d it, so a malformed `last_heartbeat` in
    /// the WORKSPACE OWNER's session record — a record the acting session does
    /// not own and cannot see — reached `emit_undecidable`: exit 0, "the guard
    /// did NOT run on it". A stamp the reader cannot parse is no evidence the
    /// owner went away, so the owner reads LIVE and the guard REFUSES.
    #[test]
    fn sfg4_a_malformed_owner_heartbeat_never_stops_the_guard() {
        for beat in unparseable_heartbeats() {
            let fx = build_fixture("idle", false);
            add_live_session(&fx.root, "sess-1");
            add_session_with_heartbeat(&fx.root, "sess-owner", beat.clone());
            write_owned_workspace(&fx.root, "main", "sess-owner");
            let e = expect_done(
                json!({
                    "tool_name": "Edit",
                    "tool_input": { "file_path": "src/app.js" },
                    "session_id": "sess-1"
                }),
                &fx.root,
            );
            assert_eq!(e.code, 2, "beat {beat}: {}", e.stderr);
            assert!(e.stderr.contains("bee write-policy"), "beat {beat}: {}", e.stderr);
            assert!(e.stderr.contains("sess-owner"), "beat {beat}: {}", e.stderr);
        }
    }

    /// The same reader, the second live call site: `active_worker_session_ids`
    /// feeds `resolve_live_worker_count`, which `check_git_bash_command` reads
    /// for the gc-2 whole-tree denial. One unparseable heartbeat in ANY session
    /// file took that guard down too. Unparseable reads as live, so both
    /// workers are counted and the bare `git commit` is REFUSED.
    #[test]
    fn sfg4_a_malformed_heartbeat_never_stops_the_live_worker_count() {
        for beat in unparseable_heartbeats() {
            let fx = build_git_fixture("swarming");
            add_session_with_heartbeat(&fx.root, "sess-a", beat.clone());
            add_session_with_heartbeat(&fx.root, "sess-b", beat.clone());
            let e = expect_done(bash("git commit -m wip"), &fx.root);
            assert_eq!(e.code, 2, "beat {beat}: {}", e.stderr);
            assert!(
                e.stderr.contains("bee concurrent-worker git guard"),
                "beat {beat}: {}",
                e.stderr
            );
            assert!(e.stderr.contains("2 workers are live"), "beat {beat}: {}", e.stderr);
        }
    }

    /// The second residual: `check_product_root_silent` answered `Err(Nd)` for a
    /// non-string `product_root`, or one naming a directory that is not there,
    /// and that error walked out of `resolve_context` →
    /// `control_root_for_state` → `resolve_write_record` → `emit_undecidable`.
    /// Two lines of config switched the WHOLE guard off. `product_root` names
    /// where PRODUCT DOCS live and this guard never reads it, so it contributes
    /// nothing: the guard goes on allowing and refusing exactly as before.
    #[test]
    fn sfg4_a_product_root_the_guard_cannot_resolve_never_stops_it() {
        for pr in [json!(42), json!("docs/nowhere-at-all"), json!(["a"]), json!(true)] {
            let fx = build_fixture("swarming", true);
            std::fs::write(
                fx.root.join(".bee").join("config.json"),
                format!("{}\n", serde_json::to_string(&json!({ "product_root": pr })).unwrap()),
            )
            .unwrap();
            let allow = expect_done(edit("src/app.js"), &fx.root);
            assert_eq!(allow.code, 0, "product_root {pr}: {}", allow.stderr);
            let deny = expect_done(edit(".bee/state.json"), &fx.root);
            assert_eq!(deny.code, 2, "product_root {pr}: {}", deny.stderr);
        }
    }

    /// The third door into the same `Err(Nd)`: a `.git` FILE that does not
    /// describe a valid linked worktree makes `resolve_context` answer `Threw`,
    /// and `control_root_for_state` turned that into `Err(Nd)` — one broken
    /// `.git` line, and the whole guard stopped running. A context the reader
    /// cannot resolve is no evidence about WHERE the store is, so the root in
    /// hand answers (JS `?? root`) and the guard keeps deciding both ways.
    #[test]
    fn sfg4_an_unresolvable_context_never_stops_the_guard() {
        let fx = build_fixture("swarming", true);
        add_live_session(&fx.root, "sess-1");
        std::fs::write(fx.root.join(".git"), "gitdir: /nowhere/at/all\n").unwrap();
        let allow = expect_done(
            json!({
                "tool_name": "Edit",
                "tool_input": { "file_path": "src/app.js" },
                "session_id": "sess-1"
            }),
            &fx.root,
        );
        assert_eq!(allow.code, 0, "{}", allow.stderr);
        let deny = expect_done(
            json!({
                "tool_name": "Edit",
                "tool_input": { "file_path": ".bee/state.json" },
                "session_id": "sess-1"
            }),
            &fx.root,
        );
        assert_eq!(deny.code, 2, "{}", deny.stderr);
    }

    // ── sfg-5 (slp-followup-gaps): the LEASE and HOLD readers, the strict
    // session read, and the lockout warning ────────────────────────────────

    /// A path lease carrying arbitrary `acquired_at` / `expires_at` values —
    /// `seed_lease` writes only well-formed RFC3339 stamps, and the whole
    /// point here is the stamps a reader cannot turn into milliseconds.
    fn seed_lease_with_stamps(
        root: &Path,
        path: &str,
        agent: &str,
        cell: &str,
        session: Option<&str>,
        acquired_at: Value,
        expires_at: Value,
    ) {
        let dir = root.join(".bee").join("runtime").join("leases").join("paths");
        std::fs::create_dir_all(&dir).unwrap();
        let record = json!({
            "resource": format!("path:{}", res_normalize_path(path)),
            "mode": "write",
            "workflow_id": cell,
            "session_id": session.unwrap_or(SESSIONLESS_SESSION_ID),
            "workspace_id": format!("agent:{}", agent),
            "epoch": 0,
            "acquired_at": acquired_at,
            "expires_at": expires_at,
            "kind": "lease"
        });
        let n = std::fs::read_dir(&dir).map(|d| d.count()).unwrap_or(0);
        std::fs::write(
            dir.join(format!("lease-{n}.json")),
            format!("{}\n", serde_json::to_string_pretty(&record).unwrap()),
        )
        .unwrap();
    }

    // The stamp table is sfg-3's `unparseable_stamps` — the same
    // `date_parse_ms` `Err(Nd)` arms, read here off a LEASE or a HOLD instead
    // of a claim. (The absent / null / blank arms are `Ok(None)` and were
    // never the bug.)

    /// The escape sfg-4 missed, one call frame further out: the cross-session
    /// hold guard reads every path lease through `list_active_reservations` →
    /// `lease_record_expired` / `lease_to_reservation`, both of which read
    /// their stamps through `date_parse_ms(..)?`. `list_path_lease_records`
    /// validates NO timestamp, so ONE lease file carrying
    /// `"expires_at": "tomorrow"` reached `emit_undecidable` — exit 0, "the
    /// guard did NOT run on it" — and turned the WHOLE write guard off. The
    /// restrictive read: an expiry the reader cannot parse is NOT expired, so
    /// the lease still conflicts and still denies.
    #[test]
    fn sfg5_a_malformed_lease_stamp_never_stops_the_guard() {
        for stamp in unparseable_stamps() {
            for (acquired, expires) in [
                (json!("2026-01-01T00:00:00.000Z"), stamp.clone()),
                (stamp.clone(), json!("2999-01-01T00:00:00.000Z")),
                (stamp.clone(), stamp.clone()),
            ] {
                let fx = build_fixture("swarming", true);
                add_live_session(&fx.root, "other");
                seed_lease_with_stamps(
                    &fx.root,
                    "src/held.txt",
                    "otto",
                    "cell-9",
                    Some("other"),
                    acquired.clone(),
                    expires.clone(),
                );
                let e = expect_done(
                    json!({
                        "tool_name": "Edit",
                        "tool_input": { "file_path": "src/held.txt" },
                        "session_id": "mine"
                    }),
                    &fx.root,
                );
                assert_eq!(e.code, 2, "{acquired}/{expires}: {}", e.stderr);
                assert!(
                    e.stderr.contains("bee cross-session hold"),
                    "{acquired}/{expires}: {}",
                    e.stderr
                );
            }
        }
    }

    /// The second lease call site — `find_conflicts`, the agent-scoped
    /// reservation deny in `swarming`. Same reader, same `Err(Nd)`, same
    /// fail-OPEN; and the same restrictive answer keeps the refusal.
    #[test]
    fn sfg5_a_malformed_lease_stamp_still_lets_the_reservation_guard_refuse() {
        for stamp in unparseable_stamps() {
            let fx = build_fixture("swarming", true);
            seed_lease_with_stamps(
                &fx.root,
                "src/held.txt",
                "otto",
                "cell-9",
                None,
                stamp.clone(),
                stamp.clone(),
            );
            let e = expect_done(
                json!({
                    "tool_name": "Edit",
                    "tool_input": { "file_path": "src/held.txt" },
                    "agent_name": "mel"
                }),
                &fx.root,
            );
            assert_eq!(e.code, 2, "stamp {stamp}: {}", e.stderr);
            assert!(e.stderr.contains("bee reservation conflict"), "stamp {stamp}: {}", e.stderr);
        }
    }

    /// The third lease call site — `resolve_live_worker_count` (paths.rs),
    /// which `check_git_bash_command` reads for the gc-2 whole-tree denial. A
    /// lease the reader cannot date used to switch that guard off too;
    /// unparseable-is-not-expired keeps both workers counted and the bare
    /// `git commit` REFUSED.
    #[test]
    fn sfg5_a_malformed_lease_stamp_never_stops_the_live_worker_count() {
        for stamp in unparseable_stamps() {
            let fx = build_git_fixture("swarming");
            for (agent, cell) in [("wk-a", "c-1"), ("wk-b", "c-2")] {
                seed_lease_with_stamps(
                    &fx.root,
                    &format!("src/{cell}.txt"),
                    agent,
                    cell,
                    None,
                    stamp.clone(),
                    stamp.clone(),
                );
            }
            let e = expect_done(bash("git commit -m wip"), &fx.root);
            assert_eq!(e.code, 2, "stamp {stamp}: {}", e.stderr);
            assert!(
                e.stderr.contains("bee concurrent-worker git guard"),
                "stamp {stamp}: {}",
                e.stderr
            );
            assert!(e.stderr.contains("2 workers are live"), "stamp {stamp}: {}", e.stderr);
        }
    }

    /// The HOLD half of the same defect: `find_foreign_holds` read
    /// `mirrored_at` through `date_parse_ms(..)?`, and `foreign_hold_expiry`
    /// read it a second time for the refusal text. A mirrored hold row with an
    /// unreadable stamp switched the cross-worktree guard off. Unparseable is
    /// NOT expired, so the hold stays active and denies.
    #[test]
    fn sfg5_a_malformed_hold_stamp_never_stops_the_guard() {
        for stamp in unparseable_stamps() {
            let wtf = build_worktree_first("swarming", "standard", true);
            let dir = wtf.root.join(".bee").join("runtime");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("cross-worktree-holds.json"),
                format!(
                    "{}\n",
                    serde_json::to_string_pretty(&json!({ "holds": [{
                        "path": "Cargo.lock",
                        "holder": "some-other-checkout",
                        "feature": "other",
                        "session": "s-other",
                        "cell": "c-1",
                        "ttl_seconds": 3600,
                        "mirrored_at": stamp.clone(),
                        "released_at": Value::Null
                    }] }))
                    .unwrap()
                ),
            )
            .unwrap();
            // An EXCLUSIVE path (DEFAULT_EXCLUSIVE_PATHS), so the foreign hold
            // is a hard block rather than the advisory arm — the verdict this
            // reader used to switch off entirely.
            let e = expect_done(edit("Cargo.lock"), &wtf.wt_root);
            assert_eq!(e.code, 2, "stamp {stamp}: {}", e.stderr);
            assert!(e.stderr.contains("bee cross-worktree hold"), "stamp {stamp}: {}", e.stderr);
            // The expiry clause is a display string, never a verdict: a stamp
            // it cannot render says so instead of claiming "no expiry".
            assert!(e.stderr.contains("expiry unknown"), "stamp {stamp}: {}", e.stderr);
        }
    }

    /// The escape sfg-4 deliberately LEFT, closed here.
    /// `is_shared_nested_checkout_target` sits on the hook's own `R<..>` path
    /// and runs for every Edit/Write with a resolvable target, so one
    /// truncated `.bee/sessions/<id>.json` made `read_session_strict` answer
    /// `Err(Nd)` and the whole guard fell open on a real write. A guard never
    /// falls open on data it merely read: it DENIES, in bee's own words,
    /// naming the file and the remedy.
    #[test]
    fn sfg5_an_unreadable_session_file_denies_instead_of_falling_open() {
        for bad in ["", "{ not json", "\u{0}\u{1}\u{2}"] {
            let fx = build_fixture("swarming", true);
            let nested = fx.root.join("repo");
            std::fs::create_dir_all(nested.join(".git")).unwrap();
            std::fs::write(nested.join("foo.js"), "// nested plain\n").unwrap();
            let sessions = fx.root.join(".bee").join("sessions");
            std::fs::create_dir_all(&sessions).unwrap();
            std::fs::write(sessions.join("broken.json"), bad).unwrap();
            let e = expect_done(
                json!({
                    "tool_name": "Edit",
                    "tool_input": { "file_path": "repo/foo.js" },
                    "session_id": "me"
                }),
                &fx.root,
            );
            assert_eq!(e.code, 2, "{bad:?}: {}", e.stderr);
            assert!(e.stderr.contains("bee shared-checkout guard"), "{bad:?}: {}", e.stderr);
            assert!(e.stderr.contains("broken.json"), "{bad:?}: {}", e.stderr);
            assert!(e.stderr.contains("bee state session release"), "{bad:?}: {}", e.stderr);
        }
    }

    /// The lockout sfg-4 opened, made visible. An unparseable `last_heartbeat`
    /// reads as a LIVE session forever with no time-based self-heal, so one
    /// bad owner record can refuse every other session in the checkout. The
    /// restrictive read STAYS — reopening the door would restore the
    /// fail-open — but the refusal is never silent: the reader names the file
    /// and the remedy on stderr, so a human sees WHY.
    #[test]
    fn sfg5_an_unparseable_heartbeat_lockout_is_never_silent() {
        for beat in unparseable_stamps() {
            let fx = build_fixture("idle", false);
            add_live_session(&fx.root, "sess-1");
            add_session_with_heartbeat(&fx.root, "sess-owner", beat.clone());
            write_owned_workspace(&fx.root, "main", "sess-owner");
            let e = expect_done(
                json!({
                    "tool_name": "Edit",
                    "tool_input": { "file_path": "src/app.js" },
                    "session_id": "sess-1"
                }),
                &fx.root,
            );
            // Queued for flush(), exactly like the corrupt-JSON warning — so
            // it never leaks on a delegating run, and it DOES reach stderr
            // ahead of the refusal on every native verdict.
            let warned = take_corrupt_json_warnings();
            assert_eq!(e.code, 2, "beat {beat}: {}", e.stderr);
            // The refusal itself is unchanged (sfg-4 pins it); what is new is
            // that the unreadable record is NAMED, with its remedy.
            assert!(warned.contains("sess-owner.json"), "beat {beat}: {warned}");
            assert!(warned.contains("last_heartbeat"), "beat {beat}: {warned}");
            assert!(warned.contains("bee state session release"), "beat {beat}: {warned}");
            // One line per file, not one per read.
            assert_eq!(warned.matches("sess-owner.json").count(), 1, "beat {beat}: {warned}");
        }
    }

    // ── sfg-6 (slp-followup-gaps): the LAST store read that could switch the
    // whole write guard off ────────────────────────────────────────────────

    /// Every `.bee/companion-session.json` body the marker reader cannot turn
    /// into JSON. A body that PARSES but says nothing usable (`[1,2,3]`,
    /// `{}`) is not here: that is a readable marker declaring no mount, and it
    /// answers `CompanionMount::None` exactly as it always did.
    fn corrupt_companion_markers() -> Vec<&'static str> {
        vec!["", "   \n", "{ not json", "{\"worktreePath\":", "\u{0}\u{1}\u{2}", "[1,2,3"]
    }

    /// The last fail-OPEN of this feature's four-round sweep.
    ///
    /// `resolve_verified_companion_mount_real` read the marker with a bare
    /// `std::fs::read` plus `serde_json::from_str`, and both error arms
    /// answered `Err(Nd)`. That error walked
    /// `target_inside_verified_companion_mount` ->
    /// `is_shared_nested_checkout_target` -> the hook's own `?` -> Delegate ->
    /// `emit_undecidable`: exit 0, "the guard did NOT run on it". The
    /// preconditions are the ordinary ones sfg-5 closed one branch above — a
    /// resolvable target plus one other live session — so a corrupt marker
    /// switched the whole guard off on everyday work.
    ///
    /// The fixture carries the live peer that
    /// `companion_marker_present_delegates_on_containment_failure` has no
    /// reason to: without a peer `is_concurrent_mode_strict` answers `false`
    /// first and the marker is never read at all.
    #[test]
    fn sfg6_a_corrupt_companion_marker_denies_instead_of_falling_open() {
        for body in corrupt_companion_markers() {
            for payload in [
                json!({"tool_name":"Edit","tool_input":{"file_path":"src/inside.txt"},"session_id":"me"}),
                json!({"tool_name":"Write","tool_input":{"file_path":"src/inside.txt"},"session_id":"me"}),
                json!({"tool_name":"Bash","tool_input":{"command":"cp new.txt src/inside.txt"},"session_id":"me"}),
            ] {
                let fx = build_fixture("swarming", true);
                add_live_session(&fx.root, "other-live");

                // Control FIRST: the same fixture, same live peer, no marker
                // at all is a native ALLOW — so the deny below is the marker
                // firing, never the peer.
                let ok = expect_done(payload.clone(), &fx.root);
                assert_eq!(ok.code, 0, "{body:?}: {}", ok.stderr);

                std::fs::write(fx.root.join(".bee").join("companion-session.json"), body).unwrap();
                let e = expect_done(payload.clone(), &fx.root);
                assert_eq!(e.code, 2, "{body:?}: {}", e.stderr);
                assert!(e.stderr.contains("bee shared-checkout guard"), "{body:?}: {}", e.stderr);
                assert!(e.stderr.contains("companion-session.json"), "{body:?}: {}", e.stderr);
                assert!(e.stderr.contains("--with-companion"), "{body:?}: {}", e.stderr);
            }
        }
    }

    /// The non-ENOENT READ error closes the same way as the parse error —
    /// neither escapes. A DIRECTORY where the marker file belongs is the
    /// portable way to make `std::fs::read` fail without ENOENT.
    #[test]
    fn sfg6_an_unopenable_companion_marker_denies_too() {
        let fx = build_fixture("swarming", true);
        add_live_session(&fx.root, "other-live");
        std::fs::create_dir_all(fx.root.join(".bee").join("companion-session.json")).unwrap();
        let e = expect_done(
            json!({"tool_name":"Edit","tool_input":{"file_path":"src/inside.txt"},"session_id":"me"}),
            &fx.root,
        );
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee shared-checkout guard"), "{}", e.stderr);
        assert!(e.stderr.contains("companion-session.json"), "{}", e.stderr);
    }

    /// The boundary of the fix, stated as a test.
    ///
    /// `companion_mount_rel` delegates on the MERE PRESENCE of the marker for
    /// a target that already failed containment — the documented,
    /// containment-gated branch in the module header. sfg-6 does not touch
    /// it, and this pins why that is not a surviving hole of the same class: a
    /// perfect marker and a corrupt one delegate IDENTICALLY there, so
    /// unreadable data decides nothing. What sfg-6 closed is the other
    /// consumer, where readability alone flipped the verdict.
    #[test]
    fn sfg6_the_containment_gated_delegate_never_turned_on_readability() {
        let well_formed = "{\"sessionId\":\"s1\",\"worktreePath\":\"/x\",\"mountPath\":\"repo\"}\n";
        for body in std::iter::once(well_formed).chain(corrupt_companion_markers()) {
            let fx = build_fixture("swarming", true);
            add_live_session(&fx.root, "other-live");
            std::fs::write(fx.root.join(".bee").join("companion-session.json"), body).unwrap();
            expect_delegate(edit("../outside.txt"), &fx.root);
        }
    }

    // ── staging-lane D0 teeth #2: a direct `git commit` inside the
    // REGISTERED staging worktree is refused unless BEE_STAGING_MACHINERY=1
    // is set — phase-independent, like gc-2, so `build_linked`'s "swarming"
    // fixture (already proven above to let a plain `git commit` through)
    // isolates this one new denial cleanly. ──────────────────────────────

    /// `BEE_STAGING_MACHINERY` is process environment, so a bare `set_var`
    /// around one assertion would leak into every other test that spawns a
    /// `git commit` check while it was set. Scoped to exactly the life of
    /// one test, same shape `verbs/worktree/tests.rs`'s `GitCeilingGuard`
    /// uses for `GIT_CEILING_DIRECTORIES`.
    struct StagingMachineryEnvGuard {
        prior: Option<std::ffi::OsString>,
    }

    impl StagingMachineryEnvGuard {
        fn new() -> Self {
            let prior = std::env::var_os("BEE_STAGING_MACHINERY");
            // SAFETY: no other thread reads/writes this specific var across
            // this guard's lifetime — nothing else in this crate consults
            // BEE_STAGING_MACHINERY outside this one test and the staging
            // verbs' own scoped guard (verbs/staging/mod.rs).
            unsafe { std::env::set_var("BEE_STAGING_MACHINERY", "1") };
            StagingMachineryEnvGuard { prior }
        }
    }

    impl Drop for StagingMachineryEnvGuard {
        fn drop(&mut self) {
            // SAFETY: see `new` above.
            match self.prior.take() {
                Some(v) => unsafe { std::env::set_var("BEE_STAGING_MACHINERY", v) },
                None => unsafe { std::env::remove_var("BEE_STAGING_MACHINERY") },
            }
        }
    }

    fn write_staging_record_fixture(main_root: &Path, worktree_root: &Path) {
        let dir = main_root.join(".bee").join("runtime");
        std::fs::create_dir_all(&dir).unwrap();
        let record = json!({
            "branch": "staging",
            "worktree_root": worktree_root.to_string_lossy(),
            "created_at": "2026-01-01T00:00:00.000Z",
            "base_sha": "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            "staged": [],
        });
        std::fs::write(
            dir.join("staging.json"),
            format!("{}\n", serde_json::to_string_pretty(&record).unwrap()),
        )
        .unwrap();
    }

    struct SecondWorktree {
        _dir: tempfile::TempDir,
        root: PathBuf,
    }

    /// A second, independently-fabricated linked worktree off the SAME
    /// `main_root` `build_linked` already set up — proves the guard is
    /// scoped to the staging worktree's OWN path, never a blanket denial of
    /// every linked worktree's commits.
    fn add_sibling_worktree(main_root: &Path, wt_id: &str) -> SecondWorktree {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        let gitdir = main_root.join(".git").join("worktrees").join(wt_id);
        std::fs::create_dir_all(&gitdir).unwrap();
        std::fs::write(root.join(".git"), format!("gitdir: {}\n", gitdir.to_string_lossy())).unwrap();
        std::fs::write(
            gitdir.join("gitdir"),
            format!("{}\n", root.join(".git").to_string_lossy()),
        )
        .unwrap();
        SecondWorktree { _dir: dir, root }
    }

    // One test, not two: `BEE_STAGING_MACHINERY` is process environment
    // (like `GIT_CEILING_DIRECTORIES` above), and `cargo test` runs cases in
    // PARALLEL THREADS by default — a separate "allowed with the env" test
    // setting it concurrently would race this one's "denied without it"
    // assertion. Sequencing both inside one function is race-free by
    // construction: nothing ELSE in this suite touches the var.
    #[test]
    fn staging_worktree_direct_commit_denied_without_env_allowed_with_it() {
        let lx = build_linked(true);
        write_staging_record_fixture(&lx.main_root, &lx.work_root);

        let denied = expect_done(bash("git commit -m \"by hand\""), &lx.work_root);
        assert_eq!(denied.code, 2, "{}", denied.stderr);
        assert!(denied.stderr.contains("staging-worktree commit guard"), "{}", denied.stderr);
        assert!(denied.stderr.contains("bee staging add"), "{}", denied.stderr);

        let _guard = StagingMachineryEnvGuard::new();
        let allowed = expect_done(bash("git commit -m \"machinery merge\""), &lx.work_root);
        assert_eq!(allowed.code, 0, "{}", allowed.stderr);
    }

    #[test]
    fn commit_outside_the_staging_worktree_is_unaffected() {
        let lx = build_linked(true);
        let other = add_sibling_worktree(&lx.main_root, "other");
        // Staging points at lx.work_root, NOT other.root.
        write_staging_record_fixture(&lx.main_root, &lx.work_root);
        let e = expect_done(bash("git commit -m \"normal feature work\""), &other.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── guard-parser-depth (gpd-1): every git invocation is classified, and
    // sh/bash/dash/zsh/ksh -c / eval wrappers no longer hide a command from
    // the guard ────────────────────────────────────────────────────────────
    //
    // Live proof this closes, from docs/history/guard-parser-depth/plan.md:
    //   git stash                            refused
    //   git status && git stash              USED TO BE allowed — now refused
    //   sh -c 'echo x > .bee/state.json'     USED TO BE allowed — now refused

    fn sq(s: &str) -> String {
        format!("'{}'", s.replace('\'', "'\\''"))
    }

    /// Wraps `inner` in `levels` nested `bash -c '<payload>'` shells, using
    /// the standard single-quote-continuation trick so the result re-parses
    /// correctly however deep it goes.
    fn wrap_bash_c(levels: usize, inner: &str) -> String {
        let mut cmd = inner.to_string();
        for _ in 0..levels {
            cmd = format!("bash -c {}", sq(&cmd));
        }
        cmd
    }

    #[test]
    fn compound_git_after_every_separator_still_refuses_the_second_invocation() {
        // p1-guard-compound-bypass: classifying only the FIRST git
        // invocation let an allowed leader (`git status`) shadow a denied
        // trailer (`git stash`) after any of && || ; |.
        for sep in ["&&", "||", ";", "|"] {
            let fx = build_git_fixture("idle");
            let cmd = format!("git status {sep} git stash");
            let e = expect_done(bash(&cmd), &fx.root);
            assert_eq!(e.code, 2, "sep {sep:?}: {}", e.stderr);
            assert!(e.stderr.contains("git stash"), "sep {sep:?}: {}", e.stderr);
        }
    }

    #[test]
    fn a_leading_refusal_is_not_shadowed_by_a_trailing_allow() {
        let fx = build_git_fixture("idle");
        let e = expect_done(bash("git stash && git status"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("git stash"));
    }

    #[test]
    fn sh_bash_eval_wrapper_around_a_git_verb_is_now_refused() {
        // p1-guard-shell-wrapper-bypass, git-verb half.
        for wrapper in ["sh -c 'git stash'", "bash -c 'git stash'", "eval 'git stash'"] {
            let fx = build_git_fixture("idle");
            let e = expect_done(bash(wrapper), &fx.root);
            assert_eq!(e.code, 2, "wrapper {wrapper:?}: {}", e.stderr);
            assert!(e.stderr.contains("git stash"), "wrapper {wrapper:?}: {}", e.stderr);
        }
    }

    #[test]
    fn sh_bash_eval_wrapper_around_a_state_file_redirect_is_now_refused() {
        // p1-guard-shell-wrapper-bypass, direct-edit-target half.
        for wrapper in [
            "sh -c 'echo x > .bee/state.json'",
            "bash -c 'echo x > .bee/state.json'",
            "eval 'echo x > .bee/state.json'",
        ] {
            let fx = build_fixture("swarming", true);
            let e = expect_done(bash(wrapper), &fx.root);
            assert_eq!(e.code, 2, "wrapper {wrapper:?}: {}", e.stderr);
            assert!(e.stderr.contains("bee state"), "wrapper {wrapper:?}: {}", e.stderr);
        }
    }

    #[test]
    fn nested_wrapper_still_refuses() {
        let fx = build_git_fixture("idle");
        let e = expect_done(bash("bash -c 'sh -c \"git stash\"'"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("git stash"), "{}", e.stderr);
    }

    #[test]
    fn wrapper_depth_bound_fully_unwraps_four_levels() {
        let fx = build_git_fixture("idle");
        let cmd = wrap_bash_c(4, "git stash");
        let e = expect_done(bash(&cmd), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("git stash"), "{}", e.stderr);
    }

    #[test]
    fn wrapper_nested_past_the_depth_bound_never_silently_allows() {
        // A 5th level cannot be unwrapped (WRAPPER_MAX_DEPTH = 4). Under an
        // idle-phase gate the broad-write fallback (guards.rs) catches it
        // outright; the must-not-break contract is that it is NEVER a quiet
        // allow either way.
        let idle = build_git_fixture("idle");
        let cmd = wrap_bash_c(5, "git stash");
        let e = expect_done(bash(&cmd), &idle.root);
        assert_eq!(e.code, 2, "{}", e.stderr);

        // In a phase where the broad-write fallback itself would not deny
        // (swarming, no reservation on "**"), the undecided payload fails
        // OPEN rather than being treated as a silent, undiagnosed allow —
        // checkGitBashCommand delegates, and hooks/mod.rs turns a delegate
        // into a LOUD fail-open (visible stderr), never a quiet one.
        let swarming = build_fixture("swarming", true);
        expect_delegate(bash(&cmd), &swarming.root);
    }

    #[test]
    fn echo_of_a_quoted_git_stash_stays_allowed() {
        // A quoted span is only re-tokenized when it is a WRAPPER's payload
        // — never merely because it contains shell-looking text.
        let fx = build_git_fixture("idle");
        let e = expect_done(bash("echo \"git stash\""), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn git_commit_message_containing_shell_operator_text_stays_allowed() {
        let fx = build_git_fixture("idle");
        let e = expect_done(bash("git commit -m \"wip && git stash\""), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn git_dash_capital_c_and_git_dir_flag_still_parse_as_today() {
        let fx = build_git_fixture("idle");
        assert_eq!(expect_done(bash("git -C sub status"), &fx.root).code, 0);
        assert_eq!(expect_done(bash("git --git-dir=.git log"), &fx.root).code, 0);
    }

    #[test]
    fn a_path_whose_basename_merely_contains_sh_is_not_a_wrapper() {
        let fx = build_git_fixture("idle");
        // If this were (wrongly) treated as a wrapper, the hidden `git
        // stash` payload would surface and get refused; matching on the
        // basename EQUALLING a shell name (never a substring) keeps it
        // allowed, exactly like an ordinary unrecognized command.
        let e = expect_done(bash("scripts/shell/thing.sh -c 'git stash'"), &fx.root);
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

    #[test]
    fn credentials_extension_exemption() {
        let fx = build_fixture("swarming", true);
        let deny = |p: &str| {
            let e = expect_done(json!({"tool_name":"Read","tool_input":{"file_path":p}}), &fx.root);
            assert_eq!(e.code, 2, "expected deny for {p}: {}", e.stderr);
            assert!(e.stderr.contains("bee privacy guard"), "{p}");
        };
        let allow = |p: &str| {
            let e = expect_done(json!({"tool_name":"Read","tool_input":{"file_path":p}}), &fx.root);
            assert_eq!(e.code, 0, "expected allow for {p}: {}", e.stderr);
        };
        // "credentials*" without a recognized source extension stays secret.
        deny("credentials");
        deny("credentials.json");
        deny("credentials.csv");
        deny("credentials.yaml");
        // A real source file merely named "credentials*" is not a secret.
        allow("src/credentials.rs");
        allow("credentials_test.go");
        allow("credentials.py");
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
        // A contained target never consults the marker ON THIS FIXTURE —
        // stays native. sfg-6: the reason is the missing live peer, not the
        // marker. `is_concurrent_mode_strict` answers `false` first, so the
        // mount check never runs. Add a peer and the marker IS read; a corrupt
        // one denies now instead of falling open — see
        // `sfg6_a_corrupt_companion_marker_denies_instead_of_falling_open`.
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
        // trun-5: retargeted from "docs/plan.md" (was allowed pre-split; now
        // refused below by `gated_phase_docs_outside_history_now_refuses`,
        // the case this cell's split exists to add) to the path bee itself
        // must still be able to write before Gate 2 clears — its own brief.
        let docs = expect_done(edit("docs/history/some-feature/plan.md"), &fx.root);
        assert_eq!(docs.code, 0, "{}", docs.stderr);
        let approved = build_fixture("planning", true);
        assert_eq!(expect_done(edit("src/app.js"), &approved.root).code, 0);
    }

    #[test]
    fn gated_phase_docs_outside_history_now_refuses() {
        // trun-5: closes the hole named in the plan — before this cell, a
        // docs-lane write to a `docs/` path OUTSIDE `docs/history/` passed
        // freely at a gated phase (blanket `docs/` on the old shared list).
        let fx = build_fixture("planning", false);
        let deny = expect_done(edit("docs/plan.md"), &fx.root);
        assert_eq!(deny.code, 2, "{}", deny.stdout);
        assert!(deny.stderr.contains("bee gate"));
        assert!(deny.stderr.contains("execution"));
        // The gated-phase "Allowed now:" list names docs/history/, not
        // blanket docs/.
        assert!(deny.stderr.contains("docs/history/"));
    }

    #[test]
    fn gated_phase_bee_own_writes_still_allowed() {
        // trun-5 (D1/D6): the split must not lock bee out of writing its own
        // records before approval — `.bee/`, `plans/`, and `AGENTS.md` stay
        // allowed at a gated phase exactly as before.
        let fx = build_fixture("planning", false);
        assert_eq!(expect_done(edit(".bee/decisions.jsonl"), &fx.root).code, 0);
        assert_eq!(expect_done(edit("plans/roadmap.md"), &fx.root).code, 0);
        assert_eq!(expect_done(edit("AGENTS.md"), &fx.root).code, 0);
    }

    #[test]
    fn idle_intake_gate_still_allows_blanket_docs() {
        // trun-5: the intake list (terminal phase, no execution gate to
        // clear) keeps today's full behavior — a docs-lane write anywhere
        // under `docs/` must not be locked out at phase idle.
        let fx = build_fixture("idle", false);
        assert_eq!(expect_done(edit("docs/plan.md"), &fx.root).code, 0);
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

    // ── heredoc fencing (ghf-1) ──────────────────────────────────────────────
    // A heredoc BODY is data the target command reads on stdin, never command
    // syntax — a body word must never be tokenized into a redirect target.

    #[test]
    fn heredoc_body_never_becomes_a_target() {
        let cmd = "git commit -F - <<'EOF'\nmv /etc/passwd /tmp\nit\nEOF";
        let t = extract_bash_targets(cmd);
        assert!(t.paths.is_empty(), "{:?}", t.paths);
        assert!(!t.broad_write, "{:?}", t.paths);
    }

    #[test]
    fn heredoc_real_redirect_before_the_operator_still_extracts() {
        let cmd = "cat > out.txt <<EOF\nsome body content\nEOF";
        let t = extract_bash_targets(cmd);
        assert_eq!(t.paths, vec!["out.txt"]);
    }

    #[test]
    fn two_heredocs_on_one_line_are_both_fenced() {
        let cmd = "cmd <<A <<B\nmv x y\nA\ntee z\nB";
        let t = extract_bash_targets(cmd);
        assert!(t.paths.is_empty(), "{:?}", t.paths);
    }

    #[test]
    fn quoted_and_dash_terminator_forms_are_fenced() {
        let single_quoted = "cat <<'EOF'\nrm -rf /\nEOF";
        assert!(extract_bash_targets(single_quoted).paths.is_empty());
        let double_quoted = "cat <<\"EOF\"\nrm -rf /\nEOF";
        assert!(extract_bash_targets(double_quoted).paths.is_empty());
        // `<<-` strips leading tabs from the body AND the terminator line.
        let dash_form = "sed <<-END\n\tcp a b\n\tEND";
        assert!(extract_bash_targets(dash_form).paths.is_empty());
    }

    #[test]
    fn unterminated_heredoc_yields_nothing_from_the_tail_and_does_not_panic() {
        let cmd = "cat <<EOF\nmv /etc/passwd /tmp\ntee /root/.ssh/authorized_keys";
        let t = extract_bash_targets(cmd);
        assert!(t.paths.is_empty(), "{:?}", t.paths);
    }

    #[test]
    fn here_string_is_not_a_heredoc_and_is_left_as_is() {
        // `<<<` is a here-string, not a heredoc — no body follows on later
        // lines, so fencing must not swallow anything after it.
        let cmd = "cat <<< \"hello\" > out.txt";
        let t = extract_bash_targets(cmd);
        assert_eq!(t.paths, vec!["out.txt"]);
    }

    // ── tokenize_deep / find_git_invocations (gpd-1) ────────────────────────

    #[test]
    fn tokenize_deep_expands_sh_bash_eval_wrappers() {
        let has_pair = |tokens: &[String], a: &str, b: &str| {
            tokens.windows(2).any(|w| w[0] == a && w[1] == b)
        };
        for wrapper in ["sh -c 'git stash'", "bash -c 'git stash'", "dash -c 'git stash'",
            "zsh -c 'git stash'", "ksh -c 'git stash'", "eval 'git stash'"]
        {
            let deep = tokenize_deep(wrapper);
            assert!(!deep.truncated, "{wrapper}: {:?}", deep.tokens);
            assert!(has_pair(&deep.tokens, "git", "stash"), "{wrapper}: {:?}", deep.tokens);
        }
        let deep = tokenize_deep("bash -c 'echo x > .bee/state.json'");
        assert!(deep.tokens.contains(&"echo".to_string()));
        assert!(deep.tokens.contains(&">".to_string()));
        assert!(deep.tokens.contains(&".bee/state.json".to_string()));
    }

    #[test]
    fn tokenize_deep_never_expands_a_quoted_span_that_is_not_a_wrapper_payload() {
        assert_eq!(tokenize_deep("echo \"git stash\"").tokens, vec!["echo", "git stash"]);
        assert_eq!(
            tokenize_deep("git commit -m \"wip && git stash\"").tokens,
            vec!["git", "commit", "-m", "wip && git stash"]
        );
    }

    #[test]
    fn tokenize_deep_basename_containing_sh_is_not_a_wrapper() {
        assert_eq!(
            tokenize_deep("scripts/shell/thing.sh -c 'git stash'").tokens,
            vec!["scripts/shell/thing.sh", "-c", "git stash"]
        );
    }

    #[test]
    fn tokenize_deep_fences_the_payload_so_it_cannot_join_two_segments() {
        // The example from guards.mjs' own doc comment: a wrapper can never
        // merge the segments on either side of it into one.
        let deep = tokenize_deep("a && sh -c 'b; c' && d");
        assert_eq!(deep.tokens, vec!["a", "&&", ";", "b", ";", "c", ";", "&&", "d"]);
    }

    #[test]
    fn tokenize_deep_bounds_recursion_and_flags_truncation() {
        fn sq(s: &str) -> String {
            format!("'{}'", s.replace('\'', "'\\''"))
        }
        fn wrap(levels: usize, inner: &str) -> String {
            let mut cmd = inner.to_string();
            for _ in 0..levels {
                cmd = format!("bash -c {}", sq(&cmd));
            }
            cmd
        }
        // Four levels fully unwrap: nothing left opaque.
        let four = tokenize_deep(&wrap(4, "git stash"));
        assert!(!four.truncated, "{:?}", four.tokens);
        assert!(
            four.tokens.windows(2).any(|w| w[0] == "git" && w[1] == "stash"),
            "{:?}",
            four.tokens
        );
        // A fifth level cannot be unwrapped — left opaque, truncated is set.
        let five = tokenize_deep(&wrap(5, "git stash"));
        assert!(five.truncated, "{:?}", five.tokens);
        assert!(
            !five.tokens.windows(2).any(|w| w[0] == "git" && w[1] == "stash"),
            "{:?}",
            five.tokens
        );
    }

    #[test]
    fn find_git_invocations_returns_every_invocation_in_order() {
        let tokens = tokenize_deep("git status && git stash").tokens;
        let invocations = find_git_invocations(&tokens);
        assert_eq!(invocations.len(), 2);
        assert_eq!(invocations[0].subcommand.as_deref(), Some("status"));
        assert_eq!(invocations[1].subcommand.as_deref(), Some("stash"));
    }

    #[test]
    fn find_git_invocations_still_skips_global_flag_values() {
        let tokens = tokenize_deep("git -C sub status && git --git-dir=.git log").tokens;
        let invocations = find_git_invocations(&tokens);
        assert_eq!(invocations.len(), 2);
        assert_eq!(invocations[0].subcommand.as_deref(), Some("status"));
        assert_eq!(invocations[1].subcommand.as_deref(), Some("log"));
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

    // ── D1: cp/mv operand roles + fd-digit/ampersand null redirects ────────

    #[test]
    fn extract_bash_targets_cp_takes_only_the_last_operand_as_target() {
        let t = extract_bash_targets("cp src.txt dst.txt");
        assert_eq!(t.paths, vec!["dst.txt"], "source must never be extracted");
        // multiple sources, last operand still wins
        let t = extract_bash_targets("cp a.txt b.txt c.txt dst_dir");
        assert_eq!(t.paths, vec!["dst_dir"]);
    }

    // P1-1: mv unlinks its source — every operand mv touches is a write, so
    // both source(s) and destination extract (unlike cp above).
    #[test]
    fn extract_bash_targets_mv_extracts_every_operand_source_and_destination() {
        let t = extract_bash_targets("mv src.txt dst.txt");
        assert_eq!(t.paths, vec!["src.txt", "dst.txt"], "{:?}", t.paths);
        let t = extract_bash_targets("mv a.txt b.txt c.txt dst_dir");
        assert_eq!(t.paths, vec!["a.txt", "b.txt", "c.txt", "dst_dir"], "{:?}", t.paths);
    }

    #[test]
    fn extract_bash_targets_cp_mv_honors_target_directory_flag() {
        let t = extract_bash_targets("cp -v a.txt b.txt -t dst_dir");
        assert_eq!(t.paths, vec!["dst_dir"], "{:?}", t.paths);
        let t = extract_bash_targets("cp a.txt --target-directory=dst_dir");
        assert_eq!(t.paths, vec!["dst_dir"], "{:?}", t.paths);
        // P1-1: mv -t extracts the directory PLUS every source (source is a
        // write since mv unlinks it) — unlike cp, which drops sources.
        let t = extract_bash_targets("mv a.txt b.txt --target-directory dst_dir");
        assert_eq!(t.paths, vec!["a.txt", "b.txt", "dst_dir"], "{:?}", t.paths);
    }

    #[test]
    fn extract_bash_targets_fd_digit_and_ampersand_null_redirects_never_become_targets() {
        // Previously the rm/mv/cp/mkdir/touch/tee operand loop never called
        // match_redirect, so a glued fd-digit redirect like `2>/dev/null`
        // was collected as a literal, bogus path operand.
        let t = extract_bash_targets("rm foo 2>/dev/null");
        assert_eq!(t.paths, vec!["foo"], "{:?}", t.paths);
        let t = extract_bash_targets("rm foo 2>>/dev/null");
        assert_eq!(t.paths, vec!["foo"], "{:?}", t.paths);
        // `&>/dev/null` glued: the tokenizer always splits a bare `&` into its
        // own token, so this arrives as ["&", ">/dev/null"] — both forms must
        // still resolve to no bogus target.
        let t = extract_bash_targets("rm foo &>/dev/null");
        assert_eq!(t.paths, vec!["foo"], "{:?}", t.paths);
        let t = extract_bash_targets("cp src.txt dst.txt 2>/dev/null");
        assert_eq!(t.paths, vec!["dst.txt"], "{:?}", t.paths);
        // P1-1: mv extracts both source and destination.
        let t = extract_bash_targets("mv src.txt dst.txt &>/dev/null");
        assert_eq!(t.paths, vec!["src.txt", "dst.txt"], "{:?}", t.paths);
        // a real (non-null) redirect glued to the operand list still extracts
        assert!(match_redirect("&>/dev/null").is_some());
        assert!(match_redirect("&>>/dev/null").is_some());
        assert_eq!(match_redirect("&>out.log").as_deref(), Some("out.log"));
    }

    #[test]
    fn extract_bash_targets_sees_through_a_wrapper_redirect() {
        for wrapper in [
            "sh -c 'echo x > .bee/state.json'",
            "bash -c 'echo x > .bee/state.json'",
            "eval 'echo x > .bee/state.json'",
        ] {
            let t = extract_bash_targets(wrapper);
            assert_eq!(t.paths, vec![".bee/state.json"], "{wrapper}");
        }
        // A quoted git verb that is NOT a wrapper payload stays hidden, same
        // as before this cell — nothing in a plain `echo "..."` argument is
        // ever re-scanned for a target.
        let t = extract_bash_targets("echo \"git add .bee/state.json\"");
        assert!(t.paths.is_empty(), "{:?}", t.paths);
    }

    #[test]
    fn extract_bash_targets_treats_a_depth_exceeded_wrapper_as_broad_write() {
        fn sq(s: &str) -> String {
            format!("'{}'", s.replace('\'', "'\\''"))
        }
        fn wrap(levels: usize, inner: &str) -> String {
            let mut cmd = inner.to_string();
            for _ in 0..levels {
                cmd = format!("bash -c {}", sq(&cmd));
            }
            cmd
        }
        let within_bound = extract_bash_targets(&wrap(4, "echo x > .bee/state.json"));
        assert!(!within_bound.broad_write, "{:?}", within_bound.paths);
        let past_bound = extract_bash_targets(&wrap(5, "echo x > .bee/state.json"));
        assert!(past_bound.broad_write);
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

    /// sfg-5, then sfg-6: the primitive answers four ways now. These rows all
    /// read the shared/not-shared verdict; the two unreadable-record answers
    /// are their own refusals, pinned by
    /// `sfg5_an_unreadable_session_file_denies_instead_of_falling_open` and
    /// `sfg6_a_corrupt_companion_marker_denies_instead_of_falling_open`, and
    /// none of these fixtures can produce either one.
    fn flagged(root: &Path, target: &Path) -> bool {
        match is_shared_nested_checkout_target(
            &root.to_string_lossy(),
            &target.to_string_lossy(),
            None,
            None,
        )
        .expect("primitive must decide natively")
        {
            SharedNested::Yes => true,
            SharedNested::No => false,
            SharedNested::UnreadableSession(f) => {
                panic!("fixture wrote no unreadable session record, got {}", f.display())
            }
            SharedNested::UnreadableCompanionMarker(f) => {
                panic!("fixture wrote no unreadable companion marker, got {}", f.display())
            }
        }
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
        // Memory + `<temp>/claude`, plus the uid-suffixed scratchpad root on unix.
        let expected = if cfg!(unix) { 3 } else { 2 };
        assert_eq!(roots.roots.len(), expected, "every injected base must resolve");
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

    /// Measured 2026-08-06: the harness scratchpad here is `/tmp/claude-1000`,
    /// while the allowlist only knew `/tmp/claude` — so the E1 exemption missed
    /// the very surface it was written for, and a scratchpad write was denied
    /// mid-session. The exemption is one uid-scoped path, never a `claude-*`
    /// prefix: a sibling uid's scratchpad stays outside it.
    #[cfg(unix)]
    #[test]
    fn gh1_the_uid_suffixed_scratchpad_is_exempt_and_a_sibling_uid_is_not() {
        let hx = build_harness_fixture();
        let uid = unsafe { libc::getuid() };
        let mine = hx.temp.join(format!("claude-{uid}")).join("sess").join("scratchpad").join("f.txt");
        let w = expect_done_with_roots(
            json!({"tool_name":"Write","tool_input":{"file_path":mine.to_string_lossy(),"content":"x\n"}}),
            &hx.fx.root,
            &hx.roots,
        );
        assert_eq!(w.code, 0, "{}", w.stderr);

        let b = expect_done_with_roots(
            bash(&format!("printf x > \"{}\"", mine.to_string_lossy().replace('\\', "/"))),
            &hx.fx.root,
            &hx.roots,
        );
        assert_eq!(b.code, 0, "{}", b.stderr);

        let theirs = hx
            .temp
            .join(format!("claude-{}", uid.wrapping_add(1)))
            .join("sess")
            .join("scratchpad")
            .join("f.txt");
        let d = expect_done_with_roots(edit(&theirs.to_string_lossy()), &hx.fx.root, &hx.roots);
        assert_eq!(d.code, 2, "{}", d.stderr);
        assert!(d.stderr.contains("could not be canonically contained"), "{}", d.stderr);
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

    // ── guard-refusal-wording D1/D2/D3: unresolvable shell-syntax targets ──
    // A Bash target still carrying unexpanded shell syntax ($VAR, backquote)
    // was never expanded by a shell — the guard only ever sees the literal
    // characters, so resolving it as a plain relative path produces a fake
    // in-repo path. It must be classified unresolvable and refused with the
    // resolution-failure wording, never the "writing X is blocked" gate
    // sentence and never GENERIC_BASH_CONTAINMENT_MESSAGE.

    #[test]
    fn bash_unexpanded_var_target_names_resolution_failure_not_a_fake_path() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("printf x > \"$WT/foo.txt\""), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(
            e.stderr.starts_with(
                "bee write guard denied Bash: the target \"$WT/foo.txt\" could not be resolved"
            ),
            "{}",
            e.stderr
        );
        assert!(e.stderr.contains("could not be resolved"), "{}", e.stderr);
        assert!(e.stderr.contains("\"$WT/foo.txt\""), "{}", e.stderr);
        // D3: names the literal-dollar-filename possibility, not only the
        // variable-expansion remedy.
        assert!(e.stderr.contains("literal filename"), "{}", e.stderr);
        assert!(e.stderr.contains("dollar sign"), "{}", e.stderr);
        assert!(!e.stderr.contains("is blocked"), "{}", e.stderr);
        assert!(!e.stderr.contains("canonically contained"), "{}", e.stderr);
    }

    #[test]
    fn bash_backquote_target_names_resolution_failure_not_a_fake_path() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("touch `whoami`"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(
            e.stderr
                .starts_with("bee write guard denied Bash: the target \"`whoami`\" could not be resolved"),
            "{}",
            e.stderr
        );
        assert!(e.stderr.contains("could not be resolved"), "{}", e.stderr);
        assert!(e.stderr.contains("`whoami`"), "{}", e.stderr);
        assert!(!e.stderr.contains("is blocked"), "{}", e.stderr);
    }

    #[test]
    fn bash_resolved_path_containment_denial_wording_is_unchanged() {
        // D3b: a target that genuinely resolves (no shell syntax) but sits
        // outside the worktree keeps its existing GENERIC_BASH_CONTAINMENT_MESSAGE
        // wording — the D1/D2 classification only widens the unresolvable
        // bucket, it never touches an already-resolved target's message.
        let fx = build_fixture("swarming", true);
        let outside = tempfile::tempdir().unwrap();
        let target = dunce::canonicalize(outside.path()).unwrap().join("elsewhere.txt");
        let bash_target = target.to_string_lossy().replace('\\', "/");
        let e = expect_done(bash(&format!("printf x > \"{bash_target}\"")), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert_eq!(e.stderr, GENERIC_BASH_CONTAINMENT_MESSAGE);
    }

    // ── D3: bounded/sanitized token echo ────────────────────────────────────
    // A raw token echoed into a refusal must never carry embedded control
    // characters (a quoted token with an embedded newline cannot inject
    // message-shaped lines) and must never run unbounded.

    #[test]
    fn unresolvable_bash_target_message_strips_control_chars() {
        let raw = "$WT/foo\nbee write guard denied Bash: fake\x07line\r\ninjected";
        let msg = unresolvable_bash_target_message(raw);
        assert!(!msg.contains('\n'), "{}", msg);
        assert!(!msg.contains('\r'), "{}", msg);
        assert!(!msg.contains('\u{7}'), "{}", msg);
        assert!(msg.contains("$WT/foobee write guard denied Bash: fakelineinjected"), "{}", msg);
    }

    #[test]
    fn unresolvable_bash_target_message_bounds_long_tokens() {
        let raw = format!("$WT/{}", "a".repeat(500));
        let msg = unresolvable_bash_target_message(&raw);
        assert!(msg.contains('…'), "{}", msg);
        assert!(!msg.contains(&"a".repeat(500)), "{}", msg);
    }

    // ── D3: the first decisive refusal survives a mixed command ────────────
    // A compound command that mixes an unresolvable shell-syntax target
    // (denied first, by position in the command) with a second, fully
    // resolved, direct-edit-guarded target must keep the FIRST denial's
    // wording — the check_write loop over rel_paths must never overwrite a
    // denial already decided for another candidate in the same command.

    #[test]
    fn bash_mixed_command_first_denial_wording_survives_check_write() {
        let fx = build_fixture("swarming", true);
        let cmd = "printf x > \"$WT/foo.txt\" && printf y > \".bee/state.json\"";
        let e = expect_done(bash(cmd), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(
            e.stderr.starts_with(
                "bee write guard denied Bash: the target \"$WT/foo.txt\" could not be resolved"
            ),
            "{}",
            e.stderr
        );
        assert!(!e.stderr.contains("direct-edit guard"), "{}", e.stderr);
    }

    // ── guard-refusal-wording P1 fix round D4/D5/D6: shell-syntax
    // classification scoped to the Bash surface only ─────────────────────
    // D4: has_unexpanded_shell_syntax moved out of the shared resolvers
    // (canonical_rel_path, resolve_target_realpath) — Edit/Write/MultiEdit
    // file_path and apply_patch targets are literal strings no shell ever
    // expands, so a `$`/backquote in a target name must resolve exactly as
    // before grw-1 on those two surfaces.

    #[test]
    fn edit_dollar_named_target_resolves_literally_after_grw2() {
        let fx = build_fixture("swarming", true);
        // A literal in-tree target still carrying `$` is a valid file name
        // on this surface — no shell ever touches it.
        let allowed = expect_done(edit("src/Foo$Bar.java"), &fx.root);
        assert_eq!(allowed.code, 0, "{}", allowed.stderr);
        // A denied `$`-named target keeps the ordinary containment wording,
        // never the Bash-only D2 resolution-failure wording.
        let denied = expect_done(edit("../$outside.txt"), &fx.root);
        assert_eq!(denied.code, 2, "{}", denied.stderr);
        assert_eq!(denied.stderr, GENERIC_CONTAINMENT_MESSAGE);
        assert!(!denied.stderr.contains("could not be resolved"), "{}", denied.stderr);
    }

    #[test]
    fn apply_patch_dollar_named_target_resolves_literally_after_grw2() {
        let fx = build_fixture("swarming", true);
        let allowed = expect_done(
            patch("*** Begin Patch\n*** Add File: src/Foo$Bar.java\n+hello\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(allowed.code, 0, "{}", allowed.stderr);
        // A denied `$`-named target keeps apply_patch's own unresolved-target
        // wording, never the Bash-only D2 resolution-failure wording.
        let denied = expect_done(
            patch("*** Begin Patch\n*** Add File: ../$outside.txt\n+hello\n*** End Patch"),
            &fx.root,
        );
        assert_eq!(denied.code, 2, "{}", denied.stderr);
        assert!(denied.stderr.contains("could not be fully proved inside the repo"), "{}", denied.stderr);
        assert!(!denied.stderr.contains("still carries"), "{}", denied.stderr);
    }

    #[test]
    fn bash_shell_syntax_target_denies_before_companion_delegate() {
        // D5: on the Bash surface an unresolvable shell-syntax token must
        // deny with the D2 wording BEFORE the companion_mount_rel branch can
        // turn it into Err(Nd)/Delegate — with .bee/companion-session.json
        // present the same command still exits 2 with the D2 wording, never
        // the dispatcher's fail-open allow.
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("companion-session.json"),
            "{\"sessionId\":\"s1\",\"worktreePath\":\"/x\",\"mountPath\":\"repo\"}\n",
        )
        .unwrap();
        let e = expect_done(bash("printf x > \"$WT/foo.txt\""), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(
            e.stderr.starts_with(
                "bee write guard denied Bash: the target \"$WT/foo.txt\" could not be resolved"
            ),
            "{}",
            e.stderr
        );
    }

    // ── write-guard-hardening D2: per-target denials decided before the
    // delegate escape can fail open ─────────────────────────────────────────
    // ADV-A/ADV-B: with the marker (or a declared guards.memory_root)
    // present, a command mixing a shell-syntax target (native denial) and a
    // containment-failing literal target (that same marker/root makes
    // undecidable, hence Err(Nd)) must deny — the sibling's Err(Nd) must
    // never swallow an already-earned denial into a fail-open delegate, in
    // either target order.

    fn outside_bash_target() -> (tempfile::TempDir, String) {
        let outside = tempfile::tempdir().unwrap();
        let target = dunce::canonicalize(outside.path()).unwrap().join("elsewhere.txt");
        let bash_target = target.to_string_lossy().replace('\\', "/");
        (outside, bash_target)
    }

    #[test]
    fn bash_mixed_marker_denies_before_delegate_order_a() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("companion-session.json"),
            "{\"sessionId\":\"s1\",\"worktreePath\":\"/x\",\"mountPath\":\"repo\"}\n",
        )
        .unwrap();
        let (_outside, bash_target) = outside_bash_target();
        let cmd = format!("printf x > \"$WT/foo.txt\" && printf y > \"{bash_target}\"");
        let e = expect_done(bash(&cmd), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(
            e.stderr.starts_with(
                "bee write guard denied Bash: the target \"$WT/foo.txt\" could not be resolved"
            ),
            "{}",
            e.stderr
        );
    }

    #[test]
    fn bash_mixed_marker_denies_before_delegate_order_b() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("companion-session.json"),
            "{\"sessionId\":\"s1\",\"worktreePath\":\"/x\",\"mountPath\":\"repo\"}\n",
        )
        .unwrap();
        let (_outside, bash_target) = outside_bash_target();
        let cmd = format!("printf y > \"{bash_target}\" && printf x > \"$WT/foo.txt\"");
        let e = expect_done(bash(&cmd), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(
            e.stderr.starts_with(
                "bee write guard denied Bash: the target \"$WT/foo.txt\" could not be resolved"
            ),
            "{}",
            e.stderr
        );
    }

    #[test]
    fn bash_mixed_memory_root_denies_before_delegate_order_a() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("config.json"),
            "{\"guards\":{\"memory_root\":\"~/.claude/projects/x/memory\"}}\n",
        )
        .unwrap();
        let (_outside, bash_target) = outside_bash_target();
        let cmd = format!("printf x > \"$WT/foo.txt\" && printf y > \"{bash_target}\"");
        let e = expect_done(bash(&cmd), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(
            e.stderr.starts_with(
                "bee write guard denied Bash: the target \"$WT/foo.txt\" could not be resolved"
            ),
            "{}",
            e.stderr
        );
    }

    #[test]
    fn bash_mixed_memory_root_denies_before_delegate_order_b() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("config.json"),
            "{\"guards\":{\"memory_root\":\"~/.claude/projects/x/memory\"}}\n",
        )
        .unwrap();
        let (_outside, bash_target) = outside_bash_target();
        let cmd = format!("printf y > \"{bash_target}\" && printf x > \"$WT/foo.txt\"");
        let e = expect_done(bash(&cmd), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(
            e.stderr.starts_with(
                "bee write guard denied Bash: the target \"$WT/foo.txt\" could not be resolved"
            ),
            "{}",
            e.stderr
        );
    }

    // Controls: the restructure is fail-closed ONLY. A genuinely undecidable
    // lone target still delegates (never becomes a false deny), and a
    // legitimately contained write still runs native (never becomes a
    // false delegate or deny) even with the marker present.

    #[test]
    fn bash_lone_containment_failure_still_delegates_with_marker() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("companion-session.json"),
            "{\"sessionId\":\"s1\",\"worktreePath\":\"/x\",\"mountPath\":\"repo\"}\n",
        )
        .unwrap();
        let (_outside, bash_target) = outside_bash_target();
        expect_delegate(bash(&format!("printf x > \"{bash_target}\"")), &fx.root);
    }

    #[test]
    fn bash_lone_containment_failure_still_delegates_with_memory_root() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("config.json"),
            "{\"guards\":{\"memory_root\":\"~/.claude/projects/x/memory\"}}\n",
        )
        .unwrap();
        let (_outside, bash_target) = outside_bash_target();
        expect_delegate(bash(&format!("printf x > \"{bash_target}\"")), &fx.root);
    }

    #[test]
    fn bash_contained_target_stays_native_with_marker_present() {
        let fx = build_fixture("swarming", true);
        std::fs::write(
            fx.root.join(".bee").join("companion-session.json"),
            "{\"sessionId\":\"s1\",\"worktreePath\":\"/x\",\"mountPath\":\"repo\"}\n",
        )
        .unwrap();
        let e = expect_done(bash("printf x > src/inside.txt"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── D1 plan.md freeze (bh-1) — feature-from-path + lane-aware gate ─────
    // D7 red-first: these resolver tests land (and are run RED against a
    // stub) before the deny wires into check_write below.

    fn write_lane(root: &Path, feature: &str, shape: bool) {
        let dir = root.join(".bee").join("lanes");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{feature}.json")),
            format!(
                "{}\n",
                serde_json::to_string_pretty(&json!({
                    "schema_version": "1.0",
                    "feature": feature,
                    "phase": "planning",
                    "approved_gates": { "context": true, "shape": shape, "execution": false, "review": false }
                }))
                .unwrap()
            ),
        )
        .unwrap();
    }

    fn default_state_for(feature: &str, shape: bool) -> Map<String, Value> {
        match json!({
            "phase": "planning",
            "feature": feature,
            "approved_gates": { "context": true, "shape": shape, "execution": false, "review": false }
        }) {
            Value::Object(m) => m,
            _ => unreachable!(),
        }
    }

    #[test]
    fn plan_freeze_feature_parses_the_exact_shape_only() {
        assert_eq!(
            plan_freeze_feature("docs/history/hook-teeth/plan.md"),
            Some("hook-teeth".to_string())
        );
        assert_eq!(plan_freeze_feature("docs/history/hook-teeth/CONTEXT.md"), None);
        assert_eq!(plan_freeze_feature("docs/history/hook-teeth/reports/plan.md"), None);
        assert_eq!(plan_freeze_feature("docs/plan.md"), None);
        assert_eq!(plan_freeze_feature("docs/history//plan.md"), None);
    }

    #[test]
    fn plan_freeze_shape_approved_reads_the_default_state_when_no_lane_exists() {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        let approved = default_state_for("demo", true);
        assert!(plan_freeze_shape_approved(&root.to_string_lossy(), &approved, "demo").unwrap());
        let unapproved = default_state_for("demo", false);
        assert!(!plan_freeze_shape_approved(&root.to_string_lossy(), &unapproved, "demo").unwrap());
    }

    #[test]
    fn plan_freeze_shape_approved_is_no_opinion_when_default_state_names_another_feature() {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        let other = default_state_for("other", true);
        // No lane file for "demo", and the default state's own feature is
        // "other" — resolution never guesses, so it reads as unapproved.
        assert!(!plan_freeze_shape_approved(&root.to_string_lossy(), &other, "demo").unwrap());
    }

    #[test]
    fn plan_freeze_shape_approved_lane_record_beats_default_state() {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        write_lane(&root, "demo", false);
        let approved_default = default_state_for("demo", true);
        // The lane record (shape: false) wins over the default state's own
        // shape: true for the same feature.
        assert!(!plan_freeze_shape_approved(&root.to_string_lossy(), &approved_default, "demo")
            .unwrap());

        write_lane(&root, "demo", true);
        let unapproved_default = default_state_for("demo", false);
        // And the reverse: lane says approved even though default doesn't.
        assert!(plan_freeze_shape_approved(&root.to_string_lossy(), &unapproved_default, "demo")
            .unwrap());
    }

    // ── D1 plan.md freeze — deny wired into check_write, full hook ─────────

    fn plan_freeze_fixture(feature: &str, shape: bool) -> Fx {
        let dir = tempfile::tempdir().unwrap();
        let root = dunce::canonicalize(dir.path()).unwrap();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        copy_lib(&root);
        let st = json!({
            "phase": "swarming",
            "mode": "standard",
            "feature": feature,
            "approved_gates": { "context": true, "shape": shape, "execution": true, "review": false }
        });
        write_state(&root, &st);
        Fx { _dir: dir, root }
    }

    #[test]
    fn plan_md_denied_once_shape_is_approved_pinned_message() {
        let fx = plan_freeze_fixture("demo", true);
        let e = expect_done(edit("docs/history/demo/plan.md"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        for needle in [
            "bee plan-freeze guard",
            "docs/history/demo/plan.md",
            "bee state plan-rev bump --lane demo",
            "unapprove the shape gate",
            "FIX",
        ] {
            assert!(e.stderr.contains(needle), "missing {needle}: {}", e.stderr);
        }
    }

    #[test]
    fn plan_md_allowed_when_shape_not_approved() {
        let fx = plan_freeze_fixture("demo", false);
        let e = expect_done(edit("docs/history/demo/plan.md"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn plan_md_allowed_for_another_feature_even_when_the_active_feature_is_approved() {
        // The default state's own feature ("demo") has shape approved, but
        // the write targets a DIFFERENT feature's plan.md with no lane
        // record of its own — resolution is scoped to the path's feature,
        // never the calling session's, so this reads as no opinion (allow).
        let fx = plan_freeze_fixture("demo", true);
        let e = expect_done(edit("docs/history/other-feature/plan.md"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn context_md_is_never_frozen_even_when_shape_is_approved() {
        let fx = plan_freeze_fixture("demo", true);
        let e = expect_done(edit("docs/history/demo/CONTEXT.md"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    #[test]
    fn plan_md_deny_wiring_honors_the_lane_record_over_default_state() {
        // Full-hook proof that check_write actually calls the lane-aware
        // resolver: default state says shape approved, but the lane record
        // (the CLI's own view) says otherwise — the lane wins, allow.
        let fx = plan_freeze_fixture("demo", true);
        write_lane(&fx.root, "demo", false);
        let e = expect_done(edit("docs/history/demo/plan.md"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }

    // ── write-guard-hardening D4: brace expansion and cd opacity are
    // unresolvable; globs stay untouched ────────────────────────────────────

    #[test]
    fn brace_expansion_classifier_matches_comma_and_range_forms() {
        assert!(has_brace_expansion("sta{t,t}e.json"));
        assert!(has_brace_expansion("f{1..9}.txt"));
        assert!(has_brace_expansion("f{a..z}.txt"));
        // a nested comma group still carries a comma somewhere inside the
        // outer braces — still classified, conservatively.
        assert!(has_brace_expansion("a{b,{c,d}}e"));
    }

    #[test]
    fn brace_expansion_classifier_leaves_singleton_braces_and_globs_alone() {
        // A brace group with neither a comma nor a `..` range is not
        // expanded by bash — a literal `{foo}` filename.
        assert!(!has_brace_expansion("{foo}"));
        assert!(!has_brace_expansion("plain.txt"));
        // Plain glob characters are deliberately out of scope for D4.
        assert!(!has_brace_expansion("*.log"));
        assert!(!has_brace_expansion("file?.txt"));
        assert!(!has_brace_expansion("[abc].txt"));
        assert!(!has_brace_expansion("./*"));
    }

    #[test]
    fn brace_expansion_message_quotes_the_bounded_token() {
        let msg = brace_expansion_bash_target_message("sta{t,t}e.json");
        assert!(msg.contains("\"sta{t,t}e.json\""), "{}", msg);
        assert!(msg.contains("brace expansion"), "{}", msg);
    }

    #[test]
    fn cd_opaque_message_quotes_the_bounded_token() {
        let msg = cd_opaque_bash_target_message("out.txt");
        assert!(msg.contains("\"out.txt\""), "{}", msg);
        assert!(msg.contains("cd"), "{}", msg);
    }

    #[test]
    fn extract_bash_targets_marks_cd_opaque_only_for_targets_after_cd() {
        // No cd at all — every target is native.
        let t = extract_bash_targets("touch out.txt");
        assert_eq!(t.paths, vec!["out.txt"]);
        assert_eq!(t.cd_opaque, vec![false]);

        // The cd comes first — the target after it is opaque.
        let t = extract_bash_targets("cd /tmp && touch out.txt");
        assert_eq!(t.paths, vec!["out.txt"]);
        assert_eq!(t.cd_opaque, vec![true]);

        // The cd comes AFTER the target — unaffected.
        let t = extract_bash_targets("touch out.txt && cd /tmp");
        assert_eq!(t.paths, vec!["out.txt"]);
        assert_eq!(t.cd_opaque, vec![false]);

        // Mixed: one target before, one after.
        let t = extract_bash_targets("touch before.txt && cd /tmp && touch after.txt");
        assert_eq!(t.paths, vec!["before.txt", "after.txt"]);
        assert_eq!(t.cd_opaque, vec![false, true]);
    }

    #[test]
    fn bash_brace_comma_target_denies_naming_brace_expansion() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("touch .bee/sta{t,t}e.json"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("brace expansion"), "{}", e.stderr);
        assert!(e.stderr.contains(".bee/sta{t,t}e.json"), "{}", e.stderr);
        // Never the containment or direct-edit wording — this target was
        // never resolved as a literal path in the first place.
        assert!(!e.stderr.contains("direct-edit"), "{}", e.stderr);
    }

    #[test]
    fn bash_brace_range_target_denies_naming_brace_expansion() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("touch f{1..9}.txt"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("brace expansion"), "{}", e.stderr);
        assert!(e.stderr.contains("f{1..9}.txt"), "{}", e.stderr);
    }

    #[test]
    fn bash_cd_then_write_denies_naming_cd_opacity() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("cd /tmp && touch out.txt"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("cd"), "{}", e.stderr);
        assert!(e.stderr.contains("out.txt"), "{}", e.stderr);
        assert!(e.stderr.contains("could not be resolved"), "{}", e.stderr);
    }

    #[test]
    fn bash_write_before_cd_keeps_its_own_denial_wording() {
        // A write BEFORE the cd is unaffected by D4 — it keeps whatever
        // verdict it always had (here: the pre-existing direct-edit denial
        // for .bee/state.json), never the cd-opacity wording.
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("printf x > .bee/state.json && cd /tmp"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee state"), "{}", e.stderr);
        assert!(!e.stderr.contains("could not be resolved"), "{}", e.stderr);
    }

    // P1-2: bash sets up redirections before it runs a builtin — a redirect
    // glued into a cd segment (`cd . > probe.json`) truncates the file for
    // real, regardless of where cd lands, so it must extract as a native
    // (non-cd-opaque) target and deny like any other direct write.
    #[test]
    fn extract_bash_targets_extracts_a_redirect_inside_a_cd_segment() {
        let t = extract_bash_targets("cd . > probe.json");
        assert_eq!(t.paths, vec!["probe.json"], "{:?}", t.paths);
        assert_eq!(t.cd_opaque, vec![false], "redirect target must not be cd-opaque");
    }

    #[test]
    fn bash_cd_segment_redirect_denies_the_protected_file_again() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("cd . > .bee/state.json"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee direct-edit guard"), "{}", e.stderr);
        assert!(e.stderr.contains(".bee/state.json"), "{}", e.stderr);
        // Must be the direct-edit denial, never the cd-opacity wording — the
        // redirect is a real write that bash sets up before running cd.
        assert!(!e.stderr.contains("could not be resolved"), "{}", e.stderr);
    }

    // P1-3: a separator right after -t/--target-directory is never its
    // argument — swallowing it used to merge the NEXT command's tokens into
    // this cp's operand list, hiding that command's own write target.
    #[test]
    fn extract_bash_targets_cp_target_directory_flag_never_eats_a_separator() {
        let t = extract_bash_targets("cp -t ; rm -rf .bee/state.json");
        assert_eq!(t.paths, vec![".bee/state.json"], "{:?}", t.paths);
        let t = extract_bash_targets("mv -t ; rm -rf .bee/state.json");
        assert_eq!(t.paths, vec![".bee/state.json"], "{:?}", t.paths);
    }

    #[test]
    fn bash_cp_target_directory_separator_swallow_denies_the_next_command_again() {
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("cp -t ; rm -rf .bee/state.json"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("bee direct-edit guard"), "{}", e.stderr);
        assert!(e.stderr.contains(".bee/state.json"), "{}", e.stderr);
    }

    #[test]
    fn bash_plain_glob_target_behaves_exactly_as_before_d4() {
        // rm *.log — a plain glob character is deliberately NOT classified
        // by D4; this must resolve exactly as it always has (denied here by
        // the pre-existing scratch-shape guard on the resolved ".log"
        // target, never by the new brace-expansion/cd-opacity wording).
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("rm *.log"), &fx.root);
        assert_eq!(e.code, 2, "{}", e.stderr);
        assert!(e.stderr.contains("scratch-shape guard"), "{}", e.stderr);
        assert!(!e.stderr.contains("brace expansion"), "{}", e.stderr);
        assert!(!e.stderr.contains("could not be resolved"), "{}", e.stderr);
    }

    #[test]
    fn bash_plain_glob_write_outside_scratch_shape_allows_as_before_d4() {
        // A glob target that resolves to a plain path with no glob-specific
        // guard opinion (unlike ".log") must still resolve and allow.
        let fx = build_fixture("swarming", true);
        let e = expect_done(bash("rm *.txt"), &fx.root);
        assert_eq!(e.code, 0, "{}", e.stderr);
    }
