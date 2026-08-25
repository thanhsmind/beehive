// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
//
// Moved verbatim out of the parent file's `#[cfg(test)] mod tests`,
// indentation and all: the fixtures are raw strings whose leading
// whitespace is content.

// The parent module's own `use` block came with the tests: they reach for
// `rsv`, `lock` and `Ordering`, which mod.rs no longer imports now that the
// code using them lives in sibling modules.
#![allow(unused_imports)]

use crate::fsutil::{self, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt, StoreRoots};
use crate::state as bstate;
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::{Err2, FlagV, Out, R2};
use crate::verbs::workspace_store as ws;
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
    use super::*;
    use serde_json::json;

    fn write_cell_fixture(root: &Path, id: &str, body: &Value) {
        let dir = cells_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), jsjson::stringify_pretty(body)).unwrap();
    }

    fn read_cell_fixture(root: &Path, id: &str) -> Value {
        match read_json(&cells_dir(root).join(format!("{id}.json"))) {
            ReadJson::Parsed(v) => v,
            ReadJson::Missing => panic!("cell {id} fixture missing"),
            ReadJson::Corrupt => panic!("cell {id} fixture corrupt"),
        }
    }

    /// A raw cell written STRAIGHT TO DISK — it never passes through
    /// `validate_new_cell`, so D7's required `role` is deliberately absent
    /// here and adding one would be noise on the ~130 tests that only claim,
    /// close, schedule or list this fixture. The rule (recorded on cell
    /// `mrs-8`): a fixture whose test RESOLVES or DISPATCHES it names its
    /// role explicitly at the call site — see the `dispatch wave` and
    /// `claim_and_reserve_for_dispatch` tests below, which set `tier` and
    /// `role` together for exactly that reason. A fixture that is never
    /// resolved stays roleless on purpose; the alternative is a blanket
    /// default that would hide the very silence this feature exists to end.
    fn cell(id: &str, status: &str, feature: &str, deps: Value) -> Value {
        json!({
            "id": id,
            "title": format!("title {id}"),
            "status": status,
            "lane": "tiny",
            "feature": feature,
            "deps": deps,
            "verify": "echo ok",
        })
    }

    /// D8 (docs/history/test-doctrine/CONTEXT.md): a minimal, always-valid
    /// `--report` JSON blob for fixtures that exercise something OTHER than
    /// the --report contract itself (D6 trailer, wp registered-worker,
    /// boundary-sentinel, frd deviation tests) — now that --report is
    /// required on every cap path, these fixtures need SOME valid report
    /// even though they are not testing its shape.
    fn default_test_report_json() -> String {
        r#"{"outcome":"o","commit":"c","files":[],"tests":"cargo test -p bee — green — fixture","deviations":[]}"#
            .to_string()
    }

    // ── natural sort: every pair below is pinned to a live V8
    //    `a.localeCompare(b, 'en', {numeric: true})` probe result. ──────────
    #[test]
    fn natural_cmp_matches_v8_locale_compare_probes() {
        use Ordering::{Equal, Greater, Less};
        let probes: &[(&str, &str, Ordering)] = &[
            ("a01", "a1", Equal),
            ("01", "1", Equal),
            ("a00", "a0", Equal),
            ("a001b", "a1b", Equal),
            ("a2", "a10", Less),
            ("f1-2", "f1-10", Less),
            ("w-2", "w-10", Less),
            ("a1b2", "a1b10", Less),
            ("a10b", "a9c", Greater),
            ("a0", "a", Greater),
            ("a1", "a1a", Less),
            ("a", "a-", Less),
            ("x", "xx", Less),
            ("a-1", "a.1", Less),
            ("a-1", "a_1", Greater),
            ("a.1", "a_1", Greater),
            ("-", ".", Less),
            (".", "_", Greater),
            ("_", "-", Less),
            ("a", "1", Greater),
            ("0", "-", Greater),
            ("aa", "a-a", Greater),
            ("abc", "ab-c", Greater),
            ("a-2", "a2", Less),
            ("x-1", "x1", Less),
            ("x.y", "xy", Less),
            ("x_y", "x-y", Less),
            ("a b", "a_b", Less),
            ("a 1", "a-1", Less),
            ("a", "A", Less),
            ("A", "a", Greater),
            ("aB", "ab", Greater),
            ("A1", "a1", Greater),
            ("Ab", "aC", Less),     // primary (b<c) beats the earlier case diff
            ("a01B", "a1b", Greater), // digits tie; tertiary B>b
            ("a01b", "A1b", Less),  // tertiary a<A, digits carry no case weight
            ("a01x", "a1X", Less),
            ("ABC", "abd", Less),
            ("demo-1", "demo-1", Equal),
        ];
        for (a, b, want) in probes {
            assert_eq!(natural_cmp(a, b), *want, "natural_cmp({a:?}, {b:?})");
        }
    }

    #[test]
    fn list_cells_sorts_naturally_and_skips_non_cells() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "w-10", &cell("w-10", "open", "f", json!([])));
        write_cell_fixture(root, "w-2", &cell("w-2", "open", "f", json!([])));
        write_cell_fixture(root, "a-1", &cell("a-1", "capped", "g", json!([])));
        // Non-.json and directory entries are never cells.
        std::fs::write(cells_dir(root).join("notes.txt"), "x").unwrap();
        std::fs::create_dir_all(cells_dir(root).join(ARCHIVE_DIR_NAME).join("old")).unwrap();
        // A literal-null cell file and a primitive cell file are skipped.
        std::fs::write(cells_dir(root).join("nul.json"), "null").unwrap();
        std::fs::write(cells_dir(root).join("num.json"), "5").unwrap();
        let ids: Vec<String> = list_cells(root, None, None)
            .unwrap_or_else(|_| panic!("no delegate expected"))
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["a-1", "w-2", "w-10"]);
    }

    #[test]
    fn list_cells_filters_by_feature_and_status_strictly() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "f-1", &cell("f-1", "open", "feat", json!([])));
        write_cell_fixture(root, "f-2", &cell("f-2", "capped", "feat", json!([])));
        write_cell_fixture(root, "g-1", &cell("g-1", "open", "other", json!([])));
        // A cell with NO feature field never matches a truthy filter.
        write_cell_fixture(root, "h-1", &json!({"id": "h-1", "status": "open"}));
        let feat_open: Vec<String> = list_cells(root, Some("feat"), Some("open"))
            .unwrap_or_else(|_| panic!())
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(feat_open, vec!["f-1"]);
        let all_open: Vec<String> = list_cells(root, None, Some("open"))
            .unwrap_or_else(|_| panic!())
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(all_open, vec!["f-1", "g-1", "h-1"]);
    }

    #[test]
    fn list_cells_skips_corrupt_cell_and_delegates_on_array_cell() {
        // CUTOVER: a corrupt cell file is no longer a delegation. readJson
        // warns and returns null, `!cell` skips it, and the rest of the store
        // still lists — exactly Node's fail-open.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        std::fs::write(cells_dir(root).join("bad.json"), "{nope").unwrap();
        write_cell_fixture(root, "good-1", &cell("good-1", "open", "f", json!([])));
        let listed = list_cells(root, None, None).expect("corrupt JSON must not delegate");
        let ids: Vec<String> = listed.iter().map(|c| js_string_or_undefined(c.get("id"))).collect();
        assert_eq!(ids, vec!["good-1"], "the corrupt file is skipped, the good one survives");

        // A lone-surrogate escape (V8's JSON.parse took it; serde never can)
        // is just corrupt input now — same skip, no delegation.
        let tmp3 = tempfile::tempdir().unwrap();
        let root3 = tmp3.path();
        std::fs::create_dir_all(cells_dir(root3)).unwrap();
        std::fs::write(cells_dir(root3).join("sur.json"), r#"{"id":"sur-1","title":"\ud800"}"#).unwrap();
        write_cell_fixture(root3, "good-2", &cell("good-2", "open", "f", json!([])));
        let listed = list_cells(root3, None, None).expect("lone surrogate must not delegate");
        let ids: Vec<String> = listed.iter().map(|c| js_string_or_undefined(c.get("id"))).collect();
        assert_eq!(ids, vec!["good-2"]);

        // JS-exotic shapes are NOT in that class and still delegate.
        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        std::fs::create_dir_all(cells_dir(root2)).unwrap();
        std::fs::write(cells_dir(root2).join("arr.json"), "[1,2]").unwrap();
        assert!(list_cells(root2, None, None).is_err(), "array cell (typeof 'object') must delegate");
    }

    // ── readiness (depsAllCapped) ──────────────────────────────────────────
    #[test]
    fn ready_requires_every_dep_capped() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "base-1", &cell("base-1", "capped", "f", json!([])));
        write_cell_fixture(root, "base-2", &cell("base-2", "open", "f", json!([])));
        write_cell_fixture(root, "ok-1", &cell("ok-1", "open", "f", json!(["base-1"])));
        write_cell_fixture(root, "wait-1", &cell("wait-1", "open", "f", json!(["base-1", "base-2"])));
        write_cell_fixture(root, "ghost-1", &cell("ghost-1", "open", "f", json!(["missing-9"])));
        write_cell_fixture(root, "free-1", &cell("free-1", "open", "f", json!([])));
        // deps: falsy value behaves as [] (readiness unconditional).
        write_cell_fixture(root, "nul-1", &json!({"id": "nul-1", "status": "open", "deps": null}));
        let Handled::Emit { result, text } = handle_ready(root, None).unwrap() else {
            panic!("ready never errors")
        };
        let ids: Vec<String> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["base-2", "free-1", "nul-1", "ok-1"]);
        assert!(text.contains("ok-1 [open] (tiny) title ok-1"));
    }

    #[test]
    fn ready_counts_archived_capped_dep_via_read_cell_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let arch = cells_dir(root).join(ARCHIVE_DIR_NAME).join("done-feature");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(
            arch.join("old-1.json"),
            jsjson::stringify_pretty(&cell("old-1", "capped", "done-feature", json!([]))),
        )
        .unwrap();
        write_cell_fixture(root, "next-1", &cell("next-1", "open", "f", json!(["old-1"])));
        let Handled::Emit { result, .. } = handle_ready(root, None).unwrap() else { panic!() };
        let ids: Vec<String> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["next-1"], "archived capped dep satisfies readiness");
    }

    #[test]
    fn ready_delegates_on_truthy_non_array_deps_and_falsy_dep_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "s-1", &json!({"id": "s-1", "status": "open", "deps": "x-1"}));
        assert!(handle_ready(root, None).is_err(), "string deps (char iteration) delegates");

        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        write_cell_fixture(root2, "z-1", &json!({"id": "z-1", "status": "open", "deps": [""]}));
        let Handled::Emit { result, text } = handle_ready(root2, None).unwrap() else { panic!() };
        assert_eq!(result.as_array().unwrap().len(), 0, "falsy dep never resolves -> not ready");
        assert_eq!(text, "No ready cells.");
    }

    // ── renderers ──────────────────────────────────────────────────────────
    #[test]
    fn renderers_match_node_templates_and_empty_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let Handled::Emit { text, .. } = handle_list(root, None, None).unwrap() else { panic!() };
        assert_eq!(text, "No cells.");
        let Handled::Emit { text, .. } = handle_ready(root, None).unwrap() else { panic!() };
        assert_eq!(text, "No ready cells.");

        // Missing fields coerce like template literals: "undefined".
        write_cell_fixture(root, "bare-1", &json!({"id": "bare-1"}));
        let Handled::Emit { text, .. } = handle_list(root, None, None).unwrap() else { panic!() };
        assert_eq!(text, "bare-1 [undefined] (undefined) undefined");
    }

    // ── show: trace assembly + verify_owner placement + error path ─────────
    #[test]
    fn show_inserts_verify_owner_after_verify_preserving_trace_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let body = json!({
            "id": "t-1",
            "title": "t",
            "status": "capped",
            "verify": "run checks",
            "trace": {
                "claimed_by": "w1",
                "attempts": [{"n": 1, "ok": false}, {"n": 2, "ok": true}],
                "verify_passed": true
            }
        });
        write_cell_fixture(root, "t-1", &body);
        let Handled::Emit { result, text } = handle_show(root, "t-1").unwrap() else { panic!() };
        let keys: Vec<&String> = result.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["id", "title", "status", "verify", "verify_owner", "trace"]);
        assert_eq!(
            result.get("verify_owner"),
            Some(&Value::String(VERIFY_OWNER_ANNOTATION.into()))
        );
        // text is the pretty render of the SAME annotated object, trace intact.
        assert_eq!(text, jsjson::stringify_pretty(&result));
        assert!(text.contains("\"attempts\": ["));
    }

    #[test]
    fn show_appends_verify_owner_when_cell_has_no_verify_key() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "nv-1", &json!({"id": "nv-1", "status": "open"}));
        let Handled::Emit { result, .. } = handle_show(root, "nv-1").unwrap() else { panic!() };
        let keys: Vec<&String> = result.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["id", "status", "verify_owner"]);
    }

    #[test]
    fn show_annotates_claimed_cell_with_live_session() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let body = json!({
            "id": "c-1",
            "title": "c",
            "status": "claimed",
            "verify": "run checks",
            "trace": default_trace()
        });
        write_cell_fixture(root, "c-1", &body);
        let now_iso = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();
        write_session_fixture(root, "sess-1", &now_iso, Some("lane-1"));
        write_claim_fixture(root, "c-1", Some("sess-1"), 3600.0, &now_iso);

        let Handled::Emit { result, .. } = handle_show(root, "c-1").unwrap() else { panic!() };
        let claim = result.get("claim").expect("claim annotation must be present");
        assert_eq!(claim.get("session"), Some(&json!("sess-1")));
        assert_eq!(claim.get("holder_alive"), Some(&json!(true)));
        assert_eq!(claim.get("verdict"), Some(&json!("held")));
        assert_eq!(claim.get("expired"), Some(&json!(false)));
        assert!(claim.get("expiry").unwrap().as_str().unwrap().starts_with("expires "));
        let keys: Vec<&String> = result.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["id", "title", "status", "verify", "verify_owner", "trace", "claim"]);
    }

    #[test]
    fn show_annotates_claimed_cell_with_closed_session_as_holder_not_alive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let body = json!({
            "id": "c-closed",
            "title": "c",
            "status": "claimed"
        });
        write_cell_fixture(root, "c-closed", &body);
        let now_iso = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();
        let s_dir = sessions_dir(root);
        std::fs::create_dir_all(&s_dir).unwrap();
        std::fs::write(
            s_dir.join("sess-closed.json"),
            jsjson::stringify_pretty(&json!({
                "id": "sess-closed",
                "status": "closed",
                "last_heartbeat": now_iso,
                "started_at": now_iso
            })),
        )
        .unwrap();
        write_claim_fixture(root, "c-closed", Some("sess-closed"), 3600.0, &now_iso);

        let Handled::Emit { result, .. } = handle_show(root, "c-closed").unwrap() else { panic!() };
        let claim = result.get("claim").expect("claim annotation must be present");
        assert_eq!(claim.get("session"), Some(&json!("sess-closed")));
        assert_eq!(claim.get("holder_alive"), Some(&json!(false)));
        assert_eq!(claim.get("verdict"), Some(&json!("held")));
        assert_eq!(claim.get("expired"), Some(&json!(false)));
    }

    #[test]
    fn show_annotates_expired_claim_sweepable_when_holder_stale_and_held_when_holder_alive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let old_iso = "2020-01-01T00:00:00.000Z";
        let now_iso = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();

        // 1. Expired claim with stale session -> sweepable
        write_cell_fixture(root, "c-exp-stale", &json!({"id": "c-exp-stale", "status": "claimed"}));
        write_session_fixture(root, "sess-stale", old_iso, None);
        write_claim_fixture(root, "c-exp-stale", Some("sess-stale"), 10.0, old_iso);

        let Handled::Emit { result: res_stale, .. } = handle_show(root, "c-exp-stale").unwrap() else { panic!() };
        let claim_stale = res_stale.get("claim").unwrap();
        assert_eq!(claim_stale.get("expired"), Some(&json!(true)));
        assert_eq!(claim_stale.get("holder_alive"), Some(&json!(false)));
        assert_eq!(claim_stale.get("verdict"), Some(&json!("sweepable")));

        // 2. Expired claim with live session -> held
        write_cell_fixture(root, "c-exp-live", &json!({"id": "c-exp-live", "status": "claimed"}));
        write_session_fixture(root, "sess-live", &now_iso, None);
        write_claim_fixture(root, "c-exp-live", Some("sess-live"), 10.0, old_iso);

        let Handled::Emit { result: res_live, .. } = handle_show(root, "c-exp-live").unwrap() else { panic!() };
        let claim_live = res_live.get("claim").unwrap();
        assert_eq!(claim_live.get("expired"), Some(&json!(true)));
        assert_eq!(claim_live.get("holder_alive"), Some(&json!(true)));
        assert_eq!(claim_live.get("verdict"), Some(&json!("held")));
    }

    #[test]
    fn show_annotates_sessionless_claim_with_session_null() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "c-nosess", &json!({"id": "c-nosess", "status": "claimed"}));
        let now_iso = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();
        write_claim_fixture(root, "c-nosess", None, 3600.0, &now_iso);

        let Handled::Emit { result, .. } = handle_show(root, "c-nosess").unwrap() else { panic!() };
        let claim = result.get("claim").expect("claim annotation must be present");
        assert_eq!(claim.get("session"), Some(&Value::Null));
        assert_eq!(claim.get("holder_alive"), Some(&json!(false)));
        assert_eq!(claim.get("verdict"), Some(&json!("held")));
    }

    #[test]
    fn list_annotates_claimed_row_and_leaves_unclaimed_row_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let cell_unclaimed = json!({
            "id": "u-1",
            "title": "unclaimed cell",
            "status": "open",
            "lane": "small"
        });
        let cell_claimed = json!({
            "id": "c-1",
            "title": "claimed cell",
            "status": "claimed",
            "lane": "small"
        });
        write_cell_fixture(root, "u-1", &cell_unclaimed);
        write_cell_fixture(root, "c-1", &cell_claimed);

        let now_iso = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();
        write_session_fixture(root, "sess-1", &now_iso, None);
        write_claim_fixture(root, "c-1", Some("sess-1"), 3600.0, &now_iso);

        let Handled::Emit { result, .. } = handle_list(root, None, None).unwrap() else { panic!() };
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        let u1 = arr.iter().find(|c| c.get("id") == Some(&json!("u-1"))).unwrap();
        let c1 = arr.iter().find(|c| c.get("id") == Some(&json!("c-1"))).unwrap();

        assert_eq!(u1.get("claim"), None);
        let u1_keys: Vec<&String> = u1.as_object().unwrap().keys().collect();
        assert_eq!(u1_keys, vec!["id", "title", "status", "lane"]);

        assert!(c1.get("claim").is_some());
        let c1_keys: Vec<&String> = c1.as_object().unwrap().keys().collect();
        assert_eq!(c1_keys, vec!["id", "title", "status", "lane", "claim"]);
    }

    #[test]
    fn summarize_cell_renders_holder_suffix_for_claimed_and_old_line_for_unclaimed() {
        let unclaimed = json!({
            "id": "u-1",
            "status": "open",
            "lane": "small",
            "title": "My title"
        });
        assert_eq!(summarize_cell(&unclaimed), "u-1 [open] (small) My title");

        let claimed_held = json!({
            "id": "c-1",
            "status": "claimed",
            "lane": "small",
            "title": "My title",
            "claim": {
                "session": "s1",
                "verdict": "held",
                "holder_alive": true,
                "expiry": "expires 2026-08-21T18:00:00.000Z"
            }
        });
        assert_eq!(
            summarize_cell(&claimed_held),
            "c-1 [claimed] (small) My title — held by session s1"
        );

        let claimed_held_dead = json!({
            "id": "c-1",
            "status": "claimed",
            "lane": "small",
            "title": "My title",
            "claim": {
                "session": "s1",
                "verdict": "held",
                "holder_alive": false,
                "expiry": "expires 2026-08-21T18:00:00.000Z"
            }
        });
        assert_eq!(
            summarize_cell(&claimed_held_dead),
            "c-1 [claimed] (small) My title — held by session s1 (holder not alive, claim still valid until expires 2026-08-21T18:00:00.000Z)"
        );

        let claimed_sessionless = json!({
            "id": "c-1",
            "status": "claimed",
            "lane": "small",
            "title": "My title",
            "claim": {
                "session": null,
                "verdict": "held",
                "holder_alive": true,
                "expiry": "no expiry"
            }
        });
        assert_eq!(
            summarize_cell(&claimed_sessionless),
            "c-1 [claimed] (small) My title — held by sessionless claim"
        );

        let claimed_sweepable = json!({
            "id": "c-1",
            "status": "claimed",
            "lane": "small",
            "title": "My title",
            "claim": {
                "session": "s1",
                "verdict": "sweepable",
                "holder_alive": false,
                "expiry": "expires 2020-01-01T00:00:00.000Z"
            }
        });
        assert_eq!(
            summarize_cell(&claimed_sweepable),
            "c-1 [claimed] (small) My title — claim expired and holder not alive (sweepable)"
        );
    }

    #[test]
    fn show_and_list_delegate_when_claim_file_is_json_array() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "c-arr", &json!({"id": "c-arr", "status": "claimed"}));
        let c_dir = claims_dir(root);
        std::fs::create_dir_all(&c_dir).unwrap();
        std::fs::write(c_dir.join("c-arr.json"), "[1, 2, 3]").unwrap();

        assert!(handle_show(root, "c-arr").is_err(), "show must delegate on json array claim");
        assert!(handle_list(root, None, None).is_err(), "list must delegate on json array claim");
    }

    #[test]
    fn show_not_found_message_matches_node_and_reads_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        match handle_show(root, "nope-1").unwrap() {
            Handled::Error(msg) => assert_eq!(msg, "Cell \"nope-1\" not found."),
            _ => panic!("expected the not-found error"),
        }
        // Malformed id short-circuits to the same message (ID_PATTERN).
        match handle_show(root, "../evil").unwrap() {
            Handled::Error(msg) => assert_eq!(msg, "Cell \"../evil\" not found."),
            _ => panic!("expected the not-found error"),
        }
        // Archived cell resolves through the archive fallback.
        let arch = cells_dir(root).join(ARCHIVE_DIR_NAME).join("f");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(
            arch.join("arc-1.json"),
            jsjson::stringify_pretty(&cell("arc-1", "capped", "f", json!([]))),
        )
        .unwrap();
        match handle_show(root, "arc-1").unwrap() {
            Handled::Emit { result, .. } => {
                assert_eq!(result.get("id"), Some(&Value::String("arc-1".into())))
            }
            _ => panic!("archived cell must resolve"),
        }
    }

    #[test]
    fn show_reports_not_found_on_corrupt_cell_and_delegates_on_non_object() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        // CUTOVER: readCell warns and falls back to null, so `show` reaches
        // the SAME not-found refusal a missing cell reaches — no delegation.
        std::fs::write(cells_dir(root).join("bad-1.json"), "{nope").unwrap();
        match handle_show(root, "bad-1").expect("corrupt cell must not delegate") {
            Handled::Error(msg) => assert_eq!(msg, "Cell \"bad-1\" not found."),
            _ => panic!("corrupt cell must take readCell's null fallback"),
        }
        // Same for a lone-surrogate escape.
        std::fs::write(cells_dir(root).join("sur-1.json"), r#"{"id":"sur-1","t":"\udfff"}"#).unwrap();
        match handle_show(root, "sur-1").expect("lone surrogate must not delegate") {
            Handled::Error(msg) => assert_eq!(msg, "Cell \"sur-1\" not found."),
            _ => panic!("lone-surrogate cell must take readCell's null fallback"),
        }
        std::fs::write(cells_dir(root).join("num-1.json"), "5").unwrap();
        assert!(handle_show(root, "num-1").is_err(), "truthy non-object cell delegates");
    }

    // ── argv routing ───────────────────────────────────────────────────────
    #[test]
    fn parse_flags_accepts_only_provable_shapes() {
        let os = |v: &[&str]| v.iter().map(OsString::from).collect::<Vec<_>>();
        // list: --json --feature f --status open (both flag forms).
        let f = parse_flags(Verb::List, &os(&["--json", "--feature", "f", "--status=open"])).unwrap();
        assert!(f.json);
        assert_eq!(f.feature.as_deref(), Some("f"));
        assert_eq!(f.status.as_deref(), Some("open"));
        // last-wins overwrite, like Node's flags[name] = value.
        let f = parse_flags(Verb::List, &os(&["--feature=a", "--feature=b"])).unwrap();
        assert_eq!(f.feature.as_deref(), Some("b"));
        // Delegations: bare positional, unknown flag, --help, missing value,
        // a `--`-shaped value token, and per-verb flag sets.
        assert!(parse_flags(Verb::List, &os(&["foo"])).is_none());
        assert!(parse_flags(Verb::List, &os(&["--bogus", "x"])).is_none());
        assert!(parse_flags(Verb::List, &os(&["--help"])).is_none());
        assert!(parse_flags(Verb::List, &os(&["--feature"])).is_none());
        assert!(parse_flags(Verb::List, &os(&["--feature", "--json"])).is_none());
        assert!(parse_flags(Verb::Ready, &os(&["--status", "open"])).is_none());
        assert!(parse_flags(Verb::Show, &os(&["--feature", "f"])).is_none());
        // show requires --id at try_native level; parse itself allows any value.
        let f = parse_flags(Verb::Show, &os(&["--id", "-weird"])).unwrap();
        assert_eq!(f.id.as_deref(), Some("-weird"));
    }

    // ═══ mutating-verb building blocks ════════════════════════════════════

    fn thrown<T>(r: MR<T>) -> String {
        match r {
            Err(Fail::Thrown(m)) => m,
            Err(Fail::Delegate) => panic!("unexpected delegate"),
            Ok(_) => panic!("expected a thrown refusal"),
        }
    }

    // ── failure-signature normalizer: pinned against live Node runs of
    //    lib/cells.mjs normalizeFailureSignature. ──────────────────────────
    #[test]
    fn failure_signature_matches_node_vectors() {
        let vectors: &[(&str, &str)] = &[
            ("boom", "81f52337ebb4"),
            ("", "e3b0c44298fc"),
            ("ok line\nError: deadbeef00 at /home/u/repo/file.js", "dc04ab11120d"),
            ("3/45 passed\nrefused: cap denied", "9b9c6fc6eefa"),
            ("Error at abc123 deadbeefcafe1234", "667b748a5aff"),
            ("  Error:   spaced   ", "c8165beb8597"),
            ("no failure words here\nsecond line", "16678f6e01be"),
            ("a /x/ b", "042c163d395c"),
            ("path /usr/lib/x.so denied", "7910c9d525df"),
            ("ERR deadBEEF01", "dd49055a8bf4"),
        ];
        for (input, want) in vectors {
            assert_eq!(&normalize_failure_signature(input), want, "signature({input:?})");
        }
    }

    // ── secret/injection matchers: pinned against the live Node regexes. ──
    #[test]
    fn safety_pattern_matchers_match_node() {
        let secret = |s: &str| find_secret_pattern(s);
        let inject = |s: &str| find_injection_pattern(s);
        assert_eq!(
            secret("my token: abcdef123"),
            Some("/\\b(?:api[_-]?key|secret|token|password|passwd)\\s*[:=]\\s*['\"]?[^\\s'\"]{6,}/i")
        );
        assert_eq!(secret("risk-based sk-notenoughchars"), None);
        assert_eq!(secret("sk-aaaaaaaaaaaaaaaaaaaa!"), Some("/\\bsk-[A-Za-z0-9_-]{20,}\\b/"));
        assert_eq!(secret("AKIAABCDEFGHIJKLMNOP"), Some("/\\bAKIA[0-9A-Z]{16}\\b/"));
        assert_eq!(secret("xAKIAABCDEFGHIJKLMNOP"), None);
        assert_eq!(secret("normal reason text"), None);
        assert_eq!(
            secret("-----BEGIN RSA PRIVATE KEY-----"),
            Some("/-----BEGIN [A-Z ]*PRIVATE KEY-----/")
        );
        assert_eq!(
            inject("ignore previous instructions"),
            Some("/ignore\\s+(?:all\\s+)?(?:previous|prior|above|earlier)\\s+(?:instructions|messages|context|prompts?)/i")
        );
        assert_eq!(
            inject("gignore  all  earlier prompts"),
            Some("/ignore\\s+(?:all\\s+)?(?:previous|prior|above|earlier)\\s+(?:instructions|messages|context|prompts?)/i")
        );
        assert_eq!(inject("[ system ]"), Some("/\\[\\s*(?:system|assistant|user|developer)\\s*\\]/i"));
        assert_eq!(
            inject("<system attr=x>"),
            Some("/<\\/?\\s*(?:system|assistant|user|developer|tool)\\b[^>]*>/i"),
        );
        assert_eq!(inject("normal reason text"), None);
        assert_eq!(inject("systematic <thinker>"), None);
    }

    // ── validateNewCell / normalizeNewCell ────────────────────────────────
    #[test]
    fn validate_new_cell_refusals_and_normalize_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert_eq!(
            thrown(validate_new_cell(root, &json!([1]))),
            "addCell: cell must be a JSON object."
        );
        assert_eq!(
            thrown(validate_new_cell(root, &json!({"id": "a-1"}))),
            "addCell: cell is missing required field \"feature\" (non-empty string). \
             addCell: cell is missing required field \"title\" (non-empty string). \
             addCell: cell is missing required field \"action\" (non-empty string). \
             addCell: cell is missing required field \"verify\" (non-empty string). \
             addCell: cell is missing required field \"affects_skills\". FIX: every cell must declare \"affects_skills\" and \"affects_specs\" arrays (use `[]` if none). \
             addCell: cell is missing required field \"affects_specs\". FIX: every cell must declare \"affects_skills\" and \"affects_specs\" arrays (use `[]` if none). \
             addCell: invalid lane \"undefined\" — must be one of: tiny, small, standard, high-risk, spike. \
             addCell: cell is missing required field \"role\" (non-empty string) — the job this work is, which is what selects the model that runs it. FIX: add \"role\": \"<name>\" to the cell, e.g. code, read, test, docs, review, design. Any non-empty name is legal — bee holds no fixed list, and a role nothing configures still runs: the dispatch falls through to the next name it asked for and warns. The one silent case is \"code\" or \"read\" on a runtime whose models.<runtime> configures NEITHER of them — the pre-roles window, where falling through is the intended no-op; set models.<runtime>.code in .bee/config.json to close it."
        );
        let base = |lane: &str| {
            json!({"id": "a-1", "feature": "f", "title": "t", "action": "a", "verify": "v", "lane": lane, "role": "code", "affects_skills": [], "affects_specs": []})
        };
        assert_eq!(
            thrown(validate_new_cell(root, &base("mega"))),
            "addCell: invalid lane \"mega\" — must be one of: tiny, small, standard, high-risk, spike."
        );
        assert_eq!(
            thrown(validate_new_cell(root, &base("standard"))),
            "addCell: lane \"standard\" requires non-empty must_haves.truths (observable truths to verify)."
        );
        let mut with_budget = base("tiny");
        with_budget["budgets"] = json!({"max_claims": 99});
        assert_eq!(
            thrown(validate_new_cell(root, &with_budget)),
            "addCell: \"budgets.max_claims\" must be an integer in [1, 9] when present, got 99."
        );
        let mut bad_key = base("tiny");
        bad_key["budgets"] = json!({"nope": 1});
        assert_eq!(
            thrown(validate_new_cell(root, &bad_key)),
            "addCell: unknown \"budgets\" key \"nope\" — must be one of: max_claims, max_failed_attempts, max_same_signature."
        );
        // no-test sentinel refused outside a declared no-test repo
        let mut sentinel = base("tiny");
        sentinel["verify"] = json!("none");
        assert!(thrown(validate_new_cell(root, &sentinel)).starts_with("addCell: verify \"none\" is refused"));
        // duplicate id
        write_cell_fixture(root, "a-1", &cell("a-1", "open", "f", json!([])));
        assert_eq!(thrown(validate_new_cell(root, &base("tiny"))), "addCell: cell \"a-1\" already exists.");
        // normalize: literal-order appends + trace defaults
        let normalized = normalize_new_cell(&json!({"id": "n-1", "title": "t"})).unwrap();
        let keys: Vec<&String> = normalized.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["id", "title", "status", "deps", "decisions", "files", "read_first", "affects_skills", "affects_specs", "trace"]);
        assert_eq!(normalized["status"], json!("open"));
        let trace_keys: Vec<&String> = normalized["trace"].as_object().unwrap().keys().collect();
        assert_eq!(
            trace_keys,
            vec![
                "worker",
                "outcome",
                "files_changed",
                "deviations",
                "friction",
                "capped_at",
                "behavior_change",
                "verification_evidence",
                "verify_output",
                "verify_passed",
                "claim_session"
            ]
        );
    }

    // cap-1: validate_new_cell_problems collects EVERY schema problem from
    // one call, not just the first — and the batch path (build_add_cells_report)
    // reports that same list per cell rather than one Thrown message.
    #[test]
    fn validate_new_cell_problems_collects_every_problem_in_one_call() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Missing id, feature, action, verify, affects_skills, affects_specs,
        // role — title present so it does not fire — plus an invalid lane.
        //
        // RETARGETED for D7 (store `4eaf1b71`), never trimmed: `role` became
        // required, so the joined list grew one sentence and the contract
        // this test pins — every schema problem from ONE call, in check
        // order, verbatim — is asserted over the LONGER list. Deleting an
        // entry to restore the old string would weaken exactly the thing
        // the test exists for.
        let broken = json!({"title": "t", "lane": "nope"});
        let expected = vec![
            "addCell: cell is missing required field \"id\" (non-empty string).".to_string(),
            "addCell: cell is missing required field \"feature\" (non-empty string).".to_string(),
            "addCell: cell is missing required field \"action\" (non-empty string).".to_string(),
            "addCell: cell is missing required field \"verify\" (non-empty string).".to_string(),
            "addCell: cell is missing required field \"affects_skills\". FIX: every cell must declare \"affects_skills\" and \"affects_specs\" arrays (use `[]` if none).".to_string(),
            "addCell: cell is missing required field \"affects_specs\". FIX: every cell must declare \"affects_skills\" and \"affects_specs\" arrays (use `[]` if none).".to_string(),
            "addCell: invalid lane \"nope\" — must be one of: tiny, small, standard, high-risk, spike."
                .to_string(),
            "addCell: cell is missing required field \"role\" (non-empty string) — the job this work is, which is what selects the model that runs it. FIX: add \"role\": \"<name>\" to the cell, e.g. code, read, test, docs, review, design. Any non-empty name is legal — bee holds no fixed list, and a role nothing configures still runs: the dispatch falls through to the next name it asked for and warns. The one silent case is \"code\" or \"read\" on a runtime whose models.<runtime> configures NEITHER of them — the pre-roles window, where falling through is the intended no-op; set models.<runtime>.code in .bee/config.json to close it."
                .to_string(),
        ];
        assert_eq!(validate_new_cell_problems(root, &broken).unwrap(), expected);
        // validate_new_cell wraps the same list into one Thrown message.
        assert_eq!(thrown(validate_new_cell(root, &broken)), expected.join(" "));
        // The batch path reports the identical list on that cell's row —
        // nothing else (no feature is a string, so no gate check fires; no
        // id is a string, so no duplicate-id check fires).
        let (ok, rows, normalized) = build_add_cells_report(root, &[broken]).unwrap();
        assert!(!ok);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "(index 0)");
        assert!(!rows[0].ok);
        assert_eq!(rows[0].problems, expected);
        assert!(normalized.is_none());
    }

    /// A cell that is valid in every way EXCEPT its `role` — the isolating
    /// fixture for the two `role` tests below. Every other required field is
    /// filled, so any problem the collector reports is the role's.
    fn role_probe(role: Option<Value>) -> Value {
        let mut c = json!({
            "id": "role-1", "feature": "f", "title": "t", "action": "a",
            "verify": "echo ok", "lane": "tiny",
            "affects_skills": [], "affects_specs": [],
        });
        if let Some(role) = role {
            c["role"] = role;
        }
        c
    }

    // D7 (store `4eaf1b71`): `role` is required on a cell exactly as `lane`
    // is — `bee cells add` refuses without it, and the refusal names the
    // remedy rather than only the rule.
    #[test]
    fn add_cell_refuses_a_cell_with_no_role_and_names_the_remedy() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // The ONLY problem on an otherwise-complete cell is the missing role.
        let problems = validate_new_cell_problems(root, &role_probe(None)).unwrap();
        assert_eq!(problems.len(), 1, "role is the only thing wrong: {problems:?}");
        let problem = &problems[0];
        assert!(
            problem.starts_with("addCell: cell is missing required field \"role\" (non-empty string)"),
            "{problem}"
        );
        assert!(problem.contains("FIX: add \"role\": \"<name>\" to the cell"), "{problem}");
        // D8's vocabulary rides the FIX line as an example, and the line says
        // in its own words that it is not a list — an author who reads only
        // the refusal must not come away thinking these six are the legal set.
        assert!(problem.contains("code, read, test, docs, review, design"), "{problem}");
        assert!(problem.contains("Any non-empty name is legal"), "{problem}");
        // Decision 561e1bda / D2: an unconfigured role still RUNS (it falls
        // through and warns), so the FIX must not threaten a failure.
        assert!(
            problem.contains("a role nothing configures still runs"),
            "the FIX must not imply an unconfigured role breaks the dispatch: {problem}"
        );

        // The real door refuses too, and the batch path writes nothing.
        assert_eq!(thrown(validate_new_cell(root, &role_probe(None))), *problem);
        let (ok, rows, normalized) = build_add_cells_report(root, &[role_probe(None)]).unwrap();
        assert!(!ok);
        assert_eq!(rows[0].problems, vec![problem.clone()]);
        assert!(normalized.is_none(), "a refused batch lands nothing on disk");
    }

    // D2 (store `06e49368`): the role set is OPEN. Validation checks presence
    // and SHAPE; it never checks membership. This test is the guard against
    // someone "helpfully" turning `ROLE_VOCABULARY` into an enum later — a
    // closed list here would undo slice 1 outright.
    #[test]
    fn role_validation_checks_shape_and_never_membership() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let refused = |role: Value| {
            let p = validate_new_cell_problems(root, &role_probe(Some(role))).unwrap();
            assert_eq!(p.len(), 1, "expected exactly the role problem, got {p:?}");
            assert!(p[0].contains("missing required field \"role\""), "{}", p[0]);
        };

        // Shape: empty, whitespace-only, and non-string values are all
        // refused — a blank or mistyped role would resolve nothing while the
        // record asserted the cell had chosen a job.
        refused(json!(""));
        refused(json!("   "));
        refused(json!("\t\n "));
        refused(json!(null));
        refused(json!(7));
        refused(json!(["code"]));
        refused(json!({"name": "code"}));

        // Membership: every recommended name is legal, AND so is a name that
        // appears in no vocabulary and in no config anywhere. If this half
        // ever goes red, a membership check was added and D2 is broken.
        for role in ROLE_VOCABULARY {
            assert!(
                validate_new_cell_problems(root, &role_probe(Some(json!(role)))).unwrap().is_empty(),
                "recommended role {role} must be legal"
            );
        }
        for role in ["migrate", "ops", "Rewrite-The-Parser", "chữa-lỗi", "a", &"x".repeat(500)] {
            assert!(
                validate_new_cell_problems(root, &role_probe(Some(json!(role)))).unwrap().is_empty(),
                "role {role} must be legal — bee holds no fixed list"
            );
        }
        // Reserved-word probe: `ceiling` is retired as a role name entirely
        // (D5), but nothing in THIS layer knows that — validation stays
        // membership-blind, so it accepts the string like any other name.
        //
        // REVIEW P1-A weighed refusing it here and DECLINED, so this
        // assertion stands unchanged on purpose rather than by oversight.
        // Two reasons. A reserved word is a closed name inside the open set
        // D2 (store `06e49368`) spent this feature's first slice opening, and
        // this very test exists to catch exactly that regression. And it would
        // not have closed the hole: refusing the name at ADD time leaves every
        // already-stored cell carrying it, and the defect was that such a cell
        // took the session model uncounted at DISPATCH time. The fix therefore
        // landed at the dispatch, and the missing link between the two layers
        // is asserted in
        // `verbs::drivers::tests::the_ration_and_the_dispatch_agree_on_which_cells_are_escalated`:
        // a name accepted here resolves an ordinary model, and only the
        // escalation flag charges the 40% ration.
        assert!(validate_new_cell_problems(root, &role_probe(Some(json!("ceiling"))))
            .unwrap()
            .is_empty());
    }

    /// D5 (store `97ce5225`): the escalation flag is a boolean and nothing
    /// else. Presence and shape only — there is no budget check here, exactly
    /// as there was none for authoring `tier: "ceiling"`; the 40% ration
    /// lives on the `bee cells escalate` door — the same door as ever, under
    /// the name the tier retirement (D4) left it with.
    #[test]
    fn the_escalation_flag_is_a_boolean_and_never_a_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let probe = |escalate: Value| {
            let mut body = role_probe(Some(json!("code")));
            body.as_object_mut().unwrap().insert("escalate".into(), escalate);
            body
        };
        for legal in [json!(true), json!(false), Value::Null] {
            assert!(
                validate_new_cell_problems(root, &probe(legal.clone())).unwrap().is_empty(),
                "{legal} must be legal"
            );
        }
        for junk in [json!("ceiling"), json!("true"), json!(1), json!([])] {
            let problems = validate_new_cell_problems(root, &probe(junk.clone())).unwrap();
            assert_eq!(problems.len(), 1, "{junk}: {problems:?}");
            assert!(problems[0].contains("must be true or false"), "{}", problems[0]);
        }
        // Omitting it entirely is legal and is NOT an escalation: absent
        // stays absent.
        let mut bare = probe(Value::Null);
        bare.as_object_mut().unwrap().remove("escalate");
        assert!(validate_new_cell_problems(root, &bare).unwrap().is_empty());
        assert!(!cell_is_escalated(&bare));
        assert!(!cell_is_escalated(&probe(json!(false))));
        assert!(cell_is_escalated(&probe(json!(true))));
        // The legacy spelling reads as escalated, which is what keeps a
        // store that has not run the migration from reading as unmarked.
        assert!(cell_is_escalated(&json!({"id": "e-2", "tier": "ceiling"})));
        assert!(!cell_is_escalated(&json!({"id": "e-3", "tier": "generation"})));
    }

    // P3-5: a cell authored with change_class "behavior" and no explicit
    // trace.behavior_change flag must default to a real behavior change —
    // otherwise the scribing-debt door never arms.
    #[test]
    fn normalize_new_cell_defaults_behavior_change_for_behavior_class() {
        // (a) change_class "behavior", no explicit flag -> resolves true.
        let normalized =
            normalize_new_cell(&json!({"id": "n-1", "title": "t", "change_class": "behavior"}))
                .unwrap();
        assert_eq!(normalized["trace"]["behavior_change"], json!(true));
        let cell_map = normalized.as_object().unwrap();
        assert!(resolve_declared_behavior_change(cell_map));

        // (b) explicit trace.behavior_change=false is respected (deliberate
        // opt-out), even with change_class "behavior".
        let normalized = normalize_new_cell(&json!({
            "id": "n-2",
            "title": "t",
            "change_class": "behavior",
            "trace": {"behavior_change": false}
        }))
        .unwrap();
        assert_eq!(normalized["trace"]["behavior_change"], json!(false));
        let cell_map = normalized.as_object().unwrap();
        assert!(!resolve_declared_behavior_change(cell_map));

        // (c) a non-"behavior" change_class without the flag stays false.
        let normalized =
            normalize_new_cell(&json!({"id": "n-3", "title": "t", "change_class": "refactor"}))
                .unwrap();
        assert_eq!(normalized["trace"]["behavior_change"], json!(false));
        let cell_map = normalized.as_object().unwrap();
        assert!(!resolve_declared_behavior_change(cell_map));
    }

    #[test]
    fn cycle_detection_and_refusal_message() {
        let cells = vec![
            json!({"id": "a", "deps": ["b"]}),
            json!({"id": "b", "deps": ["a"]}),
            json!({"id": "c", "deps": ["c"]}),
            json!({"id": "d", "deps": ["missing"]}),
        ];
        let cycles = detect_cycles(&cells);
        assert_eq!(cycles, vec![vec!["a".to_string(), "b".to_string()], vec!["c".to_string()]]);
        assert_eq!(
            format_cycle_refusal("addCell", &cycles),
            "addCell: dependency cycle refused — a -> b; c. Cycles are illegal at every dep-mutating write (D2); file overlap stays legal and is never refused."
        );
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "x-1", &cell("x-1", "open", "f", json!(["y-1"])));
        let incoming = vec![json!({"id": "y-1", "deps": ["x-1"]})];
        assert!(assert_no_cycle(root, "addCell", &incoming).is_err());
    }

    // ── regen guards ──────────────────────────────────────────────────────
    //
    // R6 CUTOVER: this used to build a fake `scripts/release_manifest.mjs` in a
    // tempdir and assert the PARSE of it. The guards read compiled-in
    // authorities now, so the fixture is gone and the test asserts the thing
    // that actually matters: the derived scope is real, the obligation fires on
    // it, and every escape hatch still works.
    #[test]
    fn regen_guard_derives_real_roots_from_the_compiled_authorities() {
        let guards = derive_regen_guards().unwrap();
        assert_eq!(guards.len(), 2, "both guards are always active — there is no absent arm");

        let manifest = &guards[0];
        assert!(
            manifest.roots.contains(&"packages/bee".to_string())
                && manifest.roots.contains(&"skills".to_string()),
            "the manifest guard must cover the shipped frame: {:?}",
            manifest.roots
        );
        assert_eq!(
            manifest.required_files,
            vec!["docs/history/codex-harness-hardening/release-manifest.json".to_string()],
            "the manifest file itself is the required file, never a covered root"
        );
        assert!(
            !manifest.roots.contains(&manifest.required_files[0]),
            "the manifest must not be both a covered root and its own required file"
        );

        let ledger = &guards[1];
        assert!(
            ledger.roots.contains(&".bee/bin/lib".to_string())
                && ledger.roots.contains(&".bee/expertise".to_string()),
            "the ledger guard must cover the vendored trees: {:?}",
            ledger.roots
        );
    }

    #[test]
    fn regen_obligation_fires_refuses_and_can_be_acked() {
        let manifest_rel = "docs/history/codex-harness-hardening/release-manifest.json";

        // A cell touching a covered root without the check refuses…
        let cell = json!({"id": "r-1", "files": ["skills/bee-hive/SKILL.md"], "verify": "echo ok"});
        let refusal = regen_obligation_refusal(cell.as_object().unwrap(), "addCell")
            .unwrap()
            .expect("must refuse");
        assert!(
            refusal.starts_with(
                "addCell: REGEN_OBLIGATION — cell \"r-1\" touches \"skills/bee-hive/SKILL.md\""
            ),
            "{refusal}"
        );
        assert!(refusal.contains("verify does not contain \"bee dev release-manifest --check\""));
        assert!(refusal.contains(&format!("files does not list \"{manifest_rel}\"")));
        // The refusal names WHERE the scope came from, so it can be checked.
        assert!(refusal.contains("devtools::release_manifest::INVENTORY_ROOTS"), "{refusal}");

        // …the ack skips it…
        let acked = json!({
            "id": "r-1",
            "files": ["skills/bee-hive/SKILL.md"],
            "verify": "x",
            "regen_obligation_ack": "wave-barrier"
        });
        assert!(regen_obligation_refusal(acked.as_object().unwrap(), "addCell")
            .unwrap()
            .is_none());

        // …and a compliant cell passes.
        let ok = json!({
            "id": "r-1",
            "files": ["skills/bee-hive/SKILL.md", manifest_rel],
            "verify": "bee dev release-manifest --check"
        });
        assert!(regen_obligation_refusal(ok.as_object().unwrap(), "addCell").unwrap().is_none());

        // The LEDGER guard fires on its own roots, with its own fix.
        let vendored = json!({"id": "r-2", "files": [".bee/bin/lib/state.mjs"], "verify": "echo ok"});
        let refusal = regen_obligation_refusal(vendored.as_object().unwrap(), "addCell")
            .unwrap()
            .expect("the ledger guard must fire on a vendored path");
        assert!(refusal.contains("bee onboard --repo-root . --json"), "{refusal}");
        assert!(refusal.contains("onboard::plan::LEDGER_COVERED_ROOTS"), "{refusal}");

        // A cell that touches nothing covered is silent.
        let unrelated = json!({"id": "r-3", "files": ["src/main.rs"], "verify": "echo ok"});
        assert!(regen_obligation_refusal(unrelated.as_object().unwrap(), "addCell")
            .unwrap()
            .is_none());
    }

    /// jo-1 (pattern-20260812): a tiny/small cell touching guard source is
    /// refused, naming both named escapes; standard/high-risk is unaffected
    /// (the close-time judge-debt door already owns it there); the ack skips
    /// it as a recorded act; a cell touching no guard source is silent.
    #[test]
    fn judge_obligation_fires_below_the_covered_lanes_and_can_be_escaped_two_ways() {
        let guard_file = "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs";

        // A tiny cell touching guard source, with neither escape, refuses…
        let cell = json!({
            "id": "j-1",
            "lane": "tiny",
            "files": [guard_file],
            "verify": "echo ok",
        });
        let refusal = judge_obligation_refusal(cell.as_object().unwrap(), "addCell")
            .expect("must refuse");
        assert!(
            refusal.starts_with(&format!(
                "addCell: JUDGE_OBLIGATION — cell \"j-1\" touches \"{guard_file}\""
            )),
            "{refusal}"
        );
        // Names WHY (pattern-20260812) and BOTH escapes.
        assert!(refusal.contains("pattern-20260812"), "{refusal}");
        assert!(refusal.contains("raise this cell's lane to \"standard\" or \"high-risk\""), "{refusal}");
        assert!(refusal.contains(&format!("set \"{JUDGE_ACK_FIELD}\"")), "{refusal}");

        // A small cell touching guard source refuses the same way.
        let small = json!({"id": "j-2", "lane": "small", "files": [guard_file], "verify": "echo ok"});
        assert!(judge_obligation_refusal(small.as_object().unwrap(), "addCell").is_some());

        // …a standard cell touching the same file is unaffected — the
        // close-time judge-debt door already owns it there.
        let standard = json!({
            "id": "j-3",
            "lane": "standard",
            "files": [guard_file],
            "verify": "echo ok",
        });
        assert!(judge_obligation_refusal(standard.as_object().unwrap(), "addCell").is_none());

        // …a high-risk cell too.
        let high_risk = json!({
            "id": "j-4",
            "lane": "high-risk",
            "files": [guard_file],
            "verify": "echo ok",
        });
        assert!(judge_obligation_refusal(high_risk.as_object().unwrap(), "addCell").is_none());

        // …the ack skips it, recorded on the cell as a named act…
        let acked = json!({
            "id": "j-1",
            "lane": "tiny",
            "files": [guard_file],
            "verify": "echo ok",
            "judge_obligation_ack": "authored red-first against a live-store sample, see decision X",
        });
        assert!(judge_obligation_refusal(acked.as_object().unwrap(), "addCell").is_none());

        // …and a tiny cell touching nothing under a judge-required root is
        // silent.
        let unrelated = json!({
            "id": "j-5",
            "lane": "tiny",
            "files": ["packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"],
            "verify": "echo ok",
        });
        assert!(judge_obligation_refusal(unrelated.as_object().unwrap(), "addCell").is_none());

        // assert_judge_obligation surfaces the same refusal as a thrown error.
        assert!(assert_judge_obligation(cell.as_object().unwrap(), "addCell").is_err());
        assert!(assert_judge_obligation(standard.as_object().unwrap(), "addCell").is_ok());
    }

    // ── claims-store protocol ─────────────────────────────────────────────
    #[test]
    fn claim_cell_file_protocol_and_release() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        // Sessionless claim: session key omitted, fence_epoch 1, floor(ttl).
        let outcome = claim_cell_file(control, None, "c-1", Some(120.9)).unwrap();
        let claim = match outcome {
            ClaimFileOutcome::Ok { claim } => claim,
            _ => panic!("first claim must win"),
        };
        let keys: Vec<&String> = claim.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["cell", "ttl_seconds", "claimed_at", "acquired_at", "fence_epoch"]);
        assert_eq!(claim["ttl_seconds"], json!(120.0));
        assert_eq!(claim["fence_epoch"], json!(1.0));
        // Second claim loses with the typed CLAIMED reason.
        match claim_cell_file(control, Some("s2"), "c-1", None).unwrap() {
            ClaimFileOutcome::Refused { code, reason } => {
                assert_eq!(code, "CLAIMED");
                assert!(reason.starts_with(
                    "cell \"c-1\" is already claimed by session \"no session (sessionless claim)\""
                ));
                assert!(reason.contains("expires "));
            }
            _ => panic!("second claim must refuse"),
        }
        // Owner-matched release removes the file; a mismatched owner leaves it.
        release_claim(control, Some("someone-else"), "c-1").unwrap();
        assert!(claims_dir(control).join("c-1.json").exists());
        release_claim(control, None, "c-1").unwrap();
        assert!(!claims_dir(control).join("c-1.json").exists());
        // Sessioned claim carries the session key before ttl_seconds.
        let claim = match claim_cell_file(control, Some("sess-9"), "c-2", None).unwrap() {
            ClaimFileOutcome::Ok { claim } => claim,
            _ => panic!(),
        };
        let keys: Vec<&String> = claim.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["cell", "session", "ttl_seconds", "claimed_at", "acquired_at", "fence_epoch"]);
        assert_eq!(claim["ttl_seconds"], json!(3600.0));
        // Bad ids throw claims.mjs requireId's exact messages.
        assert_eq!(
            thrown(claim_cell_file(control, Some("a/b"), "c-3", None).map(|_| ())),
            "session id must be a plain id (no path separators)."
        );
        assert_eq!(thrown(claim_path(control, "  ").map(|_| ())), "cell id is required.");
    }

    #[test]
    fn ownership_guard_refuses_foreign_live_claim_and_audits_force() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(claims_dir(root)).unwrap();
        // A live claim owned by another session.
        match claim_cell_file(root, Some("owner-1"), "g-1", None).unwrap() {
            ClaimFileOutcome::Ok { .. } => {}
            _ => panic!(),
        }
        let refusal = thrown(guard_claim_ownership(
            root,
            "g-1",
            default_trace(),
            "blockCell",
            Some("intruder-2"),
            false,
        ));
        assert!(refusal.starts_with("blockCell: cell \"g-1\" is claimed by session \"owner-1\""));
        assert!(refusal.ends_with("Pass --force-ownership to override (audited)."));
        // Force appends the audit row instead.
        let audited = guard_claim_ownership(
            root,
            "g-1",
            default_trace(),
            "blockCell",
            Some("intruder-2"),
            true,
        )
        .unwrap();
        let overrides = audited.get("ownership_overrides").unwrap().as_array().unwrap();
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0]["verb"], json!("blockCell"));
        assert_eq!(overrides[0]["forced_by"], json!("intruder-2"));
        assert_eq!(overrides[0]["owner_bypassed"], json!("owner-1"));
        // The owner itself passes untouched.
        let own = guard_claim_ownership(root, "g-1", default_trace(), "capCell", Some("owner-1"), false);
        assert!(own.is_ok());
    }

    // ── D4: route-record warn-to-deny escalation ────────────────────────────
    // A lane record with `approved_gates.execution: true` and no "route" key
    // both authorizes the claim (lane_record_gates) AND reads as "no route"
    // (read_lane_route: the object matches the feature, `route` is simply
    // absent — Some(false), not None), the same fixture shape already used
    // by `resolve_pipeline`'s "good" lane above. Adding a `"route": true` key
    // flips it to routed.
    fn lane_no_route(root: &Path, feature: &str) {
        let dir = lanes_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{feature}.json")),
            format!(r#"{{"feature":"{feature}","approved_gates":{{"execution":true}}}}"#),
        )
        .unwrap();
    }

    fn lane_with_route(root: &Path, feature: &str) {
        let dir = lanes_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{feature}.json")),
            format!(r#"{{"feature":"{feature}","approved_gates":{{"execution":true}},"route":true}}"#),
        )
        .unwrap();
    }

    /// Proves the counter's OWN definition before anything flips on it (D6):
    /// per (feature, session), monotonic, and neither a different feature
    /// NOR a different session sharing the same feature ever shares a file.
    #[test]
    fn no_route_claim_count_is_per_feature_and_session_and_monotonic() {
        let tmp = cn_root();
        let root = tmp.path();
        assert_eq!(no_route_claim_count(root, "f1", Some("s1")).unwrap(), 0);
        assert_eq!(bump_no_route_claim_count(root, "f1", Some("s1")).unwrap(), 1);
        assert_eq!(no_route_claim_count(root, "f1", Some("s1")).unwrap(), 1);
        // A second feature starts at 0 and stays there — f1's bumps never
        // touch f2's counter file.
        assert_eq!(no_route_claim_count(root, "f2", Some("s1")).unwrap(), 0);
        // A second SESSION of the SAME feature is independent too — bee's
        // own swarming model fans one feature's cells out to many
        // concurrently-dispatched sessions, and each gets its own one-time
        // warning rather than inheriting another session's spent count.
        assert_eq!(no_route_claim_count(root, "f1", Some("s2")).unwrap(), 0);
        // A sessionless caller is its own fixed "none" bucket.
        assert_eq!(no_route_claim_count(root, "f1", None).unwrap(), 0);
        assert_eq!(bump_no_route_claim_count(root, "f1", None).unwrap(), 1);
        assert_eq!(no_route_claim_count(root, "f1", None).unwrap(), 1);

        assert_eq!(bump_no_route_claim_count(root, "f1", Some("s1")).unwrap(), 2);
        assert_eq!(no_route_claim_count(root, "f1", Some("s1")).unwrap(), 2);
        assert_eq!(no_route_claim_count(root, "f2", Some("s1")).unwrap(), 0, "a different feature's claims never count");
        assert_eq!(no_route_claim_count(root, "f1", Some("s2")).unwrap(), 0, "a different session's claims never count");
        assert_eq!(no_route_claim_count(root, "f1", None).unwrap(), 1, "the sessionless bucket is untouched by s1's bumps");
    }

    /// D4: the first `cells claim` a SESSION makes against a routeless
    /// feature still only warns (never denies); that SAME session's second
    /// claim — even of a DIFFERENT cell of the same feature — refuses,
    /// naming the route remedy, and never mutates the cell or creates a
    /// claim file. Once the feature is routed, claiming goes through again
    /// with no refusal at all.
    #[test]
    fn claim_warns_once_per_session_then_refuses_until_routed() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_no_route(root, "nr");
        write_cell_fixture(root, "nr-1", &cell("nr-1", "open", "nr", json!([])));
        write_cell_fixture(root, "nr-2", &cell("nr-2", "open", "nr", json!([])));

        // First claim for the feature: succeeds and spends sess-1's
        // one-time warning allowance.
        let door = claim_cell_from_flags(root, "nr-1", "w1", Some("sess-1"), None).unwrap();
        assert_eq!(door.cell["status"], json!("claimed"));
        assert_eq!(no_route_claim_count(root, "nr", Some("sess-1")).unwrap(), 1);

        // Second claim by the SAME session — a different cell, feature
        // still routeless — refuses.
        let refusal = thrown(claim_cell_from_flags(root, "nr-2", "w1", Some("sess-1"), None));
        assert!(
            refusal.starts_with(
                "claim: NO_ROUTE_RECORD — cell \"nr-2\" refused — feature \"nr\" still has no route record"
            ),
            "{refusal}"
        );
        assert!(refusal.contains("D4"), "{refusal}");
        assert!(
            refusal.contains("bee state route --set --class <c> --lane <l> --flags <f> --files <n>"),
            "{refusal}"
        );
        // Refused: no claim file, no cell mutation.
        assert!(!claims_dir(root).join("nr-2.json").exists());
        let untouched = read_cell_norm(root, "nr-2").ok().unwrap().unwrap();
        assert_eq!(untouched["status"], json!("open"));
        // The refusal itself never advances the count further.
        assert_eq!(no_route_claim_count(root, "nr", Some("sess-1")).unwrap(), 1);

        // A DIFFERENT session's first claim of the same routeless feature
        // still only warns — swarming's fan-out (many worker sessions,
        // one feature) must never be blocked by another session's spent
        // warning.
        let door_other = claim_cell_from_flags(root, "nr-2", "w2", Some("sess-2"), None).unwrap();
        assert_eq!(door_other.cell["status"], json!("claimed"));
        assert_eq!(no_route_claim_count(root, "nr", Some("sess-2")).unwrap(), 1);

        // Route the feature: sess-1 now claims cleanly again too.
        lane_with_route(root, "nr");
        write_cell_fixture(root, "nr-3", &cell("nr-3", "open", "nr", json!([])));
        let door2 = claim_cell_from_flags(root, "nr-3", "w1", Some("sess-1"), None).unwrap();
        assert_eq!(door2.cell["status"], json!("claimed"));
    }

    /// D4's count must survive unclaim/reclaim of the SAME cell BY THE SAME
    /// SESSION — neither the cell's own trace.claimed_at (nulled by
    /// release_trace on unclaim) nor its claim file's fence_epoch (a fresh 1
    /// on every O_EXCL reclaim) can carry it, so a second claim of the
    /// identical cell by the identical session must still refuse once that
    /// session's one-time warning is already spent.
    #[test]
    fn claim_refusal_survives_unclaim_and_reclaim_of_the_same_cell() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_no_route(root, "sr");
        write_cell_fixture(root, "sr-1", &cell("sr-1", "open", "sr", json!([])));

        claim_cell_from_flags(root, "sr-1", "w1", Some("sess-1"), None).unwrap();
        assert_eq!(no_route_claim_count(root, "sr", Some("sess-1")).unwrap(), 1);

        // Unclaim resets the CELL's own status/trace — but not the
        // (feature, session) persisted claim count.
        unclaim_cell(root, "sr-1", Some("sess-1"), false).unwrap();
        let reopened = read_cell_norm(root, "sr-1").ok().unwrap().unwrap();
        assert_eq!(reopened["status"], json!("open"));
        assert_eq!(reopened["trace"]["claimed_at"], Value::Null);
        assert_eq!(
            no_route_claim_count(root, "sr", Some("sess-1")).unwrap(),
            1,
            "unclaim must not reset the (feature, session) count"
        );

        // Reclaiming the SAME cell by the SAME session is still that
        // session's second no-route claim — refused.
        let refusal = thrown(claim_cell_from_flags(root, "sr-1", "w1", Some("sess-1"), None));
        assert!(
            refusal.starts_with(
                "claim: NO_ROUTE_RECORD — cell \"sr-1\" refused — feature \"sr\" still has no route record"
            ),
            "{refusal}"
        );
        let still_open = read_cell_norm(root, "sr-1").ok().unwrap().unwrap();
        assert_eq!(still_open["status"], json!("open"), "a refused reclaim must not re-claim the cell");
    }

    /// Concurrency-contract ordering (real-process claim race, tests/
    /// concurrency.rs): a racing LOSER on an already-claimed cell must see
    /// the typed CLAIMED refusal, never the no-route deny — even when the
    /// racing session's own one-time no-route warning is already spent
    /// (from claiming a DIFFERENT cell earlier). A loser never had the cell
    /// to "keep claiming without a route" in the first place; D4 only fires
    /// once this caller would otherwise walk away with it.
    #[test]
    fn already_claimed_refusal_outranks_the_no_route_deny() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_no_route(root, "race");
        write_cell_fixture(root, "race-1", &cell("race-1", "open", "race", json!([])));
        write_cell_fixture(root, "race-2", &cell("race-2", "open", "race", json!([])));

        // sess-1 claims race-1 cleanly, spending ITS one-time warning.
        claim_cell_from_flags(root, "race-1", "w1", Some("sess-1"), None).unwrap();
        assert_eq!(no_route_claim_count(root, "race", Some("sess-1")).unwrap(), 1);

        // sess-1 races itself against the SAME already-claimed cell (e.g. a
        // retried dispatch): CLAIMED outranks NO_ROUTE_RECORD even though
        // sess-1's count is already spent.
        let refusal = thrown(claim_cell_from_flags(root, "race-1", "w1", Some("sess-1"), None));
        assert!(
            refusal.starts_with("claim: CLAIMED — cell \"race-1\" is already claimed by session \"sess-1\""),
            "{refusal}"
        );
        assert!(!refusal.contains("NO_ROUTE_RECORD"), "{refusal}");
        // The already-claimed refusal never advances the no-route count.
        assert_eq!(no_route_claim_count(root, "race", Some("sess-1")).unwrap(), 1);

        // A DIFFERENT session (its own count still 0) also loses the race
        // for the same already-claimed cell — still CLAIMED, not NO_ROUTE.
        let refusal2 = thrown(claim_cell_from_flags(root, "race-1", "w2", Some("sess-2"), None));
        assert!(
            refusal2.starts_with("claim: CLAIMED — cell \"race-1\" is already claimed by session \"sess-1\""),
            "{refusal2}"
        );
        assert_eq!(no_route_claim_count(root, "race", Some("sess-2")).unwrap(), 0);
    }

    // ── D2: no claim on a red base ──────────────────────────────────────────

    fn write_test_results_fixture(root: &Path, green: bool, commands: &[(&str, bool)]) {
        let rows: Vec<Value> = commands
            .iter()
            .map(|(cmd, passed)| {
                json!({
                    "command": cmd,
                    "exit": if *passed { 0.0 } else { 1.0 },
                    "duration_ms": 1.0,
                    "failure_excerpt": if *passed { Value::Null } else { Value::String("boom".into()) },
                })
            })
            .collect();
        let record = json!({ "ran_at": "2026-01-01T00:00:00.000Z", "green": green, "commands": rows });
        write_json_atomic(&test_results_path(root), &record).unwrap();
    }

    /// D7 red-first: proves the pure classifier's four outcomes against the
    /// EXACT schema finish_support::run_declared_tests writes, before any
    /// refusal wires onto it — green, red (naming the first failing
    /// command), missing, and unparseable (valid JSON, wrong shape) all read
    /// as distinct, and both "cannot know" shapes land on the same Unknown
    /// arm a claim door treats as a warn-and-proceed.
    #[test]
    fn classify_red_base_reads_green_red_missing_and_unparseable() {
        let tmp = cn_root();
        let root = tmp.path();

        // Missing: nothing has ever run the declared tests here.
        assert!(matches!(classify_red_base(root), RedBaseStatus::Unknown));

        // Green: untouched either way.
        write_test_results_fixture(root, true, &[("cargo test", true)]);
        assert!(matches!(classify_red_base(root), RedBaseStatus::Green));

        // Red: names the FIRST failing command, not the last.
        write_test_results_fixture(root, false, &[("cargo build", true), ("cargo test", false)]);
        match classify_red_base(root) {
            RedBaseStatus::Red { failing_command } => assert_eq!(failing_command, "cargo test"),
            _ => panic!("expected Red"),
        }

        // Unparseable: valid JSON, wrong shape (green not a bool) — cannot
        // know, same bucket as missing.
        std::fs::write(test_results_path(root), r#"{"green":"not-a-bool"}"#).unwrap();
        assert!(matches!(classify_red_base(root), RedBaseStatus::Unknown));

        // Corrupt: not valid JSON at all — cannot know.
        std::fs::write(test_results_path(root), "{not json").unwrap();
        assert!(matches!(classify_red_base(root), RedBaseStatus::Unknown));
    }

    /// The claim door itself: a red base refuses naming the failing command
    /// and the results path, unless `--fix-first <reason>` escapes it — and
    /// that reason lands on the WINNING claim's own trace.
    #[test]
    fn claim_refuses_on_a_red_base_unless_fix_first_escapes_it() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_with_route(root, "rb");
        write_cell_fixture(root, "rb-1", &cell("rb-1", "open", "rb", json!([])));
        write_cell_fixture(root, "rb-2", &cell("rb-2", "open", "rb", json!([])));
        write_test_results_fixture(root, false, &[("cargo test --release", false)]);

        // Red, no --fix-first: refused, pinned prefix, no claim file, no
        // cell mutation.
        let refusal = thrown(claim_cell_from_flags(root, "rb-1", "w1", Some("sess-1"), None));
        assert!(
            refusal.starts_with("claim: RED_BASE — cell \"rb-1\" refused — the last recorded test run is red"),
            "{refusal}"
        );
        assert!(refusal.contains("cargo test --release"), "{refusal}");
        assert!(refusal.contains(TEST_RESULTS_RELATIVE), "{refusal}");
        assert!(refusal.contains("--fix-first"), "{refusal}");
        // D7: `bee test` is the only writer of this record left (close and
        // worktree merge stopped running commands.test) — the remedy must
        // name it as the refresh path.
        assert!(refusal.contains("bee test"), "{refusal}");
        assert!(!claims_dir(root).join("rb-1.json").exists());
        let untouched = read_cell_norm(root, "rb-1").ok().unwrap().unwrap();
        assert_eq!(untouched["status"], json!("open"));

        // Red + --fix-first: claim succeeds, the reason lands on trace.fix_first.
        let door = claim_cell_from_flags_ex(
            root,
            "rb-2",
            "w1",
            Some("sess-1"),
            None,
            Some("known flake, fixing next"),
        )
        .unwrap();
        assert_eq!(door.cell["status"], json!("claimed"));
        assert_eq!(door.cell["trace"]["fix_first"], json!("known flake, fixing next"));
    }

    /// A green base is untouched: no refusal, and no stray trace.fix_first
    /// key when the escape was never spent.
    #[test]
    fn claim_on_a_green_base_is_untouched() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_with_route(root, "gb");
        write_cell_fixture(root, "gb-1", &cell("gb-1", "open", "gb", json!([])));
        write_test_results_fixture(root, true, &[("cargo test", true)]);

        let door = claim_cell_from_flags(root, "gb-1", "w1", Some("sess-1"), None).unwrap();
        assert_eq!(door.cell["status"], json!("claimed"));
        assert!(door.cell["trace"].get("fix_first").is_none());
    }

    /// A missing results file cannot prove red or green — the claim
    /// proceeds (the stderr warning is proven separately by the classifier
    /// test above; this test proves the DOOR takes the same "cannot know"
    /// arm rather than refusing).
    #[test]
    fn claim_on_missing_results_proceeds() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_with_route(root, "mb");
        write_cell_fixture(root, "mb-1", &cell("mb-1", "open", "mb", json!([])));
        assert!(!test_results_path(root).exists());

        let door = claim_cell_from_flags(root, "mb-1", "w1", Some("sess-1"), None).unwrap();
        assert_eq!(door.cell["status"], json!("claimed"));
    }

    /// Ordering (pinned): a racing loser on an already-claimed cell sees
    /// CLAIMED, never RED_BASE — even when the base is red by the time the
    /// second claimant arrives.
    #[test]
    fn already_claimed_refusal_outranks_the_red_base_deny() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_with_route(root, "rb-race");
        write_cell_fixture(root, "rb-race-1", &cell("rb-race-1", "open", "rb-race", json!([])));

        claim_cell_from_flags(root, "rb-race-1", "w1", Some("sess-1"), None).unwrap();
        write_test_results_fixture(root, false, &[("cargo test", false)]);

        let refusal = thrown(claim_cell_from_flags(root, "rb-race-1", "w2", Some("sess-2"), None));
        assert!(refusal.starts_with("claim: CLAIMED"), "{refusal}");
        assert!(!refusal.contains("RED_BASE"), "{refusal}");
    }

    /// Ordering (pinned): D4's no-route deny outranks D2's red-base deny —
    /// a session that has already spent its one-time no-route warning sees
    /// NO_ROUTE_RECORD, never RED_BASE, even though the base is also red.
    #[test]
    fn no_route_deny_outranks_the_red_base_deny() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_no_route(root, "rb-nr");
        write_cell_fixture(root, "rb-nr-1", &cell("rb-nr-1", "open", "rb-nr", json!([])));
        write_cell_fixture(root, "rb-nr-2", &cell("rb-nr-2", "open", "rb-nr", json!([])));

        // First claim (no test-results record yet): spends sess-1's
        // one-time no-route warning.
        claim_cell_from_flags(root, "rb-nr-1", "w1", Some("sess-1"), None).unwrap();
        // Now the base goes red.
        write_test_results_fixture(root, false, &[("cargo test", false)]);

        let refusal = thrown(claim_cell_from_flags(root, "rb-nr-2", "w1", Some("sess-1"), None));
        assert!(refusal.starts_with("claim: NO_ROUTE_RECORD"), "{refusal}");
        assert!(!refusal.contains("RED_BASE"), "{refusal}");
    }

    // ══ crf-1 — the claim ACQUIRES what the cap already releases ══════════
    //
    // docs/history/claim-reserves-files/CONTEXT.md: `finish_cap_and_release`
    // has always released by `(trace.worker, cell.id)` against reservations no
    // claim door ever took, so every dispatched worker wrote unreserved. These
    // pin the acquire half — one per locked constraint — through
    // `claim_cell_with_reservations`, the exact composition `run_claim` runs
    // (`run_claim` itself resolves its root off `std::env::current_dir()`, so
    // the door under it is what a test can address).

    /// Every ACTIVE lease in the store, as `(path, agent, cell)` — the shape
    /// these assertions actually read.
    fn held_paths(root: &Path) -> Vec<(String, String, String)> {
        let list = match rsv::list_reservations(root.to_str().unwrap(), true, rsv::now_ms()) {
            Ok(v) => v,
            Err(_) => panic!("list_reservations hit an unproven shape"),
        };
        let mut out: Vec<(String, String, String)> = list
            .iter()
            .map(|r| {
                (
                    r.path.clone(),
                    r.agent.as_ref().map_or("?".into(), jsjson::js_to_string),
                    r.cell.as_ref().map_or("?".into(), jsjson::js_to_string),
                )
            })
            .collect();
        out.sort();
        out
    }

    fn cell_with_files(id: &str, feature: &str, files: Value) -> Value {
        let mut c = cell(id, "open", feature, json!([]));
        c["files"] = files;
        c
    }

    fn main_topo(root: &Path) -> Option<(&Path, &str)> {
        Some((root, "main"))
    }

    /// Reserve one path for someone else, through the same shared door the
    /// claim now uses.
    fn foreign_lease(root: &Path, agent: &str, cell_id: &str, path: &str) {
        let params = rsv::ReserveParams {
            agent: agent.to_string(),
            cell: cell_id.to_string(),
            path: path.to_string(),
            ttl: None,
            session: Some("sess-other".to_string()),
            kind: None,
        };
        let topo = Some(rsv::Topo { main_root: root, holder: "main" });
        assert!(
            matches!(rsv::reserve_exec(topo, root.to_str().unwrap(), &params, 1), Ok(Out::Emit(_, _, 0))),
            "the fixture lease for {path} must be taken"
        );
    }

    /// The acceptance criterion, both halves in one test: claiming a cell with
    /// declared files creates one lease per path under the `--worker` identity
    /// and the claimed cell id — the SAME `(agent, cell)` key
    /// `finish_cap_and_release` releases by — and capping that cell then
    /// releases exactly those, with no new code in the release path.
    #[test]
    fn claim_reserves_the_declared_files_and_the_cap_releases_exactly_those() {
        let tmp = cn_root();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "echo ok"}}));
        lane_with_route(root, "crf");
        write_cell_fixture(
            root,
            "crf-a",
            &cell_with_files("crf-a", "crf", json!(["src/one.rs", "src/two.rs"])),
        );

        let door =
            claim_cell_with_reservations(root, main_topo(root), "crf-a", "wk-1", Some("sess-1"), None, None)
                .unwrap();
        assert_eq!(door.cell["status"], json!("claimed"));
        assert_eq!(door.cell["trace"]["worker"], json!("wk-1"));
        assert_eq!(
            held_paths(root),
            vec![
                ("src/one.rs".to_string(), "wk-1".to_string(), "crf-a".to_string()),
                ("src/two.rs".to_string(), "wk-1".to_string(), "crf-a".to_string()),
            ]
        );

        // The pair closes: the release half, untouched by this cell, names
        // exactly the paths the claim half took.
        let mut cap = wf_cap_flags("crf-a");
        cap.session_flag = Some("sess-1".to_string()); // the claim's own session caps it
        let out = finish_cap_and_release(root, main_topo(root), cap, None).expect("a clean finish");
        let Out::Emit(result, _, 0) = out else { panic!("expected a green finish") };
        // The release half walks the lease store, so its order is the store's,
        // not the declaration's — the SET is the contract.
        let mut released: Vec<String> = result["released"]
            .as_array()
            .unwrap()
            .iter()
            .map(jsjson::js_to_string)
            .collect();
        released.sort();
        assert_eq!(released, vec!["src/one.rs".to_string(), "src/two.rs".to_string()]);
        assert!(held_paths(root).is_empty(), "the cap releases every lease the claim took");
    }

    /// Constraint 1: a claim with no overlapping hold succeeds exactly as it
    /// did before this cell — the acquire adds NO key to the emitted cell
    /// record, so `run_claim`'s payload and text are byte-identical.
    #[test]
    fn a_free_claim_emits_the_same_cell_record_it_always_did() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_with_route(root, "crfb");
        let files = json!(["src/new.rs"]);
        write_cell_fixture(root, "crfb-new", &cell_with_files("crfb-new", "crfb", files.clone()));
        write_cell_fixture(root, "crfb-old", &cell_with_files("crfb-old", "crfb", json!(["src/old.rs"])));

        let with_reserve =
            claim_cell_with_reservations(root, main_topo(root), "crfb-new", "wk-1", Some("sess-1"), None, None)
                .unwrap()
                .cell;
        // The pre-crf-1 door, over the same fixture shape.
        let without = claim_cell_from_flags(root, "crfb-old", "wk-1", Some("sess-1"), None)
            .unwrap()
            .cell;

        let keys = |v: &Value| {
            let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
            k.sort();
            k
        };
        assert_eq!(keys(&with_reserve), keys(&without), "no new key rides the claim payload");
        assert_eq!(keys(&with_reserve["trace"]), keys(&without["trace"]));
        assert_eq!(with_reserve["status"], without["status"]);
        assert_eq!(with_reserve["files"], files);
    }

    /// Constraint 3: a cell declaring `files: []` — or no `files` key at all —
    /// claims exactly as today, with no new refusal and nothing reserved.
    #[test]
    fn a_cell_with_no_declared_files_claims_exactly_as_before() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_with_route(root, "crfe");
        write_cell_fixture(root, "crfe-empty", &cell_with_files("crfe-empty", "crfe", json!([])));
        // No `files` key whatsoever — `cell()`'s own shape.
        write_cell_fixture(root, "crfe-none", &cell("crfe-none", "open", "crfe", json!([])));

        for id in ["crfe-empty", "crfe-none"] {
            let door =
                claim_cell_with_reservations(root, main_topo(root), id, "wk-1", Some("sess-1"), None, None)
                    .unwrap_or_else(|_| panic!("{id} must claim with no new refusal"));
            assert_eq!(door.cell["status"], json!("claimed"));
        }
        assert!(held_paths(root).is_empty(), "zero declared paths reserves nothing");
    }

    /// Constraint 2: a conflicting claim is refused TYPED and ZERO-MUTATION.
    /// The conflict sits on the SECOND declared path on purpose, so the first
    /// one is genuinely reserved before the refusal — the rollback has real
    /// work to undo, and the store still comes back exactly as found.
    #[test]
    fn a_conflicting_claim_is_refused_typed_and_rolls_the_store_back() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_with_route(root, "crfc");
        write_cell_fixture(
            root,
            "crfc-1",
            &cell_with_files("crfc-1", "crfc", json!(["src/free.rs", "src/taken.rs"])),
        );
        foreign_lease(root, "other-agent", "other-1", "src/taken.rs");
        let before = held_paths(root);

        let refusal = thrown(claim_cell_with_reservations(
            root,
            main_topo(root),
            "crfc-1",
            "wk-1",
            Some("sess-1"),
            None,
            None,
        ));
        assert!(refusal.starts_with("claim: RESERVATION_CONFLICT — "), "{refusal}");
        assert!(refusal.contains("nothing claimed"), "{refusal}");
        assert!(
            refusal.contains("the claim was rolled back and the store restored as found"),
            "{refusal}"
        );
        assert!(!refusal.contains("ROLLBACK FAILED"), "{refusal}");
        // The refusal NAMES the holder.
        assert!(refusal.contains("- other-agent holds \"src/taken.rs\" (cell other-1)"), "{refusal}");

        // Zero mutation: the cell is open again, no claim file was left owned,
        // the partial lease is gone and the other agent's is untouched.
        assert_eq!(read_cell_fixture(root, "crfc-1")["status"], json!("open"));
        assert!(!claims_dir(root).join("crfc-1.json").exists());
        assert_eq!(held_paths(root), before);
    }

    /// Constraint 4: re-claiming a cell whose lease this same worker already
    /// holds is NOT a conflict — that is the claim-expired/swept-then-retaken
    /// path, and the lease is already the right one. A same-agent lease for a
    /// DIFFERENT cell stays a real conflict: that cell's own cap releases it.
    #[test]
    fn a_workers_own_lease_for_the_same_cell_never_refuses_the_re_claim() {
        let tmp = cn_root();
        let root = tmp.path();
        lane_with_route(root, "crfo");
        write_cell_fixture(root, "crfo-1", &cell_with_files("crfo-1", "crfo", json!(["src/mine.rs"])));
        write_cell_fixture(root, "crfo-2", &cell_with_files("crfo-2", "crfo", json!(["src/other.rs"])));

        // Ours already, for this very cell — the claim file expired or was
        // swept, the lease outlived it.
        foreign_lease(root, "wk-1", "crfo-1", "src/mine.rs");
        let door =
            claim_cell_with_reservations(root, main_topo(root), "crfo-1", "wk-1", Some("sess-1"), None, None)
                .expect("our own lease for this cell must not refuse the claim");
        assert_eq!(door.cell["status"], json!("claimed"));
        assert!(
            held_paths(root).contains(&(
                "src/mine.rs".to_string(),
                "wk-1".to_string(),
                "crfo-1".to_string()
            )),
            "the existing lease stands, unduplicated: {:?}",
            held_paths(root)
        );

        // Same agent, DIFFERENT cell: still a conflict.
        foreign_lease(root, "wk-1", "crfo-9", "src/other.rs");
        let refusal = thrown(claim_cell_with_reservations(
            root,
            main_topo(root),
            "crfo-2",
            "wk-1",
            Some("sess-1"),
            None,
            None,
        ));
        assert!(refusal.starts_with("claim: RESERVATION_CONFLICT — "), "{refusal}");
        assert!(refusal.contains("- wk-1 holds \"src/other.rs\" (cell crfo-9)"), "{refusal}");
        assert_eq!(read_cell_fixture(root, "crfo-2")["status"], json!("open"));
    }

    // ── budgets ───────────────────────────────────────────────────────────
    fn attempt(session: &str, acquired: &str, verdict: &str, sig: Option<&str>) -> Value {
        json!({
            "n": 1, "at": format!("{acquired}x"), "claim_session": session,
            "claimed_at": acquired, "acquired_at": acquired, "worker": "w",
            "verdict": verdict, "failure_signature": sig, "note": null
        })
    }

    #[test]
    fn budget_checks_close_and_reopen_the_claim_door() {
        let mut cell = json!({"id": "b-1", "trace": {"attempts": [
            attempt("s1", "2026-01-01T00:00:00.000Z", "blocked", None),
            attempt("s2", "2026-01-02T00:00:00.000Z", "tests-red", None),
            attempt("s3", "2026-01-03T00:00:00.000Z", "fail", None),
        ]}});
        // 3 distinct acquisition pairs + the attempt being made = 4 > 3.
        match check_cell_budgets(cell.as_object().unwrap()).unwrap() {
            BudgetCheck::Refused { code, reason } => {
                assert_eq!(code, "CELL_BUDGET_EXHAUSTED");
                assert_eq!(
                    reason,
                    "cell \"b-1\" exhausted its \"max_claims\" budget (limit 3, used 4) — the claim door is closed until an audited reset."
                );
            }
            _ => panic!("must refuse"),
        }
        // A budget_resets marker restarts the counters (lexical ISO compare).
        cell["trace"]["budget_resets"] = json!([{"reset_at": "2026-01-04T00:00:00.000Z"}]);
        assert!(matches!(check_cell_budgets(cell.as_object().unwrap()).unwrap(), BudgetCheck::Ok));
        // Same-signature repeats refuse independently.
        let cell = json!({"id": "b-2", "trace": {"attempts": [
            attempt("s1", "2026-01-01T00:00:00.000Z", "fail", Some("deadbeef0000")),
            attempt("s1", "2026-01-01T00:00:00.000Z", "fail", Some("deadbeef0000")),
        ]}});
        match check_cell_budgets(cell.as_object().unwrap()).unwrap() {
            BudgetCheck::Refused { code, reason } => {
                assert_eq!(code, "REPEATED_FAILURE");
                assert!(reason.contains("failed 2 time(s) with the identical signature \"deadbeef0000\""));
            }
            _ => panic!("must refuse"),
        }
        // Declared budgets are clamped to the hard max; junk falls back.
        let cell = json!({"id": "b-3", "budgets": {"max_claims": 99, "max_failed_attempts": 0.5}});
        let budgets = resolve_cell_budgets(cell.as_object().unwrap());
        assert_eq!(budgets.max_claims, 9.0);
        assert_eq!(budgets.max_failed_attempts, 4.0);
    }

    // ── frozen judge + glob covers ────────────────────────────────────────
    #[test]
    fn frozen_judge_rules_and_declared_covers() {
        assert_eq!(frozen_judge_rule("tests/a.mjs"), Some("test sources"));
        assert_eq!(frozen_judge_rule("src/__tests__/a.mjs"), Some("test sources"));
        assert_eq!(frozen_judge_rule("src/a.test.js"), Some("test file"));
        assert_eq!(frozen_judge_rule("x/__snapshots__/a.snap"), Some("snapshot"));
        assert_eq!(frozen_judge_rule(".github/workflows/ci.yml"), Some("CI config"));
        assert_eq!(frozen_judge_rule("package-lock.json"), Some("lockfile"));
        assert_eq!(frozen_judge_rule("sub/Cargo.toml"), Some("package manifest"));
        assert_eq!(frozen_judge_rule("jest.config.mjs"), Some("test config"));
        assert_eq!(frozen_judge_rule(".bee/config.json"), Some("bee verify config"));
        assert_eq!(frozen_judge_rule("src/lib.rs"), None);
        assert_eq!(frozen_judge_rule("attestation/a.js"), None);

        let declared = vec![json!("tests/"), json!("src/*.test.js"), json!("docs/**/x.md")];
        assert!(declared_covers(&declared, "tests/anything.mjs"));
        assert!(declared_covers(&declared, "src/a.test.js"));
        assert!(!declared_covers(&declared, "src/deep/a.test.js")); // '*' never crosses '/'
        assert!(declared_covers(&declared, "docs/a/b/x.md")); // '**' does
        let hits = frozen_judge_hits(&json!(["tests/a.mjs", "src/x.js", "yarn.lock"]), &json!(["tests/"]));
        assert_eq!(hits, vec![("yarn.lock".to_string(), "lockfile")]);
    }

    // ── judge verdict schema ──────────────────────────────────────────────
    #[test]
    fn judge_verdict_validation_matches_node_errors() {
        let (ok, errors) = validate_judge_verdict(&json!("free prose"));
        assert!(!ok);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("verdict must be a JSON object per schema \"judge-verdict/1\""));

        let good = json!({
            "schema": "judge-verdict/1", "verdict": "NEEDS_REVISION",
            "checks": [{"id": "c1", "status": "FAIL", "evidence": "boom"}],
            "failure_signature": "sig-1", "fixability": "automatic", "confidence": "high"
        });
        assert!(validate_judge_verdict(&good).0);

        let bad = json!({
            "schema": "judge-verdict/2", "verdict": "PASS",
            "checks": [{"id": "c1", "status": "FAIL", "evidence": "boom"}],
            "fixability": "automatic", "confidence": "high"
        });
        let (ok, errors) = validate_judge_verdict(&bad);
        assert!(!ok);
        assert!(errors.contains(&"schema must be \"judge-verdict/1\", got \"judge-verdict/2\".".to_string()));
        assert!(errors.contains(&"verdict must not be PASS when any check has status FAIL — a PASS verdict must not carry a FAIL check.".to_string()));
        assert!(errors.contains(&"failure_signature is required (non-empty string) when any check has status FAIL.".to_string()));

        assert_eq!(derive_model_independence(Some("a"), Some("pinned"), Some("b"), Some("pinned")), "confirmed");
        assert_eq!(derive_model_independence(Some("a"), Some("pinned"), Some("a"), Some("pinned")), "same-model");
        assert_eq!(derive_model_independence(Some("a"), Some("pinned"), None, None), "unverified");
    }

    // ── wl-3: judge close door (docs/history/workflow-lessons/plan.md) ─────
    //
    // `bee close` gains a blocking judge-debt door for standard/high-risk
    // routes: a capped `behavior_change` cell with no recorded judge verdict
    // (`trace.semantic_judge`) refuses close, exactly like the D1
    // scribing-debt door refuses on uncaptured capture. The door itself
    // lives in verbs/drivers/close.rs; these tests exercise it here,
    // alongside the rest of the judge surface.

    // hpf-1 (review-p1-fixes, 2026-08-12): `capped_at` moved from
    // "2026-08-10" to a stamp AFTER `JUDGE_DOOR_INTRODUCED_AT`
    // ("2026-08-11T00:00:00.000Z") — the grandfather clause this cell adds
    // means a pre-door capped_at is never debt, so every fixture below that
    // means to exercise the door itself must postdate it, or the door tests
    // would silently stop testing anything.
    fn capped_behavior_change_cell(feature: &str, id: &str, judged: bool) -> Value {
        let trace = if judged {
            json!({
                "behavior_change": true,
                "capped_at": "2026-08-12T00:00:00.000Z",
                "semantic_judge": [{"schema": "judge-verdict/1", "verdict": "PASS", "checks": []}],
            })
        } else {
            json!({
                "behavior_change": true,
                "capped_at": "2026-08-12T00:00:00.000Z",
            })
        };
        json!({"id": id, "feature": feature, "status": "capped", "trace": trace})
    }

    /// A standard-lane feature with an unjudged capped `behavior_change`
    /// cell grows a BLOCKING judge-debt door naming the cell id, with a
    /// remedy command stated on the door itself.
    #[test]
    fn judge_debt_door_blocks_a_standard_lane_feature_with_an_unjudged_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // The live shape: `mode: "feature"` with the lane classification
        // under `route.lane` — this DOES grow the door.
        write_lane_record_routed(root, "demo", "execution", Some("standard"), true);
        write_cell_fixture(root, "demo-1", &capped_behavior_change_cell("demo", "demo-1", false));

        let doors = crate::verbs::drivers::build_close_report_doors(root, "demo").unwrap();
        let judge_door = doors.iter().find(|d| d.door == "judge-debt").expect("door must exist for a standard route");
        assert!(judge_door.blocking, "an unjudged behavior_change cell must block");
        assert!(judge_door.detail.contains("demo-1"), "{}", judge_door.detail);
        assert_eq!(judge_door.command, Some("bee cells judge-record"));
    }

    /// The same cell, once a judge verdict is recorded on its trace, clears
    /// the door — still present (the route is still standard/high-risk),
    /// but no longer blocking.
    #[test]
    fn judge_debt_door_clears_once_a_judge_verdict_is_recorded() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_record_routed(root, "demo", "execution", Some("standard"), true);
        write_cell_fixture(root, "demo-1", &capped_behavior_change_cell("demo", "demo-1", true));

        let doors = crate::verbs::drivers::build_close_report_doors(root, "demo").unwrap();
        let judge_door = doors.iter().find(|d| d.door == "judge-debt").expect("door must exist for a standard route");
        assert!(!judge_door.blocking);
        assert_eq!(judge_door.detail, "clear");
    }

    /// A tiny-lane feature never grows the judge-debt door at all — not
    /// merely non-blocking — even with the very same unjudged cell.
    #[test]
    fn judge_debt_door_is_absent_for_a_tiny_lane_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_record_routed(root, "demo", "execution", Some("tiny"), true);
        write_cell_fixture(root, "demo-1", &capped_behavior_change_cell("demo", "demo-1", false));

        let doors = crate::verbs::drivers::build_close_report_doors(root, "demo").unwrap();
        assert!(doors.iter().find(|d| d.door == "judge-debt").is_none(), "tiny lane must never grow this door");
    }

    /// A feature with no lane record at all (never went through shape/gate)
    /// reads as "no route" — same absent-door treatment as tiny/small.
    #[test]
    fn judge_debt_door_is_absent_with_no_lane_record() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "demo-1", &capped_behavior_change_cell("demo", "demo-1", false));

        let doors = crate::verbs::drivers::build_close_report_doors(root, "demo").unwrap();
        assert!(doors.iter().find(|d| d.door == "judge-debt").is_none());
    }

    /// wfl-5: `route.lane` absent EVERYWHERE — the lane record exists
    /// (`mode: "feature"`) but carries no `route`, and there is no
    /// default-state `route.lane` to fall back to either — still reads as
    /// "no route", same absent-door treatment as the no-lane-record case.
    #[test]
    fn judge_debt_door_is_absent_when_route_lane_is_missing_everywhere() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_record_routed(root, "demo", "execution", None, true);
        write_cell_fixture(root, "demo-1", &capped_behavior_change_cell("demo", "demo-1", false));

        let doors = crate::verbs::drivers::build_close_report_doors(root, "demo").unwrap();
        assert!(
            doors.iter().find(|d| d.door == "judge-debt").is_none(),
            "a lane record with no route.lane, and no default-state route, must stay door-free"
        );
    }

    /// End to end: `bee close` on a standard-lane feature with an unjudged
    /// `behavior_change` cell refuses even with tests GREEN, names the cell,
    /// and states both remedy commands.
    #[test]
    fn close_refuses_judge_debt_for_a_standard_lane_feature() {
        let Some(shell) = crate::shell::posix_shell() else { return }; // pub in shell.rs
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(
            root.join(".bee").join("config.json"),
            r#"{"commands":{"test":"echo suite-green"}}"#,
        )
        .unwrap();
        write_lane_record_routed(root, "demo", "execution", Some("standard"), true);
        // Past D1: a scribing run recorded after the cap clears the
        // scribing-debt door, so the judge-debt refusal is the one that
        // actually surfaces. hpf-1: the cell's own capped_at moved to
        // "2026-08-12" (post judge-door), so this stamp moves past it too.
        std::fs::create_dir_all(root.join(".bee").join("logs")).unwrap();
        std::fs::write(
            root.join(".bee").join("logs").join("scribing-runs.jsonl"),
            "{\"feature\":\"demo\",\"ts\":\"2026-08-12T00:00:01.000Z\"}\n",
        )
        .unwrap();
        write_cell_fixture(root, "demo-1", &capped_behavior_change_cell("demo", "demo-1", false));

        let declared = crate::verbs::drivers::declared_test_commands(root).unwrap();
        let Out::Emit(result, text, code) =
            crate::verbs::drivers::close_handler(root, "demo", false, declared, Some(shell), &HashMap::new())
                .unwrap()
        else {
            panic!()
        };
        assert_eq!(code, 1, "judge debt refuses even though tests are green");
        let lines: Vec<&str> = text.split('\n').collect();
        assert!(
            lines[0].starts_with(crate::verbs::drivers::CLOSE_JUDGE_DEBT_PREFIX),
            "refusal headline must start with the pinned prefix: {}",
            lines[0]
        );
        assert!(lines[0].contains("demo-1"), "{}", lines[0]);
        assert!(lines[1].contains("bee cells judge"), "{}", lines[1]);
        assert!(lines[1].contains("bee cells judge-record"), "{}", lines[1]);
        assert!(lines[2].starts_with("next:"));
        let doors = result.get("doors").unwrap().as_array().unwrap();
        assert_eq!(doors.iter().find(|d| d["door"] == "judge-debt").unwrap()["blocking"], json!(true));
    }

    // ── hpf-1 (review-p1-fixes, 2026-08-12): route ownership, grandfather,
    // deferral, and the archived remedy ─────────────────────────────────────

    fn write_default_state_with_route(root: &Path, route_feature: &str, route_lane: &str) {
        let dir = root.join(".bee");
        std::fs::create_dir_all(&dir).unwrap();
        let body = json!({"route": {"lane": route_lane, "feature": route_feature}});
        std::fs::write(bstate::state_path(root), jsjson::stringify_pretty(&body)).unwrap();
    }

    /// P1: a default-state route recorded for a DIFFERENT (high-risk)
    /// feature must never be read as THIS (small) feature's own route — the
    /// live bug: a small feature's close was blocked by a judge-debt door
    /// that belonged to someone else's route. No lane record at all here,
    /// and the state route's owner is a stranger, so the door must not grow.
    #[test]
    fn feature_route_ignores_a_default_state_route_owned_by_another_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_default_state_with_route(root, "unrelated-high-risk-feature", "high-risk");
        write_cell_fixture(root, "small-1", &capped_behavior_change_cell("small-feature", "small-1", false));

        assert_eq!(
            crate::verbs::drivers::feature_route(root, "small-feature").unwrap(),
            None,
            "a route owned by another feature must never be read as this one's own"
        );
        let doors = crate::verbs::drivers::build_close_report_doors(root, "small-feature").unwrap();
        assert!(
            doors.iter().find(|d| d.door == "judge-debt").is_none(),
            "a small feature must not grow the judge-debt door off someone else's route"
        );
    }

    /// The other direction: a standard feature that legitimately OWNS the
    /// default-state route (its own most recent `state route --set`) must
    /// not lose its door just because the ownership check now exists.
    #[test]
    fn feature_route_reads_a_default_state_route_it_owns() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_default_state_with_route(root, "standard-feature", "standard");
        write_cell_fixture(
            root,
            "standard-1",
            &capped_behavior_change_cell("standard-feature", "standard-1", false),
        );

        assert_eq!(
            crate::verbs::drivers::feature_route(root, "standard-feature").unwrap(),
            Some("standard".to_string())
        );
        let doors = crate::verbs::drivers::build_close_report_doors(root, "standard-feature").unwrap();
        let judge_door =
            doors.iter().find(|d| d.door == "judge-debt").expect("a standard feature must keep its own door");
        assert!(judge_door.blocking);
    }

    /// wfl-5's live shape (`mode: "feature"`) must never be misread as lane
    /// "feature" — that string is not a lane class, it is the workflow
    /// class every ordinary lane record carries.
    #[test]
    fn feature_route_lane_mode_feature_is_not_a_lane_class() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // `mode: "feature"`, no `route` anywhere (lane or default state).
        write_lane_record_routed(root, "demo", "execution", None, true);
        assert_eq!(crate::verbs::drivers::feature_route(root, "demo").unwrap(), None);
    }

    /// A lane record's `mode` that genuinely happens to spell a lane class
    /// (the last-resort fallback) IS honored once no route names this
    /// feature anywhere.
    #[test]
    fn feature_route_falls_back_to_a_lane_mode_that_names_a_lane_class() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = lanes_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let body = json!({"feature": "demo", "phase": "execution", "mode": "high-risk", "approved_gates": {"execution": true}});
        std::fs::write(dir.join("demo.json"), jsjson::stringify_pretty(&body)).unwrap();
        assert_eq!(crate::verbs::drivers::feature_route(root, "demo").unwrap(), Some("high-risk".to_string()));
    }

    /// A cell capped BEFORE `JUDGE_DOOR_INTRODUCED_AT` predates the door
    /// entirely and is never debt, judged or not — the grandfather clause.
    #[test]
    fn judge_debt_grandfathers_cells_capped_before_the_door_shipped() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_record_routed(root, "demo", "execution", Some("standard"), true);
        let mut cell = capped_behavior_change_cell("demo", "demo-1", false);
        cell["trace"]["capped_at"] = json!("2026-08-05T00:00:00.000Z"); // predates the door
        write_cell_fixture(root, "demo-1", &cell);

        let debt = crate::verbs::drivers::judge_debt(root, "demo").unwrap();
        assert_eq!(debt.count, 0, "a pre-door cap is grandfathered, never debt");
        let doors = crate::verbs::drivers::build_close_report_doors(root, "demo").unwrap();
        let judge_door = doors.iter().find(|d| d.door == "judge-debt").unwrap();
        assert!(!judge_door.blocking);
    }

    /// A cell with no `capped_at` at all reads as pre-door, not debt.
    #[test]
    fn judge_debt_treats_a_missing_capped_at_as_pre_door() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_record_routed(root, "demo", "execution", Some("standard"), true);
        let mut cell = capped_behavior_change_cell("demo", "demo-1", false);
        cell["trace"].as_object_mut().unwrap().remove("capped_at");
        write_cell_fixture(root, "demo-1", &cell);

        let debt = crate::verbs::drivers::judge_debt(root, "demo").unwrap();
        assert_eq!(debt.count, 0, "no capped_at at all counts as pre-door");
    }

    /// A cell capped AT OR AFTER the door's stamp counts as debt, exactly
    /// as `capped_behavior_change_cell`'s fixture (post-door) already
    /// exercises above — pinned here explicitly against the boundary.
    #[test]
    fn judge_debt_counts_a_cell_capped_at_the_door_stamp_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_record_routed(root, "demo", "execution", Some("standard"), true);
        let mut cell = capped_behavior_change_cell("demo", "demo-1", false);
        cell["trace"]["capped_at"] = json!(crate::verbs::drivers::JUDGE_DOOR_INTRODUCED_AT);
        write_cell_fixture(root, "demo-1", &cell);

        let debt = crate::verbs::drivers::judge_debt(root, "demo").unwrap();
        assert_eq!(debt.count, 1, "capped exactly at the door's own stamp is debt (>=)");
    }

    /// A logged `judge-deferral` decision naming the feature clears the
    /// door without touching the underlying count — mirrors the
    /// scribing-debt door's `capture-deferral` escape.
    #[test]
    fn judge_debt_door_clears_with_a_logged_judge_deferral_decision() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_record_routed(root, "demo", "execution", Some("standard"), true);
        write_cell_fixture(root, "demo-1", &capped_behavior_change_cell("demo", "demo-1", false));
        std::fs::write(
            root.join(".bee").join("decisions.jsonl"),
            "{\"id\":\"d1\",\"type\":\"decide\",\"date\":\"2026-08-12T00:00:00.000Z\",\"decision\":\"defer judge for demo\",\"rationale\":\"r\",\"tags\":[\"judge-deferral\"],\"scope\":\"repo\"}\n",
        )
        .unwrap();

        let doors = crate::verbs::drivers::build_close_report_doors(root, "demo").unwrap();
        let judge_door = doors.iter().find(|d| d.door == "judge-debt").unwrap();
        assert!(!judge_door.blocking, "a logged judge-deferral decision must clear the door");
        assert!(judge_door.detail.contains("deferred"), "{}", judge_door.detail);
        assert!(judge_door.detail.contains("demo-1"), "{}", judge_door.detail);
        assert_eq!(judge_door.command, None);

        // A judge-deferral decision naming a DIFFERENT feature never lifts
        // THIS feature's block.
        assert!(!crate::verbs::drivers::has_judge_deferral_decision(root, "elsewhere").unwrap());
    }

    /// When an offending cell id resolves only under the archive,
    /// `cells judge-record` refuses it outright — the door's remedy must
    /// name the unarchive step BEFORE the judge commands.
    #[test]
    fn judge_debt_door_names_unarchive_first_for_an_archived_offender() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_lane_record_routed(root, "demo", "execution", Some("standard"), true);
        let arch = cells_dir(root).join(ARCHIVE_DIR_NAME).join("demo");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(
            arch.join("demo-1.json"),
            jsjson::stringify_pretty(&capped_behavior_change_cell("demo", "demo-1", false)),
        )
        .unwrap();

        let doors = crate::verbs::drivers::build_close_report_doors(root, "demo").unwrap();
        let judge_door = doors.iter().find(|d| d.door == "judge-debt").unwrap();
        assert!(judge_door.blocking);
        let unarchive_at = judge_door.detail.find("bee cells unarchive").expect(&judge_door.detail);
        let judge_record_at = judge_door.detail.find("bee cells judge-record").expect(&judge_door.detail);
        assert!(unarchive_at < judge_record_at, "unarchive must be named before judge-record: {}", judge_door.detail);
        assert_eq!(judge_door.command, Some("bee cells unarchive"));
    }

    // ── schedule ──────────────────────────────────────────────────────────
    #[test]
    fn compute_schedule_waves_and_diagnostics() {
        let cells = vec![
            json!({"id": "a", "status": "open", "deps": [], "files": ["x.js"]}),
            json!({"id": "b", "status": "open", "deps": [], "files": ["x.js"]}),
            json!({"id": "c", "status": "open", "deps": ["a"], "files": ["y.js"]}),
            json!({"id": "d", "status": "capped", "deps": [], "files": ["z.js"]}),
            json!({"id": "e", "status": "open", "deps": ["ghost"], "files": []}),
            json!({"id": "f", "status": "open", "deps": ["e"], "files": ["w.js"]}),
        ];
        let s = compute_schedule(&cells);
        // a/b overlap on x.js -> b defers; c waits for a; e/f unsatisfiable.
        assert_eq!(s.waves, vec![vec!["a".to_string()], vec!["b".to_string(), "c".to_string()]]);
        assert_eq!(s.unsatisfiable, vec![("e".to_string(), "ghost".to_string(), "missing")]);
        assert_eq!(s.empty_files, vec!["e".to_string()]);
        assert!(s.cycles.is_empty());
        assert!(s.obligation_conflicts.is_empty()); // none of x/y/z/w touch a regen-obligated root
    }

    #[test]
    fn compute_schedule_serializes_cells_sharing_a_regen_obligation_root() {
        // ra/rb declare disjoint files (skills/a.md vs skills/b.md — never
        // literally overlapping) but both fall under the "skills" root
        // INVENTORY_ROOTS obligates (release_manifest::INVENTORY_ROOTS,
        // read through the SAME derive_regen_guards() the cells-add
        // REGEN_OBLIGATION refusal uses — never a hand-kept root list).
        // Wave placement must serialize them exactly like a file overlap
        // would, and name the shared root.
        let cells = vec![
            json!({"id": "ra", "status": "open", "deps": [], "files": ["skills/a.md"]}),
            json!({"id": "rb", "status": "open", "deps": [], "files": ["skills/b.md"]}),
        ];
        let s = compute_schedule(&cells);
        assert_eq!(s.waves, vec![vec!["ra".to_string()], vec!["rb".to_string()]]);
        assert_eq!(s.obligation_conflicts, vec![("rb".to_string(), "ra".to_string(), "skills".to_string())]);
    }

    #[test]
    fn compute_schedule_keeps_wave_placement_for_disjoint_cells_with_no_shared_obligation() {
        // Disjoint files, neither under any derived regen-obligation root:
        // placement is byte-identical to the pre-fix behavior — same wave,
        // no conflict recorded.
        let cells = vec![
            json!({"id": "p", "status": "open", "deps": [], "files": ["docs/readme.md"]}),
            json!({"id": "q", "status": "open", "deps": [], "files": ["notes.txt"]}),
        ];
        let s = compute_schedule(&cells);
        assert_eq!(s.waves, vec![vec!["p".to_string(), "q".to_string()]]);
        assert!(s.obligation_conflicts.is_empty());
    }

    // ── test runner ───────────────────────────────────────────────────────
    // decision 13ce1858 (test-cadence-boundary D1): `cap_cell_from_flags`
    // no longer spawns the declared test command at all, in any shape —
    // this whole section (formerly exercising finish_support's own now-
    // deleted `run_declared_tests` copy directly) now proves the ABSENCE
    // of that spawn through the one door left that could still trigger it,
    // the cap.
    fn wf_boundary_cap_flags(id: &str) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: Vec::new(),
            deviations: Vec::new(),
            deviation: None,
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: None,
            inline_reason: None,
            report: Some(default_test_report_json()),
            sync_ack: None,
        }
    }

    fn wf_boundary_cell_body(id: &str) -> Value {
        json!({
            "id": id, "feature": "f", "title": "t", "action": "a",
            "verify": "npm test", "lane": "tiny", "status": "claimed",
            "deps": [], "files": [], "trace": {},
        })
    }

    #[test]
    fn cap_never_spawns_a_process_for_green_red_or_silent_looking_commands() {
        // A command that would pass, one that would print then fail, and
        // one that would fail silently — none of the three are ever run;
        // every cap lands green with "boundary", and no test-results.json
        // is ever written (there is no run to record).
        for cmd in ["exit 0", "echo boom && exit 3", "exit 7"] {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path();
            write_bee_config(root, &json!({"commands": {"test": cmd}}));
            write_cell_fixture(root, "cn-1", &wf_boundary_cell_body("cn-1"));
            let capped = cap_cell_from_flags(root, &wf_boundary_cap_flags("cn-1"), false)
                .unwrap_or_else(|e| panic!("cmd {cmd:?} must never be spawned, so it cannot refuse: {e:?}"));
            assert_eq!(capped["status"], json!("capped"));
            assert_eq!(capped["trace"]["tests"], json!("boundary"));
            assert!(!test_results_path(root).exists(), "cmd {cmd:?}: nothing ran, so nothing was recorded");
        }
    }

    // ── full-failure-evidence (ffe-2) — cap side is dead, boundary owns it ──

    #[test]
    fn a_cap_never_touches_an_existing_stale_failure_log() {
        // A stale record left behind by an earlier `bee close`/`bee
        // worktree merge` red survives a later cap byte-for-byte — the cap
        // neither runs a fresh command nor clears the old evidence; only
        // the boundary verbs own that lifecycle now.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "exit 0"}}));
        let log_rel = fsutil::write_failure_log(root, "finish", 0, "boom\n").unwrap();
        let logged_before = std::fs::read_to_string(root.join(&log_rel)).unwrap();

        write_cell_fixture(root, "cn-2", &wf_boundary_cell_body("cn-2"));
        let capped = cap_cell_from_flags(root, &wf_boundary_cap_flags("cn-2"), false).unwrap();
        assert_eq!(capped["trace"]["tests"], json!("boundary"));

        let logged_after = std::fs::read_to_string(root.join(&log_rel)).unwrap();
        assert_eq!(logged_after, logged_before, "the cap must not touch the stale log at all");
    }

    #[test]
    fn multiple_declared_commands_all_skip_the_test_door_at_cap() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(
            root,
            &json!({"commands": {"test": ["exit 0", "echo boom && exit 3"]}}),
        );
        write_cell_fixture(root, "cn-3", &wf_boundary_cell_body("cn-3"));
        let capped = cap_cell_from_flags(root, &wf_boundary_cap_flags("cn-3"), false)
            .expect("a multi-command declared list is never run at cap either");
        assert_eq!(capped["trace"]["tests"], json!("boundary"));
        assert!(!root.join(fsutil::failure_log_relative("finish", 0)).exists());
        assert!(!root.join(fsutil::failure_log_relative("finish", 1)).exists());
    }

    #[test]
    fn an_unwritable_log_target_never_blocks_a_cap_because_nothing_is_written() {
        // Pre-occupy the log target with a directory — a real run would
        // have to fail to write there; the cap never even tries.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "echo boom && exit 3"}}));
        let rel = fsutil::failure_log_relative("finish", 0);
        std::fs::create_dir_all(root.join(&rel)).unwrap();
        write_cell_fixture(root, "cn-4", &wf_boundary_cell_body("cn-4"));
        let capped = cap_cell_from_flags(root, &wf_boundary_cap_flags("cn-4"), false)
            .expect("an occupied log target can never block a cap that never writes to it");
        assert_eq!(capped["trace"]["tests"], json!("boundary"));
    }

    // ── decision log ──────────────────────────────────────────────────────
    #[test]
    fn log_decision_appends_event_and_taxonomy_candidates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // No taxonomy: bootstrap-safe append.
        log_decision(root, "«x»", "because", &["cells"]).unwrap();
        let text = std::fs::read_to_string(decisions_path(root)).unwrap();
        let event: Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
        assert_eq!(event["type"], json!("decide"));
        assert_eq!(event["decision"], json!("«x»"));
        assert_eq!(event["tags"], json!(["cells"]));
        assert_eq!(event["scope"], json!("repo"));
        // Taxonomy present: unknown tag lands in candidates[].
        std::fs::create_dir_all(root.join("docs").join("decisions")).unwrap();
        std::fs::write(
            taxonomy_path(root),
            r#"{"schema_version": 1, "tags": [{"name": "cells"}], "candidates": []}"#,
        )
        .unwrap();
        log_decision(root, "«y»", "because", &["cells", "brand-new"]).unwrap();
        let taxonomy: Value = serde_json::from_str(&std::fs::read_to_string(taxonomy_path(root)).unwrap()).unwrap();
        assert_eq!(taxonomy["candidates"], json!(["brand-new"]));
        // Safety refusal embeds the JS pattern literal.
        let refusal = thrown(log_decision(root, "token: supersecret1", "r", &["cells"]));
        assert!(refusal.starts_with("Decision rejected: field \"decision\" matches a secret pattern (/\\b(?:api[_-]?key"));
    }

    // ── writeCell funnel + archive txn helpers ────────────────────────────
    #[test]
    fn write_cell_funnel_refuses_archived_and_busy() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        // invalid id
        assert_eq!(
            thrown(write_cell(root, &json!({"id": "../evil"}))),
            "writeCell: cell needs a valid id (got \"../evil\")."
        );
        assert_eq!(
            thrown(write_cell(root, &json!({"title": "no id"}))),
            "writeCell: cell needs a valid id (got undefined)."
        );
        // archived-only id refuses CELL_ARCHIVED
        let arch = cells_dir(root).join(ARCHIVE_DIR_NAME).join("f");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(arch.join("z-1.json"), "{\"id\":\"z-1\"}").unwrap();
        assert_eq!(
            thrown(write_cell(root, &json!({"id": "z-1"}))),
            "writeCell: cell \"z-1\" is archived — unarchive its feature first (bee cells unarchive --feature <feature>)."
        );
        // live archive lock -> CELLS_ARCHIVE_BUSY
        let _held = lock::acquire_store_lock(root, "cells-archive", 1).ok().unwrap();
        let busy = thrown(write_cell(root, &json!({"id": "w-1"})));
        assert!(busy.starts_with("writeCell: cell \"w-1\" write refused — the \"cells-archive\" lock is held by pid="));
        assert!(busy.ends_with("(a live archive/unarchive transaction). Retry once it completes."));
    }

    #[test]
    fn archive_slug_journal_and_summary_helpers() {
        assert_eq!(
            thrown(assert_valid_feature_slug("archiveFeature", "../up")),
            "archiveFeature: invalid feature \"../up\" — use letters, digits, dot, dash, underscore only (no path separators, and never \".\" or \"..\"). Refusing before any file is touched."
        );
        assert_eq!(
            thrown(assert_valid_feature_slug("unarchiveFeature", "  ")),
            "unarchiveFeature: feature is required."
        );
        assert!(assert_valid_feature_slug("archiveFeature", "demo-1").is_ok());
        // Journal recovery reverses completed moves and drops the journal.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let arch = cells_archive_dir(root, "f");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        let from = cell_file(root, "j-1");
        let to = arch.join("j-1.json");
        std::fs::write(&to, "{\"id\":\"j-1\"}").unwrap(); // move completed pre-crash
        std::fs::write(
            archive_journal_path(root, "f"),
            jsjson::stringify(&json!({"op": "archive", "feature": "f", "planned": [
                {"id": "j-1", "from": from.to_string_lossy(), "to": to.to_string_lossy()}
            ]})),
        )
        .unwrap();
        recover_archive_journal(root, "f").unwrap();
        assert!(from.exists(), "completed move must be reversed");
        assert!(!archive_journal_path(root, "f").exists());
        // CUTOVER: a corrupt journal warns and takes readJson's null
        // fallback, which is the `!journal` branch — delete the journal and
        // return, leaving nothing to recover. Same as Node, minus the V8 text.
        std::fs::write(archive_journal_path(root, "f"), "{nope").unwrap();
        recover_archive_journal(root, "f").expect("corrupt journal must not delegate");
        assert!(
            !archive_journal_path(root, "f").exists(),
            "the unusable journal must be removed, exactly as `!journal` did"
        );
    }

    // ── CUTOVER: corrupt JSON is served natively ──────────────────────────
    #[test]
    fn corrupt_store_reads_fail_open_to_the_same_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();

        // read_store_json: corrupt reads exactly like missing (readJson's
        // `fallback`), with a warning instead of a delegation.
        let f = root.join(".bee").join("whatever.json");
        std::fs::write(&f, "{ nope").unwrap();
        assert!(read_store_json(&f).expect("corrupt must not delegate").is_none());
        // …and so does a lone-surrogate escape.
        std::fs::write(&f, r#"{"a":"\uD83D"}"#).unwrap();
        assert!(read_store_json(&f).expect("lone surrogate must not delegate").is_none());

        // archivedSummary: readJson(file, {}) — corrupt yields the same {}.
        std::fs::create_dir_all(cells_dir(root).join(ARCHIVE_DIR_NAME)).unwrap();
        std::fs::write(archive_summary_file(root), "not json at all {").unwrap();
        assert!(archived_summary(root).expect("corrupt summary must not delegate").is_empty());

        // worktree-holds readStore: corrupt falls into the `{holds: []}` shape
        // fallback, so claim-next still runs with no cross-worktree holds.
        std::fs::create_dir_all(root.join(".bee").join("runtime")).unwrap();
        std::fs::write(holds_ledger_path(root), "{\"holds\": [").unwrap();
        assert_eq!(
            read_holds_store(root).expect("corrupt ledger must not delegate"),
            json!({ "holds": [] })
        );
        // A null hold ENTRY is JS-exotic, not a parse failure — still delegates.
        std::fs::write(holds_ledger_path(root), "{\"holds\": [null]}").unwrap();
        assert!(matches!(read_holds_store(root), Err(Fail::Delegate)));
    }

    #[test]
    fn corrupt_lane_record_throws_instead_of_delegating() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(lanes_dir(root)).unwrap();
        let file = lanes_dir(root).join("f.json");

        // readLaneStrict's own deterministic corrupt refusal — reached now by
        // BOTH plain garbage and a lone-surrogate escape.
        std::fs::write(&file, "{ nope").unwrap();
        let feature = json!("f");
        match lane_record_gates(root, Some(&feature)) {
            Err(Fail::Thrown(msg)) => {
                assert!(msg.starts_with("readLaneStrict: lane record "), "{msg}");
                assert!(msg.contains("exists but is corrupt"), "{msg}");
            }
            other => panic!("expected a thrown corrupt refusal, got {other:?}"),
        }
        std::fs::write(&file, r#"{"feature":"f","x":"\ud800"}"#).unwrap();
        match lane_record_gates(root, Some(&feature)) {
            Err(Fail::Thrown(msg)) => assert!(msg.contains("exists but is corrupt"), "{msg}"),
            other => panic!("lone surrogate must refuse, not delegate: {other:?}"),
        }
        // readLane (fail-open display read) takes readJson's null fallback.
        assert_eq!(read_lane_route(root, "f").expect("must not delegate"), None);
    }

    #[test]
    fn parse_json_js_treats_lone_surrogates_as_not_json() {
        assert!(matches!(parse_json_js(r#"{"a":1}"#, false), JsParse::Value(_)));
        assert!(matches!(parse_json_js(r#"{"a":"\ud800"}"#, false), JsParse::NotJson));
        assert!(matches!(parse_json_js("nope", false), JsParse::NotJson));
        // |n| >= 1e21 round-trips now — no delegation, no loss.
        match parse_json_js("[1e21,1e-7]", false) {
            JsParse::Value(v) => assert_eq!(jsjson::stringify(&v), "[1e+21,1e-7]"),
            _ => panic!("large/small magnitudes must parse"),
        }
    }

    #[test]
    fn deviations_file_lone_surrogate_takes_the_free_prose_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("dev.json");
        // Node's `catch`: not JSON -> one deviation per non-blank line.
        std::fs::write(&file, "[\"\\ud800\"]").unwrap();
        let out = parse_deviations_file(file.to_str().unwrap()).expect("must not delegate");
        assert_eq!(out, vec![json!("[\"\\ud800\"]")]);
    }

    // ── trace helpers ─────────────────────────────────────────────────────
    #[test]
    fn trace_merge_release_and_attempt_append() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // merge: object overlays defaults in place, exotics delegate.
        let merged = merge_trace(Some(&json!({"worker": "w9", "extra": 1}))).unwrap();
        assert_eq!(merged.get("worker"), Some(&json!("w9")));
        assert_eq!(merged.keys().last().unwrap(), "extra");
        assert!(matches!(merge_trace(Some(&json!("abc"))), Err(Fail::Delegate)));
        assert!(matches!(merge_trace(Some(&json!([1]))), Err(Fail::Delegate)));
        assert!(merge_trace(Some(&json!(5))).is_ok()); // {...5} === {}
        // releaseTrace clears claim + verify evidence, appends absent keys.
        let released = release_trace(merged);
        assert_eq!(released.get("worker"), Some(&Value::Null));
        assert_eq!(released.get("verify_passed"), Some(&Value::Null));
        assert!(released.contains_key("verify_command"));
        assert!(released.contains_key("verified_at"));
        // appendAttempt reads the LIVE claim for its session identity.
        std::fs::create_dir_all(claims_dir(root)).unwrap();
        match claim_cell_file(root, Some("live-sess"), "t-9", None).unwrap() {
            ClaimFileOutcome::Ok { .. } => {}
            _ => panic!(),
        }
        let trace = append_attempt(root, "t-9", default_trace(), "blocked", Some("cafe00".into()), Some("why"))
            .unwrap();
        let attempts = trace.get("attempts").unwrap().as_array().unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0]["n"], json!(1.0));
        assert_eq!(attempts[0]["claim_session"], json!("live-sess"));
        assert_eq!(attempts[0]["verdict"], json!("blocked"));
        assert_eq!(attempts[0]["failure_signature"], json!("cafe00"));
        assert_eq!(attempts[0]["note"], json!("why"));
        let keys: Vec<&String> = attempts[0].as_object().unwrap().keys().collect();
        assert_eq!(
            keys,
            vec!["n", "at", "claim_session", "claimed_at", "acquired_at", "worker", "verdict", "failure_signature", "note"]
        );
    }

    // ── update validators ─────────────────────────────────────────────────
    #[test]
    fn update_field_validators_and_frozen_hints() {
        assert_eq!(update_field_problem("title", &json!("ok")), None);
        assert_eq!(
            update_field_problem("title", &json!("  ")),
            Some("must be a non-empty string".to_string())
        );
        assert_eq!(
            update_field_problem("deps", &json!(["a", 1])),
            Some("must be an array of strings".to_string())
        );
        assert_eq!(
            update_field_problem("lane", &json!("mega")),
            Some("must be one of: tiny, small, standard, high-risk, spike".to_string())
        );
        assert_eq!(update_field_problem("change_class", &Value::Null), None);
        assert_eq!(update_field_problem("behavior_change", &json!(true)), None);
        // D4 (store `97ce5225`): `tier` is still a FROZEN key — stored records
        // carry the field — but the hint no longer names a verb to set it,
        // because there is none. Retargeted, not dropped: the assertion is
        // still "a frozen key answers with the sentence that replaces it".
        let tier_hint = update_frozen_hint("tier").expect("tier stays a frozen key");
        assert!(tier_hint.contains("retired"), "{tier_hint}");
        assert!(tier_hint.contains("bee cells escalate"), "{tier_hint}");
        assert!(
            !tier_hint.contains("bee cells tier"),
            "no shipped text may send a caller to a verb that no longer exists: {tier_hint}"
        );
        assert_eq!(update_frozen_hint("status"), Some("status moves only through claim/verify/cap/block/drop"));
        assert_eq!(update_frozen_hint("nonsense"), None);
    }

    // ── cells claim-next (R6): the sweep + the selection filters ──────────

    fn cn_root() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        tmp
    }

    fn write_claim_fixture(root: &Path, id: &str, session: Option<&str>, ttl: f64, at: &str) {
        let dir = claims_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let mut claim = Map::new();
        claim.insert("cell".into(), json!(id));
        if let Some(s) = session {
            claim.insert("session".into(), json!(s));
        }
        claim.insert("ttl_seconds".into(), json!(ttl));
        claim.insert("claimed_at".into(), json!(at));
        claim.insert("acquired_at".into(), json!(at));
        std::fs::write(
            dir.join(format!("{id}.json")),
            jsjson::stringify_pretty(&Value::Object(claim)),
        )
        .unwrap();
    }

    fn write_session_fixture(root: &Path, id: &str, heartbeat: &str, lane: Option<&str>) {
        let dir = sessions_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let mut rec = Map::new();
        rec.insert("id".into(), json!(id));
        rec.insert("started_at".into(), json!(heartbeat));
        rec.insert("last_heartbeat".into(), json!(heartbeat));
        rec.insert("lane".into(), lane.map(|l| json!(l)).unwrap_or(Value::Null));
        std::fs::write(
            dir.join(format!("{id}.json")),
            jsjson::stringify_pretty(&Value::Object(rec)),
        )
        .unwrap();
    }

    const OLD: &str = "2020-01-01T00:00:00.000Z";

    // ser-2: a session record marked `status: "closed"` (SessionEnd's clean
    // exit) reads as heartbeat-stale unconditionally — the closed mark
    // itself is what releases it, even with a heartbeat still inside the
    // freshness window. Mirrors the existing "dead" reading this function
    // already gave the sweep.
    #[test]
    fn heartbeat_stale_treats_closed_session_as_stale_regardless_of_recency() {
        let fresh = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();
        let mut session = Map::new();
        session.insert("id".into(), json!("s1"));
        session.insert("last_heartbeat".into(), json!(fresh));
        session.insert("status".into(), json!("closed"));
        assert!(heartbeat_stale(Some(&session), rsv::now_ms()).unwrap());
    }

    // ser-3: `state session release` writes exactly the same
    // `status: "closed"` mark heartbeat_stale already special-cases above
    // (plus `released: true`, which this reading never inspects) — a
    // released record rides that existing not-live path without needing a
    // status value of its own, so `is_concurrent_mode` reads a
    // solely-released session as no peer even with a fresh heartbeat.
    #[test]
    fn is_concurrent_mode_reads_a_released_session_as_not_live() {
        let tmp = cn_root();
        let root = tmp.path();
        let fresh = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();
        write_session_fixture(root, "released-1", &fresh, None);
        // Patch in the release marks write_session_fixture's fixed shape
        // does not carry.
        let file = sessions_dir(root).join("released-1.json");
        let mut rec: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        rec["status"] = json!("closed");
        rec["closed_at"] = json!(fresh);
        rec["released"] = json!(true);
        std::fs::write(&file, jsjson::stringify_pretty(&rec)).unwrap();

        assert!(!is_concurrent_mode(root).unwrap(), "a released session must not read as a peer");
    }

    #[test]
    fn sweep_resets_only_the_claim_it_actually_removed() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        let fresh = rsv::iso_from_ms(now).ok().unwrap();

        // (a) expired claim, dead owner, cell still claimed BY THAT SESSION.
        write_cell_fixture(
            root,
            "a1",
            &json!({"id":"a1","status":"claimed","feature":"f","trace":{"worker":"w","claim_session":"dead"}}),
        );
        write_claim_fixture(root, "a1", Some("dead"), 60.0, OLD);
        write_session_fixture(root, "dead", OLD, None);
        // (b) expired claim, but the cell was RE-claimed by another session.
        write_cell_fixture(
            root,
            "b1",
            &json!({"id":"b1","status":"claimed","feature":"f","trace":{"worker":"w2","claim_session":"someone-else"}}),
        );
        write_claim_fixture(root, "b1", Some("dead"), 60.0, OLD);
        // (c) expired claim whose owner is LIVE — never swept.
        write_cell_fixture(
            root,
            "c1",
            &json!({"id":"c1","status":"claimed","feature":"f","trace":{"worker":"w3","claim_session":"live"}}),
        );
        write_claim_fixture(root, "c1", Some("live"), 60.0, OLD);
        write_session_fixture(root, "live", &fresh, None);
        // (d) an UNEXPIRED claim — never swept.
        write_cell_fixture(
            root,
            "d1",
            &json!({"id":"d1","status":"claimed","feature":"f","trace":{"worker":"w4","claim_session":"dead"}}),
        );
        write_claim_fixture(root, "d1", Some("dead"), 3600.0, &fresh);

        // `caller_session: None` — this pass has no caller to exclude, so it
        // sweeps like the pre-D6 code (row 1's "caller with none" probe for
        // the sweep itself: nothing is excluded, nothing else changes).
        sweep_expired_claims(root, now, None).ok().unwrap();

        let gone = |id: &str| !claims_dir(root).join(format!("{id}.json")).exists();
        assert!(gone("a1"), "expired + stale owner is swept");
        assert!(gone("b1"), "the claim file goes even when the reset is skipped");
        assert!(!gone("c1"), "a live owner is never swept");
        assert!(!gone("d1"), "an unexpired claim is never swept");

        let status = |id: &str| match read_cell_norm(root, id).ok().unwrap() {
            Some(Value::Object(m)) => js_string_or_undefined(m.get("status")),
            _ => panic!("cell {id}"),
        };
        assert_eq!(status("a1"), "blocked", "claimed -> blocked verdict (D4)");
        assert_eq!(status("b1"), "claimed", "claim_session mismatch: never overwritten");
        assert_eq!(status("c1"), "claimed");
        assert_eq!(status("d1"), "claimed");

        // The verdict's trace carries the sweep stamps, clears the claim,
        // and names the dead session + worktree in blocked_reason.
        let a1 = read_cell_norm(root, "a1").ok().unwrap().unwrap();
        let trace = a1.get("trace").unwrap();
        assert_eq!(trace.get("worker"), Some(&Value::Null));
        assert_eq!(trace.get("claimed_at"), Some(&Value::Null));
        assert_eq!(trace.get("claim_session"), Some(&Value::Null));
        assert_eq!(trace.get("swept_from_session"), Some(&json!("dead")));
        assert_eq!(
            trace.get("swept_at"),
            Some(&json!(rsv::iso_from_ms(now).ok().unwrap()))
        );
        let blocked_reason = match trace.get("blocked_reason") {
            Some(Value::String(s)) => s.clone(),
            other => panic!("expected a string blocked_reason, got {other:?}"),
        };
        assert!(blocked_reason.contains("\"dead\""), "names the dead session: {blocked_reason}");
        assert!(
            blocked_reason.contains("no workspace_id"),
            "a1's claim carries no workspace_id, so the worktree clause says so explicitly: {blocked_reason}"
        );

        // Exactly ONE decision row — b1's skipped reset logs nothing.
        let rows = std::fs::read_to_string(decisions_path(root)).unwrap();
        let lines: Vec<&str> = rows.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("sweep: cell \\\"a1\\\" reset claimed -> blocked"));
        assert!(lines[0].contains("swept session \\\"dead\\\""));

        // Idempotent: a second pass has nothing left to trigger on.
        sweep_expired_claims(root, now, None).ok().unwrap();
        let rows2 = std::fs::read_to_string(decisions_path(root)).unwrap();
        assert_eq!(rows2.lines().filter(|l| !l.trim().is_empty()).count(), 1);
    }

    /// srd-1 (sweep-recovery-door): the summary `sweep_expired_claims`
    /// returns must agree with the decision rows it logs for the same run —
    /// `released` is every claim actually removed (including the Untouched
    /// one, which logs no row at all), `parked` is exactly the Blocked ids,
    /// and `unreachable` is exactly the Unreachable ids.
    #[test]
    fn sweep_summary_sets_agree_with_the_decision_rows_it_writes() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();

        // (a) parked: expired claim, dead owner, cell still claimed BY THAT
        // SESSION -> SweepResetOutcome::Blocked.
        write_cell_fixture(
            root,
            "a1",
            &json!({"id":"a1","status":"claimed","feature":"f","trace":{"worker":"w","claim_session":"dead"}}),
        );
        write_claim_fixture(root, "a1", Some("dead"), 60.0, OLD);
        write_session_fixture(root, "dead", OLD, None);

        // (b) released but untouched: expired claim, cell RE-claimed by
        // another session -> SweepResetOutcome::Untouched, no decision row.
        write_cell_fixture(
            root,
            "b1",
            &json!({"id":"b1","status":"claimed","feature":"f","trace":{"worker":"w2","claim_session":"someone-else"}}),
        );
        write_claim_fixture(root, "b1", Some("dead"), 60.0, OLD);

        // (c) unreachable: no cell fixture in this store at all.
        write_claim_fixture_ws(root, "far-1", Some("dead"), 60.0, OLD, Some("wt-far"));
        write_workspace_fixture(root, "wt-far", "/repos/wt-far");

        let summary = sweep_expired_claims(root, now, None).ok().unwrap();

        let expect_released: std::collections::BTreeSet<String> =
            ["a1", "b1", "far-1"].iter().map(|s| s.to_string()).collect();
        let expect_parked: std::collections::BTreeSet<String> =
            ["a1"].iter().map(|s| s.to_string()).collect();
        let expect_unreachable: std::collections::BTreeSet<String> =
            ["far-1"].iter().map(|s| s.to_string()).collect();
        assert_eq!(summary.released, expect_released, "released: every claim actually removed");
        assert_eq!(summary.parked, expect_parked, "parked: exactly the Blocked ids");
        assert_eq!(summary.unreachable, expect_unreachable, "unreachable: exactly the Unreachable ids");

        let rows = std::fs::read_to_string(decisions_path(root)).unwrap();
        let lines: Vec<&str> = rows.lines().filter(|l| !l.trim().is_empty()).collect();
        // Untouched (b1) logs no row at all; only the parked and unreachable
        // ids each get exactly one decision row. Assert that per id rather
        // than by a bare total: a count with no message reports "2 != 3" and
        // names neither the id that went missing nor the row that appeared,
        // which is how this test failed under parallel load with nothing to
        // read afterwards.
        // Match the id in its QUOTED cell context, never as a bare substring:
        // every row carries a random uuid, and a uuid containing the letters
        // of a short cell id (`5b1e6123…` contains `b1`) made the bare
        // `rows.contains("b1")` form fail at random. That is what made this
        // test look load-flaky — more runs, more uuids, more collisions.
        let cell_named = |id: &str| format!("cell \\\"{id}\\\"");
        for id in ["a1", "far-1"] {
            let hits = lines.iter().filter(|l| l.contains(&cell_named(id))).count();
            assert_eq!(hits, 1, "{id} must have exactly one decision row; rows were:\n{rows}");
        }
        assert_eq!(
            lines.len(),
            summary.parked.len() + summary.unreachable.len(),
            "only parked and unreachable ids log rows; rows were:\n{rows}"
        );
        assert!(
            !rows.contains(&cell_named("b1")),
            "b1 is released but untouched — no decision row: {rows}"
        );
        for id in &summary.parked {
            assert!(
                rows.contains(&format!("cell \\\"{id}\\\" reset claimed -> blocked")),
                "parked id {id} must have a Blocked decision row: {rows}"
            );
        }
        for id in &summary.unreachable {
            assert!(rows.contains(id.as_str()), "unreachable id {id} must be named in a decision row: {rows}");
            assert!(rows.contains("not readable in this store"), "{rows}");
        }
    }

    #[test]
    fn sweep_of_a_sessionless_claim_names_none_in_its_decision_row() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        write_cell_fixture(
            root,
            "s1",
            &json!({"id":"s1","status":"claimed","feature":"f","trace":{"worker":"w"}}),
        );
        write_claim_fixture(root, "s1", None, 60.0, OLD);
        sweep_expired_claims(root, now, None).ok().unwrap();
        let s1 = read_cell_norm(root, "s1").ok().unwrap().unwrap();
        assert_eq!(s1.get("status"), Some(&json!("blocked")));
        assert_eq!(
            s1.get("trace").and_then(|t| t.get("swept_from_session")),
            Some(&Value::Null)
        );
        let blocked_reason = match s1.get("trace").and_then(|t| t.get("blocked_reason")) {
            Some(Value::String(s)) => s.clone(),
            other => panic!("expected a string blocked_reason, got {other:?}"),
        };
        assert!(blocked_reason.contains("none (sessionless)"), "{blocked_reason}");
        let rows = std::fs::read_to_string(decisions_path(root)).unwrap();
        assert!(rows.contains("swept session \\\"none (sessionless)\\\""));
    }

    // ── sweep-at-every-door (E1): caller self-exclusion (D6), the
    // claimed->blocked verdict's worktree resolution (D4), the store
    // boundary (D5), and reopen clearing what the sweep wrote ────────────

    fn write_claim_fixture_ws(
        root: &Path,
        id: &str,
        session: Option<&str>,
        ttl: f64,
        at: &str,
        workspace_id: Option<&str>,
    ) {
        let dir = claims_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let mut claim = Map::new();
        claim.insert("cell".into(), json!(id));
        if let Some(s) = session {
            claim.insert("session".into(), json!(s));
        }
        if let Some(w) = workspace_id {
            claim.insert("workspace_id".into(), json!(w));
        }
        claim.insert("ttl_seconds".into(), json!(ttl));
        claim.insert("claimed_at".into(), json!(at));
        claim.insert("acquired_at".into(), json!(at));
        std::fs::write(
            dir.join(format!("{id}.json")),
            jsjson::stringify_pretty(&Value::Object(claim)),
        )
        .unwrap();
    }

    fn write_workspace_fixture(root: &Path, id: &str, worktree_root: &str) {
        ws::register_workspace(
            root,
            ws::RegisterSpec {
                id,
                kind: "worktree",
                root: worktree_root,
                branch: None,
                base_sha: None,
            },
            "2026-01-01T00:00:00.000Z",
        )
        .unwrap();
    }

    /// row 8: the E1 contract — a claim owned by the calling session is
    /// never swept, even with both TTL and heartbeat stale.
    #[test]
    fn sweep_never_takes_the_calling_sessions_own_claim() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        write_cell_fixture(
            root,
            "mine",
            &json!({"id":"mine","status":"claimed","feature":"f","trace":{"worker":"w","claim_session":"me"}}),
        );
        write_claim_fixture(root, "mine", Some("me"), 60.0, OLD);
        write_session_fixture(root, "me", OLD, None); // stale heartbeat too — still never swept

        sweep_expired_claims(root, now, Some("me")).ok().unwrap();

        assert!(
            claims_dir(root).join("mine.json").exists(),
            "the caller's own expired, stale claim survives (D6)"
        );
        let cell = read_cell_norm(root, "mine").ok().unwrap().unwrap();
        assert_eq!(cell.get("status"), Some(&json!("claimed")));
        assert!(
            !decisions_path(root).exists()
                || std::fs::read_to_string(decisions_path(root)).unwrap().trim().is_empty()
        );
    }

    /// row 3: TTL and heartbeat are both `<=` comparisons — exactly-at-the-
    /// boundary counts as due, not "not yet".
    #[test]
    fn sweep_treats_exact_ttl_and_heartbeat_boundaries_as_due() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();

        let claimed_at = rsv::iso_from_ms(now - 60_000.0).ok().unwrap();
        write_cell_fixture(
            root,
            "ttl-edge",
            &json!({"id":"ttl-edge","status":"claimed","feature":"f","trace":{"worker":"w","claim_session":"dead"}}),
        );
        write_claim_fixture(root, "ttl-edge", Some("dead"), 60.0, &claimed_at);
        write_session_fixture(root, "dead", OLD, None);

        let hb_edge = rsv::iso_from_ms(now - HEARTBEAT_STALE_SECONDS * 1000.0).ok().unwrap();
        write_cell_fixture(
            root,
            "hb-edge",
            &json!({"id":"hb-edge","status":"claimed","feature":"f","trace":{"worker":"w","claim_session":"edge"}}),
        );
        write_claim_fixture(root, "hb-edge", Some("edge"), 60.0, OLD);
        write_session_fixture(root, "edge", &hb_edge, None);

        sweep_expired_claims(root, now, None).ok().unwrap();

        let status = |id: &str| match read_cell_norm(root, id).ok().unwrap() {
            Some(Value::Object(m)) => js_string_or_undefined(m.get("status")),
            _ => panic!("cell {id}"),
        };
        assert_eq!(status("ttl-edge"), "blocked", "TTL exactly at its boundary counts as expired (<=)");
        assert_eq!(status("hb-edge"), "blocked", "heartbeat exactly at 900s counts as stale (<=)");
    }

    /// row 5: `claimed -> blocked` is the ONLY transition the sweep may
    /// make. A claim file that outlives its cell's `status` (open, capped,
    /// dropped, already blocked) never moves that cell, and logs nothing —
    /// this is the pre-existing, silent skip, unrelated to D5's report.
    #[test]
    fn sweep_never_moves_a_cell_that_is_not_claimed() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        let fixtures = [("op", "open"), ("cp", "capped"), ("dr", "dropped"), ("bl", "blocked")];
        for (id, status) in fixtures {
            write_cell_fixture(
                root,
                id,
                &json!({"id": id, "status": status, "feature": "f", "trace": {"worker": "w", "claim_session": "dead"}}),
            );
            write_claim_fixture(root, id, Some("dead"), 60.0, OLD);
        }
        write_session_fixture(root, "dead", OLD, None);

        sweep_expired_claims(root, now, None).ok().unwrap();

        for (id, status) in fixtures {
            assert!(
                !claims_dir(root).join(format!("{id}.json")).exists(),
                "the claim is still removed for {id}"
            );
            let cell = read_cell_norm(root, id).ok().unwrap().unwrap();
            assert_eq!(cell.get("status"), Some(&json!(status)), "never moved by the sweep: {id}");
        }
        assert!(
            !decisions_path(root).exists()
                || std::fs::read_to_string(decisions_path(root)).unwrap().trim().is_empty(),
            "a non-claimed cell's swept claim logs no decision row"
        );
    }

    /// row 6, primary: the cell is absent from the sweeper's own store (D5)
    /// — a granted worktree's own `.bee/cells` most likely holds it. The
    /// claim is still removed; no cell is written anywhere in THIS store;
    /// the cell id and its worktree (resolved through the claim's
    /// `workspace_id`) are named in the decision row.
    #[test]
    fn sweep_of_an_unreachable_cell_removes_the_claim_writes_no_cell_and_names_the_worktree() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        // NOTE: no write_cell_fixture for "far-1" — this store never had it.
        write_claim_fixture_ws(root, "far-1", Some("dead"), 60.0, OLD, Some("wt-far"));
        write_session_fixture(root, "dead", OLD, None);
        write_workspace_fixture(root, "wt-far", "/repos/wt-far");

        sweep_expired_claims(root, now, None).ok().unwrap();

        assert!(!claims_dir(root).join("far-1.json").exists(), "the claim is removed either way (D5)");
        assert!(
            !cells_dir(root).join("far-1.json").exists(),
            "no cell record is ever written in this store (D5)"
        );

        let rows = std::fs::read_to_string(decisions_path(root)).unwrap();
        assert!(rows.contains("far-1"), "names the cell id: {rows}");
        assert!(rows.contains("/repos/wt-far"), "names the holding worktree: {rows}");
        assert!(rows.contains("not readable in this store"), "{rows}");
    }

    /// row 6: the claim carries no `workspace_id` at all — the report names
    /// that explicitly rather than omitting the clause.
    #[test]
    fn sweep_of_an_unreachable_cell_with_no_workspace_id_names_that_explicitly() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        write_claim_fixture(root, "far-2", Some("dead"), 60.0, OLD);
        write_session_fixture(root, "dead", OLD, None);

        sweep_expired_claims(root, now, None).ok().unwrap();

        assert!(!cells_dir(root).join("far-2.json").exists());
        let rows = std::fs::read_to_string(decisions_path(root)).unwrap();
        assert!(rows.contains("far-2"));
        assert!(rows.contains("no workspace_id"), "{rows}");
    }

    /// row 6 / 5 (missing workspace record): the claim names a
    /// `workspace_id` but no such workspace was ever registered — the
    /// report names that explicitly too.
    #[test]
    fn sweep_of_an_unreachable_cell_with_a_missing_workspace_record_names_that_explicitly() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        write_claim_fixture_ws(root, "far-3", Some("dead"), 60.0, OLD, Some("wt-gone"));
        write_session_fixture(root, "dead", OLD, None);
        // No write_workspace_fixture — the workspace record is missing.

        sweep_expired_claims(root, now, None).ok().unwrap();

        assert!(!cells_dir(root).join("far-3.json").exists());
        let rows = std::fs::read_to_string(decisions_path(root)).unwrap();
        assert!(rows.contains("wt-gone"), "{rows}");
        assert!(rows.contains("no readable record"), "{rows}");
    }

    const CELLS_REOPEN_BEHAVIOR_CHILD: &str = "verbs::cells::tests::cells_reopen_behavior_child";

    /// Runs ONLY as a child of the test below — reopens cell "x1" through
    /// the REAL `cells reopen` CLI door, resolving its store root off its
    /// own cwd (process-global, same isolation `cells_update_behavior_child`
    /// above uses).
    #[test]
    #[ignore = "spawned by reopen_clears_the_blocked_reason_the_sweep_wrote"]
    fn cells_reopen_behavior_child() {
        let (flags, use_json) =
            rsv::parse_flags(&["--id", "x1", "--reason", "investigating the crashed session's work"])
                .expect("well-formed fixture argv");
        run_reopen(flags, use_json, Instant::now());
    }

    fn cells_reopen_behavior_run(root: &Path) -> std::process::Output {
        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["--exact", CELLS_REOPEN_BEHAVIOR_CHILD, "--ignored", "--test-threads", "1"]);
        cmd.current_dir(root);
        cmd.output().expect("spawn the test binary")
    }

    /// row 9: `bee cells reopen` clears `trace.blocked_reason` because the
    /// sweep writes it at the SAME key `reopenCell` already nulls
    /// (`handlers_close.rs:914`) — the whole point of using that key rather
    /// than a top-level one.
    #[test]
    fn reopen_clears_the_blocked_reason_the_sweep_wrote() {
        let tmp = cn_root();
        let root = tmp.path();
        // `run_reopen` dispatches through the REAL CLI door, which resolves
        // its store root via `resolve_store_root` — unlike the library-level
        // sweep calls above, that needs `.bee/onboarding.json` (or a `.git`)
        // to recognize `root` as a bee repo at all.
        bp28_repo(root);
        let now = rsv::now_ms();
        write_cell_fixture(
            root,
            "x1",
            &json!({"id":"x1","status":"claimed","feature":"f","trace":{"worker":"w","claim_session":"dead"}}),
        );
        write_claim_fixture(root, "x1", Some("dead"), 60.0, OLD);
        write_session_fixture(root, "dead", OLD, None);

        sweep_expired_claims(root, now, None).ok().unwrap();
        let blocked = read_cell_norm(root, "x1").ok().unwrap().unwrap();
        assert_eq!(blocked.get("status"), Some(&json!("blocked")));
        assert!(matches!(
            blocked.get("trace").and_then(|t| t.get("blocked_reason")),
            Some(Value::String(_))
        ));

        let out = cells_reopen_behavior_run(root);
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );

        let reopened = read_cell_norm(root, "x1").ok().unwrap().unwrap();
        assert_eq!(reopened.get("status"), Some(&json!("open")));
        assert_eq!(reopened.get("trace").and_then(|t| t.get("blocked_reason")), Some(&Value::Null));
    }

    #[test]
    fn resolve_pipeline_refuses_a_bound_but_broken_lane() {
        let tmp = cn_root();
        let root = tmp.path();
        let fresh = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();

        // No session record at all → the default pipeline.
        match resolve_pipeline(root, root, "nobody").ok().unwrap() {
            Pipeline::Ok { feature, execution_approved } => {
                assert!(feature.is_none() && !execution_approved);
            }
            Pipeline::Refused { .. } => panic!("default expected"),
        }

        // Bound to a lane with no record → LANE_MISSING.
        write_session_fixture(root, "s1", &fresh, Some("nope"));
        match resolve_pipeline(root, root, "s1").ok().unwrap() {
            Pipeline::Refused { code, reason } => {
                assert_eq!(code, "LANE_MISSING");
                assert!(reason.contains("session \"s1\" is bound to lane \"nope\" but"));
                assert!(reason.contains("does not exist"));
            }
            Pipeline::Ok { .. } => panic!("LANE_MISSING expected"),
        }

        // Bound to an invalid lane NAME → LANE_INVALID (lanePath's throw).
        write_session_fixture(root, "s2", &fresh, Some("a/b"));
        match resolve_pipeline(root, root, "s2").ok().unwrap() {
            Pipeline::Refused { code, reason } => {
                assert_eq!(code, "LANE_INVALID");
                assert!(reason.contains("lane feature must be a plain id (no path separators)"));
            }
            Pipeline::Ok { .. } => panic!("LANE_INVALID expected"),
        }

        // Bound to a lane file that is not a record for THAT feature → LANE_CORRUPT.
        let lanes = root.join(".bee").join("lanes");
        std::fs::create_dir_all(&lanes).unwrap();
        std::fs::write(lanes.join("broken.json"), r#"{"feature":"other"}"#).unwrap();
        write_session_fixture(root, "s3", &fresh, Some("broken"));
        match resolve_pipeline(root, root, "s3").ok().unwrap() {
            Pipeline::Refused { code, .. } => assert_eq!(code, "LANE_CORRUPT"),
            Pipeline::Ok { .. } => panic!("LANE_CORRUPT expected"),
        }

        // A healthy bound lane resolves to ITS OWN feature and gate.
        std::fs::write(
            lanes.join("good.json"),
            r#"{"feature":"good","approved_gates":{"execution":true}}"#,
        )
        .unwrap();
        write_session_fixture(root, "s4", &fresh, Some("good"));
        match resolve_pipeline(root, root, "s4").ok().unwrap() {
            Pipeline::Ok { feature, execution_approved } => {
                assert_eq!(feature.as_deref(), Some("good"));
                assert!(execution_approved);
            }
            Pipeline::Refused { .. } => panic!("lane expected"),
        }
    }

    /// GH#20 + D1 (default-pipeline-liveness): `live_session_facts` is the
    /// single session-record walk `run_claim_next`'s fallback pool now runs —
    /// prove the pure selection logic directly rather than through the full
    /// claim-next dispatch.
    #[test]
    fn live_session_facts_gates_the_default_pipeline_on_peer_liveness() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        let fresh = rsv::iso_from_ms(now).ok().unwrap();

        // No peer at all: the default pipeline is pooled exactly as before.
        let (live_owned, unbound_peer) = live_session_facts(root, "acting", now).ok().unwrap();
        assert!(live_owned.is_empty());
        assert!(!unbound_peer, "no session records at all must never block the default pipeline");

        // The acting session's OWN record — even unbound and fresh — is never
        // its own peer (D2): the loop already skips the acting session's id.
        write_session_fixture(root, "acting", &fresh, None);
        let (live_owned, unbound_peer) = live_session_facts(root, "acting", now).ok().unwrap();
        assert!(live_owned.is_empty());
        assert!(!unbound_peer, "a session can never be its own peer");

        // A live, UNBOUND peer is, by definition, working the default
        // pipeline right now: it must gate the fallback push.
        write_session_fixture(root, "peer", &fresh, None);
        let (live_owned, unbound_peer) = live_session_facts(root, "acting", now).ok().unwrap();
        assert!(live_owned.is_empty(), "an unbound peer is never mistaken for a lane owner");
        assert!(unbound_peer, "a live unbound peer must gate the default pipeline");

        // A STALE heartbeat on that same peer never blocks it — a dead
        // session must never park work forever.
        write_session_fixture(root, "peer", OLD, None);
        let (live_owned, unbound_peer) = live_session_facts(root, "acting", now).ok().unwrap();
        assert!(!unbound_peer, "a stale peer heartbeat must never park the default pipeline forever");
        assert!(live_owned.is_empty());

        // Restored liveness but bound to a LANE: the GH#20 lane list still
        // picks it up, and it no longer counts as an unbound peer — lane
        // pooling behaviour is unchanged by this fix.
        write_session_fixture(root, "peer", &fresh, Some("lane-x"));
        let (live_owned, unbound_peer) = live_session_facts(root, "acting", now).ok().unwrap();
        assert_eq!(live_owned, vec!["lane-x".to_string()]);
        assert!(!unbound_peer, "a lane-bound peer is not working the default pipeline");
    }

    #[test]
    fn candidate_filters_skip_foreign_session_holds_and_foreign_worktree_holds() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        let cell = json!({"id":"x1","status":"open","feature":"f","files":["src/a.ts"]});
        // No holds anywhere → claimable.
        assert!(candidate_ok(root, root, "mine", &cell, now).ok().unwrap());

        // A cross-worktree hold owned by a DIFFERENT checkout blocks it.
        let runtime = root.join(".bee").join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        let stamp = rsv::iso_from_ms(now).ok().unwrap();
        std::fs::write(
            runtime.join("cross-worktree-holds.json"),
            format!(
                r#"{{"holds":[{{"holder":"wt-other","path":"src/a.ts","mirrored_at":"{stamp}","ttl_seconds":3600,"released_at":null}}]}}"#
            ),
        )
        .unwrap();
        assert!(!candidate_ok(root, root, "mine", &cell, now).ok().unwrap());
        // Our OWN holder never blocks us.
        std::fs::write(
            runtime.join("cross-worktree-holds.json"),
            format!(
                r#"{{"holds":[{{"holder":"main","path":"src/a.ts","mirrored_at":"{stamp}","ttl_seconds":3600,"released_at":null}}]}}"#
            ),
        )
        .unwrap();
        assert!(candidate_ok(root, root, "mine", &cell, now).ok().unwrap());
        // A RELEASED hold never blocks.
        std::fs::write(
            runtime.join("cross-worktree-holds.json"),
            format!(
                r#"{{"holds":[{{"holder":"wt-other","path":"src/a.ts","mirrored_at":"{stamp}","ttl_seconds":3600,"released_at":"{stamp}"}}]}}"#
            ),
        )
        .unwrap();
        assert!(candidate_ok(root, root, "mine", &cell, now).ok().unwrap());
        // A cell with NO declared files skips both hold checks entirely.
        let bare = json!({"id":"x2","status":"open","feature":"f"});
        assert!(candidate_ok(root, root, "mine", &bare, now).ok().unwrap());
    }

    /// hha-3 — the read side stops seeing its own work as foreign.
    ///
    /// `cells claim-next` is a control-plane command, so it runs from MAIN;
    /// after hha-1 the mirrored row for a cell whose feature owns a granted
    /// worktree names that WORKTREE, not the typist. Asking "is this row
    /// mine?" from main would make claim-next skip the very cell it exists to
    /// hand out. It asks who owns the CELL instead — so a hold owned by that
    /// cell's own work stream never blocks, and a hold owned by a DIFFERENT
    /// one still does.
    #[test]
    fn claim_next_keeps_a_cell_its_own_worktree_holds_and_skips_another_streams_hold() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, _ungranted) = wf_worktree_fixture(tmp.path());
        // The granted worktree owns feature "hold-holder"; the cell naming
        // that feature lives in MAIN's store — the control-plane shape (the
        // cell is claimed from main, the work happens in the worktree).
        std::fs::create_dir_all(granted.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            granted.join(".bee").join("runtime").join("worktree-identity.json"),
            format!("{}\n", json!({ "feature": "hold-holder" })),
        )
        .unwrap();
        let cell = json!({
            "id": "hha-cell", "status": "open", "feature": "hold-holder",
            "files": ["src/shared.ts"]
        });
        write_cell_fixture(&main, "hha-cell", &cell);

        let now = rsv::now_ms();
        let stamp = rsv::iso_from_ms(now).ok().unwrap();
        let write_hold = |holder: &str| {
            std::fs::write(
                main.join(".bee").join("runtime").join("cross-worktree-holds.json"),
                format!(
                    r#"{{"holds":[{{"holder":"{holder}","path":"src/shared.ts","cell":"hha-cell","mirrored_at":"{stamp}","ttl_seconds":3600,"released_at":null}}]}}"#
                ),
            )
            .unwrap();
        };

        // The cell's OWN worktree holds the path: main still hands it out.
        write_hold("wt-granted");
        assert!(
            candidate_ok(&main, &main, "mine", &cell, now).ok().unwrap(),
            "a cell's own worktree holding its paths is not a foreign hold"
        );

        // A hold owned by a DIFFERENT work stream still skips the cell.
        write_hold("wt-elsewhere");
        assert!(
            !candidate_ok(&main, &main, "mine", &cell, now).ok().unwrap(),
            "another work stream's hold must still block the claim"
        );
    }

    // ── claims (R5): the sweep's gate discipline ──────────────────────────
    // Oracle: test_claims.mjs "sweep: TTL expired AND heartbeat stale IS
    // reclaimed; no gate file leaks", "sweep: TTL expired but heartbeat FRESH
    // is never reclaimed (20260710 — no steal on a stall signal)", and the
    // sweep half of "sweep and adopt skip/refuse while the per-claim gate is
    // held — typed GATE_HELD, never wait". (The adopt half of that row, and
    // the whole msn-12 fencing surface, are covered in § adoption + fencing
    // at the end of this module.)
    #[test]
    fn sweep_skips_a_gated_claim_and_leaks_no_gate_file() {
        let tmp = cn_root();
        let root = tmp.path();
        let now = rsv::now_ms();
        let fresh = rsv::iso_from_ms(now).ok().unwrap();
        let claimed = |session: &str| {
            json!({"id":"x","status":"claimed","feature":"f","trace":{"worker":"w","claim_session":session}})
        };

        // (a) expired + stale owner, but another process holds the per-claim
        //     gate: skipped on the spot (GATE_HELD), never waited out.
        write_cell_fixture(root, "held", &claimed("dead"));
        write_claim_fixture(root, "held", Some("dead"), 60.0, OLD);
        let held_gate = claim_gate_path(root, "held").unwrap();
        std::fs::write(&held_gate, "{}").unwrap(); // another process mid-adopt

        // (b) the SAME shape with a free gate — the control that IS reclaimed.
        write_cell_fixture(root, "free", &claimed("dead"));
        write_claim_fixture(root, "free", Some("dead"), 60.0, OLD);

        // (c) expired TTL but a FRESH owner heartbeat: never reclaimed, and
        //     the gate is never even taken (the heartbeat test precedes
        //     acquireGate).
        write_cell_fixture(root, "alive", &claimed("live"));
        write_claim_fixture(root, "alive", Some("live"), 60.0, OLD);

        write_session_fixture(root, "dead", OLD, None);
        write_session_fixture(root, "live", &fresh, None);

        sweep_expired_claims(root, now, None).ok().unwrap();

        let claim_file = |id: &str| claims_dir(root).join(format!("{id}.json"));
        let status = |id: &str| match read_cell_norm(root, id).ok().unwrap() {
            Some(Value::Object(m)) => js_string_or_undefined(m.get("status")),
            _ => panic!("cell {id}"),
        };

        assert!(claim_file("held").exists(), "a gated claim is skipped, never stolen");
        assert_eq!(status("held"), "claimed", "a skipped claim's cell is never reset");
        assert_eq!(
            std::fs::read_to_string(&held_gate).unwrap(),
            "{}",
            "the other process's gate file is left exactly as found"
        );

        assert!(!claim_file("free").exists(), "expired + stale owner IS reclaimed");
        assert_eq!(status("free"), "blocked", "claimed -> blocked verdict (D4)");
        assert!(
            !claim_gate_path(root, "free").unwrap().exists(),
            "a completed sweep leaves no gate file behind"
        );

        assert!(claim_file("alive").exists(), "a fresh heartbeat is never swept");
        assert_eq!(status("alive"), "claimed");
        assert!(
            !claim_gate_path(root, "alive").unwrap().exists(),
            "the heartbeat check runs before the gate is ever taken"
        );
    }

    // ── claims (R5): releaseClaim's owner ladder ──────────────────────────
    // Oracle: test_claims.mjs "releaseClaim: NOT_OWNER for the old session
    // after adoption, owner release removes the file, NOT_FOUND after". This
    // port returns () — the typed codes are the half the unwind caller
    // ignores — so the ladder is asserted through its disk effect.
    #[test]
    fn release_claim_owner_ladder_and_gate_hygiene() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        match claim_cell_file(control, Some("owner-a"), "r-1", None).unwrap() {
            ClaimFileOutcome::Ok { .. } => {}
            _ => panic!("precondition: r-1 claimed by owner-a"),
        }
        let file = claims_dir(control).join("r-1.json");
        let gate = claim_gate_path(control, "r-1").unwrap();
        let parse = |p: &Path| -> Value {
            serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
        };
        let before = parse(&file);

        // NOT_OWNER: a foreign session's release changes nothing.
        release_claim(control, Some("owner-b"), "r-1").unwrap();
        assert!(file.exists(), "a denied release never removes the claim");
        assert_eq!(parse(&file), before, "a denied release never rewrites the claim");
        assert!(!gate.exists(), "a denied release leaks no gate file");

        // A sessionless release is not the owner here either.
        release_claim(control, None, "r-1").unwrap();
        assert!(file.exists(), "sessionless != owner \"owner-a\"");
        assert_eq!(parse(&file), before);

        // The owner's release removes it, gate-clean.
        release_claim(control, Some("owner-a"), "r-1").unwrap();
        assert!(!file.exists(), "the owner's release removes the claim file");
        assert!(!gate.exists(), "the owner's release leaks no gate file");

        // NOT_FOUND: releasing again is a no-op that never takes the gate.
        release_claim(control, Some("owner-a"), "r-1").unwrap();
        assert!(!file.exists());
        assert!(!gate.exists());
    }

    // ── claims D5 (R5): the Codex session bridge ──────────────────────────
    // Oracle: test_claims.mjs "claimCellFile (hardening-1-7-10 D5 — Codex
    // session bridge): a sessionless claim with EXACTLY ONE fresh live session
    // auto-adopts that session's identity", and its twin "…with TWO OR MORE
    // fresh live sessions still refuses typed SESSION_REQUIRED".
    #[test]
    fn sessionless_claim_adopts_one_live_session_and_refuses_two() {
        let tmp = cn_root();
        let root = tmp.path();
        let fresh = rsv::iso_from_ms(rsv::now_ms()).ok().unwrap();
        let claim_ok = |id: &str, session: Option<&str>| -> Value {
            match claim_cell_file(root, session, id, Some(60.0)).unwrap() {
                ClaimFileOutcome::Ok { claim } => claim,
                ClaimFileOutcome::Refused { code, reason } => {
                    panic!("{id}: expected a claim, got {code}: {reason}")
                }
            }
        };

        // Zero live sessions: a genuinely solo claim stays sessionless and is
        // never marked adopted.
        let solo = claim_ok("d5-solo", None);
        assert!(solo.get("session").is_none(), "a solo sessionless claim omits the session key");
        assert!(solo.get("adopted").is_none(), "nothing was adopted");

        // Exactly one fresh session: its identity is adopted rather than refused.
        write_session_fixture(root, "only-live", &fresh, None);
        let adopted = claim_ok("d5-one", None);
        assert_eq!(adopted.get("session"), Some(&json!("only-live")));
        assert_eq!(adopted.get("adopted"), Some(&json!(true)));
        let on_disk = read_claim(root, "d5-one").unwrap().unwrap();
        assert_eq!(on_disk.get("session"), Some(&json!("only-live")));
        assert_eq!(
            on_disk.get("adopted"),
            Some(&json!(true)),
            "the on-disk record carries the audit marker too"
        );

        // A STALE second session is not ambiguity — one FRESH is still one.
        write_session_fixture(root, "long-gone", OLD, None);
        assert_eq!(claim_ok("d5-still-one", None).get("session"), Some(&json!("only-live")));

        // Two fresh sessions: real ambiguity is refused, never guessed.
        write_session_fixture(root, "second-live", &fresh, None);
        match claim_cell_file(root, None, "d5-two", Some(60.0)).unwrap() {
            ClaimFileOutcome::Refused { code, reason } => {
                assert_eq!(code, "SESSION_REQUIRED");
                // Both identity routes are a pinned part of this refusal —
                // the message is what tells a stuck agent how to proceed.
                assert!(reason.contains("--session-id"), "reason: {reason}");
                assert!(reason.contains("BEE_SESSION_ID"), "reason: {reason}");
            }
            ClaimFileOutcome::Ok { claim } => panic!("two live sessions must refuse, got {claim}"),
        }
        assert!(
            !claims_dir(root).join("d5-two.json").exists(),
            "the refusal leaves no claim file behind"
        );

        // Control: an explicit session id still claims fine, and is never
        // marked adopted (adoption is only ever an inference).
        let explicit = claim_ok("d5-two", Some("second-live"));
        assert_eq!(explicit.get("session"), Some(&json!("second-live")));
        assert!(explicit.get("adopted").is_none());
    }

    // ── claims (R5): resolveSessionId's ordered chain ─────────────────────
    // Oracle: test_claims.mjs "resolveSessionId: explicit flag wins over env;
    // a blank flag falls through to env" + "(hardening-4a): BEE_SESSION_ID
    // wins over legacy CLAUDE_CODE_SESSION_ID".
    const SESSION_CHAIN_CHILD: &str = "verbs::cells::tests::session_id_env_chain_child";

    /// Runs ONLY as a child of the test below, which hands it a controlled
    /// environment. `#[ignore]` keeps it out of the normal pass: this
    /// process's env is shared with every other test in the binary (
    /// state_group's fixtures resolve BEE_SESSION_ID / CLAUDE_CODE_SESSION_ID
    /// live, and the CI runner really does export the latter), so the ordered
    /// chain is exercised out-of-process instead of by mutating env under them.
    #[test]
    #[ignore = "spawned by resolve_session_id_precedence_flag_beats_bee_beats_legacy"]
    fn session_id_env_chain_child() {
        let want = std::env::var("BEE_TEST_EXPECT").unwrap_or_default();
        assert_eq!(
            resolve_session_flag_env(None).unwrap_or_default(),
            want,
            "no-flag resolution"
        );
        // An explicit flag outranks whatever the env says, in every combination.
        assert_eq!(
            resolve_session_flag_env(Some("sess-from-flag")).as_deref(),
            Some("sess-from-flag")
        );
        // A blank / whitespace-only flag is NOT an explicit empty session: it
        // falls through to the same answer the env alone gives.
        assert_eq!(resolve_session_flag_env(Some("")).unwrap_or_default(), want);
        assert_eq!(resolve_session_flag_env(Some("   ")).unwrap_or_default(), want);
    }

    #[test]
    fn resolve_session_id_precedence_flag_beats_bee_beats_legacy() {
        // The env-free half runs in-process: whatever this machine's ambient
        // env says, an explicit flag wins and a blank one falls through to it.
        assert_eq!(
            resolve_session_flag_env(Some("sess-from-flag")).as_deref(),
            Some("sess-from-flag")
        );
        let ambient = resolve_session_flag_env(None);
        assert_eq!(resolve_session_flag_env(Some("")), ambient, "blank flag falls through");
        assert_eq!(resolve_session_flag_env(Some("   ")), ambient);

        // The ordered env half needs a controlled environment — child process.
        // (bee, legacy, expected)
        let cases: &[(Option<&str>, Option<&str>, &str)] = &[
            (None, Some("sess-legacy"), "sess-legacy"),
            (Some("sess-bee"), Some("sess-legacy"), "sess-bee"),
            (Some("   "), Some("sess-legacy"), "sess-legacy"),
            (Some("sess-bee"), None, "sess-bee"),
            (None, None, ""),
        ];
        let exe = std::env::current_exe().expect("test binary path");
        for (bee, legacy, want) in cases {
            let mut cmd = std::process::Command::new(&exe);
            cmd.args(["--exact", SESSION_CHAIN_CHILD, "--ignored", "--test-threads", "1"]);
            cmd.env_remove("BEE_SESSION_ID").env_remove("CLAUDE_CODE_SESSION_ID");
            if let Some(v) = bee {
                cmd.env("BEE_SESSION_ID", v);
            }
            if let Some(v) = legacy {
                cmd.env("CLAUDE_CODE_SESSION_ID", v);
            }
            cmd.env("BEE_TEST_EXPECT", want);
            let out = cmd.output().expect("spawn the test binary");
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            // Tripwire: a filter that matched nothing ALSO exits 0, so the
            // pass count — not the status — is what proves the case ran.
            assert!(
                text.contains("1 passed"),
                "child never ran the case (bee={bee:?} legacy={legacy:?}):\n{text}"
            );
            assert!(
                out.status.success(),
                "chain case failed (bee={bee:?} legacy={legacy:?} want={want:?}):\n{text}"
            );
        }
    }

    // ── cells add (R5): whole-batch report ────────────────────────────────
    // Oracle: test_cells.mjs "addCells aggregates EVERY failing cell in one
    // refusal", "addCells refuses a duplicate id within the batch",
    // "addCells refuses an in-batch cycle", "previewAddCells: a clean batch
    // reports ok:true …and writes nothing", "previewAddCells: a dirty batch
    // names EVERY failing cell", "previewAddCells folds a batch-wide cycle
    // into the cells it touches (ce-2)". buildAddCellsReport is the one
    // engine `cells add` and `cells add --dry-run` share.
    fn addable(id: &str) -> Value {
        json!({
            "id": id, "feature": "batch", "title": format!("title {id}"),
            "action": "do the thing", "verify": "echo ok", "lane": "tiny",
            "role": "code",
            "affects_skills": [], "affects_specs": [],
        })
    }

    #[test]
    fn add_cell_requires_affects_skills_and_affects_specs_on_every_lane() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        for lane in LANES {
            let mut base = json!({
                "id": format!("cell-{lane}"),
                "feature": "feat",
                "title": "title",
                "action": "action",
                "verify": "cargo test",
                "lane": lane,
                "role": "code",
            });
            if lane == "standard" || lane == "high-risk" {
                base["must_haves"] = json!({"truths": ["something true"]});
            }

            // Missing both
            let thrown_both = thrown(validate_new_cell(root, &base));
            assert!(
                thrown_both.contains("addCell: cell is missing required field \"affects_skills\""),
                "lane {lane} must refuse missing affects_skills: {thrown_both}"
            );
            assert!(
                thrown_both.contains("addCell: cell is missing required field \"affects_specs\""),
                "lane {lane} must refuse missing affects_specs: {thrown_both}"
            );

            // Missing affects_skills only
            let mut missing_skills = base.clone();
            missing_skills["affects_specs"] = json!([]);
            let thrown_skills = thrown(validate_new_cell(root, &missing_skills));
            assert!(
                thrown_skills.contains("addCell: cell is missing required field \"affects_skills\". FIX: every cell must declare \"affects_skills\" and \"affects_specs\" arrays (use `[]` if none)."),
                "lane {lane} must name missing affects_skills with FIX: {thrown_skills}"
            );
            assert!(!thrown_skills.contains("missing required field \"affects_specs\""));

            // Missing affects_specs only
            let mut missing_specs = base.clone();
            missing_specs["affects_skills"] = json!([]);
            let thrown_specs = thrown(validate_new_cell(root, &missing_specs));
            assert!(
                thrown_specs.contains("addCell: cell is missing required field \"affects_specs\". FIX: every cell must declare \"affects_skills\" and \"affects_specs\" arrays (use `[]` if none)."),
                "lane {lane} must name missing affects_specs with FIX: {thrown_specs}"
            );
            assert!(!thrown_specs.contains("missing required field \"affects_skills\""));

            // Non-array value
            let mut non_array = base.clone();
            non_array["affects_skills"] = json!("not-an-array");
            non_array["affects_specs"] = json!([]);
            let thrown_non_array = thrown(validate_new_cell(root, &non_array));
            assert!(
                thrown_non_array.contains("addCell: \"affects_skills\" must be an array of strings."),
                "lane {lane} must refuse non-array affects_skills: {thrown_non_array}"
            );

            // Both present as [] -> valid
            let mut valid = base.clone();
            valid["affects_skills"] = json!([]);
            valid["affects_specs"] = json!([]);
            assert!(
                validate_new_cell(root, &valid).is_ok(),
                "lane {lane} accepts [] for affects_skills and affects_specs"
            );
        }

        // Updating an existing cell that lacks the keys still succeeds
        let legacy_cell = json!({
            "id": "legacy-1",
            "feature": "feat",
            "title": "old title",
            "action": "action",
            "verify": "cargo test",
            "lane": "tiny",
            "status": "open",
        });
        write_cell_fixture(root, "legacy-1", &legacy_cell);
        assert!(update_field_problem("title", &json!("updated title")).is_none());
    }

    // wgg-1: `affects_skills` holds repo-relative PATHS. A bare skill name
    // used to sail past `cells add` and only explode at cap, inside the sync
    // door's check (c), where it can never be satisfied — the wrong format
    // caught at the wrong end of the cell's life. It is refused here now.
    #[test]
    fn add_cell_refuses_an_affects_skills_entry_that_is_not_a_skills_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("skills").join("bee-reviewing")).unwrap();
        std::fs::write(root.join("skills").join("bee-reviewing").join("SKILL.md"), "# skill\n").unwrap();

        // A bare name that names a real skill: the refusal spells out the
        // exact replacement path.
        let mut bare = addable("fmt-1");
        bare["affects_skills"] = json!(["bee-reviewing"]);
        let refusal = thrown(validate_new_cell(root, &bare));
        assert!(
            refusal.contains("addCell: \"affects_skills\" entry \"bee-reviewing\" is not a repo-relative path under \"skills/\""),
            "must name the entry: {refusal}"
        );
        assert!(
            refusal.contains("FIX: \"bee-reviewing\" is a bare skill name; use \"skills/bee-reviewing/SKILL.md\" instead."),
            "must name the exact replacement path: {refusal}"
        );

        // A bare name that names no skill still refuses — it just has no
        // exact path to offer, so it names the shape instead.
        let mut unknown = addable("fmt-2");
        unknown["affects_skills"] = json!(["no-such-skill"]);
        let unknown_refusal = thrown(validate_new_cell(root, &unknown));
        assert!(
            unknown_refusal.contains("entry \"no-such-skill\" is not a repo-relative path")
                && unknown_refusal.contains("skills/<skill-name>/SKILL.md"),
            "{unknown_refusal}"
        );
        assert!(!unknown_refusal.contains("is a bare skill name"), "{unknown_refusal}");

        // Whole-batch validation is unchanged: EVERY bad entry is named in
        // one call, in order, and nothing is written.
        let mut many = addable("fmt-3");
        many["affects_skills"] =
            json!(["bee-reviewing", "skills/bee-hive/SKILL.md", "docs/knowledge/index.md", "skills"]);
        let problems = validate_new_cell_problems(root, &many).unwrap();
        assert_eq!(problems.len(), 3, "{problems:?}");
        assert!(problems[0].contains("\"bee-reviewing\""), "{problems:?}");
        assert!(problems[1].contains("\"docs/knowledge/index.md\""), "{problems:?}");
        assert!(problems[2].contains("entry \"skills\""), "{problems:?}");

        // Paths under skills/ pass — including a nested reference file and
        // the "./" spelling the sync door normalizes the same way.
        let mut ok = addable("fmt-4");
        ok["affects_skills"] = json!([
            "skills/bee-hive/SKILL.md",
            "skills/bee-hive/references/hive-reference.md",
            "./skills/bee-hive/SKILL.md"
        ]);
        assert_eq!(validate_new_cell_problems(root, &ok).unwrap(), Vec::<String>::new());

        // affects_specs keeps its shape-only check — it has no cap-time door.
        let mut specs = addable("fmt-5");
        specs["affects_specs"] = json!(["bee-reviewing"]);
        assert!(validate_new_cell(root, &specs).is_ok());
    }

    #[test]
    fn add_cells_report_aggregates_every_failure_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let unwritten = |ids: &[&str]| {
            for id in ids {
                assert!(
                    read_cell_norm(root, id).ok().unwrap().is_none(),
                    "{id} must not exist — the report never writes"
                );
            }
        };

        // A clean batch: every verdict ok, normalized cells handed back, and
        // still nothing on disk (the --dry-run "nothing written" contract).
        let clean = vec![addable("b-1"), addable("b-2")];
        let (ok, rows, normalized) = build_add_cells_report(root, &clean).unwrap();
        assert!(ok);
        assert_eq!(rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), vec!["b-1", "b-2"]);
        assert!(rows.iter().all(|r| r.ok && r.problems.is_empty()));
        assert_eq!(normalized.as_ref().map(Vec::len), Some(2));
        unwritten(&["b-1", "b-2"]);

        // A dirty batch does NOT stop at the first bad cell: both bad cells
        // carry their own problem, the good one still verdicts ok, and no
        // normalized list comes back — that absence is the all-or-nothing
        // mechanism the writer loop depends on.
        let mut bad_lane = addable("d-2");
        bad_lane["lane"] = json!("huge");
        let mut blank_title = addable("d-3");
        blank_title["title"] = json!("");
        let dirty = vec![addable("d-1"), bad_lane, blank_title];
        let (ok, rows, normalized) = build_add_cells_report(root, &dirty).unwrap();
        assert!(!ok);
        assert!(rows[0].ok && rows[0].problems.is_empty(), "the valid cell still verdicts ok");
        assert!(!rows[1].ok);
        assert!(rows[1].problems.iter().any(|p| p.contains("lane")), "{:?}", rows[1].problems);
        assert!(!rows[2].ok);
        assert!(
            rows[2].problems.iter().any(|p| p.contains("\"title\"")),
            "the SECOND bad cell is named too, never swallowed by the first: {:?}",
            rows[2].problems
        );
        assert!(normalized.is_none(), "a dirty batch yields nothing to write");
        unwritten(&["d-1", "d-2", "d-3"]);

        // The per-cell verdict shape the dry-run payload renders.
        let payload = add_report_rows_value(&rows);
        let cells = payload.as_array().unwrap();
        assert_eq!(cells.len(), 3);
        assert_eq!(
            cells[0].as_object().unwrap().keys().collect::<Vec<_>>(),
            vec!["id", "ok", "problems"]
        );
        assert_eq!(cells[0]["id"], json!("d-1"));
        assert_eq!(cells[0]["ok"], json!(true));
        assert_eq!(cells[0]["problems"], json!([]));
        assert_eq!(cells[1]["ok"], json!(false));

        // An in-batch duplicate id: the first occurrence is clean, the repeat
        // carries the duplicate problem.
        let (ok, rows, normalized) = build_add_cells_report(root, &[addable("dup"), addable("dup")]).unwrap();
        assert!(!ok);
        assert!(rows[0].ok, "the first occurrence of the id is not the duplicate");
        assert_eq!(rows[1].problems, vec!["addCells: duplicate id \"dup\" within the batch."]);
        assert!(normalized.is_none());
        unwritten(&["dup"]);

        // A cell with no usable id is still reported, under its index.
        let mut anonymous = addable("x");
        anonymous.as_object_mut().unwrap().shift_remove("id");
        let (ok, rows, _) = build_add_cells_report(root, &[addable("keep"), anonymous]).unwrap();
        assert!(!ok);
        assert_eq!(rows[1].id, "(index 1)");
        assert!(!rows[1].ok);
    }

    #[test]
    fn add_cells_report_folds_a_batch_wide_cycle_onto_every_cell_it_touches() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Control first: the same two cells WITHOUT the back-edge are clean,
        // so the refusal below is the cycle and nothing else.
        let mut a = addable("cyc-a");
        a["deps"] = json!(["cyc-b"]);
        let b = addable("cyc-b");
        let (ok, _, normalized) = build_add_cells_report(root, &[a.clone(), b]).unwrap();
        assert!(ok, "a plain in-batch dependency is legal");
        assert_eq!(normalized.as_ref().map(Vec::len), Some(2));

        let mut b = addable("cyc-b");
        b["deps"] = json!(["cyc-a"]);
        let (ok, rows, normalized) = build_add_cells_report(root, &[a, b]).unwrap();
        assert!(!ok, "a <-> b is a cycle");
        for row in &rows {
            assert!(!row.ok, "{} must fail", row.id);
            assert!(
                row.problems.iter().any(|p| p.contains("dependency cycle refused")),
                "the cycle folds onto {} too: {:?}",
                row.id,
                row.problems
            );
        }
        assert!(normalized.is_none());
        assert!(read_cell_norm(root, "cyc-a").ok().unwrap().is_none());
        assert!(read_cell_norm(root, "cyc-b").ok().unwrap().is_none());
    }

    // ── D3: no cells before the gate (docs/history/hook-teeth CONTEXT.md) ──
    // Oracle: none — new mechanical enforcement. D7's sequencing law: the
    // resolution primitive (`gated_add_refusal`) is proven on its own FIRST
    // — lane vs. default precedence, docs-lane exemption, unknown-feature
    // "no opinion" — before it is exercised through the whole-batch add
    // report the refusal actually wires into.

    fn write_lane_record(root: &Path, feature: &str, phase: &str, mode: Option<&str>, execution: bool) {
        let dir = lanes_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let body = json!({
            "feature": feature,
            "phase": phase,
            "mode": mode,
            "approved_gates": {"execution": execution},
        });
        std::fs::write(dir.join(format!("{feature}.json")), jsjson::stringify_pretty(&body)).unwrap();
    }

    /// wfl-5 (docs/history/workflow-lessons/plan.md): the live lane-record
    /// shape carries the workflow's own mode fixed at `"feature"` and the
    /// lane CLASSIFICATION (tiny/small/standard/high-risk) under
    /// `route.lane` (state_group/workflows.rs `state route --set`), never
    /// under the top-level `mode` field `write_lane_record` above writes
    /// for the docs-exemption checks. This fixture matches that live shape
    /// for the judge-debt door tests below.
    fn write_lane_record_routed(root: &Path, feature: &str, phase: &str, route_lane: Option<&str>, execution: bool) {
        let dir = lanes_dir(root);
        std::fs::create_dir_all(&dir).unwrap();
        let route = route_lane.map(|l| json!({"lane": l}));
        let body = json!({
            "feature": feature,
            "phase": phase,
            "mode": "feature",
            "route": route,
            "approved_gates": {"execution": execution},
        });
        std::fs::write(dir.join(format!("{feature}.json")), jsjson::stringify_pretty(&body)).unwrap();
    }

    fn write_default_state(root: &Path, feature: &str, phase: &str, mode: Option<&str>, execution: bool) {
        let dir = root.join(".bee");
        std::fs::create_dir_all(&dir).unwrap();
        let body = json!({
            "feature": feature,
            "phase": phase,
            "mode": mode,
            "approved_gates": {"execution": execution},
        });
        std::fs::write(bstate::state_path(root), jsjson::stringify_pretty(&body)).unwrap();
    }

    fn addable_for(id: &str, feature: &str) -> Value {
        let mut c = addable(id);
        c["feature"] = json!(feature);
        c
    }

    #[test]
    fn gated_add_refusal_resolves_phase_and_gate_lane_beats_default_unknown_no_opinion() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Unknown feature: no lane record, no default state.json at all —
        // "no opinion", never a guess.
        assert!(gated_add_refusal(root, "nowhere").unwrap().is_none());

        // Lane record, gated (planning) + unapproved -> refused, naming
        // both the feature and the merged-gate remedy.
        write_lane_record(root, "gated-lane", "planning", None, false);
        let reason = gated_add_refusal(root, "gated-lane").unwrap().unwrap();
        assert!(reason.contains("gated-lane"), "{reason}");
        assert!(reason.contains("bee state gate --merge --approved true"), "{reason}");

        // Same lane, execution now approved -> allowed.
        write_lane_record(root, "gated-lane", "planning", None, true);
        assert!(gated_add_refusal(root, "gated-lane").unwrap().is_none());

        // swarming is not a gated phase, even with execution false.
        write_lane_record(root, "swarming-lane", "swarming", None, false);
        assert!(gated_add_refusal(root, "swarming-lane").unwrap().is_none());

        // exploring is gated too, same as planning.
        write_lane_record(root, "exploring-lane", "exploring", None, false);
        assert!(gated_add_refusal(root, "exploring-lane").unwrap().is_some());

        // A docs-lane record (mode "docs") is exempt regardless of phase or
        // gate.
        write_lane_record(root, "docs-lane", "planning", Some("docs"), false);
        assert!(gated_add_refusal(root, "docs-lane").unwrap().is_none());

        // Lane beats default: the lane record's own gated+unapproved state
        // wins even when the default state.json disagrees (approved) for
        // the very same feature.
        write_lane_record(root, "lane-wins", "planning", None, false);
        write_default_state(root, "lane-wins", "swarming", None, true);
        assert!(
            gated_add_refusal(root, "lane-wins").unwrap().is_some(),
            "the lane record must win over the default pipeline"
        );

        // No lane record at all: the default state.json is consulted ONLY
        // when its own feature names this same feature.
        write_default_state(root, "default-only", "planning", None, false);
        assert!(gated_add_refusal(root, "default-only").unwrap().is_some());
        assert!(
            gated_add_refusal(root, "some-other-feature").unwrap().is_none(),
            "a default state.json naming a DIFFERENT feature is no opinion"
        );
    }

    #[test]
    fn add_cells_refuses_whole_batch_when_target_feature_is_gated_and_unapproved() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        write_lane_record(root, "gate-add", "planning", None, false);
        let batch = vec![addable_for("ga-1", "gate-add")];
        let (ok, rows, normalized) = build_add_cells_report(root, &batch).unwrap();
        assert!(!ok);
        assert!(!rows[0].ok);
        assert!(
            rows[0]
                .problems
                .iter()
                .any(|p| p.contains("gate-add") && p.contains("bee state gate --merge --approved true")),
            "{:?}",
            rows[0].problems
        );
        assert!(normalized.is_none());
        assert!(
            read_cell_norm(root, "ga-1").ok().unwrap().is_none(),
            "a gated refusal writes nothing"
        );

        // Execution approved -> allowed.
        write_lane_record(root, "gate-add", "planning", None, true);
        let (ok, rows, normalized) = build_add_cells_report(root, &batch).unwrap();
        assert!(ok, "{:?}", rows.iter().flat_map(|r| r.problems.clone()).collect::<Vec<_>>());
        assert!(normalized.is_some());

        // swarming phase -> allowed even with execution still false.
        write_lane_record(root, "gate-add", "swarming", None, false);
        let (ok, _, _) = build_add_cells_report(root, &batch).unwrap();
        assert!(ok);

        // Mixed batch: one gated-feature cell alongside one open-feature
        // cell — the whole batch refuses (nothing written), and only the
        // gated cell's row names the gated feature.
        write_lane_record(root, "gate-add", "planning", None, false);
        let mixed = vec![addable_for("mix-open", "open-feature"), addable_for("mix-gated", "gate-add")];
        let (ok, rows, normalized) = build_add_cells_report(root, &mixed).unwrap();
        assert!(!ok);
        assert!(rows[0].ok && rows[0].problems.is_empty(), "the open feature's own row carries no gate problem");
        assert!(!rows[1].ok);
        assert!(rows[1].problems.iter().any(|p| p.contains("gate-add")));
        assert!(normalized.is_none(), "one gated cell refuses the whole batch");
    }

    // ── verify:"none" (R5): the no-test-repo sentinel ─────────────────────
    // Oracle: lib/cells.mjs assertVerifySentinelAllowed (decision 55b951e1) —
    // the sentinel is accepted only where the repo has declared itself.
    fn write_bee_config(root: &Path, config: &Value) {
        let dir = root.join(".bee");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.json"), jsjson::stringify_pretty(config)).unwrap();
    }

    #[test]
    fn verify_none_is_accepted_only_in_a_declared_no_test_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut sentinel = addable("v-1");
        sentinel["verify"] = json!("none");

        // Undeclared repo: refused on add AND on update; a real verify passes
        // either way (the control that nothing else in the cell is at fault).
        assert!(
            thrown(validate_new_cell(root, &sentinel)).starts_with("addCell: verify \"none\" is refused"),
        );
        assert!(validate_new_cell(root, &addable("v-1")).is_ok());
        assert!(
            thrown(assert_verify_sentinel_allowed(root, "updateCell", &json!("none")))
                .starts_with("updateCell: verify \"none\" is refused"),
        );
        assert!(assert_verify_sentinel_allowed(root, "updateCell", &json!("npm test")).is_ok());

        // Declared no-test repo: the same sentinel is accepted on both doors.
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        assert!(validate_new_cell(root, &sentinel).is_ok(), "a declared no-test repo accepts it");
        assert!(assert_verify_sentinel_allowed(root, "updateCell", &json!("none")).is_ok());

        // isNoTestRepo's own matrix, read back through the real config reader.
        let declares = |config: Value| -> bool {
            write_bee_config(root, &config);
            is_no_test_repo(&read_commands_slice(root).unwrap())
        };
        assert!(declares(json!({"commands": {"test": "none"}})));
        assert!(declares(json!({"commands": {"test": ["none"]}})));
        assert!(
            !declares(json!({"commands": {"verify": "none"}})),
            "commands.verify is retired — it no longer declares a no-test repo"
        );
        assert!(
            !declares(json!({"commands": {"test": ["none", "npm test"]}})),
            "a list with a real command beside the sentinel is NOT a no-test repo"
        );
        assert!(!declares(json!({"commands": {"test": "npm test"}})));
        assert!(!declares(json!({"commands": {}})));
    }

    #[test]
    fn capping_in_a_no_test_repo_and_a_declared_test_repo_both_run_no_tests_at_cap() {
        let cap_flags = |id: &str| CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: Vec::new(),
            deviations: Vec::new(),
            deviation: None,
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: None,
            inline_reason: None,
            report: Some(default_test_report_json()),
            sync_ack: None,
        };
        let cell_body = |id: &str| {
            json!({
                "id": id, "feature": "f", "title": "t", "action": "a",
                "verify": "none", "lane": "tiny", "status": "claimed",
                "deps": [], "files": [], "trace": {},
            })
        };

        // A repo that declares itself no-test: the sentinel is filtered out of
        // commands.test, no test process is ever spawned, and the cap lands.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "nt-1", &cell_body("nt-1"));
        let capped = cap_cell_from_flags(root, &cap_flags("nt-1"), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
        assert_eq!(
            capped["trace"]["tests"],
            json!("undeclared"),
            "\"none\" is not a command to run"
        );
        assert!(!test_results_path(root).exists(), "nothing ran, so nothing was recorded");

        // decision 13ce1858 (test-cadence-boundary D1): a repo declaring a
        // real command that would fail if run — "exit 3" — is NOT run at
        // cap either, so the cap lands green with `tests: "boundary"`.
        // Tests prove at the boundary (`bee close`/`bee worktree merge`)
        // now, not here.
        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        write_bee_config(root2, &json!({"commands": {"test": "exit 3"}}));
        write_cell_fixture(root2, "nt-2", &cell_body("nt-2"));
        let capped2 = cap_cell_from_flags(root2, &cap_flags("nt-2"), false)
            .expect("a cap never spawns the declared command, so it cannot go red");
        assert_eq!(capped2["status"], json!("capped"));
        assert_eq!(
            capped2["trace"]["tests"],
            json!("boundary"),
            "a declared-test repo records boundary, not a verdict earned here"
        );
        assert!(!test_results_path(root2).exists(), "nothing ran, so nothing was recorded");
        assert!(!root2.join(".bee/logs/test-failure-finish-0.log").exists());
    }

    // ══ frd-1 — `--deviation "<one line>"` on cells cap/finish ══
    //
    // Worker deviations narrated in prose never reached `trace.deviations`,
    // so `bee knowledge promote`'s pattern-candidate mining (reads only
    // `trace.deviations` + failure signatures) always reported zero. A
    // repeated `--deviation` only keeps the LAST occurrence (`rsv::Flags`'s
    // own `insert`, mirroring `--did` on `capture add`) — the flag carries
    // one line per cap/finish call, same as every other value flag here.

    fn cap_flags_frd(id: &str, deviations: Vec<&str>, deviation: Option<&str>) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: Vec::new(),
            deviations: deviations.into_iter().map(|d| json!(d)).collect(),
            deviation: deviation.map(str::to_string),
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: None,
            inline_reason: None,
            report: Some(default_test_report_json()),
            sync_ack: None,
        }
    }

    #[test]
    fn deviation_flag_appends_a_line_to_trace_deviations() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "frd-1a", &cell("frd-1a", "claimed", "f", json!([])));

        // A `--deviations-file` line and the new `--deviation` line both
        // land in the same array, in that order — --deviation appends, it
        // never replaces.
        let flags = cap_flags_frd("frd-1a", vec!["from the file"], Some("  from the flag  "));
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["deviations"],
            json!(["from the file", "from the flag"]),
            "the flag's value is trimmed and appended after the file's own lines"
        );
    }

    #[test]
    fn deviation_flag_blank_or_whitespace_only_is_refused_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "frd-1b", &cell("frd-1b", "claimed", "f", json!([])));
        let before = std::fs::read_to_string(cell_file(root, "frd-1b")).unwrap();

        for bad in ["", "   ", "\t\n "] {
            let flags = cap_flags_frd("frd-1b", Vec::new(), Some(bad));
            let refusal = thrown(cap_cell_from_flags(root, &flags, false));
            assert!(
                refusal.contains("--deviation") && refusal.contains("non-empty"),
                "refusal must name the flag: {refusal}"
            );
        }
        let after = std::fs::read_to_string(cell_file(root, "frd-1b")).unwrap();
        assert_eq!(before, after, "nothing was written on refusal — the cell file is untouched");
        let after_norm = read_cell_norm(root, "frd-1b").ok().unwrap().unwrap();
        assert_eq!(
            after_norm.get("status"),
            Some(&json!("claimed")),
            "a refused --deviation caps nothing — the cell stays exactly as claimed"
        );
    }

    #[test]
    fn omitting_deviation_is_byte_identical_to_before_the_flag_existed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "frd-1c", &cell("frd-1c", "claimed", "f", json!([])));

        // No --deviation, no --deviations-file: trace.deviations is the
        // same empty array cap has always written.
        let flags = cap_flags_frd("frd-1c", Vec::new(), None);
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(capped["trace"]["deviations"], json!([]));
    }

    // ══ dol-1 — the report's own deviations join trace.deviations ══════════
    //
    // A worker records its deviations STRUCTURALLY, in the `--report`
    // Result form. That copy landed only on `trace.report`, which
    // `bee knowledge promote` never reads — it mines `trace.deviations`
    // alone — so the lesson stayed invisible unless the orchestrator
    // hand-copied the line into `--deviation`. hss-3 is the real loss:
    // three report deviations beside `trace.deviations: []`, and its
    // feature's promote proposal printed "None". `cap_cell_from_flags` now
    // merges the report's entries into the same list: a UNION, in the order
    // deviations-file → --deviation → report, deduped by exact string
    // equality, with `trace.report` left verbatim.

    fn cap_flags_dol(
        id: &str,
        deviations: Vec<Value>,
        deviation: Option<&str>,
        report: &str,
    ) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: Vec::new(),
            deviations,
            deviation: deviation.map(str::to_string),
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: None,
            inline_reason: None,
            report: Some(report.to_string()),
            sync_ack: None,
        }
    }

    /// The Result form a worker actually sends, with `deviations` swapped
    /// for the JSON array literal under test.
    fn dol_report(deviations: &str) -> String {
        format!(
            r#"{{"outcome":"o","commit":"c","files":[],"tests":"cargo test -p bee — green — fixture","deviations":{deviations}}}"#
        )
    }

    /// (e) — `trace.report` is the parsed report verbatim, its own
    /// `deviations` included. The merge copies, it never moves.
    fn assert_report_verbatim(capped: &Value, report: &str) {
        assert_eq!(
            capped["trace"]["report"],
            serde_json::from_str::<Value>(report).unwrap(),
            "trace.report stays the verbatim D8 object — dol-1 reads it, never rewrites it"
        );
    }

    #[test]
    fn report_deviations_reach_trace_deviations_with_no_deviation_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "dol-1a", &cell("dol-1a", "claimed", "f", json!([])));

        // The exact shape that lost hss-3's lesson: deviations in the
        // report, no --deviation, nothing in --deviations-file.
        let report = dol_report(
            r#"["a cell's declared file is a hypothesis","  serialized what the plan split  ",""," "]"#,
        );
        let flags = cap_flags_dol("dol-1a", Vec::new(), None, &report);
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["deviations"],
            json!([
                "a cell's declared file is a hypothesis",
                "serialized what the plan split",
            ]),
            "report entries land trimmed, in their own order; blank entries are dropped"
        );
        assert_report_verbatim(&capped, &report);
    }

    #[test]
    fn deviations_file_then_flag_then_report_keep_that_order() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "dol-1b", &cell("dol-1b", "claimed", "f", json!([])));

        // A --deviations-file may carry arbitrary JSON values, not only
        // strings: the non-string entry passes through untouched.
        let report = dol_report(r#"["from the report","and one more"]"#);
        let flags = cap_flags_dol(
            "dol-1b",
            vec![json!({"note": "structured"}), json!("from the file")],
            Some("  from the flag  "),
            &report,
        );
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["deviations"],
            json!([
                {"note": "structured"},
                "from the file",
                "from the flag",
                "from the report",
                "and one more",
            ]),
            "deviations-file entries first, then --deviation, then the report's own"
        );
        assert_report_verbatim(&capped, &report);
    }

    #[test]
    fn a_deviation_in_both_the_flag_and_the_report_appears_once() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "dol-1c", &cell("dol-1c", "claimed", "f", json!([])));

        // The orchestrator hand-copied the line anyway (the old habit), and
        // the deviations-file already held another of them.
        let report = dol_report(r#"["said twice","  said twice  ","from the file","fresh"]"#);
        let flags = cap_flags_dol(
            "dol-1c",
            vec![json!("from the file")],
            Some("said twice"),
            &report,
        );
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["deviations"],
            json!(["from the file", "said twice", "fresh"]),
            "exact string equality after trimming dedupes against every earlier source"
        );
        assert_report_verbatim(&capped, &report);
    }

    #[test]
    fn an_empty_report_deviations_array_changes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "dol-1d", &cell("dol-1d", "claimed", "f", json!([])));

        let report = dol_report("[]");
        let flags = cap_flags_dol(
            "dol-1d",
            vec![json!("from the file")],
            Some("from the flag"),
            &report,
        );
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["deviations"],
            json!(["from the file", "from the flag"]),
            "a report with no deviations leaves frd-1's own list exactly as it was"
        );
        assert_report_verbatim(&capped, &report);
    }

    /// dol-1: the two sources are SYMMETRIC. `--deviations-file` has always
    /// carried arbitrary JSON through verbatim, and mining reads an object
    /// entry fine (`knowledge::deviation_text`'s `{type, description}`
    /// arm — a live cell already holds that shape). Dropping the same
    /// object on the report side would keep this cell's own defect alive in
    /// one branch, so it passes through untouched rather than stringified,
    /// trimmed, or skipped.
    #[test]
    fn an_object_shaped_report_deviation_passes_through_verbatim() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "dol-1e", &cell("dol-1e", "claimed", "f", json!([])));

        let report = dol_report(
            r#"[{"type":"scope","description":"the declared file was a hypothesis"},"a plain line"]"#,
        );
        let flags = cap_flags_dol("dol-1e", Vec::new(), None, &report);
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["deviations"],
            json!([
                {"type": "scope", "description": "the declared file was a hypothesis"},
                "a plain line",
            ]),
            "the object arrives as an object — mining renders it, this path never does"
        );
        assert_report_verbatim(&capped, &report);
    }

    /// dol-1: an object and the string it renders to are ONE deviation —
    /// an orchestrator who hand-copied the rendered line into `--deviation`
    /// does not double it.
    ///
    /// This case does NOT prove the `deviation_text` dedup on its own, and
    /// it is written down here so nobody reads it as that proof: the
    /// pre-dedup code SKIPPED every non-string report entry, and a skip
    /// leaves behind exactly the array a dedup does. What it locks is a
    /// naive rewrite — pass-through with raw string equality would double
    /// the deviation into `["scope: …", {object}]`. The object-FIRST case
    /// below is the one that goes red without the dedup.
    #[test]
    fn an_object_and_its_rendered_string_are_one_deviation() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "dol-1f", &cell("dol-1f", "claimed", "f", json!([])));

        let report =
            dol_report(r#"[{"type":"scope","description":"the declared file was a hypothesis"}]"#);
        let flags = cap_flags_dol(
            "dol-1f",
            Vec::new(),
            Some("scope: the declared file was a hypothesis"),
            &report,
        );
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["deviations"],
            json!(["scope: the declared file was a hypothesis"]),
            "the flag's line came first and stands; the report's object is the same deviation"
        );
        assert_report_verbatim(&capped, &report);
    }

    /// dol-1: the OTHER direction, and the one that fails without the
    /// `deviation_text` dedup. The object is already in the list from
    /// `--deviations-file`, and the report repeats it as the rendered line:
    /// a skip cannot produce this array, only the dedup can. The object
    /// keeps its place AND its form — the string never replaces it, and it
    /// is never re-added beside it.
    #[test]
    fn an_object_from_the_file_absorbs_its_rendered_twin_in_the_report() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "dol-1g", &cell("dol-1g", "claimed", "f", json!([])));

        let report = dol_report(r#"["scope: the declared file was a hypothesis","fresh line"]"#);
        let flags = cap_flags_dol(
            "dol-1g",
            vec![json!({"type": "scope", "description": "the declared file was a hypothesis"})],
            None,
            &report,
        );
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["deviations"],
            json!([
                {"type": "scope", "description": "the declared file was a hypothesis"},
                "fresh line",
            ]),
            "the file's object stands as an object; its rendered twin is the same deviation"
        );
        assert_report_verbatim(&capped, &report);
    }

    // ══ wfl-1/D8 — `--report <json>` on cells cap/finish ═══════════════════
    //
    // The structured counterpart to the worker Result form
    // (packages/bee/prompts/worker-cell.md): --report validated against
    // exactly REPORT_KEYS before any write, then stored verbatim as
    // trace.report. D8 (docs/history/test-doctrine/CONTEXT.md): --report is
    // now REQUIRED on every cap path — the same "add a flag, prove the old
    // path unchanged" posture frd-1's own omitting_deviation test took above
    // no longer applies to --report itself (see
    // `omitting_report_is_refused_report_now_required` below).

    fn cap_flags_report(id: &str, report: Option<&str>) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: Vec::new(),
            deviations: Vec::new(),
            deviation: None,
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: None,
            inline_reason: None,
            report: report.map(str::to_string),
            sync_ack: None,
        }
    }

    // D8: the worker's own `tests` claim is a proof string
    // `<command> — <result> — <scope reason>`, written by the agent that
    // ran it — never the retired `boundary`/`undeclared` enum (decision
    // 13ce1858, test-cadence-boundary D1a).
    const VALID_REPORT: &str = r#"{"outcome":"did the thing","commit":"abc123","files":["a.rs"],"tests":"cargo test -p bee — green — touched close.rs","deviations":[]}"#;

    #[test]
    fn valid_report_is_validated_and_stored_on_trace() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r1", &cell("wfl-r1", "claimed", "f", json!([])));

        let flags = cap_flags_report("wfl-r1", Some(VALID_REPORT));
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["report"],
            json!({
                "outcome": "did the thing",
                "commit": "abc123",
                "files": ["a.rs"],
                "tests": "cargo test -p bee — green — touched close.rs",
                "deviations": [],
            })
        );
    }

    /// D8: a no-test-sentinel repo's proof string names its command segment
    /// `none`, with the reason segment naming the parity/docs proof used
    /// instead — migrated from the old `undeclared` sentinel this test used
    /// to pin.
    #[test]
    fn report_tests_key_accepts_none_command_for_a_no_test_sentinel_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r1u", &cell("wfl-r1u", "claimed", "f", json!([])));

        let report =
            r#"{"outcome":"did the thing","commit":"abc123","files":[],"tests":"none — green — regen parity check only","deviations":[]}"#;
        let flags = cap_flags_report("wfl-r1u", Some(report));
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["report"]["tests"],
            json!("none — green — regen parity check only")
        );
    }

    /// D8: split on the FIRST TWO ` — ` separators only, so the reason
    /// segment may itself carry the same separator without breaking the
    /// parse.
    #[test]
    fn report_tests_key_reason_segment_may_contain_the_separator() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r1s", &cell("wfl-r1s", "claimed", "f", json!([])));

        let report = r#"{"outcome":"o","commit":"c","files":[],"tests":"cargo test -p bee — green — touched close.rs — and finish_support.rs","deviations":[]}"#;
        let flags = cap_flags_report("wfl-r1s", Some(report));
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(
            capped["trace"]["report"]["tests"],
            json!("cargo test -p bee — green — touched close.rs — and finish_support.rs")
        );
    }

    #[test]
    fn malformed_report_json_is_refused_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r2", &cell("wfl-r2", "claimed", "f", json!([])));
        let before = std::fs::read_to_string(cell_file(root, "wfl-r2")).unwrap();

        let flags = cap_flags_report("wfl-r2", Some("{not json"));
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(
            refusal.contains("--report") && refusal.contains("not valid JSON"),
            "{refusal}"
        );
        let after = std::fs::read_to_string(cell_file(root, "wfl-r2")).unwrap();
        assert_eq!(before, after, "a malformed --report writes nothing");
        let after_norm = read_cell_norm(root, "wfl-r2").ok().unwrap().unwrap();
        assert_eq!(after_norm.get("status"), Some(&json!("claimed")));
    }

    #[test]
    fn report_with_an_unknown_key_is_refused_by_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r3", &cell("wfl-r3", "claimed", "f", json!([])));

        let bad = r#"{"outcome":"o","commit":"c","files":[],"tests":"green","deviations":[],"extra":"nope"}"#;
        let flags = cap_flags_report("wfl-r3", Some(bad));
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(
            refusal.contains("unknown key \"extra\""),
            "refusal must name the offending key: {refusal}"
        );
    }

    #[test]
    fn report_missing_a_required_key_is_refused_by_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r4", &cell("wfl-r4", "claimed", "f", json!([])));

        let bad = r#"{"outcome":"o","commit":"c","files":[],"tests":"green"}"#; // no "deviations"
        let flags = cap_flags_report("wfl-r4", Some(bad));
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(
            refusal.contains("missing required key \"deviations\""),
            "refusal must name the missing key: {refusal}"
        );
    }

    /// D8: the retired `boundary`/`undeclared` enum is refused by name with
    /// a remedy naming the proof-string form — a cold worker learns the new
    /// contract instead of guessing why the old value stopped working.
    #[test]
    fn report_tests_key_legacy_boundary_or_undeclared_is_refused_with_the_proof_string_remedy() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r5", &cell("wfl-r5", "claimed", "f", json!([])));

        for legacy in ["boundary", "undeclared"] {
            let bad = format!(
                r#"{{"outcome":"o","commit":"c","files":[],"tests":"{legacy}","deviations":[]}}"#
            );
            let flags = cap_flags_report("wfl-r5", Some(&bad));
            let refusal = thrown(cap_cell_from_flags(root, &flags, false));
            assert!(
                refusal.contains(&format!("no longer accepts \"{legacy}\"")),
                "{legacy}: {refusal}"
            );
            assert!(
                refusal.contains("<command> — <result> — <scope reason>"),
                "{legacy}: {refusal}"
            );
        }
    }

    #[test]
    fn report_tests_key_malformed_proof_string_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r5c", &cell("wfl-r5c", "claimed", "f", json!([])));

        let bad = r#"{"outcome":"o","commit":"c","files":[],"tests":"maybe","deviations":[]}"#;
        let flags = cap_flags_report("wfl-r5c", Some(bad));
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(
            refusal.contains("must be a proof string \"<command> — <result> — <scope reason>\""),
            "{refusal}"
        );
    }

    /// D8/D6's spirit: a result segment reading `red` refuses the cap
    /// outright — a red is fix-first, never a done.
    #[test]
    fn report_tests_key_red_result_segment_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r5b", &cell("wfl-r5b", "claimed", "f", json!([])));

        let bad = r#"{"outcome":"o","commit":"c","files":[],"tests":"cargo test -p bee — red — touched close.rs","deviations":[]}"#;
        let flags = cap_flags_report("wfl-r5b", Some(bad));
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(refusal.contains("\"red\""), "{refusal}");
        assert!(refusal.contains("fix-first"), "{refusal}");
    }

    /// D8: --report is now required on every cap path — an absent flag
    /// refuses instead of the old "leave trace.report untouched" byte-
    /// identical behavior.
    #[test]
    fn omitting_report_is_refused_report_now_required() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(root, "wfl-r6", &cell("wfl-r6", "claimed", "f", json!([])));

        let flags = cap_flags_report("wfl-r6", None);
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(
            refusal.contains("--report is required"),
            "{refusal}"
        );
        let after_norm = read_cell_norm(root, "wfl-r6").ok().unwrap().unwrap();
        assert_eq!(
            after_norm.get("status"),
            Some(&json!("claimed")),
            "a refused --report caps nothing"
        );
    }

    // ══ D6 — the cell commit trailer (docs/history/hook-teeth/CONTEXT.md) ══
    //
    // "`cells finish` verifies a commit whose trailer names the finishing
    // cell id exists on the feature's branch (the granted worktree's HEAD
    // history, else main's) when `files_changed` is non-empty;
    // `--commit-pending <reason>` escapes and is stored on the trace. A cell
    // with no file changes is exempt." D7 red-first: the pure trailer
    // detector is pinned FIRST, against a real fixture git repo, before the
    // refusal wiring that reads it.

    fn git_ok(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the D6 fixtures");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// A one-commit git repo — the base every D6 fixture builds on. Doubles
    /// as the cell STORE root in these tests: no worktree grant exists, so
    /// `commit_trailer_history_root` falls back to exactly this directory's
    /// own HEAD history, the same directory `write_cell_fixture` writes into.
    fn commit_history_repo(root: &Path) {
        std::fs::write(root.join("f.txt"), "x").unwrap();
        git_ok(root, &["init", "-q", "-b", "main", "."]);
        git_ok(root, &["config", "user.email", "a@b.c"]);
        git_ok(root, &["config", "user.name", "t"]);
        git_ok(root, &["add", "-A"]);
        git_ok(root, &["commit", "-qm", "init"]);
    }

    fn commit_with_message(root: &Path, file_content: &str, message: &str) {
        std::fs::write(root.join("f.txt"), file_content).unwrap();
        git_ok(root, &["commit", "-qam", message]);
    }

    #[test]
    fn commit_trailer_present_matches_an_exact_trailer_line_in_recent_history() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);

        // Only the init commit exists — no qualifying commit yet.
        assert!(!commit_trailer_present(root, "bh-6"));

        // A commit whose body MENTIONS the id in prose, but not as the exact
        // trailer line, still does not satisfy it.
        commit_with_message(root, "y", "touch up bh-6 handling");
        assert!(!commit_trailer_present(root, "bh-6"));

        // The real trailer, on its own line in the body.
        commit_with_message(root, "z", "Do the thing\n\ncell: bh-6");
        assert!(commit_trailer_present(root, "bh-6"));

        // A DIFFERENT cell id's trailer never matches.
        assert!(!commit_trailer_present(root, "bh-7"));
    }

    fn cap_flags_d6(id: &str, files: Vec<&str>, commit_pending: Option<&str>) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: files.into_iter().map(|f| json!(f)).collect(),
            deviations: Vec::new(),
            deviation: None,
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: commit_pending.map(str::to_string),
            inline_reason: None,
            report: Some(default_test_report_json()),
            sync_ack: None,
        }
    }

    fn cell_body_d6(id: &str) -> Value {
        json!({
            "id": id, "feature": "hook-teeth", "title": "t", "action": "a",
            "verify": "echo ok", "lane": "tiny", "status": "claimed",
            "deps": [], "files": [], "trace": {},
        })
    }

    #[test]
    fn finish_refuses_a_non_empty_files_cap_with_no_trailer_commit() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);
        write_cell_fixture(root, "bh-6a", &cell_body_d6("bh-6a"));

        let refusal = thrown(cap_cell_from_flags(
            root,
            &cap_flags_d6("bh-6a", vec!["a.rs"], None),
            true, // finish
        ));
        assert!(
            refusal.starts_with("capCell: cell \"bh-6a\" refused — one commit per cell"),
            "{refusal}"
        );
        let after = read_cell_norm(root, "bh-6a").ok().unwrap().unwrap();
        assert_eq!(after.get("status"), Some(&json!("claimed")), "a missing trailer never caps");
    }

    #[test]
    fn finish_caps_once_the_trailer_commit_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);
        write_cell_fixture(root, "bh-6b", &cell_body_d6("bh-6b"));
        commit_with_message(root, "y", "Wire the thing\n\ncell: bh-6b");

        let capped =
            cap_cell_from_flags(root, &cap_flags_d6("bh-6b", vec!["a.rs"], None), true).unwrap();
        assert_eq!(capped["status"], json!("capped"));
    }

    #[test]
    fn finish_commit_pending_escapes_and_is_recorded_on_the_trace() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root); // no trailer commit — the escape is exercised for real
        write_cell_fixture(root, "bh-6c", &cell_body_d6("bh-6c"));

        let capped = cap_cell_from_flags(
            root,
            &cap_flags_d6("bh-6c", vec!["a.rs"], Some("commit lands after cap, batching two")),
            true,
        )
        .unwrap();
        assert_eq!(capped["status"], json!("capped"));
        assert_eq!(
            capped["trace"]["commit_pending"],
            json!("commit lands after cap, batching two")
        );
    }

    #[test]
    fn finish_with_empty_files_changed_is_exempt_from_the_trailer_check() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Deliberately NOT a git repo at all — proves the check never even
        // shells out to git when files_changed is empty.
        write_cell_fixture(root, "bh-6d", &cell_body_d6("bh-6d"));

        let capped = cap_cell_from_flags(root, &cap_flags_d6("bh-6d", vec![], None), true).unwrap();
        assert_eq!(capped["status"], json!("capped"));
    }

    #[test]
    fn cap_without_finish_never_runs_the_trailer_check() {
        // D6 scopes the check to `cells finish`; plain `cells cap`
        // (finish == false) must cap a non-empty-files cell even with zero
        // commit history (not even a git repo here).
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "bh-6e", &cell_body_d6("bh-6e"));

        let capped =
            cap_cell_from_flags(root, &cap_flags_d6("bh-6e", vec!["a.rs"], None), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
    }

    // ══ D-WP-1 — a small+ cap names a REGISTERED execution worker ═════════
    //
    // AGENTS.md ("Work in parallel, coordinate through the store"): "From
    // `small` up, cells run through dispatched workers (never zero
    // execution workers)" — nothing at the cap door read the registry to
    // enforce that until this cell. `tiny` stays exempt (it may run inline
    // by contract, same posture as the D6 trailer check's own scoping).
    // `--inline-reason "<why>"` escapes the refusal for a named, recorded
    // deviation instead of a real dispatch.

    fn cap_flags_wp(id: &str, files: Vec<&str>, inline_reason: Option<&str>) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: files.into_iter().map(|f| json!(f)).collect(),
            deviations: Vec::new(),
            deviation: None,
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: None,
            inline_reason: inline_reason.map(str::to_string),
            report: Some(default_test_report_json()),
            sync_ack: None,
        }
    }

    fn cell_body_wp(id: &str, lane: &str, worker: Option<&str>) -> Value {
        json!({
            "id": id, "feature": "f", "title": "t", "action": "a",
            "verify": "echo ok", "lane": lane, "status": "claimed",
            "deps": [], "files": [],
            "trace": worker.map(|w| json!({"worker": w})).unwrap_or_else(|| json!({})),
        })
    }

    fn write_workers_state(root: &Path, workers: Value) {
        let dir = root.join(".bee");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            bstate::state_path(root),
            jsjson::stringify_pretty(&json!({"workers": workers})),
        )
        .unwrap();
    }

    #[test]
    fn small_plus_cap_refuses_with_no_registered_worker_for_the_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "wp-a", &cell_body_wp("wp-a", "standard", Some("kevin")));

        // No .bee/state.json at all — the emptiest possible "no registry".
        let refusal =
            thrown(cap_cell_from_flags(root, &cap_flags_wp("wp-a", vec!["a.rs"], None), false));
        assert!(
            refusal.starts_with("capCell: lane \"standard\" cell \"wp-a\" refused"),
            "{refusal}"
        );
        assert!(refusal.contains("wp-a"), "{refusal}");
        assert!(refusal.contains("kevin"), "{refusal}");
        assert!(refusal.contains("bee state worker add"), "{refusal}");
        assert!(refusal.contains("--inline-reason"), "{refusal}");
        let after = read_cell_norm(root, "wp-a").ok().unwrap().unwrap();
        assert_eq!(after.get("status"), Some(&json!("claimed")), "an unregistered worker never caps");
    }

    #[test]
    fn small_plus_cap_refuses_when_the_registered_worker_names_a_different_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "wp-e", &cell_body_wp("wp-e", "standard", Some("kevin")));
        write_workers_state(
            root,
            json!([{"nickname": "kevin", "cell": "some-other-cell", "tier": "generation", "status": "running"}]),
        );

        let refusal =
            thrown(cap_cell_from_flags(root, &cap_flags_wp("wp-e", vec!["a.rs"], None), false));
        assert!(refusal.contains("no registered execution worker"), "{refusal}");
    }

    #[test]
    fn small_plus_cap_succeeds_once_the_worker_is_registered_for_that_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "wp-b", &cell_body_wp("wp-b", "standard", Some("kevin")));
        write_workers_state(
            root,
            json!([{"nickname": "kevin", "cell": "wp-b", "tier": "generation", "status": "running"}]),
        );

        let capped =
            cap_cell_from_flags(root, &cap_flags_wp("wp-b", vec!["a.rs"], None), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
    }

    #[test]
    fn small_plus_cap_inline_reason_escapes_and_is_recorded_on_the_trace() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "wp-c", &cell_body_wp("wp-c", "standard", Some("kevin")));

        // No state.json at all — the escape must still cap.
        let capped = cap_cell_from_flags(
            root,
            &cap_flags_wp("wp-c", vec!["a.rs"], Some("solo session, no dispatch available")),
            false,
        )
        .unwrap();
        assert_eq!(capped["status"], json!("capped"));
        assert_eq!(
            capped["trace"]["inline_reason"],
            json!("solo session, no dispatch available")
        );
    }

    #[test]
    fn tiny_lane_cap_is_never_checked_for_a_registered_worker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "wp-d", &cell_body_wp("wp-d", "tiny", Some("kevin")));

        // No workers[] entry at all — a tiny cap must never even look.
        let capped =
            cap_cell_from_flags(root, &cap_flags_wp("wp-d", vec!["a.rs"], None), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
    }

    // ══ fa-1 — diff-vs-test advisory (finish's green-cap-only nudge) ══════
    //
    // `trace.warnings`' new (and only) producer since the E1 impact-registry
    // check retired (finish_support.rs, near the RETIRED comment): a commit
    // that changes more than `finish.advisory_untested_lines` lines and
    // touches no test-shaped path earns exactly one stderr line and one
    // `trace.warnings` entry — never a refusal, never a change to the cap's
    // own exit code or JSON success shape.

    #[test]
    fn path_looks_like_test_heuristic_table() {
        let cases: &[(&str, bool)] = &[
            ("packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", true),
            ("crates/bee/src/hooks/write_guard/tests.rs", true),
            ("test/fixtures/a.rs", true),
            ("tests/fixtures/a.rs", true),
            ("src/foo_test.rs", true),
            ("src/foo.test.ts", true),
            ("src/tests.rs", true),
            // never a false positive on a mere substring.
            ("src/contest/a.rs", false),
            ("src/latest.rs", false),
            ("src/testament.rs", false),
            ("src/main.rs", false),
            ("docs/README.md", false),
        ];
        for (path, expected) in cases {
            assert_eq!(path_looks_like_test(path), *expected, "path: {path}");
        }
    }

    #[test]
    fn advisory_untested_lines_threshold_reads_the_nested_config_key() {
        let default = advisory_untested_lines_threshold(&Map::new());
        assert_eq!(default, DEFAULT_ADVISORY_UNTESTED_LINES);

        let configured = advisory_untested_lines_threshold(
            json!({"finish": {"advisory_untested_lines": 42}}).as_object().unwrap(),
        );
        assert_eq!(configured, 42);

        // 0 is the documented disable, not a "run always" typo — the
        // config reader just passes it through; diff_vs_test_advisory is
        // the one that treats 0 specially.
        let zero = advisory_untested_lines_threshold(
            json!({"finish": {"advisory_untested_lines": 0}}).as_object().unwrap(),
        );
        assert_eq!(zero, 0);

        // A malformed value (string instead of number) falls back to the
        // default silently, matching resolve_write_policy_mode's own
        // posture for a single-scalar nested key.
        let malformed = advisory_untested_lines_threshold(
            json!({"finish": {"advisory_untested_lines": "lots"}}).as_object().unwrap(),
        );
        assert_eq!(malformed, DEFAULT_ADVISORY_UNTESTED_LINES);
    }

    /// Stages ONLY `file` (never `-A`, which would also sweep in a cell
    /// fixture some of these tests write into `root/.bee/cells/` — this
    /// repo doubles as the cell store) — the numstat total each test
    /// asserts on must describe exactly the one file it wrote.
    fn commit_lines(root: &Path, file: &str, lines: usize, message: &str) {
        let content = (0..lines).map(|i| format!("line {i}\n")).collect::<String>();
        std::fs::write(root.join(file), content).unwrap();
        git_ok(root, &["add", "--", file]);
        git_ok(root, &["commit", "-qm", message]);
    }

    #[test]
    fn diff_vs_test_advisory_missing_git_is_a_silent_skip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path(); // deliberately not a git repo at all
        assert_eq!(diff_vs_test_advisory(root, "fa-x", 150), None);
    }

    #[test]
    fn diff_vs_test_advisory_disabled_by_a_zero_threshold() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);
        commit_lines(root, "big.rs", 500, "Do a lot\n\ncell: fa-z");
        // Would fire at any positive threshold — 0 disables it outright.
        assert_eq!(diff_vs_test_advisory(root, "fa-z", 0), None);
    }

    #[test]
    fn diff_vs_test_advisory_threshold_boundary() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);

        // Exactly at the threshold — "exceeds" means strictly over, so
        // this must NOT fire.
        commit_lines(root, "at.rs", 10, "Small change\n\ncell: fa-b");
        assert_eq!(diff_vs_test_advisory(root, "fa-b", 10), None);
    }

    #[test]
    fn diff_vs_test_advisory_fires_over_threshold_with_no_test_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);
        commit_lines(root, "over.rs", 11, "Bigger change\n\ncell: fa-c");
        let line = diff_vs_test_advisory(root, "fa-c", 10).expect("must fire over threshold");
        assert!(line.contains("11 line"), "{line}");
        assert!(line.contains("10-line"), "{line}");
    }

    #[test]
    fn diff_vs_test_advisory_skips_when_a_changed_path_looks_like_a_test() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);
        std::fs::write(root.join("tests.rs"), "x").unwrap();
        git_ok(root, &["add", "--", "tests.rs"]);
        commit_lines(root, "over.rs", 50, "Big change with a test file too\n\ncell: fa-d");
        // over.rs alone exceeds the threshold, but tests.rs is present in
        // the SAME commit — the advisory must stay silent.
        assert_eq!(diff_vs_test_advisory(root, "fa-d", 10), None);
    }

    #[test]
    fn diff_vs_test_advisory_skips_when_head_does_not_carry_this_cells_trailer() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);
        commit_lines(root, "over.rs", 50, "No trailer here at all");
        assert_eq!(diff_vs_test_advisory(root, "fa-e", 10), None);
    }

    #[test]
    fn finish_caps_and_appends_one_advisory_line_to_trace_warnings_over_threshold() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);
        write_bee_config(root, &json!({"finish": {"advisory_untested_lines": 5}}));
        write_cell_fixture(root, "fa-f", &cell_body_d6("fa-f"));
        commit_lines(root, "big.rs", 20, "Wire the thing\n\ncell: fa-f");

        let capped =
            cap_cell_from_flags(root, &cap_flags_d6("fa-f", vec!["big.rs"], None), true).unwrap();
        assert_eq!(capped["status"], json!("capped"));
        let warnings = capped["trace"]["warnings"].as_array().unwrap();
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].as_str().unwrap().contains("20 line"), "{warnings:?}");
    }

    #[test]
    fn cap_without_finish_never_gains_the_advisory_even_over_threshold() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        commit_history_repo(root);
        write_bee_config(root, &json!({"finish": {"advisory_untested_lines": 5}}));
        write_cell_fixture(root, "fa-g", &cell_body_d6("fa-g"));
        // No trailer commit needed: `cells cap` never runs D6's own
        // trailer check OR this advisory (finish == false gates both).
        let capped =
            cap_cell_from_flags(root, &cap_flags_d6("fa-g", vec!["a.rs"], None), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
        assert_eq!(capped["trace"]["warnings"], json!([]));
    }

    // ══ adoption + fencing (claims.mjs, msn-12 D4/D9 invariant 10) ═════════
    //
    // Ported from test_claims.mjs. Before this cell nothing in the Rust tree
    // consumed `fence_epoch`: claim_cell_file stamped it and no code path ever
    // compared it, so a stale holder's renew/release would have proceeded
    // silently. Each negative below CONSTRUCTS the stale state and pins the
    // exact refusal bytes, with a firing control beside it.

    fn adopt(root: &Path, cell: &str, session: &str) -> AdoptClaimOutcome {
        adopt_claim(root, cell, session).expect("adopt must not throw")
    }

    fn refused(outcome: AdoptClaimOutcome) -> ClaimRefusal {
        match outcome {
            AdoptClaimOutcome::Refused(r) => r,
            AdoptClaimOutcome::Ok { .. } => panic!("expected a typed refusal"),
        }
    }

    fn claim_on_disk(root: &Path, cell: &str) -> Value {
        let raw = std::fs::read_to_string(claims_dir(root).join(format!("{cell}.json"))).unwrap();
        serde_json::from_str(&raw).unwrap()
    }

    /// Oracle: "adoptClaim rewrites the owner IN PLACE: old owner loses, new
    /// owner holds, the claim file is present throughout" and "adoptClaim
    /// bumps fence_epoch by exactly 1, atomically with the ownership rewrite".
    #[test]
    fn adopt_rewrites_ownership_in_place_and_bumps_the_fence_by_exactly_one() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "c-1", Some("sess-a"), 600.0, OLD);
        // A pre-msn-12 claim carries no fence_epoch on disk at all.
        assert!(claim_on_disk(root, "c-1").get("fence_epoch").is_none());

        let AdoptClaimOutcome::Ok { claim, previous_owner } = adopt(root, "c-1", "sess-b") else {
            panic!("expected an adoption");
        };
        assert_eq!(previous_owner, Some(json!("sess-a")));
        assert_eq!(claim["session"], json!("sess-b"));
        assert_eq!(claim["adopted_from"], json!("sess-a"));
        assert_eq!(claim["fence_epoch"], json!(2.0), "a legacy claim reads as epoch 1, so +1 == 2");
        assert_ne!(claim["claimed_at"], json!(OLD), "fresh ownership renews the TTL clock");
        assert_eq!(claim["acquired_at"], json!(OLD), "the acquisition stamp is immutable");
        assert_eq!(claim["adopted_at"], claim["claimed_at"]);
        // Compared as RENDERED bytes: JS writes 2 and 2.0 identically, so a
        // JSON number-kind difference on the read-back is not a difference.
        assert_eq!(
            jsjson::stringify(&claim_on_disk(root, "c-1")),
            jsjson::stringify(&claim),
            "written atomically, never deleted first"
        );
        // ORACLE-PINNED BYTES: captured from a live `node` run of claims.mjs
        // adoptClaim over this exact fixture, not from a reading of the source.
        let on_disk = std::fs::read_to_string(claims_dir(root).join("c-1.json"))
            .unwrap()
            .replace(claim["claimed_at"].as_str().unwrap(), "<now>");
        assert_eq!(
            on_disk,
            "{\n  \"cell\": \"c-1\",\n  \"session\": \"sess-b\",\n  \"ttl_seconds\": 600,\n  \"claimed_at\": \"<now>\",\n  \"acquired_at\": \"2020-01-01T00:00:00.000Z\",\n  \"adopted_from\": \"sess-a\",\n  \"adopted_at\": \"<now>\",\n  \"fence_epoch\": 2\n}\n"
        );
        // Key order: a re-assigned key keeps its position; the three new ones
        // append in declaration order.
        let keys: Vec<&str> =
            claim.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec!["cell", "session", "ttl_seconds", "claimed_at", "acquired_at", "adopted_from", "adopted_at", "fence_epoch"]
        );

        // A second adoption bumps again, from the STORED epoch.
        let AdoptClaimOutcome::Ok { claim, previous_owner } = adopt(root, "c-1", "sess-c") else {
            panic!("expected an adoption");
        };
        assert_eq!(previous_owner, Some(json!("sess-b")));
        assert_eq!(claim["fence_epoch"], json!(3.0));

        // Adopting a SESSIONLESS claim drops `adopted_from` entirely rather
        // than writing null (`{...claim, adopted_from: undefined}`).
        write_claim_fixture(root, "c-2", None, 600.0, OLD);
        let AdoptClaimOutcome::Ok { claim, previous_owner } = adopt(root, "c-2", "sess-b") else {
            panic!("expected an adoption");
        };
        assert_eq!(previous_owner, None);
        assert!(claim.get("adopted_from").is_none(), "undefined is dropped, never null: {claim}");
        assert!(!claim_on_disk(root, "c-2").as_object().unwrap().contains_key("adopted_from"));

        // The gate file never leaks.
        assert!(!claim_gate_path(root, "c-1").unwrap().exists());
        assert!(!claim_gate_path(root, "c-2").unwrap().exists());
    }

    /// Oracle: "adoptClaim on a cell with no claim is a typed NOT_FOUND" and
    /// "sweep and adopt skip/refuse while the per-claim gate is held — typed
    /// GATE_HELD, never wait".
    #[test]
    fn adopt_refuses_not_found_and_gate_held_without_ever_waiting() {
        let tmp = cn_root();
        let root = tmp.path();
        let ghost = refused(adopt(root, "no-such-cell", "sess-b"));
        assert_eq!(ghost.code, "NOT_FOUND");
        assert_eq!(ghost.reason, "cell \"no-such-cell\" has no claim to adopt.");

        write_claim_fixture(root, "gated", Some("sess-a"), 600.0, OLD);
        let gate = claim_gate_path(root, "gated").unwrap();
        std::fs::write(&gate, "{}").unwrap(); // another process mid-adopt
        let before = claim_on_disk(root, "gated");
        let held = refused(adopt(root, "gated", "sess-b"));
        assert_eq!(held.code, "GATE_HELD");
        assert_eq!(
            held.reason,
            "claim \"gated\" is gated by another in-flight adopt/sweep — retry later, never wait on the gate."
        );
        assert_eq!(claim_on_disk(root, "gated"), before, "a gated adopt changes nothing");
        assert!(gate.exists(), "someone else's gate is never released by the loser");

        // Control: with the gate free the very same adopt succeeds.
        std::fs::remove_file(&gate).unwrap();
        assert!(matches!(adopt(root, "gated", "sess-b"), AdoptClaimOutcome::Ok { .. }));

        // requireId still guards both arguments.
        assert!(matches!(adopt_claim(root, "  ", "s"), Err(Fail::Thrown(m)) if m == "cell id is required."));
        assert!(matches!(adopt_claim(root, "c", " "), Err(Fail::Thrown(m)) if m == "session id is required."));
    }

    /// The second-port pin named in the fencing section header: this module's
    /// adoptClaim must leave the SAME bytes on disk as verbs/state_group.rs's
    /// narrowed twin (the `state handoff adopt` path). Re-derived rather than
    /// imported because that file is outside this cell's touchable set.
    #[test]
    fn adopt_agrees_with_the_state_group_port_on_the_shared_fixture() {
        let mine = cn_root();
        let theirs = cn_root();
        for root in [mine.path(), theirs.path()] {
            write_claim_fixture(root, "shared", Some("sess-a"), 600.0, OLD);
        }
        let AdoptClaimOutcome::Ok { .. } = adopt(mine.path(), "shared", "sess-b") else {
            panic!("expected an adoption");
        };
        let other = crate::verbs::state_group::adopt_claim(theirs.path(), "shared", "sess-b")
            .unwrap_or_else(|_| panic!("the state_group twin must also adopt"));
        let crate::verbs::state_group::AdoptOutcome::Adopted { claim, previous_owner } = other else {
            panic!("expected an adoption from the twin");
        };
        assert_eq!(previous_owner, Some(json!("sess-a")));
        let mut a = claim_on_disk(mine.path(), "shared");
        let mut b = Value::Object(claim);
        // The two ports stamp their own `now`; every other byte must agree.
        for v in [&mut a, &mut b] {
            let m = v.as_object_mut().unwrap();
            m.insert("claimed_at".into(), json!("<now>"));
            m.insert("adopted_at".into(), json!("<now>"));
        }
        assert_eq!(jsjson::stringify(&a), jsjson::stringify(&b));
        assert_eq!(a["fence_epoch"].as_f64(), Some(2.0));
    }

    /// Oracle: "renewClaimTTL refreshes claimed_at for this session's claims
    /// only, never touching adopted_from/adopted_at or fence_epoch", and "a
    /// claim whose gate is held is SKIPPED, never waited on".
    #[test]
    fn renew_touches_only_this_sessions_claims_and_never_the_fence() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "mine", Some("sess-a"), 600.0, OLD);
        write_claim_fixture(root, "theirs", Some("sess-b"), 600.0, OLD);
        write_claim_fixture(root, "nobodys", None, 600.0, OLD);
        write_claim_fixture(root, "gated", Some("sess-a"), 600.0, OLD);
        std::fs::write(claim_gate_path(root, "gated").unwrap(), "{}").unwrap();
        // Give `mine` a fence so "renewal never bumps it" is not vacuous.
        let mut with_fence = claim_on_disk(root, "mine");
        with_fence["fence_epoch"] = json!(4);
        std::fs::write(
            claims_dir(root).join("mine.json"),
            jsjson::stringify_pretty(&with_fence),
        )
        .unwrap();

        let RenewClaimOutcome::Ok { renewed, skipped } =
            renew_claim_ttl(root, "sess-a", None).unwrap()
        else {
            panic!("expected a renewal");
        };
        assert_eq!(renewed, vec!["mine".to_string()]);
        assert_eq!(skipped, vec!["gated".to_string()], "a held gate is skipped, never waited on");

        let renewed_claim = claim_on_disk(root, "mine");
        assert_ne!(renewed_claim["claimed_at"], json!(OLD), "the expiry clock advanced");
        assert_eq!(renewed_claim["acquired_at"], json!(OLD), "acquired_at never moves");
        assert_eq!(renewed_claim["fence_epoch"], json!(4), "renewal never bumps the fence");
        assert_eq!(claim_on_disk(root, "theirs")["claimed_at"], json!(OLD));
        assert_eq!(claim_on_disk(root, "nobodys")["claimed_at"], json!(OLD));
        assert_eq!(claim_on_disk(root, "gated")["claimed_at"], json!(OLD));
        assert!(!claim_gate_path(root, "mine").unwrap().exists(), "no gate leak");

        // An absent claims directory is an empty, non-throwing answer.
        let empty = cn_root();
        let RenewClaimOutcome::Ok { renewed, skipped } =
            renew_claim_ttl(empty.path(), "sess-a", None).unwrap()
        else {
            panic!("expected the empty answer");
        };
        assert!(renewed.is_empty() && skipped.is_empty());
    }

    /// Oracle: "renewClaimTTL refuses typed CLAIM_FENCE_STALE when the
    /// presented epoch is behind the claim's current fence_epoch, and renews
    /// NOTHING". NEGATIVE test: the stale state is constructed and the
    /// refusal bytes are pinned exactly.
    #[test]
    fn a_stale_presented_epoch_refuses_the_renew_and_writes_nothing() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "c-1", Some("sess-a"), 600.0, OLD);
        // A takeover already moved ownership forward: stored epoch is 3.
        let mut bumped = claim_on_disk(root, "c-1");
        bumped["fence_epoch"] = json!(3);
        std::fs::write(claims_dir(root).join("c-1.json"), jsjson::stringify_pretty(&bumped))
            .unwrap();
        let before = std::fs::read_to_string(claims_dir(root).join("c-1.json")).unwrap();

        for (presented, rendered) in
            [(json!(2), "2"), (json!(0), "0"), (json!(-1), "-1"), (json!(null), "null")]
        {
            let RenewClaimOutcome::Refused(r) =
                renew_claim_ttl(root, "sess-a", Some(&presented)).unwrap()
            else {
                panic!("{presented} must be refused");
            };
            assert_eq!(r.code, "CLAIM_FENCE_STALE");
            assert_eq!(
                r.reason,
                format!(
                    "cell \"c-1\" renew refused: presented epoch {rendered} is behind current fence_epoch 3 — a takeover already moved ownership forward; re-adopt before writing again."
                )
            );
            assert_eq!(r.extra["cell"], json!("c-1"));
            assert_eq!(r.extra["current_epoch"], json!(3.0));
            assert_eq!(
                std::fs::read_to_string(claims_dir(root).join("c-1.json")).unwrap(),
                before,
                "a fenced refusal renews nothing at all"
            );
            assert!(!claim_gate_path(root, "c-1").unwrap().exists(), "the gate is released in finally");
        }

        // Controls: the CURRENT epoch and an AHEAD epoch both renew, and
        // omitting the presentation is the legacy unfenced arm.
        for fresh in [json!(3), json!(4)] {
            let RenewClaimOutcome::Ok { renewed, .. } =
                renew_claim_ttl(root, "sess-a", Some(&fresh)).unwrap()
            else {
                panic!("presenting {fresh} must renew");
            };
            assert_eq!(renewed, vec!["c-1".to_string()]);
        }
        assert!(matches!(
            renew_claim_ttl(root, "sess-a", None).unwrap(),
            RenewClaimOutcome::Ok { .. }
        ));

        // A legacy claim with NO fence_epoch reads as 1: presenting 0 is
        // stale, presenting 1 renews.
        write_claim_fixture(root, "legacy", Some("sess-b"), 600.0, OLD);
        let RenewClaimOutcome::Refused(r) =
            renew_claim_ttl(root, "sess-b", Some(&json!(0))).unwrap()
        else {
            panic!("a legacy claim must fence at 1");
        };
        assert!(r.reason.contains("behind current fence_epoch 1"), "{}", r.reason);
        assert!(matches!(
            renew_claim_ttl(root, "sess-b", Some(&json!(1))).unwrap(),
            RenewClaimOutcome::Ok { .. }
        ));
    }

    /// Oracle: "releaseClaim refuses typed CLAIM_FENCE_STALE on a stale
    /// presentation and the claim file is left untouched" — the
    /// safety-critical half. Also pins the refusal ORDER: ownership is
    /// checked BEFORE fencing (fencing is orthogonal, never a substitute).
    #[test]
    fn a_stale_presented_epoch_refuses_the_release_and_never_removes_the_file() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "c-1", Some("sess-a"), 600.0, OLD);
        let mut bumped = claim_on_disk(root, "c-1");
        bumped["fence_epoch"] = json!(3);
        std::fs::write(claims_dir(root).join("c-1.json"), jsjson::stringify_pretty(&bumped))
            .unwrap();
        let file = claims_dir(root).join("c-1.json");
        let before = std::fs::read_to_string(&file).unwrap();

        let stale = json!(2);
        let ReleaseClaimOutcome::Refused(r) =
            release_claim_typed(root, Some("sess-a"), "c-1", Some(&stale)).unwrap()
        else {
            panic!("a stale release must refuse");
        };
        assert_eq!(r.code, "CLAIM_FENCE_STALE");
        assert_eq!(
            r.reason,
            "cell \"c-1\" release refused: presented epoch 2 is behind current fence_epoch 3 — a takeover already moved ownership forward; re-adopt before writing again."
        );
        assert!(file.exists(), "a fenced release must NEVER remove the claim file");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
        assert!(!claim_gate_path(root, "c-1").unwrap().exists());

        // Refusal ORDER: a foreign session presenting a FRESH epoch still gets
        // NOT_OWNER, not a fence answer.
        let ReleaseClaimOutcome::Refused(r) =
            release_claim_typed(root, Some("sess-x"), "c-1", Some(&json!(99))).unwrap()
        else {
            panic!("a foreign release must refuse");
        };
        assert_eq!(r.code, "NOT_OWNER");
        assert_eq!(r.reason, "cell \"c-1\" is owned by session \"sess-a\", not \"sess-x\".");
        assert!(file.exists());

        // Control: the owner presenting the current epoch releases for real.
        let ReleaseClaimOutcome::Ok { released } =
            release_claim_typed(root, Some("sess-a"), "c-1", Some(&json!(3))).unwrap()
        else {
            panic!("the owner must be able to release");
        };
        assert_eq!(released["cell"], json!("c-1"));
        assert!(!file.exists());
        // …and the NOT_FOUND rung is unchanged.
        let ReleaseClaimOutcome::Refused(r) =
            release_claim_typed(root, Some("sess-a"), "c-1", None).unwrap()
        else {
            panic!("a released claim is NOT_FOUND");
        };
        assert_eq!(r.code, "NOT_FOUND");
        assert_eq!(r.reason, "cell \"c-1\" has no claim to release.");
    }

    /// The whole point of the fence, end to end: an adoption moves ownership
    /// forward, and the STALE holder's later renew AND release are both
    /// refused with the epoch it no longer has — never silently applied.
    #[test]
    fn an_adoption_fences_out_the_previous_holders_later_writes() {
        let tmp = cn_root();
        let root = tmp.path();
        write_claim_fixture(root, "c-1", Some("sess-a"), 600.0, OLD);
        // sess-a's in-memory copy says epoch 1 (a fresh claimCellFile stamp).
        let held_epoch = json!(1);
        // A takeover happens behind its back.
        let AdoptClaimOutcome::Ok { claim, .. } = adopt(root, "c-1", "sess-b") else {
            panic!("expected an adoption");
        };
        assert_eq!(claim["fence_epoch"], json!(2.0));

        // The stale holder's renew is refused — and it is no longer the owner
        // either, so nothing is renewed on any path.
        let RenewClaimOutcome::Ok { renewed, .. } =
            renew_claim_ttl(root, "sess-a", Some(&held_epoch)).unwrap()
        else {
            panic!("session ownership alone already excludes sess-a");
        };
        assert!(renewed.is_empty());

        // The edge case session identity alone would MISS: the same session
        // re-adopts (so it owns the claim again) while a stale in-memory copy
        // still presents the pre-adoption epoch.
        let AdoptClaimOutcome::Ok { .. } = adopt(root, "c-1", "sess-a") else {
            panic!("expected a re-adoption");
        };
        assert_eq!(claim_on_disk(root, "c-1")["fence_epoch"].as_f64(), Some(3.0));
        let RenewClaimOutcome::Refused(r) =
            renew_claim_ttl(root, "sess-a", Some(&held_epoch)).unwrap()
        else {
            panic!("a stale epoch from the CURRENT owner must still fence");
        };
        assert_eq!(r.code, "CLAIM_FENCE_STALE");
        let ReleaseClaimOutcome::Refused(r) =
            release_claim_typed(root, Some("sess-a"), "c-1", Some(&held_epoch)).unwrap()
        else {
            panic!("a stale epoch must fence the release too");
        };
        assert_eq!(r.code, "CLAIM_FENCE_STALE");
        assert!(claims_dir(root).join("c-1.json").exists(), "still there — a stale fence never proceeds");
    }

    // ══ cells escalate — escalation-share budget (D3, decision 0012) ═══════
    //
    // D6 sequencing: these prove the share computation first (exactly-40
    // allowed, just-over refused) and the refusal/override contract, before
    // trusting the flip in `set_escalation` to refuse anything for real.
    //
    // RETARGETED, NEVER WEAKENED (model-role-split D4, store `97ce5225`).
    // These probes were taken against `set_tier` while `bee cells tier` was
    // the door; D4 retired the tier selector and the verb, and the escalation
    // half kept its own name as `bee cells escalate` / `set_escalation`. The
    // call sites move and the tier-field assertions become flag assertions —
    // the FIELD they read ceased to exist by decision, and D4 is that
    // decision — but every arithmetic, refusal, override and scope assertion
    // is the one that was here before, unchanged.

    fn tiered_cell(id: &str, feature: &str, tier: Option<&str>) -> Value {
        let mut body = cell(id, "open", feature, json!([]));
        if let Some(t) = tier {
            body["tier"] = json!(t);
        }
        body
    }

    /// Others: 1 ceiling + 3 non-ceiling (4 tiered). Assigning "ceiling" to
    /// the target makes 2/5 — exactly the 40% budget — which D3 allows.
    #[test]
    fn ceiling_share_of_exactly_40_percent_is_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "o-1", &tiered_cell("o-1", "f", Some("ceiling")));
        write_cell_fixture(root, "o-2", &tiered_cell("o-2", "f", Some("extraction")));
        write_cell_fixture(root, "o-3", &tiered_cell("o-3", "f", Some("generation")));
        write_cell_fixture(root, "o-4", &tiered_cell("o-4", "f", Some("extraction")));
        write_cell_fixture(root, "target", &tiered_cell("target", "f", None));

        let cell =
            set_escalation(root, "target", true, None).expect("exactly 40% must be allowed");
        assert_eq!(cell[ESCALATE_FIELD], json!(true));
        let after = read_cell_norm(root, "target").ok().unwrap().unwrap();
        assert_eq!(after[ESCALATE_FIELD], json!(true), "the write actually landed");
    }

    /// Others: 2 ceiling + 4 non-ceiling (6 tiered). Assigning "ceiling" to
    /// the target makes 3/7 (~43%) — strictly over the 40% budget — which
    /// D3 refuses without an override. The refusal names both the computed
    /// share and the threshold (message-contract precedent: router.rs).
    #[test]
    fn ceiling_share_just_over_40_percent_refuses_naming_share_and_threshold() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "o-1", &tiered_cell("o-1", "f", Some("ceiling")));
        write_cell_fixture(root, "o-2", &tiered_cell("o-2", "f", Some("ceiling")));
        write_cell_fixture(root, "o-3", &tiered_cell("o-3", "f", Some("extraction")));
        write_cell_fixture(root, "o-4", &tiered_cell("o-4", "f", Some("extraction")));
        write_cell_fixture(root, "o-5", &tiered_cell("o-5", "f", Some("generation")));
        write_cell_fixture(root, "o-6", &tiered_cell("o-6", "f", Some("generation")));
        write_cell_fixture(root, "target", &tiered_cell("target", "f", None));

        let refusal = thrown(set_escalation(root, "target", true, None));
        assert!(
            refusal.starts_with("escalateCell: cell \"target\" refused"),
            "{refusal}"
        );
        assert!(refusal.contains("3/7"), "{refusal}");
        assert!(refusal.contains("43%"), "names the computed share: {refusal}");
        assert!(refusal.contains("40%"), "names the threshold: {refusal}");
        // Refused — the marking on disk never moved.
        let after = read_cell_norm(root, "target").ok().unwrap().unwrap();
        assert!(after.get(ESCALATE_FIELD).is_none(), "a refused assignment writes nothing");
    }

    /// The same over-budget shape as above, but with `--reason` supplied:
    /// the override succeeds and the reason persists on the cell's trace as
    /// `escalation_reason` (D4 renamed the key from `tier_reason` with the
    /// selector it was named for).
    #[test]
    fn reason_override_bypasses_the_refusal_and_persists_on_the_tier_record() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "o-1", &tiered_cell("o-1", "f", Some("ceiling")));
        write_cell_fixture(root, "o-2", &tiered_cell("o-2", "f", Some("ceiling")));
        write_cell_fixture(root, "o-3", &tiered_cell("o-3", "f", Some("extraction")));
        write_cell_fixture(root, "o-4", &tiered_cell("o-4", "f", Some("extraction")));
        write_cell_fixture(root, "o-5", &tiered_cell("o-5", "f", Some("generation")));
        write_cell_fixture(root, "o-6", &tiered_cell("o-6", "f", Some("generation")));
        write_cell_fixture(root, "target", &tiered_cell("target", "f", None));

        let cell =
            set_escalation(root, "target", true, Some("owner-approved rescue ladder bump"))
                .expect("a named reason overrides the refusal");
        assert_eq!(cell[ESCALATE_FIELD], json!(true));
        assert_eq!(
            cell["trace"][ESCALATION_REASON_KEY],
            json!("owner-approved rescue ladder bump")
        );
        assert!(
            cell["trace"].get(LEGACY_ESCALATION_REASON_KEY).is_none(),
            "the retired key is never written again"
        );

        let after = read_cell_norm(root, "target").ok().unwrap().unwrap();
        assert_eq!(
            after[ESCALATE_FIELD],
            json!(true),
            "the override write actually landed"
        );
        assert_eq!(
            after["trace"][ESCALATION_REASON_KEY],
            json!("owner-approved rescue ladder bump")
        );
    }

    /// A whitespace-only reason is not an override — D1/D3's "non-blank"
    /// convention (mirrors the `--reason` required-flag blank check on
    /// block/drop) — so the over-budget assignment still refuses.
    #[test]
    fn a_blank_reason_does_not_override_the_refusal() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "o-1", &tiered_cell("o-1", "f", Some("ceiling")));
        write_cell_fixture(root, "o-2", &tiered_cell("o-2", "f", Some("ceiling")));
        write_cell_fixture(root, "o-3", &tiered_cell("o-3", "f", Some("extraction")));
        write_cell_fixture(root, "o-4", &tiered_cell("o-4", "f", Some("extraction")));
        write_cell_fixture(root, "o-5", &tiered_cell("o-5", "f", Some("generation")));
        write_cell_fixture(root, "o-6", &tiered_cell("o-6", "f", Some("generation")));
        write_cell_fixture(root, "target", &tiered_cell("target", "f", None));

        let refusal = thrown(set_escalation(root, "target", true, Some("   ")));
        assert!(refusal.starts_with("escalateCell: cell \"target\" refused"), "{refusal}");
    }

    /// Taking the flag OFF is never budget-checked, however skewed the
    /// escalated share already is.
    #[test]
    fn disarming_the_flag_never_checks_the_escalation_budget() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "o-1", &tiered_cell("o-1", "f", Some("ceiling")));
        write_cell_fixture(root, "o-2", &tiered_cell("o-2", "f", Some("ceiling")));
        write_cell_fixture(root, "o-3", &tiered_cell("o-3", "f", Some("ceiling")));
        write_cell_fixture(root, "target", &tiered_cell("target", "f", None));

        let cell = set_escalation(root, "target", false, None)
            .expect("disarming is never budget-checked");
        assert!(cell.get(ESCALATE_FIELD).is_none());
    }

    // ══ D5 (store 97ce5225) — the ration, rehomed onto the escalation flag ══
    //
    // The five probes above keep their exact arithmetic and are the "unchanged
    // in force" half. These are the moved half: the flag is what the ration
    // now counts, the flag is what `bee cells escalate` writes, and no
    // store — migrated or not — can read as "nothing is marked".

    fn escalated_cell(id: &str, feature: &str) -> Value {
        let mut body = cell(id, "open", feature, json!([]));
        body["role"] = json!("code");
        body["escalate"] = json!(true);
        body
    }

    /// The same shape as `ceiling_share_just_over_40_percent_refuses…`, with
    /// every escalation spelled as the FLAG on a post-D7 cell (a role, no
    /// tier at all). 3/7 is still over budget and still refuses, which is
    /// what "fires on the flag exactly as it fired on the tier value" means.
    #[test]
    fn the_ration_refuses_on_the_escalation_flag_exactly_as_it_did_on_the_tier() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_cell_fixture(root, "o-1", &escalated_cell("o-1", "f"));
        write_cell_fixture(root, "o-2", &escalated_cell("o-2", "f"));
        for id in ["o-3", "o-4", "o-5", "o-6"] {
            let mut body = cell(id, "open", "f", json!([]));
            body["role"] = json!("code");
            write_cell_fixture(root, id, &body);
        }
        let mut target = cell("target", "open", "f", json!([]));
        target["role"] = json!("code");
        write_cell_fixture(root, "target", &target);

        let refusal = thrown(set_escalation(root, "target", true, None));
        assert!(refusal.starts_with("escalateCell: cell \"target\" refused"), "{refusal}");
        assert!(refusal.contains("3/7"), "{refusal}");
        assert!(refusal.contains("43%"), "names the computed share: {refusal}");
        assert!(refusal.contains("40%"), "names the threshold: {refusal}");
        let after = read_cell_norm(root, "target").ok().unwrap().unwrap();
        assert!(after.get("escalate").is_none(), "a refused escalation writes nothing");

        // The reason override still overrides, and now persists under the
        // key D4 renamed it to.
        let ok = set_escalation(root, "target", true, Some("owner-approved rescue ladder"))
            .expect("a named reason overrides the refusal");
        assert_eq!(ok["escalate"], json!(true));
        assert_eq!(
            ok["trace"][ESCALATION_REASON_KEY],
            json!("owner-approved rescue ladder")
        );
    }

    /// The zero-share window D5 forbids by name. A store still carrying the
    /// LEGACY spelling — `tier: "ceiling"`, which every record written before
    /// this change carries and which `bee cells backfill-roles` converts when
    /// an operator runs it — must charge the ration exactly the same. If the
    /// ration read the flag alone, this store would compute 0.0 and the
    /// refusal could never fire.
    #[test]
    fn an_unmigrated_store_still_charges_the_ration() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Legacy only: nothing here carries the flag.
        write_cell_fixture(root, "o-1", &tiered_cell("o-1", "f", Some("ceiling")));
        write_cell_fixture(root, "o-2", &tiered_cell("o-2", "f", Some("ceiling")));
        for id in ["o-3", "o-4", "o-5", "o-6"] {
            write_cell_fixture(root, id, &tiered_cell(id, "f", Some("generation")));
        }
        write_cell_fixture(root, "target", &tiered_cell("target", "f", None));
        let refusal = thrown(set_escalation(root, "target", true, None));
        assert!(refusal.contains("3/7"), "the legacy spelling counts: {refusal}");

        // And a HALF-migrated store — one record converted, one not — counts
        // each cell exactly once rather than twice or not at all.
        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        write_cell_fixture(root2, "o-1", &tiered_cell("o-1", "f", Some("ceiling")));
        write_cell_fixture(root2, "o-2", &escalated_cell("o-2", "f"));
        for id in ["o-3", "o-4", "o-5", "o-6"] {
            write_cell_fixture(root2, id, &tiered_cell(id, "f", Some("generation")));
        }
        write_cell_fixture(root2, "target", &tiered_cell("target", "f", None));
        let refusal2 = thrown(set_escalation(root2, "target", true, None));
        assert!(refusal2.contains("3/7"), "one spelling each, counted once: {refusal2}");
    }

    /// The write half: escalating marks the flag, and `--off` disarms it —
    /// `bee cells tier --tier generation` took a cell off the session model
    /// before D5, and `bee cells escalate --off` still does.
    #[test]
    fn escalating_marks_the_flag_and_off_disarms_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Four cells, one of them escalated: 25%, comfortably under budget.
        for id in ["o-1", "o-2", "o-3"] {
            write_cell_fixture(root, id, &tiered_cell(id, "f", None));
        }
        write_cell_fixture(root, "target", &tiered_cell("target", "f", None));

        let up = set_escalation(root, "target", true, None).expect("well under budget");
        assert_eq!(up["escalate"], json!(true));
        assert!(
            up.get("tier").is_none(),
            "D4: the retired selector is never written back onto a cell"
        );
        let on_disk = read_cell_norm(root, "target").ok().unwrap().unwrap();
        assert_eq!(on_disk["escalate"], json!(true), "the flag actually landed");

        let down = set_escalation(root, "target", false, None).expect("never budget-checked");
        assert!(
            down.get("escalate").is_none(),
            "a non-escalating assignment removes the flag rather than writing false"
        );
        let after = read_cell_norm(root, "target").ok().unwrap().unwrap();
        assert!(after.get("escalate").is_none());
    }

    /// The denominator is the feature's CELLS, not the cells that recorded
    /// the retired optional `tier`. A post-D7 feature records no tier at all,
    /// so a tier-shaped denominator would be 0 for every one of them, the
    /// share would be 0.0, and this refusal could never fire again.
    #[test]
    fn the_ration_counts_every_cell_of_the_feature_not_just_tiered_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Two escalated, one plain — no `tier` anywhere in this store.
        write_cell_fixture(root, "o-1", &escalated_cell("o-1", "f"));
        write_cell_fixture(root, "o-2", &escalated_cell("o-2", "f"));
        let mut plain = cell("o-3", "open", "f", json!([]));
        plain["role"] = json!("code");
        write_cell_fixture(root, "o-3", &plain);
        let mut target = cell("target", "open", "f", json!([]));
        target["role"] = json!("code");
        write_cell_fixture(root, "target", &target);

        let refusal = thrown(set_escalation(root, "target", true, None));
        assert!(refusal.contains("3/4"), "{refusal}");
        assert!(refusal.contains("75%"), "{refusal}");
    }

    /// Another feature's escalations are not this feature's budget — the
    /// scope the share is taken over is unchanged.
    #[test]
    fn the_ration_is_scoped_to_the_cells_own_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for id in ["x-1", "x-2", "x-3"] {
            write_cell_fixture(root, id, &escalated_cell(id, "other"));
        }
        // Feature "f": the target plus two plain cells — 1/3, under budget.
        // Counted store-wide it would be 4/6 and would refuse.
        for id in ["f-1", "f-2"] {
            let mut plain = cell(id, "open", "f", json!([]));
            plain["role"] = json!("code");
            write_cell_fixture(root, id, &plain);
        }
        let mut target = cell("target", "open", "f", json!([]));
        target["role"] = json!("code");
        write_cell_fixture(root, "target", &target);
        let cell = set_escalation(root, "target", true, None)
            .expect("another feature's escalations never charge this one");
        assert_eq!(cell["escalate"], json!(true));
    }

    // ══ wf-1 — `cells finish` from a granted worktree ═════════════════════
    //
    // "Today `bee cells finish` refuses inside a granted worktree... Fix
    // that one verb." The fixture is the same real `git worktree add` shape
    // reservations/tests.rs's `worktree_fixture` uses (main + a REGISTERED
    // `wt-granted` + an unregistered `wt-ungranted`), grown a cell store and
    // a `commands.test` declaration in MAIN so the new FULL-door code path
    // is exercised end to end, not just its root arithmetic.

    /// main + `wt-granted` (registered) + `wt-ungranted` (not) — a REAL git
    /// worktree link, same fixture shape as reservations/tests.rs's own
    /// `worktree_fixture` (which this deliberately does not import: that
    /// module's fixture is private to its own `#[cfg(test)] mod tests`).
    fn wf_worktree_fixture(tmp: &Path) -> (PathBuf, PathBuf, PathBuf) {
        // Canonicalize once, up front: on a Windows runner `tempdir()` can
        // hand back an 8.3 short form (RUNNER~1), and every path this
        // fixture builds — plus every git command run against them — must
        // share one spelling with what git's own gitdir chain reports, or
        // `wf_nrm()`'s identity checks compare apples to short-name apples.
        let tmp = dunce::canonicalize(tmp).unwrap_or_else(|_| tmp.to_path_buf());
        let main = tmp.join("main");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::write(main.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        std::fs::write(main.join("f.txt"), "x").unwrap();
        git_ok(&main, &["init", "-q", "-b", "main", "."]);
        git_ok(&main, &["config", "user.email", "a@b.c"]);
        git_ok(&main, &["config", "user.name", "t"]);
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "init"]);
        let granted = tmp.join("wt-granted");
        let ungranted = tmp.join("wt-ungranted");
        git_ok(&main, &["worktree", "add", "-q", granted.to_str().unwrap(), "-b", "wt/g"]);
        git_ok(&main, &["worktree", "add", "-q", ungranted.to_str().unwrap(), "-b", "wt/u"]);
        std::fs::create_dir_all(main.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            main.join(".bee").join("runtime").join("worktree-grants.json"),
            "{\"wt-granted\": true}\n",
        )
        .unwrap();
        std::fs::create_dir_all(granted.join(".bee")).unwrap();
        std::fs::write(granted.join(".bee").join("onboarding.json"), "{}\n").unwrap();
        (main, granted, ungranted)
    }

    fn wf_roots_at(cwd: &Path) -> StoreRoots {
        match resolve_store_root_worktree(cwd) {
            RootsWt::Go(r) => r,
            _ => panic!("expected a resolvable root at {}", cwd.display()),
        }
    }

    /// Identity, not spelling — same rationale as reservations/tests.rs's
    /// own `nrm`: a tempdir path and its 8.3/case-folded twin resolve to the
    /// same directory on a Windows runner, and a byte compare fails for a
    /// reason unrelated to what is being asserted.
    fn wf_nrm(p: &Path) -> String {
        let c = dunce::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        c.to_string_lossy().replace('/', "\\")
    }

    fn wf_cap_flags(id: &str) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: Vec::new(),
            deviations: Vec::new(),
            deviation: None,
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: None,
            inline_reason: None,
            report: Some(default_test_report_json()),
            sync_ack: None,
        }
    }

    fn wf_cell_body(id: &str, worker: Option<&str>) -> Value {
        json!({
            "id": id, "feature": "f", "title": "t", "action": "a",
            "verify": "echo ok", "lane": "tiny", "status": "claimed",
            "deps": [], "files": [],
            "trace": worker.map(|w| json!({"worker": w})).unwrap_or_else(|| json!({})),
        })
    }

    /// `finish_topology` (finish_support.rs) is the exact split wf-1's
    /// must-haves depend on: the cell/claim root is ALWAYS main, and the
    /// hold topology matches `StoreRoots::hold_topology()` unchanged. From
    /// MAIN itself the answer is byte-identical to what the narrow door
    /// produced before this cell (must-have 4). decision 13ce1858
    /// (test-cadence-boundary D1): the function's third return value, the
    /// declared-test cwd, is gone with the per-cap test run — this test no
    /// longer has a cwd to assert on granted vs ungranted.
    #[test]
    fn finish_topology_puts_the_cell_root_at_main_for_every_checkout_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, ungranted) = wf_worktree_fixture(tmp.path());

        // ORDINARY (from main): root == main, holder "main".
        let (cr, topo) = finish_topology(&wf_roots_at(&main));
        assert_eq!(wf_nrm(&cr), wf_nrm(&main));
        let (m, h) = topo.expect("ordinary always has a topology");
        assert_eq!(wf_nrm(&m), wf_nrm(&main));
        assert_eq!(h, "main");

        // GRANTED worktree: cell root still at MAIN, holder the
        // git-verified worktree id.
        let (cr, topo) = finish_topology(&wf_roots_at(&granted));
        assert_eq!(wf_nrm(&cr), wf_nrm(&main), "the cell record and claim resolve at MAIN");
        let (m, h) = topo.expect("a granted worktree holds");
        assert_eq!(wf_nrm(&m), wf_nrm(&main));
        assert_eq!(h, "wt-granted");

        // UNGRANTED worktree: unchanged from today — root already IS main's
        // store, and hold release is skipped entirely (topology === null).
        let (cr, topo) = finish_topology(&wf_roots_at(&ungranted));
        assert_eq!(wf_nrm(&cr), wf_nrm(&main));
        assert!(topo.is_none());
    }

    /// Every OTHER mutating cells verb (here: the door `cap` and everything
    /// else in `try_mutating` actually dispatch through) still refuses a
    /// granted worktree by name — `finish` alone widened.
    #[test]
    fn the_narrow_door_still_refuses_a_granted_worktree_while_the_full_door_serves_it() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, _ungranted) = wf_worktree_fixture(tmp.path());

        match resolve_store_root(&granted) {
            Roots::Unsupported(crate::roots::Unsupported::GrantedWorktree { main_root }) => {
                assert_eq!(wf_nrm(&main_root), wf_nrm(&main));
            }
            _ => panic!("expected the narrow door (cap and every other verb) to still refuse"),
        }
        assert!(matches!(resolve_store_root_worktree(&granted), RootsWt::Go(_)));
    }

    /// must-have 1: `finish_cap_and_release` — `cells finish`'s tested core
    /// — caps the cell at the MAIN store (never writing anything under the
    /// worktree's own `.bee/cells`). decision 13ce1858
    /// (test-cadence-boundary D1): the cap no longer spawns the declared
    /// test command at all — a repo whose declared command would fail
    /// (no marker.txt anywhere) still caps green, records `tests:
    /// "boundary"`, and never runs a process; a no-test repo records
    /// `"undeclared"` instead. Tests prove at the boundary now (`bee
    /// close`/`bee worktree merge`), not at this door.
    #[test]
    fn finish_caps_at_main_without_running_the_declared_test_and_records_boundary() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, _ungranted) = wf_worktree_fixture(tmp.path());
        write_bee_config(&main, &json!({"commands": {"test": "test -f marker.txt"}}));
        // Deliberately no marker.txt anywhere — if the cap spawned this
        // command from either root it would fail RED; it must not spawn it
        // at all.

        write_cell_fixture(&main, "wf-cwd-a", &wf_cell_body("wf-cwd-a", None));
        let out = finish_cap_and_release(&main, None, wf_cap_flags("wf-cwd-a"), None)
            .expect("a cap never runs the declared command, so it cannot go red on a missing marker");
        let Out::Emit(cell, _, 0) = out else { panic!("expected a green cap") };
        assert_eq!(cell["status"], json!("capped"));
        assert_eq!(cell["trace"]["tests"], json!("boundary"));
        assert!(cell["trace"].get("results").is_none(), "no run means no results path recorded");
        assert!(cell["trace"].get("ran_at").is_none(), "no run means no ran_at recorded");
        assert!(
            !granted.join(".bee").join("cells").join("wf-cwd-a.json").exists(),
            "the cell record is never written to the worktree's own store"
        );
        assert_eq!(
            read_cell_norm(&main, "wf-cwd-a").ok().unwrap().unwrap()["status"],
            json!("capped"),
            "it lands in MAIN's store instead"
        );

        // A no-test repo still records the "undeclared" sentinel.
        write_bee_config(&main, &json!({"commands": {"test": "none"}}));
        write_cell_fixture(&main, "wf-cwd-b", &wf_cell_body("wf-cwd-b", None));
        let out = finish_cap_and_release(&main, None, wf_cap_flags("wf-cwd-b"), None)
            .expect("a no-test repo caps clean");
        let Out::Emit(cell, _, 0) = out else { panic!("expected a green cap") };
        assert_eq!(cell["trace"]["tests"], json!("undeclared"));
    }

    /// must-have 3: reservation/hold release names the worktree id as
    /// holder and actually releases both the local lease and the mirrored
    /// MAIN ledger hold — via `finish_topology`'s own `hold_topology()`,
    /// not a hardcoded `"main"`.
    #[test]
    fn finish_releases_reservations_under_the_worktree_holder_id() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, _ungranted) = wf_worktree_fixture(tmp.path());
        write_bee_config(&main, &json!({"commands": {"test": "echo ok"}}));
        write_cell_fixture(&main, "wf-rel-a", &wf_cell_body("wf-rel-a", Some("wt-agent")));

        // Reserve from inside the GRANTED worktree — mirrors into MAIN's
        // ledger under the worktree's own git-verified id, same as
        // reservations/tests.rs's `granted_worktree_mirrors_under_its_id_and_blocks_main`.
        let g = wf_roots_at(&granted);
        let (gm, gh) = g.hold_topology().expect("a granted worktree holds");
        assert_eq!(gh, "wt-granted");
        let g_topo = Some(rsv::Topo { main_root: &gm, holder: &gh });
        let g_root_s = g.root.to_str().unwrap().to_string();
        let params = rsv::ReserveParams {
            agent: "wt-agent".to_string(),
            cell: "wf-rel-a".to_string(),
            path: "src/shared.ts".to_string(),
            ttl: None,
            session: None,
            kind: None,
        };
        assert!(matches!(rsv::reserve_exec(g_topo, &g_root_s, &params, 1), Ok(Out::Emit(_, _, 0))));
        let ledger: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(&main)).unwrap()).unwrap();
        assert_eq!(ledger["holds"][0]["holder"], json!("wt-granted"));
        assert!(ledger["holds"][0]["released_at"].is_null());

        // `cells finish`, from the granted worktree, releases BOTH the
        // local lease and the mirrored MAIN hold under that same id.
        let (cells_root, topo) = finish_topology(&g);
        let topo_ref = topo.as_ref().map(|(m, h)| (m.as_path(), h.as_str()));
        let out = finish_cap_and_release(&cells_root, topo_ref, wf_cap_flags("wf-rel-a"), None)
            .expect("a clean finish");
        let Out::Emit(result, text, 0) = out else { panic!("expected a green finish") };
        assert_eq!(result["released"], json!(["src/shared.ts"]));
        assert!(text.contains("Released 1 reservation(s): src/shared.ts."), "{text}");

        let ledger_after: Value =
            serde_json::from_str(&std::fs::read_to_string(holds_ledger_path(&main)).unwrap()).unwrap();
        assert!(
            ledger_after["holds"][0]["released_at"].is_string(),
            "the mirrored hold was released, not just the local lease"
        );
    }

    /// The ordinary-checkout release path is unchanged: holder `"main"`,
    /// ledger at `root` — `finish_topology`'s own `None`-linked arm.
    #[test]
    fn finish_from_main_releases_under_holder_main_exactly_as_before() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "echo ok"}}));
        write_cell_fixture(root, "wf-rel-main", &wf_cell_body("wf-rel-main", Some("main-agent")));
        let params = rsv::ReserveParams {
            agent: "main-agent".to_string(),
            cell: "wf-rel-main".to_string(),
            path: "src/only.ts".to_string(),
            ttl: None,
            session: None,
            kind: None,
        };
        let m_topo = Some(rsv::Topo { main_root: root, holder: "main" });
        let root_s = root.to_str().unwrap().to_string();
        assert!(matches!(rsv::reserve_exec(m_topo, &root_s, &params, 1), Ok(Out::Emit(_, _, 0))));

        let out = finish_cap_and_release(root, Some((root, "main")), wf_cap_flags("wf-rel-main"), None)
            .expect("a clean finish");
        let Out::Emit(result, _, 0) = out else { panic!("expected a green finish") };
        assert_eq!(result["released"], json!(["src/only.ts"]));
    }

    // ══ irf-1 (PBI p-9c48a67c read-side residue) — a granted island's cell
    //    reads scope to its own feature ═══════════════════════════════════
    //
    // `git worktree add` checks out `.bee/cells` in FULL (it is
    // git-tracked), and ips-1's prune-on-register pass only ever removes
    // UNTRACKED foreign-feature files — a TRACKED one legitimately rides
    // along on disk forever. `list_cells`/`ready_cells` must never surface
    // it: `bee cells list`, `bee cells ready`, `bee status` counts, and
    // `claim-next` all read through this one native door.

    /// `wf_worktree_fixture`'s own `granted` worktree, given the creation
    /// identity `worktree register --feature <feature>` would have written
    /// (`write_creation_identity`, registry.rs) — by hand, since this suite
    /// exercises the READ side only.
    fn write_worktree_identity(worktree_root: &Path, feature: &str) {
        std::fs::create_dir_all(worktree_root.join(".bee").join("runtime")).unwrap();
        std::fs::write(
            worktree_root.join(".bee").join("runtime").join("worktree-identity.json"),
            format!("{{\"feature\":\"{feature}\"}}"),
        )
        .unwrap();
    }

    /// RED before irf-1: a fresh granted island legitimately holds another
    /// feature's tracked cell file (the ips-1 residue) — `list_cells` must
    /// scope it out, with no `--feature` flag needed to do it.
    #[test]
    fn list_cells_in_a_granted_island_never_surfaces_a_foreign_features_residue() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, granted, _ungranted) = wf_worktree_fixture(tmp.path());
        write_worktree_identity(&granted, "feat-a");
        // The residue: feature B's cell sitting in A's island (as a real
        // `git worktree add` would carry a tracked foreign file), alongside
        // A's own.
        write_cell_fixture(&granted, "a-1", &cell("a-1", "open", "feat-a", json!([])));
        write_cell_fixture(&granted, "b-1", &cell("b-1", "open", "feat-b", json!([])));

        let ids: Vec<String> = list_cells(&granted, None, None)
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["a-1"], "feature B's residue must never be listed from the island");

        // An explicit `--feature` flag agrees, and never widens the scope.
        let ids: Vec<String> = list_cells(&granted, Some("feat-b"), None)
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert!(ids.is_empty(), "asking for feature B by name still finds nothing inside A's island");
    }

    /// Same residue, `bee cells ready`'s own door (`ready_cells`, which
    /// wraps `list_cells`) — and claim-next's own read of it, with the
    /// caller's own feature named explicitly, exactly as `run_claim_next`
    /// always calls it.
    #[test]
    fn ready_cells_in_a_granted_island_never_offers_a_foreign_features_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, granted, _ungranted) = wf_worktree_fixture(tmp.path());
        write_worktree_identity(&granted, "feat-a");
        write_cell_fixture(&granted, "a-1", &cell("a-1", "open", "feat-a", json!([])));
        write_cell_fixture(&granted, "b-1", &cell("b-1", "open", "feat-b", json!([])));

        let ids: Vec<String> = ready_cells(&granted, None)
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["a-1"]);

        // claim-next's own call shape: always scoped to the requester's own
        // feature already — pinned so this stays true across the fix.
        let ids: Vec<String> = ready_cells(&granted, Some("feat-a"))
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["a-1"]);
        assert!(
            ready_cells(&granted, Some("feat-b")).unwrap().is_empty(),
            "claim-next must never offer feature B's cell from inside A's island"
        );
    }

    /// Every production caller passes `list_cells` an already-resolved store
    /// root (`resolve_store_root(cwd)`'s own `Ordinary(r)`), which for an
    /// UNGRANTED worktree is always `main_root` itself — so this is really
    /// the SAME case as the main-store test below. Defensively, though,
    /// `island_feature_scope` must also read as unscoped if it were ever
    /// handed the ungranted worktree's OWN raw directory directly:
    /// `resolve_roots_core` there answers `store_root == main_root !=
    /// worktree_root`, i.e. `granted() == false`.
    #[test]
    fn list_cells_at_an_ungranted_worktrees_own_path_stays_unscoped() {
        let tmp = tempfile::tempdir().unwrap();
        let (_main, _granted, ungranted) = wf_worktree_fixture(tmp.path());
        write_cell_fixture(&ungranted, "a-1", &cell("a-1", "open", "feat-a", json!([])));
        write_cell_fixture(&ungranted, "b-1", &cell("b-1", "open", "feat-b", json!([])));

        let ids: Vec<String> = list_cells(&ungranted, None, None)
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["a-1", "b-1"], "an ungranted worktree's read is unfiltered, like main's");
    }

    // ══ cwr-1 (claim-time-worktree-redirect D1/D2/D6) — claim/claim-next's
    //    execution-location annotation ═══════════════════════════════════
    //
    // `append_worktree_execution_annotation` is the exact function both
    // `run_claim` and `run_claim_next` call after building their success
    // payload; unit-testing it directly (explicit `main_root`, never
    // `set_current_dir`, D6) exercises the identical wiring both doors run.

    /// Grant present: the success text gains the suffix line and the JSON
    /// object gains `worktree_root`, both naming the granted worktree root.
    #[test]
    fn claim_annotation_names_the_granted_worktree_root() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, granted, _ungranted) = wf_worktree_fixture(tmp.path());
        write_worktree_identity(&granted, "f");

        let mut obj = Map::new();
        obj.insert("id".into(), json!("c-1"));
        let mut text = "Claimed c-1 for w1.".to_string();
        append_worktree_execution_annotation(&main, Some("f"), &mut obj, &mut text);

        let worktree_root = obj["worktree_root"].as_str().unwrap().to_string();
        assert!(text.contains(&worktree_root), "{text}");
        assert!(text.contains("execution runs from a session rooted there"), "{text}");
        assert_eq!(
            wf_nrm(Path::new(&worktree_root)),
            wf_nrm(&granted),
            "worktree_root resolves to the granted worktree"
        );
    }

    /// No grant recorded for the feature at all: both the text and the
    /// object stay byte-identical to what the caller built.
    #[test]
    fn claim_annotation_absent_without_a_grant() {
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path(); // no `.bee/runtime/worktree-grants.json` at all

        let mut obj = Map::new();
        obj.insert("id".into(), json!("c-2"));
        let mut text = "Claimed c-2 for w1.".to_string();
        append_worktree_execution_annotation(main, Some("f"), &mut obj, &mut text);

        assert_eq!(text, "Claimed c-2 for w1.");
        assert!(obj.get("worktree_root").is_none());
    }

    /// Unresolvable grant entry (D1/D6 named fail-open case): a grant is
    /// registered (`worktree-grants.json` names "wt-granted") but the
    /// worktree carries no readable creation identity, so
    /// `find_feature_worktree_grant` cannot tell whether it is or is not
    /// this feature's grant — same discipline as the refusal guard's own
    /// `Unresolvable` arm: never a confident annotation.
    #[test]
    fn claim_annotation_absent_on_an_unresolvable_grant_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted, _ungranted) = wf_worktree_fixture(tmp.path());
        // Deliberately no `write_worktree_identity` call — the registered
        // "wt-granted" worktree's creation identity stays unreadable.

        let mut obj = Map::new();
        obj.insert("id".into(), json!("c-3"));
        let mut text = "Claimed c-3 for w1.".to_string();
        append_worktree_execution_annotation(&main, Some("f"), &mut obj, &mut text);

        assert_eq!(text, "Claimed c-3 for w1.", "an unresolvable grant must fail open, not annotate");
        assert!(obj.get("worktree_root").is_none());
    }

    /// The MAIN store itself, several features at once — pinned
    /// byte-identical: `island_feature_scope` never engages outside a
    /// GRANTED worktree island, so this is the exact pre-fix behavior.
    #[test]
    fn list_cells_at_the_main_store_shows_every_feature_unfiltered() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, _granted, _ungranted) = wf_worktree_fixture(tmp.path());
        write_cell_fixture(&main, "a-1", &cell("a-1", "open", "feat-a", json!([])));
        write_cell_fixture(&main, "b-1", &cell("b-1", "open", "feat-b", json!([])));
        write_cell_fixture(&main, "c-1", &cell("c-1", "open", "feat-c", json!([])));

        let ids: Vec<String> = list_cells(&main, None, None)
            .unwrap()
            .iter()
            .map(|c| js_string_or_undefined(c.get("id")))
            .collect();
        assert_eq!(ids, vec!["a-1", "b-1", "c-1"]);
    }

    // ══ B-P2-8 — `cells update` arms the behavior door ═══════════════════
    //
    // `run_update` (handlers_write.rs) resolves its store root off
    // `std::env::current_dir()` — process-global — so it is exercised
    // out-of-process via a `#[ignore]`d child, the same isolation
    // `session_id_env_chain_child` (above) uses for its own process-global
    // seam, rather than mutating this binary's shared cwd under every other
    // test in the suite.

    fn bp28_repo(root: &Path) {
        let dir = root.join(".bee");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("onboarding.json"), "{}\n").unwrap();
    }

    const CELLS_UPDATE_BEHAVIOR_CHILD: &str = "verbs::cells::tests::cells_update_behavior_child";

    /// Runs ONLY as a child of the tests below — applies `patch.json` (left
    /// in its cwd by the parent) to cell "c-1" through the REAL `cells
    /// update` CLI door.
    #[test]
    #[ignore = "spawned by the cells update behavior-door tests"]
    fn cells_update_behavior_child() {
        let (flags, use_json) = rsv::parse_flags(&["--id", "c-1", "--file", "patch.json", "--json"])
            .expect("well-formed fixture argv");
        run_update(flags, use_json, Instant::now());
    }

    /// Spawns `cells_update_behavior_child` with `root` as its cwd and
    /// `patch` written to `patch.json` there, and returns the raw process
    /// output — never a stdout parse, since every case below only needs to
    /// read the cell file the child wrote back off disk.
    fn cells_update_behavior_run(root: &Path, patch: &Value) -> std::process::Output {
        std::fs::write(root.join("patch.json"), jsjson::stringify_pretty(patch)).unwrap();
        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["--exact", CELLS_UPDATE_BEHAVIOR_CHILD, "--ignored", "--test-threads", "1"]);
        cmd.current_dir(root);
        cmd.output().expect("spawn the test binary")
    }

    /// must-have: an update that sets `change_class` to `"behavior"`, with
    /// no explicit `behavior_change` in the SAME patch, arms
    /// `trace.behavior_change = true` — read back through the exact door the
    /// close-time gate uses (`resolve_declared_behavior_change`), never a
    /// raw field peek.
    #[test]
    fn update_setting_change_class_to_behavior_arms_the_close_door() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        bp28_repo(root);
        write_cell_fixture(root, "c-1", &cell("c-1", "open", "f", json!([])));

        let out = cells_update_behavior_run(root, &json!({"change_class": "behavior"}));
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let updated = read_cell(root, "c-1").unwrap().unwrap();
        let Value::Object(map) = &updated else { panic!("expected an object") };
        assert!(
            resolve_declared_behavior_change(map),
            "change_class:\"behavior\" must arm the door: {updated}"
        );
    }

    /// An explicit `behavior_change` in the SAME patch wins over the
    /// change_class default — an explicit `false` is a deliberate opt-out
    /// and stays honored.
    #[test]
    fn update_explicit_behavior_change_false_wins_over_the_default() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        bp28_repo(root);
        write_cell_fixture(root, "c-1", &cell("c-1", "open", "f", json!([])));

        let out = cells_update_behavior_run(
            root,
            &json!({"change_class": "behavior", "behavior_change": false}),
        );
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let updated = read_cell(root, "c-1").unwrap().unwrap();
        let Value::Object(map) = &updated else { panic!("expected an object") };
        assert!(
            !resolve_declared_behavior_change(map),
            "an explicit false in the same call must be honored: {updated}"
        );
    }

    /// Changing `change_class` AWAY from `"behavior"` changes nothing — the
    /// door only ever arms, it never disarms an already-armed cell.
    #[test]
    fn update_changing_change_class_away_from_behavior_leaves_the_door_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        bp28_repo(root);
        let mut seed = cell("c-1", "open", "f", json!([]));
        seed["change_class"] = json!("behavior");
        seed["trace"] = json!({"behavior_change": true});
        write_cell_fixture(root, "c-1", &seed);

        let out = cells_update_behavior_run(root, &json!({"change_class": "bugfix"}));
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let updated = read_cell(root, "c-1").unwrap().unwrap();
        let Value::Object(map) = &updated else { panic!("expected an object") };
        assert!(
            resolve_declared_behavior_change(map),
            "an already-armed door must stay armed when change_class moves away from behavior: {updated}"
        );
        assert_eq!(updated["change_class"], json!("bugfix"));
    }

    // ══ sra-2 — `cells schedule` renders the obligation conflicts it
    // computes ═══════════════════════════════════════════════════════════
    //
    // `run_schedule` (handlers_meta.rs) resolves its store root off
    // `std::env::current_dir()` — process-global — so it is exercised
    // out-of-process via a `#[ignore]`d child, the same isolation
    // `cells_update_behavior_child` (above) uses for its own process-global
    // seam, rather than mutating this binary's shared cwd under every other
    // test in the suite.

    const CELLS_SCHEDULE_JSON_CHILD: &str = "verbs::cells::tests::cells_schedule_behavior_child_json";
    const CELLS_SCHEDULE_TEXT_CHILD: &str = "verbs::cells::tests::cells_schedule_behavior_child_text";

    /// Runs ONLY as a child of the tests below — drives the REAL `cells
    /// schedule --json` CLI door over whatever cells are on disk at its cwd.
    #[test]
    #[ignore = "spawned by the cells schedule behavior-door tests"]
    fn cells_schedule_behavior_child_json() {
        let (flags, use_json) = rsv::parse_flags(&["--json"]).expect("well-formed fixture argv");
        run_schedule(flags, use_json, Instant::now());
    }

    /// Runs ONLY as a child of the tests below — drives the REAL `cells
    /// schedule` (text render) CLI door over whatever cells are on disk at
    /// its cwd.
    #[test]
    #[ignore = "spawned by the cells schedule behavior-door tests"]
    fn cells_schedule_behavior_child_text() {
        let (flags, use_json) = rsv::parse_flags(&[]).expect("well-formed fixture argv");
        run_schedule(flags, use_json, Instant::now());
    }

    /// Spawns the named schedule child with `root` as its cwd, runs it under
    /// `--nocapture` (otherwise libtest swallows a PASSING test's own
    /// stdout), and strips the surrounding libtest banner
    /// (`running 1 test` / `test <name> ... ok` / `test result: ...`),
    /// returning exactly the command's own stdout (JSON payload or text
    /// render) — nothing else.
    fn cells_schedule_run(root: &Path, child: &str) -> String {
        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["--exact", child, "--ignored", "--test-threads", "1", "--nocapture"]);
        cmd.current_dir(root);
        let out = cmd.output().expect("spawn the test binary");
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8(out.stdout).unwrap();
        let prefix = format!("test {child} ... ");
        let start = stdout.find(&prefix).expect("libtest status line") + prefix.len();
        let suffix = "ok\n\ntest result:";
        let end = stdout[start..].find(suffix).expect("libtest result banner") + start;
        stdout[start..end].to_string()
    }

    /// must-have: `skills/a.md` and `skills/b.md` never literally overlap,
    /// but both fall under the "skills" regen-obligation root, so wave
    /// placement defers "rb" behind "ra" (proven by
    /// `compute_schedule_serializes_cells_sharing_a_regen_obligation_root`
    /// above). The `cells schedule` COMMAND must render that WHY, driven
    /// through the real handler — not just the computation it calls.
    #[test]
    fn schedule_command_json_renders_the_obligation_conflict_it_computes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        bp28_repo(root);
        let mut ra = cell("ra", "open", "sched-conflict", json!([]));
        ra["files"] = json!(["skills/a.md"]);
        write_cell_fixture(root, "ra", &ra);
        let mut rb = cell("rb", "open", "sched-conflict", json!([]));
        rb["files"] = json!(["skills/b.md"]);
        write_cell_fixture(root, "rb", &rb);

        let stdout = cells_schedule_run(root, CELLS_SCHEDULE_JSON_CHILD);
        let payload: Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(payload["waves"], json!([["ra"], ["rb"]]));
        assert_eq!(
            payload["obligation_conflicts"],
            json!([{"deferred": "rb", "blocking": "ra", "root": "skills"}])
        );
    }

    #[test]
    fn schedule_command_text_renders_one_line_per_obligation_conflict() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        bp28_repo(root);
        let mut ra = cell("ra", "open", "sched-conflict", json!([]));
        ra["files"] = json!(["skills/a.md"]);
        write_cell_fixture(root, "ra", &ra);
        let mut rb = cell("rb", "open", "sched-conflict", json!([]));
        rb["files"] = json!(["skills/b.md"]);
        write_cell_fixture(root, "rb", &rb);

        let stdout = cells_schedule_run(root, CELLS_SCHEDULE_TEXT_CHILD);
        assert_eq!(stdout, "Wave 1: ra\nWave 2: rb\nrb waits for ra — shared regen root skills\n");
    }

    /// must-have: cells that never share a regen-obligation root render
    /// BYTE-IDENTICAL to the pre-sra-2 command — no empty `obligation_conflicts`
    /// noise in the text render, and an empty array (never omitted, never
    /// null) in the JSON payload.
    #[test]
    fn schedule_command_with_no_conflicts_renders_byte_identical_to_before() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        bp28_repo(root);
        write_cell_fixture(root, "p", &{
            let mut p = cell("p", "open", "sched-plain", json!([]));
            p["files"] = json!(["docs/readme.md"]);
            p
        });
        write_cell_fixture(root, "q", &{
            let mut q = cell("q", "open", "sched-plain", json!([]));
            q["files"] = json!(["notes.txt"]);
            q
        });

        let text_stdout = cells_schedule_run(root, CELLS_SCHEDULE_TEXT_CHILD);
        assert_eq!(text_stdout, "Wave 1: p, q\n");

        let json_stdout = cells_schedule_run(root, CELLS_SCHEDULE_JSON_CHILD);
        let payload: Value = serde_json::from_str(&json_stdout).unwrap();
        assert_eq!(payload["waves"], json!([["p", "q"]]));
        assert_eq!(payload["obligation_conflicts"], json!([]));
        assert_eq!(
            payload["diagnostics"],
            json!({"cycles": [], "unsatisfiable_deps": [], "empty_files": []})
        );
    }

    // ══ wfl-4 — `bee dispatch wave`: the current schedule wave, claimed and
    // prepared in one call ═════════════════════════════════════════════════
    //
    // `run_dispatch_wave` (verbs/drivers/prepare.rs) resolves its store root
    // AND its hold topology off `std::env::current_dir()` — process-global —
    // so it is exercised out-of-process via a `#[ignore]`d child, the same
    // isolation `cells_update_behavior_child` (above) uses for its own
    // process-global seam.

    const DISPATCH_WAVE_CHILD: &str = "verbs::cells::tests::wfl4_dispatch_wave_child";

    /// Runs ONLY as a child of the tests below — drives the REAL `bee
    /// dispatch wave --runtime claude --json` CLI door over whatever cells
    /// are on disk at its cwd, plus whatever extra argv
    /// `wfl4_dispatch_wave_run` relayed through `WFL4_WAVE_ARGS` (space-
    /// joined; every caller's own tokens are plain flag/value pairs with no
    /// embedded spaces, so a naive split is exact here).
    #[test]
    #[ignore = "spawned by the dispatch_wave_* tests"]
    fn wfl4_dispatch_wave_child() {
        let extra = std::env::var("WFL4_WAVE_ARGS").unwrap_or_default();
        let extra_toks: Vec<&str> = extra.split(' ').filter(|t| !t.is_empty()).collect();
        let mut argv: Vec<&str> = vec!["--runtime", "claude", "--json"];
        argv.extend(extra_toks);
        let (flags, use_json) = rsv::parse_flags(&argv).expect("well-formed fixture argv");
        crate::verbs::drivers::run_dispatch_wave(flags, use_json, Instant::now());
    }

    /// A store root with the one marker `resolve_store_root` requires, plus
    /// an optional `models` config `prepare_dispatch` reads to resolve a
    /// tier — the same two-file recipe `dispatch_prepare_claim_payload_pins_
    /// worker_registered_true`'s `repo()` fixture (drivers/tests.rs) uses for
    /// the identical claim+prepare seam.
    fn wfl4_wave_root(tmp: &tempfile::TempDir, config: &str) -> PathBuf {
        let root = tmp.path().to_path_buf();
        let dir = root.join(".bee");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("onboarding.json"), "{}\n").unwrap();
        std::fs::write(dir.join("config.json"), config).unwrap();
        root
    }

    /// Spawns `wfl4_dispatch_wave_child` with `root` as its cwd and `extra`
    /// relayed as additional argv (past the fixed `--runtime claude --json`)
    /// via `WFL4_WAVE_ARGS`, and returns the parsed payload — sliced out of
    /// the raw stdout by its outermost braces (the libtest banner surrounds
    /// it; `--nocapture` is what makes a PASSING test's own stdout visible
    /// at all), the same tolerant slice `dispatch_prepare_claim_payload_pins_
    /// worker_registered_true` (drivers/tests.rs) uses for the identical
    /// seam. A refusal's `{"error": ...}` envelope slices the same way, so
    /// this same helper covers both the success and refusal shapes.
    fn wfl4_dispatch_wave_run(root: &Path, extra: &[&str]) -> Value {
        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["--exact", DISPATCH_WAVE_CHILD, "--ignored", "--test-threads", "1", "--nocapture"]);
        cmd.current_dir(root);
        if !extra.is_empty() {
            cmd.env("WFL4_WAVE_ARGS", extra.join(" "));
        }
        let out = cmd.output().expect("spawn the test binary");
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let start =
            stdout.find('{').unwrap_or_else(|| panic!("no JSON payload in child stdout:\n{stdout}"));
        let end = stdout.rfind('}').map(|i| i + 1).unwrap_or_else(|| {
            panic!("no JSON payload in child stdout:\n{stdout}")
        });
        serde_json::from_str(&stdout[start..end])
            .unwrap_or_else(|e| panic!("child stdout was not valid JSON ({e}):\n{stdout}"))
    }

    /// must-have: "Each payload equals what per-cell prepare would emit" —
    /// two disjoint, ready, open cells in the current wave each earn a full
    /// claim+reserve+payload envelope, never a shared/second copy of
    /// `dispatch prepare --claim`'s own per-cell path.
    #[test]
    fn dispatch_wave_returns_a_payload_per_disjoint_ready_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = wfl4_wave_root(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        lane_with_route(&root, "f");
        // `tier` and `role` are set TOGETHER on every fixture this suite
        // actually dispatches. The reason is the blind spot D7 opens: a raw
        // fixture bypasses `validate_new_cell`, so it would not go red when
        // `role` became required — it would quietly resolve whatever the
        // default is, and the assertion below would still pass while proving
        // nothing about which model the wave chose. Naming the role keeps
        // this test's model choice deliberate through the tier retirement.
        let mut a = cell("wa-1", "open", "f", json!([]));
        a["files"] = json!(["docs/wa-1.md"]);
        a["tier"] = json!("generation");
        a["role"] = json!("code");
        write_cell_fixture(&root, "wa-1", &a);
        let mut b = cell("wa-2", "open", "f", json!([]));
        b["files"] = json!(["docs/wa-2.md"]);
        b["tier"] = json!("generation");
        b["role"] = json!("code");
        write_cell_fixture(&root, "wa-2", &b);

        let payload = wfl4_dispatch_wave_run(&root, &["--feature", "f"]);
        let wave = payload["wave"].as_array().unwrap_or_else(|| panic!("payload: {payload}"));
        assert_eq!(wave.len(), 2, "payload: {payload}");
        assert_eq!(payload["skipped"], json!([]), "payload: {payload}");
        for item in wave {
            assert_eq!(item["claimed"], json!(true), "payload: {payload}");
            assert_eq!(item["tool"], json!("Agent"), "payload: {payload}");
            assert_eq!(item["payload"]["subagent_type"], json!("bee-build"), "payload: {payload}");
            assert!(!item["reserved"].as_array().unwrap().is_empty(), "payload: {payload}");
        }
        let economics = payload["economics"].as_array().unwrap_or_else(|| panic!("payload: {payload}"));
        let mut ids: Vec<&str> = economics.iter().map(|e| e["id"].as_str().unwrap()).collect();
        ids.sort();
        assert_eq!(ids, vec!["wa-1", "wa-2"], "payload: {payload}");
    }

    /// must-have: "one refusal never poisons the batch" — a cell already
    /// claimed by another worker still occupies its wave slot (schedulable
    /// covers open AND claimed), the wave batch attempts it through the SAME
    /// claim door `dispatch prepare --claim` uses, and the door's own typed
    /// "not open" refusal lands the cell in `skipped` with a typed reason
    /// rather than aborting the call.
    #[test]
    fn dispatch_wave_skips_a_foreign_claimed_cell_with_a_typed_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let root = wfl4_wave_root(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        lane_with_route(&root, "f");
        let mut c = cell("wb-1", "claimed", "f", json!([]));
        c["trace"] = json!({"worker": "someone-else"});
        write_cell_fixture(&root, "wb-1", &c);

        let payload = wfl4_dispatch_wave_run(&root, &["--feature", "f"]);
        assert_eq!(payload["wave"], json!([]), "payload: {payload}");
        assert_eq!(payload["economics"], json!([]), "payload: {payload}");
        let skipped = payload["skipped"].as_array().unwrap_or_else(|| panic!("payload: {payload}"));
        assert_eq!(skipped.len(), 1, "payload: {payload}");
        assert_eq!(skipped[0]["id"], json!("wb-1"));
        assert_eq!(skipped[0]["reason"], json!("already_claimed"));
        assert!(
            skipped[0]["detail"].as_str().unwrap().contains("not \"open\""),
            "payload: {payload}"
        );
    }

    /// must-have: an empty schedule (no schedulable cells at all) returns
    /// every array empty rather than an absent/null key.
    #[test]
    fn dispatch_wave_over_an_empty_store_returns_empty_arrays() {
        let tmp = tempfile::tempdir().unwrap();
        let root = wfl4_wave_root(&tmp, "{}");

        let payload = wfl4_dispatch_wave_run(&root, &["--feature", "f"]);
        assert_eq!(payload, json!({"wave": [], "skipped": [], "economics": []}));
    }

    // ══ dispatch review P1 — scope to one feature, bound the batch ═════════

    /// must-have: "no resolvable feature is a typed refusal, never a silent
    /// all-features grab" — no `--feature`, no session lane binding, and no
    /// default-record `feature` leaves nothing to resolve; the door refuses
    /// by name rather than falling back to `cells schedule`'s own
    /// every-feature default.
    #[test]
    fn dispatch_wave_refuses_when_no_feature_resolves() {
        let tmp = tempfile::tempdir().unwrap();
        let root = wfl4_wave_root(&tmp, "{}");

        let payload = wfl4_dispatch_wave_run(&root, &[]);
        let error = payload["error"].as_str().unwrap_or_else(|| panic!("payload: {payload}"));
        assert!(error.contains("no feature resolved"), "payload: {payload}");
        assert!(error.contains("--feature"), "payload: {payload}");
    }

    /// must-have: "a wave never claims a cell outside the resolved feature"
    /// — two features each have a ready, disjoint cell; `--feature f` claims
    /// only `f`'s cell, and `g`'s cell never appears in `wave`, `skipped`,
    /// or `economics`.
    #[test]
    fn dispatch_wave_never_spans_a_second_feature() {
        let tmp = tempfile::tempdir().unwrap();
        let root = wfl4_wave_root(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        lane_with_route(&root, "f");
        lane_with_route(&root, "g");
        let mut f1 = cell("wc-1", "open", "f", json!([]));
        f1["files"] = json!(["docs/wc-1.md"]);
        f1["tier"] = json!("generation");
        f1["role"] = json!("code");
        write_cell_fixture(&root, "wc-1", &f1);
        let mut g1 = cell("wg-1", "open", "g", json!([]));
        g1["files"] = json!(["docs/wg-1.md"]);
        g1["tier"] = json!("generation");
        g1["role"] = json!("code");
        write_cell_fixture(&root, "wg-1", &g1);

        let payload = wfl4_dispatch_wave_run(&root, &["--feature", "f"]);
        let wave = payload["wave"].as_array().unwrap_or_else(|| panic!("payload: {payload}"));
        assert_eq!(wave.len(), 1, "payload: {payload}");
        assert_eq!(payload["skipped"], json!([]), "payload: {payload}");
        let economics = payload["economics"].as_array().unwrap_or_else(|| panic!("payload: {payload}"));
        let ids: Vec<&str> = economics.iter().map(|e| e["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["wc-1"], "payload: {payload}");
        // g-1 stands untouched — still open, never claimed by this wave.
        let g_after = read_cell_fixture(&root, "wg-1");
        assert_eq!(g_after["status"], json!("open"), "payload: {payload}");
    }

    /// must-have: "--limit bounds the claims" — two disjoint ready cells of
    /// the same feature, `--limit 1` claims exactly one and leaves the other
    /// untouched (open, unclaimed, absent from every returned array) rather
    /// than reporting it `skipped`.
    #[test]
    fn dispatch_wave_limit_caps_the_claims() {
        let tmp = tempfile::tempdir().unwrap();
        let root = wfl4_wave_root(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        lane_with_route(&root, "f");
        let mut a = cell("wl-1", "open", "f", json!([]));
        a["files"] = json!(["docs/wl-1.md"]);
        a["tier"] = json!("generation");
        a["role"] = json!("code");
        write_cell_fixture(&root, "wl-1", &a);
        let mut b = cell("wl-2", "open", "f", json!([]));
        b["files"] = json!(["docs/wl-2.md"]);
        b["tier"] = json!("generation");
        b["role"] = json!("code");
        write_cell_fixture(&root, "wl-2", &b);

        let payload = wfl4_dispatch_wave_run(&root, &["--feature", "f", "--limit", "1"]);
        let wave = payload["wave"].as_array().unwrap_or_else(|| panic!("payload: {payload}"));
        assert_eq!(wave.len(), 1, "payload: {payload}");
        assert_eq!(payload["skipped"], json!([]), "payload: {payload}");
        // Exactly one of the two ready cells stands claimed; the other was
        // never attempted at all (still open).
        let economics = payload["economics"].as_array().unwrap_or_else(|| panic!("payload: {payload}"));
        assert_eq!(economics.len(), 1, "payload: {payload}");
        let claimed_id = economics[0]["id"].as_str().unwrap_or_else(|| panic!("payload: {payload}"));
        let untouched = if claimed_id == "wl-1" { "wl-2" } else { "wl-1" };
        let untouched_after = read_cell_fixture(&root, untouched);
        assert_eq!(untouched_after["status"], json!("open"), "payload: {payload}");
    }

    /// must-have: `wave_skip_reason` names an unwind failure by its own
    /// reason rather than folding it back into `reservation_conflict` —
    /// dispatch review P2. `claim_and_reserve_for_dispatch`'s own
    /// reservation-conflict message already embeds its unwind note in the
    /// SAME string, so the "UNWIND FAILED" check must win over the
    /// "reservation conflict" substring it always co-occurs with here.
    #[test]
    fn wave_skip_reason_flags_a_failed_unwind_before_a_reservation_conflict() {
        use crate::verbs::drivers::wave_skip_reason;
        let ok_conflict = "dispatch prepare --claim: reservation conflict on cell \"x\" — \
             nothing dispatched; the claim was unwound and state restored as found:";
        assert_eq!(wave_skip_reason(ok_conflict), "reservation_conflict");
        let failed_unwind = "dispatch prepare --claim: reservation conflict on cell \"x\" — \
             nothing dispatched; UNWIND FAILED (release: ok; unclaim: boom) — restore by hand: ...:";
        assert_eq!(wave_skip_reason(failed_unwind), "unwind_failed");
    }

    // ── dispatch review delta (hpf-3): a taken claim vs. an untaken one ────

    /// must-have: "the wave never force-unclaims a cell whose claim it did
    /// not take". `claim_cell_from_flags` can hit its own exotic-shape
    /// delegate (a truthy, non-array `deps`, handlers_write.rs:915) BEFORE
    /// it ever mutates the claim — even over a cell ALREADY claimed by a
    /// live agent. The old wave unwind treated every `Err` alike and
    /// force-unclaimed unconditionally, bypassing `guard_claim_ownership`
    /// and handing that other agent's claim back to "open" — a write
    /// straight through a claim conflict. The fix must leave the foreign
    /// claim exactly as found and report a real, unwind-free reason.
    #[test]
    fn dispatch_wave_never_force_unclaims_a_claim_it_never_took() {
        let tmp = tempfile::tempdir().unwrap();
        let root = wfl4_wave_root(&tmp, r#"{"models":{"claude":{"generation":"sonnet"}}}"#);
        lane_with_route(&root, "f");
        // `deps` truthy but not an array trips the claim door's own
        // exotic-shape delegate — the SAME trigger a corrupt/foreign-
        // authored cell record could hit for real — before any claim
        // mutation, regardless of the cell's own status.
        let mut c = cell("wu-1", "claimed", "f", json!("bogus"));
        c["trace"] = json!({"worker": "someone-else"});
        write_cell_fixture(&root, "wu-1", &c);

        let payload = wfl4_dispatch_wave_run(&root, &["--feature", "f"]);
        assert_eq!(payload["wave"], json!([]), "payload: {payload}");
        assert_eq!(payload["economics"], json!([]), "payload: {payload}");
        let skipped = payload["skipped"].as_array().unwrap_or_else(|| panic!("payload: {payload}"));
        assert_eq!(skipped.len(), 1, "payload: {payload}");
        assert_eq!(skipped[0]["id"], json!("wu-1"), "payload: {payload}");
        assert_eq!(skipped[0]["reason"], json!("claim_refused"), "payload: {payload}");
        let detail = skipped[0]["detail"].as_str().unwrap_or_else(|| panic!("payload: {payload}"));
        assert!(
            !detail.contains("UNWIND FAILED"),
            "a benign, never-taken claim must carry no unwind note: {detail}"
        );

        // The other agent's claim stands exactly as found — never force-
        // unclaimed by a wave that never took it.
        let after = read_cell_fixture(&root, "wu-1");
        assert_eq!(after["status"], json!("claimed"), "payload: {payload}");
        assert_eq!(after["trace"]["worker"], json!("someone-else"), "payload: {payload}");
    }

    /// must-have: "a real unwind clears claim, reservations and the worker
    /// row" — the carried-over P2 item. `unwind_wave_claim` released
    /// reservations and unclaimed the cell but never removed the
    /// `workers[]` row `dp-r1` registered for the same claim, leaving a
    /// `running` row against a cell the unwind just returned to `open`.
    /// Exercises `claim_and_reserve_for_dispatch` + `unwind_wave_claim`
    /// directly (both take `root` explicitly, no cwd dependency) — the same
    /// real claim+register `dispatch wave`'s `prepare_failed` unwind undoes.
    #[test]
    fn unwind_wave_claim_clears_the_claim_reservations_and_the_worker_row() {
        use crate::verbs::drivers::{claim_and_reserve_for_dispatch, unwind_wave_claim};

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("config.json"), "{}").unwrap();
        // Gate 2 (execution) must read approved for the claim door to admit
        // this cell at all — the same lane fixture the wave tests above use.
        lane_with_route(&root, "f");
        let mut c = cell("uw-1", "open", "f", json!([]));
        c["files"] = json!(["docs/uw-1.md"]);
        c["tier"] = json!("generation");
        c["role"] = json!("code");
        write_cell_fixture(&root, "uw-1", &c);

        // A real claim: taken, one file reserved, worker row registered.
        let outcome = claim_and_reserve_for_dispatch(&root, None, "uw-1", "w-uw-1", None)
            .expect("the claim door itself must not delegate over a plain, well-formed cell")
            .expect("a fresh, unreserved cell must not hit a reservation conflict");
        let (_cell, reserved, worker_registered, registration_error) = outcome;
        assert!(!reserved.is_empty(), "at least one declared file must have been reserved");
        assert!(worker_registered, "registration_error: {registration_error:?}");
        let claimed = read_cell_fixture(&root, "uw-1");
        assert_eq!(claimed["status"], json!("claimed"), "cell: {claimed}");
        let state_path = root.join(".bee").join("state.json");
        let state_before: Value = serde_json::from_str(&std::fs::read_to_string(&state_path).unwrap()).unwrap();
        let workers_before = state_before["workers"].as_array().unwrap();
        assert!(
            workers_before
                .iter()
                .any(|w| w["nickname"] == json!("w-uw-1") && w["cell"] == json!("uw-1")),
            "state before unwind: {state_before}"
        );

        // Undo it, exactly as `dispatch wave`'s `prepare_failed` unwind
        // would over its own, still-fresh claim.
        let note = unwind_wave_claim(&root, None, "w-uw-1", "uw-1");
        assert!(!note.contains("UNWIND FAILED"), "unwind note: {note}");

        let after = read_cell_fixture(&root, "uw-1");
        assert_eq!(after["status"], json!("open"), "cell: {after}");

        let state_after: Value = serde_json::from_str(&std::fs::read_to_string(&state_path).unwrap()).unwrap();
        let workers_after = state_after["workers"].as_array().unwrap();
        assert!(
            !workers_after
                .iter()
                .any(|w| w["nickname"] == json!("w-uw-1") && w["cell"] == json!("uw-1")),
            "the worker row must be removed by the unwind: {state_after}"
        );

        let active = match rsv::list_reservations(root.to_str().unwrap(), true, rsv::now_ms()) {
            Ok(v) => v,
            Err(_) => panic!("list_reservations hit an unproven shape"),
        };
        assert!(
            !active.iter().any(|r| matches!(&r.agent, Some(Value::String(s)) if s == "w-uw-1")),
            "the reservation must be released by the unwind"
        );
    }

    // ══ jo-2 — the judge obligation wired into the live `cells add` path ═══
    //
    // jo-1 built `assert_judge_obligation` and proved it at the function
    // level only; `validate_new_cell` (validate.rs) now calls it beside
    // `assert_regen_obligation`, so this exercises it through the SAME real
    // `cells add` door the tests above use for `run_update`/`run_schedule`.
    // `run_add` (handlers_write.rs) resolves its store root off
    // `std::env::current_dir()` — process-global — so it is exercised
    // out-of-process via a `#[ignore]`d child, the same isolation
    // `cells_update_behavior_child` (above) uses for its own process-global
    // seam.

    const CELLS_ADD_JUDGE_CHILD: &str = "verbs::cells::tests::cells_add_judge_obligation_child";

    /// Runs ONLY as a child of the tests below — drives the REAL `cells add
    /// --json` CLI door over `cell.json` (a single object or a batch array)
    /// left in its cwd by the parent, through the exact entry point
    /// (`run_add`) `bee cells add` itself dispatches to.
    #[test]
    #[ignore = "spawned by the judge obligation end-to-end tests (jo-2)"]
    fn cells_add_judge_obligation_child() {
        let (flags, use_json) =
            rsv::parse_flags(&["--file", "cell.json", "--json"]).expect("well-formed fixture argv");
        run_add(flags, use_json, Instant::now());
    }

    /// Spawns `cells_add_judge_obligation_child` with `root` as its cwd and
    /// `payload` (a single cell or a batch array) written to `cell.json`
    /// there, and returns the child's raw stdout — the REAL `println!`
    /// output `run_add`'s own `emit()`/`fail()` paths produce (a normalized
    /// cell, a batch array, or `{"error": ...}`), sliced out of the libtest
    /// banner the same way `cells_schedule_run` (above) does for its own
    /// out-of-process seam.
    fn cells_add_judge_obligation_run(root: &Path, payload: &Value) -> String {
        std::fs::write(root.join("cell.json"), jsjson::stringify_pretty(payload)).unwrap();
        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["--exact", CELLS_ADD_JUDGE_CHILD, "--ignored", "--test-threads", "1", "--nocapture"]);
        cmd.current_dir(root);
        let out = cmd.output().expect("spawn the test binary");
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let prefix = format!("test {CELLS_ADD_JUDGE_CHILD} ... ");
        let start = stdout.find(&prefix).expect("libtest status line") + prefix.len();
        let suffix = "ok\n\ntest result:";
        let end = stdout[start..].find(suffix).expect("libtest result banner") + start;
        stdout[start..end].to_string()
    }

    /// A `.bee` marker with nothing else declared — the same minimal root
    /// `bp28_repo` (above) sets up for `run_update`'s own out-of-process
    /// seam.
    fn judge_root() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        bp28_repo(tmp.path());
        tmp
    }

    /// A well-formed cell whose `files` touch `JUDGE_REQUIRED_ROOTS`
    /// (`packages/bee-rs/crates/bee/src/hooks`) — the judge-required root
    /// `obligation.rs` declares.
    fn judge_touching_cell(id: &str, lane: &str) -> Value {
        let mut c = json!({
            "id": id,
            "feature": "jo2",
            "title": format!("touches a guard ({id})"),
            "action": "edit guard source",
            "verify": "echo ok",
            "lane": lane,
            "role": "code",
            "files": ["packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs"],
            "affects_skills": [],
            "affects_specs": [],
        });
        if lane == "standard" || lane == "high-risk" {
            c["must_haves"] = json!({"truths": ["the guard still refuses the bad case"]});
        }
        c
    }

    /// must-have: the refusal fires through the REAL `cells add` path, not
    /// just `judge_obligation_refusal` at the function level — a tiny cell
    /// touching `packages/bee-rs/crates/bee/src/hooks` is refused, the
    /// refusal names BOTH escapes (raise the lane, or set the ack), and
    /// nothing lands on disk.
    #[test]
    fn cells_add_refuses_a_tiny_cell_touching_a_guard_root_naming_both_escapes() {
        let tmp = judge_root();
        let root = tmp.path();

        let stdout = cells_add_judge_obligation_run(root, &judge_touching_cell("jo2-a", "tiny"));
        assert!(stdout.contains("JUDGE_OBLIGATION"), "stdout: {stdout}");
        assert!(
            stdout.contains("raise this cell's lane to"),
            "must name escape #1 (raise the lane): {stdout}"
        );
        assert!(
            stdout.contains(JUDGE_ACK_FIELD),
            "must name escape #2 (the ack field): {stdout}"
        );
        assert!(
            read_cell_norm(root, "jo2-a").unwrap().is_none(),
            "a refused add must write nothing: {stdout}"
        );
    }

    /// must-have: the SAME cell at lane "standard" is accepted — the
    /// close-time judge-debt door already owes the independent read at that
    /// lane, so this authoring-time door steps aside (`JUDGE_DOOR_COVERED_
    /// LANES`).
    #[test]
    fn cells_add_accepts_the_same_cell_at_lane_standard() {
        let tmp = judge_root();
        let root = tmp.path();

        let stdout = cells_add_judge_obligation_run(root, &judge_touching_cell("jo2-b", "standard"));
        assert!(!stdout.contains("JUDGE_OBLIGATION"), "stdout: {stdout}");
        assert!(!stdout.contains("\"error\""), "must not refuse: {stdout}");
        let stored = read_cell_norm(root, "jo2-b")
            .unwrap()
            .unwrap_or_else(|| panic!("cell must be written: {stdout}"));
        assert_eq!(stored["lane"], json!("standard"), "stored: {stored}");
    }

    /// must-have: the same tiny cell carrying `judge_obligation_ack` is
    /// accepted, and the ack is recorded on the stored cell — a named skip,
    /// never a silent one.
    #[test]
    fn cells_add_accepts_a_tiny_cell_carrying_the_judge_obligation_ack() {
        let tmp = judge_root();
        let root = tmp.path();
        let mut c = judge_touching_cell("jo2-c", "tiny");
        c[JUDGE_ACK_FIELD] = json!("deliberately skipping the independent read for this tiny fix");

        let stdout = cells_add_judge_obligation_run(root, &c);
        assert!(!stdout.contains("JUDGE_OBLIGATION"), "stdout: {stdout}");
        let stored = read_cell_norm(root, "jo2-c")
            .unwrap()
            .unwrap_or_else(|| panic!("cell must be written: {stdout}"));
        assert_eq!(
            stored[JUDGE_ACK_FIELD],
            json!("deliberately skipping the independent read for this tiny fix"),
            "stored: {stored}"
        );
    }

    /// must-have: a batch where ONE cell trips the obligation refuses the
    /// WHOLE batch and writes nothing — matching how the regen refusal
    /// composes into the same whole-batch validation (every cell checked
    /// before any is written, one call naming every problem;
    /// `build_add_cells_report`).
    #[test]
    fn cells_add_batch_refuses_the_whole_batch_when_one_cell_trips_the_judge_obligation() {
        let tmp = judge_root();
        let root = tmp.path();
        let batch = json!([addable("jo2-clean"), judge_touching_cell("jo2-tripped", "tiny")]);

        let stdout = cells_add_judge_obligation_run(root, &batch);
        assert!(stdout.contains("JUDGE_OBLIGATION"), "stdout: {stdout}");
        assert!(stdout.contains("jo2-tripped"), "stdout: {stdout}");
        assert!(
            read_cell_norm(root, "jo2-clean").unwrap().is_none(),
            "the clean cell in the same batch must not be written either: {stdout}"
        );
        assert!(
            read_cell_norm(root, "jo2-tripped").unwrap().is_none(),
            "the tripping cell must not be written: {stdout}"
        );
    }

    // ═══ sync door (koh-6, D3/D4) ═══════════════════════════════════════════
    //
    // Three cap-time checks over the touched set (last commit's numstat rows
    // union `--files`, or `--files` alone with no resolvable commit):
    // (a) ownership, (b) applied_at, (c) affects_skills prediction — every
    // one a hard refusal, escaped only by a non-blank `--sync-ack`. A cell
    // predating `affects_skills` skips (c) alone.

    fn write_area_overview(root: &Path, area: &str, code: &[&str], skills: &[&str]) {
        let dir = root.join("docs/knowledge/areas").join(area);
        std::fs::create_dir_all(&dir).unwrap();
        let code_list = code.join(", ");
        let skills_list = skills.join(", ");
        let text = format!(
            "---\ntype: bee.area\ntitle: {area}\ntags: []\nbee:\n  id: area-{area}\n  areas: [{area}]\n  owns.code: [{code_list}]\n  owns.skills: [{skills_list}]\n  owns.tests: []\n---\n{area} overview.\n"
        );
        std::fs::write(dir.join("overview.md"), text).unwrap();
    }

    fn write_rule_home(root: &Path, area: &str, file: &str, rule_id: &str, applied_at: &[&str]) {
        let dir = root.join("docs/knowledge/areas").join(area);
        std::fs::create_dir_all(&dir).unwrap();
        let applied_list = applied_at.join(", ");
        let text = format!(
            "---\ntype: bee.area\ntitle: {rule_id}\ntags: []\nbee:\n  id: {rule_id}-home\n  areas: [{area}]\n  applied_at: [{applied_list}]\n---\n<!-- rule: {rule_id} -->\nRule text.\n<!-- /rule -->\n"
        );
        std::fs::write(dir.join(file), text).unwrap();
    }

    fn cap_flags_sync(id: &str, files_changed: Vec<&str>, sync_ack: Option<&str>) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: files_changed.into_iter().map(|f| json!(f)).collect(),
            deviations: Vec::new(),
            deviation: None,
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: None,
            inline_reason: None,
            report: Some(default_test_report_json()),
            sync_ack: sync_ack.map(str::to_string),
        }
    }

    #[test]
    fn owned_code_touched_without_its_skill_is_refused_and_sync_ack_escapes_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_area_overview(root, "demo", &["src/demo/*"], &["skills/demo/SKILL.md"]);
        write_cell_fixture(root, "sd-1", &cell("sd-1", "claimed", "f", json!([])));

        let flags = cap_flags_sync("sd-1", vec!["src/demo/thing.rs"], None);
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(refusal.contains("SYNC_DOOR"), "{refusal}");
        assert!(refusal.contains("demo"), "must name the area: {refusal}");
        assert!(
            refusal.contains("skills/demo/SKILL.md"),
            "must name the untouched owned skill: {refusal}"
        );
        assert_eq!(
            read_cell_norm(root, "sd-1").unwrap().unwrap()["status"],
            json!("claimed"),
            "a refused cap writes nothing"
        );

        // --sync-ack escapes it: capped, the reason lands on trace.sync_ack
        // AND as a trace.deviations line.
        let acked = cap_flags_sync(
            "sd-1",
            vec!["src/demo/thing.rs"],
            Some("skill update deferred to a follow-up cell"),
        );
        let capped = cap_cell_from_flags(root, &acked, false).unwrap();
        assert_eq!(
            capped["trace"]["sync_ack"],
            json!("skill update deferred to a follow-up cell")
        );
        assert_eq!(
            capped["trace"]["deviations"],
            json!(["sync-ack: skill update deferred to a follow-up cell"])
        );
    }

    #[test]
    fn blank_sync_ack_is_refused_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_area_overview(root, "demo", &["src/demo/*"], &["skills/demo/SKILL.md"]);
        write_cell_fixture(root, "sd-2", &cell("sd-2", "claimed", "f", json!([])));
        let before = std::fs::read_to_string(cell_file(root, "sd-2")).unwrap();

        for bad in ["", "   ", "\t\n "] {
            let flags = cap_flags_sync("sd-2", vec!["src/demo/thing.rs"], Some(bad));
            let refusal = thrown(cap_cell_from_flags(root, &flags, false));
            assert!(
                refusal.contains("--sync-ack") && refusal.contains("non-empty"),
                "refusal must name the flag: {refusal}"
            );
        }
        let after = std::fs::read_to_string(cell_file(root, "sd-2")).unwrap();
        assert_eq!(before, after, "nothing was written on refusal");
    }

    #[test]
    fn rule_home_touched_without_every_applied_at_file_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_rule_home(
            root,
            "demo",
            "rule.md",
            "demo-rule-1",
            &["skills/demo/SKILL.md", "docs/specs/demo.md"],
        );
        write_cell_fixture(root, "sd-3", &cell("sd-3", "claimed", "f", json!([])));

        // Only the home is touched — neither applied_at file is.
        let flags = cap_flags_sync("sd-3", vec!["docs/knowledge/areas/demo/rule.md"], None);
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(refusal.contains("SYNC_DOOR"), "{refusal}");
        assert!(refusal.contains("demo-rule-1"), "must name the rule: {refusal}");
        assert!(refusal.contains("skills/demo/SKILL.md"), "{refusal}");
        assert!(refusal.contains("docs/specs/demo.md"), "{refusal}");

        // Touching both applied_at files alongside the home clears it.
        let flags_ok = cap_flags_sync(
            "sd-3",
            vec![
                "docs/knowledge/areas/demo/rule.md",
                "skills/demo/SKILL.md",
                "docs/specs/demo.md",
            ],
            None,
        );
        let capped = cap_cell_from_flags(root, &flags_ok, false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
    }

    #[test]
    fn affects_skills_prediction_mismatch_is_refused_naming_the_diff() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        let mut c = cell("sd-4", "claimed", "f", json!([]));
        c["affects_skills"] = json!(["skills/predicted/SKILL.md"]);
        write_cell_fixture(root, "sd-4", &c);

        // Touches a DIFFERENT skills/** path than predicted.
        let flags = cap_flags_sync("sd-4", vec!["skills/other/SKILL.md"], None);
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(refusal.contains("SYNC_DOOR"), "{refusal}");
        assert!(
            refusal.contains("skills/other/SKILL.md") && refusal.contains("skills/predicted/SKILL.md"),
            "must name both sides of the diff: {refusal}"
        );

        // Touching exactly the predicted path clears it.
        let flags_ok = cap_flags_sync("sd-4", vec!["skills/predicted/SKILL.md"], None);
        let capped = cap_cell_from_flags(root, &flags_ok, false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
    }

    // wgg-1: check (c)'s COMPARISON is unchanged; only its wording grows.
    // A prediction written as a bare skill name can never match a touched
    // path, so the refusal names it as the input error it is and prints the
    // path that would have matched — belt and braces for cells written
    // before `cells add` began refusing the format.
    #[test]
    fn a_bare_skill_name_prediction_is_named_as_a_format_error() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        std::fs::create_dir_all(root.join("skills").join("bee-reviewing")).unwrap();
        std::fs::write(root.join("skills").join("bee-reviewing").join("SKILL.md"), "# skill\n").unwrap();
        let mut c = cell("sd-5", "claimed", "f", json!([]));
        c["affects_skills"] = json!(["bee-reviewing"]);
        write_cell_fixture(root, "sd-5", &c);

        let flags = cap_flags_sync("sd-5", vec!["skills/bee-reviewing/SKILL.md"], None);
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(
            refusal.contains(
                "predicted but untouched: bee-reviewing (a bare skill name, not a path — use \"skills/bee-reviewing/SKILL.md\")"
            ),
            "the refusal must name the format error and the path: {refusal}"
        );
        // The comparison itself did not move: the touched path is still
        // reported as unpredicted, on the same refusal.
        assert!(
            refusal.contains("touched but unpredicted: skills/bee-reviewing/SKILL.md"),
            "{refusal}"
        );
    }

    #[test]
    fn legacy_cell_without_affects_skills_skips_prediction_and_notes_the_skip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        // No affects_skills key at all — predates koh-5.
        write_cell_fixture(root, "sd-5", &cell("sd-5", "claimed", "f", json!([])));

        let flags = cap_flags_sync("sd-5", vec!["skills/whatever/SKILL.md"], None);
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(capped["status"], json!("capped"), "legacy cell is never refused on (c)");
        assert_eq!(
            capped["trace"]["deviations"],
            json!(["sync: no prediction on legacy cell"]),
            "the skip is made visible when the touched set actually carried a skills/** path"
        );
    }

    #[test]
    fn a_cell_touching_no_owned_path_caps_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_area_overview(root, "demo", &["src/demo/*"], &["skills/demo/SKILL.md"]);
        write_cell_fixture(root, "sd-6", &cell("sd-6", "claimed", "f", json!([])));

        // Touches neither the area's code nor any skills/** path — the door
        // has nothing to say, and no legacy line appears either (nothing
        // predicted, nothing touched under skills/).
        let flags = cap_flags_sync("sd-6", vec!["src/unrelated/thing.rs"], None);
        let capped = cap_cell_from_flags(root, &flags, false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
        assert_eq!(capped["trace"]["deviations"], json!([]));
    }

    #[test]
    fn no_resolvable_commit_falls_back_to_files_alone() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // No git repo at all in this tempdir — head_commit_numstat must
        // return None, so the touched set is --files alone.
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        write_area_overview(root, "demo", &["src/demo/*"], &["skills/demo/SKILL.md"]);
        write_cell_fixture(root, "sd-7", &cell("sd-7", "claimed", "f", json!([])));

        let flags = cap_flags_sync("sd-7", vec!["src/demo/thing.rs"], None);
        let refusal = thrown(cap_cell_from_flags(root, &flags, false));
        assert!(
            refusal.contains("SYNC_DOOR") && refusal.contains("skills/demo/SKILL.md"),
            "the refusal must still fire off --files alone: {refusal}"
        );
    }

    #[test]
    fn update_can_backfill_affects_skills_on_an_old_cell() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        bp28_repo(root);
        // `cells_update_behavior_child` (above) hardcodes `--id c-1` — the
        // fixture id must match it, same as every other test on this child.
        write_cell_fixture(root, "c-1", &cell("c-1", "open", "f", json!([])));

        let out = cells_update_behavior_run(
            root,
            &json!({"affects_skills": ["skills/demo/SKILL.md"], "affects_specs": []}),
        );
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let updated = read_cell(root, "c-1").unwrap().unwrap();
        assert_eq!(updated["affects_skills"], json!(["skills/demo/SKILL.md"]));
        assert_eq!(updated["affects_specs"], json!([]));
    }

    // wgg-1: the backfill road runs the SAME format door `cells add` runs —
    // a bare skill name cannot be smuggled in through `cells update`.
    #[test]
    fn update_refuses_a_bare_skill_name_in_affects_skills() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        bp28_repo(root);
        std::fs::create_dir_all(root.join("skills").join("demo")).unwrap();
        std::fs::write(root.join("skills").join("demo").join("SKILL.md"), "# demo\n").unwrap();
        write_cell_fixture(root, "c-1", &cell("c-1", "open", "f", json!([])));

        // Same child, spawned with --nocapture: libtest swallows a passing
        // child's output, and the refusal IS the output under test here.
        std::fs::write(
            root.join("patch.json"),
            jsjson::stringify_pretty(&json!({"affects_skills": ["demo"], "affects_specs": []})),
        )
        .unwrap();
        let exe = std::env::current_exe().expect("test binary path");
        let mut cmd = std::process::Command::new(&exe);
        cmd.args([
            "--exact",
            CELLS_UPDATE_BEHAVIOR_CHILD,
            "--ignored",
            "--test-threads",
            "1",
            "--nocapture",
        ]);
        cmd.current_dir(root);
        let out = cmd.output().expect("spawn the test binary");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            text.contains("updateCell: \\\"affects_skills\\\" entry \\\"demo\\\" is not a repo-relative path"),
            "must refuse the bare name: {text}"
        );
        assert!(
            text.contains("use \\\"skills/demo/SKILL.md\\\" instead."),
            "must name the exact replacement path: {text}"
        );
        assert!(text.contains("The whole patch is refused; the cell is untouched."), "{text}");

        // Nothing was written: the cell still has no affects_skills at all.
        let untouched = read_cell(root, "c-1").unwrap().unwrap();
        assert!(untouched.get("affects_skills").is_none(), "{untouched}");
    }

    // ══ merge-ready-fact — the stored merge_ready fact ═════════════════════
    //
    // D1's four writers live in workflow_store/merge_ready.rs and carry
    // their own unit tests over the record seam. What is pinned HERE is the
    // WIRING: the cap that leaves nothing outstanding is the ONE writer,
    // every reopen door removes the fact, and neither can change the verb's
    // own result.

    fn merge_ready_cap_flags(id: &str) -> CapFlags {
        CapFlags {
            id: id.to_string(),
            outcome: None,
            friction: None,
            files_changed: Vec::new(),
            deviations: Vec::new(),
            deviation: None,
            override_reason: String::new(),
            session_flag: None,
            force_ownership: false,
            commit_pending: None,
            inline_reason: None,
            report: Some(default_test_report_json()),
            sync_ack: None,
        }
    }

    /// A main checkout with one GRANTED linked worktree whose identity names
    /// `feature`. `find_granted_worktree_for_feature` validates the gitdir
    /// link in BOTH directions, so this has to be a real `git worktree add`
    /// — hand-written link files never resolve. Returns main's root, which
    /// doubles as the cell STORE root exactly as the cap door uses it (`cells
    /// finish` always runs at main's store, so `list_cells` applies no island
    /// scope here), and the worktree id.
    fn merge_ready_granted_worktree(tmp: &Path, feature: &str) -> (PathBuf, String) {
        let main = tmp.join("main");
        std::fs::create_dir_all(&main).unwrap();
        bp28_repo(&main);
        write_bee_config(&main, &json!({"commands": {"test": "none"}}));
        std::fs::write(main.join("f.txt"), "x").unwrap();
        git_ok(&main, &["init", "-q", "-b", "main", "."]);
        git_ok(&main, &["config", "user.email", "a@b.c"]);
        git_ok(&main, &["config", "user.name", "t"]);
        git_ok(&main, &["add", "-A"]);
        git_ok(&main, &["commit", "-qm", "init"]);
        let id = format!("wt-{feature}");
        let worktree = tmp.join(&id);
        git_ok(
            &main,
            &["worktree", "add", "-q", worktree.to_str().unwrap(), "-b", &format!("wt/{feature}")],
        );
        let mut grants = Map::new();
        grants.insert(id.clone(), json!(true));
        let runtime = main.join(".bee").join("runtime");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::write(
            runtime.join("worktree-grants.json"),
            jsjson::stringify_pretty(&Value::Object(grants)),
        )
        .unwrap();
        let wt_runtime = worktree.join(".bee").join("runtime");
        std::fs::create_dir_all(&wt_runtime).unwrap();
        std::fs::write(
            wt_runtime.join("worktree-identity.json"),
            jsjson::stringify_pretty(&json!({"feature": feature})),
        )
        .unwrap();
        (main, id)
    }

    /// The feature's lane record — the record the fact lands on.
    fn merge_ready_lane(root: &Path, feature: &str) {
        let dir = root.join(".bee").join("lanes");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{feature}.json")),
            jsjson::stringify_pretty(
                &json!({"schema_version": "1.0", "feature": feature, "phase": "swarming"}),
            ),
        )
        .unwrap();
    }

    /// What the feature record on DISK carries — `null` for "no fact".
    fn merge_ready_on_lane(root: &Path, feature: &str) -> Value {
        let raw = std::fs::read_to_string(
            root.join(".bee").join("lanes").join(format!("{feature}.json")),
        )
        .unwrap();
        let lane: Value = serde_json::from_str(&raw).unwrap();
        lane.get("merge_ready").cloned().unwrap_or(Value::Null)
    }

    #[test]
    fn merge_ready_is_set_by_the_cap_that_leaves_nothing_outstanding() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, wt_id) = merge_ready_granted_worktree(tmp.path(), "demo");
        merge_ready_lane(&root, "demo");
        write_cell_fixture(&root, "mr-1", &cell("mr-1", "capped", "demo", json!([])));
        write_cell_fixture(&root, "mr-2", &cell("mr-2", "claimed", "demo", json!([])));

        let capped = cap_cell_from_flags(&root, &merge_ready_cap_flags("mr-2"), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
        let fact = capped["merge_ready"].clone();
        assert_eq!(fact["branch"], json!("wt/demo"), "the worktree's real branch: {fact}");
        assert_eq!(fact["worktree_id"], json!(wt_id));
        assert_eq!(fact["uat"], json!("pending"), "no uat gate approved yet");
        assert_eq!(fact["blocked_by"], json!([]));
        assert!(fact["since"].is_string(), "the wait is stamped: {fact}");
        // The cap RESULT carries it, and so does the feature record on disk.
        assert_eq!(merge_ready_on_lane(&root, "demo"), fact);
        // The CELL file never grew the key — the fact belongs to the feature.
        assert!(read_cell_fixture(&root, "mr-2").get("merge_ready").is_none());
    }

    #[test]
    fn merge_ready_stays_unset_while_a_sibling_cell_is_still_open() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, _wt_id) = merge_ready_granted_worktree(tmp.path(), "demo");
        merge_ready_lane(&root, "demo");
        write_cell_fixture(&root, "mr-1", &cell("mr-1", "claimed", "demo", json!([])));
        write_cell_fixture(&root, "mr-2", &cell("mr-2", "open", "demo", json!([])));

        let capped = cap_cell_from_flags(&root, &merge_ready_cap_flags("mr-1"), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
        assert_eq!(
            capped["merge_ready"],
            json!(null),
            "work is still outstanding — the feature is not ready to merge"
        );
        assert_eq!(merge_ready_on_lane(&root, "demo"), json!(null));
    }

    #[test]
    fn merge_ready_stays_unset_without_a_worktree_grant() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_bee_config(root, &json!({"commands": {"test": "none"}}));
        merge_ready_lane(root, "demo");
        write_cell_fixture(root, "mr-1", &cell("mr-1", "claimed", "demo", json!([])));

        let capped = cap_cell_from_flags(root, &merge_ready_cap_flags("mr-1"), false).unwrap();
        assert_eq!(capped["status"], json!("capped"));
        assert_eq!(
            capped["merge_ready"],
            json!(null),
            "the fact names a branch and a worktree — with no grant there is neither"
        );
        assert_eq!(merge_ready_on_lane(root, "demo"), json!(null));
    }

    /// The fail-open promise, at the door that matters most: the cap has
    /// already written the cell and released the claim by the time the fact
    /// is attempted, so a record it cannot read must cost the cap nothing.
    #[test]
    fn merge_ready_over_a_corrupt_record_leaves_the_cap_result_intact() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, _wt_id) = merge_ready_granted_worktree(tmp.path(), "demo");
        let lane_file = root.join(".bee").join("lanes").join("demo.json");
        std::fs::create_dir_all(lane_file.parent().unwrap()).unwrap();
        std::fs::write(&lane_file, "{not json").unwrap();
        write_cell_fixture(&root, "mr-1", &cell("mr-1", "claimed", "demo", json!([])));

        let capped = cap_cell_from_flags(&root, &merge_ready_cap_flags("mr-1"), false)
            .expect("a record the fact cannot read never turns a landed cap into a refusal");
        assert_eq!(capped["status"], json!("capped"));
        assert_eq!(capped["merge_ready"], json!(null));
        assert_eq!(
            read_cell_fixture(&root, "mr-1")["status"],
            json!("capped"),
            "the cap itself landed on disk"
        );
        assert_eq!(
            std::fs::read_to_string(&lane_file).unwrap(),
            "{not json",
            "the corrupt record is left exactly as found"
        );
    }

    /// D2: a reopen un-finishes the feature, and the NEXT last-cap starts the
    /// wait over rather than resurrecting the old `since`.
    #[test]
    fn merge_ready_is_cleared_by_a_reopen_and_re_set_with_a_fresh_since() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, _wt_id) = merge_ready_granted_worktree(tmp.path(), "demo");
        merge_ready_lane(&root, "demo");
        // `cells_reopen_behavior_child` hardcodes `--id x1`, so the fixture
        // id must match it — the same coupling every other test on that
        // child carries.
        write_cell_fixture(&root, "x1", &cell("x1", "claimed", "demo", json!([])));

        let first = cap_cell_from_flags(&root, &merge_ready_cap_flags("x1"), false).unwrap();
        let since_1 = first["merge_ready"]["since"]
            .as_str()
            .expect("the last cap set the fact")
            .to_string();

        let out = cells_reopen_behavior_run(&root);
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(read_cell_fixture(&root, "x1")["status"], json!("open"));
        assert_eq!(
            merge_ready_on_lane(&root, "demo"),
            json!(null),
            "an open cell means the feature is no longer ready to merge"
        );

        let second = cap_cell_from_flags(&root, &merge_ready_cap_flags("x1"), false).unwrap();
        let since_2 = second["merge_ready"]["since"]
            .as_str()
            .expect("the next last-cap sets it again")
            .to_string();
        assert!(since_2 > since_1, "the wait restarts: {since_1} -> {since_2}");
    }

    /// D2 at the third reopen door: `cells unclaim` takes a claimed cell back
    /// to open. It can only ever fire for a feature whose fact was set by an
    /// EARLIER cap and then re-claimed, which is exactly this fixture.
    #[test]
    fn merge_ready_is_cleared_by_an_unclaim() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, _wt_id) = merge_ready_granted_worktree(tmp.path(), "demo");
        merge_ready_lane(&root, "demo");
        write_cell_fixture(&root, "mr-1", &cell("mr-1", "claimed", "demo", json!([])));
        let capped = cap_cell_from_flags(&root, &merge_ready_cap_flags("mr-1"), false).unwrap();
        assert!(capped["merge_ready"]["since"].is_string(), "the fact was set first");

        // A follow-up cell claimed after the fact was written, then handed
        // back: the feature has open work again.
        write_cell_fixture(&root, "mr-2", &cell("mr-2", "claimed", "demo", json!([])));
        unclaim_cell(&root, "mr-2", None, true).unwrap();
        assert_eq!(read_cell_fixture(&root, "mr-2")["status"], json!("open"));
        assert_eq!(merge_ready_on_lane(&root, "demo"), json!(null));
    }

    const MERGE_READY_JUDGE_CHILD: &str = "verbs::cells::tests::merge_ready_judge_record_child";

    /// Runs ONLY as a child of the test below — records the NEEDS_REVISION
    /// verdict its parent left at `verdict.json` in its cwd, against cell
    /// "mrj-1", through the REAL `cells judge-record` CLI door. The capped
    /// -> open flip that verdict performs lives inside that door's dispatch
    /// closure, which has no library-level seam taking an explicit root.
    #[test]
    #[ignore = "spawned by merge_ready_is_cleared_by_a_needs_revision_judge_record"]
    fn merge_ready_judge_record_child() {
        let (flags, use_json) = rsv::parse_flags(&["--id", "mrj-1", "--file", "verdict.json"])
            .expect("well-formed fixture argv");
        run_judge_record(flags, use_json, Instant::now());
    }

    #[test]
    fn merge_ready_is_cleared_by_a_needs_revision_judge_record() {
        let tmp = tempfile::tempdir().unwrap();
        let (root, _wt_id) = merge_ready_granted_worktree(tmp.path(), "demo");
        merge_ready_lane(&root, "demo");
        write_cell_fixture(&root, "mrj-1", &cell("mrj-1", "claimed", "demo", json!([])));
        let capped = cap_cell_from_flags(&root, &merge_ready_cap_flags("mrj-1"), false).unwrap();
        assert!(capped["merge_ready"]["since"].is_string(), "the fact was set first");

        std::fs::write(
            root.join("verdict.json"),
            jsjson::stringify_pretty(&json!({
                "schema": "judge-verdict/1",
                "verdict": "NEEDS_REVISION",
                "checks": [{"id": "t1", "status": "FAIL", "evidence": "the truth is not met"}],
                "fixability": "automatic",
                "confidence": "high",
                "failure_signature": "truth-missing",
            })),
        )
        .unwrap();

        let exe = std::env::current_exe().expect("test binary path");
        let out = std::process::Command::new(&exe)
            .args(["--exact", MERGE_READY_JUDGE_CHILD, "--ignored", "--test-threads", "1"])
            .current_dir(&root)
            .output()
            .expect("spawn the test binary");
        assert!(
            out.status.success(),
            "child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            read_cell_fixture(&root, "mrj-1")["status"],
            json!("open"),
            "the verdict reopened the cell"
        );
        assert_eq!(
            merge_ready_on_lane(&root, "demo"),
            json!(null),
            "a reopen is a reopen, whichever door performs it"
        );
    }

    // ── cells backfill-roles (model-role-split D9, store 4eaf1b71) ────────

    /// Every stored cell file's exact bytes, keyed by path. Byte-level on
    /// purpose: "the store did not change" is the claim these tests make,
    /// and a parsed comparison would hide a reordered key or a rewritten
    /// separator that a `git diff` on 500 files would not.
    fn store_bytes(root: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
        let mut out = std::collections::BTreeMap::new();
        for file in stored_cell_files(root) {
            out.insert(file.to_string_lossy().into_owned(), std::fs::read(&file).unwrap());
        }
        out
    }

    fn write_archived_cell_fixture(root: &Path, feature: &str, id: &str, body: &Value) {
        let dir = cells_archive_dir(root, feature);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), jsjson::stringify_pretty(body)).unwrap();
    }

    fn roleless(id: &str, tier: Value) -> Value {
        let mut cell = json!({"id": id, "title": id, "status": "capped", "lane": "tiny", "feature": "demo"});
        if !tier.is_null() {
            cell.as_object_mut().unwrap().insert("tier".into(), tier);
        }
        cell
    }

    /// A store whose shape is deliberately NOT the 484 / 2 / 20 the decision
    /// measured on 2026-08-24: 11 cells — 3 `generation`, 3 no-tier,
    /// 3 `ceiling`, 1 `extraction`, and 1 that already carries a role. If any
    /// count in the verb were remembered rather than measured, this store is
    /// where it shows.
    fn backfill_store() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(cells_dir(&root)).unwrap();
        for id in ["g-1", "g-2", "g-3"] {
            write_cell_fixture(&root, id, &roleless(id, json!("generation")));
        }
        for id in ["n-1", "n-2"] {
            write_cell_fixture(&root, id, &roleless(id, Value::Null));
        }
        for id in ["c-1", "c-2"] {
            write_cell_fixture(&root, id, &roleless(id, json!("ceiling")));
        }
        write_cell_fixture(&root, "x-1", &roleless("x-1", json!("extraction")));
        // Already roled, and roled AGAINST what D9 would have picked for its
        // tier ("design" on a `generation` cell, where D9 says "code") —
        // which is what proves the verb reads the role that is there rather
        // than re-deriving one over the top of it.
        let mut roled = roleless("r-1", json!("generation"));
        roled.as_object_mut().unwrap().insert("role".into(), json!("design"));
        write_cell_fixture(&root, "r-1", &roled);
        // Archived history is stored history — D9 says "the stored cells".
        write_archived_cell_fixture(&root, "old-feature", "a-1", &roleless("a-1", json!("ceiling")));
        write_archived_cell_fixture(&root, "old-feature", "a-2", &roleless("a-2", Value::Null));
        (tmp, root)
    }

    #[test]
    fn d9_maps_every_legal_tier_and_refuses_to_guess_at_any_other() {
        // The three legal tiers plus the absent one — D9's whole mapping.
        assert_eq!(d9_role_for_tier(Some("generation")), Some("code"));
        assert_eq!(d9_role_for_tier(None), Some("code"));
        assert_eq!(d9_role_for_tier(Some("")), Some("code"));
        assert_eq!(d9_role_for_tier(Some("  ")), Some("code"), "a blank tier is no tier");
        assert_eq!(d9_role_for_tier(Some("ceiling")), Some("code"));
        assert_eq!(d9_role_for_tier(Some("extraction")), Some("read"));
        // The deliberate hole. A value outside the legal three is data this
        // mapping has no answer for, and "code" is not a safe guess — it is
        // the silent default D7 exists to end.
        assert_eq!(d9_role_for_tier(Some("premium")), None);
        assert_eq!(d9_role_for_tier(Some("Generation")), None, "the match is exact, not fuzzy");
        // The source labels line up with the mapping, entry for entry.
        for (source, role) in ROLE_BACKFILL_SOURCES {
            let tier = match source {
                "no-tier" => None,
                other => Some(other.trim_start_matches("tier:")),
            };
            assert_eq!(d9_source_for_tier(tier), source, "{source}");
            assert_eq!(d9_role_for_tier(tier), Some(role), "{source}");
        }
    }

    #[test]
    fn backfill_dry_run_reports_its_counts_and_writes_nothing() {
        let (_tmp, root) = backfill_store();
        let before = store_bytes(&root);
        let report = backfill_roles(&root, true).expect("dry run must not refuse");

        // Every number is measured off THIS store: 11 files, 1 already
        // roled, 10 to assign. Nothing here is the decision's 484 / 2 / 20.
        assert_eq!(report.scanned, 11);
        assert_eq!(report.already_roled, 1);
        assert_eq!(report.assigned, 10);
        assert_eq!(report.written, 0, "--dry-run writes nothing, so `written` must stay 0");
        assert_eq!(report.by_source, [3, 3, 3, 1], "generation / no-tier / ceiling / extraction");
        assert_eq!(report.by_role(), vec![("code", 9), ("read", 1)]);
        // D5: the three `ceiling` cells (two live, one archived) are the ones
        // that also take the escalation flag.
        assert_eq!(report.escalated, 3);
        assert!(report.unmapped.is_empty());
        assert!(report.unreadable.is_empty());

        assert_eq!(store_bytes(&root), before, "--dry-run must leave every byte of the store alone");
        let text = role_backfill_text(&report, true);
        assert!(text.contains("11 stored cell(s) scanned"), "{text}");
        assert!(text.contains("10 would take a role"), "{text}");
        assert!(text.contains("Nothing was written"), "{text}");
        let obj = role_backfill_json(&report, true);
        assert_eq!(obj["dry_run"], json!(true));
        assert_eq!(obj["written"], json!(0));
        assert_eq!(obj["by_source"]["tier:extraction"], json!(1));
        assert_eq!(obj["by_role"]["read"], json!(1));
    }

    #[test]
    fn backfill_applies_d9_and_converts_ceiling_onto_the_escalation_flag() {
        let (_tmp, root) = backfill_store();
        let report = backfill_roles(&root, false).expect("apply must not refuse");
        assert_eq!(report.assigned, 10);
        assert_eq!(report.written, 10, "assigned and written agree once it is applied");

        for id in ["g-1", "g-2", "g-3", "n-1", "n-2", "c-1", "c-2"] {
            assert_eq!(read_cell_fixture(&root, id)["role"], json!("code"), "{id}");
        }
        assert_eq!(read_cell_fixture(&root, "x-1")["role"], json!("read"));
        // D5 (store `97ce5225`): a stored `tier: "ceiling"` is converted onto
        // the escalation flag in this same pass — flag and role together, so
        // no store ever answers half of one and half of the other.
        for id in ["c-1", "c-2"] {
            assert_eq!(read_cell_fixture(&root, id)["escalate"], json!(true), "{id}");
        }
        assert_eq!(report.escalated, 3, "two live ceilings plus the archived one");
        // The tier STRING survives beside the flag, and D4 (store `97ce5225`)
        // keeps it that way: retiring `tier` as the model SELECTOR is not an
        // order to rewrite stored history, and `cell_is_escalated` still
        // reads the one value that ever meant anything.
        for id in ["c-1", "c-2"] {
            assert_eq!(read_cell_fixture(&root, id)["tier"], json!("ceiling"), "{id}");
        }
        // A cell that was never `ceiling` takes no flag: absent stays absent.
        for id in ["g-1", "n-1", "x-1"] {
            assert!(read_cell_fixture(&root, id).get("escalate").is_none(), "{id}");
        }
        assert_eq!(read_cell_fixture(&root, "g-1")["tier"], json!("generation"));
        assert_eq!(read_cell_fixture(&root, "x-1")["tier"], json!("extraction"));
        assert!(
            read_cell_fixture(&root, "n-1").get("tier").is_none(),
            "a cell that recorded no tier must not acquire one"
        );
        // The role is the ONLY new key: everything else is byte-for-byte the
        // value the store already held.
        let migrated = read_cell_fixture(&root, "c-1");
        assert_eq!(migrated["status"], json!("capped"));
        assert_eq!(migrated["lane"], json!("tiny"));
        assert_eq!(migrated["feature"], json!("demo"));

        // Archived history is migrated too.
        let archived = |id: &str| {
            let file = cells_archive_dir(&root, "old-feature").join(format!("{id}.json"));
            match read_json(&file) {
                ReadJson::Parsed(v) => v,
                _ => panic!("archived cell {id} is missing or corrupt at {}", file.display()),
            }
        };
        assert_eq!(archived("a-1")["role"], json!("code"));
        assert_eq!(archived("a-1")["tier"], json!("ceiling"), "an archived ceiling keeps its tier too");
        assert_eq!(
            archived("a-1")["escalate"],
            json!(true),
            "and an archived ceiling is converted too — `cells unarchive` brings it back live"
        );
        assert_eq!(archived("a-2")["role"], json!("code"));
    }

    /// D4 (store `97ce5225`) — the third job this one pass carries: the
    /// escalation reason moved off the retired selector's name, so a stored
    /// trace still spelling it `tier_reason` is renamed to
    /// `escalation_reason` here, VALUE untouched.
    ///
    /// Its own store rather than `backfill_store()`, so the counts above stay
    /// the counts they were measured as. Four shapes in one pass: a legacy
    /// key alone, a legacy key on a cell that ALSO needs a role and the flag,
    /// a trace already migrated (which must not be touched or overwritten),
    /// and a trace carrying neither.
    #[test]
    fn backfill_renames_the_retired_escalation_reason_key() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();

        let with_trace = |id: &str, tier: Value, trace: Value, role: Option<&str>| {
            let mut c = roleless(id, tier);
            let map = c.as_object_mut().unwrap();
            map.insert("trace".into(), trace);
            if let Some(r) = role {
                map.insert("role".into(), json!(r));
            }
            c
        };
        // Already roled, so the rename is the ONLY reason this file is opened.
        write_cell_fixture(
            root,
            "legacy-only",
            &with_trace("legacy-only", Value::Null, json!({"tier_reason": "owner said so"}), Some("code")),
        );
        // Needs all three at once.
        write_cell_fixture(
            root,
            "legacy-all",
            &with_trace("legacy-all", json!("ceiling"), json!({"tier_reason": "rescue ladder"}), None),
        );
        // Already migrated: nothing to do, and its current reason must not be
        // clobbered by a stale legacy key sitting beside it.
        write_cell_fixture(
            root,
            "already",
            &with_trace(
                "already",
                Value::Null,
                json!({"escalation_reason": "current", "tier_reason": "stale"}),
                Some("code"),
            ),
        );
        write_cell_fixture(
            root,
            "no-reason",
            &with_trace("no-reason", Value::Null, json!({"worker": "w-1"}), Some("code")),
        );

        // Dry run measures it and writes nothing.
        let before = store_bytes(root);
        let dry = backfill_roles(root, true).unwrap();
        assert_eq!(dry.reasons_renamed, 2, "legacy-only and legacy-all, never `already`");
        assert_eq!(store_bytes(root), before, "--dry-run writes nothing");
        let dry_text = role_backfill_text(&dry, true);
        assert!(dry_text.contains("trace.tier_reason"), "{dry_text}");
        assert!(dry_text.contains("would be renamed"), "{dry_text}");
        assert_eq!(role_backfill_json(&dry, true)["reasons_renamed"], json!(2));

        let report = backfill_roles(root, false).unwrap();
        assert_eq!(report.reasons_renamed, 2);

        let migrated = read_cell_fixture(root, "legacy-only");
        assert_eq!(migrated["trace"]["escalation_reason"], json!("owner said so"));
        assert!(
            migrated["trace"].get("tier_reason").is_none(),
            "the retired key is removed, not left beside its replacement"
        );

        let all = read_cell_fixture(root, "legacy-all");
        assert_eq!(all["role"], json!("code"), "the role still lands");
        assert_eq!(all["escalate"], json!(true), "and so does the flag");
        assert_eq!(all["trace"]["escalation_reason"], json!("rescue ladder"));
        assert!(all["trace"].get("tier_reason").is_none());

        // An already-migrated trace keeps the reason it holds.
        let already = read_cell_fixture(root, "already");
        assert_eq!(already["trace"]["escalation_reason"], json!("current"));

        // Idempotent: a second pass finds nothing left to rename.
        let second = backfill_roles(root, false).unwrap();
        assert_eq!(second.reasons_renamed, 0);
        assert_eq!(second.written, 0, "and opens no file for writing");
    }

    #[test]
    fn a_cell_that_already_carries_a_role_is_left_byte_identical() {
        let (_tmp, root) = backfill_store();
        let file = cell_file(&root, "r-1");
        let before = std::fs::read(&file).unwrap();
        let report = backfill_roles(&root, false).unwrap();
        assert_eq!(report.already_roled, 1);
        assert_eq!(
            std::fs::read(&file).unwrap(),
            before,
            "r-1 already carried \"design\"; the migration must not open it for writing at all"
        );
        assert_eq!(
            read_cell_fixture(&root, "r-1")["role"],
            json!("design"),
            "an existing role is never re-derived from tier"
        );
    }

    /// The exact record shape the role-only pass left behind: a cell that
    /// ALREADY carries a role and still records `tier: "ceiling"`, because
    /// the escalation flag did not exist when its role was written. Having a
    /// role is not having been converted — this cell is opened for writing,
    /// its role is left alone, and it takes the flag.
    #[test]
    fn a_roled_ceiling_cell_is_still_converted_onto_the_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(cells_dir(&root)).unwrap();
        let mut leftover = roleless("k-1", json!("ceiling"));
        leftover.as_object_mut().unwrap().insert("role".into(), json!("code"));
        write_cell_fixture(&root, "k-1", &leftover);

        let report = backfill_roles(&root, false).unwrap();
        assert_eq!(report.already_roled, 1, "it does carry a role");
        assert_eq!(report.assigned, 0, "so no role is assigned");
        assert_eq!(report.escalated, 1, "but the escalation is still converted");
        assert_eq!(report.written, 1);
        let after = read_cell_fixture(&root, "k-1");
        assert_eq!(after["escalate"], json!(true));
        assert_eq!(after["role"], json!("code"), "the role it already had is never re-derived");

        // And it is done: a second pass opens nothing.
        assert_eq!(backfill_roles(&root, false).unwrap().written, 0);
    }

    /// The property the plan asks for by name: run it, run it again, and the
    /// second run changes nothing. Byte-identity, not prose.
    #[test]
    fn backfill_is_idempotent_down_to_the_byte() {
        let (_tmp, root) = backfill_store();
        let first = backfill_roles(&root, false).unwrap();
        assert_eq!(first.written, 10);
        let after_first = store_bytes(&root);

        let second = backfill_roles(&root, false).unwrap();
        assert_eq!(second.scanned, 11, "the second pass still scans the whole store");
        assert_eq!(second.assigned, 0, "there is nothing left to assign");
        assert_eq!(second.written, 0, "and therefore nothing to write");
        assert_eq!(second.already_roled, 11, "every cell now carries a role");
        assert_eq!(second.by_source, [0, 0, 0, 0]);
        assert_eq!(store_bytes(&root), after_first, "the second run must change nothing");

        // A dry run over an already-migrated store agrees, and still writes
        // nothing.
        let third = backfill_roles(&root, true).unwrap();
        assert_eq!((third.assigned, third.written, third.already_roled), (0, 0, 11));
        assert_eq!(store_bytes(&root), after_first);
    }

    /// An interrupted first pass is finished by running again — the cells
    /// already done are indistinguishable from cells authored with a role.
    #[test]
    fn a_partially_migrated_store_is_finished_by_a_re_run() {
        let (_tmp, root) = backfill_store();
        // Stand in for the interrupted run: two cells landed, the rest did not.
        for id in ["g-1", "n-1"] {
            let mut cell = read_cell_fixture(&root, id);
            cell.as_object_mut().unwrap().insert("role".into(), json!("code"));
            write_json_atomic(&cell_file(&root, id), &cell).unwrap();
        }
        let done_bytes = std::fs::read(cell_file(&root, "g-1")).unwrap();
        let report = backfill_roles(&root, false).unwrap();
        assert_eq!(report.already_roled, 3, "r-1 plus the two the interrupted run finished");
        assert_eq!(report.written, 8, "only the remainder is written");
        assert_eq!(
            std::fs::read(cell_file(&root, "g-1")).unwrap(),
            done_bytes,
            "a cell the interrupted run already finished is not touched a second time"
        );
        assert_eq!(backfill_roles(&root, false).unwrap().written, 0);
    }

    // ── The unlocked scan (review r2, P1-B) ───────────────────────────────
    //
    // What these close: the pass took the `cells-archive` lock for its WRITE
    // half only, and every write was a whole object cloned during the
    // unlocked scan. Demonstrated on a 40 000-cell store — a
    // `cells escalate --off` that COMPLETED at t=25ms was back on disk as
    // `escalate: true` when the pass finished at t=1.007s. Not a half-write:
    // a complete write, completely reversed. Only writers that finished
    // inside the scan were lost, which is why casual testing never saw it.
    //
    // Why the seam and not a thread: every backfill test above is
    // single-threaded, and the whole suite passed with `acquire_named_lock`
    // DELETED. A sleep-and-race test would swap that silence for flakiness.
    // `backfill_roles_interleaved` hands the test the window itself — plan
    // built, lock not yet taken — so the interleaving is decided by the
    // test, not by the machine.

    /// `backfill_store` plus `e-1`, in the demonstration's exact shape: no
    /// role (so the pass DOES plan a write for it) and the escalation flag
    /// already set (so the flag is NOT part of what the pass plans to
    /// change). That combination is what makes the reversal invisible — the
    /// pass never meant to touch the flag, and reversed it anyway.
    fn escalated_backfill_store() -> (tempfile::TempDir, PathBuf) {
        let (tmp, root) = backfill_store();
        let mut armed = roleless("e-1", json!("ceiling"));
        armed.as_object_mut().unwrap().insert(ESCALATE_FIELD.into(), json!(true));
        write_cell_fixture(&root, "e-1", &armed);
        (tmp, root)
    }

    #[test]
    fn an_operator_write_that_lands_before_the_lock_is_never_reversed() {
        let (_tmp, root) = escalated_backfill_store();
        let disarm_root = root.clone();
        let report = backfill_roles_interleaved(&root, false, || {
            // A real operator door, in the window the review measured: the
            // plan is built and the lock is not held, so this write is
            // ALLOWED — exactly why the defect only ever lost the writers
            // that completed.
            set_escalation(&disarm_root, "e-1", false, None)
                .expect("the disarm lands while the pass is still unlocked");
        })
        .expect("the pass itself must not refuse");

        let after = read_cell_fixture(&root, "e-1");
        assert!(
            after.get(ESCALATE_FIELD).is_none(),
            "the operator's disarm must survive: the write half re-reads under the lock and \
             merges the keys it owns, instead of restoring a clone taken before the disarm \
             existed — got {after}"
        );
        assert_eq!(
            after["role"],
            json!("code"),
            "and the migration still adds the one key it did plan for this cell"
        );
        assert_eq!(report.escalated, 3, "c-1, c-2 and archived a-1 — never e-1, which was already flagged");
        assert!(
            report.changed_during_pass.is_empty(),
            "nothing was dropped: the plan for e-1 (role only) still applied in full — {:?}",
            report.changed_during_pass
        );
    }

    #[test]
    fn a_feature_archived_before_the_lock_is_not_recreated_as_a_live_duplicate() {
        let (_tmp, root) = backfill_store();
        let close_root = root.clone();
        let report = backfill_roles_interleaved(&root, false, || {
            // `bee close`'s own archiving door, unblocked in this window for
            // the same reason the disarm above is.
            archive_feature_for_close(&close_root, "demo").expect("every demo cell is capped");
        })
        .expect("a feature archived under the pass is not an error");

        assert!(
            !cell_file(&root, "g-1").exists(),
            "an archived cell must not be recreated at its live path — readCell prefers the \
             live copy, so a whole-object write here forges a duplicate of archived history"
        );
        assert_eq!(report.written, 2, "only old-feature's archive, which nothing moved");
        assert_eq!(
            report.changed_during_pass.len(),
            8,
            "the eight demo records that moved are named, not written — {:?}",
            report.changed_during_pass
        );
        // Idempotent as ever: the next run finishes them at their new home.
        let second = backfill_roles(&root, false).unwrap();
        assert_eq!(second.written, 8);
        assert!(second.changed_during_pass.is_empty(), "{:?}", second.changed_during_pass);
    }

    /// The lock line itself, which nothing above ever asked about: deleting
    /// `acquire_named_lock` left the whole suite green. Costs one bounded
    /// wait (MAX_ATTEMPTS × RETRY_DELAY_MS) on purpose — the refusal IS the
    /// assertion, and a live holder is never stale-taken inside that window.
    #[test]
    fn the_write_half_refuses_while_the_archive_lock_is_held() {
        let (_tmp, root) = backfill_store();
        let mut held =
            lock::acquire_store_lock(&root, "cells-archive", 1).expect("the store starts unlocked");

        let refusal = thrown(backfill_roles(&root, false));
        assert!(refusal.contains("cells-archive"), "{refusal}");
        assert!(
            read_cell_fixture(&root, "g-1").get("role").is_none(),
            "a refused pass writes nothing at all"
        );

        held.release();
        assert_eq!(
            backfill_roles(&root, false).unwrap().written,
            10,
            "and the same pass goes through once the holder is gone"
        );
    }

    #[test]
    fn an_unmapped_tier_and_an_unreadable_file_are_named_never_guessed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(cells_dir(root)).unwrap();
        write_cell_fixture(root, "ok-1", &roleless("ok-1", json!("generation")));
        write_cell_fixture(root, "odd-1", &roleless("odd-1", json!("premium")));
        std::fs::write(cells_dir(root).join("broken-1.json"), "{not json").unwrap();
        std::fs::write(cells_dir(root).join("array-1.json"), "[]").unwrap();

        let report = backfill_roles(root, false).unwrap();
        assert_eq!(report.scanned, 4);
        assert_eq!(report.assigned, 1);
        assert_eq!(report.written, 1);
        assert_eq!(
            report.unmapped,
            vec![("odd-1".to_string(), "premium".to_string())],
            "a tier D9 does not map is reported by id, not silently defaulted to \"code\""
        );
        assert!(
            read_cell_fixture(root, "odd-1").get("role").is_none(),
            "and the cell itself is left alone"
        );
        let unreadable: Vec<&str> = report.unreadable.iter().map(String::as_str).collect();
        assert_eq!(unreadable.len(), 2, "{unreadable:?}");
        assert!(unreadable.iter().any(|p| p.ends_with(".bee/cells/broken-1.json")), "{unreadable:?}");
        assert!(unreadable.iter().any(|p| p.ends_with(".bee/cells/array-1.json")), "{unreadable:?}");
        let text = role_backfill_text(&report, false);
        assert!(text.contains("odd-1 (tier \"premium\")"), "{text}");
        assert!(text.contains("2 unreadable file(s) skipped"), "{text}");
    }

    #[test]
    fn backfill_over_an_empty_store_reports_zero_and_refuses_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let report = backfill_roles(tmp.path(), true).expect("a store with no cells dir is not an error");
        assert_eq!((report.scanned, report.assigned, report.written), (0, 0, 0));
        // Every source is reported AS zero — an absent row and an empty row
        // must not read the same.
        let obj = role_backfill_json(&report, true);
        for (source, _) in ROLE_BACKFILL_SOURCES {
            assert_eq!(obj["by_source"][source], json!(0), "{source}");
        }
    }

    /// The registry↔dispatcher law, the direction mrs-12 was the other half
    /// of: the verb this cell serves must also be DECLARED, or `bee --help
    /// --all` calls it unknown and the CLI-shape guard has no schema to
    /// check a call against.
    #[test]
    fn backfill_roles_is_declared_in_the_registry() {
        let (entry, rest) = crate::catalog::resolve(&["cells", "backfill-roles"])
            .expect("cells backfill-roles must be in the registry payload");
        assert_eq!(entry.name, "cells.backfill-roles");
        assert!(rest.is_empty(), "{rest:?}");
        assert!(entry.unavailable.is_none(), "the dispatcher serves it, so no unavailable marker");
        assert!(entry.required.is_empty(), "both flags are optional");
        for flag in ["dry-run", "json"] {
            assert!(entry.properties.contains_key(flag), "--{flag} is undeclared");
        }
        assert_eq!(entry.examples[0], "bee cells backfill-roles --dry-run --json");
    }

    /// D4 (store `97ce5225`) — the tier verb is gone, and gone in every
    /// direction the registry↔dispatcher law reaches.
    ///
    /// mrs-12 proved one half: a verb the dispatcher serves and the registry
    /// does not declare is reported unknown by `bee --help --all` and gives
    /// the CLI-shape guard no schema. The inverse is the half this asserts —
    /// a verb the registry still declares and no code serves would be
    /// advertised by `--help` and answered by nothing, which is the exact
    /// defect tests/registry_dispatch.rs was written for.
    #[test]
    fn the_tier_verb_is_retired_and_escalate_stands_in_its_place() {
        assert!(
            crate::catalog::resolve(&["cells", "tier"]).is_none(),
            "cells tier must not be declared: no code serves it any more"
        );
        assert!(
            !crate::catalog::group_subverbs("cells").contains(&"tier".to_string()),
            "and `bee cells` must not advertise it"
        );

        let (entry, rest) = crate::catalog::resolve(&["cells", "escalate"])
            .expect("cells escalate must be in the registry payload");
        assert_eq!(entry.name, "cells.escalate");
        assert!(rest.is_empty(), "{rest:?}");
        assert!(entry.unavailable.is_none(), "the dispatcher serves it, so no unavailable marker");
        assert_eq!(entry.required, vec!["id".to_string()], "only --id is required");
        for flag in ["id", "reason", "off", "json"] {
            assert!(entry.properties.contains_key(flag), "--{flag} is undeclared");
        }
        // No enum anywhere on it: D4 retires the closed three-value list with
        // the selector, and nothing replaces it.
        for (flag, schema) in entry.properties.iter() {
            assert!(schema.get("enum").is_none(), "--{flag} must carry no enum: {schema}");
        }
    }
