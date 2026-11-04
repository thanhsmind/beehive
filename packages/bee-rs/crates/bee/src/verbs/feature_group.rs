use crate::jsjson;
use crate::session_identity::locate_caller;
use crate::verbs::record_timing;
use crate::verbs::reservations::{keys_known, parse_flags, Flags};
use crate::verbs::state_group::{
    acquire_state_lock, ensure_workflow_record_for_feature, start_lane, validate_route_set_flags,
};
use crate::verbs::status_full::find_granted_worktree_for_feature;
use crate::verbs::workflow_store::{
    acquire_named_lock, find_live_workflow, lane_lock_name, list_workflows, read_lane_strict,
    rebuild_lane_projection, update_workflow_assuming_lock, wf_id, write_lane,
};
use crate::verbs::worktree::{
    build_instruction, build_session_transition, create_feature_worktree,
    emit_pi_transition_marker_if_present, feature_slug_ok, p, prelude, CErr, CompanionSpec, Pre,
};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::process::ExitCode;
use std::time::Instant;

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "feature" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    match verb {
        "start" => run_start(&args[2..], t0),
        "finish" => run_finish(&args[2..], t0),
        _ => None,
    }
}

fn run_start(tokens: &[OsString], t0: Instant) -> Option<ExitCode> {
    let strs: Vec<&str> = tokens.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    let (flags, use_json) = parse_flags(&strs)?;
    if !keys_known(&flags, &["feature", "class", "lane", "flags", "files"]) {
        return None;
    }

    let feature = match flags.req_str("feature") {
        Some(f) => f.to_string(),
        None => {
            let msg = "bee feature start: missing required flag: --feature";
            if use_json {
                println!("{}", jsjson::stringify(&json!({"error": msg})));
            } else {
                eprintln!("{msg}");
            }
            return Some(ExitCode::FAILURE);
        }
    };

    if !feature_slug_ok(&feature) {
        let msg = format!(
            "[WORKTREE_INVALID_SLUG] feature slug {} must match /^[a-z0-9][a-z0-9-]*$/ (lowercase letters/digits, starting with a letter or digit, hyphens allowed after that).",
            jsjson::stringify(&Value::String(feature))
        );
        if use_json {
            println!("{}", jsjson::stringify(&json!({"error": msg})));
        } else {
            eprintln!("{msg}");
        }
        return Some(ExitCode::FAILURE);
    }

    let mut route_flags = Flags(Vec::new());
    if let Some(v) = flags.get("class") {
        route_flags.insert("class", v.clone());
    }
    if let Some(v) = flags.get("lane") {
        route_flags.insert("lane", v.clone());
    }
    if let Some(v) = flags.get("files") {
        route_flags.insert("files", v.clone());
    }
    if let Some(v) = flags.get("flags") {
        route_flags.insert("flags", v.clone());
    }

    let mut route_object = match validate_route_set_flags(&route_flags) {
        Ok(Ok(obj)) => obj,
        Ok(Err(msg)) => {
            if use_json {
                println!("{}", jsjson::stringify(&json!({"error": msg})));
            } else {
                eprintln!("{msg}");
            }
            return Some(ExitCode::FAILURE);
        }
        Err(_) => return None,
    };


    let ctx = match prelude("feature start", use_json, t0)? {
        Pre::Go(c) => *c,
        Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&format!(
            "\"bee feature start\" must be run from inside the main checkout, not a \"{}\" checkout — run it from the main repo root, then open your next session inside the created worktree.",
            ctx.kind
        )));
    }
    let main_root = ctx.work_root.clone();

    let main_root_s = p(&main_root);
    if let Ok(ctrl_root) = crate::verbs::reservations::control_root_for(&main_root_s) {
        let session_id = crate::verbs::reservations::resolve_session_id(None, &ctrl_root).ok().flatten();
        match crate::nested_checkout::has_any_shared_nested_checkout(
            &main_root,
            &ctrl_root,
            session_id.as_deref(),
        ) {
            Ok(true) => {
                return Some(ctx.fail(&format!(
                    "refusing to create a worktree: another session is concurrently live on {main_root_s} and it contains a shared nested checkout a companion mount must cover — running unguarded is how one session silently ate another's work. Re-run with \"bee worktree new --feature {feature} --with-companion\" so the shared checkout is mounted and tracked (the paved road for concurrent shared-checkout work — AGENTS.md rule 13). This creates a NEW companion-mounted worktree; it does not retrofit the checkout you are in."
                )));
            }
            Err(detect) => {
                return Some(ctx.fail(&format!(
                    "refusing to create a worktree: could not determine whether {main_root_s} holds a shared nested checkout another live session could reach — the detection check itself errored ({}). This guard fails CLOSED on a detection error rather than risk silently allowing an unguarded worktree. FIX: resolve the underlying filesystem error, then retry.",
                    detect.reason
                )));
            }
            _ => {}
        }
    }

    let mut lock_busy: Option<String> = None;
    let created = match create_feature_worktree(
        &main_root,
        &feature,
        None,
        CompanionSpec::default(),
        &mut lock_busy,
    ) {
        Ok(c) => c,
        Err(CErr::Refuse(msg)) => return Some(ctx.fail(&msg)),
        Err(CErr::Ex) => match lock_busy {
            Some(msg) => return Some(ctx.fail(&msg)),
            None => return None,
        },
    };

    let caller = locate_caller();
    let instruction = build_instruction(
        caller.as_ref(),
        "enter-worktree",
        &p(&created.worktree_root),
        Some(&created.id),
    );
    if let Ok(transition) = build_session_transition(
        "enter-worktree",
        &main_root,
        &created.worktree_root,
        &created.id,
        Some(&feature),
        None,
    ) {
        emit_pi_transition_marker_if_present(&transition);
    }

    let workflows = match list_workflows(&main_root) {
        Ok(w) => w,
        Err(_) => return None,
    };
    let guard = match acquire_state_lock(&main_root) {
        Ok(g) => g,
        Err(_) => return None,
    };
    let legacy = start_lane(
        &main_root,
        &main_root,
        &feature,
        None,
        "exploring",
        None,
        &[],
        &workflows,
    );
    drop(guard);
    let legacy = match legacy {
        Ok(Ok(rec)) => rec,
        Ok(Err(refusal)) => return Some(ctx.fail(&refusal)),
        Err(_) => return None,
    };

    if ensure_workflow_record_for_feature(
        &main_root,
        &feature,
        "exploring",
        None,
        legacy.get("summary"),
        legacy.get("next_action"),
        None,
    )
    .is_err()
    {
        return None;
    }

    let lock_name = lane_lock_name(&feature);
    if let Ok(guard) = acquire_named_lock(&main_root, &lock_name) {
        let _ = rebuild_lane_projection(&main_root, &feature);
        drop(guard);
    }

    let mut lane_map = match read_lane_strict(&main_root, &feature) {
        Ok(Some(m)) => m,
        _ => return None,
    };
    route_object.insert("feature".into(), json!(feature));
    lane_map.insert("route".into(), Value::Object(route_object.clone()));
    if write_lane(&main_root, &lane_map).is_err() {
        return None;
    }

    if let Ok(live) = list_workflows(&main_root) {
        if let Some(wf) = find_live_workflow(&live, &feature) {
            let mut patch = Map::new();
            patch.insert("route".into(), Value::Object(route_object.clone()));
            let _ = update_workflow_assuming_lock(&main_root, &wf_id(wf), patch);
        }
    }

    if use_json {
        println!(
            "{}",
            jsjson::stringify(&json!({
                "worktree": p(&created.worktree_root),
                "instruction": instruction,
            }))
        );
    } else {
        println!("{}", p(&created.worktree_root));
        println!("{instruction}");
    }
    record_timing(&main_root, "feature start", t0, true);
    Some(ExitCode::SUCCESS)
}

fn run_finish(tokens: &[OsString], t0: Instant) -> Option<ExitCode> {
    let strs: Vec<&str> = tokens.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    let (flags, use_json) = parse_flags(&strs)?;
    if !keys_known(&flags, &["feature", "areas"]) {
        return None;
    }

    let feature = match flags.req_str("feature") {
        Some(f) => f.to_string(),
        None => {
            let msg = "bee feature finish: missing required flag: --feature";
            if use_json {
                println!("{}", jsjson::stringify(&json!({"error": msg})));
            } else {
                eprintln!("{msg}");
            }
            return Some(ExitCode::FAILURE);
        }
    };
    let areas = match flags.req_str("areas") {
        Some(a) => a.to_string(),
        None => {
            let msg = "bee feature finish: missing required flag: --areas";
            if use_json {
                println!("{}", jsjson::stringify(&json!({"error": msg})));
            } else {
                eprintln!("{msg}");
            }
            return Some(ExitCode::FAILURE);
        }
    };

    let ctx = match prelude("feature finish", use_json, t0)? {
        Pre::Go(c) => *c,
        Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&format!(
            "\"bee feature finish\" must be run from inside the main checkout, not a \"{}\" checkout — nothing was merged; run it from the main repo root.",
            ctx.kind
        )));
    }
    let main_root = ctx.work_root.clone();

    let worktree_id = find_granted_worktree_for_feature(&main_root, &feature);
    let id = match worktree_id {
        Some((id, _)) => id,
        None => {
            let repo_basename = main_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repo");
            format!("{repo_basename}--wt--{feature}")
        }
    };

    let mut merge_argv = vec![
        OsString::from("worktree"),
        OsString::from("merge"),
        OsString::from("--id"),
        OsString::from(id),
    ];
    if use_json {
        merge_argv.push(OsString::from("--json"));
    }
    let code = match crate::verbs::worktree::try_native(&merge_argv, Instant::now()) {
        Some(c) => c,
        None => ExitCode::FAILURE,
    };
    if code != ExitCode::SUCCESS {
        record_timing(&main_root, "feature finish", t0, false);
        return Some(code);
    }

    let lane_path = main_root.join(".bee").join("lanes").join(format!("{feature}.json"));
    if lane_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&lane_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                if let Some(old_phase) = val.get("phase").and_then(|p| p.as_str()) {
                    if old_phase == "exploring" || old_phase == "planning" {
                        let mut set_argv = vec![
                            OsString::from("state"),
                            OsString::from("set"),
                            OsString::from("--lane"),
                            OsString::from(&feature),
                            OsString::from("--owner"),
                            OsString::from(old_phase),
                            OsString::from("--phase"),
                            OsString::from("swarming"),
                        ];
                        if use_json {
                            set_argv.push(OsString::from("--json"));
                        }
                        let code = match crate::verbs::state_group::try_native(&set_argv, Instant::now()) {
                            Some(c) => c,
                            None => ExitCode::FAILURE,
                        };
                        if code != ExitCode::SUCCESS {
                            record_timing(&main_root, "feature finish", t0, false);
                            return Some(code);
                        }
                        println!("feature finish: lane {feature} phase {old_phase} -> swarming (merge proved the cells)");
                    }
                }
            }
        }
    }

    let mut scribing_argv = vec![
        OsString::from("state"),
        OsString::from("scribing-run"),
        OsString::from("--feature"),
        OsString::from(&feature),
        OsString::from("--areas"),
        OsString::from(&areas),
        OsString::from("--next-action"),
        OsString::from("close"),
    ];
    if main_root.join(".bee").join("lanes").join(format!("{feature}.json")).exists() {
        scribing_argv.push(OsString::from("--lane"));
        scribing_argv.push(OsString::from(&feature));
    }
    if use_json {
        scribing_argv.push(OsString::from("--json"));
    }
    let code = match crate::verbs::state_group::try_native(&scribing_argv, Instant::now()) {
        Some(c) => c,
        None => ExitCode::FAILURE,
    };
    if code != ExitCode::SUCCESS {
        record_timing(&main_root, "feature finish", t0, false);
        return Some(code);
    }

    let mut close_argv = vec![
        OsString::from("close"),
        OsString::from("--feature"),
        OsString::from(&feature),
    ];
    if use_json {
        close_argv.push(OsString::from("--json"));
    }
    let code = match crate::verbs::drivers::try_native(&close_argv, Instant::now()) {
        Some(c) => c,
        None => ExitCode::FAILURE,
    };
    let ok = code == ExitCode::SUCCESS;
    record_timing(&main_root, "feature finish", t0, ok);
    Some(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    struct CwdGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        orig: PathBuf,
    }

    impl CwdGuard {
        fn enter(target: &Path) -> Self {
            let guard = crate::verbs::drivers::TEST_CWD_LOCK
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let orig = std::env::current_dir().unwrap();
            std::env::set_current_dir(target).unwrap();
            Self { _lock: guard, orig }
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.orig);
        }
    }

    fn init_repo(tmp: &Path) -> PathBuf {
        let repo = tmp.join("testrepo");
        std::fs::create_dir_all(repo.join(".bee").join("lanes")).unwrap();
        std::fs::create_dir_all(repo.join(".bee").join("workflows")).unwrap();
        std::fs::create_dir_all(repo.join(".bee").join("cells")).unwrap();
        std::fs::create_dir_all(repo.join(".bee").join("logs")).unwrap();
        std::fs::write(
            repo.join(".bee").join("onboarding.json"),
            r#"{"version":1,"completed":true}"#,
        )
        .unwrap();
        std::fs::write(
            repo.join(".bee").join("config.json"),
            r#"{"commands":{"test":"none"}}"#,
        )
        .unwrap();
        std::fs::write(repo.join(".bee").join("worktrees.json"), "{}\n").unwrap();
        std::fs::write(
            repo.join(".bee").join("state.json"),
            r#"{"schema_version":"1.0","phase":"idle","feature":null,"approved_gates":{}}"#,
        )
        .unwrap();
        std::fs::write(repo.join(".gitignore"), ".bee/*\n").unwrap();
        std::fs::write(repo.join("file.txt"), "hello\n").unwrap();

        std::process::Command::new("git")
            .args(["init", "-q", "-b", "main", "."])
            .current_dir(&repo)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.local"])
            .current_dir(&repo)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "test"])
            .current_dir(&repo)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&repo)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-qm", "init"])
            .current_dir(&repo)
            .status()
            .unwrap();

        repo
    }

    #[test]
    fn feature_start_leaves_worktree_lane_record_and_route() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);

        let argv = vec![
            OsString::from("feature"),
            OsString::from("start"),
            OsString::from("--feature"),
            OsString::from("my-feat"),
            OsString::from("--class"),
            OsString::from("feature"),
            OsString::from("--lane"),
            OsString::from("standard"),
            OsString::from("--files"),
            OsString::from("3"),
        ];

        let code = try_native(&argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let wt_path = tmp.path().join("testrepo--wt--my-feat");
        assert!(wt_path.exists());
        assert!(wt_path.join(".bee").join("state.json").exists());

        let lane_path = repo.join(".bee").join("lanes").join("my-feat.json");
        assert!(lane_path.exists());
        let lane_content: Value =
            serde_json::from_str(&std::fs::read_to_string(&lane_path).unwrap()).unwrap();
        assert_eq!(lane_content["feature"], "my-feat");
        assert_eq!(lane_content["mode"], Value::Null);
        assert_eq!(lane_content["phase"], "exploring");

        let route = &lane_content["route"];
        assert_eq!(route["class"], "feature");
        assert_eq!(route["lane"], "standard");
        assert_eq!(route["product_files"], 3.0);
        assert_eq!(route["flags"], json!([]));
    }

    #[test]
    fn feature_finish_refuses_when_no_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);

        let argv = vec![
            OsString::from("feature"),
            OsString::from("finish"),
            OsString::from("--feature"),
            OsString::from("unknown-feat"),
            OsString::from("--areas"),
            OsString::from("auth"),
        ];

        let code = try_native(&argv, Instant::now());
        assert_ne!(code, Some(ExitCode::SUCCESS));
    }

    #[test]
    fn feature_finish_runs_merge_scribing_stamp_and_close() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);

        let start_argv = vec![
            OsString::from("feature"),
            OsString::from("start"),
            OsString::from("--feature"),
            OsString::from("feat-small"),
            OsString::from("--class"),
            OsString::from("feature"),
            OsString::from("--lane"),
            OsString::from("small"),
            OsString::from("--files"),
            OsString::from("1"),
        ];
        let code = try_native(&start_argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let lane_path = repo.join(".bee").join("lanes").join("feat-small.json");
        let mut lane_content: Value =
            serde_json::from_str(&std::fs::read_to_string(&lane_path).unwrap()).unwrap();
        lane_content["phase"] = json!("swarming");
        std::fs::write(&lane_path, serde_json::to_string_pretty(&lane_content).unwrap()).unwrap();

        let finish_argv = vec![
            OsString::from("feature"),
            OsString::from("finish"),
            OsString::from("--feature"),
            OsString::from("feat-small"),
            OsString::from("--areas"),
            OsString::from("auth"),
        ];
        let code = try_native(&finish_argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let ledger = repo.join(".bee").join("logs").join("scribing-runs.jsonl");
        assert!(ledger.exists());
        let ledger_content = std::fs::read_to_string(&ledger).unwrap();
        assert!(ledger_content.contains("feat-small"));
    }

    #[test]
    fn feature_finish_moves_exploring_lane_to_swarming_after_merge() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);

        let start_argv = vec![
            OsString::from("feature"),
            OsString::from("start"),
            OsString::from("--feature"),
            OsString::from("feat-exp"),
            OsString::from("--class"),
            OsString::from("feature"),
            OsString::from("--lane"),
            OsString::from("small"),
            OsString::from("--files"),
            OsString::from("1"),
        ];
        let code = try_native(&start_argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let lane_path = repo.join(".bee").join("lanes").join("feat-exp.json");
        let lane_before: Value =
            serde_json::from_str(&std::fs::read_to_string(&lane_path).unwrap()).unwrap();
        assert_eq!(lane_before["phase"], "exploring");

        let finish_argv = vec![
            OsString::from("feature"),
            OsString::from("finish"),
            OsString::from("--feature"),
            OsString::from("feat-exp"),
            OsString::from("--areas"),
            OsString::from("auth"),
        ];
        let code = try_native(&finish_argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let ledger = repo.join(".bee").join("logs").join("scribing-runs.jsonl");
        assert!(ledger.exists());
        let ledger_content = std::fs::read_to_string(&ledger).unwrap();
        assert!(ledger_content.contains("feat-exp"));
    }

    #[test]
    fn feature_finish_moves_planning_lane_to_swarming_after_merge() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);

        let start_argv = vec![
            OsString::from("feature"),
            OsString::from("start"),
            OsString::from("--feature"),
            OsString::from("feat-plan"),
            OsString::from("--class"),
            OsString::from("feature"),
            OsString::from("--lane"),
            OsString::from("small"),
            OsString::from("--files"),
            OsString::from("1"),
        ];
        let code = try_native(&start_argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let lane_path = repo.join(".bee").join("lanes").join("feat-plan.json");
        let mut lane_content: Value =
            serde_json::from_str(&std::fs::read_to_string(&lane_path).unwrap()).unwrap();
        lane_content["phase"] = json!("planning");
        std::fs::write(&lane_path, serde_json::to_string_pretty(&lane_content).unwrap()).unwrap();

        let finish_argv = vec![
            OsString::from("feature"),
            OsString::from("finish"),
            OsString::from("--feature"),
            OsString::from("feat-plan"),
            OsString::from("--areas"),
            OsString::from("auth"),
        ];
        let code = try_native(&finish_argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let ledger = repo.join(".bee").join("logs").join("scribing-runs.jsonl");
        assert!(ledger.exists());
        let ledger_content = std::fs::read_to_string(&ledger).unwrap();
        assert!(ledger_content.contains("feat-plan"));
    }

    #[test]
    fn feature_finish_from_worktree_refuses_before_merging() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);

        let start_argv = vec![
            OsString::from("feature"),
            OsString::from("start"),
            OsString::from("--feature"),
            OsString::from("feat-wt"),
            OsString::from("--class"),
            OsString::from("feature"),
            OsString::from("--lane"),
            OsString::from("small"),
            OsString::from("--files"),
            OsString::from("1"),
        ];
        let code = try_native(&start_argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let lane_path = repo.join(".bee").join("lanes").join("feat-wt.json");
        let mut lane_content: Value =
            serde_json::from_str(&std::fs::read_to_string(&lane_path).unwrap()).unwrap();
        lane_content["phase"] = json!("swarming");
        std::fs::write(&lane_path, serde_json::to_string_pretty(&lane_content).unwrap()).unwrap();

        let wt_path = tmp.path().join("testrepo--wt--feat-wt");
        std::env::set_current_dir(&wt_path).unwrap();

        let finish_argv = vec![
            OsString::from("feature"),
            OsString::from("finish"),
            OsString::from("--feature"),
            OsString::from("feat-wt"),
            OsString::from("--areas"),
            OsString::from("auth"),
        ];
        let code = try_native(&finish_argv, Instant::now());
        assert_ne!(code, Some(ExitCode::SUCCESS));

        let ledger = repo.join(".bee").join("logs").join("scribing-runs.jsonl");
        let stamped = std::fs::read_to_string(&ledger).unwrap_or_default();
        assert!(!stamped.contains("feat-wt"));
        let lane_after: Value =
            serde_json::from_str(&std::fs::read_to_string(&lane_path).unwrap()).unwrap();
        assert_eq!(lane_after["phase"], "swarming");
    }

    #[test]
    fn feature_finish_never_approves_uat_door() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);

        let start_argv = vec![
            OsString::from("feature"),
            OsString::from("start"),
            OsString::from("--feature"),
            OsString::from("feat-std"),
            OsString::from("--class"),
            OsString::from("feature"),
            OsString::from("--lane"),
            OsString::from("standard"),
            OsString::from("--files"),
            OsString::from("1"),
        ];
        let code = try_native(&start_argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let finish_argv = vec![
            OsString::from("feature"),
            OsString::from("finish"),
            OsString::from("--feature"),
            OsString::from("feat-std"),
            OsString::from("--areas"),
            OsString::from("auth"),
        ];
        let code = try_native(&finish_argv, Instant::now());
        assert_ne!(code, Some(ExitCode::SUCCESS));

        let lane_path = repo.join(".bee").join("lanes").join("feat-std.json");
        let lane_content: Value =
            serde_json::from_str(&std::fs::read_to_string(&lane_path).unwrap()).unwrap();
        assert_eq!(
            lane_content["approved_gates"]["uat"],
            json!(false)
        );
    }

    #[test]
    fn feature_help_lists_start_and_finish() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);

        let argv = vec![OsString::from("feature"), OsString::from("--help")];
        let code = crate::verbs::help::try_native(&argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));
    }

    #[test]
    fn route_set_no_flags_and_empty_flags_records_zero_flags() {
        let mut f1 = Flags(Vec::new());
        f1.insert("class", crate::verbs::reservations::FlagV::S("feature".into()));
        f1.insert("lane", crate::verbs::reservations::FlagV::S("standard".into()));
        f1.insert("files", crate::verbs::reservations::FlagV::S("5".into()));

        let r1 = match validate_route_set_flags(&f1) {
            Ok(Ok(r)) => r,
            _ => panic!("expected Ok"),
        };
        assert_eq!(r1["flags"], json!([]));

        let mut f2 = Flags(Vec::new());
        f2.insert("class", crate::verbs::reservations::FlagV::S("feature".into()));
        f2.insert("lane", crate::verbs::reservations::FlagV::S("standard".into()));
        f2.insert("files", crate::verbs::reservations::FlagV::S("5".into()));
        f2.insert("flags", crate::verbs::reservations::FlagV::S("".into()));

        let r2 = match validate_route_set_flags(&f2) {
            Ok(Ok(r)) => r,
            _ => panic!("expected Ok"),
        };
        assert_eq!(r2["flags"], json!([]));
    }

    #[test]
    fn small_route_opened_by_feature_start_passes_merged_gate_without_plan() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        let _guard = CwdGuard::enter(&repo);

        let start_argv = vec![
            OsString::from("feature"),
            OsString::from("start"),
            OsString::from("--feature"),
            OsString::from("feat-gate-test"),
            OsString::from("--class"),
            OsString::from("feature"),
            OsString::from("--lane"),
            OsString::from("small"),
            OsString::from("--files"),
            OsString::from("1"),
        ];
        let code = try_native(&start_argv, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let derive_flags = crate::verbs::reservations::parse_flags(&["--lane", "feat-gate-test"]).unwrap().0;
        let code = crate::verbs::state_group::run_plan_conflicts_derive(derive_flags, false, Instant::now());
        assert_eq!(code, Some(ExitCode::SUCCESS));

        let lane_flags = crate::verbs::reservations::parse_flags(&[
            "--lane",
            "feat-gate-test",
            "--merge",
            "--approved",
            "true",
        ])
        .unwrap()
        .0;
        let out = crate::verbs::state_group::run_gate_body(&repo, &lane_flags).unwrap();
        match out {
            crate::verbs::reservations::Out::Emit(_, _, code) => assert_eq!(code, 0),
            crate::verbs::reservations::Out::Thrown(msg) => panic!("merged gate must pass, got: {msg}"),
        }
    }
}
